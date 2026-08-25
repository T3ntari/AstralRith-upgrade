import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { handleError } from '@/store/notifications.js'

export const useBotsStore = defineStore('bots', () => {
  const bots = ref([])
  const loading = ref(false)
  const searchQuery = ref('')

  const filteredBots = computed(() => {
    if (!searchQuery.value.trim()) return bots.value
    const q = searchQuery.value.toLowerCase()
    return bots.value.filter(
      (b) =>
        b.name.toLowerCase().includes(q) ||
        b.host.toLowerCase().includes(q) ||
        (b.username && b.username.toLowerCase().includes(q)),
    )
  })

  const connectedBots = computed(() => bots.value.filter((b) => b.status === 'connected'))
  const disconnectedBots = computed(() => bots.value.filter((b) => b.status !== 'connected'))

  async function refresh() {
    loading.value = true
    try {
      bots.value = (await invoke('plugin:bots|bot_list').catch(() => [])) || []
    } catch (err) {
      handleError({ message: `Failed to load bots: ${err.message}` })
    } finally {
      loading.value = false
    }
  }

  async function createBot(profile) {
    try {
      const bot = await invoke('plugin:bots|bot_create', {
        name: profile.name || 'New Bot',
        host: profile.host || 'localhost',
        port: profile.port || 25565,
        version: profile.version || '1.21.4',
        authType: profile.authType || 'offline',
        username: profile.username || 'Bot',
        script: profile.script || null,
        plugins: profile.plugins ? JSON.stringify(profile.plugins) : null,
      })
      await refresh()
      return bot
    } catch (err) {
      handleError({ message: `Failed to create bot: ${err.message}` })
    }
  }

  async function updateBot(id, changes) {
    try {
      await invoke('plugin:bots|bot_update', {
        id,
        name: changes.name || null,
        host: changes.host || null,
        port: changes.port || null,
        version: changes.version || null,
        authType: changes.authType || null,
        username: changes.username || null,
        script: changes.script || null,
        plugins: changes.plugins ? JSON.stringify(changes.plugins) : null,
      })
      await refresh()
      return bots.value.find((b) => b.id === id)
    } catch (err) {
      handleError({ message: `Failed to update bot: ${err.message}` })
    }
  }

  async function deleteBot(id) {
    try {
      await invoke('plugin:bots|bot_delete', { id })
      await refresh()
    } catch (err) {
      handleError({ message: `Failed to delete bot: ${err.message}` })
    }
  }

  async function duplicateBot(id) {
    try {
      const newBot = await invoke('plugin:bots|bot_duplicate', { id })
      await refresh()
      return newBot
    } catch (err) {
      handleError({ message: `Failed to duplicate bot: ${err.message}` })
    }
  }

  return {
    bots,
    loading,
    searchQuery,
    filteredBots,
    connectedBots,
    disconnectedBots,
    refresh,
    createBot,
    updateBot,
    deleteBot,
    duplicateBot,
  }
})
