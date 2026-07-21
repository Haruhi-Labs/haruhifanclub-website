let burstSequence = 0

/** 从点赞按钮当前位置生成轻量爱心反馈，只动画 transform/opacity。 */
export function launchLikeHeart(origin) {
  if (!origin || typeof document === 'undefined') return

  const rect = origin.getBoundingClientRect()
  const heart = document.createElement('span')
  const reducedMotion = window.matchMedia?.('(prefers-reduced-motion: reduce)').matches
  const sequence = burstSequence++
  const drift = ((sequence * 17) % 25) - 12
  const rotation = ((sequence * 13) % 17) - 8

  heart.textContent = '♥'
  heart.setAttribute('aria-hidden', 'true')
  Object.assign(heart.style, {
    position: 'fixed',
    left: `${rect.left + rect.width / 2}px`,
    top: `${rect.top + rect.height * 0.28}px`,
    zIndex: '4000',
    color: '#d44957',
    font: '900 21px/1 system-ui, sans-serif',
    pointerEvents: 'none',
    textShadow: '0 4px 12px rgba(133, 36, 52, 0.28)',
    transformOrigin: '50% 50%',
    willChange: 'transform, opacity',
  })
  document.body.appendChild(heart)

  if (typeof heart.animate !== 'function') {
    heart.remove()
    return
  }

  const frames = reducedMotion
    ? [
        { opacity: 0, transform: 'translate3d(-50%, 0, 0) scale(0.9)' },
        { opacity: 1, transform: 'translate3d(-50%, -3px, 0) scale(1)' },
        { opacity: 0, transform: 'translate3d(-50%, -6px, 0) scale(1)' },
      ]
    : [
        { opacity: 0, transform: 'translate3d(-50%, 3px, 0) scale(0.72)' },
        { offset: 0.16, opacity: 1, transform: 'translate3d(-50%, -5px, 0) scale(1)' },
        {
          opacity: 0,
          transform: `translate3d(calc(-50% + ${drift}px), -62px, 0) rotate(${rotation}deg) scale(1.12)`,
        },
      ]
  const animation = heart.animate(frames, {
    duration: reducedMotion ? 180 : 720,
    easing: 'cubic-bezier(0.22, 1, 0.36, 1)',
    fill: 'forwards',
  })
  animation.finished.finally(() => heart.remove())
}
