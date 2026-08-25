<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { StopCircleIcon, FolderOpenIcon, GameIcon } from '@modrinth/assets'
import { ButtonStyled, Avatar } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { formatCategory } from '@modrinth/utils'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'

dayjs.extend(relativeTime)

const props = defineProps({
  runningInstances: {
    type: Array,
    default: () => [],
  },
  profiles: {
    type: Array,
    default: () => [],
  },
})

const emit = defineEmits(['stop', 'force-stop', 'open-folder'])

const now = ref(Date.now())
let timer = null

onMounted(() => {
  timer = setInterval(() => {
    now.value = Date.now()
  }, 1000)
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
})

function getProfile(process) {
  return props.profiles.find((p) => p.path === process.profile_path)
}

function getIconSrc(profile) {
  if (profile?.icon_path) return convertFileSrc(profile.icon_path)
  return null
}

function formatElapsedTime(startTime) {
  const start = dayjs(startTime)
  const diff = now.value - start.valueOf()
  const totalSeconds = Math.floor(diff / 1000)
  const hours = Math.floor(totalSeconds / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  const seconds = totalSeconds % 60
  if (hours > 0) {
    return `${hours}h ${String(minutes).padStart(2, '0')}m`
  }
  return `${minutes}m ${String(seconds).padStart(2, '0')}s`
}

function handleStop(process) {
  emit('stop', process)
}

function handleForceStop(process) {
  emit('force-stop', process)
}

function handleOpenFolder(profile) {
  emit('open-folder', profile)
}
</script>

<template>
  <div v-if="runningInstances.length > 0" class="mb-6">
    <h2 class="text-lg font-bold text-contrast mb-3 flex items-center gap-2">
      <span class="relative flex h-2.5 w-2.5">
        <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
        <span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-green-500"></span>
      </span>
      Running Now
      <span class="text-sm font-normal text-secondary">({{ runningInstances.length }})</span>
    </h2>

    <div class="bg-bg-raised rounded-xl overflow-hidden">
      <!-- Header -->
      <div class="grid grid-cols-[auto_1fr_1fr_auto_120px_100px] gap-4 px-4 py-2.5 text-xs font-semibold text-secondary uppercase tracking-wider border-b border-button-bg">
        <span class="w-8"></span>
        <span>Instance</span>
        <span>Version</span>
        <span>Status</span>
        <span>Uptime</span>
        <span class="text-right">Actions</span>
      </div>

      <!-- Rows -->
      <div
        v-for="process in runningInstances"
        :key="process.uuid"
        class="grid grid-cols-[auto_1fr_1fr_auto_120px_100px] gap-4 px-4 py-3 items-center border-b border-button-bg last:border-b-0 hover:bg-button-bg/30 transition-colors"
      >
        <Avatar
          :src="getIconSrc(getProfile(process))"
          :tint-by="process.profile_path"
          size="32px"
          no-shadow
          alt="Instance icon"
          loading="lazy"
        />
        <div class="min-w-0">
          <p class="m-0 text-sm font-bold text-contrast truncate">
            {{ getProfile(process)?.name || process.profile_path }}
          </p>
          <p class="m-0 text-xs text-secondary truncate">
            {{ getProfile(process)?.game_version }}
          </p>
        </div>
        <div class="flex items-center gap-1.5">
          <GameIcon class="w-3.5 h-3.5 text-secondary shrink-0" />
          <span class="text-xs text-secondary">
            {{ formatCategory(getProfile(process)?.loader || 'vanilla') }}
            {{ getProfile(process)?.loader_version || '' }}
          </span>
        </div>
        <div class="flex items-center gap-1.5">
          <span class="relative flex h-2 w-2">
            <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
            <span class="relative inline-flex rounded-full h-2 w-2 bg-green-500"></span>
          </span>
          <span class="text-xs text-green-500 font-semibold">Running</span>
        </div>
        <span class="text-xs text-secondary font-mono">
          {{ formatElapsedTime(process.start_time) }}
        </span>
        <div class="flex items-center justify-end gap-1">
          <ButtonStyled color="standard" circular>
            <button
              v-tooltip="'Open Folder'"
              @click="handleOpenFolder(getProfile(process))"
            >
              <FolderOpenIcon />
            </button>
          </ButtonStyled>
          <ButtonStyled color="red" circular>
            <button
              v-tooltip="'Stop'"
              @click="handleStop(process)"
            >
              <StopCircleIcon />
            </button>
          </ButtonStyled>
        </div>
      </div>
    </div>
  </div>
</template>
