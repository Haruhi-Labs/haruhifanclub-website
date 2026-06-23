<template>
  <section class="container-card exchange-page">
    <header class="exchange-header">
      <p class="eyebrow">Point Exchange</p>
      <h1>兑换</h1>
      <p>本页先展示本地 mock 兑换项目，后续再接入真实兑换流程。</p>
    </header>

    <div class="exchange-grid">
      <article v-for="item in exchangeItems" :key="item.id" class="exchange-card panel">
        <div class="item-art" :class="item.tone">
          <span>{{ item.icon }}</span>
        </div>
        <div class="item-body">
          <div class="item-points">{{ item.points }} pts</div>
          <h2>{{ item.title }}</h2>
          <p>{{ item.desc }}</p>
          <button type="button" class="mock-btn" disabled>暂未开放</button>
        </div>
      </article>
    </div>
  </section>
</template>

<script setup>
const exchangeItems = [
  {
    id: 1,
    icon: '🎫',
    tone: 'cyan',
    points: 30,
    title: '活动优先登记券',
    desc: '用于未来绘画部活动的优先登记占位。'
  },
  {
    id: 2,
    icon: '🖼️',
    tone: 'pink',
    points: 60,
    title: '作品展示位',
    desc: '兑换首页功能卡或公告页展示推荐位。'
  },
  {
    id: 3,
    icon: '🎁',
    tone: 'gold',
    points: 100,
    title: '社团纪念小礼',
    desc: '预留给后续线下或邮寄类兑换项目。'
  }
]
</script>

<style scoped>
.exchange-page {
  max-width: 1080px;
}

.exchange-header {
  margin-bottom: 24px;
  text-align: center;
}

.eyebrow {
  margin: 0 0 8px;
  color: var(--accent);
  font-size: 12px;
  font-weight: 950;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.exchange-header h1 {
  margin: 0;
  color: var(--text);
  font-size: clamp(32px, 5vw, 48px);
  font-weight: 950;
}

.exchange-header p {
  margin: 12px 0 0;
  color: var(--muted);
  font-weight: 700;
}

.exchange-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 18px;
}

.exchange-card {
  overflow: hidden;
}

.item-art {
  min-height: 150px;
  display: grid;
  place-items: center;
  background: linear-gradient(135deg, #d9f99d, #bae6fd);
}

.item-art.pink {
  background: linear-gradient(135deg, #fbcfe8, #ddd6fe);
}

.item-art.gold {
  background: linear-gradient(135deg, #fde68a, #fed7aa);
}

.item-art span {
  font-size: 54px;
  filter: drop-shadow(0 8px 18px rgba(0, 0, 0, 0.16));
}

.item-body {
  padding: 20px;
}

.item-points {
  display: inline-flex;
  padding: 5px 10px;
  border-radius: 999px;
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 12px;
  font-weight: 950;
}

.item-body h2 {
  margin: 12px 0 8px;
  color: var(--text);
  font-size: 22px;
  font-weight: 950;
}

.item-body p {
  min-height: 52px;
  margin: 0 0 18px;
  color: var(--muted);
  line-height: 1.6;
  font-weight: 700;
}

.mock-btn {
  width: 100%;
  border: 1px solid var(--line);
  border-radius: 14px;
  padding: 10px 14px;
  background: rgba(255, 255, 255, 0.7);
  color: var(--muted);
  font-weight: 900;
  cursor: not-allowed;
}

@media (max-width: 820px) {
  .exchange-grid {
    grid-template-columns: 1fr;
  }
}
</style>
