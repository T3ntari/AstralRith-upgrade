<script setup>
import { ref, computed } from 'vue'
import {
  XIcon,
  PlayIcon,
  StopCircleIcon,
  PlusIcon,
  TrashIcon,
  ChevronRightIcon,
  TransferIcon,
  RedoIcon,
  TimerIcon,
  ShieldIcon,
  CompassIcon,
  MessageIcon,
  HammerIcon,
  RightArrowIcon,
  LoaderIcon,
} from '@modrinth/assets'
import { Button, Badge, DropdownSelect } from '@modrinth/ui'

const props = defineProps({
  bot: { type: Object, required: false, default: null },
  visible: { type: Boolean, default: false },
})

const emit = defineEmits(['close', 'start-tasks', 'stop-tasks'])

// Task type definitions
const TASK_TYPES = [
  { id: 'goto', name: 'Go To', icon: 'MapPin', description: 'Navigate to coordinates', fields: [
    { id: 'x', name: 'X', type: 'number', default: 0 },
    { id: 'y', name: 'Y', type: 'number', default: 64 },
    { id: 'z', name: 'Z', type: 'number', default: 0 },
  ]},
  { id: 'follow', name: 'Follow', icon: 'User', description: 'Follow a player or entity', fields: [
    { id: 'target', name: 'Target', type: 'text', default: '' },
    { id: 'range', name: 'Range', type: 'number', default: 3 },
  ]},
  { id: 'lookat', name: 'Look At', icon: 'Eye', description: 'Look at coordinates', fields: [
    { id: 'x', name: 'X', type: 'number', default: 0 },
    { id: 'y', name: 'Y', type: 'number', default: 64 },
    { id: 'z', name: 'Z', type: 'number', default: 0 },
  ]},
  { id: 'chat', name: 'Chat', icon: 'MessageSquare', description: 'Send a chat message', fields: [
    { id: 'message', name: 'Message', type: 'text', default: 'Hello!' },
  ]},
  { id: 'command', name: 'Execute Command', icon: 'TerminalSquare', description: 'Run a server command', fields: [
    { id: 'command', name: 'Command', type: 'text', default: '' },
  ]},
  { id: 'collect', name: 'Collect', icon: 'Package', description: 'Collect nearby items', fields: [
    { id: 'item', name: 'Item', type: 'text', default: '' },
    { id: 'radius', name: 'Radius', type: 'number', default: 16 },
  ]},
  { id: 'mine', name: 'Mine', icon: 'Hammer', description: 'Mine blocks', fields: [
    { id: 'block', name: 'Block', type: 'text', default: 'stone' },
    { id: 'count', name: 'Count', type: 'number', default: 64 },
  ]},
  { id: 'build', name: 'Build', icon: 'Box', description: 'Build a structure', fields: [
    { id: 'schematic', name: 'Schematic', type: 'text', default: '' },
  ]},
  { id: 'wait', name: 'Wait', icon: 'Clock', description: 'Wait for a duration', fields: [
    { id: 'seconds', name: 'Seconds', type: 'number', default: 5 },
  ]},
  { id: 'loop', name: 'Loop', icon: 'Repeat', description: 'Repeat tasks N times', fields: [
    { id: 'times', name: 'Times', type: 'number', default: 3 },
  ]},
]

const COMPOSABLE_TYPES = [
  { id: 'sequence', name: 'Sequence', icon: 'ArrowRight', description: 'Run tasks in order' },
  { id: 'parallel', name: 'Parallel', icon: 'GitBranch', description: 'Run tasks simultaneously' },
  { id: 'condition', name: 'Condition', icon: 'Shield', description: 'Run tasks based on condition' },
  { id: 'repeat', name: 'Repeat', icon: 'Repeat', description: 'Repeat task group' },
  { id: 'timeout', name: 'Timeout', icon: 'Clock', description: 'Limit task execution time' },
]

const taskQueue = ref([])
const running = ref(false)
const selectedTaskType = ref(TASK_TYPES[0])
const taskForm = ref({})
const showTypeSelector = ref(false)

function openTaskBuilder(type) {
  selectedTaskType.value = type
  taskForm.value = {}
  for (const field of type.fields) {
    taskForm.value[field.id] = field.default
  }
  showTypeSelector.value = true
}

function addTask() {
  const task = {
    id: Date.now(),
    type: selectedTaskType.value.id,
    name: selectedTaskType.value.name,
    icon: selectedTaskType.value.icon,
    params: { ...taskForm.value },
    status: 'pending',
  }
  taskQueue.value.push(task)
  showTypeSelector.value = false
}

function removeTask(taskId) {
  taskQueue.value = taskQueue.value.filter(t => t.id !== taskId)
}

function moveTask(taskId, direction) {
  const idx = taskQueue.value.findIndex(t => t.id === taskId)
  if (idx === -1) return
  const newIdx = direction === 'up' ? idx - 1 : idx + 1
  if (newIdx < 0 || newIdx >= taskQueue.value.length) return
  const temp = taskQueue.value[idx]
  taskQueue.value[idx] = taskQueue.value[newIdx]
  taskQueue.value[newIdx] = temp
}

function startExecution() {
  if (taskQueue.value.length === 0) return
  running.value = true
  // Mark first as running
  taskQueue.value.forEach((t, i) => {
    t.status = i === 0 ? 'running' : 'pending'
  })
  emit('start-tasks', { botId: props.bot?.id, tasks: taskQueue.value })
}

function stopExecution() {
  running.value = false
  taskQueue.value.forEach(t => {
    if (t.status === 'running') t.status = 'stopped'
  })
  emit('stop-tasks', { botId: props.bot?.id })
}

function clearQueue() {
  taskQueue.value = []
  running.value = false
}
</script>

<template>
  <Teleport to="body">
    <Transition name="modal-fade">
      <div
        v-if="visible"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
        @click.self="emit('close')"
      >
        <div
          class="bg-bg-raised border border-secondary rounded-xl shadow-2xl w-full max-w-2xl mx-4 flex flex-col"
          style="height: 75vh; max-height: 700px;"
        >
          <!-- Header -->
          <div class="flex items-center gap-3 px-4 py-3 border-b border-secondary">
            <span class="font-bold text-contrast text-lg">Task Panel</span>
            <span v-if="bot" class="text-sm text-secondary">— {{ bot.name }}</span>
            <div class="ml-auto flex items-center gap-2">
              <Button color="green" :action="startExecution" :disabled="running || taskQueue.length === 0">
                <PlayIcon class="w-4 h-4 mr-1" />
                Start
              </Button>
              <Button color="red" :action="stopExecution" :disabled="!running">
                <StopCircleIcon class="w-4 h-4 mr-1" />
                Stop
              </Button>
              <Button :transparent="true" :icon-only="true" :action="clearQueue" title="Clear queue">
                <TrashIcon class="w-4 h-4" />
              </Button>
              <Button :transparent="true" :icon-only="true" :action="() => emit('close')" title="Close">
                <XIcon class="w-4 h-4" />
              </Button>
            </div>
          </div>

          <!-- Task type selector / builder -->
          <div v-if="showTypeSelector" class="px-4 py-3 border-b border-secondary bg-bg-base/50">
            <div class="flex items-center gap-2 mb-3">
              <span class="font-semibold text-contrast">{{ selectedTaskType.name }}</span>
              <span class="text-sm text-secondary">— {{ selectedTaskType.description }}</span>
              <div class="ml-auto">
                <Button :transparent="true" :icon-only="true" :action="() => showTypeSelector = false">
                  <XIcon class="w-4 h-4" />
                </Button>
              </div>
            </div>
            <div class="space-y-2">
              <div v-for="field in selectedTaskType.fields" :key="field.id" class="flex items-center gap-3">
                <label class="text-sm text-secondary w-24 flex-shrink-0">{{ field.name }}</label>
                <input
                  v-model="taskForm[field.id]"
                  :type="field.type"
                  class="flex-1 bg-black/30 border border-secondary rounded-lg px-3 py-1.5 text-sm text-contrast focus:outline-none focus:border-brand/50 transition-colors"
                />
              </div>
            </div>
            <div class="mt-3 flex justify-end">
              <Button color="green" :action="addTask">
                <PlusIcon class="w-4 h-4 mr-1" />
                Add Task
              </Button>
            </div>
          </div>

          <!-- Task types grid (when not building) -->
          <div v-if="!showTypeSelector" class="px-4 py-3 border-b border-secondary">
            <div class="text-xs text-secondary mb-2 uppercase tracking-wider">Add Task</div>
            <div class="flex flex-wrap gap-1.5">
              <Button
                v-for="taskType in TASK_TYPES"
                :key="taskType.id"
                :outline="true"
                :action="() => openTaskBuilder(taskType)"
              >
                {{ taskType.name }}
              </Button>
            </div>
            <div class="text-xs text-secondary mt-3 mb-1 uppercase tracking-wider">Composable</div>
            <div class="flex flex-wrap gap-1.5">
              <Button
                v-for="comp in COMPOSABLE_TYPES"
                :key="comp.id"
                :outline="true"
                :color="'purple'"
                :action="() => openTaskBuilder(comp)"
              >
                {{ comp.name }}
              </Button>
            </div>
          </div>

          <!-- Task queue -->
          <div class="flex-1 overflow-y-auto px-4 py-3">
            <div class="text-xs text-secondary mb-2 uppercase tracking-wider">
              Task Queue ({{ taskQueue.length }})
            </div>
            <div v-if="taskQueue.length === 0" class="text-center py-12 text-secondary/50">
              <PlusIcon class="w-8 h-8 mx-auto mb-2 opacity-40" />
              <p>No tasks in queue. Add tasks above.</p>
            </div>
            <div v-else class="space-y-1.5">
              <div
                v-for="(task, index) in taskQueue"
                :key="task.id"
                class="flex items-center gap-3 px-3 py-2 rounded-lg border transition-colors"
                :class="[
                  task.status === 'running'
                    ? 'border-green-500/30 bg-green-500/5'
                    : task.status === 'completed'
                      ? 'border-green-500/20 bg-green-500/5 opacity-60'
                      : task.status === 'stopped'
                        ? 'border-red-500/20 bg-red-500/5'
                        : 'border-secondary bg-bg-base/30'
                ]"
              >
                <span class="text-xs text-secondary w-6 text-center">{{ index + 1 }}</span>
                <div
                  v-if="task.status === 'running'"
                  class="w-2 h-2 rounded-full bg-green-400 animate-pulse flex-shrink-0"
                />
                <div
                  v-else-if="task.status === 'completed'"
                  class="w-2 h-2 rounded-full bg-green-600 flex-shrink-0"
                />
                <div
                  v-else-if="task.status === 'stopped'"
                  class="w-2 h-2 rounded-full bg-red-400 flex-shrink-0"
                />
                <div
                  v-else
                  class="w-2 h-2 rounded-full bg-secondary/50 flex-shrink-0"
                />
                <span class="font-medium text-contrast text-sm">{{ task.name }}</span>
                <span class="text-xs text-secondary truncate">
                  {{ Object.entries(task.params).filter(([_, v]) => v).map(([k, v]) => `${k}=${v}`).join(', ') }}
                </span>
                <div class="ml-auto flex items-center gap-1">
                  <Button
                    :icon-only="true"
                    :transparent="true"
                    :action="() => moveTask(task.id, 'up')"
                    :disabled="index === 0"
                    title="Move up"
                  >
                    <ChevronRightIcon class="w-3 h-3 -rotate-90" />
                  </Button>
                  <Button
                    :icon-only="true"
                    :transparent="true"
                    :action="() => moveTask(task.id, 'down')"
                    :disabled="index === taskQueue.length - 1"
                    title="Move down"
                  >
                    <ChevronRightIcon class="w-3 h-3 rotate-90" />
                  </Button>
                  <Button
                    :icon-only="true"
                    :transparent="true"
                    :color="'red'"
                    :action="() => removeTask(task.id)"
                    title="Remove"
                  >
                    <XIcon class="w-3.5 h-3.5" />
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-fade-enter-active,
.modal-fade-leave-active {
  transition: opacity 0.2s ease;
}
.modal-fade-enter-active > div,
.modal-fade-leave-active > div {
  transition: transform 0.2s ease;
}
.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
}
.modal-fade-enter-from > div,
.modal-fade-leave-to > div {
  transform: scale(0.95) translateY(10px);
}
</style>
