import { ref } from 'vue'
import { getVersion } from '@tauri-apps/api/app'
import { getArtifact, getOS } from '@/helpers/utils.js'
import { loading_listener } from '@/helpers/events.js'

export const allowState = ref(false)
export const installState = ref(false)
export const updateState = ref(false)
export const remoteVersion = ref('')
export const localVersion = ref('')
export const updateProgress = ref(0)
export const updateMessage = ref('')
export const latestBetaCommitTruncatedSha = ref('')
export const latestBetaCommitLink = ref('')
export const launcherUrl = 'https://github.com/T3ntari/AstralRith-upgrade/releases'

const os = ref('')
const REPO = 'T3ntari/AstralRith-upgrade'
const releaseLink = `https://api.github.com/repos/${REPO}/releases/latest`
const branchesLink = `https://api.github.com/repos/${REPO}/branches`
const failedFetch = [`Failed to fetch remote releases:`, `Failed to fetch remote commits:`]
const betaBranch = `beta`
const osNames = ['macos', 'windows', 'linux']
const macExtension = `.dmg`
const windowsExtension = `.msi`
// Linux ships .deb (Debian/Ubuntu/Mint) and .rpm (Fedora). Prefer .deb.
const linuxExtensions = [`.deb`, `.rpm`, `.AppImage`]
const blacklistedBuilds = [
  `dev`,
  `nightly`,
  `dirty`,
  `dirty-dev`,
  `dirty-nightly`,
  `dirty_dev`,
  `dirty_nightly`,
]

/**
 * Fetches branches and their latest commit info from GitHub.
 */
export async function getBranches() {
  try {
    const response = await fetch(branchesLink)
    if (!response.ok) throw new Error(response.status)
    const data = await response.json()

    await Promise.all(
      data.map(async (branch) => {
        try {
          const resp = await fetch(branch.commit.url)
          if (!resp.ok) throw new Error(resp.status)
          const commitData = await resp.json()
          if (branch.name.toLowerCase() === betaBranch) {
            latestBetaCommitTruncatedSha.value = commitData.sha.slice(0, 7)
            latestBetaCommitLink.value = commitData.html_url
          }
        } catch (err) {
          console.error(failedFetch[1], err)
        }
      }),
    )
  } catch (error) {
    latestBetaCommitTruncatedSha.value = error.message
    latestBetaCommitLink.value = undefined
    console.error(failedFetch[1], error)
  }
}

/**
 * Subscribes to real download progress events emitted by the Rust backend
 * (LoadingBarType::LauncherUpdate). Returns an unsubscribe function.
 */
export function onDownloadProgress(callback) {
  return loading_listener((payload) => {
    if (!payload?.event) return
    if (payload.event.type !== 'launcher_update') return
    const fraction = payload.fraction
    let pct = 0
    if (fraction == null) {
      // null fraction => bar completed
      pct = 100
    } else if (fraction <= 1) {
      pct = Math.round(fraction * 100)
    } else {
      pct = Math.min(100, Math.round(fraction))
    }
    callback({
      progress: pct,
      message: payload.message || '',
      done: fraction == null,
    })
  })
}

/**
 * Fetches remote release data and handles updates/downloads.
 *
 * @param {boolean} elementIdBool - Whether to update the DOM element.
 * @param {boolean} downloadArtifactBool - Whether to download the artifact.
 */
export async function getRemote(elementIdBool, downloadArtifactBool) {
  try {
    const response = await fetch(releaseLink)
    if (!response.ok) throw new Error(`HTTP ${response.status}`)
    const data = await response.json()

    const [currentOs, currentVersion] = await Promise.all([getOS(), getVersion()])
    os.value = currentOs

    const latestRelease = data.name || data.tag_name || ''
    localVersion.value = `v${currentVersion}`
    remoteVersion.value = latestRelease

    // proper semver compare (strip v, compare numerically)
    const stripV = (v) => String(v).trim().replace(/^v/i, '')
    const cmpVersions = (a, b) => {
      const pa = stripV(a).split(/[.+-]/).map((x) => parseInt(x, 10) || 0)
      const pb = stripV(b).split(/[.+-]/).map((x) => parseInt(x, 10) || 0)
      const len = Math.max(pa.length, pb.length)
      for (let i = 0; i < len; i++) {
        const va = pa[i] || 0
        const vb = pb[i] || 0
        if (va !== vb) return va > vb ? 1 : -1
      }
      return 0
    }
    const cleanRemote = stripV(latestRelease)
    const cleanLocal = stripV(currentVersion)
    const isBlacklisted = blacklistedBuilds.some((b) =>
      cleanRemote.toLowerCase().startsWith(b.toLowerCase()),
    )
    const isNewer =
      latestRelease && !isBlacklisted && cleanRemote !== cleanLocal && cmpVersions(cleanRemote, cleanLocal) > 0

    // Require an actual installer asset for this OS (any supported ext).
    if (isNewer && osNames.includes(os.value.toLowerCase())) {
      const exts = getExtensions()
      const hasAsset = (data.assets || []).some(
        (a) => !blacklistedBuilds.some((b) => a.name.startsWith(b)) && exts.some((e) => a.name.endsWith(e)),
      )
      if (hasAsset) {
        updateState.value = true
        allowState.value = true
      } else {
        console.log(
          `[AR] newer version ${latestRelease} found but no ${exts.join('/')} asset for ${os.value} - suppressing badge`,
        )
        updateState.value = false
        allowState.value = false
      }
    } else {
      updateState.value = false
      allowState.value = false
    }

    console.log(
      '[AR] Update check — local:',
      localVersion.value,
      'remote:',
      remoteVersion.value,
      'available:',
      updateState.value,
    )

    if (downloadArtifactBool && updateState.value) {
      installState.value = true
      updateProgress.value = 0
      updateMessage.value = 'Starting download...'
      const builds = data.assets || []
      const fileName = getInstaller(getExtensions(), builds)
      if (fileName != null) {
        await getArtifact(fileName[1], fileName[0], os.value, true)
        installState.value = false
        updateProgress.value = 100
        updateMessage.value = 'Update downloaded.'
        updateState.value = false
      } else {
        console.error(
          `No installer found for OS ${os.value} with extensions ${getExtensions().join('/')}`,
          builds.map((b) => b.name),
        )
        installState.value = false
        throw new Error(
          `No installer found for ${os.value} (${getExtensions().join('/') || 'unknown extension'}). Check releases page: ${launcherUrl}`,
        )
      }
    }
  } catch (error) {
    console.error(failedFetch[0], error)
    updateState.value = false
    allowState.value = false
    installState.value = false
  }
}

function getInstaller(osExtensions, builds) {
  for (const build of builds) {
    let blacklisted = false
    for (const item of blacklistedBuilds) {
      if (build.name.startsWith(item)) {
        blacklisted = true
        break
      }
    }
    if (blacklisted) continue
    for (const ext of osExtensions) {
      if (build.name.endsWith(ext)) {
        console.log(build.browser_download_url)
        return [build.name, build.browser_download_url]
      }
    }
  }
  return null
}

export function getExtensions() {
  const osLower = os.value.toLowerCase()
  if (osLower === osNames[0]) return [macExtension]
  if (osLower === osNames[1]) return [windowsExtension]
  if (osLower === osNames[2]) return linuxExtensions
  return []
}
