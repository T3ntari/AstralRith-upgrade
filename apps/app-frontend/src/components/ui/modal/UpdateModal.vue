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
        <span class="update-progress-text">Downloading and installing update... Please wait.</span>
      </div>

      <div class="update-actions">
        <ButtonStyled type="transparent" @click="decline">
          <button :disabled="installState">Remind me later</button>
        </ButtonStyled>
        <ButtonStyled color="brand" @click="accept">
          <button :disabled="installState">
            <DownloadIcon />
            {{ installState ? 'Updating...' : 'Update now' }}
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
  launcherUrl,
} from '@/helpers/update.js'
import { getVersion } from '@tauri-apps/api/app'

const modal = ref(null)
const localVersion = ref('')
const remoteVersion = ref('')

const emit = defineEmits(['update-complete'])

function show() {
  localVersion.value = ''
  remoteVersion.value = ''
  getVersion()
    .then((v) => (localVersion.value = `v${v}`))
    .catch(() => (localVersion.value = 'Unknown'))
  getRemote(false, false)
    .then(() => {
      remoteVersion.value = document.getElementById('releaseData')?.textContent || 'Unknown'
    })
    .catch(() => (remoteVersion.value = 'Unknown'))
  modal.value.show()
}

function hide() {
  modal.value.hide()
}

async function accept() {
  try {
    await getRemote(true, true)
    installState.value = false
    hide()
    emit('update-complete')
  } catch (err) {
    console.error('Update failed:', err)
  }
}

function decline() {
  updateState.value = false
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
}

.update-progress-text {
  font-size: 0.85rem;
  color: var(--color-secondary);
  text-align: center;
}

.update-actions {
  display: flex;
  gap: 0.75rem;
  justify-content: flex-end;
  margin-top: 0.5rem;
}
</style>
