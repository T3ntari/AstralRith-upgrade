import { ref } from 'vue'
import { getVersion } from '@tauri-apps/api/app'
import { getArtifact, getOS } from '@/helpers/utils.js'


export const allowState = ref(false)
export const installState = ref(false)
export const updateState = ref(false)
export const remoteVersion = ref('')
export const localVersion = ref('')
export const updateProgress = ref(0)
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

    const isNewer =
      latestRelease &&
      !latestRelease.startsWith(`v${currentVersion}`) &&
      latestRelease !== currentVersion

    if (osNames.includes(os.value.toLowerCase()) && isNewer) {
      updateState.value = true
      allowState.value = true
    } else {
      updateState.value = false
      allowState.value = false
    }

    console.log('[AR] Update check — local:', localVersion.value, 'remote:', remoteVersion.value, 'available:', updateState.value)

    if (downloadArtifactBool && updateState.value) {
      installState.value = true
      updateProgress.value = 0
      const builds = data.assets || []
      const fileName = getInstaller(getExtension(), builds)
      if (fileName != null) {
        await getArtifact(fileName[1], fileName[0], os.value, true)
      }
      installState.value = false
      updateProgress.value = 100
      updateState.value = false
    }
  } catch (error) {
    console.error(failedFetch[0], error)
    updateState.value = false
    allowState.value = false
    installState.value = false
  }
}

function getInstaller(osExtension, builds) {
  for (const build of builds) {
    let blacklisted = false
    for (const item of blacklistedBuilds) {
      if (build.name.startsWith(item)) {
        blacklisted = true
        break
      }
    }
    if (build.name.endsWith(osExtension) && !blacklisted) {
      console.log(build.browser_download_url)
      return [build.name, build.browser_download_url]
    }
  }
  return null
}

function getExtension() {
  const osLower = os.value.toLowerCase()
  if (osLower === osNames[0]) return macExtension
  if (osLower === osNames[1]) return windowsExtension
  return null
}
