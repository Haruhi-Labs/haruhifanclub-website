<template>
  <div class="min-h-screen bg-[#FAF9DE] flex flex-col items-center py-12 px-4 text-[#4A3B32]">
    <h1 class="text-3xl font-bold mb-8 font-serif text-[#5C4B41]">
      藏书阁管理
    </h1>

    <!-- 登录框 -->
    <div
      v-if="!isLoggedIn"
      class="bg-white p-8 rounded-lg shadow-sm border border-[#E6DFD0] w-full max-w-md"
    >
      <p class="mb-4 text-sm text-[#8C7B70]">管理员登录</p>
      <input
        type="text"
        v-model="loginForm.username"
        class="w-full px-4 py-2 border border-[#D1C4B6] rounded focus:outline-none focus:border-[#D97757] mb-3"
        placeholder="用户名"
        @keyup.enter="login"
      />
      <input
        type="password"
        v-model="loginForm.password"
        class="w-full px-4 py-2 border border-[#D1C4B6] rounded focus:outline-none focus:border-[#D97757] mb-4"
        placeholder="密码"
        @keyup.enter="login"
      />
      <button
        @click="login"
        class="w-full bg-[#D97757] text-white py-2 rounded hover:bg-[#C05F40] transition-colors"
      >
        进入
      </button>
      <p v-if="loginError" class="mt-3 text-sm text-red-500">{{ loginError }}</p>
    </div>

    <!-- 管理面板 -->
    <div v-else class="w-full max-w-5xl space-y-8">
      <!-- 上传区 -->
      <div class="bg-white p-8 rounded-lg shadow-sm border border-[#E6DFD0]">
        <h2 class="text-xl font-bold mb-4">上传新书 (EPUB)</h2>
        <div class="flex flex-col sm:flex-row gap-4 items-center">
          <input
            type="file"
            ref="fileInput"
            accept=".epub"
            class="block w-full text-sm text-slate-500 file:mr-4 file:py-2 file:px-4 file:rounded-full file:border-0 file:text-sm file:font-semibold file:bg-[#F2EFE4] file:text-[#5C4B41] hover:file:bg-[#EBE5D5]"
          />
          <button
            @click="uploadBook"
            :disabled="uploading"
            class="px-6 py-2 bg-[#D97757] text-white rounded disabled:opacity-50 hover:bg-[#C05F40]"
          >
            {{ uploading ? '解析中...' : '上传' }}
          </button>
        </div>
        <p
          v-if="message"
          :class="{
            'text-green-600': success,
            'text-red-500': !success,
          }"
          class="mt-3 text-sm"
        >
          {{ message }}
        </p>
      </div>

      <!-- 分栏说明 -->
      <div
        class="bg-[#F2EFE4] border border-dashed border-[#E6DFD0] rounded-lg p-4 text-xs text-[#8C7B70]"
      >
        <p class="mb-1 font-semibold text-[#5C4B41]">分栏 & 排序说明</p>
        <p class="mb-1">
          「分栏」会影响书架上的分组展示，例如：
          <span class="font-mono">正传小说 / 设定集 / 社区同人</span>。
        </p>
        <p>
          「排序值」为数字，<span class="font-semibold">数值越小越靠前</span>。
          默认会按导入时间生成，你可以在这里自由调整。
        </p>
      </div>

      <!-- 书籍列表 -->
      <div class="bg-white rounded-lg shadow-sm border border-[#E6DFD0] overflow-hidden">
        <div
          class="p-4 bg-[#F2EFE4] font-bold border-b border-[#E6DFD0] flex items-center justify-between"
        >
          <span>已发布书籍</span>
          <span class="text-xs text-[#8C7B70]">共 {{ books.length }} 本</span>
        </div>

        <ul>
          <li
            v-for="book in books"
            :key="book.id"
            class="p-4 border-b border-gray-100 flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between hover:bg-gray-50"
          >
            <!-- 左边：封面 + 可编辑信息 -->
            <div class="flex items-start gap-3 min-w-0 flex-1">
              <div class="w-10 h-14 bg-gray-200 overflow-hidden rounded flex-shrink-0">
                <img
                  v-if="book.cover_path"
                  :src="getCoverUrl(book.cover_path)"
                  class="w-full h-full object-cover"
                />
              </div>

              <div class="min-w-0 space-y-1 flex-1">
                <!-- 标题 -->
                <input
                  v-model="book.title"
                  class="w-full text-sm font-bold bg-transparent border-b border-dashed border-transparent focus:border-[#D97757] focus:outline-none"
                  placeholder="书名"
                />

                <!-- 作者 -->
                <input
                  v-model="book.author"
                  class="w-full text-xs text-gray-600 bg-transparent border-b border-dashed border-transparent focus:border-[#D97757] focus:outline-none"
                  placeholder="作者"
                />

                <!-- 分栏 + 排序 -->
                <div class="mt-2 flex flex-wrap items-center gap-3 text-xs">
                  <div class="flex items-center gap-1">
                    <span class="text-[#8C7B70]">分栏</span>
                    <select
                      v-model="book.category"
                      class="border border-[#D1C4B6] rounded px-2 py-1 text-xs bg-white focus:outline-none focus:ring-1 focus:ring-[#D97757]"
                    >
                      <option :value="null">自动归类（默认）</option>
                      <option
                        v-for="cat in CATEGORY_OPTIONS"
                        :key="cat.key"
                        :value="cat.key"
                      >
                        {{ cat.label }}
                      </option>
                    </select>
                  </div>

                  <div class="flex items-center gap-1">
                    <span class="text-[#8C7B70]">排序值</span>
                    <input
                      type="number"
                      v-model.number="book.order"
                      class="w-24 border border-[#D1C4B6] rounded px-2 py-1 text-xs bg-white focus:outline-none focus:ring-1 focus:ring-[#D97757]"
                      placeholder="越小越靠前"
                    />
                  </div>

                  <span
                    v-if="book.category"
                    class="px-2 py-0.5 rounded-full bg-[#FAF9DE] border border-[#E6DFD0] text-[10px] text-[#8C7B70]"
                  >
                    {{ getCategoryLabel(book.category) }}
                  </span>
                </div>
              </div>
            </div>

            <!-- 右边：操作 -->
            <div class="flex items-center gap-3 justify-end">
              <button
                @click="saveBook(book)"
                class="px-3 py-1 text-xs rounded bg-[#4CAF50]/90 text-white hover:bg-[#43A047]"
              >
                保存
              </button>
              <button
                @click="deleteBook(book.id)"
                class="px-3 py-1 text-xs rounded text-red-500 hover:bg-red-50"
              >
                删除
              </button>
            </div>
          </li>
        </ul>

        <div
          v-if="!books.length"
          class="p-6 text-center text-sm text-[#8C7B70]"
        >
          当前还没有书籍，请先上传 EPUB 文件。
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue';
import { createApiClient, createAdminAuth } from '@haruhi/api-client';

// 统一后端：模块 API 走 /api/novel，鉴权走共享的 createAdminAuth（已内置 novel 权限校验），静态文件走 /uploads
const api = createApiClient('/api/novel');
const admin = createAdminAuth('novel');
const ASSET_BASE = '/uploads';

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

const getCoverUrl = (path) => (path ? `${ASSET_BASE}/${path}` : '');

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

