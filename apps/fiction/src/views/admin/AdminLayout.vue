<script setup>
import { computed } from 'vue'
import { RouterView, RouterLink, useRoute } from 'vue-router'
import { session } from '@/api'
import { fictionRoleLevel } from '@/lib/admin'

const route = useRoute()
const logoSrc = `${import.meta.env.BASE_URL}haruhi-logo-192.png`
const user = computed(() => session.state.user)
const roleLabel = computed(() => {
  const u = user.value
  if (!u) return ''
  if (u.isSuperAdmin) return '超级管理员'
  return u.apps?.fiction?.roleName || (fictionRoleLevel(u) >= 4 ? '管理' : '审核')
})

const nav = [
  { name: 'admin', to: '/admin', label: '总览', desc: '数据一览' },
  { name: 'admin-works', to: '/admin/works', label: '作品管理', desc: '精选 · 上下架 · 删除' },
  { name: 'admin-comments', to: '/admin/comments', label: '评论管理', desc: '隐藏 · 恢复' },
]
const activeName = computed(() => route.name)
</script>

<template>
  <div class="fadmin">
    <aside class="fadmin__side">
      <div class="fadmin__brand">
        <img :src="logoSrc" alt="" class="fadmin__logo" />
        <div class="fadmin__brand-text">
          <strong>文库后台</strong>
          <small>春日文库 · 管理控制台</small>
        </div>
      </div>

      <nav class="fadmin__nav">
        <RouterLink
          v-for="n in nav"
          :key="n.name"
          :to="n.to"
          class="fadmin__navitem"
          :class="{ 'is-active': activeName === n.name }"
        >
          <span class="fadmin__navlabel">{{ n.label }}</span>
          <span class="fadmin__navdesc">{{ n.desc }}</span>
        </RouterLink>
      </nav>

      <div class="fadmin__side-foot">
        <div class="fadmin__who">
          <span class="fadmin__who-name">{{ user?.nickname || user?.username || '管理员' }}</span>
          <span class="fadmin__who-role">{{ roleLabel }}</span>
        </div>
        <RouterLink to="/" class="fadmin__exit">← 返回站点</RouterLink>
      </div>
    </aside>

    <main class="fadmin__main">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.fadmin {
  display: grid;
  grid-template-columns: 248px 1fr;
  align-content: start; /* 防止 grid 行被 min-height 拉伸（移动端会在顶栏与内容间留空隙） */
  min-height: 100vh;
  background: var(--sos-bg-page);
}

/* ---- 侧边栏 ---- */
.fadmin__side {
  position: sticky;
  top: 0;
  align-self: start;
  height: 100vh;
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-5);
  padding: var(--sos-space-6) var(--sos-space-4);
  background: var(--sos-bg-surface);
  border-right: 1px solid var(--sos-border-subtle);
}
.fadmin__brand {
  display: flex;
  align-items: center;
  gap: var(--sos-space-3);
  padding: 0 var(--sos-space-2);
}
.fadmin__logo {
  width: 40px;
  height: 40px;
  border-radius: var(--sos-radius-md);
  flex: none;
}
.fadmin__brand-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
}
.fadmin__brand-text strong {
  font-family: var(--sos-display-family, var(--sos-font-display));
  font-size: var(--sos-text-md);
  color: var(--sos-text-primary);
}
.fadmin__brand-text small {
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
}
.fadmin__nav {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-2);
  flex: 1;
}
.fadmin__navitem {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: var(--sos-space-3) var(--sos-space-3);
  border-radius: var(--sos-radius-md);
  text-decoration: none;
  color: var(--sos-text-secondary);
  border: 1px solid transparent;
  transition: background 0.15s ease, color 0.15s ease, border-color 0.15s ease;
}
.fadmin__navitem:hover {
  background: var(--sos-bg-page);
  color: var(--sos-text-primary);
}
.fadmin__navitem.is-active {
  background: var(--sos-accent-soft, color-mix(in srgb, var(--sos-accent) 12%, transparent));
  border-color: color-mix(in srgb, var(--sos-accent) 30%, transparent);
  color: var(--sos-accent);
}
.fadmin__navlabel {
  font-size: var(--sos-text-sm);
  font-weight: 600;
}
.fadmin__navdesc {
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
}
.fadmin__navitem.is-active .fadmin__navdesc {
  color: color-mix(in srgb, var(--sos-accent) 70%, var(--sos-text-tertiary));
}
.fadmin__side-foot {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-3);
  padding-top: var(--sos-space-4);
  border-top: 1px solid var(--sos-border-subtle);
}
.fadmin__who {
  display: flex;
  flex-direction: column;
  padding: 0 var(--sos-space-2);
}
.fadmin__who-name {
  font-size: var(--sos-text-sm);
  color: var(--sos-text-primary);
  font-weight: 600;
}
.fadmin__who-role {
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
}
.fadmin__exit {
  font-size: var(--sos-text-sm);
  color: var(--sos-text-secondary);
  text-decoration: none;
  padding: var(--sos-space-2) var(--sos-space-2);
  border-radius: var(--sos-radius-md);
}
.fadmin__exit:hover {
  color: var(--sos-accent);
  background: var(--sos-bg-page);
}

/* ---- 内容区 ---- */
.fadmin__main {
  min-width: 0;
  padding: var(--sos-space-7) var(--sos-space-7) var(--sos-space-10);
}

/* ---- 移动端：侧栏变顶部横向导航 ---- */
@media (max-width: 880px) {
  .fadmin {
    grid-template-columns: 1fr;
  }
  .fadmin__side {
    position: sticky;
    top: 0;
    height: auto;
    z-index: 10;
    flex-direction: row;
    align-items: center;
    gap: var(--sos-space-3);
    padding: var(--sos-space-3) var(--sos-space-4);
    overflow-x: auto;
  }
  .fadmin__brand {
    flex: none;
  }
  .fadmin__brand-text {
    display: none;
  }
  .fadmin__nav {
    flex-direction: row;
    flex: 1;
    gap: var(--sos-space-2);
  }
  .fadmin__navitem {
    flex: none;
    padding: var(--sos-space-2) var(--sos-space-3);
  }
  .fadmin__navdesc {
    display: none;
  }
  .fadmin__side-foot {
    flex: none;
    flex-direction: row;
    align-items: center;
    gap: var(--sos-space-3);
    padding-top: 0;
    border-top: none;
  }
  .fadmin__who {
    display: none;
  }
  .fadmin__main {
    padding: var(--sos-space-5) var(--sos-space-4) var(--sos-space-9);
  }
}
</style>
