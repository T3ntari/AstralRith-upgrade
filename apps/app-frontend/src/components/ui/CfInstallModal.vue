<script setup>
import { DownloadIcon, ExternalIcon, InfoIcon } from '@modrinth/assets'
import { Button, Badge } from '@modrinth/ui'
import { computed, ref } from 'vue'
import { getCompatibleFiles, getFileDownloadUrl, projectUrl } from '@/helpers/curseforge.js'
import { handleError } from '@/store/notifications.js'
import { openUrl } from '@tauri-apps/plugin-opener'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'

const props = defineProps({
  mod: {
    type: Object,
    required: true,
  },
  instance: {
    type: Object,
    default: null,
  },
  modpack: {
    type: Boolean,
    default: false,
  },
})

const emit = defineEmits(['install'])

const cfInstallModal = ref(null)
const files = ref([])
const loading = ref(false)
const error = ref('')
const gameVersionFilter = ref('')
const loaderFilter = ref('')
const releaseFilter = ref('')
const installingFile = ref(null)
const manualFile = ref(null)

const loaderOptions = computed(() => {
  const set = new Set()
  for (const file of files.value) {
    for (const loader of file.loader_types ?? []) set.add(loader)
  }
  return Array.from(set).sort()
})

const gameVersionOptions = computed(() => {
  const set = new Set()
  for (const file of files.value) {
    for (const version of file.game_versions ?? []) set.add(version)
  }
  // semantic sort: newest first
  return Array.from(set).sort((a, b) => compareVersions(b, a))
})

const filteredFiles = computed(() => {
  let result = files.value
  if (gameVersionFilter.value) {
    result = result.filter((f) => (f.game_versions ?? []).includes(gameVersionFilter.value))
  }
  if (loaderFilter.value) {
    result = result.filter((f) =>
      (f.loader_types ?? []).some((l) => l.toLowerCase() === loaderFilter.value.toLowerCase()),
    )
  }
  if (releaseFilter.value) {
    result = result.filter((f) => f.release_type === releaseFilter.value)
  }
  return result
})

function isCompatible(file) {
  if (!props.instance) return true
  const mc = props.instance.game_version
  if (mc && (file.game_versions ?? []).length > 0) {
    const matches = (file.game_versions ?? []).some(
      (v) => v === mc || v.startsWith(`${mc}.`) || v === mc.replace(/\..+$/, ''),
    )
    if (!matches) return false
  }
  const loader = props.instance.loader
  if (loader && loader !== 'vanilla' && (file.loader_types ?? []).length > 0) {
    return (file.loader_types ?? []).some((l) => l.toLowerCase() === loader.toLowerCase())
  }
  return true
}

function compareVersions(a, b) {
  const parse = (v) => {
    const m = /^(\d+)\.(\d+)(?:\.(\d+))?/.exec(v ?? '')
    if (!m) return [0, 0, 0]
    return [Number(m[1]), Number(m[2]), m[3] ? Number(m[3]) : 0]
  }
  const [a1, a2, a3] = parse(a)
  const [b1, b2, b3] = parse(b)
  if (a1 !== b1) return a1 - b1
  if (a2 !== b2) return a2 - b2
  return a3 - b3
}

const releaseTypeColor = (releaseType) => {
  if (releaseType === 'release') return 'green'
  if (releaseType === 'beta') return 'gold'
  return 'red'
}

const formatBytes = (bytes) => {
  if (!bytes) return ''
  if (bytes >= 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
  if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KB`
  return `${bytes} B`
}

const loadFiles = async () => {
  loading.value = true
  error.value = ''
  manualFile.value = null
  try {
    const minecraftVersion = props.instance?.game_version ?? ''
    const loader = props.instance?.loader ?? ''
    files.value = await getCompatibleFiles(Number(props.mod.id), {
      minecraftVersion,
      loader,
      releaseType: '',
      pageSize: 100,
    })
    if (files.value.length === 0) {
      error.value = 'No files found for this project.'
    }
  } catch (err) {
    error.value = `Failed to load versions: ${err.message}`
  } finally {
    loading.value = false
  }
}

const installFile = async (file) => {
  if (installingFile.value) return
  installingFile.value = String(file.id)
  manualFile.value = null
  try {
    // Resolve the download through the Rust layer (CDN/manual-aware).
    const resolved = await getFileDownloadUrl(Number(props.mod.id), Number(file.id))
    if (resolved?.requires_manual_download) {
      manualFile.value = resolved
      return
    }
    emit('install', { file, resolved })
    cfInstallModal.value.hide()
  } catch (err) {
    handleError({ message: `Failed to install: ${err.message}` })
  } finally {
    installingFile.value = null
  }
}

defineExpose({
  show: () => {
    gameVersionFilter.value = ''
    loaderFilter.value = ''
    releaseFilter.value = ''
    manualFile.value = null
    loadFiles()
    cfInstallModal.value.show()
  },
})
</script>

<template>
  <ModalWrapper
    ref="cfInstallModal"
    class="cf-install-modal"
    :header="`Install ${modpack ? 'modpack' : 'mod'}: ${mod.name}`"
  >
    <div class="modal-body">
      <div class="filters">
        <select v-model="gameVersionFilter" class="input">
          <option value="">Any game version</option>
          <option v-for="version in gameVersionOptions" :key="version" :value="version">
            {{ version }}
          </option>
        </select>
        <select v-model="loaderFilter" class="input">
          <option value="">Any loader</option>
          <option v-for="loader in loaderOptions" :key="loader" :value="loader">
            {{ loader }}
          </option>
        </select>
        <select v-model="releaseFilter" class="input">
          <option value="">Any release</option>
          <option value="release">Release</option>
          <option value="beta">Beta</option>
          <option value="alpha">Alpha</option>
        </select>
      </div>

      <div v-if="loading" class="empty">Loading versions...</div>
      <div v-else-if="error" class="empty">{{ error }}</div>
      <div v-else class="table">
        <div class="table-row with-columns table-head">
          <div class="name-cell table-cell table-text">Version</div>
          <div class="table-cell table-text">Supports</div>
          <div class="table-cell table-text">Size</div>
          <div class="table-cell table-text" />
        </div>
        <div class="scrollable">
          <div
            v-for="file in filteredFiles"
            :key="file.id"
            class="table-row with-columns selectable"
            :class="{ incompatible: !isCompatible(file) }"
          >
            <div class="name-cell table-cell">
              <div class="version-name">
                {{ file.name }}
                <Badge :color="releaseTypeColor(file.release_type)" class="ml-1">
                  {{ file.release_type }}
                </Badge>
                <Badge v-if="file.is_server_pack" color="blue" class="ml-1">Server pack</Badge>
              </div>
            </div>
            <div class="table-cell table-text">
              <div class="supports">
                <span
                  v-for="version in (file.game_versions ?? []).slice(0, 3)"
                  :key="version"
                  class="support-chip"
                >
                  {{ version }}
                </span>
                <span v-if="(file.game_versions ?? []).length > 3"
                  >+{{ file.game_versions.length - 3 }}</span
                >
              </div>
              <div class="supports">
                <span
                  v-for="loader in (file.loader_types ?? []).slice(0, 3)"
                  :key="loader"
                  class="support-chip"
                >
                  {{ loader }}
                </span>
              </div>
              <div v-if="!isCompatible(file)" class="incompat-note">
                Not compatible with this instance
              </div>
            </div>
            <div class="table-cell table-text">{{ formatBytes(file.size) }}</div>
            <div class="table-cell table-text">
              <Button
                color="primary"
                :disabled="installingFile === String(file.id)"
                @click="installFile(file)"
              >
                <DownloadIcon />
                {{ installingFile === String(file.id) ? 'Installing' : 'Install' }}
              </Button>
            </div>
          </div>
          <div v-if="filteredFiles.length === 0 && !loading && !error" class="empty">
            No versions match the selected filters.
          </div>
        </div>
      </div>

      <!-- Manual download fallback -->
      <div v-if="manualFile" class="manual-download">
        <InfoIcon class="h-5 w-5 shrink-0 text-brand" />
        <div>
          <div class="font-semibold text-contrast">Automatic download unavailable</div>
          <p class="m-0 mt-1 text-sm text-secondary">
            This CurseForge file requires manual download.
          </p>
          <div class="flex gap-2 mt-3">
            <Button @click="openUrl(manualFile.project_url || projectUrl(props.mod, null, props.modpack))">
              <ExternalIcon /> Open CurseForge
            </Button>
            <Button @click="manualFile = null">Cancel</Button>
          </div>
        </div>
      </div>
    </div>
  </ModalWrapper>
</template>


<style scoped lang="scss">
.filters {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 0.75rem;

  .input {
    flex: 1;
    background-color: var(--color-button-bg);
    color: var(--color-contrast);
    border: 1px solid var(--color-button-border);
    border-radius: var(--radius-md);
    padding: 0.4rem 0.6rem;
  }
}

.empty {
  padding: 1.5rem;
  text-align: center;
  color: var(--color-secondary);
  font-size: 0.85rem;
}

.version-name {
  display: flex;
  align-items: center;
  font-weight: 600;
  color: var(--color-contrast);
}

.supports {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem;
  margin-top: 0.2rem;
}

.support-chip {
  font-size: 0.7rem;
  color: var(--color-secondary);
  background-color: var(--color-button-bg);
  border-radius: 0.35rem;
  padding: 0.1rem 0.4rem;
}

.incompatible {
  opacity: 0.55;
}

.incompat-note {
  font-size: 0.7rem;
  color: var(--color-red, #e5484d);
  margin-top: 0.25rem;
}

.table-head {
  color: var(--color-secondary);
  font-size: 0.8rem;
}

.scrollable {
  max-height: 24rem;
  overflow-y: auto;
}

.manual-download {
  display: flex;
  gap: 0.75rem;
  align-items: flex-start;
  margin-top: 1rem;
  padding: 1rem;
  background-color: var(--color-button-bg);
  border-radius: var(--radius-md);
}
</style>
