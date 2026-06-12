// src/utils/uiSound.js

import { UI } from '../config/ui.js'

let _ctx = null
let _lastAt = 0

// 可选音效文件取自 UI.sfx.clickUrl（见 config/ui.js）；留空则直接用 WebAudio
// 合成点击声，不再发起注定 404 的文件请求（仓库从未携带 /sfx/click.mp3）。
let _audio = null
let _audioBroken = false
const CLICK_URL = String(UI?.sfx?.clickUrl || '').trim()
const VOLUME = Number(UI?.sfx?.volume ?? 0.35)

function nowMs(){ return Date.now() }

function ensureAudio(){
  if(_audio || _audioBroken) return
  if(!CLICK_URL){
    // 未配置音效文件：标记不可用，直接走合成声兜底
    _audioBroken = true
    return
  }
  try{
    _audio = new Audio(CLICK_URL)
    _audio.preload = 'auto'
    _audio.volume = VOLUME
  }catch{
    _audioBroken = true
    _audio = null
  }
}

function ensureCtx(){
  if(_ctx) return _ctx
  try{
    _ctx = new (window.AudioContext || window.webkitAudioContext)()
  }catch{
    _ctx = null
  }
  return _ctx
}

function beepFallback(){
  const ctx = ensureCtx()
  if(!ctx) return

  // iOS/Safari 需要用户手势后 resume
  if(ctx.state === 'suspended'){
    ctx.resume().catch(() => {})
  }

  const t0 = ctx.currentTime
  const osc = ctx.createOscillator()
  const gain = ctx.createGain()

  osc.type = 'triangle'
  osc.frequency.setValueAtTime(520, t0)
  osc.frequency.exponentialRampToValueAtTime(420, t0 + 0.04)

  gain.gain.setValueAtTime(0.0001, t0)
  gain.gain.exponentialRampToValueAtTime(0.12, t0 + 0.01)
  gain.gain.exponentialRampToValueAtTime(0.0001, t0 + 0.06)

  osc.connect(gain)
  gain.connect(ctx.destination)

  osc.start(t0)
  osc.stop(t0 + 0.07)
}

async function tryPlayAudio(){
  ensureAudio()
  if(!_audio || _audioBroken) return false
  try{
    _audio.currentTime = 0
    await _audio.play()
    return true
  }catch{
    // 避免反复 404/权限错误刷屏：失败一次就认定不可用
    _audioBroken = true
    return false
  }
}

// ✅ 你 App.vue 里用的是这个
export function playUiClick(){
  const t = nowMs()
  // 节流：避免连点爆音
  if(t - _lastAt < 35) return
  _lastAt = t

  // 先试播放文件音效，失败就用 WebAudio 兜底
  tryPlayAudio().then((ok) => {
    if(!ok) beepFallback()
  })
}

// ✅ 你很多组件在用这个（FilterPanel/Header等），必须提供同名导出
export function playClick(){
  playUiClick()
}

// 预留：以后你想在设置里让用户换音效，可用这个
export function setUiClickSoundUrl(url){
  try{
    const u = String(url || '').trim()
    if(!u){
      _audio = null
      _audioBroken = true
      return
    }
    _audioBroken = false
    _audio = new Audio(u)
    _audio.preload = 'auto'
    _audio.volume = 0.35
  }catch{
    _audioBroken = true
    _audio = null
  }
}
