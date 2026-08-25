<template>
  <ModalWrapper ref="modal" header="Look Up Player Skin" :has-to-type="false">
    <div class="skin-lookup-body">
      <!-- Input -->
      <div class="input-row">
        <div class="iconified-input flex-1">
          <SearchIcon class="text-lg" />
          <input
            v-model="username"
            placeholder="Enter Minecraft username..."
            type="text"
            class="input-base"
            @keydown.enter="lookup"
          />
        </div>
        <Button color="brand" :disabled="!username.trim() || loading" @click="lookup">
          <SpinnerIcon v-if="loading" class="animate-spin" />
          {{ loading ? 'Looking...' : 'Look Up' }}
        </Button>
      </div>

      <!-- Error -->
      <div v-if="error" class="error-box">
        <p class="m-0">{{ error }}</p>
      </div>

      <!-- Result -->
      <div v-if="result" class="result-card">
        <div class="result-left">
          <img
            :src="result.headUrl"
            :alt="result.name"
            class="skin-avatar"
          />
          <img
            :src="result.bodyUrl"
            :alt="result.name + ' full body'"
            class="skin-body"
          />
        </div>
        <div class="result-right">
          <h3 class="m-0 text-lg font-bold text-contrast">{{ result.name }}</h3>
          <p class="m-0 text-sm text-secondary">UUID: {{ result.uuid }}</p>
          <p class="m-0 text-sm text-secondary">Model: {{ result.slim ? 'Slim (Alex)' : 'Classic (Steve)' }}</p>
          <div class="result-actions">
            <Button color="brand" :disabled="selecting" @click="selectSkin">
              <CheckIcon v-if="!selecting" />
              <SpinnerIcon v-if="selecting" class="animate-spin" />
              {{ selecting ? 'Saving...' : 'Use This Skin' }}
            </Button>
          </div>
          <p v-if="selectedMsg" class="m-0 text-sm" style="color: #a3e635">{{ selectedMsg }}</p>
        </div>
      </div>

      <!-- Empty state -->
      <div v-if="!result && !loading && !error" class="empty-state">
        <p class="m-0 text-secondary">Type a Minecraft username to look up their skin</p>
      </div>
    </div>
  </ModalWrapper>
</template>

<script setup>
import { ref } from 'vue'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { Button } from '@modrinth/ui'
import { SearchIcon, CheckIcon, SpinnerIcon } from '@modrinth/assets'

const modal = ref(null)
const username = ref('')
const loading = ref(false)
const error = ref(null)
const result = ref(null)
const selecting = ref(false)
const selectedMsg = ref('')

const emit = defineEmits(['skin-selected'])

async function lookup() {
  const name = username.value.trim()
  if (!name) return

  loading.value = true
  error.value = null
  result.value = null
  selectedMsg.value = ''

  try {
    // ashcon.app returns everything in one call: uuid, skin url, model type
    const resp = await fetch(`https://api.ashcon.app/mojang/v2/user/${encodeURIComponent(name)}`)

    if (resp.status === 404) {
      error.value = `Player "${name}" not found. Check the spelling.`
      return
    }
    if (!resp.ok) {
      error.value = `API error (${resp.status}). Try again later.`
      return
    }

    const data = await resp.json()

    const isSlim = data.textures?.slim || false
    const skinUrl = data.textures?.skin?.url || null

    result.value = {
      name: data.username,
      uuid: data.uuid.replace(/-/g, ''),
      slim: isSlim,
      headUrl: `https://mc-heads.net/avatar/${data.username}/128`,
      bodyUrl: `https://mc-heads.net/body/${data.username}/160`,
      skinUrl,
    }
  } catch (err) {
    error.value = `Network error: ${err.message}`
  } finally {
    loading.value = false
  }
}

async function selectSkin() {
  if (!result.value) return
  selecting.value = true

  try {
    emit('skin-selected', {
      name: result.value.name,
      uuid: result.value.uuid,
      modelType: result.value.slim ? 'slim' : 'classic',
      headUrl: result.value.headUrl,
      skinUrl: result.value.skinUrl,
    })
    selectedMsg.value = `✓ ${result.value.name} selected!`
    setTimeout(() => modal.value?.hide(), 800)
  } catch (err) {
    error.value = `Failed: ${err.message}`
  } finally {
    selecting.value = false
  }
}

function show() {
  // Reset state on open
  username.value = ''
  result.value = null
  error.value = null
  loading.value = false
  selectedMsg.value = ''
  modal.value?.show()
}

defineExpose({ show })
</script>

<style scoped>
.skin-lookup-body {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  min-width: 440px;
  max-width: 520px;
}

.input-row {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.error-box {
  background: rgba(255, 60, 60, 0.12);
  border: 1px solid rgba(255, 60, 60, 0.3);
  border-radius: 8px;
  padding: 0.75rem;
  color: #ff6b6b;
  font-size: 0.875rem;
}

.result-card {
  display: flex;
  gap: 1.25rem;
  padding: 1rem;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 12px;
}

.result-left {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
}

.skin-avatar {
  width: 128px;
  height: 128px;
  border-radius: 8px;
  image-rendering: pixelated;
  background: var(--color-button-bg);
}

.skin-body {
  width: 80px;
  height: 160px;
  border-radius: 6px;
  image-rendering: pixelated;
  background: var(--color-button-bg);
}

.result-right {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  justify-content: center;
}

.result-actions {
  margin-top: 0.5rem;
}

.empty-state {
  text-align: center;
  padding: 2rem;
}
</style>
