import { ref } from 'vue'
import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'
import { loading_listener } from '@/helpers/events.js'

// Track last downloaded version in localStorage to prevent update loops
function getLastDownloadedVersion() {
  try { return localStorage.getItem('astralrinth_last_downloaded_version') || '' } catch { return '' }
}
function setLastDownloadedVersion(v) {
  try { localStorage.setItem('astralrinth_last_downloaded_version', v) } catch {}
}

export const allowState = ref(false)
export const installState = ref(false)
export const updateState = ref(false)
export const remoteVersion = ref('')
export const localVersion = ref('')
export const updateProgress = ref(0)
export const updateMessage = ref('')
export const launcherUrl = 'https://github.com/T3ntari/AstralRith-upgrade/releases'
export const allReleases = ref([])
export const olderVersions = ref([])

const REPO = 'T3ntari/AstralRith-upgrade'
const GITHUB_API = `https://api.github.com/repos/${REPO}`
const os = ref('')
const osNames = ['macos', 'windows', 'linux']
const linuxExtensions = ['.deb', '.rpm', '.AppImage']
const blacklistedBuilds = ['dev', 'nightly', 'dirty', 'dirty-dev', 'dirty-nightly']

function getExtensions() {
  const o = os.value.toLowerCase()
  if (o === 'macos') return ['.dmg']
  if (o === 'windows') return ['.msi']
  if (o === 'linux') return linuxExtensions
  return []
}

function stripV(v) {
  const s = String(v || '').trim()
  // Extract version number: skip any prefix like "AstralRinth v" and extract X.Y.Z...
  const match = s.match(/(\d[\d.]*(?:[+\-]\d[\d.]*)*)/)
  return match ? match[1] : s.replace(/^v/i, '')
}

function cmpVersions(a, b) {
  const pa = stripV(a).split(/[.+-]/).map(x => parseInt(x, 10) || 0)
  const pb = stripV(b).split(/[.+-]/).map(x => parseInt(x, 10) || 0)
  const len = Math.max(pa.length, pb.length)
  for (let i = 0; i < len; i++) {
    const va = pa[i] || 0, vb = pb[i] || 0
    if (va !== vb) return va > vb ? 1 : -1
  }
  return 0
}

function isBlacklisted(v) {
  const s = String(v || '').trim().toLowerCase()
  return blacklistedBuilds.some(b => s.includes(b))
}

function findAsset(assets, extensions) {
  if (!assets) return null
  return assets.find(a =>
    !blacklistedBuilds.some(b => a.name.startsWith(b)) &&
    extensions.some(e => a.name.endsWith(e))
  )
}

/**
 * Subscribe to download progress events from the Rust backend
 */
export function onDownloadProgress(callback) {
  return loading_listener((payload) => {
    if (!payload?.event) return
    if (payload.event.type !== 'launcher_update') return
    const fraction = payload.fraction
    let pct = 0
    if (fraction == null) pct = 100
    else if (fraction <= 1) pct = Math.round(fraction * 100)
    else pct = Math.min(100, Math.round(fraction))
    callback({ progress: pct, message: payload.message || '', done: fraction == null })
  })
}

/**
 * Check for update using native fetch (GitHub has CORS *)
 */
export async function getRemote(_elementId, downloadArtifact) {
  const resp = await fetch(`${GITHUB_API}/releases/latest`)
  if (!resp.ok) throw new Error(`GitHub API: ${resp.status}`)

  const data = await resp.json()
  const [currentOs, currentVersion] = await Promise.all([getOS(), getVersion()])
  os.value = currentOs

  const latestRelease = data.name || data.tag_name || ''
  localVersion.value = `v${currentVersion}`
  remoteVersion.value = latestRelease

  const cleanRemote = stripV(latestRelease)
  const cleanLocal = stripV(currentVersion)
  const lastDownloaded = stripV(getLastDownloadedVersion())
  const alreadyDownloaded = lastDownloaded && cmpVersions(cleanRemote, lastDownloaded) <= 0
  const isNewer = latestRelease && !isBlacklisted(latestRelease) && cmpVersions(cleanRemote, cleanLocal) > 0 && !alreadyDownloaded

  if (isNewer && osNames.includes(os.value.toLowerCase())) {
    const exts = getExtensions()
    const asset = findAsset(data.assets, exts)
    if (asset) {
      updateState.value = true
      allowState.value = true
    } else {
      updateState.value = false
      allowState.value = false
    }
  } else {
    updateState.value = false
    allowState.value = false
  }

  if (downloadArtifact && updateState.value) {
    installState.value = true
    updateProgress.value = 0
    updateMessage.value = 'Starting download...'

    const exts = getExtensions()
    const asset = findAsset(data.assets, exts)
    if (asset) {
      const { getArtifact } = await import('@/helpers/utils.js')
      await getArtifact(asset.browser_download_url, asset.name, os.value, true)
      installState.value = false
      updateProgress.value = 100
      updateMessage.value = 'Update downloaded.'
      updateState.value = false
      setLastDownloadedVersion(remoteVersion.value || cleanRemote)
    } else {
      installState.value = false
      throw new Error(`No installer for ${os.value}`)
    }
  }
}

async function getOS() {
  const { getOS: getOSFn } = await import('@/helpers/utils.js')
  return await getOSFn()
}

/**
 * Fetch all releases for the version list
 */
export async function fetchAllReleases() {
  const resp = await fetch(`${GITHUB_API}/releases?per_page=10`)
  if (!resp.ok) throw new Error(`GitHub API: ${resp.status}`)

  const data = await resp.json()
  allReleases.value = data

  const currentVersion = await getVersion()
  const exts = getExtensions()

  // Older versions (before current)
  const older = data
    .filter(r => {
      const tag = stripV(r.tag_name || r.name || '')
      return tag && !isBlacklisted(tag) && cmpVersions(tag, stripV(currentVersion)) < 0
    })
    .slice(0, 5)
    .map(r => {
      const asset = findAsset(r.assets, exts)
      return {
        tag: r.tag_name || r.name,
        name: r.name || r.tag_name,
        date: r.published_at,
        body: r.body || '',
        assetName: asset?.name || null,
        assetUrl: asset?.browser_download_url || null,
        size: asset?.size || 0,
      }
    })

  olderVersions.value = older
}

/**
 * Download a specific older version
 */
export async function downloadOlderVersion(version) {
  if (!version?.assetUrl) return
  const { getArtifact } = await import('@/helpers/utils.js')
  const currentOs = await getOS()
  await getArtifact(version.assetUrl, version.assetName, currentOs, true)
}
