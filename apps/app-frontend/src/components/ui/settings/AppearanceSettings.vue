<script setup lang="ts">
import { Toggle, ThemeSelector, TeleportDropdownMenu, DropdownSelect } from '@modrinth/ui'
import { useTheming } from '@/store/state'
import { get, set } from '@/helpers/settings'
import { watch, ref } from 'vue'
import { getOS } from '@/helpers/utils'

const themeStore = useTheming()

const os = ref(await getOS())
const settings = ref(await get())

const gpuOptions = ref([
  { id: 'auto', name: 'Automatic (system default)' },
  { id: 'nvidia', name: 'NVIDIA (dedicated)' },
  { id: 'integrated', name: 'Integrated (iGPU)' },
])

watch(
  settings,
  async () => {
    await set(settings.value)
  },
  { deep: true },
)
</script>
<template>
  <h2 class="m-0 text-lg font-extrabold text-contrast">Color theme</h2>
  <p class="m-0 mt-1">Select your preferred color theme for Modrinth App.</p>

  <ThemeSelector
    :update-color-theme="
      (theme) => {
        themeStore.setThemeState(theme)
        settings.theme = theme
      }
    "
    :current-theme="settings.theme"
    :theme-options="themeStore.themeOptions"
    system-theme-color="system"
  />

  <div class="mt-4 flex items-center justify-between">
    <div>
      <h2 class="m-0 text-lg font-extrabold text-contrast">Advanced rendering</h2>
      <p class="m-0 mt-1">
        Enables advanced rendering such as blur effects that may cause performance issues without
        hardware-accelerated rendering.
      </p>
    </div>

    <Toggle
      id="advanced-rendering"
      :model-value="themeStore.advancedRendering"
      :checked="themeStore.advancedRendering"
      @update:model-value="
        (e) => {
          themeStore.advancedRendering = e
          settings.advanced_rendering = themeStore.advancedRendering
        }
      "
    />
  </div>

  <div class="mt-4 flex items-center justify-between">
    <div>
      <h2 class="m-0 text-lg font-extrabold text-contrast">Efficient mode</h2>
      <p class="m-0 mt-1">
        For low-end devices. Disables shadows, blurs, gradients and animations so the launcher
        stays fast and smooth on weaker hardware.
      </p>
    </div>
    <Toggle
      id="efficient-mode"
      :model-value="settings.efficient_mode"
      :checked="settings.efficient_mode"
      @update:model-value="
        (e) => {
          settings.efficient_mode = e
          themeStore.setEfficientMode(e)
        }
      "
    />
  </div>

  <div class="mt-4 flex items-center justify-between">
    <div>
      <h2 class="m-0 text-lg font-extrabold text-contrast">Launcher VSYNC</h2>
      <p class="m-0 mt-1">Enable true VSYNC for the launcher window (requires restart).</p>
    </div>
    <Toggle
      id="launcher-vsync"
      :model-value="settings.launcher_vsync"
      :checked="settings.launcher_vsync"
      @update:model-value="
        (e) => {
          settings.launcher_vsync = e
        }
      "
    />
  </div>

  <div class="mt-4 flex items-center justify-between gap-4">
    <div>
      <h2 class="m-0 text-lg font-extrabold text-contrast">GPU for Minecraft</h2>
      <p class="m-0 mt-1">
        Select which GPU to use for Minecraft instances. Useful for laptops with NVIDIA Optimus or multiple GPUs.
      </p>
    </div>
    <DropdownSelect
      v-model="settings.gpu_preference"
      :options="gpuOptions"
      :display-name="(opt) => opt.name"
      class="max-w-[20rem]"
      @update:model-value="(val) => settings.gpu_preference = val.id"
    >
      <span class="font-semibold text-secondary">{{ gpuOptions.find(o => o.id === settings.gpu_preference)?.name ?? settings.gpu_preference }}</span>
    </DropdownSelect>
  </div>

  <div v-if="os !== 'MacOS'" class="mt-4 flex items-center justify-between gap-4">
    <div>
      <h2 class="m-0 text-lg font-extrabold text-contrast">Native Decorations</h2>
      <p class="m-0 mt-1">Use system window frame (app restart required).</p>
    </div>
    <Toggle
      id="native-decorations"
      :model-value="settings.native_decorations"
      :checked="settings.native_decorations"
      @update:model-value="
        (e) => {
          settings.native_decorations = e
        }
      "
    />
  </div>

  <div class="mt-4 flex items-center justify-between">
    <div>
      <h2 class="m-0 text-lg font-extrabold text-contrast">Minimize launcher</h2>
      <p class="m-0 mt-1">Minimize the launcher when a Minecraft process starts.</p>
    </div>
    <Toggle
      id="minimize-launcher"
      :model-value="settings.hide_on_process_start"
      :checked="settings.hide_on_process_start"
      @update:model-value="
        (e) => {
          settings.hide_on_process_start = e
        }
      "
    />
  </div>

  <div class="mt-4 flex items-center justify-between">
    <div>
      <h2 class="m-0 text-lg font-extrabold text-contrast">Default landing page</h2>
      <p class="m-0 mt-1">Change the page to which the launcher opens on.</p>
    </div>
    <TeleportDropdownMenu
      id="opening-page"
      v-model="settings.default_page"
      name="Opening page dropdown"
      :options="['Home', 'Library']"
    />
  </div>

  <div class="mt-4 flex items-center justify-between">
    <div>
      <h2 class="m-0 text-lg font-extrabold text-contrast">Toggle sidebar</h2>
      <p class="m-0 mt-1">Enables the ability to toggle the sidebar.</p>
    </div>
    <Toggle
      id="toggle-sidebar"
      :model-value="settings.toggle_sidebar"
      :checked="settings.toggle_sidebar"
      @update:model-value="
        (e) => {
          settings.toggle_sidebar = e
          themeStore.toggleSidebar = settings.toggle_sidebar
        }
      "
    />
  </div>
</template>
