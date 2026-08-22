<template>
  <div ref="rootEl" class="model-select relative">
    <Button class="model-trigger" @click="toggleOpen">
      <SparklesIcon class="shrink-0 text-brand" />
      <span class="model-trigger-name">{{ selectedModel?.displayName ?? 'Select a model' }}</span>
      <DropdownIcon class="shrink-0" :class="{ 'rotate-180': open }" />
    </Button>

    <div v-if="loadOnMount && error" class="model-status-error">
      Couldn't load models: {{ error }} — check the API key in the model selector.
    </div>
    <div v-else-if="loadOnMount && loading" class="model-status-loading">
      Loading models...
    </div>

    <Teleport to="body">
      <transition name="popover">
        <div
          v-if="open"
          ref="popoverEl"
          class="model-popover card-shadow"
          :style="popoverStyle"
        >
          <div class="popover-header">
            <div class="iconified-input">
              <SearchIcon />
              <input
                ref="searchInput"
                v-model="search"
                type="text"
                placeholder="Search models..."
                class="input"
              />
            </div>
          </div>
          <div class="model-list">
            <div
              v-for="model in filteredModels"
              :key="model.name"
              class="model-row"
              :class="{ 'model-row--selected': model.name === modelValue?.name }"
              @click="selectModel(model)"
            >
              <span class="star" :class="{ 'star--rec': isRecommended(model) }">
                <StarIcon v-if="isRecommended(model)" />
              </span>
              <div class="model-row-main">
                <div class="model-row-title">
                  {{ model.displayName }}
                  <span v-if="model.version" class="model-version">v{{ model.version }}</span>
                </div>
                <div class="model-row-desc">{{ model.description }}</div>
              </div>
              <VTooltip placement="left" :distance="10" :delay="150">
                <template #popper>
                  <div class="model-details">
                    <div class="model-details-title">{{ model.displayName }}</div>
                    <div class="model-details-row">
                      <span>Context window</span>
                      <strong>{{ formatTokenLimit(model.inputTokenLimit) }} tokens</strong>
                    </div>
                    <div class="model-details-row">
                      <span>Max output</span>
                      <strong>{{ formatTokenLimit(model.outputTokenLimit) }} tokens</strong>
                    </div>
                    <div v-if="model.temperature !== undefined" class="model-details-row">
                      <span>Temperature</span>
                      <strong>{{ model.temperature }}</strong>
                    </div>
                    <div v-if="model.topP !== undefined" class="model-details-row">
                      <span>Top-P</span>
                      <strong>{{ model.topP }}</strong>
                    </div>
                    <div v-if="model.topK !== undefined" class="model-details-row">
                      <span>Top-K</span>
                      <strong>{{ model.topK }}</strong>
                    </div>
                  </div>
                </template>
                <span class="hover-info">ⓘ</span>
              </VTooltip>
            </div>
            <div v-if="loading" class="model-empty">Loading models...</div>
            <div v-else-if="error" class="model-empty">{{ error }}</div>
            <div v-else-if="filteredModels.length === 0" class="model-empty">
              No models match your search.
            </div>
          </div>
          <div class="popover-footer">
            <KeyIcon class="shrink-0" />
            <input
              v-model="apiKey"
              type="password"
              placeholder="Gemini API key"
              class="api-key-input"
              @blur="saveKey"
            />
          </div>
        </div>
      </transition>
    </Teleport>
  </div>
</template>

<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import {
  SparklesIcon,
  SearchIcon,
  DropdownIcon,
  StarIcon,
  KeyIcon,
} from '@modrinth/assets'
import { Button } from '@modrinth/ui'
import { VTooltip } from 'floating-vue'
import {
  fetchModels,
  getGeminiApiKey,
  setGeminiApiKey,
  formatTokenLimit,
  isRecommended,
} from '@/helpers/gemini.js'

const props = defineProps({
  modelValue: {
    type: Object,
    default: null,
  },
  loadOnMount: {
    type: Boolean,
    default: false,
  },
})

const emit = defineEmits(['update:model-value'])

const rootEl = ref(null)
const popoverEl = ref(null)
const open = ref(false)
const search = ref('')
const models = ref([])
const loading = ref(false)
const error = ref('')
const apiKey = ref(getGeminiApiKey())
const searchInput = ref(null)
const popoverStyle = ref({})

const selectedModel = computed(() => props.modelValue)

const filteredModels = computed(() => {
  const term = search.value.trim().toLowerCase()
  if (!term) return models.value
  return models.value.filter(
    (model) =>
      model.displayName.toLowerCase().includes(term) ||
      model.name.toLowerCase().includes(term) ||
      model.description.toLowerCase().includes(term),
  )
})

const POPOVER_WIDTH = 416

const positionPopover = () => {
  if (!open.value || !rootEl.value) return
  const rect = rootEl.value.getBoundingClientRect()
  const viewportHeight = window.innerHeight
  const viewportWidth = window.innerWidth

  let top = rect.bottom + 8
  let maxHeight = viewportHeight - top - 8
  if (maxHeight < 300) {
    top = Math.max(8, rect.top - 8 - Math.min(560, viewportHeight - 16))
    maxHeight = viewportHeight - top - 8
  }

  const left = Math.min(rect.left, Math.max(8, viewportWidth - POPOVER_WIDTH - 8))

  popoverStyle.value = {
    position: 'fixed',
    top: `${top}px`,
    left: `${left}px`,
    width: `${Math.min(POPOVER_WIDTH, viewportWidth - 16)}px`,
    maxHeight: `${maxHeight}px`,
  }
}

const onWindowScroll = () => {
  if (open.value) positionPopover()
}

const onWindowResize = () => {
  if (open.value) positionPopover()
}

const onWindowClick = (event) => {
  if (!open.value) return
  if (popoverEl.value?.contains(event.target)) return
  if (rootEl.value?.contains(event.target)) return
  open.value = false
}

const onKeyDown = (event) => {
  if (event.key === 'Escape' && open.value) {
    open.value = false
  }
}

const toggleOpen = async () => {
  open.value = !open.value
  if (open.value) {
    search.value = ''
    if (models.value.length === 0) {
      await loadModels()
    }
    await nextTick()
    positionPopover()
    requestAnimationFrame(() => searchInput.value?.focus())
  }
}

const selectModel = (model) => {
  emit('update:model-value', model)
  open.value = false
}

const saveKey = () => {
  setGeminiApiKey(apiKey.value)
  // If a key is set/changed, refresh the model list so it picks up the new key
  if (models.value.length > 0 || error.value) {
    loadModels()
  }
}

const loadModels = async () => {
  loading.value = true
  error.value = ''
  try {
    const all = await fetchModels()
    models.value = all.sort((a, b) => {
      const aRec = isRecommended(a) ? 0 : 1
      const bRec = isRecommended(b) ? 0 : 1
      if (aRec !== bRec) return aRec - bRec
      return a.displayName.localeCompare(b.displayName)
    })
    if (!props.modelValue && models.value.length > 0) {
      emit('update:model-value', models.value[0])
    }
  } catch (err) {
    error.value = err.message
  } finally {
    loading.value = false
    if (open.value) {
      await nextTick()
      positionPopover()
    }
  }
}

watch(
  () => props.modelValue,
  (model) => {
    if (!model && models.value.length > 0) {
      emit('update:model-value', models.value[0])
    }
  },
)

onMounted(() => {
  if (props.loadOnMount && models.value.length === 0) {
    loadModels()
  }
  window.addEventListener('scroll', onWindowScroll, true)
  window.addEventListener('resize', onWindowResize)
  window.addEventListener('click', onWindowClick)
  window.addEventListener('keydown', onKeyDown)
})

onBeforeUnmount(() => {
  window.removeEventListener('scroll', onWindowScroll, true)
  window.removeEventListener('resize', onWindowResize)
  window.removeEventListener('click', onWindowClick)
  window.removeEventListener('keydown', onKeyDown)
})
</script>

<style scoped lang="scss">
.model-select {
  .model-trigger {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    border-radius: var(--radius-md);
    padding: 0.5rem 0.9rem;

    .rotate-180 {
      transform: rotate(180deg);
      transition: transform 0.2s ease;
    }
  }

  .model-trigger-name {
    font-weight: 600;
    color: var(--color-contrast);
    max-width: 16rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.model-status-error {
  position: absolute;
  top: calc(100% + 0.35rem);
  left: 0;
  max-width: 24rem;
  font-size: 0.78rem;
  color: var(--color-red);
  background-color: var(--color-raised-bg);
  border: 1px solid var(--color-button-border);
  border-radius: var(--radius-md);
  padding: 0.4rem 0.6rem;
  z-index: 500;
}

.model-status-loading {
  position: absolute;
  top: calc(100% + 0.35rem);
  left: 0;
  font-size: 0.78rem;
  color: var(--color-secondary);
}

.model-popover {
  z-index: 10000;
  background-color: var(--color-super-raised-bg);
  border: 1px solid var(--color-button-border);
  border-radius: var(--radius-lg);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.popover-header {
  padding: 0.6rem;
  border-bottom: 1px solid var(--color-button-border);

  .iconified-input {
    width: 100%;
  }
}

.model-list {
  flex: 1;
  overflow-y: auto;
  padding: 0.3rem;
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}

.model-row {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.55rem 0.6rem;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background-color 0.15s ease;

  &:hover {
    background-color: var(--color-button-bg);
  }

  &--selected {
    background-color: var(--color-brand-highlight);

    .model-row-title {
      color: var(--color-brand);
    }
  }
}

.star {
  width: 1.1rem;
  height: 1.1rem;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: var(--color-button-border);

  &--rec {
    color: #ffc107;
  }
}

.model-row-main {
  flex: 1;
  min-width: 0;
}

.model-row-title {
  font-weight: 600;
  font-size: 0.9rem;
  color: var(--color-contrast);
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.model-version {
  font-size: 0.7rem;
  color: var(--color-secondary);
  background-color: var(--color-button-bg);
  border-radius: 0.3rem;
  padding: 0.05rem 0.35rem;
}

.model-row-desc {
  font-size: 0.75rem;
  color: var(--color-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.hover-info {
  color: var(--color-secondary);
  font-size: 0.8rem;
  flex-shrink: 0;
  opacity: 0.6;
}

.model-empty {
  padding: 1rem;
  text-align: center;
  color: var(--color-secondary);
  font-size: 0.85rem;
}

.popover-footer {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.6rem;
  border-top: 1px solid var(--color-button-border);
  color: var(--color-secondary);

  .api-key-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: var(--color-contrast);
    font-size: 0.8rem;

    &::placeholder {
      color: var(--color-secondary);
    }
  }
}

.popover-enter-active,
.popover-leave-active {
  transition:
    opacity 0.18s ease,
    transform 0.18s ease;
}

.popover-enter-from,
.popover-leave-to {
  opacity: 0;
  transform: translateY(-0.35rem) scale(0.98);
}

:deep(.v-popper__popper) {
  z-index: 20000;
}

.model-details {
  min-width: 13rem;
  padding: 0.15rem;

  .model-details-title {
    font-weight: 700;
    margin-bottom: 0.4rem;
    color: var(--color-contrast);
  }

  .model-details-row {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    font-size: 0.8rem;
    padding: 0.15rem 0;
    color: var(--color-secondary);

    strong {
      color: var(--color-contrast);
      font-weight: 600;
    }
  }
}
</style>
