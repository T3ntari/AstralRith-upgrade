<template>
  <div class="content-source-toggle" role="switch" :aria-checked="modelValue" @click="toggle">
    <div class="thumb" :class="{ 'thumb--cf': modelValue }">
      <svg
        v-if="!modelValue"
        class="logo logo--mr"
        viewBox="0 0 590 591"
        xmlns="http://www.w3.org/2000/svg"
        xml:space="preserve"
      >
        <path
          d="M300.366,311.86L283.216,266.381L336.966,211.169L404.9,196.531L424.57,220.74L393.254,252.46L365.941,261.052L346.425,281.11L355.987,307.719L375.387,328.306L402.745,321.031L422.216,299.648L464.729,286.185L477.395,314.677L433.529,368.46L360.02,391.735L327.058,355.031L138.217,468.344C129.245,456.811 118.829,440.485 112.15,424.792L300.366,311.86Z"
        />
        <path
          d="M655.189,194.555L505.695,234.873C513.927,256.795 516.638,269.674 518.915,283.863L668.152,243.609C665.764,227.675 661.5,211.444 655.189,194.555Z"
        />
      </svg>
      <div v-else class="cf-badge">
        <span>CF</span>
      </div>
    </div>
    <div class="side side--mr" :class="{ 'side--active': !modelValue }">
      <span class="side-label">Modrinth</span>
    </div>
    <div class="side side--cf" :class="{ 'side--active': modelValue }">
      <span class="side-label">CurseForge</span>
    </div>
  </div>
</template>

<script setup>
const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false,
  },
})

const emit = defineEmits(['update:model-value'])

const toggle = () => {
  emit('update:model-value', !props.modelValue)
}
</script>

<style scoped lang="scss">
.content-source-toggle {
  position: relative;
  display: inline-flex;
  align-items: stretch;
  width: 16rem;
  height: 2.75rem;
  border-radius: 9999px;
  background-color: var(--color-button-bg);
  border: 1px solid var(--color-button-border);
  cursor: pointer;
  user-select: none;
  overflow: hidden;
}

.thumb {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 50%;
  height: calc(100% - 6px);
  border-radius: 9999px;
  background-color: var(--color-raised-bg);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.35);
  display: flex;
  align-items: center;
  justify-content: center;
  transition:
    transform 0.25s ease,
    background-color 0.25s ease;

  &--cf {
    transform: translateX(100%);
  }
}

.logo {
  width: 1.6rem;
  height: 1.6rem;

  path {
    fill: var(--color-green);
  }
}

.cf-badge {
  width: 1.6rem;
  height: 1.6rem;
  border-radius: 0.45rem;
  background: linear-gradient(135deg, #f16436 0%, #ff9a5c 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 900;
  font-size: 0.7rem;
  color: #fff;
  letter-spacing: -0.02em;
}

.side {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.25s ease;

  &--active {
    background-color: transparent;
  }

  &.side--active {
    .side-label {
      color: var(--color-contrast);
    }
  }
}

.side-label {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--color-secondary);
  transition: color 0.25s ease;
}
</style>
