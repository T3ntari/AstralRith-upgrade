import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { handleError } from '@/store/notifications.js'

/**
 * Module categories matching utility client conventions.
 */
export const MODULE_CATEGORIES = [
  { id: 'combat', name: 'Combat', icon: 'Sword' },
  { id: 'movement', name: 'Movement', icon: 'Run' },
  { id: 'player', name: 'Player', icon: 'User' },
  { id: 'render', name: 'Render', icon: 'Eye' },
  { id: 'world', name: 'World', icon: 'Globe' },
  { id: 'exploit', name: 'Exploit', icon: 'Bug' },
  { id: 'automation', name: 'Automation', icon: 'Cog' },
  { id: 'misc', name: 'Misc', icon: 'MoreHorizontal' },
  { id: 'client', name: 'Client', icon: 'Settings' },
]

/**
 * Setting types for the generic setting renderer.
 */
export const SETTING_TYPES = {
  BOOLEAN: 'boolean',
  INTEGER: 'integer',
  FLOAT: 'float',
  ENUM: 'enum',
  COLOR: 'color',
  KEYBIND: 'keybind',
  STRING: 'string',
  MULTI_SELECT: 'multi_select',
  SLIDER: 'slider',
  ACTION: 'action',
}

/**
 * Built-in module definitions.
 */
const BUILTIN_MODULES = [
  // Combat
  {
    id: 'killaura',
    name: 'KillAura',
    description: 'Automatically targets and attacks nearby entities.',
    category: 'combat',
    enabled: false,
    keybind: null,
    settings: [
      { id: 'range', name: 'Range', type: SETTING_TYPES.SLIDER, value: 4.0, min: 1.0, max: 6.0, step: 0.1 },
      { id: 'cps', name: 'CPS', type: SETTING_TYPES.INTEGER, value: 10, min: 1, max: 20 },
      { id: 'mode', name: 'Mode', type: SETTING_TYPES.ENUM, value: 'single', options: ['single', 'switch', 'multi'] },
      { id: 'players', name: 'Target Players', type: SETTING_TYPES.BOOLEAN, value: true },
      { id: 'mobs', name: 'Target Mobs', type: SETTING_TYPES.BOOLEAN, value: true },
      { id: 'animals', name: 'Target Animals', type: SETTING_TYPES.BOOLEAN, value: false },
    ],
    supportedVersions: ['1.8.x', '1.12.x', '1.16.x', '1.20.x', '1.21.x'],
    tags: ['combat', 'pvp', 'attack', 'aura'],
    compatibility: { multiplayer: true, singleplayer: true },
  },
  {
    id: 'velocity',
    name: 'Velocity',
    description: 'Modifies knockback velocity received.',
    category: 'combat',
    enabled: false,
    keybind: null,
    settings: [
      { id: 'horizontal', name: 'Horizontal %', type: SETTING_TYPES.SLIDER, value: 0, min: 0, max: 100, step: 1 },
      { id: 'vertical', name: 'Vertical %', type: SETTING_TYPES.SLIDER, value: 0, min: 0, max: 100, step: 1 },
    ],
    supportedVersions: ['1.8.x', '1.12.x', '1.16.x', '1.20.x', '1.21.x'],
    tags: ['combat', 'pvp', 'knockback', 'vh'],
    compatibility: { multiplayer: true, singleplayer: true },
  },
  {
    id: 'autototem',
    name: 'AutoTotem',
    description: 'Automatically equips totems in the offhand.',
    category: 'combat',
    enabled: false,
    keybind: null,
    settings: [
      { id: 'health_threshold', name: 'Health Threshold', type: SETTING_TYPES.SLIDER, value: 8, min: 1, max: 20, step: 1 },
    ],
    supportedVersions: ['1.16.x', '1.20.x', '1.21.x'],
    tags: ['combat', 'totem', 'defense'],
    compatibility: { multiplayer: true, singleplayer: true },
  },
  // Movement
  {
    id: 'sprint',
    name: 'AutoSprint',
    description: 'Automatically sprints in the direction you are moving.',
    category: 'movement',
    enabled: false,
    keybind: null,
    settings: [],
    supportedVersions: ['1.8.x', '1.12.x', '1.16.x', '1.20.x', '1.21.x'],
    tags: ['movement', 'sprint', 'speed'],
    compatibility: { multiplayer: true, singleplayer: true },
  },
  {
    id: 'speed',
    name: 'Speed',
    description: 'Increases movement speed.',
    category: 'movement',
    enabled: false,
    keybind: null,
    settings: [
      { id: 'speed_multiplier', name: 'Speed Multiplier', type: SETTING_TYPES.SLIDER, value: 1.5, min: 1.0, max: 5.0, step: 0.1 },
      { id: 'mode', name: 'Mode', type: SETTING_TYPES.ENUM, value: 'vanilla', options: ['vanilla', 'strafe', 'bhop'] },
    ],
    supportedVersions: ['1.8.x', '1.12.x', '1.16.x', '1.20.x', '1.21.x'],
    tags: ['movement', 'speed', 'hack'],
    compatibility: { multiplayer: true, singleplayer: true },
  },
  {
    id: 'fly',
    name: 'Fly',
    description: 'Allows creative-style flying in survival.',
    category: 'movement',
    enabled: false,
    keybind: null,
    settings: [
      { id: 'speed', name: 'Speed', type: SETTING_TYPES.SLIDER, value: 2.0, min: 0.5, max: 10.0, step: 0.5 },
      { id: 'mode', name: 'Mode', type: SETTING_TYPES.ENUM, value: 'vanilla', options: ['vanilla', 'packet', 'static'] },
    ],
    supportedVersions: ['1.8.x', '1.12.x', '1.16.x', '1.20.x', '1.21.x'],
    tags: ['movement', 'fly', 'hack'],
    compatibility: { multiplayer: true, singleplayer: true },
  },
  // Player
  {
    id: 'autototem',
    name: 'AutoTotem',
    description: 'Automatically equips totems of undying in the offhand slot.',
    category: 'player',
    enabled: false,
    keybind: null,
    settings: [
      { id: 'health_threshold', name: 'Health Threshold', type: SETTING_TYPES.SLIDER, value: 8, min: 1, max: 20, step: 1 },
    ],
    supportedVersions: ['1.16.x', '1.20.x', '1.21.x'],
    tags: ['player', 'totem', 'defense', 'survival'],
    compatibility: { multiplayer: true, singleplayer: true },
  },
  {
    id: 'cheststealer',
    name: 'ChestStealer',
    description: 'Automatically takes items from opened chests.',
    category: 'player',
    enabled: false,
    keybind: null,
    settings: [
      { id: 'delay', name: 'Delay (ms)', type: SETTING_TYPES.INTEGER, value: 100, min: 0, max: 1000 },
      { id: 'filter', name: 'Item Filter', type: SETTING_TYPES.STRING, value: '' },
    ],
    supportedVersions: ['1.8.x', '1.12.x', '1.16.x', '1.20.x', '1.21.x'],
    tags: ['player', 'chest', 'steal', 'loot'],
    compatibility: { multiplayer: true, singleplayer: true },
  },
  // Render
  {
    id: 'fullbright',
    name: 'Fullbright',
    description: 'Makes everything fully lit, removing darkness.',
    category: 'render',
    enabled: false,
    keybind: null,
    settings: [
      { id: 'mode', name: 'Mode', type: SETTING_TYPES.ENUM, value: 'gamma', options: ['gamma', 'night_vision'] },
    ],
    supportedVersions: ['1.8.x', '1.12.x', '1.16.x', '1.20.x', '1.21.x'],
    tags: ['render', 'brightness', 'night', 'vision'],
    compatibility: { multiplayer: true, singleplayer: true },
  },
  {
    id: 'esp',
    name: 'ESP',
    description: 'Highlights entities through walls.',
    category: 'render',
    enabled: false,
    keybind: null,
    settings: [
      { id: 'players', name: 'Show Players', type: SETTING_TYPES.BOOLEAN, value: true },
      { id: 'mobs', name: 'Show Mobs', type: SETTING_TYPES.BOOLEAN, value: true },
      { id: 'items', name: 'Show Items', type: SETTING_TYPES.BOOLEAN, value: false },
      { id: 'healthbar', name: 'Health Bar', type: SETTING_TYPES.BOOLEAN, value: true },
      { id: 'player_color', name: 'Player Color', type: SETTING_TYPES.COLOR, value: '#ff0000' },
      { id: 'mob_color', name: 'Mob Color', type: SETTING_TYPES.COLOR, value: '#00ff00' },
    ],
    supportedVersions: ['1.8.x', '1.12.x', '1.16.x', '1.20.x', '1.21.x'],
    tags: ['render', 'esp', 'hack', 'xray', 'wallhack'],
    compatibility: { multiplayer: true, singleplayer: true },
  },
  {
    id: 'nametags',
    name: 'NameTags',
    description: 'Enhanced name tag rendering with health and distance.',
    category: 'render',
    enabled: false,
    keybind: null,
    settings: [
      { id: 'show_health', name: 'Show Health', type: SETTING_TYPES.BOOLEAN, value: true },
      { id: 'show_distance', name: 'Show Distance', type: SETTING_TYPES.BOOLEAN, value: true },
      { id: 'scale', name: 'Scale', type: SETTING_TYPES.SLIDER, value: 1.5, min: 0.5, max: 4.0, step: 0.1 },
    ],
    supportedVersions: ['1.8.x', '1.12.x', '1.16.x', '1.20.x', '1.21.x'],
    tags: ['render', 'nametag', 'hud'],
    compatibility: { multiplayer: true, singleplayer: true },
  },
  // World
  {
    id: 'xray',
    name: 'XRay',
    description: 'Reveals valuable ores through blocks.',
    category: 'world',
    enabled: false,
    keybind: null,
    settings: [
      { id: 'mode', name: 'Mode', type: SETTING_TYPES.ENUM, value: 'xray', options: ['xray', 'cave'] },
      { id: 'ores', name: 'Ores', type: SETTING_TYPES.MULTI_SELECT, value: ['diamond', 'emerald', 'gold', 'iron', 'lapis', 'redstone'], options: ['diamond', 'emerald', 'gold', 'iron', 'lapis', 'redstone', 'coal', 'copper'] },
    ],
    supportedVersions: ['1.8.x', '1.12.x', '1.16.x', '1.20.x', '1.21.x'],
    tags: ['world', 'xray', 'ores', 'hack'],
    compatibility: { multiplayer: true, singleplayer: true },
  },
  {
    id: 'autoshear',
    name: 'AutoShear',
    description: 'Automatically shears nearby sheep.',
    category: 'world',
    enabled: false,
    keybind: null,
    settings: [
      { id: 'range', name: 'Range', type: SETTING_TYPES.SLIDER, value: 4.0, min: 1.0, max: 6.0, step: 0.5 },
    ],
    supportedVersions: ['1.8.x', '1.12.x', '1.16.x', '1.20.x', '1.21.x'],
    tags: ['world', 'shear', 'farm', 'automation'],
    compatibility: { multiplayer: true, singleplayer: true },
  },
  // Exploit
  {
    id: 'lagback',
    name: 'LagBack',
    description: 'Reverts position when server sends lag compensation.',
    category: 'exploit',
    enabled: false,
    keybind: null,
    settings: [],
    supportedVersions: ['1.8.x', '1.12.x', '1.16.x', '1.20.x', '1.21.x'],
    tags: ['exploit', 'lag', 'prediction'],
    compatibility: { multiplayer: true, singleplayer: false },
  },
  // Automation
  {
    id: 'baritone',
    name: 'Baritone',
    description: 'Pathfinding and automated task execution.',
    category: 'automation',
    enabled: false,
    keybind: null,
    settings: [
      { id: 'speed', name: 'Speed', type: SETTING_TYPES.ENUM, value: 'normal', options: ['slow', 'normal', 'fast'] },
      { id: 'sprint', name: 'Sprint', type: SETTING_TYPES.BOOLEAN, value: true },
    ],
    supportedVersions: ['1.16.x', '1.20.x', '1.21.x'],
    tags: ['automation', 'pathfinding', 'baritone', 'bot'],
    compatibility: { multiplayer: true, singleplayer: true },
  },
  // Misc
  {
    id: 'autoreconnect',
    name: 'AutoReconnect',
    description: 'Automatically reconnects when disconnected from server.',
    category: 'misc',
    enabled: false,
    keybind: null,
    settings: [
      { id: 'delay', name: 'Delay (seconds)', type: SETTING_TYPES.INTEGER, value: 5, min: 1, max: 30 },
    ],
    supportedVersions: ['1.8.x', '1.12.x', '1.16.x', '1.20.x', '1.21.x'],
    tags: ['misc', 'reconnect', 'server'],
    compatibility: { multiplayer: true, singleplayer: false },
  },
  {
    id: 'antiafk',
    name: 'AntiAFK',
    description: 'Prevents being kicked for inactivity.',
    category: 'misc',
    enabled: false,
    keybind: null,
    settings: [
      { id: 'mode', name: 'Mode', type: SETTING_TYPES.ENUM, value: 'spin', options: ['spin', 'walk', 'jump', 'swing'] },
    ],
    supportedVersions: ['1.8.x', '1.12.x', '1.16.x', '1.20.x', '1.21.x'],
    tags: ['misc', 'afk', 'anticheat'],
    compatibility: { multiplayer: true, singleplayer: true },
  },
  // Client
  {
    id: 'clickgui',
    name: 'ClickGUI',
    description: 'In-game module configuration GUI.',
    category: 'client',
    enabled: true,
    keybind: { key: 'shift', code: 'ShiftLeft' },
    settings: [
      { id: 'opacity', name: 'Opacity', type: SETTING_TYPES.SLIDER, value: 0.85, min: 0.3, max: 1.0, step: 0.05 },
      { id: 'blur', name: 'Background Blur', type: SETTING_TYPES.BOOLEAN, value: true },
    ],
    supportedVersions: ['1.8.x', '1.12.x', '1.16.x', '1.20.x', '1.21.x'],
    tags: ['client', 'gui', 'clickgui'],
    compatibility: { multiplayer: true, singleplayer: true },
  },
]

export const useModulesStore = defineStore('modules', () => {
  const modules = ref([])
  const profiles = ref([])
  const activeProfileId = ref('default')
  const searchQuery = ref('')
  const loading = ref(false)

  const filteredModules = computed(() => {
    let result = [...modules.value]

    if (searchQuery.value.trim()) {
      const q = searchQuery.value.toLowerCase()
      result = result.filter(
        (m) =>
          m.name.toLowerCase().includes(q) ||
          m.description.toLowerCase().includes(q) ||
          m.id.toLowerCase().includes(q) ||
          m.tags.some((t) => t.includes(q)),
      )
    }

    return result
  })

  const modulesByCategory = computed(() => {
    const map = {}
    for (const cat of MODULE_CATEGORIES) {
      map[cat.id] = filteredModules.value.filter((m) => m.category === cat.id)
    }
    return map
  })

  const enabledCount = computed(() => modules.value.filter((m) => m.enabled).length)

  async function refresh() {
    loading.value = true
    try {
      // Load from localStorage for now (Rust backend TBD)
      const stored = localStorage.getItem('ar_modules')
      if (stored) {
        const saved = JSON.parse(stored)
        // Merge saved states with builtin definitions
        modules.value = BUILTIN_MODULES.map((m) => {
          const s = saved.find((x) => x.id === m.id)
          return s ? { ...m, enabled: s.enabled, keybind: s.keybind || m.keybind, settings: s.settings || m.settings } : { ...m }
        })
      } else {
        modules.value = BUILTIN_MODULES.map((m) => ({ ...m }))
      }

      // Load profiles
      const storedProfiles = localStorage.getItem('ar_module_profiles')
      profiles.value = storedProfiles
        ? JSON.parse(storedProfiles)
        : [{ id: 'default', name: 'Default', modules: {} }]
    } catch (err) {
      handleError({ message: `Failed to load modules: ${err.message}` })
    } finally {
      loading.value = false
    }
  }

  function saveToLocal() {
    localStorage.setItem(
      'ar_modules',
      JSON.stringify(
        modules.value.map((m) => ({
          id: m.id,
          enabled: m.enabled,
          keybind: m.keybind,
          settings: m.settings,
        })),
      ),
    )
  }

  function toggleModule(id) {
    const mod = modules.value.find((m) => m.id === id)
    if (mod) {
      mod.enabled = !mod.enabled
      saveToLocal()
    }
  }

  function setModuleEnabled(id, enabled) {
    const mod = modules.value.find((m) => m.id === id)
    if (mod) {
      mod.enabled = enabled
      saveToLocal()
    }
  }

  function updateModuleSetting(moduleId, settingId, value) {
    const mod = modules.value.find((m) => m.id === moduleId)
    if (mod) {
      const setting = mod.settings.find((s) => s.id === settingId)
      if (setting) {
        setting.value = value
        saveToLocal()
      }
    }
  }

  function setKeybind(moduleId, keybind) {
    const mod = modules.value.find((m) => m.id === moduleId)
    if (mod) {
      mod.keybind = keybind
      saveToLocal()
    }
  }

  function resetModule(moduleId) {
    const builtin = BUILTIN_MODULES.find((m) => m.id === moduleId)
    const mod = modules.value.find((m) => m.id === moduleId)
    if (builtin && mod) {
      mod.enabled = builtin.enabled
      mod.keybind = builtin.keybind
      mod.settings = JSON.parse(JSON.stringify(builtin.settings))
      saveToLocal()
    }
  }

  return {
    modules,
    profiles,
    activeProfileId,
    searchQuery,
    loading,
    filteredModules,
    modulesByCategory,
    enabledCount,
    refresh,
    toggleModule,
    setModuleEnabled,
    updateModuleSetting,
    setKeybind,
    resetModule,
    MODULE_CATEGORIES,
  }
})
