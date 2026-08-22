<script setup>
import { ref } from 'vue'
import { Button } from '@modrinth/ui'
import { KeyIcon } from '@modrinth/assets'
import { getCfApiKey, setCfApiKey, clearCfApiKey } from '@/helpers/curseforge.js'
import { handleError } from '@/store/notifications.js'

const apiKey = ref(getCfApiKey())
const saved = ref(false)

async function saveKey() {
  try {
    setCfApiKey(apiKey.value)
    saved.value = true
    setTimeout(() => {
      saved.value = false
    }, 2000)
  } catch (err) {
    handleError({ message: `Failed to save CurseForge API key: ${err.message}` })
  }
}

async function removeKey() {
  clearCfApiKey()
  apiKey.value = ''
}
</script>

<template>
  <h2 class="m-0 text-lg font-extrabold text-contrast">CurseForge API key</h2>
  <p class="m-0 mt-1 mb-2 leading-tight text-secondary">
    Used by CurseForge mode on the Discover page to search, browse and install mods and modpacks
    from CurseForge. Get a free key at
    <a class="text-primary underline" href="https://console.curseforge.com" target="_blank">
      console.curseforge.com
    </a>
    . The key is stored only on this device.
  </p>

  <div class="m-1 my-2">
    <div class="iconified-input w-full">
      <KeyIcon />
      <input
        id="cfApiKey"
        v-model="apiKey"
        type="password"
        class="input"
        placeholder="Paste your CurseForge API key..."
      />
      <Button class="r-btn" @click="saveKey">
        {{ saved ? 'Saved' : 'Save' }}
      </Button>
      <Button v-if="apiKey" class="r-btn" @click="removeKey"> Remove </Button>
    </div>
  </div>
</template>
