<template>
  <section class="container-card art-home">
    <div class="home-hero panel">
      <div class="hero-copy">
        <p class="eyebrow">SOS Brigade Art Club</p>
        <h1>应援团绘画部</h1>
        <p class="hero-text">
          这里是画廊首页。顶部导航已经接入画廊、投稿、积分、公告和兑换等功能入口。
        </p>
      </div>
      <div class="hero-mark" aria-hidden="true">
        <span>画</span>
      </div>
    </div>
  </section>
</template>

<style scoped>
.art-home {
  max-width: 1120px;
}

.home-hero {
  position: relative;
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 24px;
  align-items: center;
  padding: 34px;
  overflow: hidden;
}

.home-hero::before {
  content: "";
  position: absolute;
  inset: 0;
  background:
    radial-gradient(circle at 12% 18%, rgba(20, 184, 166, 0.16), transparent 32%),
    radial-gradient(circle at 88% 20%, rgba(236, 72, 153, 0.14), transparent 30%);
  pointer-events: none;
}

.hero-copy,
.hero-mark {
  position: relative;
}

.eyebrow {
  margin: 0 0 8px;
  color: var(--accent);
  font-size: 12px;
  font-weight: 950;
  letter-spacing: 0;
  text-transform: uppercase;
}

.home-hero h1 {
  margin: 0;
  color: var(--text);
  font-size: clamp(32px, 5vw, 56px);
  line-height: 1;
  font-weight: 950;
}

.hero-text {
  max-width: 620px;
  margin: 16px 0 0;
  color: var(--muted);
  font-size: 16px;
  line-height: 1.7;
  font-weight: 700;
}

.hero-mark {
  width: 128px;
  height: 128px;
  border-radius: 32px;
  display: grid;
  place-items: center;
  background: linear-gradient(135deg, #fef3c7, #d9f99d 48%, #bae6fd);
  box-shadow: var(--shadow-2), inset 0 1px 0 rgba(255, 255, 255, 0.85);
  transform: rotate(5deg);
}

.hero-mark span {
  color: #155e75;
  font-size: 54px;
  font-weight: 950;
}

@media (max-width: 640px) {
  .home-hero {
    grid-template-columns: 1fr;
    padding: 24px;
  }

  .hero-mark {
    display: none;
  }
}
</style>
