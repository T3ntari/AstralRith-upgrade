<template>
  <ModalWrapper ref="modal" :header="modalTitle" :has-to-type="false">
    <div class="update-body">
      <!-- Top: Current version -->
      <div class="version-hero">
        <div class="version-hero-icon">
          <svg viewBox="0 0 48 48" fill="none" class="hero-svg">
            <defs>
              <linearGradient id="heroGrad" x1="0" y1="0" x2="48" y2="48">
                <stop offset="0%" :stop-color="hasUpdate ? '#65a30d' : '#6b7280'" />
                <stop offset="100%" :stop-color="hasUpdate ? '#a3e635' : '#9ca3af'" />
              </linearGradient>
            </defs>
            <circle cx="24" cy="24" r="22" stroke="url(#heroGrad)" stroke-width="2.5" fill="none" opacity="0.3" />
            <circle cx="24" cy="24" r="16" stroke="url(#heroGrad)" stroke-width="1.5" fill="none" opacity="0.15" />
            <path d="M24 12v8l6 4" stroke="url(#heroGrad)" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" />
            <circle cx="24" cy="24" r="3" fill="url(#heroGrad)" />
          </svg>
        </div>
        <div class="version-hero-text">
          <h2 class="m-0 text-xl font-extrabold text-contrast">
            {{ hasUpdate ? 'Update Available' : 'You\'re Up to Date' }}
          </h2>
          <p class="m-0 text-sm text-secondary mt-1">
            <span class="ver-pill">{{ localVersion }}</span>
            <template v-if="hasUpdate">
              <span class="arrow">→</span>
              <span class="ver-pill accent">{{ remoteVersion }}</span>
            </template>
            <template v-else> — running the latest</template>
          </p>
        </div>
      </div>

      <!-- Status badges -->
      <div class="status-row">
        <div class="status-badge" :class="hasUpdate ? 'accent' : 'muted'">
          <span class="status-dot" />
          {{ hasUpdate ? 'New version found' : 'Latest version' }}
        </div>
        <div v-if="checking" class="status-badge checking">
          <span class="spin-dot" /> Checking...
        </div>
      </div>

      <!-- Check for updates -->
      <button v-if="!hasUpdate && !installing && !finished && !checking" class="check-btn" @click="doCheck">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M8 1v2M8 13v2M1 8h2M13 8h2M3.05 3.05l1.41 1.41M11.54 11.54l1.41 1.41M3.05 12.95l1.41-1.41M11.54 4.46l1.41-1.41" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
        Check for updates
      </button>

      <!-- Checking spinner -->
      <div v-if="checking" class="checking-box">
        <div class="pulse-ring" />
        <span class="text-sm text-secondary">Contacting GitHub...</span>
      </div>

      <!-- Error -->
      <div v-if="updateError" class="error-box">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><circle cx="8" cy="8" r="7" stroke="#ef4444" stroke-width="1.5"/><path d="M8 5v3M8 10.5v.5" stroke="#ef4444" stroke-width="1.5" stroke-linecap="round"/></svg>
        {{ updateError }}
      </div>

      <!-- Download progress -->
      <div v-if="installing" class="progress-section">
        <div class="progress-header">
          <span class="text-sm font-bold text-contrast">{{ statusText }}</span>
          <span class="text-sm font-bold" style="color: #a3e635">{{ progress }}%</span>
        </div>
        <div class="progress-track">
          <div class="progress-fill" :style="{ width: progress + '%' }">
            <div class="progress-glow" />
          </div>
        </div>
        <div class="progress-stats">
          <span class="text-xs text-secondary">{{ progress }}% complete</span>
          <button class="cancel-link" @click="cancelUpdate">Cancel</button>
        </div>
      </div>

      <!-- Finished -->
      <div v-if="finished" class="done-box">
        <div class="done-icon">✓</div>
        <div class="done-content">
          <span class="text-sm font-bold text-contrast">{{ updateMessage || 'Download complete' }}</span>
          <div class="done-actions">
            <button class="restart-btn" @click="restartApp">
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><path d="M1 1v4h4M13 13V9H9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/><path d="M11.5 5A5.5 5.5 0 0 0 2.3 2.3l-1.3 1.3M2.5 9a5.5 5.5 0 0 0 9.2 2.7l1.3-1.3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
              Restart App
            </button>
          </div>
        </div>
      </div>

      <!-- Update action -->
      <div v-if="hasUpdate && !installing && !finished && !checking" class="update-actions">
        <button class="btn-secondary" @click="decline">Later</button>
        <button class="btn-primary" @click="accept">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M8 1v10M4 7l4 4 4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/><path d="M2 13h12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
          Download & Install
        </button>
      </div>

      <!-- Older versions -->
      <div v-if="olderVersions.length > 0 && !installing" class="older-section">
        <div class="older-header">
          <h3 class="m-0 text-sm font-bold text-contrast">Previous Versions</h3>
          <span class="text-xs text-secondary">{{ olderVersions.length }} available</span>
        </div>
        <div class="older-list">
          <div v-for="ver in olderVersions" :key="ver.tag" class="older-row">
            <div class="older-left">
              <span class="older-tag">{{ ver.tag }}</span>
              <span class="older-date">{{ formatDate(ver.date) }}</span>
            </div>
            <div class="older-right">
              <span v-if="ver.size" class="older-size">{{ formatSize(ver.size) }}</span>
              <button v-if="ver.assetUrl" class="dl-btn" @click="handleOlderDownload(ver)">
                <svg width="12" height="12" viewBox="0 0 12 12" fill="none"><path d="M6 1v8M3 6l3 3 3-3" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/><path d="M1 10h10" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg>
                Download
              </button>
              <span v-else class="no-asset">N/A</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Release notes -->
      <div v-if="showNotes && latestNotes" class="notes-section">
        <button class="notes-toggle" @click="showNotes = !showNotes">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none"><path d="M2 4l4 4 4-4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg>
          Release Notes
        </button>
        <div class="notes-content">{{ latestNotes }}</div>
      </div>
    </div>
  </ModalWrapper>
</template>

<script setup>
import { ref, computed } from 'vue'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import {
  getRemote, updateState, updateProgress, updateMessage,
  remoteVersion as remoteVersionRef, olderVersions, fetchAllReleases,
  downloadOlderVersion, onDownloadProgress,
} from '@/helpers/update.js'
import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'

const modal = ref(null)
const localVersion = ref('')
const remoteVersion = ref('')
const updateError = ref('')
const checking = ref(false)
const installing = ref(false)
const finished = ref(false)
const showNotes = ref(false)
const progress = computed(() => Math.min(100, Math.round(updateProgress.value)))
const statusText = computed(() => updateMessage.value || 'Downloading...')
const hasUpdate = ref(false)
const latestNotes = ref('')
const modalTitle = computed(() => hasUpdate.value ? 'Update Available' : 'Version Manager')
let unlistenProgress = null

const emit = defineEmits(['update-complete'])

async function show() {
  updateError.value = ''
  finished.value = false
  installing.value = false
  checking.value = false

  try { localVersion.value = `v${await getVersion()}` } catch { localVersion.value = 'v?' }

  modal.value.show()

  loadingReleases.value = true
  try {
    await Promise.all([getRemote(false, false), fetchAllReleases()])
    remoteVersion.value = remoteVersionRef.value || ''
    hasUpdate.value = updateState.value
    if (allReleases.value?.length) {
      latestNotes.value = allReleases.value[0]?.body || ''
    }
  } catch (err) {
    updateError.value = err?.message || String(err)
  } finally {
    loadingReleases.value = false
  }
}

import { allReleases } from '@/helpers/update.js'
const loadingReleases = ref(false)

function hide() { stopTracking(); modal.value?.hide() }
function stopTracking() { if (unlistenProgress) { try { unlistenProgress() } catch {} unlistenProgress = null } }

async function doCheck() {
  checking.value = true
  updateError.value = ''
  try {
    await Promise.all([getRemote(false, false), fetchAllReleases()])
    remoteVersion.value = remoteVersionRef.value || ''
    hasUpdate.value = updateState.value
    if (allReleases.value?.length) latestNotes.value = allReleases.value[0]?.body || ''
  } catch (err) {
    updateError.value = err?.message || String(err)
  } finally {
    checking.value = false
  }
}

async function accept() {
  try {
    updateError.value = ''
    installing.value = true
    finished.value = false
    updateProgress.value = 0
    updateMessage.value = 'Preparing download...'

    unlistenProgress = onDownloadProgress(({ progress: p, message, done }) => {
      updateProgress.value = p
      if (message) updateMessage.value = message
      if (done) {
        updateProgress.value = 100
        installing.value = false
        finished.value = true
        updateMessage.value = 'Download complete!'
        updateState.value = false
        emit('update-complete')
      }
    })

    await getRemote(true, true)
    if (!finished.value) {
      installing.value = false
      finished.value = true
      updateMessage.value = 'Download complete!'
      emit('update-complete')
    }
  } catch (err) {
    installing.value = false
    updateError.value = err?.message || String(err)
    stopTracking()
  }
}

function decline() { hide() }
function cancelUpdate() { installing.value = false; stopTracking(); hide() }

async function restartApp() {
  try { await invoke('restart_app') } catch { window.location.reload() }
}

function formatDate(d) {
  return d ? new Date(d).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' }) : ''
}

function formatSize(bytes) {
  if (!bytes) return ''
  if (bytes > 1048576) return (bytes / 1048576).toFixed(1) + ' MB'
  return (bytes / 1024).toFixed(0) + ' KB'
}

async function handleOlderDownload(ver) {
  try {
    updateMessage.value = `Downloading ${ver.tag}...`
    updateProgress.value = 0
    installing.value = true

    unlistenProgress = onDownloadProgress(({ progress: p, message, done }) => {
      updateProgress.value = p
      if (message) updateMessage.value = message
      if (done) {
        installing.value = false
        finished.value = true
        updateMessage.value = `${ver.tag} downloaded!`
      }
    })

    await downloadOlderVersion(ver)
    if (!finished.value) {
      installing.value = false
      finished.value = true
      updateMessage.value = `${ver.tag} downloaded!`
    }
  } catch (err) {
    installing.value = false
    updateError.value = `Download failed: ${err.message}`
    stopTracking()
  }
}

defineExpose({ show, hide })
</script>

<style scoped>
.update-body {
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
  min-width: 520px;
  max-width: 600px;
  padding: 0.25rem;
}

/* Hero */
.version-hero {
  display: flex;
  gap: 1rem;
  align-items: center;
  padding: 1rem;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 12px;
}
.version-hero-icon { flex-shrink: 0; width: 48px; height: 48px; }
.hero-svg { width: 48px; height: 48px; }
.version-hero-text { flex: 1; }

.ver-pill {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 6px;
  background: var(--color-button-bg);
  font-weight: 700;
  font-size: 0.8rem;
  font-family: monospace;
}
.ver-pill.accent {
  background: rgba(163, 230, 53, 0.15);
  color: #a3e635;
}
.arrow {
  margin: 0 6px;
  color: var(--color-secondary);
}

/* Status */
.status-row { display: flex; gap: 0.5rem; flex-wrap: wrap; }
.status-badge {
  display: flex; align-items: center; gap: 6px;
  padding: 4px 10px; border-radius: 20px;
  font-size: 0.75rem; font-weight: 600;
}
.status-badge.accent { background: rgba(163, 230, 53, 0.12); color: #a3e635; }
.status-badge.muted { background: var(--color-button-bg); color: var(--color-secondary); }
.status-badge.checking { background: rgba(99, 102, 241, 0.12); color: #818cf8; }
.status-dot {
  width: 6px; height: 6px; border-radius: 50%;
  background: currentColor;
}
.spin-dot {
  width: 8px; height: 8px; border-radius: 50%;
  border: 1.5px solid transparent;
  border-top-color: currentColor;
  animation: spin 0.8s linear infinite;
}

/* Check button */
.check-btn {
  display: flex; align-items: center; gap: 8px;
  padding: 8px 16px; border-radius: 8px;
  background: var(--color-button-bg);
  border: 1px solid var(--color-border);
  color: var(--color-contrast);
  font-weight: 600; font-size: 0.85rem;
  cursor: pointer; transition: all 0.15s;
}
.check-btn:hover { background: var(--color-button-hover-bg, var(--color-button-bg)); border-color: var(--color-contrast); }

/* Checking */
.checking-box {
  display: flex; align-items: center; gap: 12px;
  padding: 1rem; justify-content: center;
}
.pulse-ring {
  width: 12px; height: 12px; border-radius: 50%;
  background: #818cf8;
  animation: pulse 1.5s ease-in-out infinite;
}

/* Error */
.error-box {
  display: flex; align-items: flex-start; gap: 8px;
  padding: 0.75rem; border-radius: 8px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: #f87171; font-size: 0.85rem;
}

/* Progress */
.progress-section {
  padding: 1rem; border-radius: 10px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
}
.progress-header { display: flex; justify-content: space-between; margin-bottom: 8px; }
.progress-track {
  height: 6px; border-radius: 99px;
  background: var(--color-button-bg);
  overflow: hidden; position: relative;
}
.progress-fill {
  height: 100%; border-radius: 99px;
  background: linear-gradient(90deg, #65a30d, #a3e635);
  transition: width 0.3s ease;
  position: relative;
}
.progress-glow {
  position: absolute; right: 0; top: -2px;
  width: 16px; height: 10px; border-radius: 50%;
  background: rgba(163, 230, 53, 0.4);
  filter: blur(4px);
  animation: glow-pulse 1s ease-in-out infinite;
}
.progress-stats {
  display: flex; justify-content: space-between;
  margin-top: 8px;
}
.cancel-link {
  background: none; border: none; color: #f87171;
  cursor: pointer; font-size: 0.75rem; font-weight: 600;
  padding: 0;
}

/* Done */
.done-box {
  display: flex; align-items: center; gap: 12px;
  padding: 0.875rem 1rem; border-radius: 10px;
  background: rgba(34, 197, 94, 0.08);
  border: 1px solid rgba(34, 197, 94, 0.2);
}
.done-icon {
  width: 32px; height: 32px; border-radius: 50%;
  background: rgba(34, 197, 94, 0.15);
  display: flex; align-items: center; justify-content: center;
  color: #22c55e; font-weight: 900; font-size: 1rem;
  flex-shrink: 0;
}
.done-content { flex: 1; display: flex; flex-direction: column; gap: 6px; }
.done-actions { display: flex; gap: 8px; }
.restart-btn {
  display: flex; align-items: center; gap: 6px;
  padding: 5px 12px; border-radius: 6px;
  background: rgba(34, 197, 94, 0.15);
  border: 1px solid rgba(34, 197, 94, 0.3);
  color: #4ade80; font-size: 0.8rem; font-weight: 600;
  cursor: pointer; transition: all 0.15s;
}
.restart-btn:hover { background: rgba(34, 197, 94, 0.25); }

/* Action buttons */
.update-actions { display: flex; gap: 8px; justify-content: flex-end; }
.btn-secondary {
  padding: 8px 16px; border-radius: 8px;
  background: var(--color-button-bg);
  border: 1px solid var(--color-border);
  color: var(--color-secondary); font-weight: 600;
  font-size: 0.85rem; cursor: pointer;
}
.btn-primary {
  display: flex; align-items: center; gap: 6px;
  padding: 8px 20px; border-radius: 8px;
  background: linear-gradient(135deg, #65a30d, #4d7c0f);
  border: none; color: #fff;
  font-weight: 700; font-size: 0.85rem;
  cursor: pointer; transition: all 0.15s;
}
.btn-primary:hover { filter: brightness(1.1); transform: translateY(-1px); }

/* Older versions */
.older-section {
  border-top: 1px solid var(--color-border);
  padding-top: 0.75rem;
}
.older-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; }
.older-list { display: flex; flex-direction: column; gap: 4px; }
.older-row {
  display: flex; align-items: center; justify-content: space-between;
  padding: 0.5rem 0.75rem; border-radius: 8px;
  background: var(--color-bg);
  border: 1px solid transparent;
  transition: border-color 0.15s;
}
.older-row:hover { border-color: var(--color-border); }
.older-left { display: flex; flex-direction: column; gap: 1px; }
.older-tag { font-weight: 700; font-size: 0.85rem; color: var(--color-contrast); font-family: monospace; }
.older-date { font-size: 0.7rem; color: var(--color-secondary); }
.older-right { display: flex; align-items: center; gap: 10px; }
.older-size { font-size: 0.7rem; color: var(--color-secondary); }
.dl-btn {
  display: flex; align-items: center; gap: 4px;
  padding: 4px 10px; border-radius: 6px;
  background: var(--color-button-bg);
  border: 1px solid var(--color-border);
  color: var(--color-contrast); font-size: 0.75rem;
  font-weight: 600; cursor: pointer;
  transition: all 0.15s;
}
.dl-btn:hover { background: rgba(163, 230, 53, 0.12); border-color: #a3e635; color: #a3e635; }
.no-asset { font-size: 0.7rem; color: var(--color-secondary); }

/* Release notes */
.notes-section { border-top: 1px solid var(--color-border); padding-top: 0.5rem; }
.notes-toggle {
  display: flex; align-items: center; gap: 6px;
  background: none; border: none; padding: 4px 0;
  color: var(--color-secondary); font-size: 0.8rem;
  font-weight: 600; cursor: pointer;
}
.notes-content {
  margin-top: 8px; padding: 0.75rem;
  background: var(--color-bg); border-radius: 8px;
  font-size: 0.8rem; color: var(--color-secondary);
  white-space: pre-wrap; max-height: 200px; overflow-y: auto;
  line-height: 1.5;
}

/* Animations */
@keyframes spin { to { transform: rotate(360deg); } }
@keyframes pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.5; transform: scale(0.8); }
}
@keyframes glow-pulse {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 0.8; }
}
</style>
