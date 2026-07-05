<script setup>
// 工坊首页：两大功能入口 + 服务状态 + 使用须知。
import { RouterLink } from 'vue-router'
import { SosBadge, SosNotice } from '@haruhi/ui'
import { status } from '@/lib/store'

const features = [
  {
    to: '/tts',
    key: 'tts',
    title: '语音合成',
    en: 'Text to Speech',
    desc: '选择凉宫春日、阿虚、长门有希等 12 位角色与语气，输入台词，生成角色朗读语音。',
    icon: 'M12 3a3 3 0 0 0-3 3v6a3 3 0 0 0 6 0V6a3 3 0 0 0-3-3Zm-7 9a7 7 0 0 0 14 0M12 19v3',
  },
  {
    to: '/rvc',
    key: 'rvc',
    title: '声线转换',
    en: 'Voice Conversion',
    desc: '上传你的清唱或语音音频，转换成春日的声线；支持变调与咬字保护等参数。',
    icon: 'M4 12h2m3-5v10m3-14v18m3-14v10m3-7h2',
  },
]
</script>

<template>
  <div class="vo-page">
    <header class="vo-hero">
      <p class="sos-eyebrow">SOS 团 · 声音实验室</p>
      <h1 class="vo-hero__title">春日语音工坊</h1>
      <p class="vo-hero__sub">
        用 AI 复现《凉宫春日》角色的声音——文字变台词、人声换声线。
        推理跑在团员自己的机器上，在线时段即开即用。
      </p>
    </header>

    <div class="vo-features">
      <RouterLink v-for="f in features" :key="f.key" :to="f.to" class="vo-feature">
        <span class="vo-feature__icon" aria-hidden="true">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path :d="f.icon" />
          </svg>
        </span>
        <span class="vo-feature__body">
          <span class="vo-feature__head">
            <strong>{{ f.title }}</strong>
            <SosBadge v-if="status.known" :variant="(f.key === 'tts' ? status.ttsOnline : status.rvcOnline) ? 'signal' : 'outline'">
              {{ (f.key === 'tts' ? status.ttsOnline : status.rvcOnline) ? '在线' : '离线' }}
            </SosBadge>
          </span>
          <small class="vo-feature__en">{{ f.en }}</small>
          <span class="vo-feature__desc">{{ f.desc }}</span>
        </span>
        <span class="vo-feature__go" aria-hidden="true">→</span>
      </RouterLink>
    </div>

    <SosNotice title="使用须知" class="vo-home-notice">
      <ul class="vo-notice-list">
        <li>发起合成 / 转换需要登录应援团统一账号；浏览页面无需登录。</li>
        <li>算力来自团员本地机器，单任务串行、每人有短暂冷却，高峰期请排队。</li>
        <li>显示「离线」表示机器暂未开机，稍后再来即可，页面功能不受影响。</li>
        <li>生成的 AI 语音仅供爱好者交流，禁止用于冒充他人、伪造言论或任何商用。</li>
      </ul>
    </SosNotice>
  </div>
</template>
