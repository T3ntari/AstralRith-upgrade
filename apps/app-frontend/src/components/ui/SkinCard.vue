<script setup>
import { ref, computed } from 'vue'
import { HeartIcon, MoreVerticalIcon, CopyIcon, TrashIcon, EditIcon } from '@modrinth/assets'
import { Button, ButtonStyled } from '@modrinth/ui'
import Skin3DPreview from '@/components/ui/Skin3DPreview.vue'
import ContextMenu from '@/components/ui/ContextMenu.vue'

const props = defineProps({
  skin: { type: Object, required: true },
})

const emit = defineEmits(['action'])

const contextMenu = ref(null)
const previewRef = ref(null)
const isHovering = ref(false)


const contextOptions = [
  { name: 'apply' },
  { name: 'favorite' },
  { name: 'duplicate' },
  { name: 'rename' },
  { name: 'divider' },
  { name: 'delete', color: 'red' },
]

const sourceLabel = computed(() => {
  switch (props.skin.source) {
    case 'account': return 'Account'
    case 'imported': return 'Imported'
    case 'edited': return 'Edited'
    default: return 'Local'
  }
})

function handleContext(e) {
  e.preventDefault()
  contextMenu.value?.showMenu(e, props.skin, contextOptions)
}

function handleContextAction(event) {
  emit('action', event.option)
}
</script>

<template>
  <div
    class="skin-card group"
    @contextmenu="handleContext"
    @mouseenter="isHovering = true"
    @mouseleave="isHovering = false"
  >
    <!-- 3D Preview -->
    <div class="preview-container">
      <Skin3DPreview
        ref="previewRef"
        :skin-data="skin.assetUrl"
        :model-type="skin.modelType"
        :width="160"
        :height="200"
        :auto-rotate="!isHovering"
        :animate="true"
        :interactive="true"
        class="w-full h-full"
      />

      <!-- Favorite badge -->
      <button
        v-if="skin.favorite"
        class="favorite-badge"
        @click.stop="emit('action', 'favorite')"
        :title="skin.favorite ? 'Remove from favorites' : 'Add to favorites'"
      >
        <HeartIcon class="w-4 h-4" fill="currentColor" />
      </button>
    </div>

    <!-- Info -->
    <div class="p-2">
      <div class="flex items-center gap-1 overflow-hidden">
        <span class="font-semibold text-sm text-contrast truncate">{{ skin.name }}</span>
      </div>
      <div class="flex items-center gap-1 mt-1">
        <span class="text-xs text-secondary bg-button-bg rounded px-1.5 py-0.5">
          {{ skin.modelType }}
        </span>
        <span class="text-xs text-secondary bg-button-bg rounded px-1.5 py-0.5">
          {{ sourceLabel }}
        </span>
      </div>
    </div>

    <!-- Actions overlay -->
    <div
      class="absolute top-2 right-2 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity"
    >
      <Button
        icon-only
        class="action-btn"
        @click.stop="emit('action', 'favorite')"
        :title="skin.favorite ? 'Remove from favorites' : 'Add to favorites'"
      >
        <HeartIcon
          class="w-3.5 h-3.5"
          :fill="skin.favorite ? 'currentColor' : 'none'"
          :class="skin.favorite ? 'text-red-400' : 'text-secondary'"
        />
      </Button>
      <Button
        icon-only
        class="action-btn"
        title="More options"
        @click.stop="handleContext"
      >
        <MoreVerticalIcon class="w-3.5 h-3.5 text-secondary" />
      </Button>
    </div>

    <!-- Apply button (bottom) -->
    <div class="absolute bottom-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity">
      <ButtonStyled color="brand" type="outlined" class="text-xs">
        <button
          class="!px-2 !py-1"
          @click.stop="emit('action', 'apply')"
        >
          Apply
        </button>
      </ButtonStyled>
    </div>

    <ContextMenu ref="contextMenu" @option-clicked="handleContextAction">
      <template #apply>
        <div class="flex items-center gap-2">
          <span>Apply</span>
        </div>
      </template>
      <template #favorite>
        <div class="flex items-center gap-2">
          <HeartIcon class="w-4 h-4" :fill="skin.favorite ? 'currentColor' : 'none'" />
          <span>{{ skin.favorite ? 'Remove favorite' : 'Favorite' }}</span>
        </div>
      </template>
      <template #duplicate>
        <div class="flex items-center gap-2">
          <CopyIcon class="w-4 h-4" />
          <span>Duplicate</span>
        </div>
      </template>
      <template #rename>
        <div class="flex items-center gap-2">
          <EditIcon class="w-4 h-4" />
          <span>Rename</span>
        </div>
      </template>
      <template #delete>
        <div class="flex items-center gap-2">
          <TrashIcon class="w-4 h-4" />
          <span>Delete</span>
        </div>
      </template>
    </ContextMenu>
  </div>
</template>

<style scoped lang="scss">
.skin-card {
  position: relative;
  border-radius: 12px;
  background: var(--color-raised-bg);
  border: 1px solid var(--color-button-bg);
  overflow: hidden;
  cursor: pointer;
  transition: border-color 0.2s, box-shadow 0.2s;

  &:hover {
    border-color: var(--color-button-border);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }
}

.preview-container {
  position: relative;
  width: 100%;
  aspect-ratio: 4 / 5;
  overflow: hidden;
}

.favorite-badge {
  position: absolute;
  top: 6px;
  left: 6px;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: rgba(0, 0, 0, 0.5);
  border: none;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-red, #ef4444);
  cursor: pointer;
  z-index: 2;
}

.action-btn {
  background: rgba(0, 0, 0, 0.5) !important;
  border: none !important;
  border-radius: 6px !important;
  width: 28px !important;
  height: 28px !important;
  min-width: 28px !important;
  min-height: 28px !important;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
