/**
 * Skin management service — all Tauri command wrappers.
 */
import { invoke } from '@tauri-apps/api/core'
import { fetch as tauriFetch } from '@tauri-apps/plugin-http'

export async function listSkins() {
  return await invoke('plugin:skins|skins_list')
}

export async function addSkin({ name, bytes, modelType, source, accountId, favorite }) {
  return await invoke('plugin:skins|skins_add', {
    name,
    bytes,
    modelType: modelType || 'classic',
    source: source || 'local',
    accountId: accountId || null,
    favorite: favorite || false,
  })
}

export async function deleteSkin(id) {
  return await invoke('plugin:skins|skins_delete', { id })
}

export async function updateSkin({ id, name, favorite, modelType, sortOrder, accountId }) {
  return await invoke('plugin:skins|skins_update', {
    id,
    name: name || null,
    favorite: favorite !== undefined ? favorite : null,
    modelType: modelType || null,
    sortOrder: sortOrder !== undefined ? sortOrder : null,
    accountId: accountId !== undefined ? accountId : null,
  })
}

export async function duplicateSkin(id) {
  return await invoke('plugin:skins|skins_duplicate', { id })
}

export async function reorderSkins(ids) {
  return await invoke('plugin:skins|skins_reorder', { ids })
}

export async function markSkinUsed(id) {
  return await invoke('plugin:skins|skins_mark_used', { id })
}

export async function validateSkin({ name, bytes }) {
  return await invoke('plugin:skins|skins_validate', { name: name || '', bytes })
}

/**
 * Convert a File (from <input> or drag/drop) to a byte array.
 */
export async function fileToBytes(file) {
  const buffer = await file.arrayBuffer()
  return new Uint8Array(buffer)
}

/**
 * Fetch a Minecraft player's skin by username or UUID using Mojang API.
 * Returns { name, uuid, skinUrl, skinBytes, modelType } or null.
 */
function fetchWithTimeout(url, ms = 10000) {
  const controller = new AbortController()
  const timer = setTimeout(() => controller.abort(), ms)
  return tauriFetch(url, { signal: controller.signal }).finally(() => clearTimeout(timer))
}

export async function lookupPlayerSkin(identifier) {
  try {
    // Determine if it's a UUID (with or without dashes) or username
    const isUuid = /^[0-9a-f]{32}$/i.test(identifier.replace(/-/g, ''))

    let profileUrl
    if (isUuid) {
      const uuid = identifier.replace(/-/g, '')
      profileUrl = `https://api.mojang.com/user/profiles/${uuid}`
    } else {
      profileUrl = `https://api.mojang.com/users/profiles/minecraft/${encodeURIComponent(identifier)}`
    }

    const profileResp = await fetchWithTimeout(profileUrl)
    if (!profileResp.ok) {
      if (profileResp.status === 204 || profileResp.status === 404) {
        throw new Error(`Player "${identifier}" not found`)
      }
      throw new Error(`Mojang API error (${profileResp.status})`)
    }

    const profile = await profileResp.json()
    if (!profile.id || !profile.name) {
      throw new Error('Invalid profile data')
    }

    // Fetch skin texture
    const sessionUrl = `https://sessionserver.mojang.com/session/minecraft/profile/${profile.id}`
    const sessionResp = await fetchWithTimeout(sessionUrl)
    if (!sessionResp.ok) {
      throw new Error('Could not fetch player profile data')
    }

    const session = await sessionResp.json()
    if (!session.properties) {
      return { name: profile.name, uuid: profile.id, skinUrl: null, modelType: 'classic' }
    }

    const textureProp = session.properties.find(p => p.name === 'textures')
    if (!textureProp) {
      return { name: profile.name, uuid: profile.id, skinUrl: null, modelType: 'classic' }
    }

    const textureData = JSON.parse(atob(textureProp.value))
    const skinEntry = textureData.textures?.SKIN
    if (!skinEntry) {
      return { name: profile.name, uuid: profile.id, skinUrl: null, modelType: 'classic' }
    }

    const modelType = skinEntry.metadata?.model === 'slim' ? 'slim' : 'classic'
    const skinUrl = skinEntry.url

    // Download the skin PNG
    const skinResp = await fetchWithTimeout(skinUrl)
    if (!skinResp.ok) {
      return { name: profile.name, uuid: profile.id, skinUrl, modelType }
    }

    const blob = await skinResp.blob()
    const buffer = await blob.arrayBuffer()
    const skinBytes = new Uint8Array(buffer)

    return { name: profile.name, uuid: profile.id, skinUrl, skinBytes, modelType }
  } catch (err) {
    throw err
  }
}
