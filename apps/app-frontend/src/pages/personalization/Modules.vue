<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import {
  GridIcon,
  EyeIcon,
  BookIcon,
  PlusIcon,
  DownloadIcon,
  UploadIcon,
  TrashIcon,
  XIcon,
  CheckIcon,
  SettingsIcon,
  KeyIcon,
} from '@modrinth/assets'
import { Button, Badge } from '@modrinth/ui'
import { useModulesStore } from '@/store/modules.js'
import ModuleCard from '@/components/modules/ModuleCard.vue'
import ModuleCategorySidebar from '@/components/modules/ModuleCategorySidebar.vue'
import ModuleSettings from '@/components/modules/ModuleSettings.vue'
import ModuleSearch from '@/components/modules/ModuleSearch.vue'
import HudEditor from '@/components/modules/HudEditor.vue'

const store = useModulesStore()

// Tab state
const activeTab = ref('modules')
const tabs = [
  { id: 'modules', name: 'Modules', icon: GridIcon },
  { id: 'hud', name: 'HUD', icon: EyeIcon },
  { id: 'profiles', name: 'Profiles', icon: BookIcon },
]

// Category filter
const activeCategory = ref('all')

// Expanded module
const expandedModuleId = ref(null)

// Keybind capture state
const capturingKeybind = ref(null)

// Profile management
const showNewProfile = ref(false)
const newProfileName = ref('')
const confirmDeleteProfile = ref(null)

onMounted(() => {
  store.refresh()
})

// Computed
const displayedModules = computed(() => {
  if (activeCategory.value === 'all') return store.filteredModules
  return store.filteredModules.filter(m => m.category === activeCategory.value)
})

const moduleCounts = computed(() => {
  const counts = {}
  for (const m of store.filteredModules) {
    counts[m.category] = (counts[m.category] || 0) + 1
  }
  return counts
})

const enabledCounts = computed(() => {
  const counts = {}
  for (const m of store.modules) {
    if (m.enabled) {
      counts[m.category] = (counts[m.category] || 0) + 1
    }
  }
  return counts
})

// Keyboard shortcuts
function handleKeydown(e) {
  if (e.key === 'Escape') {
    expandedModuleId.value = null
    capturingKeybind.value = null
    showNewProfile.value = false
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})
onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})

// Methods
function toggleExpand(moduleId) {
  expandedModuleId.value = expandedModuleId.value === moduleId ? null : moduleId
}

function handleToggle(moduleId) {
  store.toggleModule(moduleId)
}

function handleUpdateSetting({ moduleId, settingId, value }) {
  store.updateModuleSetting(moduleId, settingId, value)
}

function handleReset(moduleId) {
  store.resetModule(moduleId)
}

function startKeybindCapture(module) {
  capturingKeybind.value = module.id
}

function handleKeybindKeydown(e) {
  if (!capturingKeybind.value) return
  e.preventDefault()
  e.stopPropagation()

  const keybind = {
    key: e.key === ' ' ? 'Space' : e.key === 'Escape' ? null : e.key,
    code: e.code,
  }

  if (keybind.key) {
    store.setKeybind(capturingKeybind.value, keybind)
  }
  capturingKeybind.value = null
}

onMounted(() => {
  window.addEventListener('keydown', handleKeybindKeydown, true)
})
onUnmounted(() => {
  window.removeEventListener('keydown', handleKeybindKeydown, true)
})

// Profile functions
function createProfile() {
  if (!newProfileName.value.trim()) return
  const profile = {
    id: Date.now().toString(),
    name: newProfileName.value.trim(),
    modules: {},
  }
  store.profiles.push(profile)
  localStorage.setItem('ar_module_profiles', JSON.stringify(store.profiles))
  newProfileName.value = ''
  showNewProfile.value = false
}

function switchProfile(profileId) {
  store.activeProfileId = profileId
  const profile = store.profiles.find(p => p.id === profileId)
  if (profile) {
    // Apply profile module states
    for (const m of store.modules) {
      const saved = profile.modules[m.id]
      if (saved !== undefined) {
        m.enabled = saved
      }
    }
    store.saveToLocal?.()
  }
}

function saveCurrentToProfile(profileId) {
  const profile = store.profiles.find(p => p.id === profileId)
  if (profile) {
    profile.modules = {}
    for (const m of store.modules) {
      profile.modules[m.id] = m.enabled
    }
    localStorage.setItem('ar_module_profiles', JSON.stringify(store.profiles))
  }
}

function deleteProfile(profileId) {
  store.profiles = store.profiles.filter(p => p.id !== profileId)
  if (store.activeProfileId === profileId && store.profiles.length > 0) {
    store.activeProfileId = store.profiles[0].id
  }
  localStorage.setItem('ar_module_profiles', JSON.stringify(store.profiles))
  confirmDeleteProfile.value = null
}

function exportProfile(profile) {
  const data = JSON.stringify(profile, null, 2)
  const blob = new Blob([data], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${profile.name.replace(/[^a-z0-9]/gi, '_')}.json`
  a.click()
  URL.revokeObjectURL(url)
}

function importProfile() {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.json'
  input.onchange = async (e) => {
    const file = e.target.files?.[0]
    if (!file) return
    try {
      const text = await file.text()
      const profile = JSON.parse(text)
      if (profile.id && profile.name && profile.modules) {
        profile.id = Date.now().toString() // New ID to avoid conflicts
        store.profiles.push(profile)
        localStorage.setItem('ar_module_profiles', JSON.stringify(store.profiles))
      }
    } catch (err) {
      console.error('Failed to import profile:', err)
    }
  }
  input.click()
}

function selectCategory(cat) {
  activeCategory.value = cat
  store.searchQuery = ''
}

const activeProfile = computed(() =>
  store.profiles.find(p => p.id === store.activeProfileId)
)
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Tab bar -->
    <div class="flex items-center gap-1 px-6 pt-4 border-b border-secondary">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="flex items-center gap-1.5 px-4 py-2 text-sm font-medium transition-colors relative -mb-px"
        :class="activeTab === tab.id
          ? 'text-brand border-b-2 border-brand'
          : 'text-secondary hover:text-contrast border-b-2 border-transparent'
        "
        @click="activeTab = tab.id"
      >
        <component :is="tab.icon" class="w-4 h-4" />
        {{ tab.name }}
      </button>

      <div class="ml-auto flex items-center gap-2 py-2">
        <template v-if="activeTab === 'modules'">
          <span class="text-sm text-secondary">
            {{ store.enabledCount }}/{{ store.modules.length }} active
          </span>
        </template>
        <template v-if="activeTab === 'profiles'">
          <Button :action="importProfile" :outline="true">
            <UploadIcon class="w-4 h-4 mr-1" />
            Import
          </Button>
          <Button color="green" :action="() => showNewProfile = true">
            <PlusIcon class="w-4 h-4 mr-1" />
            New Profile
          </Button>
        </template>
      </div>
    </div>

    <!-- Tab content -->
    <div class="flex-1 overflow-hidden">
      <!-- Modules Tab -->
      <div v-if="activeTab === 'modules'" class="flex h-full">
        <!-- Category sidebar -->
        <div class="px-4 pt-4 overflow-y-auto">
          <ModuleCategorySidebar
            :active-category="activeCategory"
            :module-counts="moduleCounts"
            :enabled-counts="enabledCounts"
            @select="selectCategory"
          />
        </div>

        <!-- Module list -->
        <div class="flex-1 flex flex-col overflow-hidden px-4 pt-4 pb-4">
          <!-- Search -->
          <div class="mb-3">
            <ModuleSearch />
          </div>

          <!-- Module cards -->
          <div class="flex-1 overflow-y-auto space-y-2 pr-1">
            <template v-if="displayedModules.length > 0">
              <ModuleCard
                v-for="mod in displayedModules"
                :key="mod.id"
                :module="mod"
                :expanded="expandedModuleId === mod.id"
                @toggle="handleToggle"
                @expand="toggleExpand"
                @keybind="startKeybindCapture"
                @reset="handleReset"
              >
                <template #settings>
                  <ModuleSettings
                    :settings="mod.settings"
                    :module-id="mod.id"
                    @update-setting="handleUpdateSetting"
                  />
                </template>
              </ModuleCard>
            </template>

            <!-- Empty state -->
            <div
              v-else
              class="flex flex-col items-center justify-center py-16 gap-3 text-center"
            >
              <GridIcon class="w-10 h-10 text-secondary/30" />
              <p class="text-secondary text-sm m-0">
                {{ store.searchQuery ? 'No modules match your search' : 'No modules in this category' }}
              </p>
            </div>
          </div>
        </div>
      </div>

      <!-- HUD Tab -->
      <div v-else-if="activeTab === 'hud'" class="h-full px-6 py-4 overflow-y-auto">
        <HudEditor />
      </div>

      <!-- Profiles Tab -->
      <div v-else-if="activeTab === 'profiles'" class="h-full px-6 py-4 overflow-y-auto">
        <!-- New profile form -->
        <div v-if="showNewProfile" class="mb-4 rounded-xl border border-secondary bg-bg-raised px-4 py-3">
          <div class="flex items-center gap-2 mb-2">
            <PlusIcon class="w-4 h-4 text-green-400" />
            <span class="font-semibold text-contrast">New Profile</span>
          </div>
          <div class="flex items-center gap-2">
            <input
              v-model="newProfileName"
              type="text"
              class="flex-1 bg-black/30 border border-secondary rounded-lg px-3 py-1.5 text-sm text-contrast focus:outline-none focus:border-brand/50 transition-colors"
              placeholder="Profile name..."
              @keydown.enter="createProfile"
              autofocus
            />
            <Button color="green" :action="createProfile">Create</Button>
            <Button :action="() => showNewProfile = false">Cancel</Button>
          </div>
        </div>

        <!-- Profiles list -->
        <div class="space-y-2">
          <div
            v-for="profile in store.profiles"
            :key="profile.id"
            class="rounded-xl border px-4 py-3 transition-all duration-200"
            :class="[
              profile.id === store.activeProfileId
                ? 'border-brand/30 bg-brand/5'
                : 'border-secondary bg-bg-raised hover:bg-bg-raised/80'
            ]"
          >
            <div class="flex items-center gap-3">
              <div
                class="w-3 h-3 rounded-full flex-shrink-0"
                :class="profile.id === store.activeProfileId ? 'bg-brand' : 'bg-secondary/40'"
              />
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <span class="font-semibold text-contrast">{{ profile.name }}</span>
                  <Badge
                    v-if="profile.id === store.activeProfileId"
                    color="brand"
                    type="active"
                  />
                  <span class="text-xs text-secondary">
                    {{ Object.values(profile.modules).filter(Boolean).length }} modules active
                  </span>
                </div>
              </div>

              <div class="flex items-center gap-1.5">
                <Button
                  v-if="profile.id !== store.activeProfileId"
                  color="green"
                  :action="() => switchProfile(profile.id)"
                >
                  Switch
                </Button>
                <Button
                  v-if="profile.id === store.activeProfileId"
                  :outline="true"
                  :action="() => saveCurrentToProfile(profile.id)"
                >
                  Save to profile
                </Button>
                <Button :transparent="true" :icon-only="true" :action="() => exportProfile(profile)" title="Export">
                  <DownloadIcon class="w-4 h-4" />
                </Button>
                <template v-if="confirmDeleteProfile === profile.id">
                  <Button color="red" :icon-only="true" :action="() => deleteProfile(profile.id)" title="Confirm">
                    <CheckIcon class="w-4 h-4" />
                  </Button>
                  <Button :transparent="true" :icon-only="true" :action="() => confirmDeleteProfile = null" title="Cancel">
                    <XIcon class="w-4 h-4" />
                  </Button>
                </template>
                <template v-else>
                  <Button
                    :transparent="true"
                    :icon-only="true"
                    :action="() => confirmDeleteProfile = profile.id"
                    title="Delete"
                  >
                    <TrashIcon class="w-4 h-4" />
                  </Button>
                </template>
              </div>
            </div>

            <!-- Profile module preview -->
            <div v-if="profile.id === store.activeProfileId" class="mt-2 pt-2 border-t border-secondary/50">
              <div class="flex flex-wrap gap-1.5">
                <span
                  v-for="mod in store.modules.filter(m => m.enabled)"
                  :key="mod.id"
                  class="text-xs px-1.5 py-0.5 rounded bg-brand/10 text-brand/80 border border-brand/20"
                >
                  {{ mod.name }}
                </span>
                <span
                  v-if="store.modules.filter(m => m.enabled).length === 0"
                  class="text-xs text-secondary/50"
                >
                  No modules active
                </span>
              </div>
            </div>
          </div>

          <!-- Empty state -->
          <div
            v-if="store.profiles.length === 0"
            class="flex flex-col items-center justify-center py-16 gap-3 text-center"
          >
            <BookIcon class="w-10 h-10 text-secondary/30" />
            <p class="text-secondary text-sm m-0">No profiles created yet</p>
            <Button color="green" :action="() => showNewProfile = true">
              <PlusIcon class="w-4 h-4 mr-1" />
              Create your first profile
            </Button>
          </div>
        </div>
      </div>
    </div>

    <!-- Keybind capture overlay -->
    <Teleport to="body">
      <Transition name="modal-fade">
        <div
          v-if="capturingKeybind"
          class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
          @click="capturingKeybind = null"
        >
          <div class="bg-bg-raised border border-brand/30 rounded-xl shadow-2xl px-8 py-6 text-center">
            <KeyIcon class="w-10 h-10 text-brand mx-auto mb-3" />
            <h3 class="text-lg font-bold text-contrast m-0 mb-1">Press a key</h3>
            <p class="text-sm text-secondary m-0">
              Press any key to set as keybind, or <kbd class="px-1.5 py-0.5 rounded bg-bg-base border border-secondary text-xs">Esc</kbd> to cancel.
            </p>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.modal-fade-enter-active,
.modal-fade-leave-active {
  transition: opacity 0.15s ease;
}
.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
}
</style>
