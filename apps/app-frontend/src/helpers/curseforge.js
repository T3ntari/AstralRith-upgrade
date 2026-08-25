/**
 * CurseForge provider service (frontend).
 *
 * All CurseForge API traffic goes through Tauri commands backed by the Rust
 * provider layer. The application credential never enters frontend state,
 * localStorage, URLs, logs, or analytics.
 *
 * The frontend only ever sees normalized UnifiedProject / UnifiedFile models.
 */
import { invoke } from '@tauri-apps/api/core'

const CF_MODE_KEY = 'ar.cf_mode'

export function getCfMode() {
  return localStorage.getItem(CF_MODE_KEY) === 'true'
}

export function setCfMode(enabled) {
  localStorage.setItem(CF_MODE_KEY, enabled ? 'true' : 'false')
  document.documentElement.classList.toggle('cf-mode', enabled)
}

// Legacy localStorage key — removed: the credential never lives in the
// browser. Kept as a no-op so any leftover code path cannot read a stale key.
const LEGACY_CF_API_KEY = 'ar.cf_api_key'

export function getCfApiKey() {
  return ''
}

export function setCfApiKey() {
  // Intentionally a no-op: API keys are infrastructure, not frontend state.
}

export function clearCfApiKey() {
  try {
    localStorage.removeItem(LEGACY_CF_API_KEY)
  } catch {
    /* ignore */
  }
}

export const CF_SORT_FIELDS = {
  1: 'Featured',
  2: 'Popularity',
  3: 'Last updated',
  4: 'Name',
  5: 'Author',
  6: 'Total downloads',
  7: 'Recently updated',
  8: 'Recently added',
}

export const CF_LOADER_NAMES = {
  0: 'Any loader',
  1: 'Forge',
  3: 'LiteLoader',
  4: 'Fabric',
  5: 'Quilt',
  6: 'NeoForge',
  7: 'Every loader',
}

export function classIdForProjectType(projectType) {
  const ids = {
    mod: 6,
    modpack: 4471,
    resourcepack: 12,
    shader: 6552,
    datapack: 6945,
  }
  return ids[projectType] ?? 6
}

export function loaderIdForPlatform(platform) {
  if (!platform) return 0
  const name = String(platform).toLowerCase()
  if (name === 'forge') return 1
  if (name === 'fabric') return 4
  if (name === 'quilt') return 5
  if (name === 'neoforge') return 6
  return 0
}

/**
 * Search CurseForge projects (normalized results + real pagination).
 */
export async function searchMods({
  projectType,
  query = '',
  page = 0,
  pageSize = 20,
  sortField = 2,
  sortOrder = 'desc',
  gameVersion = '',
  modLoaderType = 0,
  categoryId = undefined,
}) {
  return await invoke('plugin:cf|cf_search', {
    projectType,
    query,
    page,
    pageSize,
    sortField,
    sortOrder,
    gameVersion,
    modLoaderType,
    categoryId,
  })
}

export async function getCategories(classId) {
  return await invoke('plugin:cf|cf_get_categories', { classId })
}

export async function getGameVersions() {
  return await invoke('plugin:cf|cf_get_game_versions')
}

export async function getModloaders() {
  return await invoke('plugin:cf|cf_get_mod_loaders')
}

export async function getModFiles(
  modId,
  { gameVersion = '', modLoaderType = 0, pageSize = 50 } = {},
) {
  return await invoke('plugin:cf|cf_get_project_files', {
    projectId: modId,
    gameVersion: gameVersion || null,
    modLoaderType: modLoaderType || null,
    pageSize,
  })
}

/** Compatibility-filtered + sorted files for the version picker. */
export async function getCompatibleFiles(
  modId,
  { minecraftVersion, loader = '', releaseType = '', pageSize = 50 } = {},
) {
  return await invoke('plugin:cf|cf_get_compatible_files', {
    projectId: modId,
    minecraftVersion,
    loader: loader || null,
    releaseType: releaseType || null,
    pageSize,
  })
}

export async function getFile(modId, fileId) {
  return await invoke('plugin:cf|cf_get_file', { projectId: modId, fileId })
}

export async function getProject(projectId) {
  return await invoke('plugin:cf|cf_get_project', { projectId })
}

export async function getProjectBySlug(slug, projectType) {
  return await invoke('plugin:cf|cf_get_project_by_slug', { slug, projectType })
}

export async function getFileDownloadUrl(modId, fileId) {
  return await invoke('plugin:cf|cf_resolve_download', { projectId: modId, fileId })
}

export function projectUrl(mod, fileId, isModpack = false) {
  const section = isModpack || Number(mod.class_id) === 4471 ? 'modpacks' : 'mc-mods'
  const base = `https://www.curseforge.com/minecraft/${section}/${mod.slug}`
  return fileId ? `${base}/files/${fileId}` : base
}

export function formatDownloadCount(count) {
  if (!count && count !== 0) return '0'
  if (count >= 1_000_000_000) return (count / 1_000_000_000).toFixed(1) + 'B'
  if (count >= 1_000_000) return (count / 1_000_000).toFixed(1) + 'M'
  if (count >= 1_000) return (count / 1_000).toFixed(1) + 'K'
  return String(count)
}

/**
 * Parses a CurseForge project URL into { slug, projectType } when possible.
 * Supports:
 *   https://www.curseforge.com/minecraft/mc-mods/<slug>/...
 *   https://www.curseforge.com/minecraft/modpacks/<slug>/...
 * Returns null for unsupported URLs.
 */
export function parseCurseForgeUrl(url) {
  try {
    const u = new URL(url)
    const parts = u.pathname.split('/').filter(Boolean)
    // ["minecraft", "mc-mods" | "modpacks", "<slug>", ...]
    if (parts[0] !== 'minecraft') return null
    const kind = parts[1]
    const slug = parts[2]
    if (!slug) return null
    if (kind === 'mc-mods') return { slug, projectType: 'mod' }
    if (kind === 'modpacks') return { slug, projectType: 'modpack' }
    return null
  } catch {
    return null
  }
}

/** Resolve a CurseForge URL to a normalized project via the Rust provider. */
export async function projectFromUrl(url) {
  const parsed = parseCurseForgeUrl(url)
  if (!parsed) return null
  return await invoke('plugin:cf|cf_get_project_by_slug', {
    slug: parsed.slug,
    projectType: parsed.projectType,
  })
}

/** Installed CurseForge pack identity for an instance (or null). */
export async function installedPack(profilePath) {
  return await invoke('plugin:cf|cf_installed_pack', { profilePath })
}

/** Whether a newer compatible CurseForge pack file exists. Returns [fileId, date] or null. */
export async function checkPackUpdate(profilePath) {
  return await invoke('plugin:cf|cf_check_pack_update', { profilePath })
}
