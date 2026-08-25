import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
  listSkins,
  addSkin,
  deleteSkin,
  updateSkin,
  duplicateSkin,
  reorderSkins,
  markSkinUsed,
  validateSkin,
  fileToBytes,
} from '@/helpers/skins'
import { handleError } from '@/store/notifications.js'

export const useSkinsStore = defineStore('skins', () => {
  const skins = ref([])
  const loading = ref(false)
  const searchQuery = ref('')
  const filterMode = ref('all') // all | favorites | recent | classic | slim | local | imported | account
  const sortBy = ref('recently_added') // recently_added | recently_used | name | custom

  const filteredSkins = computed(() => {
    let result = [...skins.value]

    // Search
    if (searchQuery.value.trim()) {
      const q = searchQuery.value.toLowerCase()
      result = result.filter(
        (s) =>
          s.name.toLowerCase().includes(q) ||
          s.fileName.toLowerCase().includes(q) ||
          (s.accountId && s.accountId.toLowerCase().includes(q)),
      )
    }

    // Filter
    if (filterMode.value === 'favorites') {
      result = result.filter((s) => s.favorite)
    } else if (filterMode.value === 'recently_used') {
      result = result.filter((s) => s.lastUsed)
    } else if (filterMode.value === 'classic') {
      result = result.filter((s) => s.modelType === 'classic')
    } else if (filterMode.value === 'slim') {
      result = result.filter((s) => s.modelType === 'slim')
    } else if (filterMode.value === 'local') {
      result = result.filter((s) => s.source === 'local')
    } else if (filterMode.value === 'imported') {
      result = result.filter((s) => s.source === 'imported')
    } else if (filterMode.value === 'account') {
      result = result.filter((s) => s.source === 'account')
    }

    // Sort
    if (sortBy.value === 'recently_used') {
      result.sort((a, b) => (b.lastUsed ? new Date(b.lastUsed).getTime() : 0) - (a.lastUsed ? new Date(a.lastUsed).getTime() : 0))
    } else if (sortBy.value === 'name') {
      result.sort((a, b) => a.name.localeCompare(b.name))
    } else if (sortBy.value === 'custom') {
      result.sort((a, b) => a.sortOrder - b.sortOrder)
    }
    // recently_added default (by created date desc)
    else if (sortBy.value === 'recently_added') {
      result.sort((a, b) => new Date(b.created).getTime() - new Date(a.created).getTime())
    }

    return result
  })

  const favoriteCount = computed(() => skins.value.filter((s) => s.favorite).length)

  async function refresh() {
    loading.value = true
    try {
      skins.value = await listSkins()
    } catch (err) {
      handleError({ message: `Failed to load skins: ${err.message}` })
    } finally {
      loading.value = false
    }
  }

  async function importSkin(file) {
    const bytes = await fileToBytes(file)
    const record = await validateSkin({ name: file.name.replace(/\.png$/i, ''), bytes })
    await refresh()
    return record
  }

  async function createSkin(name, bytes, modelType = 'classic') {
    const record = await addSkin({ name, bytes, modelType, source: 'local' })
    await refresh()
    return record
  }

  async function remove(id) {
    await deleteSkin(id)
    await refresh()
  }

  async function update(id, changes) {
    const result = await updateSkin({ id, ...changes })
    await refresh()
    return result
  }

  async function duplicate(id) {
    await duplicateSkin(id)
    await refresh()
  }

  async function setFavorite(id, value) {
    await updateSkin({ id, favorite: value })
    await refresh()
  }

  async function markUsed(id) {
    await markSkinUsed(id)
  }

  async function reorder(reorderedIds) {
    await reorderSkins(reorderedIds)
    await refresh()
  }

  return {
    skins,
    loading,
    searchQuery,
    filterMode,
    sortBy,
    filteredSkins,
    favoriteCount,
    refresh,
    importSkin,
    createSkin,
    remove,
    update,
    duplicate,
    setFavorite,
    markUsed,
    reorder,
  }
})
