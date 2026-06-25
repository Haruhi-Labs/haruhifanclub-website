<!-- src/views/FeedbackView.vue -->
<template>
  <div class="feedback-view">
    <main class="sos-page sos-page--contained sos-page--loose feedback-page">
      <!-- 标题区 -->
      <header class="sos-page-header">
        <div class="sos-page-header__content">
          <p class="sos-eyebrow">长门有希的书架</p>
          <h1 class="sos-page-header__title">同人投稿 &amp; 问题反馈</h1>
          <p class="sos-page-header__copy">
            欢迎向凉宫春日应援团提交
            <strong>同人作品、勘误建议、功能需求、Bug 反馈</strong>。
            表单当前托管在腾讯文档，数据仅用于社区交流与站点维护，未经许可不会公开。
          </p>
        </div>
        <router-link to="/" class="sos-button sos-button--secondary sos-button--sm">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
            <polyline points="15 18 9 12 15 6" />
          </svg>
          <span>返回书架</span>
        </router-link>
      </header>

      <!-- 同人投稿表单 -->
      <section class="feedback-section">
        <div class="feedback-section__head">
          <h2 class="sos-title feedback-section__title">同人投稿表单</h2>
          <a
            href="https://docs.qq.com/form/page/DWENKa0tUT1VzQk9H"
            target="_blank"
            rel="noopener"
            class="sos-button sos-button--link sos-button--sm"
          >
            在新窗口打开
          </a>
        </div>
        <p class="sos-copy sos-copy--small">
          希望你的同人作品被书架收录，可填写作品简介、署名等信息并上传作品。
          若内嵌表单无法加载，点击右上角“在新窗口打开”直接前往腾讯文档。
        </p>
        <div class="sos-surface sos-surface--subtle feedback-embed">
          <iframe
            src="https://docs.qq.com/form/page/DWENKa0tUT1VzQk9H?embedded=1"
            class="feedback-embed__frame"
            title="同人投稿表单"
            loading="lazy"
          ></iframe>
        </div>
      </section>

      <!-- 问题反馈表单 -->
      <section class="feedback-section">
        <div class="feedback-section__head">
          <h2 class="sos-title feedback-section__title">站点问题 &amp; 建议反馈</h2>
          <a
            href="https://docs.qq.com/form/page/DWHppYktEV0JVc2dT"
            target="_blank"
            rel="noopener"
            class="sos-button sos-button--link sos-button--sm"
          >
            在新窗口打开
          </a>
        </div>
        <p class="sos-copy sos-copy--small">
          阅读中遇到
          <strong>错字漏字、章节缺失、排版问题、功能 Bug</strong>，或有新功能建议，
          都可以通过这个表单告诉我们。可选填联系方式，便于必要时确认细节。
        </p>
        <div class="sos-surface sos-surface--subtle feedback-embed">
          <iframe
            src="https://docs.qq.com/form/page/DWHppYktEV0JVc2dT?embedded=1"
            class="feedback-embed__frame"
            title="站点问题与建议反馈表单"
            loading="lazy"
          ></iframe>
        </div>
      </section>

      <!-- 说明 -->
      <aside class="sos-notice sos-notice--info">
        <span class="sos-notice__icon" aria-hidden="true">
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="9" />
            <path d="M12 11v5M12 8h.01" stroke-linecap="round" />
          </svg>
        </span>
        <div>
          <p class="sos-notice__title">不习惯腾讯文档？</p>
          <p class="sos-notice__copy">
            也可以通过页脚的社交平台或“加入我们”群聊联系管理员；未来会增加本站内置的反馈表单。
          </p>
        </div>
      </aside>
    </main>
  </div>
</template>

<script setup>
// 当前页面仅为嵌入文档与文案展示，不需要额外脚本逻辑
</script>

<style scoped>
.feedback-page {
  padding-block: var(--sos-space-10) var(--sos-space-16);
}

.feedback-section {
  display: grid;
  gap: var(--sos-space-3);
}
.feedback-section__head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--sos-space-4);
}
.feedback-section__title {
  font-size: var(--sos-text-xl);
}

/* 表单嵌入容器：奶油纸承载面，iframe 圆角收口 */
.feedback-embed {
  margin-top: var(--sos-space-1);
  padding: 0;
  overflow: hidden;
  border-radius: var(--sos-radius-lg);
}
.feedback-embed__frame {
  display: block;
  width: 100%;
  height: 640px;
  border: 0;
  background: transparent;
}

@media (max-width: 640px) {
  .feedback-embed__frame {
    height: 540px;
  }
}
</style>
