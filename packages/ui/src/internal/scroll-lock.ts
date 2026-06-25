// 跨弹窗共享的滚动锁引用计数：多个弹窗并存时，关一个不会误解锁其它仍打开的弹窗。
let lockCount = 0

export function lockScroll(): void {
  if (typeof document === 'undefined') return
  if (lockCount === 0) document.body.style.overflow = 'hidden'
  lockCount += 1
}

export function unlockScroll(): void {
  if (typeof document === 'undefined') return
  lockCount = Math.max(0, lockCount - 1)
  if (lockCount === 0) document.body.style.overflow = ''
}
