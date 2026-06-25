<template>
  <div class="admin-view">
    <main class="sos-page sos-page--contained admin-page">
      <header class="admin-head">
        <p class="sos-eyebrow">长门有希的书架</p>
        <h1 class="sos-title admin-title">藏书阁管理</h1>
      </header>

      <!-- 登录框 -->
      <section
        v-if="!isLoggedIn"
        class="sos-surface sos-surface--padded admin-login"
      >
        <h2 class="admin-card-title">管理员登录</h2>
        <div class="sos-field">
          <label class="sos-field__label" for="admin-username">用户名</label>
          <input
            id="admin-username"
            v-model="loginForm.username"
            type="text"
            class="sos-input"
            placeholder="用户名"
            autocomplete="username"
            @keyup.enter="login"
          />
        </div>
        <div class="sos-field">
          <label class="sos-field__label" for="admin-password">密码</label>
          <input
            id="admin-password"
            v-model="loginForm.password"
            type="password"
            class="sos-input"
            placeholder="密码"
            autocomplete="current-password"
            @keyup.enter="login"
          />
        </div>
        <button
          type="button"
          class="sos-button sos-button--primary sos-button--block"
          @click="login"
        >
          进入
        </button>
        <p v-if="loginError" class="admin-msg is-error" role="alert">{{ loginError }}</p>
      </section>

      <!-- 管理面板 -->
      <div v-else class="admin-panel">
        <!-- 上传区 -->
        <section class="sos-surface sos-surface--padded">
          <h2 class="admin-card-title">上传新书（EPUB）</h2>
          <div class="admin-upload">
            <input
              ref="fileInput"
              type="file"
              accept=".epub"
              class="admin-file"
            />
            <button
              type="button"
              class="sos-button sos-button--primary"
              :disabled="uploading"
              :aria-busy="uploading"
              @click="uploadBook"
            >
              {{ uploading ? '解析中…' : '上传' }}
            </button>
          </div>
          <p
            v-if="message"
            class="admin-msg"
            :class="success ? 'is-success' : 'is-error'"
            role="status"
          >
            {{ message }}
          </p>
        </section>

        <!-- 分栏说明 -->
        <aside class="sos-surface sos-surface--subtle admin-note">
          <p class="admin-note__title">分栏 &amp; 排序说明</p>
          <p>
            「分栏」会影响书架上的分组展示，例如：
            <code class="admin-code">正传小说 / 设定集 / 社区同人</code>。
          </p>
          <p>
            「排序值」为数字，<strong>数值越小越靠前</strong>。
            默认按导入时间生成，可在这里自由调整。
          </p>
        </aside>

        <!-- 书籍列表 -->
        <section class="sos-surface admin-list">
          <header class="admin-list__head">
            <span class="admin-list__title">已发布书籍</span>
            <span class="sos-badge">共 {{ books.length }} 本</span>
          </header>

          <ul class="admin-books">
            <li v-for="book in books" :key="book.id" class="admin-book">
              <!-- 左边：封面 + 可编辑信息 -->
              <div class="admin-book__main">
                <div class="admin-book__cover">
                  <img
                    v-if="book.cover_path"
                    :src="getCoverUrl(book.cover_path)"
                    :alt="book.title"
                  />
                </div>

                <div class="admin-book__fields">
                  <input
                    v-model="book.title"
                    class="admin-edit admin-edit--title"
                    placeholder="书名"
                    aria-label="书名"
                  />
                  <input
                    v-model="book.author"
                    class="admin-edit admin-edit--author"
                    placeholder="作者"
                    aria-label="作者"
                  />

                  <div class="admin-book__meta">
                    <label class="admin-inline">
                      <span class="admin-inline__label">分栏</span>
                      <select v-model="book.category" class="sos-select admin-inline__select">
                        <option :value="null">自动归类（默认）</option>
                        <option
                          v-for="cat in CATEGORY_OPTIONS"
                          :key="cat.key"
                          :value="cat.key"
                        >
                          {{ cat.label }}
                        </option>
                      </select>
                    </label>

                    <label class="admin-inline">
                      <span class="admin-inline__label">排序值</span>
                      <input
                        v-model.number="book.order"
                        type="number"
                        class="sos-input admin-inline__order"
                        placeholder="越小越靠前"
                      />
                    </label>

                    <span v-if="book.category" class="sos-badge sos-badge--outline">
                      {{ getCategoryLabel(book.category) }}
                    </span>
                  </div>
                </div>
              </div>

              <!-- 右边：操作 -->
              <div class="admin-book__actions">
                <button
                  type="button"
                  class="sos-button sos-button--secondary sos-button--sm"
                  @click="saveBook(book)"
                >
                  保存
                </button>
                <button
                  type="button"
                  class="sos-button sos-button--ghost sos-button--sm admin-delete"
                  @click="deleteBook(book.id)"
                >
                  删除
                </button>
              </div>
            </li>
          </ul>

          <div v-if="!books.length" class="admin-empty">
            当前还没有书籍，请先上传 EPUB 文件。
          </div>
        </section>
      </div>
    </main>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue';
import { createApiClient, createAdminAuth, resolveUploadUrl } from '@haruhi/api-client';

// 统一后端：模块 API 走 /api/novel，鉴权走共享的 createAdminAuth（已内置 novel 权限校验），静态文件走 /uploads
const api = createApiClient('/api/novel');
const admin = createAdminAuth('novel');

const isLoggedIn = ref(false);
const loginForm = reactive({ username: '', password: '' });
const loginError = ref('');
const books = ref([]);
const fileInput = ref(null);
const uploading = ref(false);
const message = ref('');
const success = ref(false);

// 分栏选项（要和书架那边保持一致）
const CATEGORY_OPTIONS = [
  { key: 'main', label: '正传小说' },
  { key: 'setting', label: '设定集' },
  { key: 'short', label: '官方短篇' },
  { key: 'fanfic', label: '社区同人' },
];

const getCategoryLabel = (key) => {
  const found = CATEGORY_OPTIONS.find((c) => c.key === key);
  return found ? found.label : key;
};

const login = async () => {
  loginError.value = '';
  // 共享鉴权：永不抛错，已内置 novel 权限校验，失败返回 { ok:false, error }
  const r = await admin.login(loginForm.username.trim(), loginForm.password);
  if (!r.ok) {
    loginError.value = r.error;
    return;
  }
  isLoggedIn.value = true;
  await refreshBooks();
};

const refreshBooks = async () => {
  try {
    books.value = (await api.get('/books')) || [];
  } catch (e) {
    console.error('获取书单失败', e);
  }
};

const getCoverUrl = (path) => resolveUploadUrl(path);

const uploadBook = async () => {
  const file = fileInput.value?.files?.[0];
  if (!file) return;

  uploading.value = true;
  message.value = '';

  const formData = new FormData();
  formData.append('file', file);

  try {
    await api.postForm('/admin/upload', formData);
    message.value = '上传成功！自动提取封面和标题完成。';
    success.value = true;
    fileInput.value.value = '';
    await refreshBooks();
  } catch (e) {
    message.value = '上传失败：' + (e.data?.error || e.message);
    success.value = false;
    if (e.status === 401) alert('登录已过期，请重新登录');
  } finally {
    uploading.value = false;
  }
};

const deleteBook = async (id) => {
  if (!confirm('确定删除吗？')) return;
  try {
    await api.del('/admin/books/' + id);
    await refreshBooks();
  } catch (e) {
    alert('删除失败：' + (e.data?.error || e.message));
  }
};

// 保存当前这一条书籍的信息（标题 / 作者 / 分栏 / 排序）
const saveBook = async (book) => {
  try {
    const payload = {
      title: book.title,
      author: book.author,
      category: book.category ?? null,
    };

    if (book.order !== undefined && book.order !== null && book.order !== '') {
      payload.order = Number(book.order);
    }

    const res = await api.patch('/admin/books/' + book.id, payload);

    const savedTitle = res?.book?.title || book.title;
    message.value = `《${savedTitle}》已保存`;
    success.value = true;

    // 关键：重新拉一次列表，让前端列表按最新排序值刷新
    await refreshBooks();
  } catch (e) {
    console.error('保存书籍信息失败', e);
    message.value = '保存失败：' + (e.data?.error || e.message);
    success.value = false;
  }
};

onMounted(async () => {
  // 共享鉴权：有有效会话且具备 novel 权限则返回 user，否则已内部登出并返回 null
  const user = await admin.restore();
  if (user) {
    isLoggedIn.value = true;
    await refreshBooks();
  }
});
</script>

<style scoped>
.admin-page {
  padding-block: var(--sos-space-12) var(--sos-space-16);
}
.admin-head {
  display: grid;
  justify-items: center;
  gap: var(--sos-space-2);
  text-align: center;
}
.admin-title {
  font-size: var(--sos-text-3xl);
}
.admin-card-title {
  margin: 0 0 var(--sos-space-4);
  font-family: var(--sos-display-family);
  font-size: var(--sos-text-lg);
  font-weight: var(--sos-weight-heavy);
  color: var(--sos-text-primary);
}

/* 登录卡居中 */
.admin-login {
  width: 100%;
  max-width: 26rem;
  margin-inline: auto;
  display: grid;
  gap: var(--sos-space-4);
}

.admin-panel {
  display: grid;
  gap: var(--sos-space-8);
}

/* 上传 */
.admin-upload {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--sos-space-4);
}
.admin-file {
  flex: 1;
  min-width: 0;
  font-size: var(--sos-text-sm);
  color: var(--sos-text-tertiary);
}
.admin-file::file-selector-button {
  margin-right: var(--sos-space-3);
  padding: var(--sos-space-2) var(--sos-space-4);
  border: 0;
  border-radius: var(--sos-radius-full);
  background: var(--sos-bg-muted);
  color: var(--sos-text-primary);
  font-size: var(--sos-text-sm);
  font-weight: var(--sos-weight-heavy);
  cursor: pointer;
  transition: background-color var(--sos-duration-fast) var(--sos-ease-out);
}
.admin-file::file-selector-button:hover {
  background: var(--sos-accent-soft);
}

/* 反馈文案 */
.admin-msg {
  margin: 0;
  font-size: var(--sos-text-sm);
}
.admin-msg.is-success {
  color: var(--sos-success);
}
.admin-msg.is-error {
  color: var(--sos-danger);
}

/* 说明 */
.admin-note {
  display: grid;
  gap: var(--sos-space-1);
  font-size: var(--sos-text-xs);
  color: var(--sos-text-secondary);
  border-style: dashed;
}
.admin-note__title {
  font-weight: var(--sos-weight-heavy);
  color: var(--sos-text-primary);
}
.admin-code {
  font-family: var(--sos-font-mono);
  font-size: var(--sos-text-2xs);
  padding: 0 0.3em;
  border-radius: var(--sos-radius-xs);
  background: var(--sos-bg-muted);
}

/* 书籍列表 */
.admin-list {
  padding: 0;
  overflow: hidden;
}
.admin-list__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--sos-space-4) var(--sos-space-5);
  background: var(--sos-bg-subtle);
  border-bottom: 1px solid var(--sos-border-subtle);
}
.admin-list__title {
  font-weight: var(--sos-weight-heavy);
  color: var(--sos-text-primary);
}
.admin-books {
  list-style: none;
  margin: 0;
  padding: 0;
}
.admin-book {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-3);
  padding: var(--sos-space-4) var(--sos-space-5);
  border-bottom: 1px solid var(--sos-border-subtle);
  transition: background-color var(--sos-duration-fast) var(--sos-ease-out);
}
.admin-book:last-child {
  border-bottom: 0;
}
.admin-book:hover {
  background: color-mix(in srgb, var(--sos-bg-subtle) 50%, transparent);
}
.admin-book__main {
  display: flex;
  align-items: flex-start;
  gap: var(--sos-space-3);
  min-width: 0;
  flex: 1;
}
.admin-book__cover {
  width: 2.5rem;
  height: 3.5rem;
  flex-shrink: 0;
  overflow: hidden;
  border-radius: var(--sos-media-radius);
  border: 1px solid var(--sos-border-subtle);
  background: var(--sos-bg-muted);
}
.admin-book__cover > img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.admin-book__fields {
  min-width: 0;
  flex: 1;
  display: grid;
  gap: var(--sos-space-1);
}
.admin-book__meta {
  margin-top: var(--sos-space-2);
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--sos-space-3);
  font-size: var(--sos-text-xs);
}
.admin-inline {
  display: inline-flex;
  align-items: center;
  gap: var(--sos-space-1);
}
.admin-inline__label {
  color: var(--sos-text-tertiary);
}
.admin-inline__select,
.admin-inline__order {
  height: auto;
  padding: var(--sos-space-1) var(--sos-space-2);
  font-size: var(--sos-text-xs);
}
.admin-inline__order {
  width: 7rem;
}

/* 行内可编辑：虚线下划线，聚焦着色 */
.admin-edit {
  width: 100%;
  background: transparent;
  border: 0;
  border-bottom: 1px dashed transparent;
  color: var(--sos-text-primary);
  transition: border-color var(--sos-duration-fast) var(--sos-ease-out);
}
.admin-edit:hover {
  border-bottom-color: var(--sos-border-default);
}
.admin-edit:focus {
  outline: none;
  border-bottom-color: var(--sos-accent);
}
.admin-edit--title {
  font-size: var(--sos-text-sm);
  font-weight: var(--sos-weight-heavy);
}
.admin-edit--author {
  font-size: var(--sos-text-xs);
  color: var(--sos-text-secondary);
}

.admin-book__actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--sos-space-2);
}
.admin-delete {
  color: var(--sos-danger);
}
.admin-delete:hover:not(:disabled) {
  background: var(--sos-danger-soft);
}

.admin-empty {
  padding: var(--sos-space-6);
  text-align: center;
  font-size: var(--sos-text-sm);
  color: var(--sos-text-tertiary);
}

@media (min-width: 640px) {
  .admin-book {
    flex-direction: row;
    align-items: flex-start;
    justify-content: space-between;
  }
}
</style>

