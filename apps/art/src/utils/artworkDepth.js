const instances = new WeakMap()
const registrations = new Map()
let reducedMotion = null
let finePointer = null
let capabilityListenersAttached = false

const DEFAULTS = {
  maxTilt: 7,
  lift: 4,
  scale: 1.012,
}

function numeric(value, fallback) {
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : fallback
}

function optionsFrom(binding) {
  const value = binding?.value || {}
  return {
    maxTilt: numeric(value.maxTilt, DEFAULTS.maxTilt),
    lift: numeric(value.lift, DEFAULTS.lift),
    scale: numeric(value.scale, DEFAULTS.scale),
  }
}

function ensureCapabilityQueries() {
  reducedMotion ||= window.matchMedia('(prefers-reduced-motion: reduce)')
  finePointer ||= window.matchMedia('(hover: hover) and (pointer: fine)')
}

function depthEnabled() {
  ensureCapabilityQueries()
  return finePointer.matches && !reducedMotion.matches
}

function createDepthController(element, binding) {
  const options = optionsFrom(binding)
  const current = { rx: 0, ry: 0, lift: 0, scale: 1, glareX: 50, glareY: 50, glare: 0 }
  const target = { ...current }
  let frame = 0
  let hovered = false
  let pressed = false
  let rect = null

  function write() {
    element.style.setProperty('--depth-rx', `${current.rx.toFixed(3)}deg`)
    element.style.setProperty('--depth-ry', `${current.ry.toFixed(3)}deg`)
    element.style.setProperty('--depth-lift', `${current.lift.toFixed(3)}px`)
    element.style.setProperty('--depth-scale', current.scale.toFixed(5))
    element.style.setProperty('--depth-glare-x', `${current.glareX.toFixed(2)}%`)
    element.style.setProperty('--depth-glare-y', `${current.glareY.toFixed(2)}%`)
    element.style.setProperty('--depth-glare-opacity', current.glare.toFixed(3))
  }

  function reset(immediate = false) {
    hovered = false
    pressed = false
    rect = null
    Object.assign(target, { rx: 0, ry: 0, lift: 0, scale: 1, glareX: 50, glareY: 50, glare: 0 })
    element.classList.remove('is-depth-active', 'is-depth-pressed')
    if (immediate) {
      Object.assign(current, target)
      if (frame) cancelAnimationFrame(frame)
      frame = 0
      write()
      return
    }
    schedule()
  }

  function tick() {
    frame = 0
    const smoothing = hovered ? 0.18 : 0.12
    let delta = 0
    for (const key of Object.keys(current)) {
      const difference = target[key] - current[key]
      current[key] += difference * smoothing
      delta = Math.max(delta, Math.abs(difference))
    }
    write()
    if (delta > 0.008) schedule()
  }

  function schedule() {
    if (!frame) frame = requestAnimationFrame(tick)
  }

  function updatePointer(event) {
    if (!depthEnabled()) return
    rect ||= element.getBoundingClientRect()
    const nx = Math.max(-1, Math.min(1, ((event.clientX - rect.left) / rect.width - 0.5) * 2))
    const ny = Math.max(-1, Math.min(1, ((event.clientY - rect.top) / rect.height - 0.5) * 2))
    target.rx = -ny * options.maxTilt * 0.72
    target.ry = nx * options.maxTilt
    target.lift = pressed ? -1 : -options.lift
    target.scale = pressed ? 0.996 : options.scale
    target.glareX = (nx + 1) * 50
    target.glareY = (ny + 1) * 50
    target.glare = pressed ? 0.11 : 0.22
    schedule()
  }

  function onEnter(event) {
    if (!depthEnabled()) return
    hovered = true
    rect = element.getBoundingClientRect()
    element.classList.add('is-depth-active')
    updatePointer(event)
  }

  function onMove(event) {
    if (!hovered) return
    updatePointer(event)
  }

  function onLeave() {
    reset(false)
  }

  function onDown(event) {
    if (!depthEnabled()) return
    pressed = true
    element.classList.add('is-depth-pressed')
    updatePointer(event)
  }

  function onUp(event) {
    pressed = false
    element.classList.remove('is-depth-pressed')
    if (hovered) updatePointer(event)
  }

  element.addEventListener('pointerenter', onEnter, { passive: true })
  element.addEventListener('pointermove', onMove, { passive: true })
  element.addEventListener('pointerleave', onLeave, { passive: true })
  element.addEventListener('pointerdown', onDown, { passive: true })
  element.addEventListener('pointerup', onUp, { passive: true })
  element.addEventListener('pointercancel', onLeave, { passive: true })
  write()

  return {
    destroy() {
      reset(true)
      if (frame) cancelAnimationFrame(frame)
      element.removeEventListener('pointerenter', onEnter)
      element.removeEventListener('pointermove', onMove)
      element.removeEventListener('pointerleave', onLeave)
      element.removeEventListener('pointerdown', onDown)
      element.removeEventListener('pointerup', onUp)
      element.removeEventListener('pointercancel', onLeave)
    },
  }
}

function syncController(element) {
  const controller = instances.get(element)
  if (depthEnabled()) {
    if (!controller) instances.set(element, createDepthController(element, registrations.get(element)))
  } else if (controller) {
    controller.destroy()
    instances.delete(element)
  }
}

function syncCapabilities() {
  registrations.forEach((_binding, element) => syncController(element))
}

function attachCapabilityListeners() {
  if (capabilityListenersAttached) return
  ensureCapabilityQueries()
  reducedMotion.addEventListener?.('change', syncCapabilities)
  finePointer.addEventListener?.('change', syncCapabilities)
  capabilityListenersAttached = true
}

function detachCapabilityListeners() {
  if (!capabilityListenersAttached || registrations.size) return
  reducedMotion.removeEventListener?.('change', syncCapabilities)
  finePointer.removeEventListener?.('change', syncCapabilities)
  capabilityListenersAttached = false
}

export const artworkDepthDirective = {
  mounted(element, binding) {
    registrations.set(element, binding)
    attachCapabilityListeners()
    syncController(element)
  },
  beforeUnmount(element) {
    instances.get(element)?.destroy()
    instances.delete(element)
    registrations.delete(element)
    detachCapabilityListeners()
  },
}
