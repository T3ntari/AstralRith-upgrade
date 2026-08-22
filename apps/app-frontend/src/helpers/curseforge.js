import { fetch as tauriFetch } from '@tauri-apps/plugin-http'

const CF_BASE = 'https://api.curseforge.com/v1'
const CF_GAME_ID = 432

export const CF_CLASS_IDS = {
  mod: 6,
  modpack: 4471,
  resourcepack: 12,
  shader: 6552,
  datapack: 6945,
}

export const CF_LOADER_IDS = {
  any: 0,
  forge: 1,
  liteloader: 3,
  fabric: 4,
  quilt: 5,
  neoforge: 6,
  every: 7,
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

export const CF_SORT_FIELDS = {
  1: 'Featured',
  2: 'Popularity',
  3: 'Last updated',
  4: 'Name',
  6: 'Total downloads',
  7: 'Recently updated',
  8: 'Recently added',
}

const CF_MODE_KEY = 'ar.cf_mode'
const CF_API_KEY = 'ar.cf_api_key'

export function getCfMode() {
  return localStorage.getItem(CF_MODE_KEY) === 'true'
}

export function setCfMode(enabled) {
  localStorage.setItem(CF_MODE_KEY, enabled ? 'true' : 'false')
  document.documentElement.classList.toggle('cf-mode', enabled)
}

export function getCfApiKey() {
  return localStorage.getItem(CF_API_KEY) || ''
}

export function setCfApiKey(key) {
  localStorage.setItem(CF_API_KEY, key.trim())
}

export function clearCfApiKey() {
  localStorage.removeItem(CF_API_KEY)
}

export function classIdForProjectType(projectType) {
  return CF_CLASS_IDS[projectType] ?? CF_CLASS_IDS.mod
}

export function loaderIdForPlatform(platform) {
  if (!platform) return CF_LOADER_IDS.any
  const name = platform.toLowerCase()
  if (name === 'forge') return CF_LOADER_IDS.forge
  if (name === 'fabric') return CF_LOADER_IDS.fabric
  if (name === 'quilt') return CF_LOADER_IDS.quilt
  if (name === 'neoforge') return CF_LOADER_IDS.neoforge
  return CF_LOADER_IDS.any
}

async function cfRequest(path, { params = {}, method = 'GET', body, raw = false } = {}) {
  const apiKey = getCfApiKey()
  if (!apiKey) {
    throw new Error('No CurseForge API key configured')
  }

  const url = new URL(`${CF_BASE}${path}`)
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== null && value !== '') {
      url.searchParams.set(key, String(value))
    }
  }

  const headers = {
    'x-api-key': apiKey,
    Accept: 'application/json',
  }

  const response = await tauriFetch(url.toString(), {
    method,
    headers,
    body: body ? JSON.stringify(body) : undefined,
  })

  if (!response.ok) {
    if (response.status === 403) {
      throw new Error('CurseForge rejected the API key. Make sure it is valid and enabled.')
    }
    throw new Error(`CurseForge API error: ${response.status}`)
  }

  const data = await response.json()
  return raw ? data : data.data
}

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
  const params = {
    gameId: CF_GAME_ID,
    classId: classIdForProjectType(projectType),
    index: page * pageSize,
    pageSize,
    searchFilter: query,
    sortField,
    sortOrder,
  }
  if (gameVersion) params.gameVersion = gameVersion
  if (modLoaderType && modLoaderType !== CF_LOADER_IDS.any) {
    params.modLoaderType = modLoaderType
  }
  if (categoryId) params.categoryId = categoryId

  const response = await cfRequest('/mods/search', { params, raw: true })
  return {
    results: response.data ?? [],
    totalCount: response.pagination?.totalCount ?? response.data?.length ?? 0,
  }
}

export async function getCategories(classId) {
  return await cfRequest('/categories', {
    params: { gameId: CF_GAME_ID, classId },
  })
}

export async function getGameVersions() {
  const versions = await cfRequest('/minecraft/version')
  return versions
    .filter((v) => v.versionType === 'release')
    .map((v) => v.versionString)
    .filter((v) => !v.includes('_'))
}

export async function getModloaders() {
  const loaders = await cfRequest('/minecraft/modloader')
  return loaders
}

export async function getModFiles(
  modId,
  { gameVersion = '', modLoaderType = 0, pageSize = 50 } = {},
) {
  const params = { modId, pageSize }
  if (gameVersion) params.gameVersion = gameVersion
  if (modLoaderType && modLoaderType !== CF_LOADER_IDS.any) {
    params.modLoaderType = modLoaderType
  }
  return await cfRequest(`/mods/${modId}/files`, { params })
}

export async function getFileDownloadUrl(modId, fileId) {
  return await cfRequest(`/mods/${modId}/files/${fileId}/download-url`)
}

export function projectUrl(mod) {
  return `https://www.curseforge.com/minecraft/${mod.classId === 4471 ? 'modpacks' : 'mc-mods'}/${mod.slug}`
}

export function formatDownloadCount(count) {
  if (!count && count !== 0) return '0'
  if (count >= 1_000_000_000) return (count / 1_000_000_000).toFixed(1) + 'B'
  if (count >= 1_000_000) return (count / 1_000_000).toFixed(1) + 'M'
  if (count >= 1_000) return (count / 1_000).toFixed(1) + 'K'
  return String(count)
}

export function cfResultToCard(mod, projectType) {
  const authors = mod.authors?.map((a) => a.name) ?? []
  const categories = mod.categories?.map((c) => ({ name: c.name, slug: c.slug })) ?? []

  return {
    project_id: String(mod.id),
    id: String(mod.id),
    slug: mod.slug,
    title: mod.name,
    description: mod.summary,
    icon_url: mod.logo?.url,
    author: authors.join(', '),
    downloads: mod.downloadCount,
    follows: mod.thumbnails?.length ? mod.downloadCount : 0,
    project_type: projectType,
    display_categories: categories.map((c) => c.name),
    versions: mod.gameVersions ?? [],
    client_side: 'optional',
    server_side: 'optional',
    date_created: mod.dateCreated,
    date_modified: mod.dateModified,
    cfId: mod.id,
    cfGameVersions: mod.gameVersions ?? [],
  }
}
