<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import {
  PlayIcon,
  StopCircleIcon,
  SearchIcon,
  PlusIcon,
  ServerIcon,
  RedoIcon,
} from '@modrinth/assets'
import { ButtonStyled } from '@modrinth/ui'
import { useClientsStore } from '@/store/clients'
import { showProfileInFolder } from '@/helpers/utils.js'
import { handleError } from '@/store/notifications.js'
import { removeProfile, duplicateProfile } from '@/helpers/clients.js'
import InstanceCard from '@/components/clients/InstanceCard.vue'
import RunningPanel from '@/components/clients/RunningPanel.vue'
import InstanceDetail from '@/components/clients/InstanceDetail.vue'
import InstanceCreationModal from '@/components/ui/InstanceCreationModal.vue'
import InstanceSettingsModal from '@/components/ui/modal/InstanceSettingsModal.vue'
import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'
import { process_listener } from '@/helpers/events.js'

const clientsStore = useClientsStore()

// Refs
const creationModal = ref(null)
const settingsModal = ref(null)
const deleteModal = ref(null)
const searchInput = ref(null)
const selectedProfilePath = ref(null)

// Load data on mount
onMounted(async () => {
  await clientsStore.refresh()
})

// Keyboard shortcuts
function handleKeydown(e) {
  // F5 to refresh
  if (e.key === 'F5') {
    e.preventDefault()
    clientsStore.refresh()
    return
  }

  // Enter to launch selected
  if (e.key === 'Enter' && selectedProfile.value && !clientsStore.loading) {
    e.preventDefault()
    handleLaunch(selectedProfile.value)
    return
  }

  // Delete to remove selected
  if (e.key === 'Delete' && selectedProfile.value) {
    e.preventDefault()
    confirmDelete(selectedProfile.value)
    return
  }

  // Escape to deselect
  if (e.key === 'Escape') {
    selectedProfilePath.value = null
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})

// Listen for process events to auto-refresh
const unlisten = await process_listener(() => {
  clientsStore.refresh()
})

onUnmounted(() => {
  if (unlisten) unlisten()
})

// Computed
const selectedProfile = computed(() => {
  if (!selectedProfilePath.value) return null
  return clientsStore.profiles.find((p) => p.path === selectedProfilePath.value) || null
})

const selectedProfileRunning = computed(() => {
  if (!selectedProfile.value) return false
  return clientsStore.getProcessesForProfile(selectedProfile.value.path).length > 0
})

const selectedProfileProcesses = computed(() => {
  if (!selectedProfile.value) return []
  return clientsStore.getProcessesForProfile(selectedProfile.value.path)
})

// Instance count for display
const instanceCount = computed(() => clientsStore.profiles.length)
const runningCount = computed(() => clientsStore.runningProfiles.length)

// Actions
function handleSelect(profile) {
  if (selectedProfilePath.value === profile.path) {
    selectedProfilePath.value = null
  } else {
    selectedProfilePath.value = profile.path
  }
}

async function handleLaunch(profile) {
  await clientsStore.launch(profile.path)
}

async function handleStop(profile) {
  const processes = clientsStore.getProcessesForProfile(profile.path)
  for (const proc of processes) {
    await clientsStore.stop(proc.uuid)
  }
}

async function handleStopProcess(process) {
  await clientsStore.stop(process.uuid)
}

async function handleOpenFolder(profile) {
  try {
    await showProfileInFolder(profile.path)
  } catch (err) {
    handleError({ message: `Failed to open folder: ${err.message}` })
  }
}

function handleSettings(profile) {
  selectedProfilePath.value = profile.path
  settingsModal.value?.show()
}

function confirmDelete(profile) {
  deleteModal.value?.show()
}

async function handleDeleteProceed() {
  if (!selectedProfile.value) return
  try {
    await removeProfile(selectedProfile.value.path)
    selectedProfilePath.value = null
    await clientsStore.refresh()
  } catch (err) {
    handleError({ message: `Failed to delete instance: ${err.message}` })
  }
}

async function handleDuplicate(profile) {
  try {
    await duplicateProfile(profile.path)
    await clientsStore.refresh()
  } catch (err) {
    handleError({ message: `Failed to duplicate instance: ${err.message}` })
  }
}

async function handleRename(profile) {
  // Basic rename - could be enhanced with a modal
  const newName = prompt('Enter new name:', profile.name)
  if (newName && newName !== profile.name) {
    try {
      // Use the edit helper
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('plugin:profile|profile_edit', {
        path: profile.path,
        editProfile: { name: newName },
      })
      await clientsStore.refresh()
    } catch (err) {
      handleError({ message: `Failed to rename instance: ${err.message}` })
    }
  }
}
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Top bar -->
    <div class="flex items-center justify-between px-6 py-4 border-b border-divider shrink-0">
      <div class="flex items-center gap-3">
        <h1 class="m-0 text-2xl font-extrabold text-contrast">Clients</h1>
        <span v-if="!clientsStore.loading" class="text-sm text-secondary font-medium">
          {{ instanceCount }} instance{{ instanceCount !== 1 ? 's' : '' }}
          <span v-if="runningCount > 0" class="text-green-500">
            · {{ runningCount }} running
          </span>
        </span>
      </div>
      <div class="flex items-center gap-2">
        <!-- Search -->
        <div class="relative">
          <SearchIcon class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-secondary pointer-events-none" />
          <input
            ref="searchInput"
            v-model="clientsStore.searchQuery"
            type="text"
            placeholder="Search instances..."
            class="bg-button-bg text-contrast text-sm rounded-lg pl-9 pr-3 py-2 w-64 border border-transparent focus:border-brand focus:outline-none transition-colors placeholder:text-secondary"
          />
        </div>

        <!-- Refresh -->
        <ButtonStyled color="standard" circular>
          <button
            v-tooltip="'Refresh (F5)'"
            :disabled="clientsStore.loading"
            @click="clientsStore.refresh()"
          >
            <RedoIcon :class="{ 'animate-spin': clientsStore.loading }" />
          </button>
        </ButtonStyled>

        <!-- Import/Create -->
        <ButtonStyled color="brand">
          <button @click="creationModal?.show()">
            <PlusIcon />
            Import
          </button>
        </ButtonStyled>
      </div>
    </div>

    <!-- Content area -->
    <div class="flex-1 overflow-y-auto px-6 py-4">
      <!-- Loading skeleton -->
      <div v-if="clientsStore.loading && clientsStore.profiles.length === 0" class="space-y-4">
        <div v-for="n in 6" :key="n" class="h-32 bg-bg-raised rounded-xl animate-pulse border border-button-bg"></div>
      </div>

      <!-- Empty state -->
      <div
        v-else-if="!clientsStore.loading && clientsStore.profiles.length === 0"
        class="flex flex-col items-center justify-center py-20 gap-4"
      >
        <div class="w-24 h-24 rounded-full bg-button-bg flex items-center justify-center">
          <ServerIcon class="w-12 h-12 text-secondary opacity-50" />
        </div>
        <h3 class="m-0 text-lg font-bold text-contrast">No instances yet</h3>
        <p class="m-0 text-secondary text-center max-w-sm">
          Import an existing instance or create a new one to get started.
        </p>
        <ButtonStyled color="brand">
          <button @click="creationModal?.show()">
            <PlusIcon />
            Create Instance
          </button>
        </ButtonStyled>
      </div>

      <!-- Main content -->
      <div v-else>
        <!-- Running instances panel -->
        <RunningPanel
          :running-instances="clientsStore.processes"
          :profiles="clientsStore.profiles"
          @stop="handleStopProcess"
          @force-stop="handleStopProcess"
          @open-folder="handleOpenFolder"
        />

        <!-- Two column layout: grid + detail -->
        <div class="flex gap-4">
          <!-- Instance grid -->
          <div class="flex-1 min-w-0">
            <h2 class="text-lg font-bold text-contrast mb-3">
              All Instances
              <span v-if="clientsStore.searchQuery" class="text-sm font-normal text-secondary">
                ({{ clientsStore.filteredProfiles.length }} results)
              </span>
            </h2>

            <!-- No search results -->
            <div
              v-if="clientsStore.filteredProfiles.length === 0 && clientsStore.searchQuery"
              class="flex flex-col items-center justify-center py-12 gap-3"
            >
              <SearchIcon class="w-10 h-10 text-secondary opacity-50" />
              <p class="m-0 text-secondary text-center">
                No instances matching "<span class="text-contrast font-medium">{{ clientsStore.searchQuery }}</span>"
              </p>
            </div>

            <!-- Grid -->
            <div v-else class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3">
              <InstanceCard
                v-for="profile in clientsStore.filteredProfiles"
                :key="profile.path"
                :profile="profile"
                :is-running="clientsStore.getProcessesForProfile(profile.path).length > 0"
                :is-selected="selectedProfilePath === profile.path"
                :loading="clientsStore.loading"
                @select="handleSelect"
                @launch="handleLaunch"
                @stop="handleStop"
                @open-folder="handleOpenFolder"
                @delete="confirmDelete"
                @duplicate="handleDuplicate"
                @rename="handleRename"
                @settings="handleSettings"
              />
            </div>
          </div>

          <!-- Detail panel -->
          <div
            v-if="selectedProfile"
            class="w-72 shrink-0 hidden lg:block"
          >
            <InstanceDetail
              :profile="selectedProfile"
              :is-running="selectedProfileRunning"
              :running-processes="selectedProfileProcesses"
              @launch="handleLaunch"
              @stop="handleStop"
              @open-folder="handleOpenFolder"
              @settings="handleSettings"
              @close="selectedProfilePath = null"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Modals -->
    <InstanceCreationModal ref="creationModal" />

    <InstanceSettingsModal
      v-if="selectedProfile"
      ref="settingsModal"
      :instance="selectedProfile"
    />

    <ConfirmModalWrapper
      ref="deleteModal"
      title="Delete instance?"
      description="This will permanently delete this instance and all of its files. This action cannot be undone."
      proceed-label="Delete"
      :danger="true"
      @proceed="handleDeleteProceed"
    />
  </div>
</template>
