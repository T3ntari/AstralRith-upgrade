<script setup>
import { DownloadIcon } from '@modrinth/assets'
import { Button, Badge } from '@modrinth/ui'
import { computed, ref } from 'vue'
import { getModFiles } from '@/helpers/curseforge.js'
import { handleError } from '@/store/notifications.js'
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
const loaderFilter = ref(0)
const installingFile = ref(null)

const availableGameVersions = computed(() => {
  const set = new Set()
  for (const file of files.value) {
    for (const version of file.gameVersions ?? []) set.add(version)
  }
  return Array.from(set).sort((a, b) => b.localeCompare(a))
})

const availableLoaders = computed(() => {
  const set = new Set()
  for (const file of files.value) {
    for (const loader of file.modLoader ?? []) set.add(loader)
  }
  return Array.from(set).sort()
})

const filteredFiles = computed(() => {
  let result = files.value
  if (gameVersionFilter.value) {
    result = result.filter((f) => (f.gameVersions ?? []).includes(gameVersionFilter.value))
  }
  if (loaderFilter.value) {
    result = result.filter((f) =>
      (f.modLoader ?? []).some((l) => l.toLowerCase() === loaderFilter.value.toLowerCase()),
    )
  }
  return result
})

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
  try {
    const gameVersion = props.instance?.game_version ?? ''
    const loaderId = 0
    files.value = await getModFiles(props.mod.id, {
      gameVersion,
      modLoaderType: loaderId,
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
  installingFile.value = file.id
  try {
    emit('install', file)
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
    loaderFilter.value = 0
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
          <option v-for="version in availableGameVersions" :key="version" :value="version">
            {{ version }}
          </option>
        </select>
        <select v-model="loaderFilter" class="input">
          <option :value="0">Any loader</option>
          <option v-for="loader in availableLoaders" :key="loader" :value="loader">
            {{ loader }}
          </option>
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
          >
            <div class="name-cell table-cell">
              <div class="version-name">
                {{ file.displayName }}
                <Badge :color="releaseTypeColor(file.releaseType)" class="ml-1">
                  {{ file.releaseType }}
                </Badge>
              </div>
            </div>
            <div class="table-cell table-text">
              <div class="supports">
                <span
                  v-for="version in (file.gameVersions ?? []).slice(0, 4)"
                  :key="version"
                  class="support-chip"
                >
                  {{ version }}
                </span>
                <span v-if="(file.gameVersions ?? []).length > 4">+{{ file.gameVersions.length - 4 }}</span>
              </div>
              <div class="supports">
                <span v-for="loader in (file.modLoader ?? []).slice(0, 3)" :key="loader" class="support-chip">
                  {{ loader }}
                </span>
              </div>
            </div>
            <div class="table-cell table-text">{{ formatBytes(file.fileLength) }}</div>
            <div class="table-cell table-text">
              <Button
                color="primary"
                :disabled="installingFile === file.id"
                @click="installFile(file)"
              >
                <DownloadIcon />
                {{ installingFile === file.id ? 'Installing' : 'Install' }}
              </Button>
            </div>
          </div>
          <div v-if="filteredFiles.length === 0 && !loading && !error" class="empty">
            No versions match the selected filters.
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

.table-head {
  color: var(--color-secondary);
  font-size: 0.8rem;
}

.scrollable {
  max-height: 24rem;
  overflow-y: auto;
}
</style>
