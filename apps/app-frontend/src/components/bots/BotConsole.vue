<script setup>
import { ref, nextTick, watch, onMounted } from 'vue'
import {
  TerminalSquareIcon,
  XIcon,
  TrashIcon,
  SendIcon,
  InfoIcon,
} from '@modrinth/assets'
import { Button } from '@modrinth/ui'

const props = defineProps({
  bot: { type: Object, required: true },
  visible: { type: Boolean, default: false },
})

const emit = defineEmits(['close', 'send-command'])

const messages = ref([])
const commandInput = ref('')
const consoleRef = ref(null)
const autoScroll = ref(true)

// Seed with initial messages
onMounted(() => {
  messages.value = [
    { id: 1, type: 'system', text: `Console connected to ${props.bot.name}`, time: new Date() },
    { id: 2, type: 'info', text: `Server: ${props.bot.host}:${props.bot.port}`, time: new Date() },
  ]
})

function scrollToBottom() {
  if (autoScroll.value && consoleRef.value) {
    nextTick(() => {
      consoleRef.value.scrollTop = consoleRef.value.scrollHeight
    })
  }
}

watch(messages, () => scrollToBottom(), { deep: true })

function sendCommand() {
  const cmd = commandInput.value.trim()
  if (!cmd) return

  messages.value.push({
    id: Date.now(),
    type: 'input',
    text: cmd,
    time: new Date(),
  })

  emit('send-command', { botId: props.bot.id, command: cmd })

  // Simulate a response
  const responses = [
    { type: 'info', text: `Executing: ${cmd}` },
    { type: 'chat', text: `[Server] Processed command successfully` },
  ]
  const resp = responses[Math.floor(Math.random() * responses.length)]
  messages.value.push({
    id: Date.now() + 1,
    type: resp.type,
    text: resp.text,
    time: new Date(),
  })

  commandInput.value = ''
}

function clearConsole() {
  messages.value = [
    { id: Date.now(), type: 'system', text: 'Console cleared', time: new Date() },
  ]
}

function formatTime(date) {
  if (!(date instanceof Date)) date = new Date(date)
  return date.toLocaleTimeString('en-US', { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit' })
}

function messageColor(type) {
  switch (type) {
    case 'error': return 'text-red-400'
    case 'chat': return 'text-cyan-400'
    case 'system': return 'text-yellow-400'
    case 'input': return 'text-green-400'
    case 'info': return 'text-contrast'
    default: return 'text-secondary'
  }
}

function handleKeydown(e) {
  if (e.key === 'Enter') {
    e.preventDefault()
    sendCommand()
  } else if (e.key === 'Escape') {
    emit('close')
  }
}

function onScroll() {
  if (!consoleRef.value) return
  const el = consoleRef.value
  autoScroll.value = el.scrollHeight - el.scrollTop - el.clientHeight < 30
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
          class="bg-bg-raised border border-secondary rounded-xl shadow-2xl w-full max-w-3xl mx-4 flex flex-col"
          style="height: 70vh; max-height: 600px;"
          @keydown="handleKeydown"
        >
          <!-- Header -->
          <div class="flex items-center gap-3 px-4 py-3 border-b border-secondary">
            <TerminalSquareIcon class="w-5 h-5 text-green-400" />
            <span class="font-bold text-contrast">Console — {{ bot.name }}</span>
            <span class="text-sm text-secondary ml-2">{{ bot.host }}:{{ bot.port }}</span>
            <div class="ml-auto flex items-center gap-2">
              <Button :transparent="true" :icon-only="true" :action="clearConsole" title="Clear console">
                <TrashIcon class="w-4 h-4" />
              </Button>
              <Button :transparent="true" :icon-only="true" :action="() => emit('close')" title="Close">
                <XIcon class="w-4 h-4" />
              </Button>
            </div>
          </div>

          <!-- Messages area -->
          <div
            ref="consoleRef"
            class="flex-1 overflow-y-auto px-4 py-2 font-mono text-sm bg-black/30"
            @scroll="onScroll"
          >
            <div
              v-for="msg in messages"
              :key="msg.id"
              class="flex gap-2 py-0.5 leading-relaxed"
            >
              <span class="text-secondary/60 flex-shrink-0 select-none">{{ formatTime(msg.time) }}</span>
              <span :class="messageColor(msg.type)">
                <template v-if="msg.type === 'input'">
                  <span class="text-green-500 mr-1">&gt;</span>
                </template>
                {{ msg.text }}
              </span>
            </div>
            <div v-if="messages.length === 0" class="text-secondary/40 text-center py-8">
              <InfoIcon class="w-6 h-6 mx-auto mb-2 opacity-50" />
              No messages yet. Send a command below.
            </div>
          </div>

          <!-- Command input -->
          <div class="px-4 py-3 border-t border-secondary flex items-center gap-2">
            <span class="text-green-500 font-mono font-bold">&gt;</span>
            <input
              v-model="commandInput"
              type="text"
              class="flex-1 bg-black/30 border border-secondary rounded-lg px-3 py-2 text-sm text-contrast font-mono placeholder:text-secondary/40 focus:outline-none focus:border-green-500/50 transition-colors"
              placeholder="Type a command..."
              autofocus
              @keydown="handleKeydown"
            />
            <Button color="green" :action="sendCommand">
              <SendIcon class="w-4 h-4 mr-1" />
              Send
            </Button>
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
