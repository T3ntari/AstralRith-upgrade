<script setup>
import { computed } from 'vue'
import {
  PlayIcon,
  StopCircleIcon,
  FolderOpenIcon,
  CogIcon,
  GameIcon,
  TimerIcon,
  CalendarIcon,
  BoxIcon,
  XIcon,
  SpinnerIcon,
} from '@modrinth/assets'
import { ButtonStyled, Avatar, TagItem } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { formatCategory } from '@modrinth/utils'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import duration from 'dayjs/plugin/duration'

dayjs.extend(relativeTime)
dayjs.extend(duration)

const props = defineProps({
  profile: {
    type: Object,
    default: null,
  },
  isRunning: {
    type: Boolean,
    default: false,
  },
  runningProcesses: {
    type: Array,
    default: () => [],
  },
})

const emit = defineEmits(['launch', 'stop', 'open-folder', 'settings', 'close'])

const iconSrc = computed(() => {
  if (props.profile?.icon_path) return convertFileSrc(props.profile.icon_path)
  return null
})

const loaderLabel = computed(() => {
  if (!props.profile) return ''
  return formatCategory(props.profile.loader)
})

const installLabel = computed(() => {
  if (!props.profile) return ''
  switch (props.profile.install_stage) {
    case 'installed': return 'Installed'
    case 'installing': return 'Installing...'
    case 'pack_installing': return 'Pack installing...'
    case 'not_installed': return 'Not installed'
    default: return 'Unknown'
  }
})

const installColor = computed(() => {
  switch (props.profile?.install_stage) {
    case 'installed': return 'green'
    case 'installing':
    case 'pack_installing': return 'orange'
    case 'not_installed': return 'red'
    default: return 'gray'
  }
})

const totalPlaytime = computed(() => {
  if (!props.profile) return null
  const seconds = props.profile.recent_time_played || 0
  if (seconds === 0) return 'No playtime recorded'

  const dur = dayjs.duration(seconds, 'seconds')
  const hours = Math.floor(dur.asHours())
  const minutes = dur.minutes()

  if (hours > 24) {
    const days = Math.floor(hours / 24)
    const remHours = hours % 24
    return `${days}d ${remHours}h ${minutes}m`
  }
  if (hours > 0) return `${hours}h ${minutes}m`
  return `${minutes}m`
})

const lastPlayed = computed(() => {
  if (!props.profile?.last_played) return 'Never played'
  return dayjs(props.profile.last_played).fromNow()
})

const hasMemoryOverride = computed(() => {
  return props.profile?.memory?.maximum != null
})

const memoryMB = computed(() => {
  return props.profile?.memory?.maximum || 0
})

const hasJavaOverride = computed(() => {
  return !!props.profile?.java_path
})

const hasLinkedModpack = computed(() => {
  return !!props.profile?.linked_data
})

const isInstalling = computed(() => {
  return props.profile?.install_stage === 'installing' || props.profile?.install_stage === 'pack_installing'
})
</script>

<template>
  <div
    v-if="profile"
    class="bg-bg-raised rounded-xl overflow-hidden flex flex-col"
  >
    <!-- Header with close button -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-button-bg">
      <h3 class="m-0 text-sm font-bold text-contrast">Instance Details</h3>
      <ButtonStyled color="standard" circular type="transparent">
        <button @click="emit('close')">
          <XIcon />
        </button>
      </ButtonStyled>
    </div>

    <!-- Profile icon and name -->
    <div class="p-4 flex flex-col items-center gap-3">
      <Avatar
        :src="iconSrc"
        :tint-by="profile.path"
        size="64px"
        no-shadow
        alt="Instance icon"
        loading="lazy"
      />
      <div class="text-center">
        <h2 class="m-0 text-lg font-bold text-contrast">{{ profile.name }}</h2>
        <p class="m-0 text-sm text-secondary mt-0.5">{{ profile.path }}</p>
      </div>
    </div>

    <!-- Quick actions -->
    <div class="px-4 pb-3 flex gap-2">
      <ButtonStyled v-if="!isRunning && !isInstalling" color="brand" class="flex-1">
        <button class="w-full" @click="emit('launch', profile)">
          <PlayIcon class="translate-x-[1px]" />
          Launch
        </button>
      </ButtonStyled>
      <ButtonStyled v-else-if="isRunning" color="red" class="flex-1">
        <button class="w-full" @click="emit('stop', profile)">
          <StopCircleIcon />
          Stop
        </button>
      </ButtonStyled>
      <ButtonStyled v-else color="standard" class="flex-1" disabled>
        <button class="w-full">
          <SpinnerIcon class="animate-spin" />
          Installing...
        </button>
      </ButtonStyled>
      <ButtonStyled color="standard" circular>
        <button v-tooltip="'Open Folder'" @click="emit('open-folder', profile)">
          <FolderOpenIcon />
        </button>
      </ButtonStyled>
      <ButtonStyled color="standard" circular>
        <button v-tooltip="'Settings'" @click="emit('settings', profile)">
          <CogIcon />
        </button>
      </ButtonStyled>
    </div>

    <!-- Details -->
    <div class="px-4 pb-4 flex flex-col gap-3 text-sm">
      <!-- Status -->
      <div class="flex items-center justify-between">
        <span class="text-secondary">Status</span>
        <span
          class="font-semibold px-2 py-0.5 rounded text-xs"
          :class="{
            'bg-green-500/20 text-green-400': installColor === 'green',
            'bg-orange-500/20 text-orange-400': installColor === 'orange',
            'bg-red-500/20 text-red-400': installColor === 'red',
            'bg-gray-500/20 text-gray-400': installColor === 'gray',
          }"
        >
          {{ installLabel }}
        </span>
      </div>

      <!-- Game version -->
      <div class="flex items-center justify-between">
        <span class="text-secondary flex items-center gap-1.5">
          <GameIcon class="w-4 h-4" />
          Game
        </span>
        <span class="text-contrast font-medium">{{ profile.game_version }}</span>
      </div>

      <!-- Loader -->
      <div class="flex items-center justify-between">
        <span class="text-secondary">Loader</span>
        <div class="flex items-center gap-1.5">
          <span class="text-contrast font-medium">{{ loaderLabel }}</span>
          <span v-if="profile.loader_version" class="text-xs text-secondary">
            {{ profile.loader_version }}
          </span>
        </div>
      </div>

      <!-- Separator -->
      <div class="h-px bg-button-bg"></div>

      <!-- Time played -->
      <div class="flex items-center justify-between">
        <span class="text-secondary flex items-center gap-1.5">
          <TimerIcon class="w-4 h-4" />
          Playtime
        </span>
        <span class="text-contrast font-medium">{{ totalPlaytime }}</span>
      </div>

      <!-- Last played -->
      <div class="flex items-center justify-between">
        <span class="text-secondary flex items-center gap-1.5">
          <CalendarIcon class="w-4 h-4" />
          Last played
        </span>
        <span class="text-contrast font-medium">{{ lastPlayed }}</span>
      </div>

      <!-- Separator -->
      <div class="h-px bg-button-bg"></div>

      <!-- Java settings -->
      <div v-if="hasJavaOverride" class="flex items-center justify-between">
        <span class="text-secondary">Java</span>
        <TagItem>Custom Java</TagItem>
      </div>

      <!-- Memory settings -->
      <div v-if="hasMemoryOverride" class="flex items-center justify-between">
        <span class="text-secondary">Memory</span>
        <span class="text-contrast font-medium">{{ memoryMB }} MB</span>
      </div>

      <!-- Linked modpack -->
      <div v-if="hasLinkedModpack" class="flex items-center justify-between">
        <span class="text-secondary flex items-center gap-1.5">
          <BoxIcon class="w-4 h-4" />
          Linked Modpack
        </span>
        <TagItem>{{ profile.linked_data.project_id }}</TagItem>
      </div>
    </div>
  </div>
</template>
