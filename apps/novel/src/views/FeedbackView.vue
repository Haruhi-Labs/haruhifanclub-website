<!-- src/views/FeedbackView.vue -->
<template>
  <div class="min-h-screen bg-[#FAF9DE] text-[#4A3B32]">
    <main class="max-w-5xl mx-auto px-4 md:px-8 py-8 md:py-12">
      <!-- 标题区 -->
      <section class="mb-8">
        <div class="mb-3 flex items-center justify-between gap-4">
          <h1
            class="text-2xl md:text-3xl font-bold tracking-wide text-[#4A3B32]"
          >
            同人投稿 &amp; 问题反馈
          </h1>

          <!-- 返回首页按钮 -->
          <router-link
            to="/"
            class="inline-flex items-center gap-1 rounded-full border border-[#D7CBBE] bg-white/70 px-3 py-1.5 text-xs md:text-sm text-[#6B5D52] shadow-sm hover:bg-white hover:border-[#C4B6A4] hover:text-[#C26B57] transition"
          >
            <svg
              class="h-3.5 w-3.5"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <polyline points="15 18 9 12 15 6" />
            </svg>
            <span>返回首页</span>
          </router-link>
        </div>

        <p class="text-sm md:text-base text-[#7A6C60] leading-relaxed">
          欢迎你向凉宫春日应援团提交
          <span class="font-semibold">同人作品、勘误建议、功能需求、Bug 反馈</span>
          等信息。
          当前表单托管在腾讯文档中，数据仅用于社区交流与站点维护，未经许可不会公开。
        </p>
      </section>

      <!-- 同人投稿表单 -->
      <section class="mb-10 space-y-3">
        <div class="flex items-baseline justify-between gap-4">
          <h2 class="text-lg font-semibold text-[#4A3B32]">同人投稿表单</h2>
          <a
            href="https://docs.qq.com/form/page/DWENKa0tUT1VzQk9H"
            target="_blank"
            rel="noopener"
            class="text-xs text-[#C26B57] hover:text-[#B4543E] underline underline-offset-2"
          >
            在新窗口打开
          </a>
        </div>
        <p class="text-xs md:text-sm text-[#9B8F80]">
          如果你希望你的同人作品被书架收录，你可以填写作品简介、署名等信息，并上传你的作品。
          如果内嵌表单无法加载，可以点击右上角“在新窗口打开”直接前往腾讯文档。
        </p>

        <div
          class="mt-2 w-full rounded-2xl bg-[#F2EFE4] shadow-sm ring-1 ring-black/[0.03] overflow-hidden"
          style="min-height: 480px;"
        >
          <!-- 同人投稿表单（嵌入） -->
          <iframe
            src="https://docs.qq.com/form/page/DWENKa0tUT1VzQk9H?embedded=1"
            class="w-full h-[520px] md:h-[640px] border-0"
            loading="lazy"
          ></iframe>
        </div>
      </section>

      <!-- 问题反馈表单 -->
      <section class="mb-10 space-y-3">
        <div class="flex items-baseline justify-between gap-4">
          <h2 class="text-lg font-semibold text-[#4A3B32]">
            站点问题 &amp; 建议反馈
          </h2>
          <a
            href="https://docs.qq.com/form/page/DWHppYktEV0JVc2dT"
            target="_blank"
            rel="noopener"
            class="text-xs text-[#C26B57] hover:text-[#B4543E] underline underline-offset-2"
          >
            在新窗口打开
          </a>
        </div>
        <p class="text-xs md:text-sm text-[#9B8F80]">
          如果你在阅读过程中遇到
          <span class="font-medium">错字漏字、章节缺失、排版问题、功能 Bug</span>
          等情况，或有新的功能建议，都可以通过这个表单告诉我们。
          可选填写联系方式，以便必要时向你确认细节。
        </p>

        <div
          class="mt-2 w-full rounded-2xl bg-[#F2EFE4] shadow-sm ring-1 ring-black/[0.03] overflow-hidden"
          style="min-height: 480px;"
        >
          <!-- 问题反馈表单（嵌入） -->
          <iframe
            src="https://docs.qq.com/form/page/DWHppYktEV0JVc2dT?embedded=1"
            class="w-full h-[520px] md:h-[640px] border-0"
            loading="lazy"
          ></iframe>
        </div>
      </section>

      <!-- 说明 -->
      <section
        class="mt-6 rounded-xl bg-white/70 px-4 py-3 text-xs md:text-[13px] text-[#9B8F80] border border-[#E6DFD0]"
      >
        <p>
          提示：如果你不习惯使用腾讯文档，也可以通过页脚中的社交平台或“加入我们”群聊联系管理员；
          未来也可能会增加本站内置的反馈表单。
        </p>
      </section>
    </main>
  </div>
</template>

<script setup>
// 当前页面仅为嵌入文档与文案展示，不需要额外脚本逻辑
</script>
