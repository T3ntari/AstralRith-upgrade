<script setup>
import { ref, watch } from 'vue'
import { SearchIcon, XIcon } from '@modrinth/assets'
import { useModulesStore } from '@/store/modules.js'

const store = useModulesStore()
const inputRef = ref(null)
const localQuery = ref('')

watch(localQuery, (val) => {
  store.searchQuery = val
})

function clear() {
  localQuery.value = ''
  store.searchQuery = ''
}

function handleKeydown(e) {
  if (e.key === 'Escape') {
    clear()
    inputRef.value?.blur()
  }
}

// Keyboard shortcut: Ctrl+K or / to focus search
function handleGlobalKeydown(e) {
  if ((e.ctrlKey || e.metaKey && e.key === 'k') || (e.key === '/' && !e.ctrlKey && !e.metaKey)) {
    if (document.activeElement?.tagName === 'INPUT' || document.activeElement?.tagName === 'TEXTAREA') return
    e.preventDefault()
    inputRef.value?.focus()
  }
}

if (typeof window !== 'undefined') {
  window.addEventListener('keydown', handleGlobalKeydown)
}
</script>

<template>
  <div class="relative">
    <SearchIcon class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-secondary" />
    <input
      ref="inputRef"
      v-model="localQuery"
      type="text"
      class="w-full pl-9 pr-20 py-2 bg-bg-base border border-secondary rounded-lg text-sm text-contrast placeholder:text-secondary/50 focus:outline-none focus:border-brand/50 transition-colors"
      placeholder="Search modules... (Ctrl+K)"
      @keydown="handleKeydown"
    />
    <div class="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-2">
      <span v-if="localQuery" class="text-xs text-secondary">
        {{ store.filteredModules.length }} result(s)
      </span>
      <button
        v-if="localQuery"
        class="text-secondary hover:text-contrast transition-colors"
        @click="clear"
      >
        <XIcon class="w-3.5 h-3.5" />
      </button>
    </div>
  </div>
</template>
