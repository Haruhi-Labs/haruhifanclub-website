<template>
  <section class="container-card art-board">
    <header class="board-header">
      <p class="eyebrow">Club Board</p>
      <h1>公告</h1>
      <p>本页暂时使用本地公告数据，用于后续接入活动和站内通知。</p>
    </header>

    <div class="notice-list">
      <article v-for="notice in notices" :key="notice.id" class="notice-card panel">
        <div class="notice-date">
          <span>{{ notice.month }}</span>
          <b>{{ notice.day }}</b>
        </div>
        <div class="notice-body">
          <div class="notice-meta">{{ notice.type }}</div>
          <h2>{{ notice.title }}</h2>
          <p>{{ notice.body }}</p>
          <div class="notice-tags">
            <span v-for="tag in notice.tags" :key="tag">#{{ tag }}</span>
          </div>
        </div>
      </article>
    </div>
  </section>
</template>

<script setup>
const notices = [
  {
    id: 1,
    month: '06',
    day: '23',
    type: '画廊维护',
    title: '画廊导航与夜间模式试运行',
    body: '新增功能入口后，画廊将继续以本地 mock 数据辅助开发调试。',
    tags: ['开发中', '夜间模式']
  },
  {
    id: 2,
    month: '06',
    day: '20',
    type: '投稿说明',
    title: '投稿请确认作品来源和授权',
    body: '个人作品建议填写创作者唯一 ID，转载作品请保留来源说明。',
    tags: ['投稿', '授权']
  },
  {
    id: 3,
    month: '06',
    day: '18',
    type: '积分规则',
    title: '积分兑换功能准备中',
    body: '兑换页目前展示本地 mock 项目，真实兑换流程会在后续接入。',
    tags: ['积分', '兑换']
  }
]
</script>

<style scoped>
.art-board {
  max-width: 960px;
}

.board-header {
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

.board-header h1 {
  margin: 0;
  color: var(--text);
  font-size: clamp(32px, 5vw, 48px);
  font-weight: 950;
}

.board-header p {
  margin: 12px 0 0;
  color: var(--muted);
  font-weight: 700;
}

.notice-list {
  display: grid;
  gap: 16px;
}

.notice-card {
  display: grid;
  grid-template-columns: 82px 1fr;
  gap: 20px;
  padding: 22px;
}

.notice-date {
  height: 82px;
  border-radius: 20px;
  display: grid;
  place-items: center;
  align-content: center;
  background: linear-gradient(135deg, var(--accent-soft), rgba(255, 255, 255, 0.75));
  color: var(--accent);
  box-shadow: var(--shadow-in);
}

.notice-date span {
  font-size: 13px;
  font-weight: 950;
}

.notice-date b {
  font-size: 28px;
  line-height: 1;
}

.notice-meta {
  color: var(--accent);
  font-size: 12px;
  font-weight: 950;
}

.notice-body h2 {
  margin: 6px 0 8px;
  color: var(--text);
  font-size: 22px;
  font-weight: 950;
}

.notice-body p {
  margin: 0;
  color: var(--muted);
  line-height: 1.7;
  font-weight: 700;
}

.notice-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 14px;
}

.notice-tags span {
  padding: 5px 10px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.72);
  color: var(--muted);
  font-size: 12px;
  font-weight: 850;
}

@media (max-width: 640px) {
  .notice-card {
    grid-template-columns: 1fr;
  }

  .notice-date {
    width: 82px;
  }
}
</style>
