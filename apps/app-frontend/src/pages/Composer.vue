<template>
  <div class="composer">
    <header class="composer-header">
      <div class="composer-header-left">
        <SparklesIcon class="text-brand" />
        <h1 class="m-0 text-xl font-extrabold text-contrast">AI Composer</h1>
      </div>
    </header>

    <div ref="scrollArea" class="composer-chat">
      <div v-if="messages.length === 0" class="composer-empty">
        <SparklesIcon class="composer-empty-icon" />
        <p class="text-secondary">Ask anything — powered by Gemini</p>
      </div>

      <div
        v-for="(message, index) in messages"
        :key="index"
        class="composer-message"
        :class="`composer-message--${message.role}`"
      >
        <div class="composer-message-label">
          {{ message.role === 'assistant' ? selectedModel?.displayName ?? 'Gemini' : 'You' }}
        </div>
        <div class="markdown-body" v-html="renderMessage(message.content)" />
      </div>

      <div v-if="error" class="composer-error">
        {{ error }}
      </div>
    </div>

    <footer class="composer-footer">
      <div class="composer-toolbar">
        <ModelSelect v-model="selectedModel" load-on-mount />
        <span class="composer-model-info">
          up to {{ formatTokenLimit(selectedModel?.inputTokenLimit) }} tokens context
        </span>
      </div>
      <textarea
        ref="promptInput"
        v-model="prompt"
        class="composer-input"
        rows="3"
        placeholder="Write a message to Gemini..."
        :disabled="streaming"
        @keydown.enter.exact.prevent="send"
      />
      <div class="composer-actions">
        <span class="composer-model-info">
          {{ selectedModel?.displayName ?? 'No model' }}
        </span>
        <div class="composer-buttons">
          <Button v-if="messages.length > 0 && !streaming" class="r-btn" @click="clearChat">
            <TrashIcon />
          </Button>
          <Button v-if="streaming" class="r-btn" color="danger" @click="stopStream">
            <StopCircleIcon />
          </Button>
          <Button
            class="r-btn"
            color="primary"
            :disabled="streaming || !prompt.trim() || !selectedModel"
            @click="send"
          >
            <SendIcon />
            Send
          </Button>
        </div>
      </div>
    </footer>
  </div>
</template>

<script setup>
import { nextTick, onMounted, ref } from 'vue'
import { SparklesIcon, SendIcon, StopCircleIcon, TrashIcon } from '@modrinth/assets'
import { Button } from '@modrinth/ui'
import { renderHighlightedString } from '@modrinth/utils'
import ModelSelect from '@/components/ui/ModelSelect.vue'
import { streamGenerate, formatTokenLimit } from '@/helpers/gemini.js'

const selectedModel = ref(null)
const messages = ref([])
const prompt = ref('')
const streaming = ref(false)
const error = ref('')
const scrollArea = ref(null)
const promptInput = ref(null)

let abortController = null

onMounted(() => {
  promptInput.value?.focus()
})

const renderMessage = (content) => renderHighlightedString(content ?? '')

const scrollToBottom = async () => {
  await nextTick()
  scrollArea.value?.scrollTo({ top: scrollArea.value.scrollHeight, behavior: 'smooth' })
}

const send = async () => {
  const content = prompt.value.trim()
  if (!content || streaming.value || !selectedModel.value) return

  messages.value.push({ role: 'user', content })
  prompt.value = ''
  const assistantIndex = messages.value.length
  messages.value.push({ role: 'assistant', content: '' })
  streaming.value = true
  error.value = ''
  await scrollToBottom()

  abortController = new AbortController()

  try {
    await streamGenerate(
      selectedModel.value.name,
      messages.value.slice(0, assistantIndex),
      (delta) => {
        messages.value[assistantIndex].content += delta
        scrollToBottom()
      },
      abortController.signal,
    )
  } catch (err) {
    if (err.name !== 'AbortError') {
      error.value = `Failed to generate response: ${err.message}`
      messages.value[assistantIndex].content =
        messages.value[assistantIndex].content || ''
    }
  } finally {
    streaming.value = false
    abortController = null
  }
}

const stopStream = () => {
  abortController?.abort()
}

const clearChat = () => {
  messages.value = []
  error.value = ''
}
</script>

<style scoped lang="scss">
.composer {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 1.5rem;
  gap: 1rem;
}

.composer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  flex-wrap: wrap;
}

.composer-header-left {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.composer-chat {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 0.25rem;
}

.composer-empty {
  margin: auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;

  .composer-empty-icon {
    width: 3rem;
    height: 3rem;
    color: var(--color-brand);
    opacity: 0.6;
  }
}

.composer-message {
  max-width: 85%;
  padding: 0.9rem 1.1rem;
  border-radius: var(--radius-lg);

  &--user {
    align-self: flex-end;
    background-color: var(--color-brand-highlight);
    border: 1px solid var(--color-brand-shadow);
  }

  &--assistant {
    align-self: flex-start;
    background-color: var(--color-raised-bg);
    border: 1px solid var(--color-button-border);
  }
}

.composer-message-label {
  font-size: 0.75rem;
  font-weight: 700;
  color: var(--color-secondary);
  margin-bottom: 0.4rem;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.composer-error {
  align-self: center;
  color: var(--color-red);
  font-size: 0.85rem;
  background-color: var(--color-red-highlight);
  padding: 0.6rem 1rem;
  border-radius: var(--radius-md);
}

.composer-footer {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  background-color: var(--color-raised-bg);
  border: 1px solid var(--color-button-border);
  border-radius: var(--radius-lg);
  padding: 0.8rem;
}

.composer-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.composer-input {
  width: 100%;
  resize: vertical;
  min-height: 3rem;
  max-height: 12rem;
  background: transparent;
  border: none;
  outline: none;
  color: var(--color-contrast);
  font-family: var(--font-standard);
  font-size: 0.95rem;
  line-height: 1.5;

  &::placeholder {
    color: var(--color-secondary);
  }

  &:disabled {
    opacity: 0.6;
  }
}

.composer-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.composer-model-info {
  font-size: 0.78rem;
  color: var(--color-secondary);
}

.composer-buttons {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
</style>
