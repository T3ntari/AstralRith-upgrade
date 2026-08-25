<template>
  <div
    class="card-shadow p-4 bg-bg-raised rounded-xl flex gap-3 group cursor-pointer hover:brightness-90 transition-all cf-search-card"
    @click="handleOpen"
  >
    <div class="icon w-[96px] h-[96px] relative">
      <Avatar :src="mod.icon_url" size="96px" class="search-icon origin-top transition-all" loading="lazy" />
    </div>
    <div class="flex flex-col gap-2 overflow-hidden">
      <div class="gap-2 overflow-hidden no-wrap text-ellipsis">
        <span class="text-lg font-extrabold text-contrast m-0 leading-none">
          {{ mod.name }}
        </span>
        <span v-if="authorName" class="text-secondary"> by {{ authorName }}</span>
      </div>
      <div class="m-0 line-clamp-2">
        {{ mod.summary }}
      </div>
      <div v-if="categories.length > 0" class="mt-auto flex items-center gap-1 no-wrap">
        <TagsIcon class="h-4 w-4 shrink-0" />
        <div
          v-for="cat in categories.slice(0, 4)"
          :key="cat"
          class="text-sm font-semibold text-secondary flex gap-1 px-[0.375rem] py-0.5 bg-button-bg rounded-full"
        >
          {{ cat }}
        </div>
        <span
          v-if="categories.length > 4"
          class="text-sm font-semibold text-secondary flex gap-1 px-[0.375rem] py-0.5 bg-button-bg rounded-full"
        >
          +{{ categories.length - 4 }}
        </span>
      </div>
    </div>
    <div class="flex flex-col gap-2 items-end shrink-0 ml-auto">
      <div class="flex items-center gap-2">
        <DownloadIcon class="shrink-0" />
        <span>
          {{ formatDownloadCount(mod.download_count) }}
          <span class="text-secondary">downloads</span>
        </span>
      </div>
      <div class="flex items-center gap-2">
        <HeartIcon class="shrink-0" />
        <span>
          {{ formatDownloadCount(mod.download_count) }}
          <span class="text-secondary">followers</span>
        </span>
      </div>
      <div class="mt-auto relative">
        <div class="absolute bottom-0 right-0 w-fit">
          <ButtonStyled color="brand" type="outlined">
            <button
              :disabled="installed || installing"
              class="shrink-0 no-wrap"
              @click.stop="handleInstall"
            >
              <template v-if="!installed">
                <DownloadIcon v-if="modpack || instance" />
                <PlusIcon v-else />
              </template>
              <CheckIcon v-else />
              {{
                installing
                  ? 'Installing'
                  : installed
                    ? 'Installed'
                    : instance
                      ? modpack
                        ? 'Install pack'
                        : 'Install'
                      : 'Open in CurseForge'
              }}
            </button>
          </ButtonStyled>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { TagsIcon, DownloadIcon, HeartIcon, PlusIcon, CheckIcon } from '@modrinth/assets'
import { Avatar, ButtonStyled } from '@modrinth/ui'
import { computed } from 'vue'
import { openUrl } from '@tauri-apps/plugin-opener'
import { formatDownloadCount, projectUrl } from '@/helpers/curseforge.js'

const props = defineProps({
  mod: {
    type: Object,
    required: true,
  },
  instance: {
    type: Object,
    default: null,
  },
  modpack: {
    type: Boolean,
    default: false,
  },
  installed: {
    type: Boolean,
    default: false,
  },
  installing: {
    type: Boolean,
    default: false,
  },
})

const emit = defineEmits(['install'])

const categories = computed(() => (props.mod.categories ?? []).map((c) => c.name))
const authorName = computed(() => (props.mod.authors ?? []).map((a) => a.name).join(', ') || props.mod.author || '')

const handleOpen = () => {
  if (props.installing) return
  openUrl(projectUrl(props.mod, null, props.modpack))
}

const handleInstall = async () => {
  if (props.installing) return
  if (!props.instance) {
    handleOpen()
    return
  }
  emit('install')
}
</script>

<style scoped>
.cf-search-card {
  contain: layout style;
  content-visibility: auto;
  contain-intrinsic-size: auto 120px;
}
</style>
