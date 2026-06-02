import { playUiClick } from './uiSound.js'

/**
 * 兼容旧代码：GalleryView.vue 里 import { playClick } from '../utils/sound'
 * 我们直接把它映射到新的播放逻辑。
 */
export function playClick(){
  playUiClick()
}

// 如果你旧代码里还有别的名字，也顺手兼容
export function playTap(){
  playUiClick()
}
export function playSelect(){
  playUiClick()
}
