<template>
  <div
    ref="root"
    class="art-space"
    :class="`art-space--${variant}`"
    aria-hidden="true"
  >
    <div class="art-space__calm-current">
      <i></i>
      <i></i>
    </div>
    <div class="art-space__soft-ripples"></div>
    <canvas ref="canvas" class="art-space__canvas"></canvas>

    <div class="art-space__aurora art-space__aurora--a"></div>
    <div class="art-space__aurora art-space__aurora--b"></div>
    <div class="art-space__aurora art-space__aurora--c"></div>
    <div class="art-space__scan"></div>
    <div class="art-space__vignette"></div>

    <div class="art-space__ruler art-space__ruler--left">
      <i v-for="tick in 14" :key="`left-${tick}`"></i>
    </div>
    <div class="art-space__ruler art-space__ruler--right">
      <i v-for="tick in 14" :key="`right-${tick}`"></i>
    </div>

    <div class="art-space__orbit art-space__orbit--a"></div>
    <div class="art-space__orbit art-space__orbit--b"></div>

    <div class="art-space__telemetry art-space__telemetry--top">
      <span>{{ variant === 'gallery' ? 'GALLERY FIELD' : 'CREATION FIELD' }}</span>
      <b>LIVE / 03D</b>
    </div>
    <div class="art-space__telemetry art-space__telemetry--bottom">
      <b>DEPTH {{ depthReadout }}</b>
      <span>POINTER FORCE / SCROLL DRIVE</span>
    </div>
  </div>
</template>

<script setup>
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'

const props = defineProps({
  variant: {
    type: String,
    default: 'gallery',
    validator: value => ['gallery', 'upload'].includes(value),
  },
})

const root = ref(null)
const canvas = ref(null)
const depthReadout = ref('00.0')

const pointer = {
  x: 0,
  y: 0,
  targetX: 0,
  targetY: 0,
  normalizedX: 0,
  normalizedY: 0,
  velocityX: 0,
  velocityY: 0,
  lastX: 0,
  lastY: 0,
  active: false,
  sampled: false,
}

const camera = { x: 0, y: 0, roll: 0 }
const shock = { x: 0, y: 0, power: 0, age: 0 }
const scene = { particles: [], frames: [], solids: [], streams: [], glyphs: [] }

let context = null
let animationFrame = 0
let resizeObserver = null
let mediaQuery = null
let width = 0
let height = 0
let dpr = 1
let lastTime = 0
let travel = 0
let scrollVelocity = 0
let lastScrollY = 0
let scrollProgress = 0
let hidden = false
let reducedMotion = false
let seed = 1
let readoutFrame = 0
let styleWriteFrame = 0
let formationIndex = 0
let formationCooldownUntil = 0
let ambientYaw = 0
let scrollYaw = 0
const rootStyleCache = new Map()
const documentStyleCache = new Map()

const fieldWake = {
  x: 0,
  y: 0,
  rotation: 0,
  velocityX: 0,
  velocityY: 0,
  velocityRotation: 0,
}

const view = {
  yaw: 0,
  targetYaw: 0,
  velocity: 0,
}

const projectionCache = {
  fov: 1,
  horizon: 0,
  cosine: 1,
  sine: 0,
  maxDimension: 1,
}

const NEAR = 2.4
const FAR = 44
const RING_CENTER_Z = 23
let sceneCenterZ = RING_CENTER_Z
const CUBE_CORNERS = [
  { x: -1, y: -1, z: -1 }, { x: 1, y: -1, z: -1 },
  { x: 1, y: 1, z: -1 }, { x: -1, y: 1, z: -1 },
  { x: -1, y: -1, z: 1 }, { x: 1, y: -1, z: 1 },
  { x: 1, y: 1, z: 1 }, { x: -1, y: 1, z: 1 },
]
const CUBE_EDGES = [
  [0, 1], [1, 2], [2, 3], [3, 0],
  [4, 5], [5, 6], [6, 7], [7, 4],
  [0, 4], [1, 5], [2, 6], [3, 7],
]
const FRAME_SHAPES = ['box', 'pyramid', 'prism', 'box', 'cone', 'octahedron']
const SOLID_SHAPES = ['octahedron', 'cone', 'pyramid', 'prism', 'box']

function random() {
  seed = (seed * 1664525 + 1013904223) >>> 0
  return seed / 4294967296
}

function range(min, max) {
  return min + (max - min) * random()
}

function motionState(order, kind) {
  return {
    order,
    kind,
    offsetX: 0,
    offsetY: 0,
    offsetZ: 0,
    velocityX: 0,
    velocityY: 0,
    velocityZ: 0,
    rotationX: 0,
    rotationY: 0,
    rotationZ: 0,
    angularVelocityX: 0,
    angularVelocityY: 0,
    angularVelocityZ: 0,
    targetX: 0,
    targetY: 0,
    targetZ: 0,
    targetRotationX: 0,
    targetRotationY: 0,
    targetRotationZ: 0,
    morphPhase: range(0, Math.PI * 2),
    projected: { x: 0, y: 0, z: 0, rawZ: 0, scale: 1, visible: false },
  }
}

function ringPlacement(index, count, minRadius, maxRadius) {
  const angle = index / Math.max(1, count) * Math.PI * 2 + range(-0.18, 0.18)
  const radius = range(minRadius, maxRadius)
  return {
    angle,
    radius,
    x: Math.sin(angle) * radius,
    z: sceneCenterZ + Math.cos(angle) * radius,
  }
}

function projectionFov() {
  return Math.min(width * 0.92, height * 1.2)
}

function projectionHorizon() {
  return height * (props.variant === 'upload' ? 0.42 : 0.39)
}

function refreshProjectionCache() {
  projectionCache.fov = projectionFov()
  projectionCache.horizon = projectionHorizon()
  projectionCache.cosine = Math.cos(view.yaw)
  projectionCache.sine = Math.sin(view.yaw)
  projectionCache.maxDimension = Math.max(width, height)
}

function verticalLane(index, phase = 0) {
  return (phase + index * 0.61803398875) % 1
}

function verticalWorldPosition(placement, index, phase = 0, jitter = 0.025) {
  const compact = width < 700
  const margin = compact ? 0.075 : 0.06
  const lane = verticalLane(index, phase)
  const screenY = height * (
    margin
    + lane * (1 - margin * 2)
    + range(-jitter, jitter)
  )
  const turnedDepth = Math.max(4.5, turnWorld(placement).z)
  return (screenY - projectionHorizon()) * turnedDepth / Math.max(1, projectionFov())
}

function shapeDefinition(shape) {
  if (shape === 'pyramid') {
    return {
      vertices: [
        { x: -1, y: 0.82, z: -1 }, { x: 1, y: 0.82, z: -1 },
        { x: 1, y: 0.82, z: 1 }, { x: -1, y: 0.82, z: 1 },
        { x: 0, y: -1.28, z: 0 },
      ],
      edges: [[0, 1], [1, 2], [2, 3], [3, 0], [0, 4], [1, 4], [2, 4], [3, 4]],
      accentNode: 4,
    }
  }
  if (shape === 'prism') {
    return {
      vertices: [
        { x: -1, y: 0.86, z: -1 }, { x: 1, y: 0.86, z: -1 }, { x: 0, y: -1.12, z: -1 },
        { x: -1, y: 0.86, z: 1 }, { x: 1, y: 0.86, z: 1 }, { x: 0, y: -1.12, z: 1 },
      ],
      edges: [[0, 1], [1, 2], [2, 0], [3, 4], [4, 5], [5, 3], [0, 3], [1, 4], [2, 5]],
      accentNode: 5,
    }
  }
  if (shape === 'octahedron') {
    return {
      vertices: [
        { x: 0, y: -1.25, z: 0 }, { x: 0, y: 1.25, z: 0 },
        { x: -1, y: 0, z: 0 }, { x: 0, y: 0, z: -1 },
        { x: 1, y: 0, z: 0 }, { x: 0, y: 0, z: 1 },
      ],
      edges: [
        [0, 2], [0, 3], [0, 4], [0, 5],
        [1, 2], [1, 3], [1, 4], [1, 5],
        [2, 3], [3, 4], [4, 5], [5, 2],
      ],
      accentNode: 0,
    }
  }
  if (shape === 'cone') {
    const rimCount = 8
    const vertices = [{ x: 0, y: -1.32, z: 0 }]
    for (let index = 0; index < rimCount; index += 1) {
      const angle = index / rimCount * Math.PI * 2
      vertices.push({ x: Math.cos(angle), y: 0.92, z: Math.sin(angle) })
    }
    const edges = []
    for (let index = 0; index < rimCount; index += 1) {
      edges.push([index + 1, (index + 1) % rimCount + 1])
      edges.push([0, index + 1])
    }
    return { vertices, edges, accentNode: 0 }
  }
  return {
    vertices: CUBE_CORNERS,
    edges: CUBE_EDGES,
    face: [4, 5, 6, 7],
    accentNode: 4,
  }
}

function createElasticMesh(shape = 'box', segments = 4) {
  const definition = shapeDefinition(shape)
  const nodes = definition.vertices.map(vertex => ({
    baseX: vertex.x,
    baseY: vertex.y,
    baseZ: vertex.z,
    offsetX: 0,
    offsetY: 0,
    offsetZ: 0,
    velocityX: 0,
    velocityY: 0,
    velocityZ: 0,
  }))
  const edges = definition.edges.map(([start, end]) => {
    const indices = [start]
    const a = definition.vertices[start]
    const b = definition.vertices[end]
    for (let step = 1; step < segments; step += 1) {
      const progress = step / segments
      indices.push(nodes.length)
      nodes.push({
        baseX: a.x + (b.x - a.x) * progress,
        baseY: a.y + (b.y - a.y) * progress,
        baseZ: a.z + (b.z - a.z) * progress,
        offsetX: 0,
        offsetY: 0,
        offsetZ: 0,
        velocityX: 0,
        velocityY: 0,
        velocityZ: 0,
      })
    }
    indices.push(end)
    return indices
  })
  return {
    shape,
    nodes,
    edges,
    face: definition.face || null,
    accentNode: definition.accentNode || 0,
    awake: false,
    restFrames: 0,
    projected: nodes.map(() => ({ x: 0, y: 0, z: 0, rawZ: 0, scale: 1, visible: false })),
    transform: { time: Number.NaN },
  }
}

function palette() {
  const dark = document.documentElement.classList.contains('art-lights-out')
  if (props.variant === 'upload') {
    return dark
      ? { cool: '118, 228, 221', soft: '145, 174, 211', hot: '255, 183, 103', ink: '198, 232, 235' }
      : { cool: '35, 151, 151', soft: '86, 132, 158', hot: '218, 124, 92', ink: '45, 90, 105' }
  }
  return dark
    ? { cool: '103, 220, 226', soft: '149, 174, 234', hot: '242, 126, 190', ink: '194, 235, 240' }
    : { cool: '49, 151, 163', soft: '92, 121, 165', hot: '208, 92, 145', ink: '44, 88, 105' }
}

function rebuildScene() {
  seed = props.variant === 'gallery' ? 1729 : 2879
  const compact = width < 700
  const compactScale = compact ? 1.15 : 1
  sceneCenterZ = compact ? 18.5 : RING_CENTER_Z
  const particleCount = compact ? 38 : 64
  const frameCount = compact ? 5 : props.variant === 'gallery' ? 11 : 8
  const solidCount = compact ? 3 : 7

  scene.particles = Array.from({ length: particleCount }, (_, index) => {
    const placement = ringPlacement(index, particleCount, compact ? 6 : 8.5, compact ? 13.5 : 19)
    return {
      ...placement,
      ring: true,
      y: verticalWorldPosition(placement, index, 0.08, compact ? 0.018 : 0.025),
      size: range(0.55, 1.75) * compactScale,
      phase: range(0, Math.PI * 2),
      speed: range(0.25, 0.8),
      group: index % 7,
      ...motionState(index, 'particle'),
    }
  })

  scene.frames = Array.from({ length: frameCount }, (_, index) => {
    const placement = ringPlacement(index, frameCount, compact ? 7 : 10, compact ? 13 : 18)
    const shape = FRAME_SHAPES[index % FRAME_SHAPES.length]
    return {
      ...placement,
      ring: true,
      shape,
      y: verticalWorldPosition(placement, index, compact ? 0.12 : 0.18, compact ? 0.018 : 0.028),
      width: range(1.05, 2.9) * compactScale,
      height: range(0.8, 2.35) * compactScale,
      depth: (shape === 'box' ? range(0.32, 0.76) : range(0.72, 1.45)) * compactScale,
      rx: range(-0.22, 0.22),
      ry: range(-0.48, 0.48),
      rz: range(-0.18, 0.18),
      phase: range(0, Math.PI * 2),
      hot: index % 5 === 0,
      mesh: createElasticMesh(shape, compact ? 3 : 4),
      ...motionState(particleCount + index, 'frame'),
    }
  })

  scene.solids = Array.from({ length: solidCount }, (_, index) => {
    const placement = ringPlacement(index + 0.5, solidCount, compact ? 6.5 : 9, compact ? 12 : 17)
    const shape = SOLID_SHAPES[index % SOLID_SHAPES.length]
    return {
      ...placement,
      ring: true,
      shape,
      y: verticalWorldPosition(placement, index, compact ? 0.24 : 0.31, compact ? 0.016 : 0.025),
      size: range(0.35, 0.9) * compactScale,
      rx: range(0, Math.PI),
      ry: range(0, Math.PI),
      rz: range(0, Math.PI),
      spin: range(0.08, 0.24) * (index % 2 ? -1 : 1),
      hot: index % 3 === 0,
      mesh: createElasticMesh(shape, compact ? 2 : 3),
      ...motionState(particleCount + frameCount + index, 'solid'),
    }
  })

  scene.streams = Array.from({ length: compact ? 2 : 4 }, (_, index) => {
    const placement = { x: 0, z: sceneCenterZ }
    return {
      side: index % 2 ? 1 : -1,
      offset: compact ? range(3.8, 6.2) : range(5.2, 8.8),
      y: verticalWorldPosition(placement, index, compact ? 0.18 : 0.1, 0.012),
      phase: range(0, Math.PI * 2),
      hot: index === 2,
      projected: Array.from(
        { length: 22 },
        () => ({ x: 0, y: 0, z: 0, rawZ: 0, scale: 1, visible: false }),
      ),
    }
  })

  const glyphText = 'SOS'
  const glyphCount = 3
  const glyphAngles = [Math.PI * 4 / 3, 0, Math.PI * 2 / 3]
  scene.glyphs = Array.from({ length: glyphCount }, (_, index) => {
    const angle = glyphAngles[index]
    const radius = compact ? 7.2 : 9.8
    const placement = {
      x: Math.sin(angle) * radius,
      z: sceneCenterZ + Math.cos(angle) * radius,
    }
    return {
      char: glyphText[index % glyphText.length],
      angle,
      ...placement,
      y: verticalWorldPosition(placement, index, 0.2, 0.012),
      size: range(1.55, 2.05) * (compact ? 1.08 : 1),
      hot: index === 1,
      projected: { x: 0, y: 0, z: 0, rawZ: 0, scale: 1, visible: false },
    }
  })

  formationIndex = 0
  applyFormationTargets()
}

function resize() {
  if (!canvas.value) return
  width = window.innerWidth
  height = window.innerHeight
  dpr = Math.min(window.devicePixelRatio || 1, width < 700 ? 2 : 1.4)
  canvas.value.width = Math.max(1, Math.round(width * dpr))
  canvas.value.height = Math.max(1, Math.round(height * dpr))
  canvas.value.style.width = `${width}px`
  canvas.value.style.height = `${height}px`
  context = canvas.value.getContext('2d')
  context?.setTransform(dpr, 0, 0, dpr, 0, 0)
  pointer.x ||= width * 0.5
  pointer.y ||= height * 0.5
  pointer.targetX ||= width * 0.5
  pointer.targetY ||= height * 0.5
  refreshProjectionCache()
  rebuildScene()
  if (reducedMotion) render(0, 0)
}

function worldDepth(baseZ, speed = 1) {
  const span = FAR - NEAR
  return NEAR + ((((baseZ - NEAR - travel * speed) % span) + span) % span)
}

function turnWorld(point) {
  const relativeZ = point.z - sceneCenterZ
  return {
    x: point.x * projectionCache.cosine - relativeZ * projectionCache.sine,
    y: point.y,
    z: sceneCenterZ + point.x * projectionCache.sine + relativeZ * projectionCache.cosine,
  }
}

function projectCoordinates(pointX, pointY, pointZ, output) {
  const relativeZ = pointZ - sceneCenterZ
  const turnedX = pointX * projectionCache.cosine - relativeZ * projectionCache.sine
  const rawZ = sceneCenterZ + pointX * projectionCache.sine + relativeZ * projectionCache.cosine
  const z = Math.max(0.5, rawZ)
  const scale = projectionCache.fov / z
  let x = width * 0.5 + (turnedX - camera.x) * scale
  let y = projectionCache.horizon + (pointY - camera.y) * scale

  if (shock.power > 0.002) {
    const dx = x - shock.x
    const dy = y - shock.y
    const distance = Math.hypot(dx, dy) || 1
    const wave = Math.sin(distance * 0.032 - shock.age * 10) * shock.power
    const falloff = Math.max(0, 1 - distance / projectionCache.maxDimension)
    x += (dx / distance) * wave * 18 * falloff
    y += (dy / distance) * wave * 18 * falloff
  }

  output.x = x
  output.y = y
  output.z = z
  output.rawZ = rawZ
  output.scale = scale
  output.visible = rawZ > 0.8 && x > -320 && x < width + 320 && y > -320 && y < height + 320
  return output
}

function withAlpha(rgb, alpha) {
  return `rgba(${rgb}, ${Math.max(0, Math.min(1, alpha))})`
}

function dynamicItems() {
  return [...scene.particles, ...scene.frames, ...scene.solids]
}

function motionSpeed(item) {
  if (item.kind === 'particle') return item.speed
  if (item.kind === 'frame') return 0.88
  return 0.62
}

function itemWorldDepth(item) {
  const depth = item.ring ? item.z : worldDepth(item.z, motionSpeed(item))
  return depth + item.offsetZ
}

function applyFormationTargets() {
  const mode = formationIndex % 4
  const uploadScale = props.variant === 'upload' ? 0.72 : 1

  dynamicItems().forEach(item => {
    const angle = item.morphPhase + item.z * 0.22
    let desiredX = item.x
    let desiredY = item.y
    let desiredZ = 0

    if (mode === 1) {
      desiredX += Math.sin(angle * 1.25) * 2.8
      desiredY += Math.cos(angle * 1.25) * 1.65
      desiredZ = Math.sin(angle * 0.72) * 1.1
    } else if (mode === 2) {
      desiredX += Math.sin(item.z * 0.34 + item.morphPhase) * 3.2
      desiredY += Math.sin(item.z * 0.52 + item.morphPhase * 0.6) * 1.85
      desiredZ = Math.cos(angle * 1.4) * 1.35
    } else if (mode === 3) {
      const side = item.order % 2 ? 1 : -1
      desiredX += side * (1.8 + Math.sin(item.morphPhase) * 1.2)
      desiredY += Math.cos(item.morphPhase * 1.35) * 1.6
      desiredZ = Math.sin(item.morphPhase * 1.8) * 1.05
    }

    const coherence = (item.kind === 'particle' ? 0.72 : item.kind === 'frame' ? 0.54 : 0.6) * uploadScale
    item.targetX = (desiredX - item.x) * coherence
    item.targetY = (desiredY - item.y) * coherence
    item.targetZ = desiredZ * coherence
    item.targetRotationX = mode ? Math.sin(angle + mode) * 0.24 * coherence : 0
    item.targetRotationY = mode ? Math.cos(angle * 0.8 - mode) * 0.38 * coherence : 0
    item.targetRotationZ = mode ? Math.sin(angle * 1.45) * 0.2 * coherence : 0
  })
}

function advanceFormation() {
  formationIndex = (formationIndex + 1) % 4
  applyFormationTargets()
}

function stepObjectPhysics(item, dt, pointerSpeed) {
  let influence = 0

  if (pointer.active) {
    const depth = itemWorldDepth(item)
    const projected = projectCoordinates(
      item.x + item.offsetX,
      item.y + item.offsetY,
      depth,
      item.projected,
    )
    const influenceRadius = Math.min(260, Math.max(150, width * 0.2))
    const dx = projected.x - pointer.x
    const dy = projected.y - pointer.y
    const distance = Math.hypot(dx, dy) || 1

    if (projected.visible && distance < influenceRadius) {
      influence = (1 - distance / influenceRadius) ** 2
      const unitX = dx / distance
      const unitY = dy / distance
      const worldPerPixel = projected.z / Math.max(1, projectionCache.fov)
      const push = 58 + Math.min(72, pointerSpeed) * 1.8
      const cappedVelocityX = Math.max(-72, Math.min(72, pointer.velocityX))
      const cappedVelocityY = Math.max(-72, Math.min(72, pointer.velocityY))
      const dragX = cappedVelocityX * 0.72
      const dragY = cappedVelocityY * 0.72
      const response = item.kind === 'particle' ? 6.8 : item.kind === 'frame' ? 2.4 : 2.1

      item.velocityX += (unitX * push + dragX) * worldPerPixel * influence * response * dt
      item.velocityY += (unitY * push + dragY) * worldPerPixel * influence * response * dt
      item.velocityZ += (0.18 + pointerSpeed * 0.012) * influence * response * dt
      item.angularVelocityX += (unitY * 0.8 + cappedVelocityY * 0.012) * influence * dt
      item.angularVelocityY += (-unitX * 0.9 + cappedVelocityX * 0.014) * influence * dt
      item.angularVelocityZ += (unitX * dragY - unitY * dragX) * influence * 0.006 * dt
    }
  }

  const stiffness = item.kind === 'particle' ? 5.4 : 3.8
  const rotationStiffness = item.kind === 'particle' ? 4.2 : 3.1
  item.velocityX += (item.targetX - item.offsetX) * stiffness * dt
  item.velocityY += (item.targetY - item.offsetY) * stiffness * dt
  item.velocityZ += (item.targetZ - item.offsetZ) * stiffness * dt
  item.angularVelocityX += (item.targetRotationX - item.rotationX) * rotationStiffness * dt
  item.angularVelocityY += (item.targetRotationY - item.rotationY) * rotationStiffness * dt
  item.angularVelocityZ += (item.targetRotationZ - item.rotationZ) * rotationStiffness * dt

  const damping = Math.exp(-(item.kind === 'particle' ? 4.2 : 3.15) * dt)
  const angularDamping = Math.exp(-3.4 * dt)
  item.velocityX *= damping
  item.velocityY *= damping
  item.velocityZ *= damping
  item.angularVelocityX *= angularDamping
  item.angularVelocityY *= angularDamping
  item.angularVelocityZ *= angularDamping
  item.offsetX += item.velocityX * dt
  item.offsetY += item.velocityY * dt
  item.offsetZ += item.velocityZ * dt
  item.rotationX += item.angularVelocityX * dt
  item.rotationY += item.angularVelocityY * dt
  item.rotationZ += item.angularVelocityZ * dt

  return influence
}

function elasticItemTransform(item, time) {
  const transform = item.mesh.transform
  if (transform.time === time) return transform
  const framePulse = item.kind === 'frame'
    ? Math.sin(time * 0.00032 + item.phase) * 0.05
    : 0
  const spin = item.kind === 'solid' ? time * 0.00012 * item.spin * 8 : 0
  const ambientY = item.kind === 'frame' ? Math.sin(time * 0.00015 + item.phase) * 0.08 : 0
  const rx = item.rx + item.rotationX + spin
  const ry = item.ry + item.rotationY + ambientY + spin * 1.3
  const rz = item.rz + item.rotationZ - spin * 0.7
  transform.time = time
  transform.scaleX = item.kind === 'frame' ? item.width * (0.5 + framePulse) : item.size
  transform.scaleY = item.kind === 'frame' ? item.height * (0.5 + framePulse) : item.size
  transform.scaleZ = item.kind === 'frame' ? item.depth * 0.5 : item.size
  transform.cx = Math.cos(rx)
  transform.sx = Math.sin(rx)
  transform.cy = Math.cos(ry)
  transform.sy = Math.sin(ry)
  transform.cz = Math.cos(rz)
  transform.sz = Math.sin(rz)
  return transform
}

function elasticMeshPoints(item, worldZ, time) {
  const transform = elasticItemTransform(item, time)
  item.mesh.nodes.forEach((node, index) => {
    const x = node.baseX * transform.scaleX + node.offsetX
    const y = node.baseY * transform.scaleY + node.offsetY
    const z = node.baseZ * transform.scaleZ + node.offsetZ
    const rotatedY = y * transform.cx - z * transform.sx
    const rotatedZ = y * transform.sx + z * transform.cx
    const rotatedX = x * transform.cy + rotatedZ * transform.sy
    const turnedZ = -x * transform.sy + rotatedZ * transform.cy
    const turnedX = rotatedX * transform.cz - rotatedY * transform.sz
    const turnedY = rotatedX * transform.sz + rotatedY * transform.cz
    projectCoordinates(
      item.x + item.offsetX + turnedX,
      item.y + item.offsetY + turnedY,
      worldZ + turnedZ,
      item.mesh.projected[index],
    )
  })
  return item.mesh.projected
}

function stepElasticMesh(item, dt, time, pointerSpeed) {
  if (!item.mesh) return 0
  if (!pointer.active && !item.mesh.awake) return 0
  let strongest = 0

  if (pointer.active) {
    item.mesh.awake = true
    item.mesh.restFrames = 0
    const worldZ = itemWorldDepth(item)
    const radius = Math.min(238, Math.max(135, width * 0.18))
    const fov = Math.min(width * 0.92, height * 1.2)
    const cappedVelocityX = Math.max(-82, Math.min(82, pointer.velocityX))
    const cappedVelocityY = Math.max(-82, Math.min(82, pointer.velocityY))
    const projected = elasticMeshPoints(item, worldZ, time)
    item.mesh.nodes.forEach((node, index) => {
      const point = projected[index]
      if (!point.visible) return
      const dx = point.x - pointer.x
      const dy = point.y - pointer.y
      const distance = Math.hypot(dx, dy) || 1
      if (distance >= radius) return
      const influence = (1 - distance / radius) ** 2
      strongest = Math.max(strongest, influence)
      const unitX = dx / distance
      const unitY = dy / distance
      const worldPerPixel = point.z / Math.max(1, fov)
      const dragX = cappedVelocityX * 1.9
      const dragY = cappedVelocityY * 1.9
      const radial = 52 + Math.min(70, pointerSpeed) * 0.55
      const response = item.kind === 'frame' ? 13.5 : 11.5

      node.velocityX += (dragX + unitX * radial) * worldPerPixel * influence * response * dt
      node.velocityY += (dragY + unitY * radial) * worldPerPixel * influence * response * dt
      node.velocityZ += (unitX * cappedVelocityX + unitY * cappedVelocityY) * 0.006 * influence * response * dt
    })
  }

  const coupling = item.kind === 'frame' ? 8.5 : 10
  item.mesh.edges.forEach(edge => {
    for (let index = 1; index < edge.length; index += 1) {
      const a = item.mesh.nodes[edge[index - 1]]
      const b = item.mesh.nodes[edge[index]]
      const forceX = (b.offsetX - a.offsetX) * coupling * dt
      const forceY = (b.offsetY - a.offsetY) * coupling * dt
      const forceZ = (b.offsetZ - a.offsetZ) * coupling * dt
      a.velocityX += forceX
      a.velocityY += forceY
      a.velocityZ += forceZ
      b.velocityX -= forceX
      b.velocityY -= forceY
      b.velocityZ -= forceZ
    }
  })

  const homeStiffness = item.kind === 'frame' ? 10.5 : 12
  const damping = Math.exp(-(item.kind === 'frame' ? 4.7 : 5.2) * dt)
  const maxOffset = item.kind === 'frame' ? 1.45 : 0.82
  let maxMotion = 0
  item.mesh.nodes.forEach(node => {
    node.velocityX += -node.offsetX * homeStiffness * dt
    node.velocityY += -node.offsetY * homeStiffness * dt
    node.velocityZ += -node.offsetZ * homeStiffness * dt
    node.velocityX *= damping
    node.velocityY *= damping
    node.velocityZ *= damping
    node.offsetX += node.velocityX * dt
    node.offsetY += node.velocityY * dt
    node.offsetZ += node.velocityZ * dt
    const magnitude = Math.hypot(node.offsetX, node.offsetY, node.offsetZ)
    maxMotion = Math.max(
      maxMotion,
      magnitude,
      Math.abs(node.velocityX),
      Math.abs(node.velocityY),
      Math.abs(node.velocityZ),
    )
    if (magnitude > maxOffset) {
      const limit = maxOffset / magnitude
      node.offsetX *= limit
      node.offsetY *= limit
      node.offsetZ *= limit
      node.velocityX *= 0.72
      node.velocityY *= 0.72
      node.velocityZ *= 0.72
    }
  })
  if (!pointer.active) {
    item.mesh.restFrames = maxMotion < 0.00015 ? item.mesh.restFrames + 1 : 0
    if (item.mesh.restFrames >= 6) {
      item.mesh.nodes.forEach(node => {
        node.offsetX = 0
        node.offsetY = 0
        node.offsetZ = 0
        node.velocityX = 0
        node.velocityY = 0
        node.velocityZ = 0
      })
      item.mesh.awake = false
      item.mesh.restFrames = 0
    }
  }
  return strongest
}

function updateScenePhysics(dt, time, pointerSpeed) {
  let agitation = 0
  dynamicItems().forEach(item => {
    agitation += stepObjectPhysics(item, dt, pointerSpeed)
    agitation += stepElasticMesh(item, dt, time, pointerSpeed) * 0.7
  })

  if (pointerSpeed > 7 && agitation > 0.35 && time > formationCooldownUntil) {
    advanceFormation()
    formationCooldownUntil = time + 1450
  }

  fieldWake.velocityX += pointer.velocityX * agitation * 0.22 * dt
  fieldWake.velocityY += pointer.velocityY * agitation * 0.22 * dt
  fieldWake.velocityRotation += (pointer.velocityX - pointer.velocityY) * agitation * 0.0007 * dt
  fieldWake.velocityX += -fieldWake.x * 2.8 * dt
  fieldWake.velocityY += -fieldWake.y * 2.8 * dt
  fieldWake.velocityRotation += -fieldWake.rotation * 3.1 * dt
  const wakeDamping = Math.exp(-2.7 * dt)
  fieldWake.velocityX *= wakeDamping
  fieldWake.velocityY *= wakeDamping
  fieldWake.velocityRotation *= wakeDamping
  fieldWake.x += fieldWake.velocityX * dt
  fieldWake.y += fieldWake.velocityY * dt
  fieldWake.rotation += fieldWake.velocityRotation * dt
}

function applyElasticImpulse(item, x, y, time) {
  if (!item.mesh) return
  const radius = Math.min(330, width * 0.27)
  const points = elasticMeshPoints(item, itemWorldDepth(item), time)
  let affected = false
  item.mesh.nodes.forEach((node, index) => {
    const point = points[index]
    if (!point.visible) return
    const dx = point.x - x
    const dy = point.y - y
    const distance = Math.hypot(dx, dy) || 1
    if (distance >= radius) return
    affected = true
    const influence = (1 - distance / radius) ** 2
    const worldPerPixel = point.z / Math.max(1, projectionCache.fov)
    node.velocityX += dx / distance * 235 * worldPerPixel * influence
    node.velocityY += dy / distance * 235 * worldPerPixel * influence
    node.velocityZ += (item.order % 2 ? 1 : -1) * influence * 0.9
  })
  if (affected) {
    item.mesh.awake = true
    item.mesh.restFrames = 0
  }
}

function applyPointerImpulse(x, y) {
  const radius = Math.min(380, width * 0.3)
  const impulseTime = performance.now()
  dynamicItems().forEach(item => {
    applyElasticImpulse(item, x, y, impulseTime)
    const depth = itemWorldDepth(item)
    const projected = projectCoordinates(
      item.x + item.offsetX,
      item.y + item.offsetY,
      depth,
      item.projected,
    )
    const dx = projected.x - x
    const dy = projected.y - y
    const distance = Math.hypot(dx, dy) || 1
    if (!projected.visible || distance >= radius) return
    const influence = (1 - distance / radius) ** 2
    const worldPerPixel = projected.z / Math.max(1, projectionCache.fov)
    item.velocityX += dx / distance * 190 * worldPerPixel * influence
    item.velocityY += dy / distance * 190 * worldPerPixel * influence
    item.velocityZ += influence * 1.8
    item.angularVelocityX += dy / distance * influence * 0.85
    item.angularVelocityY += -dx / distance * influence * 0.85
    item.angularVelocityZ += (item.order % 2 ? 1 : -1) * influence * 0.55
  })
  advanceFormation()
  formationCooldownUntil = performance.now() + 1200
}

function drawStreams(colors, time) {
  scene.streams.forEach((stream, streamIndex) => {
    context.beginPath()
    let started = false
    for (let index = 0; index < 22; index += 1) {
      const baseZ = NEAR + index * ((FAR - NEAR) / 21)
      const z = worldDepth(baseZ, 0.72)
      const sway = Math.sin(time * 0.00042 + stream.phase + z * 0.34) * 0.72
      const lift = Math.cos(time * 0.00031 + stream.phase + z * 0.22) * 0.42
      const point = projectCoordinates(
        stream.side * stream.offset + sway,
        stream.y + lift,
        z,
        stream.projected[index],
      )
      if (!point.visible) continue
      if (!started) {
        context.moveTo(point.x, point.y)
        started = true
      } else {
        context.lineTo(point.x, point.y)
      }
    }
    context.lineWidth = streamIndex === 0 ? 1.4 : 0.8
    context.strokeStyle = withAlpha(stream.hot ? colors.hot : colors.soft, stream.hot ? 0.16 : 0.11)
    context.stroke()
  })
}

function traceElasticEdges(mesh, points) {
  mesh.edges.forEach(edge => {
    let started = false
    edge.forEach(nodeIndex => {
      const point = points[nodeIndex]
      if (!point.visible) {
        started = false
        return
      }
      if (started) context.lineTo(point.x, point.y)
      else context.moveTo(point.x, point.y)
      started = true
    })
  })
}

function drawGlyphs(time) {
  context.save()
  context.textAlign = 'center'
  context.textBaseline = 'middle'
  scene.glyphs.forEach(glyph => {
    const point = projectCoordinates(glyph.x, glyph.y, glyph.z, glyph.projected)
    if (!point.visible) return
    const angle = glyph.angle - view.yaw
    const facing = Math.max(0.22, Math.abs(Math.cos(angle)))
    const frontness = (1 - Math.cos(angle)) * 0.5
    const fontSize = Math.max(28, Math.min(176, glyph.size * point.scale * 2.45))
    const alpha = (0.024 + frontness * 0.095) * (0.92 + Math.sin(time * 0.00028 + glyph.angle) * 0.08)
    const breath = (Math.sin(time * 0.00028 + glyph.angle * 0.18) + 1) * 0.5
    const startHue = 48 - breath * 30
    const endHue = 38 - breath * 32
    const glowHue = 44 - breath * 36

    context.save()
    context.translate(point.x, point.y)
    context.rotate(Math.sin(angle) * 0.055)
    context.scale(facing, 1)
    context.font = `800 ${fontSize}px ui-sans-serif, system-ui, sans-serif`
    context.lineWidth = Math.max(0.7, fontSize * 0.008)
    const gradient = context.createLinearGradient(-fontSize * 0.48, -fontSize * 0.36, fontSize * 0.48, fontSize * 0.36)
    gradient.addColorStop(0, `hsla(${startHue}, 94%, 62%, ${Math.min(0.34, 0.045 + alpha * 1.45)})`)
    gradient.addColorStop(1, `hsla(${endHue}, 92%, 58%, ${Math.min(0.32, 0.04 + alpha * 1.35)})`)
    context.shadowColor = `hsla(${glowHue}, 96%, 61%, ${0.075 + breath * 0.075})`
    context.shadowBlur = 3.5 + breath * 4.5
    context.strokeStyle = gradient
    context.fillStyle = `hsla(${endHue}, 92%, 60%, ${0.012 + alpha * 0.28})`
    context.strokeText(glyph.char, 0, 0)
    context.fillText(glyph.char, 0, 0)
    context.restore()
  })
  context.restore()
}

function drawFrames(colors, time) {
  const ordered = scene.frames
    .map(item => {
      const depth = itemWorldDepth(item)
      const screenDepth = projectCoordinates(
        item.x + item.offsetX,
        item.y + item.offsetY,
        depth,
        item.projected,
      ).z
      return { item, depth, screenDepth }
    })
    .sort((a, b) => b.screenDepth - a.screenDepth)

  ordered.forEach(({ item, depth, screenDepth }) => {
    const vertices = elasticMeshPoints(item, depth, time)
    if (!vertices.some(vertex => vertex.visible)) return
    const proximity = 1 - Math.min(1, Math.max(0, (screenDepth - NEAR) / (FAR - NEAR)))
    const color = item.hot ? colors.hot : colors.cool

    context.beginPath()
    traceElasticEdges(item.mesh, vertices)
    context.lineCap = 'round'
    context.lineJoin = 'round'
    context.lineWidth = 1.25 + proximity * 1.35
    context.strokeStyle = withAlpha(colors.ink, 0.045 + proximity * 0.11)
    context.stroke()
    context.lineWidth = 0.85 + proximity * 1.05
    context.strokeStyle = withAlpha(color, 0.12 + proximity * 0.27)
    context.stroke()

    const face = item.mesh.face
    if (screenDepth < 17 && face?.every(index => vertices[index].visible)) {
      context.beginPath()
      context.moveTo(vertices[face[0]].x, vertices[face[0]].y)
      face.slice(1).forEach(index => context.lineTo(vertices[index].x, vertices[index].y))
      context.closePath()
      context.fillStyle = withAlpha(color, 0.022 + proximity * 0.042)
      context.fill()
    }

    const corner = vertices[item.mesh.accentNode]
    if (corner.visible && screenDepth < 24) {
      const tick = Math.min(10, 2.5 + proximity * 8)
      context.beginPath()
      context.moveTo(corner.x - tick, corner.y)
      context.lineTo(corner.x + tick, corner.y)
      context.moveTo(corner.x, corner.y - tick)
      context.lineTo(corner.x, corner.y + tick)
      context.lineWidth = 0.75
      context.strokeStyle = withAlpha(color, 0.16 + proximity * 0.18)
      context.stroke()
    }
  })
}

function drawSolids(colors, time) {
  scene.solids.forEach(item => {
    const depth = itemWorldDepth(item)
    const vertices = elasticMeshPoints(item, depth, time)
    if (!vertices.some(vertex => vertex.visible)) return
    const screenDepth = projectCoordinates(
      item.x + item.offsetX,
      item.y + item.offsetY,
      depth,
      item.projected,
    ).z
    const proximity = 1 - Math.min(1, Math.max(0, screenDepth / FAR))
    context.beginPath()
    traceElasticEdges(item.mesh, vertices)
    context.lineCap = 'round'
    context.lineJoin = 'round'
    context.lineWidth = 1.15 + proximity * 1.1
    context.strokeStyle = withAlpha(colors.ink, 0.045 + proximity * 0.1)
    context.stroke()
    context.lineWidth = 0.8 + proximity * 0.85
    context.strokeStyle = withAlpha(item.hot ? colors.hot : colors.soft, 0.13 + proximity * 0.24)
    context.stroke()
  })
}

function drawParticles(colors, time) {
  const connectParticles = width >= 700
  const projected = connectParticles ? [] : null
  scene.particles.forEach(item => {
    const z = itemWorldDepth(item)
    const drift = Math.sin(time * 0.00048 + item.phase) * 0.3
    const point = projectCoordinates(
      item.x + item.offsetX + drift,
      item.y + item.offsetY + Math.cos(time * 0.00036 + item.phase) * 0.22,
      z,
      item.projected,
    )
    if (!point.visible) return

    const proximity = 1 - Math.min(1, point.z / FAR)
    const radius = Math.min(4.2, item.size * (0.55 + proximity * 2.3))
    const rgb = item.group === 0 ? colors.hot : item.group % 3 === 0 ? colors.soft : colors.cool
    context.beginPath()
    context.arc(point.x, point.y, radius, 0, Math.PI * 2)
    context.fillStyle = withAlpha(rgb, 0.12 + proximity * 0.48)
    context.fill()
    if (connectParticles) projected.push({ x: point.x, y: point.y, z: point.z, group: item.group, rgb })
  })

  if (!connectParticles) return
  context.lineWidth = 0.65
  for (let index = 0; index < projected.length; index += 1) {
    const a = projected[index]
    for (let otherIndex = index + 1; otherIndex < projected.length; otherIndex += 1) {
      const b = projected[otherIndex]
      if (a.group !== b.group || Math.abs(a.z - b.z) > 8) continue
      const distance = Math.hypot(a.x - b.x, a.y - b.y)
      if (distance > 118) continue
      context.beginPath()
      context.moveTo(a.x, a.y)
      context.lineTo(b.x, b.y)
      context.strokeStyle = withAlpha(a.rgb, (1 - distance / 118) * 0.12)
      context.stroke()
    }
  }
}

function drawShock(colors) {
  if (shock.power < 0.008) return
  for (let ring = 0; ring < 3; ring += 1) {
    const radius = shock.age * (92 + ring * 28)
    context.beginPath()
    context.arc(shock.x, shock.y, radius, 0, Math.PI * 2)
    context.lineWidth = Math.max(0.5, 1.4 - ring * 0.35)
    context.strokeStyle = withAlpha(ring === 1 ? colors.hot : colors.cool, shock.power * (0.2 - ring * 0.035))
    context.stroke()
  }
}

function setCachedStyleProperty(element, cache, name, value) {
  if (!element || cache.get(name) === value) return
  cache.set(name, value)
  element.style.setProperty(name, value)
}

function update(dt, time) {
  const smoothing = 1 - Math.pow(0.0009, dt)
  pointer.x += (pointer.targetX - pointer.x) * smoothing
  pointer.y += (pointer.targetY - pointer.y) * smoothing
  const pointerSpeed = Math.hypot(pointer.velocityX, pointer.velocityY)

  const targetCameraX = pointer.normalizedX * (props.variant === 'gallery' ? 0.42 : 0.3)
  const targetCameraY = pointer.normalizedY * 0.2 + scrollProgress * 0.16
  camera.x += (targetCameraX - camera.x) * (1 - Math.pow(0.006, dt))
  camera.y += (targetCameraY - camera.y) * (1 - Math.pow(0.008, dt))
  camera.roll += ((pointer.normalizedX * -0.004 + scrollVelocity * 0.00002) - camera.roll) * (1 - Math.pow(0.01, dt))

  ambientYaw += dt * (props.variant === 'gallery' ? 0.008 : 0.005)
  view.targetYaw = scrollYaw + ambientYaw
  const previousYaw = view.yaw
  view.yaw += (view.targetYaw - view.yaw) * (1 - Math.pow(0.004, dt))
  view.velocity = dt > 0 ? (view.yaw - previousYaw) / dt : 0
  refreshProjectionCache()
  scrollVelocity *= Math.pow(0.018, dt)
  travel += dt * (props.variant === 'gallery' ? 0.19 : 0.13) + scrollVelocity * dt * 0.0015
  shock.age += dt
  shock.power *= Math.pow(0.018, dt)
  updateScenePhysics(dt, time, pointerSpeed)
  pointer.velocityX *= Math.pow(0.002, dt)
  pointer.velocityY *= Math.pow(0.002, dt)

  if (width >= 700 && (readoutFrame += 1) % 8 === 0) {
    depthReadout.value = String(((travel % 100) + 100) % 100).padStart(4, '0').slice(0, 4)
  }

  if ((styleWriteFrame += 1) % 2 === 0) {
    const turnSine = projectionCache.sine
    const rootElement = root.value
    setCachedStyleProperty(rootElement, rootStyleCache, '--mx', `${pointer.x}px`)
    setCachedStyleProperty(rootElement, rootStyleCache, '--my', `${pointer.y}px`)
    setCachedStyleProperty(rootElement, rootStyleCache, '--nx', pointer.normalizedX.toFixed(3))
    setCachedStyleProperty(rootElement, rootStyleCache, '--ny', pointer.normalizedY.toFixed(3))
    setCachedStyleProperty(rootElement, rootStyleCache, '--scroll', scrollProgress.toFixed(3))
    setCachedStyleProperty(rootElement, rootStyleCache, '--energy', Math.min(1, pointerSpeed / 35).toFixed(3))
    setCachedStyleProperty(rootElement, rootStyleCache, '--wake-x', `${Math.max(-38, Math.min(38, fieldWake.x)).toFixed(2)}px`)
    setCachedStyleProperty(rootElement, rootStyleCache, '--wake-y', `${Math.max(-30, Math.min(30, fieldWake.y)).toFixed(2)}px`)
    setCachedStyleProperty(rootElement, rootStyleCache, '--wake-rot', `${Math.max(-8, Math.min(8, fieldWake.rotation)).toFixed(3)}deg`)
    setCachedStyleProperty(rootElement, rootStyleCache, '--view-turn', `${(view.yaw * 180 / Math.PI).toFixed(3)}deg`)
    setCachedStyleProperty(rootElement, rootStyleCache, '--turn-x', `${(turnSine * 42).toFixed(2)}px`)
    setCachedStyleProperty(rootElement, rootStyleCache, '--turn-x-neg', `${(turnSine * -52).toFixed(2)}px`)
    setCachedStyleProperty(rootElement, rootStyleCache, '--turn-x-soft', `${(turnSine * 9).toFixed(2)}px`)
    setCachedStyleProperty(document.documentElement, documentStyleCache, '--art-backdrop-x', `${(turnSine * 12).toFixed(2)}px`)
    setCachedStyleProperty(document.documentElement, documentStyleCache, '--art-mask-x', `${(turnSine * -19).toFixed(2)}px`)
    setCachedStyleProperty(document.documentElement, documentStyleCache, '--art-backdrop-tilt', `${(turnSine * 0.22).toFixed(3)}deg`)
  }
}

function render(time) {
  if (!context || !width || !height) return
  const colors = palette()
  context.setTransform(dpr, 0, 0, dpr, 0, 0)
  context.clearRect(0, 0, width, height)
  context.save()
  context.translate(width * 0.5, height * 0.5)
  context.rotate(camera.roll)
  context.translate(-width * 0.5, -height * 0.5)
  drawGlyphs(time)
  drawStreams(colors, time)
  drawFrames(colors, time)
  drawSolids(colors, time)
  drawParticles(colors, time)
  drawShock(colors)
  context.restore()
}

function animate(time) {
  if (hidden || reducedMotion) return
  const minimumFrameInterval = width < 700 ? 1000 / 30 : 0
  if (minimumFrameInterval && time - lastTime < minimumFrameInterval - 1) {
    animationFrame = window.requestAnimationFrame(animate)
    return
  }
  const dt = Math.min(0.034, Math.max(0.001, (time - lastTime) / 1000 || 0.016))
  lastTime = time
  update(dt, time)
  render(time)
  animationFrame = window.requestAnimationFrame(animate)
}

function start() {
  if (animationFrame || hidden || reducedMotion) return
  lastTime = performance.now()
  animationFrame = window.requestAnimationFrame(animate)
}

function stop() {
  if (animationFrame) window.cancelAnimationFrame(animationFrame)
  animationFrame = 0
}

function onPointerMove(event) {
  const nextX = event.clientX
  const nextY = event.clientY
  if (pointer.sampled) {
    pointer.velocityX += nextX - pointer.lastX
    pointer.velocityY += nextY - pointer.lastY
  }
  pointer.sampled = true
  pointer.lastX = nextX
  pointer.lastY = nextY
  pointer.targetX = nextX
  pointer.targetY = nextY
  pointer.normalizedX = nextX / Math.max(1, width) * 2 - 1
  pointer.normalizedY = nextY / Math.max(1, height) * 2 - 1
  pointer.active = event.pointerType !== 'touch'
}

function onPointerDown(event) {
  shock.x = event.clientX
  shock.y = event.clientY
  shock.power = 1
  shock.age = 0
  applyPointerImpulse(event.clientX, event.clientY)
}

function onPointerLeave() {
  pointer.active = false
  pointer.targetX = width * 0.5
  pointer.targetY = height * 0.5
  pointer.normalizedX = 0
  pointer.normalizedY = 0
}

function onScroll() {
  const nextY = window.scrollY
  const delta = nextY - lastScrollY
  lastScrollY = nextY
  const scrollDelta = Math.max(-120, Math.min(120, delta))
  scrollVelocity += scrollDelta * 0.5
  scrollYaw = nextY * 0.00086
  view.targetYaw = scrollYaw + ambientYaw
  const scrollable = Math.max(1, document.documentElement.scrollHeight - window.innerHeight)
  scrollProgress = Math.max(0, Math.min(1, nextY / scrollable))
  if (reducedMotion) {
    view.yaw = view.targetYaw
    refreshProjectionCache()
    render(0)
  }
}

function onVisibilityChange() {
  hidden = document.hidden
  if (hidden) stop()
  else start()
}

function onMotionChange(event) {
  reducedMotion = event.matches
  if (reducedMotion) {
    stop()
    update(0, 0)
    render(0, 0)
  } else {
    start()
  }
}

watch(() => props.variant, () => {
  travel = 0
  refreshProjectionCache()
  rebuildScene()
  if (reducedMotion) render(0, 0)
})

onMounted(() => {
  mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)')
  reducedMotion = mediaQuery.matches
  lastScrollY = window.scrollY
  onScroll()
  resizeObserver = new ResizeObserver(resize)
  resizeObserver.observe(root.value)
  window.addEventListener('pointermove', onPointerMove, { passive: true })
  window.addEventListener('pointerdown', onPointerDown, { passive: true })
  document.documentElement.addEventListener('pointerleave', onPointerLeave, { passive: true })
  window.addEventListener('scroll', onScroll, { passive: true })
  document.addEventListener('visibilitychange', onVisibilityChange)
  mediaQuery.addEventListener?.('change', onMotionChange)
  resize()
  start()
})

onBeforeUnmount(() => {
  stop()
  resizeObserver?.disconnect()
  mediaQuery?.removeEventListener?.('change', onMotionChange)
  window.removeEventListener('pointermove', onPointerMove)
  window.removeEventListener('pointerdown', onPointerDown)
  document.documentElement.removeEventListener('pointerleave', onPointerLeave)
  window.removeEventListener('scroll', onScroll)
  document.removeEventListener('visibilitychange', onVisibilityChange)
  document.documentElement.style.removeProperty('--art-backdrop-x')
  document.documentElement.style.removeProperty('--art-mask-x')
  document.documentElement.style.removeProperty('--art-backdrop-tilt')
  rootStyleCache.clear()
  documentStyleCache.clear()
})
</script>

<style scoped>
.art-space {
  --mx: 50vw;
  --my: 50vh;
  --nx: 0;
  --ny: 0;
  --scroll: 0;
  --energy: 0;
  --wake-x: 0px;
  --wake-y: 0px;
  --wake-rot: 0deg;
  --view-turn: 0deg;
  --turn-x: 0px;
  --turn-x-neg: 0px;
  --turn-x-soft: 0px;
  --space-cool: 39, 156, 164;
  --space-soft: 92, 120, 166;
  --space-hot: 214, 94, 149;
  position: fixed;
  z-index: 0;
  inset: 0;
  overflow: hidden;
  pointer-events: none;
  perspective: 900px;
  contain: strict;
  opacity: 1;
}

.art-space--upload {
  --space-cool: 35, 151, 151;
  --space-soft: 86, 132, 158;
  --space-hot: 218, 124, 92;
}

.art-space__canvas,
.art-space__calm-current,
.art-space__soft-ripples,
.art-space__scan,
.art-space__vignette,
.art-space__aurora {
  position: absolute;
  inset: 0;
}

.art-space__calm-current {
  inset: -24%;
  overflow: hidden;
  opacity: 0.42;
  transform: translate3d(var(--turn-x-soft), 0, 0);
  will-change: transform;
}

.art-space__calm-current i {
  position: absolute;
  width: 70%;
  aspect-ratio: 1.7;
  border-radius: 48% 52% 60% 40% / 55% 42% 58% 45%;
  filter: blur(58px);
  will-change: transform;
}

.art-space__calm-current i:first-child {
  top: 2%;
  left: -4%;
  background: radial-gradient(ellipse, rgba(217, 255, 247, 0.32), rgba(var(--space-cool), 0.1) 48%, transparent 72%);
  animation: art-space-calm-a 24s ease-in-out infinite alternate;
}

.art-space__calm-current i:last-child {
  right: -3%;
  bottom: -2%;
  background: radial-gradient(ellipse, rgba(255, 234, 219, 0.28), rgba(var(--space-hot), 0.08) 46%, transparent 72%);
  animation: art-space-calm-b 31s ease-in-out infinite alternate;
}

.art-space__soft-ripples {
  inset: -12%;
  opacity: 0.11;
  background:
    repeating-radial-gradient(ellipse at 28% 32%, transparent 0 28px, rgba(255, 255, 255, 0.26) 30px, transparent 33px 58px),
    repeating-radial-gradient(ellipse at 76% 70%, transparent 0 42px, rgba(var(--space-cool), 0.16) 44px, transparent 47px 76px);
  background-size: 128% 112%, 118% 134%;
  mask-image: linear-gradient(115deg, transparent 4%, #000 30%, #000 72%, transparent 96%);
  animation: art-space-ripples 38s linear infinite alternate;
}

.art-space__canvas {
  width: 100%;
  height: 100%;
  opacity: 0.98;
}

.art-space__aurora {
  width: clamp(340px, 46vw, 780px);
  height: clamp(340px, 46vw, 780px);
  border-radius: 50%;
  filter: blur(48px);
  will-change: transform;
}

.art-space__aurora--a {
  top: -24%;
  left: -12%;
  background: radial-gradient(circle, rgba(var(--space-cool), 0.12), transparent 67%);
  transform: translate3d(calc(var(--nx) * 34px + var(--turn-x)), calc(var(--ny) * 24px + var(--scroll) * 70px), 0);
}

.art-space__aurora--b {
  top: 34%;
  right: -18%;
  left: auto;
  background: radial-gradient(circle, rgba(var(--space-hot), 0.085), transparent 68%);
  transform: translate3d(calc(var(--nx) * -52px + var(--turn-x-neg)), calc(var(--ny) * -28px - var(--scroll) * 90px), 0);
}

.art-space__aurora--c {
  top: auto;
  right: 18%;
  bottom: -34%;
  left: auto;
  background: radial-gradient(circle, rgba(var(--space-soft), 0.09), transparent 68%);
  transform: translate3d(calc(var(--nx) * 22px + var(--turn-x)), calc(var(--ny) * 18px), 0);
}

.art-space__scan {
  opacity: 0.13;
  transform: translate3d(var(--turn-x-soft), 0, 0);
  background:
    repeating-linear-gradient(0deg, rgba(255, 255, 255, 0.22) 0 1px, transparent 1px 5px),
    linear-gradient(90deg, transparent 0 49.9%, rgba(var(--space-cool), 0.13) 50%, transparent 50.1%);
  mask-image: linear-gradient(to bottom, transparent, #000 12%, #000 88%, transparent);
}

.art-space__vignette {
  background:
    radial-gradient(circle 180px at var(--mx) var(--my), rgba(255, 255, 255, calc(0.03 + var(--energy) * 0.04)), transparent 72%),
    linear-gradient(90deg, rgba(var(--space-soft), 0.04), transparent 14% 86%, rgba(var(--space-cool), 0.04));
}

.art-space__ruler {
  position: absolute;
  top: 10vh;
  bottom: 8vh;
  width: 25px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  opacity: 0.38;
  transform: translateY(calc(var(--scroll) * -32px));
}

.art-space__ruler--left { left: 14px; align-items: flex-start; }
.art-space__ruler--right { right: 14px; align-items: flex-end; }
.art-space__ruler i {
  display: block;
  width: 13px;
  height: 1px;
  background: rgba(var(--space-cool), 0.5);
}
.art-space__ruler i:nth-child(4n + 1) { width: 24px; background: rgba(var(--space-hot), 0.58); }

.art-space__orbit {
  position: absolute;
  width: clamp(240px, 32vw, 520px);
  aspect-ratio: 1;
  border: 1px solid rgba(var(--space-soft), 0.12);
  border-radius: 50%;
  transform-style: preserve-3d;
  will-change: transform;
}

.art-space__orbit::before,
.art-space__orbit::after {
  content: '';
  position: absolute;
  border-radius: inherit;
  inset: 12%;
  border: 1px dashed rgba(var(--space-cool), 0.14);
}

.art-space__orbit::after {
  inset: 43%;
  border-style: solid;
  background: rgba(var(--space-hot), 0.06);
  box-shadow: 0 0 28px rgba(var(--space-hot), 0.1);
}

.art-space__orbit--a {
  top: 2vh;
  right: -11vw;
  transform:
    translate3d(
      calc(var(--nx) * -24px + var(--wake-x) * -0.7),
      calc(var(--ny) * -18px + var(--wake-y) * -0.7),
      -120px
    )
    rotateX(68deg)
    rotateZ(calc(18deg + var(--scroll) * 60deg - var(--wake-rot) - var(--view-turn) * 0.35))
    scaleX(calc(1 + var(--energy) * 0.16))
    scaleY(calc(1 - var(--energy) * 0.08));
}

.art-space__orbit--b {
  bottom: -24vh;
  left: -8vw;
  opacity: 0.7;
  transform:
    translate3d(
      calc(var(--nx) * 18px + var(--wake-x) * 0.5),
      calc(var(--ny) * 16px + var(--wake-y) * 0.5),
      -180px
    )
    rotateX(72deg)
    rotateZ(calc(-28deg - var(--scroll) * 45deg + var(--wake-rot) + var(--view-turn) * 0.28))
    scaleY(calc(1 + var(--energy) * 0.12));
}

.art-space--upload .art-space__orbit--a {
  top: 19vh;
  right: -16vw;
  border-radius: 18%;
  transform:
    translate3d(
      calc(var(--nx) * -18px + var(--wake-x) * -0.55),
      calc(var(--ny) * -12px + var(--wake-y) * -0.55),
      -160px
    )
    rotateX(66deg)
    rotateZ(calc(10deg + var(--scroll) * 32deg - var(--wake-rot) - var(--view-turn) * 0.24));
}

.art-space__telemetry {
  position: absolute;
  display: flex;
  gap: 10px;
  align-items: center;
  color: rgba(var(--space-soft), 0.56);
  font-family: ui-monospace, "SFMono-Regular", Consolas, monospace;
  font-size: 8px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}
.art-space__telemetry b { color: rgba(var(--space-cool), 0.72); font-weight: 700; }
.art-space__telemetry--top { top: 11vh; right: 30px; }
.art-space__telemetry--bottom { bottom: 24px; left: 30px; }

@keyframes art-space-calm-a {
  0% { transform: translate3d(-4%, -3%, 0) rotate(-4deg) scale(0.94); }
  45% { transform: translate3d(18%, 9%, 0) rotate(5deg) scale(1.08); }
  100% { transform: translate3d(34%, -2%, 0) rotate(-1deg) scale(1); }
}

@keyframes art-space-calm-b {
  0% { transform: translate3d(5%, 5%, 0) rotate(4deg) scale(1.04); }
  50% { transform: translate3d(-17%, -11%, 0) rotate(-6deg) scale(0.92); }
  100% { transform: translate3d(-31%, 4%, 0) rotate(2deg) scale(1.08); }
}

@keyframes art-space-ripples {
  0% { transform: translate3d(calc(-2% + var(--turn-x-soft)), -1%, 0) rotate(-1deg) scale(1.02); }
  100% { transform: translate3d(calc(3% + var(--turn-x-soft)), 2%, 0) rotate(1deg) scale(1.08); }
}

@media (max-width: 700px) {
  .art-space__canvas { opacity: 0.92; }
  .art-space__ruler { opacity: 0.22; }
  .art-space__telemetry--top,
  .art-space__telemetry--bottom { display: none; }
  .art-space__orbit { opacity: 0.48; }
  .art-space__aurora { filter: blur(34px); }
  .art-space__calm-current { opacity: 0.5; }
}

@media (prefers-reduced-motion: reduce) {
  .art-space__aurora,
  .art-space__orbit,
  .art-space__ruler { transform: none; }
  .art-space__calm-current i,
  .art-space__soft-ripples { animation: none; }
}
</style>
