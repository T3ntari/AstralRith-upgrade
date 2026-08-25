import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
  getProfiles,
  getAllProcesses,
  getProcessesByProfile,
  killProcess,
  launchProfile,
} from '@/helpers/clients'
import { handleError } from '@/store/notifications.js'

export const useClientsStore = defineStore('clients', () => {
  const profiles = ref([])
  const processes = ref([])
  const loading = ref(false)
  const searchQuery = ref('')

  const filteredProfiles = computed(() => {
    if (!searchQuery.value.trim()) return profiles.value
    const q = searchQuery.value.toLowerCase()
    return profiles.value.filter(
      (p) =>
        (p.name && p.name.toLowerCase().includes(q)) ||
        (p.game_version && p.game_version.toLowerCase().includes(q)),
    )
  })

  const runningProfiles = computed(() => {
    const runningPaths = new Set(processes.value.map((p) => p.profile_path))
    return profiles.value.filter((p) => runningPaths.has(p.path))
  })

  const stoppedProfiles = computed(() => {
    const runningPaths = new Set(processes.value.map((p) => p.profile_path))
    return profiles.value.filter((p) => !runningPaths.has(p.path))
  })

  async function refresh() {
    loading.value = true
    try {
      const [profs, procs] = await Promise.all([getProfiles(), getAllProcesses()])
      profiles.value = profs || []
      processes.value = procs || []
    } catch (err) {
      handleError({ message: `Failed to load instances: ${err.message}` })
    } finally {
      loading.value = false
    }
  }

  async function launch(path) {
    try {
      await launchProfile(path)
      await refresh()
    } catch (err) {
      handleError({ message: `Failed to launch: ${err.message}` })
    }
  }

  async function stop(uuid) {
    try {
      await killProcess(uuid)
      await refresh()
    } catch (err) {
      handleError({ message: `Failed to stop: ${err.message}` })
    }
  }

  function getProcessesForProfile(profilePath) {
    return processes.value.filter((p) => p.profile_path === profilePath)
  }

  return {
    profiles,
    processes,
    loading,
    searchQuery,
    filteredProfiles,
    runningProfiles,
    stoppedProfiles,
    refresh,
    launch,
    stop,
    getProcessesForProfile,
  }
})
