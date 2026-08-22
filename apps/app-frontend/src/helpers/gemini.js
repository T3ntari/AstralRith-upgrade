import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const GEMINI_API_KEY = 'ar.gemini_api_key'
const DEFAULT_GEMINI_API_KEY = 'AIzaSyDaxnM03umXzRJdcavXFcrvApAYrUb1Kpw'

export function getGeminiApiKey() {
  return localStorage.getItem(GEMINI_API_KEY) || DEFAULT_GEMINI_API_KEY
}

export function setGeminiApiKey(key) {
  localStorage.setItem(GEMINI_API_KEY, key.trim())
}

export const RECOMMENDED_MODELS = new Set([
  'models/gemini-2.5-pro',
  'models/gemini-2.5-flash',
  'models/gemini-2.5-flash-lite',
  'models/gemini-2.5-flash-8b',
  'models/gemini-2.0-flash',
])

export function isRecommended(model) {
  return RECOMMENDED_MODELS.has(model.name)
}

export function formatTokenLimit(limit) {
  if (!limit) return 'N/A'
  if (limit >= 1_000_000) return `${(limit / 1_000_000).toFixed(1)}M`
  if (limit >= 1_000) return `${(limit / 1_000).toFixed(1)}K`
  return String(limit)
}

export async function fetchModels() {
  const apiKey = getGeminiApiKey()
  const models = await invoke('plugin:gemini|gemini_list_models', { apiKey })
  if (!Array.isArray(models)) return []
  return models.map((model) => ({
    name: model.name,
    displayName: model.displayName,
    description: model.description ?? '',
    inputTokenLimit: model.inputTokenLimit ?? 0,
    outputTokenLimit: model.outputTokenLimit ?? 0,
    temperature: model.temperature ?? undefined,
    topP: model.topP ?? undefined,
    topK: model.topK ?? undefined,
    version: model.version ?? undefined,
  }))
}

/**
 * Streams a generation through the Rust backend.
 * The backend emits `gemini-delta` events ({ requestId, text } chunks,
 * { requestId, done: true } on completion, { requestId, error } on failure).
 * Calls onDelta(text) for each piece of text as it arrives.
 * Pass an AbortController signal to cancel.
 */
export async function streamGenerate(modelName, messages, onDelta, signal) {
  const apiKey = getGeminiApiKey()
  const requestId = crypto.randomUUID()

  const contents = messages
    .filter((message) => message.content && message.content.trim())
    .map((message) => ({
      role: message.role === 'assistant' ? 'model' : 'user',
      parts: [{ text: message.content }],
    }))

  if (contents.length === 0) {
    throw new Error('No messages to send')
  }

  if (signal?.aborted) {
    throw new Error('Request cancelled')
  }

  return new Promise((resolve, reject) => {
    let settled = false
    let unlisten = null

    const onAbort = () => {
      if (settled) return
      settled = true
      unlisten?.()
      signal?.removeEventListener('abort', onAbort)
      invoke('plugin:gemini|gemini_generate_cancel', { requestId }).catch(() => {})
      reject(new DOMException('Aborted', 'AbortError'))
    }

    signal?.addEventListener('abort', onAbort)

    listen('gemini-delta', (event) => {
      const payload = event.payload
      if (!payload || payload.requestId !== requestId || settled) return

      if (payload.text) {
        onDelta(payload.text)
      } else if (payload.done) {
        settled = true
        unlisten?.()
        signal?.removeEventListener('abort', onAbort)
        resolve()
      } else if (payload.error) {
        settled = true
        unlisten?.()
        signal?.removeEventListener('abort', onAbort)
        reject(new Error(payload.error))
      }
    }).then((unlistenFn) => {
      if (settled) {
        unlistenFn()
        return
      }
      unlisten = unlistenFn
      invoke('plugin:gemini|gemini_generate_start', {
        apiKey,
        modelName,
        contents,
        requestId,
      }).catch((err) => {
        if (settled) return
        settled = true
        unlistenFn()
        signal?.removeEventListener('abort', onAbort)
        reject(new Error(err?.message ?? String(err)))
      })
    })
  })
}
