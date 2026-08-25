<script setup>
import { ref, computed } from 'vue'
import {
  SettingsIcon,
  KeyIcon,
  ChevronRightIcon,
  ShieldIcon,
  MonitorIcon,
  GameIcon,
} from '@modrinth/assets'
import { Button, Badge } from '@modrinth/ui'

const props = defineProps({
  module: { type: Object, required: true },
  expanded: { type: Boolean, default: false },
})

const emit = defineEmits(['toggle', 'expand', 'keybind', 'reset'])

const CATEGORY_COLORS = {
  combat: 'red',
  movement: 'blue',
  player: 'green',
  render: 'purple',
  world: 'orange',
  exploit: 'gray',
  automation: 'green',
  misc: 'gray',
  client: 'brand',
}

const categoryColor = computed(() => CATEGORY_COLORS[props.module.category] || 'gray')

const keybindLabel = computed(() => {
  if (!props.module.keybind) return 'None'
  return props.module.keybind.key || props.module.keybind.code || 'None'
})

function formatKeybind(kb) {
  if (!kb) return 'None'
  if (kb.key) {
    const key = kb.key
    return key.charAt(0).toUpperCase() + key.slice(1)
  }
  return kb.code || 'None'
}
</script>

<template>
  <div
    class="rounded-xl border transition-all duration-200"
    :class="[
      module.enabled
        ? 'border-brand/20 bg-brand/5'
        : 'border-secondary bg-bg-raised hover:bg-bg-raised/80'
    ]"
  >
    <!-- Main row -->
    <div class="flex items-center gap-3 px-4 py-3">
      <!-- Toggle switch -->
      <button
        class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-brand/50 focus:ring-offset-2 focus:ring-offset-bg-raised"
        :class="module.enabled ? 'bg-brand' : 'bg-secondary/30'"
        role="switch"
        :aria-checked="module.enabled"
        @click="emit('toggle', module.id)"
      >
        <span
          class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
          :class="module.enabled ? 'translate-x-4' : 'translate-x-0'"
        />
      </button>

      <!-- Name and category -->
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <span class="font-bold text-contrast">{{ module.name }}</span>
          <Badge :color="categoryColor" :type="module.category" />
          <span v-if="module.keybind" class="text-xs px-1.5 py-0.5 rounded bg-bg-base border border-secondary text-secondary font-mono">
            {{ formatKeybind(module.keybind) }}
          </span>
        </div>
        <p class="text-sm text-secondary m-0 mt-0.5">{{ module.description }}</p>
      </div>

      <!-- Compatibility icons -->
      <div class="flex items-center gap-1 flex-shrink-0">
        <span
          v-if="module.compatibility?.multiplayer"
          class="text-green-400"
          title="Multiplayer compatible"
        >
          <MonitorIcon class="w-4 h-4" />
        </span>
        <span
          v-if="module.compatibility?.singleplayer"
          class="text-blue-400"
          title="Singleplayer compatible"
        >
          <GameIcon class="w-4 h-4" />
        </span>
      </div>

      <!-- Action buttons -->
      <div class="flex items-center gap-1 flex-shrink-0">
        <Button
          :icon-only="true"
          :transparent="true"
          :action="() => emit('keybind', module)"
          title="Set keybind"
        >
          <KeyIcon class="w-4 h-4" />
        </Button>
        <Button
          :icon-only="true"
          :transparent="true"
          :action="() => emit('expand', module.id)"
          title="Settings"
        >
          <SettingsIcon class="w-4 h-4" />
        </Button>
      </div>
    </div>

    <!-- Expanded settings -->
    <div v-if="expanded" class="px-4 pb-4 border-t border-secondary/50 pt-3">
      <slot name="settings" />
      <div class="flex justify-end mt-3 pt-2 border-t border-secondary/30">
        <Button :outline="true" :action="() => emit('reset', module.id)">
          Reset to defaults
        </Button>
      </div>
    </div>
  </div>
</template>
