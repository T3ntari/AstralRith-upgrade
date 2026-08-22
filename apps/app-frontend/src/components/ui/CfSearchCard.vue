<template>
  <div
    class="card-shadow p-4 bg-bg-raised rounded-xl flex gap-3 group cursor-pointer hover:brightness-90 transition-all"
    @click="handleOpen"
  >
    <div class="icon w-[96px] h-[96px] relative">
      <Avatar :src="mod.logo?.url" size="96px" class="search-icon origin-top transition-all" />
    </div>
    <div class="flex flex-col gap-2 overflow-hidden">
      <div class="gap-2 overflow-hidden no-wrap text-ellipsis">
        <span class="text-lg font-extrabold text-contrast m-0 leading-none">
          {{ mod.name }}
        </span>
        <span v-if="mod.authors?.length" class="text-secondary">
          by {{ mod.authors.map((a) => a.name).join(', ') }}
        </span>
      </div>
      <div class="m-0 line-clamp-2">
        {{ mod.summary }}
      </div>
      <div v-if="mod.categories?.length" class="mt-auto flex items-center gap-1 no-wrap">
        <TagsIcon class="h-4 w-4 shrink-0" />
        <div
          v-for="cat in mod.categories.slice(0, 4)"
          :key="cat.id"
          class="text-sm font-semibold text-secondary flex gap-1 px-[0.375rem] py-0.5 bg-button-bg rounded-full"
        >
          {{ cat.name }}
        </div>
      </div>
    </div>
    <div class="flex flex-col gap-2 items-end shrink-0 ml-auto">
      <div class="flex items-center gap-2">
        <DownloadIcon class="shrink-0" />
        <span>
          {{ formatDownloadCount(mod.downloadCount) }}
          <span class="text-secondary">downloads</span>
        </span>
      </div>
      <div class="flex items-center gap-2">
        <HeartIcon class="shrink-0" />
        <span>
          {{ formatDownloadCount(mod.thumbnails?.length ?? mod.downloadCount) }}
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

const handleOpen = () => {
  if (props.installing) return
  openUrl(projectUrl(props.mod))
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
