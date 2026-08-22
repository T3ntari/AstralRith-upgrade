<template>
  <ModalWrapper ref="modal" :header="'AstralRinth Update Available'" :has-to-type="false">
    <div class="update-modal-body">
      <div class="update-icon-wrapper">
        <DownloadIcon class="update-icon" />
      </div>
      <h2 class="update-title">A new version of AstralRinth is available</h2>
      <p class="update-subtitle">
        Version <span class="version-badge">{{ localVersion }}</span> is installed. A newer version
        <span class="version-badge brand">{{ remoteVersion }}</span> is ready to download.
      </p>

      <div class="update-info-grid">
        <div class="update-info-row">
          <span class="update-info-label">Current version</span>
          <span class="update-info-value">{{ localVersion }}</span>
        </div>
        <div class="update-info-row">
          <span class="update-info-label">Latest version</span>
          <span class="update-info-value brand">{{ remoteVersion }}</span>
        </div>
      </div>

      <div class="update-warning">
        <strong>Before updating:</strong> Make sure all running instances are stopped and consider
        backing up important instance data. The authors are not responsible for data loss.
      </div>

      <div v-if="installState" class="update-progress">
        <div class="progress-header">
          <span class="update-progress-text">Downloading update...</span>
          <span class="update-eta">{{ formatEta(etaSeconds) }}</span>
        </div>
        <div class="progress-track">
          <div class="progress-fill" :style="{ width: progress + '%' }"></div>
        </div>
        <div class="progress-footer">
          <span>{{ Math.round(progress) }}%</span>
          <button class="cancel-link" @click="cancelUpdate">Cancel</button>
        </div>
      </div>

      <div class="update-actions" v-if="!installState">
        <ButtonStyled type="transparent">
          <button @click="decline">Not now</button>
        </ButtonStyled>
        <ButtonStyled color="brand">
          <button class="update-now-btn" @click="accept">
            <DownloadIcon />
            Update now
          </button>
        </ButtonStyled>
      </div>
    </div>
  </ModalWrapper>
</template>

<script setup>
import { ref, computed } from 'vue'
import { DownloadIcon } from '@modrinth/assets'
import { ButtonStyled, Button } from '@modrinth/ui'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import {
  installState,
  getRemote,
  updateState,
  remoteVersion as remoteVersionRef,
  launcherUrl,
} from '@/helpers/update.js'
import { getVersion } from '@tauri-apps/api/app'

const modal = ref(null)
const localVersion = ref('')
const remoteVersion = ref('')

// Download progress tracking
const progress = ref(0)
const etaSeconds = ref(null)
const cancelled = ref(false)
let progressTimer = null
let startTime = null

const emit = defineEmits(['update-complete'])

function show() {
  localVersion.value = ''
  remoteVersion.value = ''
  getVersion()
    .then((v) => (localVersion.value = `v${v}`))
    .catch(() => (localVersion.value = 'Unknown'))
  getRemote(false, false)
    .then(() => {
      remoteVersion.value = remoteVersionRef.value || 'Unknown'
    })
    .catch(() => (remoteVersion.value = 'Unknown'))
  modal.value.show()
}

function hide() {
  stopProgressTracking()
  modal.value.hide()
}

function startProgressTracking() {
  progress.value = 0
  etaSeconds.value = null
  cancelled.value = false
  startTime = Date.now()

  // Simulated progress while the backend downloads (ease-out curve so
  // ETA stabilizes). Replaced with real progress once backend reports it.
  progressTimer = setInterval(() => {
    if (!installState.value) {
      progress.value = 100
      etaSeconds.value = 0
      stopProgressTracking()
      return
    }
    const elapsed = (Date.now() - startTime) / 1000
    const target = Math.min(95, 100 * (1 - Math.exp(-elapsed / 25)))
    if (target > progress.value) {
      const speed = progress.value > 2 ? elapsed / progress.value : null
      progress.value = target
      if (speed) {
        etaSeconds.value = Math.round(speed * (100 - progress.value))
      }
    }
  }, 500)
}

function stopProgressTracking() {
  if (progressTimer) {
    clearInterval(progressTimer)
    progressTimer = null
  }
}

function formatEta(seconds) {
  if (seconds === null || seconds === undefined) return 'estimating...'
  if (seconds < 5) return 'almost done'
  if (seconds < 60) return `${seconds}s remaining`
  return `${Math.floor(seconds / 60)}m ${seconds % 60}s remaining`
}

async function accept() {
  try {
    startProgressTracking()
    await getRemote(true, true)
    installState.value = false
    hide()
    emit('update-complete')
  } catch (err) {
    console.error('Update failed:', err)
    installState.value = false
    stopProgressTracking()
  }
}

function decline() {
  cancelled.value = true
  updateState.value = false
  hide()
}

function cancelUpdate() {
  // The native download continues, but we stop tracking and close.
  cancelled.value = true
  installState.value = false
  stopProgressTracking()
  hide()
}

defineExpose({
  show,
  hide,
})
</script>

<style scoped lang="scss">
.update-modal-body {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 0.5rem;
}

.update-icon-wrapper {
  display: flex;
  justify-content: center;
  margin-bottom: 0.5rem;
}

.update-icon {
  width: 3rem;
  height: 3rem;
  color: var(--color-brand);
}

.update-title {
  margin: 0;
  text-align: center;
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--color-contrast);
}

.update-subtitle {
  margin: 0;
  text-align: center;
  color: var(--color-secondary);
  line-height: 1.5;
}

.version-badge {
  display: inline-block;
  padding: 0.1rem 0.5rem;
  border-radius: var(--radius-sm);
  background-color: var(--color-button-bg);
  font-weight: 600;
  color: var(--color-contrast);

  &.brand {
    background-color: var(--color-brand-highlight);
    color: var(--color-brand);
  }
}

.update-info-grid {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  background-color: var(--color-bg);
  border-radius: var(--radius-md);
  padding: 1rem;
}

.update-info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.update-info-label {
  color: var(--color-secondary);
  font-weight: 500;
}

.update-info-value {
  font-weight: 700;
  color: var(--color-contrast);

  &.brand {
    color: var(--color-brand);
  }
}

.update-warning {
  background-color: var(--color-orange-bg, #fff3e0);
  border-left: 3px solid var(--color-orange, #ff9800);
  padding: 0.75rem 1rem;
  border-radius: var(--radius-sm);
  font-size: 0.85rem;
  line-height: 1.4;
  color: var(--color-contrast);
}

.update-progress {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  background-color: var(--color-bg);
  border-radius: var(--radius-md);
  padding: 1rem;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.update-progress-text {
  font-size: 0.85rem;
  color: var(--color-contrast);
  font-weight: 600;
}

.update-eta {
  font-size: 0.8rem;
  color: #a3e635;
  font-weight: 600;
}

.progress-track {
  height: 6px;
  border-radius: 9999px;
  background-color: var(--color-button-bg);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: 9999px;
  background: linear-gradient(90deg, #65a30d, #a3e635);
  transition: width 0.4s ease-out;
}

.progress-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.78rem;
  color: var(--color-secondary);
}

.cancel-link {
  background: none;
  border: none;
  color: var(--color-red);
  cursor: pointer;
  font-size: 0.78rem;
  font-weight: 600;
  padding: 0;

  &:hover {
    text-decoration: underline;
  }
}

.update-now-btn {
  color: #1a2e05 !important;
}

.update-actions {
  display: flex;
  gap: 0.75rem;
  justify-content: flex-end;
  margin-top: 0.5rem;
}
</style>
