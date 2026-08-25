/**
 * MCHeads skin browsing service (mcheads.org API).
 *
 * Free, no API key. Endpoints:
 *   avatar/<name|uuid>/<left|right>/<size>  -> full body render PNG
 *   head/<name|uuid>/<size>                  -> head render PNG
 *   skin/<name|uuid>                         -> raw 64x64 skin PNG
 */
import { fetch as tauriFetch } from '@tauri-apps/plugin-http'

export const MCHEADS_API = 'https://api.mcheads.org'

// Curated famous players to seed the browse grid (verified premium accounts)
export const FEATURED_PLAYERS = [
  'Notch', 'jeb_', 'Dinnerbone', 'Grumm', 'Searge', 'Dream', 'Technoblade',
  'Ph1LzA', 'GeorgeNotFound', 'Sapnap', 'TommyInnit', 'Tubbo', 'WilburSoot',
  'Ranboo', 'Fundy', 'Nihachu', 'KarlJacobs', 'Quackity', 'Skeppy', 'BadBoyHalo',
  'Punz', 'Purpled', 'Antfrost', 'Awesamdude', 'Eret', 'Hbomb94', 'CaptainSparklez',
  'DanTDM', 'Ssundee', 'PrestonPlayz', 'JeromeASF', 'BajanCanadian', 'Graser',
  'Tentari', 'Dantdm', 'Xisumavoid', 'Grian', 'MumboJumbo', 'Scar', 'PearlescentMoon',
]

export function avatarUrl(nameOrUuid, size = 128, side = 'left') {
  return `${MCHEADS_API}/avatar/${encodeURIComponent(nameOrUuid)}/${side}/${size}`
}

export function headUrl(nameOrUuid, size = 64) {
  return `${MCHEADS_API}/head/${encodeURIComponent(nameOrUuid)}/${size}`
}

export function skinUrl(nameOrUuid) {
  return `${MCHEADS_API}/skin/${encodeURIComponent(nameOrUuid)}`
}

/**
 * Fetch the raw skin PNG bytes for a player (for saving/applying).
 */
export async function fetchSkinBytes(nameOrUuid) {
  try {
    const resp = await tauriFetch(skinUrl(nameOrUuid))
    if (!resp.ok) throw new Error(`Skin fetch failed (${resp.status})`)
    const blob = await resp.blob()
    return new Uint8Array(await blob.arrayBuffer())
  } catch (err) {
    throw err
  }
}

/**
 * Validate that a username is a real premium Minecraft account using
 * the Mojang API (profile lookup). Returns { name, uuid } or null.
 */
export async function lookupMojangProfile(nameOrUuid) {
  try {
    const isUuid = /^[0-9a-f]{32}$/i.test(nameOrUuid.replace(/-/g, ''))
    const url = isUuid
      ? `https://api.mojang.com/user/profiles/${nameOrUuid.replace(/-/g, '')}`
      : `https://api.mojang.com/users/profiles/minecraft/${encodeURIComponent(nameOrUuid)}`

    const controller = new AbortController()
    const timer = setTimeout(() => controller.abort(), 10000)
    let resp
    try {
      resp = await tauriFetch(url, { signal: controller.signal })
    } finally {
      clearTimeout(timer)
    }

    if (!resp.ok) {
      if (resp.status === 204 || resp.status === 404) return null
      throw new Error(`Mojang API error (${resp.status})`)
    }
    const data = await resp.json()
    if (!data?.id) return null
    return { name: data.name, uuid: data.id }
  } catch (err) {
    if (err?.name === 'AbortError') throw new Error('Mojang API timed out')
    throw err
  }
}
