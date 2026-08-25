<script setup>
import { ref, computed } from 'vue'
import {
  UploadIcon,
  TrashIcon,
  PlayIcon,
  StopCircleIcon,
  ShieldIcon,
  CodeIcon,
  FileIcon,
  XIcon,
  CheckIcon,
  InfoIcon,
} from '@modrinth/assets'
import { Button, Checkbox, Badge } from '@modrinth/ui'

const props = defineProps({
  visible: { type: Boolean, default: false },
})

const emit = defineEmits(['close', 'import-script'])

// Available scripts data
const scripts = ref([
  {
    id: 'farm',
    name: 'farm.js',
    runtime: 'Mineflayer',
    version: '1.2.0',
    permissions: ['movement', 'inventory', 'chat'],
    enabled: true,
    trusted: true,
    description: 'Automated farming bot with crop detection.',
  },
  {
    id: 'guard',
    name: 'guard.js',
    runtime: 'Mineflayer',
    version: '1.0.3',
    permissions: ['movement', 'combat', 'inventory'],
    enabled: true,
    trusted: true,
    description: 'Guard bot that patrols and attacks hostile mobs.',
  },
  {
    id: 'builder',
    name: 'builder.js',
    runtime: 'Mineflayer',
    version: '2.1.0',
    permissions: ['movement', 'inventory', 'world_edit'],
    enabled: false,
    trusted: true,
    description: 'Automated builder from schematic files.',
  },
  {
    id: 'chat',
    name: 'chat.js',
    runtime: 'Vanilla',
    version: '1.0.0',
    permissions: ['chat'],
    enabled: true,
    trusted: false,
    description: 'Chat responder with custom triggers.',
  },
])

const confirmDelete = ref(null)
const fileInput = ref(null)

const enabledScripts = computed(() => scripts.value.filter(s => s.enabled))

function toggleScript(id) {
  const script = scripts.value.find(s => s.id === id)
  if (script) script.enabled = !script.enabled
}

function requestDelete(script) {
  confirmDelete.value = script.id
}

function confirmDeleteScript(id) {
  scripts.value = scripts.value.filter(s => s.id !== id)
  confirmDelete.value = null
}

function cancelDelete() {
  confirmDelete.value = null
}

function importScript() {
  if (fileInput.value) fileInput.value.click()
}

function handleFileSelect(event) {
  const files = event.target?.files
  if (!files) return
  for (const file of files) {
    if (!file.name.endsWith('.js')) continue
    scripts.value.push({
      id: Date.now() + Math.random(),
      name: file.name,
      runtime: 'Unknown',
      version: '0.0.0',
      permissions: [],
      enabled: false,
      trusted: false,
      description: 'Imported script — not yet verified.',
    })
  }
  emit('import-script', files)
  if (fileInput.value) fileInput.value.value = ''
}

function runtimeBadgeColor(runtime) {
  switch (runtime) {
    case 'Mineflayer': return 'green'
    case 'Vanilla': return 'blue'
    default: return 'gray'
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div
        v-if="visible"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
        @click.self="emit('close')"
      >
        <div
          class="bg-bg-raised border border-secondary rounded-xl shadow-2xl w-full max-w-2xl mx-4 flex flex-col"
          style="max-height: 70vh;"
        >
          <!-- Header -->
          <div class="flex items-center gap-3 px-4 py-3 border-b border-secondary">
            <CodeIcon class="w-5 h-5 text-blue-400" />
            <span class="font-bold text-contrast text-lg">Bot Scripts</span>
            <span class="text-sm text-secondary ml-1">({{ enabledScripts.length }} active)</span>
            <div class="ml-auto flex items-center gap-2">
              <input
                ref="fileInput"
                type="file"
                accept=".js"
                multiple
                class="hidden"
                @change="handleFileSelect"
              />
              <Button color="green" :action="importScript">
                <UploadIcon class="w-4 h-4 mr-1" />
                Import Script
              </Button>
              <Button :transparent="true" :icon-only="true" :action="() => emit('close')">
                <XIcon class="w-4 h-4" />
              </Button>
            </div>
          </div>

          <!-- Scripts list -->
          <div class="flex-1 overflow-y-auto px-4 py-3 space-y-2">
            <div
              v-for="script in scripts"
              :key="script.id"
              class="rounded-lg border p-3 transition-colors"
              :class="[
                script.enabled ? 'border-green-500/20 bg-green-500/5' : 'border-secondary bg-bg-base/30'
              ]"
            >
              <!-- Security warning for untrusted -->
              <div
                v-if="!script.trusted"
                class="flex items-center gap-2 px-2 py-1 rounded bg-yellow-500/10 border border-yellow-500/20 text-yellow-400 text-xs mb-2"
              >
                <ShieldIcon class="w-3.5 h-3.5 flex-shrink-0" />
                <span>Untrusted script — review before enabling.</span>
              </div>

              <div class="flex items-start gap-3">
                <FileIcon class="w-5 h-5 text-secondary mt-0.5 flex-shrink-0" />
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 mb-1">
                    <span class="font-semibold text-contrast">{{ script.name }}</span>
                    <Badge :color="runtimeBadgeColor(script.runtime)" :type="script.runtime" />
                    <span class="text-xs text-secondary">v{{ script.version }}</span>
                  </div>
                  <p class="text-sm text-secondary m-0">{{ script.description }}</p>
                  <div v-if="script.permissions.length" class="flex flex-wrap gap-1 mt-1.5">
                    <span
                      v-for="perm in script.permissions"
                      :key="perm"
                      class="text-xs px-1.5 py-0.5 rounded bg-bg-base border border-secondary text-secondary"
                    >
                      {{ perm }}
                    </span>
                  </div>
                </div>

                <div class="flex items-center gap-2 flex-shrink-0">
                  <Checkbox
                    :model-value="script.enabled"
                    :label="script.enabled ? 'On' : 'Off'"
                    @update:model-value="toggleScript(script.id)"
                  />

                  <template v-if="confirmDelete === script.id">
                    <Button color="red" :action="() => confirmDeleteScript(script.id)" :icon-only="true" title="Confirm delete">
                      <CheckIcon class="w-4 h-4" />
                    </Button>
                    <Button :transparent="true" :icon-only="true" :action="cancelDelete" title="Cancel">
                      <XIcon class="w-4 h-4" />
                    </Button>
                  </template>
                  <template v-else>
                    <Button :transparent="true" :icon-only="true" :action="() => requestDelete(script)" title="Delete script">
                      <TrashIcon class="w-4 h-4" />
                    </Button>
                  </template>
                </div>
              </div>
            </div>

            <div v-if="scripts.length === 0" class="text-center py-12 text-secondary/50">
              <CodeIcon class="w-8 h-8 mx-auto mb-2 opacity-40" />
              <p>No scripts available. Import a .js file to get started.</p>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-fade-enter-active,
.modal-fade-leave-active {
  transition: opacity 0.2s ease;
}
.modal-fade-enter-active > div,
.modal-fade-leave-active > div {
  transition: transform 0.2s ease;
}
.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
}
.modal-fade-enter-from > div,
.modal-fade-leave-to > div {
  transform: scale(0.95) translateY(10px);
}
</style>
