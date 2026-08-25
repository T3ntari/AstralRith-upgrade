<script setup>
import { ref, computed } from 'vue'
import {
  PlayIcon,
  StopCircleIcon,
  FolderOpenIcon,
  CogIcon,
  GameIcon,
  TimerIcon,
  SpinnerIcon,
} from '@modrinth/assets'
import { ButtonStyled, Avatar, OverflowMenu } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { formatCategory } from '@modrinth/utils'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'

dayjs.extend(relativeTime)

const props = defineProps({
  profile: {
    type: Object,
    required: true,
  },
  isRunning: {
    type: Boolean,
    default: false,
  },
  isSelected: {
    type: Boolean,
    default: false,
  },
  loading: {
    type: Boolean,
    default: false,
  },
})

const emit = defineEmits(['launch', 'stop', 'open-folder', 'delete', 'duplicate', 'rename', 'select', 'settings'])

const contextMenuRef = ref(null)

const iconSrc = computed(() => {
  if (props.profile.icon_path) {
    return convertFileSrc(props.profile.icon_path)
  }
  return null
})

const loaderLabel = computed(() => {
  return formatCategory(props.profile.loader)
})

const installLabel = computed(() => {
  switch (props.profile.install_stage) {
    case 'installed': return null
    case 'installing': return 'Installing...'
    case 'pack_installing': return 'Pack installing...'
    case 'not_installed': return 'Not installed'
    default: return null
  }
})

const isInstalling = computed(() => {
  return props.profile.install_stage === 'installing' || props.profile.install_stage === 'pack_installing'
})

const timePlayedFormatted = computed(() => {
  const totalSeconds = props.profile.recent_time_played || 0
  if (totalSeconds === 0) return null
  const hours = Math.floor(totalSeconds / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  if (hours > 0) return `${hours}h ${minutes}m`
  return `${minutes}m`
})

const lastPlayedFormatted = computed(() => {
  if (!props.profile.last_played) return 'Never played'
  return dayjs(props.profile.last_played).fromNow()
})

function handleClick() {
  emit('select', props.profile)
}

function handleLaunch(e) {
  e?.stopPropagation()
  emit('launch', props.profile)
}

function handleStop(e) {
  e?.stopPropagation()
  emit('stop', props.profile)
}

function handleOpenFolder(e) {
  e?.stopPropagation()
  emit('open-folder', props.profile)
}

function handleDelete(e) {
  e?.stopPropagation()
  emit('delete', props.profile)
}

function handleDuplicate(e) {
  e?.stopPropagation()
  emit('duplicate', props.profile)
}

function handleRename(e) {
  e?.stopPropagation()
  emit('rename', props.profile)
}

function handleSettings(e) {
  e?.stopPropagation()
  emit('settings', props.profile)
}
</script>

<template>
  <div
    class="group relative bg-bg-raised rounded-xl overflow-hidden cursor-pointer transition-all duration-200 hover:brightness-95"
    :class="{
      'ring-2 ring-brand': isSelected,
      'opacity-60 pointer-events-none': isInstalling,
    }"
    @click="handleClick"
    @contextmenu.prevent="$refs.contextMenuRef?.showMenu($event, profile, contextOptions)"
  >
    <!-- Icon area with hover overlay -->
    <div class="relative aspect-square overflow-hidden">
      <!-- Background icon -->
      <div class="absolute inset-0 flex items-center justify-center bg-button-bg">
        <Avatar
          :src="iconSrc"
          :tint-by="profile.path"
          size="100%"
          no-shadow
          alt="Instance icon"
          loading="lazy"
          class="w-full h-full rounded-none!"
        />
      </div>

      <!-- Install overlay -->
      <div
        v-if="isInstalling"
        class="absolute inset-0 bg-black/60 flex flex-col items-center justify-center gap-2"
      >
        <SpinnerIcon class="animate-spin w-8 h-8 text-brand" />
        <span class="text-sm font-semibold text-contrast">{{ installLabel }}</span>
      </div>

      <!-- Hover overlay with actions -->
      <div
        v-else
        class="absolute inset-0 bg-black/0 group-hover:bg-black/50 transition-all duration-200 flex items-center justify-center gap-2 opacity-0 group-hover:opacity-100"
      >
        <ButtonStyled color="brand" size="large" circular>
          <button
            v-if="!isRunning"
            v-tooltip="'Launch'"
            class="shadow-lg"
            @click="handleLaunch"
          >
            <PlayIcon class="translate-x-[2px]" />
          </button>
          <button
            v-else
            v-tooltip="'Stop'"
            class="shadow-lg"
            @click="handleStop"
          >
            <StopCircleIcon />
          </button>
        </ButtonStyled>

        <ButtonStyled color="standard" size="large" circular>
          <button
            v-tooltip="'Open Folder'"
            class="shadow-lg"
            @click="handleOpenFolder"
          >
            <FolderOpenIcon />
          </button>
        </ButtonStyled>

        <ButtonStyled color="standard" size="large" circular>
          <button
            v-tooltip="'Settings'"
            class="shadow-lg"
            @click="handleSettings"
          >
            <CogIcon />
          </button>
        </ButtonStyled>
      </div>

      <!-- Running indicator -->
      <div
        v-if="isRunning"
        class="absolute top-2 right-2"
      >
        <span class="relative flex h-3 w-3">
          <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
          <span class="relative inline-flex rounded-full h-3 w-3 bg-green-500"></span>
        </span>
      </div>

      <!-- Overflow menu -->
      <div class="absolute top-2 left-2 opacity-0 group-hover:opacity-100 transition-opacity">
        <OverflowMenu
          :options="[
            {
              id: 'Launch',
              action: handleLaunch,
              shown: !isRunning,
            },
            {
              id: 'Stop',
              action: handleStop,
              color: 'red',
              shown: isRunning,
            },
            { id: 'divider', divider: true },
            {
              id: 'Open Folder',
              action: handleOpenFolder,
            },
            {
              id: 'Settings',
              action: handleSettings,
            },
            { id: 'divider', divider: true },
            {
              id: 'Duplicate',
              action: handleDuplicate,
            },
            {
              id: 'Rename',
              action: handleRename,
            },
            { id: 'divider', divider: true },
            {
              id: 'Delete',
              action: handleDelete,
              color: 'red',
            },
          ]"
          tooltip="More options"
        >
          <CogIcon />
        </OverflowMenu>
      </div>
    </div>

    <!-- Info area -->
    <div class="p-3 flex flex-col gap-1">
      <p class="m-0 text-sm font-bold text-contrast leading-tight line-clamp-1">
        {{ profile.name }}
      </p>
      <div class="flex items-center gap-1.5 text-xs text-secondary">
        <GameIcon class="w-3.5 h-3.5 shrink-0" />
        <span class="truncate">{{ profile.game_version }}</span>
        <span class="px-1.5 py-0.5 bg-button-bg rounded text-[10px] font-semibold uppercase leading-none">
          {{ loaderLabel }}
        </span>
      </div>
      <div class="flex items-center gap-1 text-xs text-secondary mt-0.5">
        <TimerIcon class="w-3 h-3 shrink-0" />
        <span class="truncate">{{ lastPlayedFormatted }}</span>
        <span v-if="timePlayedFormatted" class="text-tertiary">· {{ timePlayedFormatted }}</span>
      </div>
    </div>
  </div>
</template>
