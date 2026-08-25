<script setup>
import { ref, reactive, watch } from 'vue'
import {
  XIcon,
  PlusIcon,
  UserIcon,
  GlobeIcon,
  ServerIcon,
  GameIcon,
  ShieldIcon,
  CodeIcon,
} from '@modrinth/assets'
import { Button, DropdownSelect } from '@modrinth/ui'

const props = defineProps({
  visible: { type: Boolean, default: false },
  editBot: { type: Object, default: null },
})

const emit = defineEmits(['close', 'save'])

const isEditing = ref(false)

const form = reactive({
  name: '',
  host: 'localhost',
  port: 25565,
  version: '1.21.4',
  authType: 'offline',
  username: '',
  script: '',
  plugins: [],
})

const errors = reactive({
  name: '',
  host: '',
  port: '',
  username: '',
})

const versionOptions = ['1.21.4', '1.21.2', '1.20.6', '1.20.4', '1.20.1', '1.19.4', '1.16.5', '1.12.2', '1.8.9']
const authOptions = ['offline', 'microsoft']

// Watch for edit bot changes
watch(() => props.visible, (val) => {
  if (val && props.editBot) {
    isEditing.value = true
    Object.assign(form, {
      name: props.editBot.name || '',
      host: props.editBot.host || 'localhost',
      port: props.editBot.port || 25565,
      version: props.editBot.version || '1.21.4',
      authType: props.editBot.authType || 'offline',
      username: props.editBot.username || '',
      script: props.editBot.script || '',
      plugins: props.editBot.plugins || [],
    })
  } else if (val) {
    isEditing.value = false
    Object.assign(form, {
      name: '',
      host: 'localhost',
      port: 25565,
      version: '1.21.4',
      authType: 'offline',
      username: '',
      script: '',
      plugins: [],
    })
  }
  clearErrors()
})

function clearErrors() {
  errors.name = ''
  errors.host = ''
  errors.port = ''
  errors.username = ''
}

function validate() {
  clearErrors()
  let valid = true

  if (!form.name.trim()) {
    errors.name = 'Bot name is required'
    valid = false
  }
  if (!form.host.trim()) {
    errors.host = 'Server host is required'
    valid = false
  }
  const port = parseInt(form.port)
  if (isNaN(port) || port < 1 || port > 65535) {
    errors.port = 'Port must be 1-65535'
    valid = false
  }
  if (!form.username.trim()) {
    errors.username = 'Username is required'
    valid = false
  }

  return valid
}

function save() {
  if (!validate()) return
  emit('save', {
    ...form,
    port: parseInt(form.port),
  })
}

function handleKeydown(e) {
  if (e.key === 'Escape') {
    emit('close')
  }
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
          class="bg-bg-raised border border-secondary rounded-xl shadow-2xl w-full max-w-lg mx-4"
          @keydown="handleKeydown"
        >
          <!-- Header -->
          <div class="flex items-center gap-3 px-4 py-3 border-b border-secondary">
            <PlusIcon v-if="!isEditing" class="w-5 h-5 text-green-400" />
            <span class="font-bold text-contrast text-lg">{{ isEditing ? 'Edit Bot' : 'New Bot' }}</span>
            <div class="ml-auto">
              <Button :transparent="true" :icon-only="true" :action="() => emit('close')">
                <XIcon class="w-4 h-4" />
              </Button>
            </div>
          </div>

          <!-- Form -->
          <div class="px-4 py-4 space-y-3">
            <!-- Bot Name -->
            <div>
              <label class="block text-sm text-secondary mb-1">
                <UserIcon class="w-3.5 h-3.5 inline mr-1" />
                Bot Name
              </label>
              <input
                v-model="form.name"
                type="text"
                class="w-full bg-black/30 border rounded-lg px-3 py-2 text-sm text-contrast focus:outline-none focus:border-brand/50 transition-colors"
                :class="errors.name ? 'border-red-500/50' : 'border-secondary'"
                placeholder="e.g. BuilderBot"
              />
              <p v-if="errors.name" class="text-xs text-red-400 mt-1 m-0">{{ errors.name }}</p>
            </div>

            <!-- Server Host + Port -->
            <div class="grid grid-cols-3 gap-3">
              <div class="col-span-2">
                <label class="block text-sm text-secondary mb-1">
                  <GlobeIcon class="w-3.5 h-3.5 inline mr-1" />
                  Server Host
                </label>
                <input
                  v-model="form.host"
                  type="text"
                  class="w-full bg-black/30 border rounded-lg px-3 py-2 text-sm text-contrast focus:outline-none focus:border-brand/50 transition-colors"
                  :class="errors.host ? 'border-red-500/50' : 'border-secondary'"
                  placeholder="localhost"
                />
                <p v-if="errors.host" class="text-xs text-red-400 mt-1 m-0">{{ errors.host }}</p>
              </div>
              <div>
                <label class="block text-sm text-secondary mb-1">
                  <ServerIcon class="w-3.5 h-3.5 inline mr-1" />
                  Port
                </label>
                <input
                  v-model.number="form.port"
                  type="number"
                  min="1"
                  max="65535"
                  class="w-full bg-black/30 border rounded-lg px-3 py-2 text-sm text-contrast focus:outline-none focus:border-brand/50 transition-colors"
                  :class="errors.port ? 'border-red-500/50' : 'border-secondary'"
                />
                <p v-if="errors.port" class="text-xs text-red-400 mt-1 m-0">{{ errors.port }}</p>
              </div>
            </div>

            <!-- Version + Auth -->
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="block text-sm text-secondary mb-1">
                  <GameIcon class="w-3.5 h-3.5 inline mr-1" />
                  Minecraft Version
                </label>
                <DropdownSelect
                  v-model="form.version"
                  :options="versionOptions"
                  name="version-select"
                />
              </div>
              <div>
                <label class="block text-sm text-secondary mb-1">
                  <ShieldIcon class="w-3.5 h-3.5 inline mr-1" />
                  Auth Type
                </label>
                <DropdownSelect
                  v-model="form.authType"
                  :options="authOptions"
                  name="auth-select"
                />
              </div>
            </div>

            <!-- Username -->
            <div>
              <label class="block text-sm text-secondary mb-1">
                <UserIcon class="w-3.5 h-3.5 inline mr-1" />
                Username
              </label>
              <input
                v-model="form.username"
                type="text"
                class="w-full bg-black/30 border rounded-lg px-3 py-2 text-sm text-contrast focus:outline-none focus:border-brand/50 transition-colors"
                :class="errors.username ? 'border-red-500/50' : 'border-secondary'"
                :placeholder="form.authType === 'offline' ? 'Player123' : 'Sign in with Microsoft'"
              />
              <p v-if="errors.username" class="text-xs text-red-400 mt-1 m-0">{{ errors.username }}</p>
            </div>

            <!-- Script -->
            <div>
              <label class="block text-sm text-secondary mb-1">
                <CodeIcon class="w-3.5 h-3.5 inline mr-1" />
                Script (optional)
              </label>
              <input
                v-model="form.script"
                type="text"
                class="w-full bg-black/30 border border-secondary rounded-lg px-3 py-2 text-sm text-contrast focus:outline-none focus:border-brand/50 transition-colors"
                placeholder="e.g. farm.js"
              />
            </div>
          </div>

          <!-- Footer -->
          <div class="flex items-center justify-end gap-2 px-4 py-3 border-t border-secondary">
            <Button :action="() => emit('close')">Cancel</Button>
            <Button color="green" :action="save">
              {{ isEditing ? 'Save Changes' : 'Create Bot' }}
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
