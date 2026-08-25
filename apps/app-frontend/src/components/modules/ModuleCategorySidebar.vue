<script setup>
import { computed } from 'vue'
import {
  SlashIcon,
  GaugeIcon,
  UserIcon,
  EyeIcon,
  GlobeIcon,
  IssuesIcon,
  CogIcon,
  MoreHorizontalIcon,
  SettingsIcon,
  GridIcon,
} from '@modrinth/assets'
import { MODULE_CATEGORIES } from '@/store/modules.js'

const props = defineProps({
  activeCategory: { type: String, default: 'all' },
  moduleCounts: { type: Object, default: () => ({}) },
  enabledCounts: { type: Object, default: () => ({}) },
})

const emit = defineEmits(['select'])

const CATEGORY_ICONS = {
  combat: SlashIcon,
  movement: GaugeIcon,
  player: UserIcon,
  render: EyeIcon,
  world: GlobeIcon,
  exploit: IssuesIcon,
  automation: CogIcon,
  misc: MoreHorizontalIcon,
  client: SettingsIcon,
}

const CATEGORY_COLORS = {
  combat: 'text-red-400',
  movement: 'text-blue-400',
  player: 'text-green-400',
  render: 'text-purple-400',
  world: 'text-orange-400',
  exploit: 'text-yellow-400',
  automation: 'text-emerald-400',
  misc: 'text-gray-400',
  client: 'text-brand',
}

const allCount = computed(() => {
  return Object.values(props.moduleCounts).reduce((a, b) => a + b, 0)
})

const allEnabledCount = computed(() => {
  return Object.values(props.enabledCounts).reduce((a, b) => a + b, 0)
})
</script>

<template>
  <div class="w-48 flex-shrink-0 border-r border-secondary pr-4 py-3">
    <div class="text-xs text-secondary uppercase tracking-wider mb-2 px-2">Categories</div>

    <!-- All -->
    <button
      class="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg text-sm transition-colors mb-0.5"
      :class="activeCategory === 'all' ? 'bg-brand/10 text-brand' : 'text-secondary hover:text-contrast hover:bg-bg-raised'"
      @click="emit('select', 'all')"
    >
      <GridIcon class="w-4 h-4" />
      <span class="flex-1 text-left">All</span>
      <span class="text-xs opacity-60">{{ allEnabledCount }}/{{ allCount }}</span>
    </button>

    <!-- Individual categories -->
    <button
      v-for="cat in MODULE_CATEGORIES"
      :key="cat.id"
      class="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg text-sm transition-colors mb-0.5"
      :class="activeCategory === cat.id ? 'bg-brand/10 text-brand' : 'text-secondary hover:text-contrast hover:bg-bg-raised'"
      @click="emit('select', cat.id)"
    >
      <component :is="CATEGORY_ICONS[cat.id]" class="w-4 h-4" :class="activeCategory === cat.id ? '' : CATEGORY_COLORS[cat.id]" />
      <span class="flex-1 text-left">{{ cat.name }}</span>
      <span class="text-xs opacity-60">{{ (enabledCounts[cat.id] || 0) }}/{{ moduleCounts[cat.id] || 0 }}</span>
    </button>
  </div>
</template>
