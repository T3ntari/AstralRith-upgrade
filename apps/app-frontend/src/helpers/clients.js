/**
 * Client/instance management Tauri command wrappers.
 */
import { invoke } from '@tauri-apps/api/core'

/** Get all Minecraft instances (existing profile system) */
export async function getProfiles() {
  return await invoke('plugin:profile|profile_get_all')
}

/** Get all client profiles (module configurations) */
export async function getClientProfiles() {
  return await invoke('plugin:client-profiles|client_profile_list')
}

/** Create a client profile */
export async function createClientProfile({ name, modules, keybinds, hud, render, versionCompatibility }) {
  return await invoke('plugin:client-profiles|client_profile_create', {
    name,
    modules: modules ? JSON.stringify(modules) : null,
    keybinds: keybinds ? JSON.stringify(keybinds) : null,
    hud: hud ? JSON.stringify(hud) : null,
    render: render ? JSON.stringify(render) : null,
    versionCompatibility: versionCompatibility ? JSON.stringify(versionCompatibility) : null,
  })
}

/** Update a client profile */
export async function updateClientProfile({ id, name, modules, keybinds, hud, render, versionCompatibility, active }) {
  return await invoke('plugin:client-profiles|client_profile_update', {
    id,
    name: name || null,
    modules: modules ? JSON.stringify(modules) : null,
    keybinds: keybinds ? JSON.stringify(keybinds) : null,
    hud: hud ? JSON.stringify(hud) : null,
    render: render ? JSON.stringify(render) : null,
    versionCompatibility: versionCompatibility ? JSON.stringify(versionCompatibility) : null,
    active: active !== undefined ? active : null,
  })
}

/** Delete a client profile */
export async function deleteClientProfile(id) {
  return await invoke('plugin:client-profiles|client_profile_delete', { id })
}

/** Duplicate a client profile */
export async function duplicateClientProfile(id) {
  return await invoke('plugin:client-profiles|client_profile_duplicate', { id })
}

/** Get a single profile by path */
export async function getProfile(path) {
  return await invoke('plugin:profile|profile_get', { path })
}

/** Get all running processes */
export async function getAllProcesses() {
  return await invoke('plugin:process|process_get_all')
}

/** Get processes by profile path */
export async function getProcessesByProfile(path) {
  return await invoke('plugin:process|process_get_by_profile_path', { path })
}

/** Kill a process */
export async function killProcess(uuid) {
  return await invoke('plugin:process|process_kill', { uuid })
}

/** Wait for a process to complete */
export async function waitForProcess(uuid) {
  return await invoke('plugin:process|process_wait_for', { uuid })
}

/** Launch a profile */
export async function launchProfile(path) {
  return await invoke('plugin:profile|profile_run', { path })
}

/** Remove a profile */
export async function removeProfile(path) {
  return await invoke('plugin:profile|profile_remove', { path })
}

/** Duplicate a profile */
export async function duplicateProfile(path) {
  return await invoke('plugin:profile-create|profile_duplicate', { path })
}

/** Get logs for a profile */
export async function getLogs(path) {
  return await invoke('plugin:logs|logs_get', { path })
}

/** Get instance settings */
export async function getSettings(path) {
  return await invoke('plugin:settings|settings_get', { path })
}
