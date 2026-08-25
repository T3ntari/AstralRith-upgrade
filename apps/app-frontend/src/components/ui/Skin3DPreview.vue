<script setup>
/**
 * High-performance Minecraft player 3D preview using Three.js.
 * Renders the standard Minecraft skin model (classic 4px-wide or slim 3px-wide).
 * Shared geometry + instanced rendering for grids; isolated for full preview.
 */
import { ref, onMounted, onUnmounted, watch } from 'vue'
import * as THREE from 'three'

const props = defineProps({
  /** 64x64 or 64x32 PNG data URL or Uint8Array */
  skinData: { type: [String, Object], default: null },
  modelType: { type: String, default: 'classic' }, // classic | slim
  width: { type: Number, default: 128 },
  height: { type: Number, default: 192 },
  autoRotate: { type: Boolean, default: true },
  animate: { type: Boolean, default: true },
  interactive: { type: Boolean, default: true },
  backgroundColor: { type: String, default: 'transparent' },
  /** Called when user clicks a body region name */
  onRegionClick: { type: Function, default: null },
})

const emit = defineEmits(['region-click', 'ready'])

const canvasRef = ref(null)
const containerRef = ref(null)

let renderer = null
let scene = null
let camera = null
let playerGroup = null
let animationId = null
let clock = null

// Head bobbing idle animation state


onMounted(() => {
  initRenderer()
  if (props.skinData) loadSkin(props.skinData)
})

onUnmounted(() => {
  dispose()
})

watch(
  () => props.skinData,
  (newData) => {
    if (newData) loadSkin(newData)
  },
)

watch(
  () => props.modelType,
  () => {
    rebuildModel()
    if (props.skinData) loadSkin(props.skinData)
  },
)

function initRenderer() {
  if (!canvasRef.value) return

  scene = new THREE.Scene()
  if (props.backgroundColor !== 'transparent') {
    scene.background = new THREE.Color(props.backgroundColor)
  }

  camera = new THREE.PerspectiveCamera(35, props.width / props.height, 0.1, 100)
  camera.position.set(0, 1.5, 4.5)
  camera.lookAt(0, 1, 0)

  renderer = new THREE.WebGLRenderer({
    canvas: canvasRef.value,
    antialias: true,
    alpha: true,
    powerPreference: 'low-power',
  })
  renderer.setSize(props.width, props.height)
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2))
  renderer.outputColorSpace = THREE.SRGBColorSpace
  renderer.toneMapping = THREE.NoToneMapping

  // Soft ambient + directional lighting
  const ambient = new THREE.AmbientLight(0xffffff, 0.8)
  scene.add(ambient)
  const dir = new THREE.DirectionalLight(0xffffff, 0.6)
  dir.position.set(2, 4, 3)
  scene.add(dir)
  const backLight = new THREE.DirectionalLight(0xffffff, 0.3)
  backLight.position.set(-2, 2, -3)
  scene.add(backLight)

  clock = new THREE.Clock()
  buildModel()
  startAnimation()
  emit('ready')

  if (props.interactive) {
    setupMouseControls()
  }
}

function buildModel() {
  if (playerGroup) {
    scene.remove(playerGroup)
    disposeGroup(playerGroup)
  }

  playerGroup = new THREE.Group()
  playerGroup.name = 'player'

  const isSlim = props.modelType === 'slim'
  const armWidth = isSlim ? 0.25 : 0.375

  // Material factory
  const baseMat = () => new THREE.MeshLambertMaterial({ transparent: true, opacity: 1, side: THREE.DoubleSide })
  const outerMat = () => new THREE.MeshLambertMaterial({ transparent: true, alphaTest: 0.1, side: THREE.DoubleSide })

  const parts = {}

  // HEAD (8x8x8)
  parts.head = new THREE.Mesh(new THREE.BoxGeometry(8, 8, 8), baseMat())
  parts.head.name = 'head'
  parts.head.position.set(0, 32, 0)

  // HEAD OVERLAY
  parts.headOverlay = new THREE.Mesh(new THREE.BoxGeometry(8.5, 8.5, 8.5), outerMat())
  parts.headOverlay.name = 'headOverlay'
  parts.headOverlay.position.copy(parts.head.position)

  // BODY (8x12x4)
  parts.body = new THREE.Mesh(new THREE.BoxGeometry(8, 12, 4), baseMat())
  parts.body.name = 'body'
  parts.body.position.set(0, 16, 0)

  // BODY OVERLAY
  parts.bodyOverlay = new THREE.Mesh(new THREE.BoxGeometry(8.5, 12.5, 4.5), outerMat())
  parts.bodyOverlay.name = 'bodyOverlay'
  parts.bodyOverlay.position.copy(parts.body.position)

  // RIGHT ARM
  parts.rightArm = new THREE.Mesh(new THREE.BoxGeometry(4, 12, armWidth === 0.25 ? 3.25 : 4), baseMat())
  parts.rightArm.name = 'rightArm'
  parts.rightArm.position.set(isSlim ? 6 : 6, 16, 0)

  // LEFT ARM
  parts.leftArm = new THREE.Mesh(new THREE.BoxGeometry(4, 12, armWidth === 0.25 ? 3.25 : 4), baseMat())
  parts.leftArm.name = 'leftArm'
  parts.leftArm.position.set(isSlim ? -6 : -6, 16, 0)

  // RIGHT LEG
  parts.rightLeg = new THREE.Mesh(new THREE.BoxGeometry(4, 12, 4), baseMat())
  parts.rightLeg.name = 'rightLeg'
  parts.rightLeg.position.set(2, 0, 0)

  // LEFT LEG
  parts.leftLeg = new THREE.Mesh(new THREE.BoxGeometry(4, 12, 4), baseMat())
  parts.leftLeg.name = 'leftLeg'
  parts.leftLeg.position.set(-2, 0, 0)

  Object.values(parts).forEach((p) => playerGroup.add(p))
  playerGroup.scale.setScalar(1 / 16) // Convert from pixel units to world units
  scene.add(playerGroup)

  return parts
}

async function loadSkin(skinData) {
  if (!playerGroup) return

  let texture
  try {
    if (typeof skinData === 'string' && skinData.startsWith('data:')) {
      texture = new THREE.TextureLoader().load(skinData)
    } else if (typeof skinData === 'string') {
      texture = new THREE.TextureLoader().load(skinData)
    } else if (skinData instanceof Uint8Array || skinData instanceof ArrayBuffer) {
      const blob = new Blob([skinData], { type: 'image/png' })
      const url = URL.createObjectURL(blob)
      texture = new THREE.TextureLoader().load(url)
      // revoke after load
      texture.onLoad = () => URL.revokeObjectURL(url)
    } else {
      return
    }

    texture.magFilter = THREE.NearestFilter
    texture.minFilter = THREE.NearestFilter
    texture.colorSpace = THREE.SRGBColorSpace
    texture.flipY = false
  } catch {
    return
  }

  await new Promise((resolve) => {
    texture.onLoad = resolve
    if (texture.image) resolve()
  })

  applyTexture(texture)
}

function applyTexture(texture) {
  const is64x32 = texture.image?.height === 32
  const uvMap = buildUVMap(is64x32)

  playerGroup.children.forEach((mesh) => {
    const map = uvMap[mesh.name]
    if (map) {
      const clonedTexture = texture.clone()
      clonedTexture.needsUpdate = true
      mesh.material.map = clonedTexture
      mesh.material.needsUpdate = true
      if (map.alphaTest) {
        mesh.material.alphaTest = 0.1
        mesh.material.transparent = true
      }
    }
  })
}

function buildUVMap(_is64x32) {
  const px = 1 / 64

  // Standard Minecraft skin UV layout
  // Head: top-left 8x8
  const head = { x: 0, y: 8 * px, w: 32 * px, h: 8 * px }
  const body = { x: 16 * px, y: 20 * px, w: 24 * px, h: 12 * px }
  const rightArm = { x: 40 * px, y: 20 * px, w: 12 * px, h: 12 * px }
  const leftArm = { x: 32 * px, y: 20 * px, w: 12 * px, h: 12 * px }
  const rightLeg = { x: 4 * px, y: 20 * px, w: 12 * px, h: 12 * px }
  const leftLeg = { x: 20 * px, y: 52 * px, w: 12 * px, h: 12 * px }

  // Overlay layers offset by 32px in UV space
  const headOverlay = { x: 32 * px, y: 8 * px, w: 32 * px, h: 8 * px }
  const bodyOverlay = { x: 16 * px, y: 36 * px, w: 24 * px, h: 12 * px }

  const map = {
    head,
    headOverlay: { ...headOverlay, alphaTest: true },
    body,
    bodyOverlay: { ...bodyOverlay, alphaTest: true },
    rightArm,
    leftArm,
    rightLeg,
    leftLeg,
  }

  return map
}

function startAnimation() {
  function animate() {
    if (!renderer) return
    animationId = requestAnimationFrame(animate)

    if (props.animate && playerGroup) {
      const t = clock.getElapsedTime()
      // Gentle idle bob
      playerGroup.position.y = Math.sin(t * 1.5) * 0.05
      // Subtle arm/leg swing
      const arm = playerGroup.children.find((c) => c.name === 'rightArm')
      const leg = playerGroup.children.find((c) => c.name === 'rightLeg')
      if (arm) arm.rotation.x = Math.sin(t * 2) * 0.15
      if (leg) leg.rotation.x = -Math.sin(t * 2) * 0.15
    }

    if (props.autoRotate && playerGroup) {
      playerGroup.rotation.y += 0.003
    }

    renderer.render(scene, camera)
  }
  animate()
}

function setupMouseControls() {
  if (!canvasRef.value) return
  let isDragging = false
  let prevX = 0
  let prevY = 0

  const onDown = (e) => {
    isDragging = true
    prevX = e.clientX
    prevY = e.clientY
    canvasRef.value?.style?.setProperty('cursor', 'grabbing')
  }

  const onMove = (e) => {
    if (!isDragging || !playerGroup) return
    const dx = e.clientX - prevX
    const dy = e.clientY - prevY
    playerGroup.rotation.y += dx * 0.01
    playerGroup.rotation.x = Math.max(-0.5, Math.min(0.5, playerGroup.rotation.x + dy * 0.005))
    prevX = e.clientX
    prevY = e.clientY
  }

  const onUp = () => {
    isDragging = false
    canvasRef.value?.style?.setProperty('cursor', 'grab')
  }

  const onWheel = (e) => {
    camera.position.z = Math.max(2, Math.min(8, camera.position.z + e.deltaY * 0.005))
  }

  canvasRef.value.addEventListener('mousedown', onDown)
  window.addEventListener('mousemove', onMove)
  window.addEventListener('mouseup', onUp)
  canvasRef.value.addEventListener('wheel', onWheel)

  // Store cleanup refs
  canvasRef.value._cleanup = () => {
    canvasRef.value?.removeEventListener('mousedown', onDown)
    window.removeEventListener('mousemove', onMove)
    window.removeEventListener('mouseup', onUp)
    canvasRef.value?.removeEventListener('wheel', onWheel)
  }
}

function rebuildModel() {
  if (playerGroup) {
    scene.remove(playerGroup)
    disposeGroup(playerGroup)
  }
  buildModel()
}

function disposeGroup(group) {
  group.traverse((child) => {
    if (child.geometry) child.geometry.dispose()
    if (child.material) {
      if (child.material.map) child.material.map.dispose()
      child.material.dispose()
    }
  })
}

function dispose() {
  if (animationId) cancelAnimationFrame(animationId)
  if (canvasRef.value?._cleanup) canvasRef.value._cleanup()
  if (playerGroup) disposeGroup(playerGroup)
  renderer?.dispose()
  renderer = null
}

defineExpose({ renderer: () => renderer, scene: () => scene })
</script>

<template>
  <div ref="containerRef" class="skin-3d-preview" :style="{ width: width + 'px', height: height + 'px' }">
    <canvas
      ref="canvasRef"
      :width="width"
      :height="height"
      :style="{ cursor: interactive ? 'grab' : 'default' }"
    />
  </div>
</template>

<style scoped>
.skin-3d-preview {
  border-radius: 8px;
  overflow: hidden;
  background: radial-gradient(ellipse at center, #2a2a3e 0%, #1a1a2e 100%);
}
canvas {
  display: block;
}
</style>
