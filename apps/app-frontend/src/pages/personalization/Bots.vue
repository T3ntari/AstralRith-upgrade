<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import {
  SearchIcon,
  PlusIcon,
  CodeIcon,
  TerminalSquareIcon,
  PlayIcon,
  XIcon,
  LinkIcon,
} from '@modrinth/assets'
import { Button } from '@modrinth/ui'
import { useBotsStore } from '@/store/bots.js'
import BotCard from '@/components/bots/BotCard.vue'
import BotConsole from '@/components/bots/BotConsole.vue'
import BotTaskPanel from '@/components/bots/BotTaskPanel.vue'
import BotScriptManager from '@/components/bots/BotScriptManager.vue'
import NewBotModal from '@/components/bots/NewBotModal.vue'

const store = useBotsStore()

// Modal states
const showNewBot = ref(false)
const editingBot = ref(null)
const showConsole = ref(false)
const consoleBot = ref(null)
const showTaskPanel = ref(false)
const taskBot = ref(null)
const showScriptManager = ref(false)

onMounted(() => {
  store.refresh()
})

// Keyboard shortcuts
function handleKeydown(e) {
  // Ctrl+N: New bot
  if ((e.ctrlKey || e.metaKey) && e.key === 'n' && !showNewBot.value && !showConsole.value && !showTaskPanel.value) {
    e.preventDefault()
    openNewBot()
  }
  // Escape: close any open modal
  if (e.key === 'Escape') {
    closeAll()
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})
onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})

function closeAll() {
  showNewBot.value = false
  editingBot.value = null
  showConsole.value = false
  consoleBot.value = null
  showTaskPanel.value = false
  taskBot.value = null
  showScriptManager.value = false
}

function openNewBot() {
  editingBot.value = null
  showNewBot.value = true
}

function openEditBot(bot) {
  editingBot.value = bot
  showNewBot.value = true
}

function openConsole(bot) {
  consoleBot.value = bot
  showConsole.value = true
}

function openTasks(bot) {
  taskBot.value = bot
  showTaskPanel.value = true
}

async function handleSaveBot(data) {
  if (editingBot.value) {
    await store.updateBot(editingBot.value.id, data)
  } else {
    await store.createBot(data)
  }
  showNewBot.value = false
  editingBot.value = null
}

async function handleConnect(botId) {
  await store.updateBot(botId, { status: 'connected', ping: Math.floor(Math.random() * 100) + 20, health: 20 })
}

async function handleDisconnect(botId) {
  await store.updateBot(botId, { status: 'disconnected', ping: null, health: null, position: null })
}

async function handleDuplicate(botId) {
  await store.duplicateBot(botId)
}

async function handleDeleteBot(bot) {
  await store.deleteBot(bot.id)
}

function handleSendCommand(data) {
  // In a real implementation this would send to the Tauri backend
  console.log('Send command:', data)
}
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Header -->
    <div class="flex items-center gap-3 px-6 py-4 border-b border-secondary">
      <TerminalSquareIcon class="w-6 h-6 text-brand" />
      <h1 class="text-xl font-bold text-contrast m-0">Bots</h1>
      <span class="text-sm text-secondary">{{ store.bots.length }} bot(s)</span>

      <div class="ml-auto flex items-center gap-2">
        <!-- Search -->
        <div class="relative">
          <SearchIcon class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-secondary" />
          <input
            v-model="store.searchQuery"
            type="text"
            class="pl-9 pr-3 py-1.5 bg-bg-base border border-secondary rounded-lg text-sm text-contrast placeholder:text-secondary/50 focus:outline-none focus:border-brand/50 transition-colors w-56"
            placeholder="Search bots..."
          />
          <button
            v-if="store.searchQuery"
            class="absolute right-2 top-1/2 -translate-y-1/2 text-secondary hover:text-contrast transition-colors"
            @click="store.searchQuery = ''"
          >
            <XIcon class="w-3.5 h-3.5" />
          </button>
        </div>

        <Button :action="() => showScriptManager = true">
          <CodeIcon class="w-4 h-4 mr-1" />
          Scripts
        </Button>
        <Button color="green" :action="openNewBot">
          <PlusIcon class="w-4 h-4 mr-1" />
          New Bot
        </Button>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto px-6 py-4">
      <!-- Loading -->
      <div v-if="store.loading" class="flex items-center justify-center py-16 text-secondary">
        Loading bots...
      </div>

      <!-- Empty state -->
      <div
        v-else-if="store.bots.length === 0"
        class="flex flex-col items-center justify-center py-16 gap-4"
      >
        <TerminalSquareIcon class="w-16 h-16 text-secondary opacity-30" />
        <h3 class="m-0 text-lg text-contrast">No bots yet</h3>
        <p class="m-0 text-secondary text-center max-w-sm">
          Create a new bot to automate tasks on Minecraft servers.
        </p>
        <Button color="green" :action="openNewBot">
          <PlusIcon class="w-4 h-4 mr-1" />
          Create your first bot
        </Button>
      </div>

      <template v-else>
        <!-- Connected bots -->
        <div v-if="store.connectedBots.length > 0" class="mb-6">
          <div class="flex items-center gap-2 mb-3">
            <div class="w-2.5 h-2.5 rounded-full bg-green-400 animate-pulse" />
            <h2 class="text-sm font-semibold text-contrast uppercase tracking-wider m-0">
              Connected
            </h2>
            <span class="text-xs text-secondary">({{ store.connectedBots.length }})</span>
          </div>
          <div class="space-y-2">
            <BotCard
              v-for="bot in store.connectedBots"
              :key="bot.id"
              :bot="bot"
              @connect="handleConnect"
              @disconnect="handleDisconnect"
              @console="openConsole"
              @tasks="openTasks"
              @edit="openEditBot"
              @duplicate="handleDuplicate"
              @delete="handleDeleteBot"
            />
          </div>
        </div>

        <!-- Disconnected bots -->
        <div v-if="store.disconnectedBots.length > 0" class="mb-6">
          <div class="flex items-center gap-2 mb-3">
            <div class="w-2.5 h-2.5 rounded-full bg-secondary/50" />
            <h2 class="text-sm font-semibold text-contrast uppercase tracking-wider m-0">
              Disconnected
            </h2>
            <span class="text-xs text-secondary">({{ store.disconnectedBots.length }})</span>
          </div>
          <div class="space-y-2">
            <BotCard
              v-for="bot in store.disconnectedBots"
              :key="bot.id"
              :bot="bot"
              @connect="handleConnect"
              @disconnect="handleDisconnect"
              @console="openConsole"
              @tasks="openTasks"
              @edit="openEditBot"
              @duplicate="handleDuplicate"
              @delete="handleDeleteBot"
            />
          </div>
        </div>

        <!-- No results -->
        <div
          v-if="store.filteredBots.length === 0 && store.searchQuery"
          class="text-center py-12 text-secondary/50"
        >
          <SearchIcon class="w-8 h-8 mx-auto mb-2 opacity-40" />
          <p>No bots match "{{ store.searchQuery }}"</p>
        </div>
      </template>
    </div>

    <!-- Modals -->
    <NewBotModal
      :visible="showNewBot"
      :edit-bot="editingBot"
      @close="showNewBot = false; editingBot = null"
      @save="handleSaveBot"
    />
    <BotConsole
      :bot="consoleBot"
      :visible="showConsole"
      @close="showConsole = false; consoleBot = null"
      @send-command="handleSendCommand"
    />
    <BotTaskPanel
      :bot="taskBot"
      :visible="showTaskPanel"
      @close="showTaskPanel = false; taskBot = null"
      @start-tasks="(data) => console.log('Start tasks:', data)"
      @stop-tasks="(data) => console.log('Stop tasks:', data)"
    />
    <BotScriptManager
      :visible="showScriptManager"
      @close="showScriptManager = false"
      @import-script="(files) => console.log('Import scripts:', files)"
    />
  </div>
</template>
