<script setup>
import { ref, reactive } from 'vue'
import {
  EyeIcon,
  EyeOffIcon,
  TransferIcon,
  PaintBrushIcon,
  TextQuoteIcon,
} from '@modrinth/assets'
import { Button, Checkbox, Slider } from '@modrinth/ui'

// HUD element definitions
const hudElements = reactive([
  {
    id: 'fps',
    name: 'FPS Counter',
    visible: true,
    position: 'top-left',
    fontSize: 12,
    color: '#ffffff',
    opacity: 1.0,
    anchor: { x: 10, y: 10 },
  },
  {
    id: 'coordinates',
    name: 'Coordinates',
    visible: true,
    position: 'top-left',
    fontSize: 12,
    color: '#ffffff',
    opacity: 1.0,
    anchor: { x: 10, y: 30 },
  },
  {
    id: 'direction',
    name: 'Direction',
    visible: true,
    position: 'top-left',
    fontSize: 12,
    color: '#ffffff',
    opacity: 1.0,
    anchor: { x: 10, y: 50 },
  },
  {
    id: 'server',
    name: 'Server Info',
    visible: false,
    position: 'top-right',
    fontSize: 12,
    color: '#ffffff',
    opacity: 1.0,
    anchor: { x: 10, y: 10 },
  },
  {
    id: 'ping',
    name: 'Ping',
    visible: true,
    position: 'top-right',
    fontSize: 12,
    color: '#00ff00',
    opacity: 1.0,
    anchor: { x: 10, y: 30 },
  },
  {
    id: 'player_count',
    name: 'Player Count',
    visible: false,
    position: 'top-right',
    fontSize: 12,
    color: '#ffffff',
    opacity: 1.0,
    anchor: { x: 10, y: 50 },
  },
  {
    id: 'armor',
    name: 'Armor Status',
    visible: true,
    position: 'bottom-left',
    fontSize: 12,
    color: '#ffffff',
    opacity: 1.0,
    anchor: { x: 10, y: 10 },
  },
  {
    id: 'active_modules',
    name: 'Active Modules',
    visible: true,
    position: 'bottom-right',
    fontSize: 12,
    color: '#ffffff',
    opacity: 0.8,
    anchor: { x: 10, y: 10 },
  },
  {
    id: 'watermark',
    name: 'Watermark',
    visible: true,
    position: 'top-left',
    fontSize: 16,
    color: '#6366f1',
    opacity: 1.0,
    anchor: { x: 10, y: 0 },
  },
  {
    id: 'clock',
    name: 'Clock',
    visible: false,
    position: 'top-right',
    fontSize: 12,
    color: '#ffffff',
    opacity: 1.0,
    anchor: { x: 10, y: 0 },
  },
])

const positions = ['top-left', 'top-right', 'bottom-left', 'bottom-right', 'center']

const selectedElement = ref(null)

function selectElement(element) {
  selectedElement.value = selectedElement.value?.id === element.id ? null : element
}

function toggleVisibility(element) {
  element.visible = !element.visible
}
</script>

<template>
  <div class="flex gap-6 h-full">
    <!-- HUD elements list -->
    <div class="w-80 flex-shrink-0 space-y-1 overflow-y-auto">
      <div class="text-xs text-secondary uppercase tracking-wider mb-2">HUD Elements</div>

      <div
        v-for="element in hudElements"
        :key="element.id"
        class="rounded-lg border px-3 py-2 cursor-pointer transition-all duration-150"
        :class="[
          selectedElement?.id === element.id
            ? 'border-brand/30 bg-brand/5'
            : 'border-secondary bg-bg-raised hover:bg-bg-raised/80'
        ]"
        @click="selectElement(element)"
      >
        <div class="flex items-center gap-2">
          <button
            class="p-0.5 rounded transition-colors"
            :class="element.visible ? 'text-green-400 hover:text-green-300' : 'text-secondary/40 hover:text-secondary'"
            @click.stop="toggleVisibility(element)"
            :title="element.visible ? 'Hide element' : 'Show element'"
          >
            <EyeIcon v-if="element.visible" class="w-4 h-4" />
            <EyeOffIcon v-else class="w-4 h-4" />
          </button>
          <span class="text-sm font-medium text-contrast flex-1">{{ element.name }}</span>
          <div
            class="w-3 h-3 rounded-full border border-secondary/50"
            :style="{ backgroundColor: element.color, opacity: element.opacity }"
          />
          <span class="text-xs text-secondary">{{ element.position }}</span>
        </div>
      </div>
    </div>

    <!-- Preview area -->
    <div class="flex-1 flex flex-col">
      <!-- Preview -->
      <div class="flex-1 rounded-xl border border-secondary bg-black/40 relative overflow-hidden mb-4" style="min-height: 300px;">
        <!-- Fake game background -->
        <div class="absolute inset-0 bg-gradient-to-br from-gray-800 to-gray-900 opacity-50" />
        <div class="absolute inset-0 flex items-center justify-center text-secondary/20 text-lg font-mono">
          Game Preview
        </div>

        <!-- HUD elements overlay -->
        <div
          v-for="element in hudElements"
          :key="element.id"
          v-show="element.visible"
          class="absolute text-xs font-mono pointer-events-none transition-all duration-200"
          :class="[
            `hud-${element.position}`,
            selectedElement?.id === element.id ? 'ring-2 ring-brand/50 ring-offset-1 ring-offset-transparent rounded' : ''
          ]"
          :style="{
            color: element.color,
            fontSize: element.fontSize + 'px',
            opacity: element.opacity,
          }"
        >
          <template v-if="element.id === 'fps'">FPS: 144</template>
          <template v-else-if="element.id === 'coordinates'">XYZ: 142.5 / 72.0 / -33.2</template>
          <template v-else-if="element.id === 'direction'">Facing: North (0.0°)</template>
          <template v-else-if="element.id === 'server'">mc.hypixel.net</template>
          <template v-else-if="element.id === 'ping'">Ping: 42ms</template>
          <template v-else-if="element.id === 'player_count'">Players: 128/500</template>
          <template v-else-if="element.id === 'armor'">Armor: ♦♦♦♦</template>
          <template v-else-if="element.id === 'active_modules'">
            <div class="space-y-0.5 text-right">
              <div>KillAura</div>
              <div>Fullbright</div>
              <div>Sprint</div>
            </div>
          </template>
          <template v-else-if="element.id === 'watermark'">AstralRinth</template>
          <template v-else-if="element.id === 'clock'">{{ new Date().toLocaleTimeString() }}</template>
        </div>
      </div>

      <!-- Element settings -->
      <div
        v-if="selectedElement"
        class="rounded-xl border border-secondary bg-bg-raised px-4 py-3"
      >
        <div class="flex items-center gap-2 mb-3">
          <span class="font-semibold text-contrast">{{ selectedElement.name }}</span>
          <span class="text-xs text-secondary">settings</span>
        </div>

        <div class="grid grid-cols-2 gap-4">
          <!-- Position -->
          <div>
            <label class="text-xs text-secondary block mb-1">Position</label>
            <div class="flex flex-wrap gap-1.5">
              <button
                v-for="pos in positions"
                :key="pos"
                class="px-2 py-1 text-xs rounded border transition-colors"
                :class="selectedElement.position === pos ? 'border-brand bg-brand/10 text-brand' : 'border-secondary text-secondary hover:text-contrast'"
                @click="selectedElement.position = pos"
              >
                {{ pos }}
              </button>
            </div>
          </div>

          <!-- Font size -->
          <div>
            <div class="flex items-center justify-between mb-1">
              <label class="text-xs text-secondary">Font Size</label>
              <span class="text-xs text-contrast font-mono">{{ selectedElement.fontSize }}px</span>
            </div>
            <Slider
              :model-value="selectedElement.fontSize"
              :min="8"
              :max="32"
              :step="1"
              @update:model-value="(val) => selectedElement.fontSize = val"
            />
          </div>

          <!-- Color -->
          <div>
            <label class="text-xs text-secondary block mb-1">Color</label>
            <div class="flex items-center gap-2">
              <input
                type="color"
                :value="selectedElement.color"
                class="w-8 h-8 rounded border border-secondary cursor-pointer bg-transparent"
                @input="(e) => selectedElement.color = e.target.value"
              />
              <span class="text-xs text-secondary font-mono">{{ selectedElement.color }}</span>
            </div>
          </div>

          <!-- Opacity -->
          <div>
            <div class="flex items-center justify-between mb-1">
              <label class="text-xs text-secondary">Opacity</label>
              <span class="text-xs text-contrast font-mono">{{ Math.round(selectedElement.opacity * 100) }}%</span>
            </div>
            <Slider
              :model-value="selectedElement.opacity"
              :min="0"
              :max="1"
              :step="0.05"
              @update:model-value="(val) => selectedElement.opacity = val"
            />
          </div>
        </div>
      </div>

      <div v-else class="rounded-xl border border-secondary/50 bg-bg-raised/30 px-4 py-6 text-center text-secondary text-sm">
        Select a HUD element to edit its settings
      </div>
    </div>
  </div>
</template>

<style scoped>
.hud-top-left {
  top: 10px;
  left: 10px;
}
.hud-top-right {
  top: 10px;
  right: 10px;
  text-align: right;
}
.hud-bottom-left {
  bottom: 10px;
  left: 10px;
}
.hud-bottom-right {
  bottom: 10px;
  right: 10px;
  text-align: right;
}
.hud-center {
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
}
</style>
