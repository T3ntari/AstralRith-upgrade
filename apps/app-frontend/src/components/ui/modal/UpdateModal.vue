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

      <div v-if="updateError" class="update-error">
        <p class="update-error-text">{{ updateError }}</p>
        <a :href="launcherUrl" target="_blank" class="update-error-link">Open releases page</a>
      </div>

      <div v-if="installState" class="update-progress">
        <div class="progress-header">
          <span class="update-progress-text">{{ statusText }}</span>
          <span class="update-eta">{{ formatEta(etaSeconds) }}</span>
        </div>
        <div class="progress-track">
          <div class="progress-fill" :style="{ width: progress + '%' }"></div>
        </div>
        <div class="progress-footer">
          <span>{{ Math.round(progress) }}%</span>
          <button class="cancel-link" @click="cancelUpdate" :disabled="installing">Cancel</button>
        </div>
      </div>

      <div v-else-if="finished" class="update-finished">
        <p class="update-finished-text">
          ✅ Update downloaded. The installer should open automatically.
          <template v-if="launchFailed"> If it did not open, find the file in your Downloads folder.</template>
        </p>
        <div class="update-actions">
          <ButtonStyled type="transparent">
            <button @click="closeAfterFinish">Close</button>
          </ButtonStyled>
          <ButtonStyled color="brand">
            <a :href="launcherUrl" target="_blank" class="update-now-btn">Open releases page</a>
          </ButtonStyled>
        </div>
      </div>

      <div class="update-actions" v-else>
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
import { ref, computed, onBeforeUnmount } from 'vue'
import { DownloadIcon } from '@modrinth/assets'
import { ButtonStyled } from '@modrinth/ui'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import {
  installState,
  getRemote,
  updateState,
  updateProgress,
  updateMessage,
  remoteVersion as remoteVersionRef,
  launcherUrl,
  onDownloadProgress,
} from '@/helpers/update.js'
import { getVersion } from '@tauri-apps/api/app'

const modal = ref(null)
const localVersion = ref('')
const remoteVersion = ref('')
const updateError = ref('')

// Real download progress (from Rust loading events) with ETA computed locally.
const progress = computed(() => Math.min(100, Math.round(updateProgress.value)))
const etaSeconds = ref(null)
const cancelled = ref(false)
const installing = ref(false)
const finished = ref(false)
const launchFailed = ref(false)
let startTime = null
let unlistenProgress = null
let etaTimer = null

const statusText = computed(() => updateMessage.value || 'Downloading update...')

const emit = defineEmits(['update-complete'])

function show() {
  localVersion.value = ''
  remoteVersion.value = ''
  updateError.value = ''
  finished.value = false
  launchFailed.value = false
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
  stopTracking()
  modal.value.hide()
}

function stopTracking() {
  if (unlistenProgress) {
    try {
      unlistenProgress()
    } catch (e) {
      /* ignore */
    }
    unlistenProgress = null
  }
  if (etaTimer) {
    clearInterval(etaTimer)
    etaTimer = null
  }
}

function startProgressTracking() {
  updateProgress.value = 0
  updateMessage.value = 'Starting download...'
  etaSeconds.value = null
  cancelled.value = false
  startTime = Date.now()

  // Compute ETA from real progress deltas every 500ms.
  let lastProgress = 0
  let lastTime = startTime
  etaTimer = setInterval(() => {
    const now = Date.now()
    const p = progress.value
    if (p > lastProgress && p < 100) {
      const dt = (now - lastTime) / 1000
      const dp = p - lastProgress
      if (dp > 0) {
        const speed = dp / dt // percent per second
        const remaining = (100 - p) / speed
        etaSeconds.value = Math.round(remaining)
      }
      lastProgress = p
      lastTime = now
    } else if (p >= 100) {
      etaSeconds.value = 0
    }
  }, 500)

  // Real progress from the Rust backend.
  unlistenProgress = onDownloadProgress(({ progress: p, message, done }) => {
    updateProgress.value = p
    if (message) updateMessage.value = message
    if (done) {
      updateProgress.value = 100
      updateMessage.value = 'Update downloaded.'
      etaSeconds.value = 0
      finishSuccess()
    }
  })
}

function finishSuccess() {
  stopTracking()
  installing.value = false
  installState.value = false
  finished.value = true
  updateState.value = false
  // Keep the dialog open showing "downloaded" state so nothing "disappears".
  // The installer launch happens inside Rust after download completes.
}

function formatEta(seconds) {
  if (seconds === null || seconds === undefined || seconds < 0) return 'estimating...'
  if (seconds < 5) return 'almost done'
  if (seconds < 60) return `${seconds}s remaining`
  return `${Math.floor(seconds / 60)}m ${seconds % 60}s remaining`
}

async function accept() {
  try {
    updateError.value = ''
    installing.value = true
    finished.value = false
    startProgressTracking()
    await getRemote(true, true)
    // getRemote resolves after the Rust download + installer launch handoff.
    if (!finished.value) {
      finishSuccess()
    }
    emit('update-complete')
  } catch (err) {
    console.error('Update failed:', err)
    installing.value = false
    installState.value = false
    stopTracking()
    updateError.value = err?.message || String(err)
  }
}

function decline() {
  cancelled.value = true
  updateState.value = false
  hide()
}

function cancelUpdate() {
  cancelled.value = true
  installing.value = false
  installState.value = false
  stopTracking()
  hide()
}

function closeAfterFinish() {
  hide()
}

onBeforeUnmount(() => {
  stopTracking()
})

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

  &:disabled {
    opacity: 0.5;
    cursor: default;
  }
}

.update-error {
  background-color: var(--color-red-bg, #fee2e2);
  border-left: 3px solid var(--color-red, #ef4444);
  padding: 0.75rem 1rem;
  border-radius: var(--radius-sm);
  font-size: 0.85rem;
  line-height: 1.4;
  color: var(--color-contrast);
}

.update-error-text {
  margin: 0 0 0.5rem 0;
  word-break: break-word;
}

.update-error-link {
  color: var(--color-brand);
  font-weight: 600;
  text-decoration: underline;
}

.update-now-btn {
  color: #1a2e05 !important;
  text-decoration: none;
}

.update-actions {
  display: flex;
  gap: 0.75rem;
  justify-content: flex-end;
  margin-top: 0.5rem;
}

.update-finished {
  background-color: var(--color-green-bg, #dcfce7);
  border-left: 3px solid var(--color-green, #22c55e);
  padding: 0.75rem 1rem;
  border-radius: var(--radius-sm);
  font-size: 0.85rem;
  line-height: 1.4;
  color: var(--color-contrast);
}

.update-finished-text {
  margin: 0 0 0.5rem 0;
}
</style>
