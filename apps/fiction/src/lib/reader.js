// 阅读器偏好：字号/行距/字体/主题/栏宽，持久化到 localStorage，全站阅读页共享。
import { reactive, watch } from 'vue'

const KEY = 'fiction-reader'
const DEFAULTS = {
  fontSize: 19, // px
  lineHeight: 1.9,
  fontFamily: 'serif', // serif | sans
  theme: 'paper', // paper | sepia | green | dark
  width: 'normal', // narrow | normal | wide
}

export const THEMES = [
  { key: 'paper', label: '纸白', bg: '#f6f0e4', text: '#3b322a', panel: '#fffdf8' },
  { key: 'sepia', label: '米黄', bg: '#f2e8d5', text: '#5a4635', panel: '#fbf4e6' },
  { key: 'green', label: '护眼', bg: '#d3e8d3', text: '#2e3b2e', panel: '#e4f2e4' },
  { key: 'dark', label: '夜间', bg: '#191a1d', text: '#c8c5bd', panel: '#232428' },
]
const WIDTHS = { narrow: '32rem', normal: '40rem', wide: '48rem' }

function load() {
  try {
    return { ...DEFAULTS, ...JSON.parse(localStorage.getItem(KEY) || '{}') }
  } catch {
    return { ...DEFAULTS }
  }
}

const state = reactive(load())

watch(
  state,
  () => {
    try {
      localStorage.setItem(KEY, JSON.stringify(state))
    } catch {
      /* 隐私模式等场景忽略 */
    }
  },
  { deep: true },
)

export function useReaderSettings() {
  return state
}

export function themeOf(key) {
  return THEMES.find((t) => t.key === key) || THEMES[0]
}

export function widthOf(key) {
  return WIDTHS[key] || WIDTHS.normal
}
