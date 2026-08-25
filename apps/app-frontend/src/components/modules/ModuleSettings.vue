<script setup>
import { computed } from 'vue'
import { SETTING_TYPES } from '@/store/modules.js'
import { Button, Checkbox, Slider, DropdownSelect } from '@modrinth/ui'

const props = defineProps({
  settings: { type: Array, required: true },
  moduleId: { type: String, required: true },
})

const emit = defineEmits(['update-setting'])

function updateValue(settingId, value) {
  emit('update-setting', { moduleId: props.moduleId, settingId, value })
}
</script>

<template>
  <div class="space-y-3">
    <div v-for="setting in settings" :key="setting.id" class="flex flex-col gap-1">
      <!-- Boolean / Toggle -->
      <template v-if="setting.type === SETTING_TYPES.BOOLEAN">
        <div class="flex items-center justify-between">
          <label class="text-sm text-secondary">{{ setting.name }}</label>
          <Checkbox
            :model-value="setting.value"
            @update:model-value="(val) => updateValue(setting.id, val)"
          />
        </div>
      </template>

      <!-- Integer -->
      <template v-else-if="setting.type === SETTING_TYPES.INTEGER">
        <label class="text-sm text-secondary">{{ setting.name }}</label>
        <input
          :value="setting.value"
          type="number"
          :min="setting.min"
          :max="setting.max"
          class="w-full bg-black/30 border border-secondary rounded-lg px-3 py-1.5 text-sm text-contrast focus:outline-none focus:border-brand/50 transition-colors"
          @change="(e) => updateValue(setting.id, parseInt(e.target.value) || 0)"
        />
      </template>

      <!-- Float / Slider -->
      <template v-else-if="setting.type === SETTING_TYPES.FLOAT || setting.type === SETTING_TYPES.SLIDER">
        <div class="flex items-center justify-between mb-1">
          <label class="text-sm text-secondary">{{ setting.name }}</label>
          <span class="text-sm text-contrast font-mono">{{ setting.value }}</span>
        </div>
        <Slider
          :model-value="setting.value"
          :min="setting.min || 0"
          :max="setting.max || 100"
          :step="setting.step || 0.1"
          @update:model-value="(val) => updateValue(setting.id, val)"
        />
      </template>

      <!-- Enum / Dropdown -->
      <template v-else-if="setting.type === SETTING_TYPES.ENUM">
        <label class="text-sm text-secondary">{{ setting.name }}</label>
        <DropdownSelect
          :model-value="setting.value"
          :options="setting.options || []"
          :name="`${moduleId}-${setting.id}`"
          @update:model-value="(val) => updateValue(setting.id, val)"
        />
      </template>

      <!-- Color -->
      <template v-else-if="setting.type === SETTING_TYPES.COLOR">
        <div class="flex items-center justify-between">
          <label class="text-sm text-secondary">{{ setting.name }}</label>
          <div class="flex items-center gap-2">
            <input
              type="color"
              :value="setting.value"
              class="w-8 h-8 rounded border border-secondary cursor-pointer bg-transparent"
              @input="(e) => updateValue(setting.id, e.target.value)"
            />
            <span class="text-xs text-secondary font-mono">{{ setting.value }}</span>
          </div>
        </div>
      </template>

      <!-- String -->
      <template v-else-if="setting.type === SETTING_TYPES.STRING">
        <label class="text-sm text-secondary">{{ setting.name }}</label>
        <input
          :value="setting.value"
          type="text"
          class="w-full bg-black/30 border border-secondary rounded-lg px-3 py-1.5 text-sm text-contrast focus:outline-none focus:border-brand/50 transition-colors"
          @input="(e) => updateValue(setting.id, e.target.value)"
        />
      </template>

      <!-- Keybind -->
      <template v-else-if="setting.type === SETTING_TYPES.KEYBIND">
        <div class="flex items-center justify-between">
          <label class="text-sm text-secondary">{{ setting.name }}</label>
          <input
            :value="setting.value"
            type="text"
            class="w-32 bg-black/30 border border-secondary rounded-lg px-3 py-1.5 text-sm text-contrast text-center font-mono focus:outline-none focus:border-brand/50 transition-colors"
            readonly
            placeholder="Click to set"
          />
        </div>
      </template>

      <!-- Multi Select -->
      <template v-else-if="setting.type === SETTING_TYPES.MULTI_SELECT">
        <label class="text-sm text-secondary">{{ setting.name }}</label>
        <div class="flex flex-wrap gap-2">
          <label
            v-for="option in setting.options || []"
            :key="option"
            class="flex items-center gap-1.5 px-2 py-1 rounded-lg border cursor-pointer transition-colors text-sm"
            :class="(setting.value || []).includes(option) ? 'border-brand/30 bg-brand/10 text-brand' : 'border-secondary bg-bg-base text-secondary hover:text-contrast'"
          >
            <input
              type="checkbox"
              class="sr-only"
              :checked="(setting.value || []).includes(option)"
              @change="() => {
                const current = setting.value || []
                const newVal = current.includes(option)
                  ? current.filter(v => v !== option)
                  : [...current, option]
                updateValue(setting.id, newVal)
              }"
            />
            {{ option }}
          </label>
        </div>
      </template>

      <!-- Action -->
      <template v-else-if="setting.type === SETTING_TYPES.ACTION">
        <Button :outline="true" :action="() => updateValue(setting.id, null)">
          {{ setting.name }}
        </Button>
      </template>
    </div>
  </div>
</template>
