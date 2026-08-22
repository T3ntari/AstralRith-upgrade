import { ref } from 'vue'
import { getVersion } from '@tauri-apps/api/app'
import { getArtifact, getOS } from '@/helpers/utils.js'


export const allowState = ref(false)
export const installState = ref(false)
export const updateState = ref(false)
export const latestBetaCommitTruncatedSha = ref('')
export const latestBetaCommitLink = ref('')
export const launcherUrl = 'https://www.astralium.su/get/ar'

const os = ref('')
const releaseLink = `https://api.github.com/repos/SmilerRyan/AstralRinth/releases/latest`
const branchesLink = `https://api.github.com/repos/SmilerRyan/AstralRinth/branches`
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
    if (!response.ok) throw new Error(response.status)
    const data = await response.json()

    const [currentOs, currentVersion] = await Promise.all([getOS(), getVersion()])
    os.value = currentOs

    const latestRelease = data.name
    let remoteVersion = undefined

    if (!elementIdBool) {
      const releaseData = document.getElementById('releaseData')
      if (releaseData == null) {
        console.error('Release data element not found.')
        return
      }
      releaseData.textContent = latestRelease
      remoteVersion = `${releaseData.textContent}`
    } else {
      remoteVersion = latestRelease
    }

    if (osNames.includes(os.value.toLowerCase())) {
      if (remoteVersion.startsWith('v' + currentVersion)) {
        updateState.value = false
        allowState.value = false
      } else {
        updateState.value = true
        allowState.value = true
      }
    } else {
      updateState.value = false
      allowState.value = false
    }

    console.log('Update available state is', updateState.value)
    console.log('Remote version is', remoteVersion)
    console.log('Local version is', currentVersion)
    console.log('Operating System is', os.value)

    if (downloadArtifactBool) {
      installState.value = true
      const builds = data.assets
      const fileName = getInstaller(getExtension(), builds)
      if (fileName != null) {
        await getArtifact(fileName[1], fileName[0], os.value, true)
      }
      installState.value = false
    }
  } catch (error) {
    console.error(failedFetch[0], error)
    if (!elementIdBool) {
      const errorData = document.getElementById('releaseData')
      if (errorData) {
        errorData.textContent = `${error.message}`
      }
      updateState.value = false
      allowState.value = false
      installState.value = false
    }
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
