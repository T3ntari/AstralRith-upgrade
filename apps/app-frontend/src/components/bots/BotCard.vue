<script setup>
import { ref, computed } from 'vue'
import {
  PlayIcon,
  StopCircleIcon,
  SettingsIcon,
  TerminalSquareIcon,
  EditIcon,
  TrashIcon,
  CopyIcon,
  ChevronRightIcon,
  GlobeIcon,
  TimerIcon,
  CompassIcon,
  HeartIcon,
} from '@modrinth/assets'
import { Button, Badge } from '@modrinth/ui'
import { useBotsStore } from '@/store/bots.js'

const props = defineProps({
  bot: { type: Object, required: true },
})

const emit = defineEmits(['connect', 'disconnect', 'console', 'tasks', 'edit', 'duplicate', 'delete'])

const store = useBotsStore()
const expanded = ref(false)

const statusColor = computed(() => {
  switch (props.bot.status) {
    case 'connected': return 'green'
    case 'error': return 'red'
    default: return 'gray'
  }
})

const statusLabel = computed(() => {
  switch (props.bot.status) {
    case 'connected': return 'Connected'
    case 'error': return 'Error'
    default: return 'Offline'
  }
})

function formatPosition(pos) {
  if (!pos) return 'N/A'
  return `${Math.floor(pos.x)} / ${Math.floor(pos.y)} / ${Math.floor(pos.z)}`
}
</script>

<template>
  <div
    class="rounded-xl border border-secondary p-4 transition-all duration-200"
    :class="[
      bot.status === 'connected'
        ? 'bg-green-500/5 border-green-500/20'
        : bot.status === 'error'
          ? 'bg-red-500/5 border-red-500/20'
          : 'bg-bg-raised hover:bg-bg-raised/80'
    ]"
  >
    <!-- Header row -->
    <div class="flex items-center gap-3 mb-2">
      <div
        class="w-3 h-3 rounded-full flex-shrink-0"
        :class="[
          bot.status === 'connected' ? 'bg-green-400 animate-pulse' : '',
          bot.status === 'error' ? 'bg-red-400' : '',
          bot.status === 'disconnected' ? 'bg-secondary' : '',
        ]"
      />
      <span class="font-bold text-contrast text-base">{{ bot.name }}</span>
      <Badge :color="statusColor" :type="statusLabel" />
      <div class="ml-auto flex items-center gap-2">
        <span class="text-sm text-secondary">{{ bot.host }}:{{ bot.port }}</span>
        <Button
          :icon-only="true"
          :transparent="true"
          :action="() => expanded = !expanded"
        >
          <ChevronRightIcon
            class="w-4 h-4 transition-transform duration-200"
            :class="{ 'rotate-90': expanded }"
          />
        </Button>
      </div>
    </div>

    <!-- Info row -->
    <div class="flex items-center gap-4 text-sm text-secondary mb-3">
      <span class="flex items-center gap-1">
        <GlobeIcon class="w-3.5 h-3.5" />
        {{ bot.version || '1.21.4' }}
      </span>
      <span v-if="bot.authType" class="flex items-center gap-1">
        {{ bot.authType === 'microsoft' ? 'Microsoft' : 'Offline' }}
      </span>
      <span v-if="bot.username" class="flex items-center gap-1">
        {{ bot.username }}
      </span>
      <span v-if="bot.status === 'connected' && bot.ping != null" class="flex items-center gap-1">
        <TimerIcon class="w-3.5 h-3.5" />
        {{ bot.ping }}ms
      </span>
    </div>

    <!-- Connected-only info -->
    <div v-if="bot.status === 'connected'" class="flex items-center gap-4 text-sm text-secondary mb-3">
      <span v-if="bot.position" class="flex items-center gap-1">
        <CompassIcon class="w-3.5 h-3.5" />
        {{ formatPosition(bot.position) }}
      </span>
      <span v-if="bot.health != null" class="flex items-center gap-1">
        <HeartIcon class="w-3.5 h-3.5 text-red-400" />
        {{ bot.health }}
      </span>
    </div>

    <!-- Action buttons -->
    <div class="flex items-center gap-2 flex-wrap">
      <template v-if="bot.status === 'connected'">
        <Button color="red" :action="() => emit('disconnect', bot.id)">
          <StopCircleIcon class="w-4 h-4 mr-1" />
          Disconnect
        </Button>
        <Button color="blue" :action="() => emit('console', bot)">
          <TerminalSquareIcon class="w-4 h-4 mr-1" />
          Console
        </Button>
        <Button color="purple" :action="() => emit('tasks', bot)">
          <SettingsIcon class="w-4 h-4 mr-1" />
          Tasks
        </Button>
      </template>
      <template v-else>
        <Button color="green" :action="() => emit('connect', bot.id)">
          <PlayIcon class="w-4 h-4 mr-1" />
          Connect
        </Button>
        <Button :action="() => emit('edit', bot)">
          <EditIcon class="w-4 h-4 mr-1" />
          Edit
        </Button>
        <Button :action="() => emit('duplicate', bot.id)">
          <CopyIcon class="w-4 h-4 mr-1" />
          Duplicate
        </Button>
        <Button color="red" :action="() => emit('delete', bot)">
          <TrashIcon class="w-4 h-4 mr-1" />
          Delete
        </Button>
      </template>
    </div>

    <!-- Expanded details -->
    <div v-if="expanded" class="mt-3 pt-3 border-t border-secondary text-sm text-secondary space-y-1">
      <div class="grid grid-cols-2 gap-2">
        <div><span class="text-contrast">ID:</span> {{ bot.id?.slice(0, 8) }}...</div>
        <div><span class="text-contrast">Auth:</span> {{ bot.authType || 'offline' }}</div>
        <div><span class="text-contrast">Created:</span> {{ new Date(bot.created).toLocaleDateString() }}</div>
        <div><span class="text-contrast">Modified:</span> {{ new Date(bot.modified).toLocaleDateString() }}</div>
        <div v-if="bot.script"><span class="text-contrast">Script:</span> {{ bot.script }}</div>
        <div v-if="bot.plugins?.length"><span class="text-contrast">Plugins:</span> {{ bot.plugins.join(', ') }}</div>
      </div>
    </div>
  </div>
</template>
