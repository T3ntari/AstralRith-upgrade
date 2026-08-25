<template>
  <canvas ref="canvasRef" :width="width" :height="height" class="skin-canvas" :style="{ width: displayWidth + 'px', height: displayHeight + 'px', imageRendering: 'pixelated' }" />
</template>

<script setup>
import { ref, onMounted, watch } from 'vue'

const props = defineProps({
  imageUrl: { type: String, default: '' },
  skinBytes: { type: Uint8Array, default: null },
  scale: { type: Number, default: 8 },
  view: { type: String, default: 'front' }, // front | back | both
  displayWidth: { type: Number, default: 96 },
  displayHeight: { type: Number, default: 128 },
})

const canvasRef = ref(null)

// Draw a player figure from a 64x64 (or 64x32) skin texture
function drawSkin(canvas, image, view) {
  const ctx = canvas.getContext('2d')
  ctx.clearRect(0, 0, canvas.width, canvas.height)
  const s = props.scale
  const S = 8 * s // head size in canvas px
  const T = 4 * s // limb thickness

  // Head at top center
  const headX = (canvas.width - S) / 2
  const headY = 0

  // Draw head front
  if (view === 'front' || view === 'both') {
    ctx.drawImage(image, 8, 8, 8, 8, headX, headY, S, S)
    // Head overlay (helmet layer)
    ctx.drawImage(image, 40, 8, 8, 8, headX, headY, S, S)
  } else if (view === 'back') {
    ctx.drawImage(image, 24, 8, 8, 8, headX, headY, S, S)
    ctx.drawImage(image, 56, 8, 8, 8, headX, headY, S, S)
  }

  // Body
  const bodyX = (canvas.width - T * 2) / 2
  const bodyY = headY + S
  if (view === 'front' || view === 'both') {
    ctx.drawImage(image, 20, 20, 8, 12, bodyX, bodyY, T * 2, 12 * s)
    ctx.drawImage(image, 20, 36, 8, 12, bodyX, bodyY, T * 2, 12 * s)
  } else {
    ctx.drawImage(image, 32, 20, 8, 12, bodyX, bodyY, T * 2, 12 * s)
    ctx.drawImage(image, 32, 36, 8, 12, bodyX, bodyY, T * 2, 12 * s)
  }

  // Legs
  const legY = bodyY + 12 * s
  const legW = T
  const legH = 12 * s
  if (view === 'front' || view === 'both') {
    ctx.drawImage(image, 4, 20, 4, 12, bodyX, legY, legW, legH)
    ctx.drawImage(image, 4, 36, 4, 12, bodyX, legY, legW, legH)
    ctx.drawImage(image, 12, 20, 4, 12, bodyX + T, legY, legW, legH)
    ctx.drawImage(image, 12, 36, 4, 12, bodyX + T, legY, legW, legH)
  } else {
    ctx.drawImage(image, 8, 20, 4, 12, bodyX, legY, legW, legH)
    ctx.drawImage(image, 8, 36, 4, 12, bodyX, legY, legW, legH)
    ctx.drawImage(image, 16, 20, 4, 12, bodyX + T, legY, legW, legH)
    ctx.drawImage(image, 16, 36, 4, 12, bodyX + T, legY, legW, legH)
  }

  // Arms
  const armY = bodyY
  const armW = T
  const armH = 12 * s
  if (view === 'front' || view === 'both') {
    // Left arm
    ctx.drawImage(image, 44, 20, 4, 12, bodyX - T, armY, armW, armH)
    ctx.drawImage(image, 44, 36, 4, 12, bodyX - T, armY, armW, armH)
    // Right arm
    ctx.drawImage(image, 52, 20, 4, 12, bodyX + T * 2, armY, armW, armH)
    ctx.drawImage(image, 52, 36, 4, 12, bodyX + T * 2, armY, armW, armH)
  } else {
    // Left arm back
    ctx.drawImage(image, 48, 20, 4, 12, bodyX - T, armY, armW, armH)
    ctx.drawImage(image, 48, 36, 4, 12, bodyX - T, armY, armW, armH)
    // Right arm back
    ctx.drawImage(image, 56, 20, 4, 12, bodyX + T * 2, armY, armW, armH)
    ctx.drawImage(image, 56, 36, 4, 12, bodyX + T * 2, armY, armW, armH)
  }
}

async function render() {
  const canvas = canvasRef.value
  if (!canvas) return
  const ctx = canvas.getContext('2d')

  if (props.view === 'both') {
    canvas.width = props.displayWidth * 2
    canvas.height = props.displayHeight
    // Draw front on left half, back on right half
  } else {
    canvas.width = props.displayWidth
    canvas.height = props.displayHeight
  }

  ctx.clearRect(0, 0, canvas.width, canvas.height)

  let img = null
  if (props.skinBytes && props.skinBytes.length > 0) {
    const blob = new Blob([props.skinBytes], { type: 'image/png' })
    const url = URL.createObjectURL(blob)
    img = new Image()
    img.onload = () => {
      if (props.view === 'both') {
        // front on left
        const tempCanvas = document.createElement('canvas')
        tempCanvas.width = props.displayWidth
        tempCanvas.height = props.displayHeight
        drawSkin(tempCanvas, img, 'front')
        ctx.drawImage(tempCanvas, 0, 0)
        // back on right
        const tempCanvas2 = document.createElement('canvas')
        tempCanvas2.width = props.displayWidth
        tempCanvas2.height = props.displayHeight
        drawSkin(tempCanvas2, img, 'back')
        ctx.drawImage(tempCanvas2, props.displayWidth, 0)
      } else {
        drawSkin(canvas, img, props.view)
      }
      URL.revokeObjectURL(url)
    }
    img.onerror = () => URL.revokeObjectURL(url)
    img.src = url
  } else if (props.imageUrl) {
    img = new Image()
    img.crossOrigin = 'anonymous'
    img.onload = () => {
      if (props.view === 'both') {
        const tempCanvas = document.createElement('canvas')
        tempCanvas.width = props.displayWidth
        tempCanvas.height = props.displayHeight
        drawSkin(tempCanvas, img, 'front')
        ctx.drawImage(tempCanvas, 0, 0)
        const tempCanvas2 = document.createElement('canvas')
        tempCanvas2.width = props.displayWidth
        tempCanvas2.height = props.displayHeight
        drawSkin(tempCanvas2, img, 'back')
        ctx.drawImage(tempCanvas2, props.displayWidth, 0)
      } else {
        drawSkin(canvas, img, props.view)
      }
    }
    img.onerror = () => {
      // draw default steve silhouette
    }
    img.src = props.imageUrl
  }
}

onMounted(() => render())
watch(() => [props.imageUrl, props.skinBytes, props.view, props.scale], () => render(), { deep: false })
</script>

<style scoped>
.skin-canvas {
  image-rendering: pixelated;
  image-rendering: crisp-edges;
  border-radius: 4px;
  background: transparent;
}
</style>
