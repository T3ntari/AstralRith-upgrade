/**
 * Lightweight pre-cache for Discover / scroll-heavy pages.
 *
 * - Caches search results + project metadata in localStorage.
 * - Total budget ~10 MB, chunked per key (a key = one search page or one project).
 * - Oldest entries are evicted first when the budget is exceeded.
 * - Only caches things marked `cacheable` (popular mods, discover feeds), never
 *   user-specific data.
 */
import { ref } from 'vue'

const STORAGE_KEY = 'ar_precache_v1'
const MAX_BYTES = 10 * 1024 * 1024 // 10 MB budget
const MAX_CHUNK_BYTES = 512 * 1024 // per-key chunk cap (~512 KB)
const TTL_MS = 1000 * 60 * 60 * 6 // 6h default TTL
const TTL_SEARCH_MS = 1000 * 60 * 15 // 15 min for search feeds (fresher)

const cacheEnabled = ref(true)
const cacheStats = ref({ entries: 0, bytes: 0 })

function readStore() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return {}
    return JSON.parse(raw)
  } catch (e) {
    console.warn('[Precache] store read failed, resetting', e)
    try {
      localStorage.removeItem(STORAGE_KEY)
    } catch (e2) {
      /* ignore */
    }
    return {}
  }
}

function writeStore(store) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(store))
  } catch (e) {
    // Quota exceeded — drop oldest entries and retry once.
    const entries = Object.entries(store).sort((a, b) => a[1].ts - b[1].ts)
    let removed = 0
    for (const [k] of entries) {
      if (removed > 20) break
      delete store[k]
      removed++
    }
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(store))
    } catch (e2) {
      console.warn('[Precache] store write failed', e2)
    }
  }
}

function computeBytes(store) {
  try {
    return new Blob([JSON.stringify(store)]).size
  } catch (e) {
    return 0
  }
}

function evict(store, needed) {
  let bytes = computeBytes(store)
  const entries = Object.entries(store).sort((a, b) => a[1].ts - b[1].ts)
  for (const [k, v] of entries) {
    if (bytes + needed <= MAX_BYTES) break
    delete store[k]
    bytes -= v.size ?? 0
  }
}

/**
 * Store a value under key.
 * @param {string} key
 * @param {*} value
 * @param {object} opts { ttlMs, chunkable }
 */
export function cacheSet(key, value, opts = {}) {
  if (!cacheEnabled.value) return
  try {
    const store = readStore()
    const json = JSON.stringify(value)
    let size = 0
    try {
      size = new Blob([json]).size
    } catch (e) {
      size = json.length
    }
    if (size > MAX_CHUNK_BYTES) {
      console.warn(`[Precache] chunk too large (${size} B) for ${key}, skipping`)
      return
    }
    const ttl = opts.ttlMs ?? (key.startsWith('search:') ? TTL_SEARCH_MS : TTL_MS)
    store[key] = { v: value, ts: Date.now(), ttl, size }
    evict(store, size)
    writeStore(store)
    cacheStats.value.entries = Object.keys(store).length
    cacheStats.value.bytes = computeBytes(store)
  } catch (e) {
    console.warn('[Precache] set failed', e)
  }
}

/**
 * Fetch a cached value, or null when missing/expired.
 */
export function cacheGet(key) {
  try {
    const store = readStore()
    const entry = store[key]
    if (!entry) return null
    if (Date.now() - entry.ts > (entry.ttl ?? TTL_MS)) {
      delete store[key]
      writeStore(store)
      return null
    }
    return entry.v
  } catch (e) {
    return null
  }
}

/**
 * Remove a key (or all keys with the prefix).
 */
export function cacheInvalidate(prefix) {
  try {
    const store = readStore()
    for (const k of Object.keys(store)) {
      if (k.startsWith(prefix)) delete store[k]
    }
    writeStore(store)
  } catch (e) {
    /* ignore */
  }
}

/** @returns {boolean} whether the search feed is fresh enough to skip a refetch */
export function isSearchFresh(key) {
  try {
    const store = readStore()
    const entry = store[key]
    if (!entry) return false
    return Date.now() - entry.ts < TTL_SEARCH_MS
  } catch (e) {
    return false
  }
}

export function setCacheEnabled(v) {
  cacheEnabled.value = v
}

export { cacheEnabled, cacheStats }
