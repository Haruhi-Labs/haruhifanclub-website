<template>
  <article class="work-page">
    <nav class="work-path" aria-label="面包屑导航">
      <button type="button" class="work-path__back" @click="goBack">
        <ArrowLeft :size="17" :stroke-width="2.2" aria-hidden="true" />
        <span>返回</span>
      </button>
      <RouterLink :to="{ name: 'gallery' }">画廊</RouterLink>
      <span aria-hidden="true">/</span>
      <span>ARTWORK {{ route.params.id }}</span>
    </nav>

    <div v-if="loading" class="work-state" aria-live="polite">
      <span class="work-state__scanner" aria-hidden="true"></span>
      <strong>正在展开作品档案</strong>
      <small>读取原图与创作者信息…</small>
    </div>

    <div v-else-if="error || !art" class="work-state work-state--error">
      <strong>没有找到这件作品</strong>
      <small>{{ error || '作品可能已撤下，或暂时不可访问。' }}</small>
      <RouterLink class="work-button work-button--accent" :to="{ name: 'gallery' }">返回画廊</RouterLink>
    </div>

    <template v-else>
      <div class="work-layout">
        <section class="work-canvas" :aria-label="`${art.title || '作品'}图片`">
          <div v-if="images.length > 1" class="work-canvas__summary">
            <span>MULTI IMAGE / {{ String(images.length).padStart(2, '0') }}</span>
            <small>向下连续浏览全部图片</small>
          </div>
          <template v-for="(image, index) in images" :key="`${art.id}-${index}`">
            <figure class="work-image">
              <button
                type="button"
                class="work-image__open"
                :aria-label="`查看第 ${index + 1} 张原图`"
                @click="openViewer(index)"
              >
                <img
                  :src="image.image_url || image.original_url"
                  :alt="images.length > 1 ? `${art.title || '作品'}，第 ${index + 1} 张` : (art.title || '作品')"
                  :loading="index === 0 ? 'eager' : 'lazy'"
                  decoding="async"
                />
                <span class="work-image__hint">
                  <Maximize2 :size="16" aria-hidden="true" />
                  查看原图
                </span>
              </button>
              <figcaption v-if="images.length > 1">
                <span>{{ String(index + 1).padStart(2, '0') }}</span>
                <span>/</span>
                <span>{{ String(images.length).padStart(2, '0') }}</span>
                <a
                  v-if="image.original_url"
                  :href="image.original_url"
                  :download="downloadFilename(index)"
                  aria-label="下载这张原图"
                >
                  <Download :size="14" aria-hidden="true" />
                </a>
              </figcaption>
            </figure>
          </template>
        </section>

        <section class="work-meta" aria-label="作品信息">
          <h1>{{ art.title || '未命名作品' }}</h1>
          <p v-if="art.description" class="work-meta__description">{{ art.description }}</p>

          <div v-if="tags.length" class="work-tags" aria-label="作品标签">
            <button v-for="tag in tags" :key="tag" type="button" @click="searchTag(tag)">#{{ tag }}</button>
          </div>

          <div class="work-meta__activity" aria-label="作品数据与操作">
            <button
              type="button"
              class="work-metric"
              :class="{ 'is-liked': liked }"
              :disabled="liking"
              :aria-label="liked ? `已喜欢，${likeCount}` : `喜欢，${likeCount}`"
              @click="likeArtwork"
            >
              <Heart :size="17" :fill="liked ? 'currentColor' : 'none'" aria-hidden="true" />
              <span>{{ compactNumber(likeCount) }}</span>
            </button>
            <span class="work-metric" :aria-label="`${viewCount} 次浏览`">
              <Eye :size="17" aria-hidden="true" />
              <span>{{ compactNumber(viewCount) }}</span>
            </span>
            <a class="work-metric" href="#artwork-comments" :aria-label="`${commentCount} 条评论`">
              <MessageCircle :size="17" aria-hidden="true" />
              <span>{{ compactNumber(commentCount) }}</span>
            </a>
            <span class="work-meta__tools">
              <span ref="licensePopover" class="work-license">
                <button
                  type="button"
                  class="work-license__trigger"
                  aria-haspopup="dialog"
                  aria-controls="artwork-public-license"
                  :aria-expanded="licensePopoverOpen"
                  aria-label="查看大众授权情况"
                  title="查看大众授权情况"
                  @click.stop="toggleLicensePopover"
                >
                  <CircleHelp :size="19" aria-hidden="true" />
                </button>
                <Transition name="work-license-popover">
                  <div
                    v-if="licensePopoverOpen"
                    id="artwork-public-license"
                    class="work-license__popover"
                    role="dialog"
                    aria-label="大众授权情况"
                    @click.stop
                  >
                    <header>
                      <strong>大众授权</strong>
                      <small>以作者当前公开设置为准</small>
                    </header>
                    <ul>
                      <li v-for="item in publicLicenseRows" :key="item.label" :class="{ 'is-granted': item.granted }">
                        <span>{{ item.label }}</span>
                        <Check v-if="item.granted" :size="17" :stroke-width="2.6" aria-label="允许" />
                        <X v-else :size="17" :stroke-width="2.4" aria-label="不允许" />
                      </li>
                    </ul>
                  </div>
                </Transition>
              </span>
              <button
                type="button"
                class="work-favorite"
                :class="{ 'is-favorited': favorited }"
                :disabled="favoriting"
                :aria-label="`${favorited ? '取消收藏' : '收藏作品'}，${favoriteCount} 人已收藏`"
                :title="favorited ? '取消收藏' : '收藏作品'"
                @click="toggleFavorite"
              >
                <Star :size="19" :fill="favorited ? 'currentColor' : 'none'" aria-hidden="true" />
              </button>
              <a
                v-if="firstOriginalUrl"
                :href="firstOriginalUrl"
                :download="downloadFilename(0)"
                aria-label="下载原图"
                title="下载原图"
              ><Download :size="19" aria-hidden="true" /></a>
              <a
                v-if="safeOriginUrl"
                :href="safeOriginUrl"
                target="_blank"
                rel="noopener noreferrer"
                aria-label="查看作品原始出处"
                title="查看作品原始出处"
              ><ExternalLink :size="18" aria-hidden="true" /></a>
            </span>
          </div>

          <time class="work-meta__time" :datetime="publishedIso">{{ publishedAt }}</time>
          <span class="work-meta__rule" aria-hidden="true"></span>
        </section>

        <section class="work-creator" aria-label="创作者与其他作品">
          <header class="work-creator__head">
            <button v-if="authorUid" type="button" class="work-creator__author" @click="openAuthor">
              <span class="work-avatar">
                <img v-if="art.uploader_avatar" :src="art.uploader_avatar" alt="" />
                <span v-else>{{ authorInitial }}</span>
              </span>
              <span><b>{{ authorName }}</b><small>@{{ authorUid }}</small></span>
            </button>
            <div v-else class="work-creator__author is-static">
              <span class="work-avatar">{{ authorInitial }}</span>
              <span><b>{{ authorName }}</b><small>画廊收藏</small></span>
            </div>
            <button
              v-if="authorUid && !creatorSocial.isSelf"
              type="button"
              class="work-follow"
              :class="{ 'is-following': creatorSocial.isFollowing }"
              :disabled="followLoading"
              @click="toggleFollow"
            >
              {{ creatorSocial.isFollowing ? '已关注' : '关注' }}
            </button>
          </header>
          <div v-if="creatorSequenceWorks.length" ref="creatorStrip" class="work-creator__works">
            <button
              v-for="item in creatorSequenceWorks"
              :key="item.id"
              type="button"
              :class="{ 'is-current': String(item.id) === String(art.id) }"
              :title="item.title || '未命名作品'"
              :aria-label="`查看作品：${item.title || '未命名作品'}`"
              @click="openSequenceWork(item)"
            >
              <span :style="{ backgroundImage: `url(${artworkThumb(item)})` }" aria-hidden="true"></span>
              <img :src="artworkThumb(item)" :alt="item.title || '画廊作品'" loading="lazy" decoding="async" />
            </button>
          </div>
          <p v-else class="work-muted">暂无更多公开作品。</p>
        </section>
      </div>

      <section id="artwork-comments" class="work-comments">
        <header class="work-section-head">
          <div>
            <h2>评论</h2>
          </div>
          <b>{{ comments.length }}</b>
        </header>

        <div class="work-comments__layout">
          <div class="work-comment-list">
            <div v-if="loadingComments" class="work-muted">正在接收回音…</div>
            <div v-else-if="!comments.length" class="work-comment-empty">
              <MessageCircle :size="24" aria-hidden="true" />
              <strong>这里还很安静</strong>
              <span>成为第一个认真谈谈这件作品的人。</span>
            </div>
            <article v-for="comment in comments" :key="comment.id" class="work-comment">
              <span class="work-comment__avatar">{{ commentInitial(comment) }}</span>
              <div class="work-comment__body">
                <header>
                  <b>{{ comment.user_name || '匿名访客' }}</b>
                  <time>{{ formatDateTime(comment.created_at) }}</time>
                </header>
                <p>{{ comment.body }}</p>
                <button type="button" @click="likeComment(comment)">
                  <Heart :size="14" aria-hidden="true" />
                  {{ Number(comment.like_total || 0) }}
                </button>
              </div>
            </article>
          </div>

          <form v-if="isLoggedIn" class="work-compose" @submit.prevent="postComment">
            <span class="work-panel__label">LEAVE A MESSAGE</span>
            <label for="artwork-comment">以 {{ accountNickname || '账号昵称' }} 署名</label>
            <textarea
              id="artwork-comment"
              v-model="commentBody"
              rows="5"
              maxlength="800"
              placeholder="说说你从这件作品中看到了什么…"
            ></textarea>
            <div>
              <small>{{ commentBody.length }} / 800</small>
              <button class="work-button work-button--accent" type="submit" :disabled="posting || !commentBody.trim()">
                <Send :size="16" aria-hidden="true" />
                {{ posting ? '发送中…' : '发送评论' }}
              </button>
            </div>
            <p v-if="commentNotice" class="work-compose__notice">{{ commentNotice }}</p>
          </form>
          <div v-else class="work-compose work-compose--login">
            <span class="work-panel__label">LEAVE A MESSAGE</span>
            <strong>登录后参与作品讨论</strong>
            <p>你的账号昵称会作为署名，不需要额外填写身份信息。</p>
            <button class="work-button work-button--accent" type="button" @click="goLogin">登录 / 注册</button>
          </div>
        </div>
      </section>

      <section v-if="relatedWorks.length" class="work-discovery work-discovery--related">
        <header class="work-section-head">
          <div>
            <h2>推荐作品</h2>
          </div>
          <p>根据标签、主题与作品属性生成</p>
        </header>
        <ArtworkShelf
          :items="relatedWorks"
          tracking-source="artwork-related"
          @open="openWork"
        />
      </section>
    </template>
  </article>

  <Teleport to="body">
    <div
      v-if="viewerOpen"
      class="image-viewer"
      role="dialog"
      aria-modal="true"
      :aria-label="`${art?.title || '作品'}原图查看器`"
      @click.self="closeViewer"
    >
      <header class="image-viewer__bar">
        <span>{{ viewerIndex + 1 }} / {{ images.length }}</span>
        <div>
          <button type="button" aria-label="缩小" @click="viewerScale = Math.max(1, viewerScale - 0.5)"><Minus :size="19" /></button>
          <span>{{ Math.round(viewerScale * 100) }}%</span>
          <button type="button" aria-label="放大" @click="viewerScale = Math.min(4, viewerScale + 0.5)"><Plus :size="19" /></button>
          <a
            v-if="viewerImage?.original_url"
            :href="viewerImage.original_url"
            :download="downloadFilename(viewerIndex)"
            aria-label="下载当前原图"
          ><Download :size="19" /></a>
          <button type="button" aria-label="关闭" @click="closeViewer"><X :size="21" /></button>
        </div>
      </header>
      <button v-if="images.length > 1" class="image-viewer__nav is-prev" type="button" aria-label="上一张" @click="prevViewer">
        <ChevronLeft :size="28" />
      </button>
      <div class="image-viewer__stage" @dblclick="toggleViewerZoom">
        <img
          v-if="viewerImage"
          :src="viewerImage.original_url || viewerImage.image_url"
          :alt="art?.title || '作品原图'"
          :style="{ transform: `scale(${viewerScale})` }"
          draggable="false"
        />
      </div>
      <button v-if="images.length > 1" class="image-viewer__nav is-next" type="button" aria-label="下一张" @click="nextViewer">
        <ChevronRight :size="28" />
      </button>
    </div>
  </Teleport>
</template>

<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSession } from '@haruhi/auth-ui'
import { absoluteUrl, canonicalUrl, usePageMeta } from '@haruhi/seo'
import {
  ArrowLeft,
  Check,
  ChevronLeft,
  ChevronRight,
  CircleHelp,
  Download,
  Eye,
  ExternalLink,
  Heart,
  Maximize2,
  MessageCircle,
  Minus,
  Plus,
  Send,
  Star,
  X,
} from 'lucide-vue-next'
import ArtworkShelf from '../components/ArtworkShelf.vue'
import { api, thumbUrl } from '../services/api.js'
import { finishArtworkView, startArtworkView } from '../services/recommendationTracker.js'
import { useGalleryStore } from '../stores/galleryStore.js'

const route = useRoute()
const router = useRouter()
const session = useSession('/api')
const galleryStore = useGalleryStore()

const art = ref(null)
const loading = ref(true)
const error = ref('')
const comments = ref([])
const loadingComments = ref(false)
const creatorSequenceWorks = ref([])
const relatedWorks = ref([])
const commentBody = ref('')
const posting = ref(false)
const commentNotice = ref('')
const liking = ref(false)
const liked = ref(false)
const favorited = ref(false)
const favoriting = ref(false)
const favoriteCount = ref(0)
const creatorSocial = ref({ isFollowing: false, isSelf: false, followerCount: 0 })
const followLoading = ref(false)
const creatorStrip = ref(null)
const licensePopover = ref(null)
const licensePopoverOpen = ref(false)
const viewerOpen = ref(false)
const viewerIndex = ref(0)
const viewerScale = ref(1)
let loadVersion = 0

const isLoggedIn = computed(() => Boolean(session.state.user))
const accountNickname = computed(() => session.state.user?.nickname || session.state.user?.username || '')
const authorUid = computed(() => String(art.value?.uploader_uid || '').trim())
const authorName = computed(() => art.value?.uploader_display_name || art.value?.uploader_name || authorUid.value || '匿名创作者')
const authorInitial = computed(() => String(authorName.value || '画').slice(0, 1).toUpperCase())
const images = computed(() => {
  if (Array.isArray(art.value?.images) && art.value.images.length) return art.value.images
  if (!art.value) return []
  return [{ image_url: art.value.image_url || '', original_url: art.value.original_url || art.value.image_url || '' }]
})
const firstOriginalUrl = computed(() => images.value[0]?.original_url || '')
const safeOriginUrl = computed(() => {
  const value = String(art.value?.origin_url || '').trim()
  if (!value) return ''
  try {
    const parsed = new URL(value)
    return ['http:', 'https:'].includes(parsed.protocol) ? parsed.href : ''
  } catch {
    return ''
  }
})
const viewerImage = computed(() => images.value[viewerIndex.value] || null)
const tags = computed(() => Array.isArray(art.value?.tags) ? art.value.tags : [])
const likeCount = computed(() => Number(art.value?.popularity?.likes ?? art.value?.like_total ?? 0))
const viewCount = computed(() => Number(art.value?.popularity?.views || 0))
const commentCount = computed(() => Math.max(Number(art.value?.popularity?.comments || 0), comments.value.length))
const publishedAt = computed(() => formatDateTime(art.value?.created_at || art.value?.reviewed_at))
const publishedIso = computed(() => {
  const value = art.value?.created_at || art.value?.reviewed_at
  const date = value ? new Date(value) : null
  return date && !Number.isNaN(date.getTime()) ? date.toISOString() : ''
})
const PUBLIC_LICENSE_OPTIONS = [
  '可在b站、小红书等社交媒体转载',
  '允许用于视频等个人创作',
  '允许用于制作无料发放',
]
const publicLicenseRows = computed(() => {
  const licenses = new Set(Array.isArray(art.value?.licenses) ? art.value.licenses : [])
  return PUBLIC_LICENSE_OPTIONS.map(label => ({
    label,
    granted: licenses.has(`NET:${label}`) || licenses.has(label),
  }))
})

usePageMeta(() => {
  if (!art.value) return null
  const title = art.value.title || '未命名作品'
  return {
    title: `${title} · ${authorName.value} · 春日画廊`,
    description: String(art.value.description || `${authorName.value} 在春日画廊发布的作品《${title}》。`).slice(0, 150),
    canonical: canonicalUrl(`/artwork/${encodeURIComponent(art.value.id)}`),
    ogType: 'article',
    ogImage: art.value.image_url ? absoluteUrl(art.value.image_url) : undefined,
    jsonLd: {
      '@context': 'https://schema.org',
      '@type': 'VisualArtwork',
      name: title,
      image: images.value.map((image) => absoluteUrl(image.image_url || image.original_url)).filter(Boolean),
      creator: { '@type': 'Person', name: authorName.value },
      datePublished: art.value.reviewed_at || art.value.created_at || undefined,
      description: art.value.description || undefined,
    },
  }
})

function compactNumber(value) {
  return new Intl.NumberFormat('zh-CN', {
    notation: Number(value) >= 10_000 ? 'compact' : 'standard',
    maximumFractionDigits: 1,
  }).format(Number(value) || 0)
}

function artworkThumb(item) {
  return thumbUrl(item?.image_url || item?.imageUrl || item?.url || '', 640)
}

function formatDateTime(value) {
  if (!value) return '-'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return String(value)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

function safeFilename(value) {
  return String(value || '').replace(/[\\/:*?"<>|]/g, '_').trim()
}

function downloadFilename(index) {
  const date = art.value?.created_at ? new Date(art.value.created_at) : null
  const dateText = date && !Number.isNaN(date.getTime())
    ? `${date.getFullYear()}${String(date.getMonth() + 1).padStart(2, '0')}${String(date.getDate()).padStart(2, '0')}`
    : '00000000'
  const suffix = images.value.length > 1 ? `_${index + 1}` : ''
  const source = images.value[index]?.original_url || ''
  const ext = source.split(/[?#]/)[0].split('.').pop() || 'jpg'
  return `${dateText}-${safeFilename(authorName.value)}-${safeFilename(art.value?.title || '作品')}${suffix}.${ext}`
}

async function loadComments(id, version) {
  loadingComments.value = true
  try {
    const response = await api.listComments(id)
    if (version === loadVersion) comments.value = response.data || []
  } catch {
    if (version === loadVersion) comments.value = []
  } finally {
    if (version === loadVersion) loadingComments.value = false
  }
}

async function loadDiscovery(current, version) {
  const [relatedResult, creatorResult, profileResult] = await Promise.allSettled([
    api.relatedArtworks(current.id, 8),
    api.creatorNeighbors(current),
    current.uploader_uid
      ? api.guildProfile(current.uploader_uid)
      : Promise.resolve({ social: {} }),
  ])
  if (version !== loadVersion) return
  relatedWorks.value = relatedResult.status === 'fulfilled'
    ? (relatedResult.value.data || []).filter((item) => String(item.id) !== String(current.id)).slice(0, 8)
    : []
  creatorSequenceWorks.value = creatorResult.status === 'fulfilled'
    ? (creatorResult.value.data || []).slice(0, 3)
    : []
  creatorSocial.value = profileResult.status === 'fulfilled'
    ? { ...creatorSocial.value, ...(profileResult.value.social || {}) }
    : creatorSocial.value
  centerCreatorWork()
}

async function loadArtwork() {
  const id = String(route.params.id || '').trim()
  const version = ++loadVersion
  finishArtworkView()
  closeViewer()
  loading.value = true
  error.value = ''
  art.value = null
  comments.value = []
  relatedWorks.value = []
  creatorSequenceWorks.value = []
  creatorSocial.value = { isFollowing: false, isSelf: false, followerCount: 0 }
  commentBody.value = ''
  commentNotice.value = ''
  liked.value = false
  favorited.value = false
  favoriteCount.value = 0
  licensePopoverOpen.value = false
  if (!id) {
    error.value = '作品编号无效。'
    loading.value = false
    return
  }
  const item = await galleryStore.fetchArtworkById(id)
  if (version !== loadVersion) return
  if (!item) {
    error.value = '作品可能已撤下，或暂时不可访问。'
    loading.value = false
    return
  }
  art.value = item
  favorited.value = Boolean(item.favorited)
  favoriteCount.value = Number(item.favorite_count || 0)
  loading.value = false
  startArtworkView(item)
  void loadComments(item.id, version)
  void loadDiscovery(item, version)
}

async function likeArtwork() {
  if (!art.value || liking.value || liked.value) return
  liking.value = true
  const before = Number(art.value.popularity?.likes ?? art.value.like_total ?? 0)
  if (art.value.popularity) art.value.popularity.likes = before + 1
  art.value.like_total = before + 1
  liked.value = true
  try {
    const response = await api.likeArtwork(art.value.id)
    const total = Number(response?.totalLikes ?? response?.total_likes ?? before + 1)
    art.value.like_total = total
    if (art.value.popularity) art.value.popularity.likes = total
  } catch (cause) {
    art.value.like_total = before
    if (art.value.popularity) art.value.popularity.likes = before
    liked.value = false
    commentNotice.value = cause?.message || '喜欢操作失败，请稍后重试。'
  } finally {
    liking.value = false
  }
}

async function toggleFavorite() {
  if (!isLoggedIn.value) {
    goLogin()
    return
  }
  if (!art.value?.id || favoriting.value) return
  favoriting.value = true
  try {
    const response = await api.toggleArtworkFavorite(art.value.id)
    favorited.value = Boolean(response.favorited)
    favoriteCount.value = Number(response.favoriteCount || 0)
    art.value.favorited = favorited.value
    art.value.favorite_count = favoriteCount.value
  } catch (cause) {
    commentNotice.value = cause?.message || '收藏操作失败，请稍后重试。'
  } finally {
    favoriting.value = false
  }
}

async function toggleFollow() {
  if (!isLoggedIn.value) {
    goLogin()
    return
  }
  if (!authorUid.value || followLoading.value) return
  followLoading.value = true
  try {
    const response = await api.toggleGuildFollow(authorUid.value)
    creatorSocial.value = {
      ...creatorSocial.value,
      isFollowing: Boolean(response.following),
      followerCount: Number(response.followerCount || 0),
    }
  } catch (cause) {
    commentNotice.value = cause?.message || '关注操作失败，请稍后重试。'
  } finally {
    followLoading.value = false
  }
}

async function postComment() {
  const body = commentBody.value.trim()
  if (!art.value?.id || !isLoggedIn.value || !body || posting.value) return
  posting.value = true
  commentNotice.value = ''
  try {
    await api.postComment({ artwork_id: art.value.id, body })
    commentBody.value = ''
    await loadComments(art.value.id, loadVersion)
    commentNotice.value = '评论已发送。'
  } catch (cause) {
    commentNotice.value = cause?.message || '评论发送失败，请稍后重试。'
  } finally {
    posting.value = false
  }
}

async function likeComment(comment) {
  if (!comment || comment._liking) return
  comment._liking = true
  const before = Number(comment.like_total || 0)
  comment.like_total = before + 1
  try {
    const response = await api.likeComment(comment.id)
    comment.like_total = Number(response?.totalLikes ?? response?.total_likes ?? before + 1)
  } catch {
    comment.like_total = before
  } finally {
    comment._liking = false
  }
}

function commentInitial(comment) {
  return String(comment?.user_name || '画').slice(0, 1).toUpperCase()
}

function openWork(item) {
  if (!item?.id) return
  router.push({ name: 'artwork-detail', params: { id: item.id } })
}

function openSequenceWork(item) {
  if (!item?.id || String(item.id) === String(art.value?.id)) return
  openWork(item)
}

function centerCreatorWork() {
  nextTick(() => {
    const strip = creatorStrip.value
    const current = strip?.querySelector('.is-current')
    if (!strip || !current) return
    strip.scrollLeft = current.offsetLeft - (strip.clientWidth - current.clientWidth) / 2
  })
}

function openAuthor() {
  if (!authorUid.value) return
  router.push({ name: 'adventurer-profile', params: { uid: authorUid.value }, query: { from: 'artwork' } })
}

function searchTag(tag) {
  router.push({ name: 'gallery-search', query: { q: tag, field: 'tag' } })
}

function goLogin() {
  router.push({ name: 'login', query: { redirect: route.fullPath } })
}

function goBack() {
  if (window.history.length > 1) router.back()
  else router.push({ name: 'gallery' })
}

function openViewer(index) {
  viewerIndex.value = index
  viewerScale.value = 1
  viewerOpen.value = true
}

function closeViewer() {
  viewerOpen.value = false
  viewerScale.value = 1
}

function prevViewer() {
  viewerIndex.value = (viewerIndex.value - 1 + images.value.length) % images.value.length
  viewerScale.value = 1
}

function nextViewer() {
  viewerIndex.value = (viewerIndex.value + 1) % images.value.length
  viewerScale.value = 1
}

function toggleViewerZoom() {
  viewerScale.value = viewerScale.value > 1 ? 1 : 2
}

function toggleLicensePopover() {
  licensePopoverOpen.value = !licensePopoverOpen.value
}

function onDocumentPointerDown(event) {
  if (!licensePopoverOpen.value || licensePopover.value?.contains(event.target)) return
  licensePopoverOpen.value = false
}

function onKeydown(event) {
  if (event.key === 'Escape' && licensePopoverOpen.value) {
    licensePopoverOpen.value = false
    return
  }
  if (!viewerOpen.value) return
  if (event.key === 'Escape') closeViewer()
  if (event.key === 'ArrowLeft' && images.value.length > 1) prevViewer()
  if (event.key === 'ArrowRight' && images.value.length > 1) nextViewer()
}

watch(() => route.params.id, loadArtwork)
watch(viewerOpen, (open) => {
  document.documentElement.classList.toggle('art-image-viewer-open', open)
})

onMounted(() => {
  session.ensureReady?.()
  loadArtwork()
  document.addEventListener('pointerdown', onDocumentPointerDown)
  window.addEventListener('keydown', onKeydown)
})

onBeforeUnmount(() => {
  loadVersion += 1
  finishArtworkView()
  document.documentElement.classList.remove('art-image-viewer-open')
  document.removeEventListener('pointerdown', onDocumentPointerDown)
  window.removeEventListener('keydown', onKeydown)
})
</script>

<style scoped>
.work-page {
  width: min(1380px, calc(100% - 64px));
  min-height: 70vh;
  margin: 0 auto;
  padding: 8px 0 110px;
  color: var(--sos-text-primary);
}

.work-path {
  display: flex;
  min-height: 42px;
  align-items: center;
  gap: 10px;
  color: var(--sos-text-tertiary);
  font-size: 12px;
  font-weight: 800;
  letter-spacing: 0.08em;
}

.work-path a,
.work-path button {
  color: inherit;
  text-decoration: none;
}

.work-path__back {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 0;
  cursor: pointer;
  background: transparent;
  border: 0;
}

.work-path__back:hover,
.work-path a:hover { color: var(--sos-accent); }

.work-panel__label {
  display: block;
  color: var(--sos-text-tertiary);
  font-size: 10px;
  font-weight: 900;
  letter-spacing: 0.1em;
}

.work-avatar {
  display: grid;
  width: 42px;
  height: 42px;
  overflow: hidden;
  place-items: center;
  flex: 0 0 auto;
  color: white;
  font-weight: 900;
  background: linear-gradient(145deg, var(--sos-accent), var(--sos-accent-2));
  border: 1px solid rgba(255, 255, 255, 0.78);
  border-radius: 50%;
}

.work-avatar img { width: 100%; height: 100%; object-fit: cover; }

.work-button {
  display: inline-flex;
  min-height: 43px;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 0 17px;
  color: var(--sos-text-primary);
  font: inherit;
  font-size: 13px;
  font-weight: 850;
  text-decoration: none;
  cursor: pointer;
  background: color-mix(in srgb, var(--sos-bg-surface) 96%, transparent);
  border: 1px solid var(--sos-border-subtle);
  border-radius: 999px;
  box-shadow: 0 8px 22px -18px rgba(20, 52, 58, 0.8);
}

.work-button b { font-size: 11px; font-variant-numeric: tabular-nums; }
.work-button:disabled { cursor: wait; opacity: 0.64; }
.work-button.is-liked { color: #d44957; border-color: color-mix(in srgb, #d44957 45%, white); }
.work-button--accent { color: white; background: var(--sos-accent); border-color: var(--sos-accent); }

.work-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(292px, 340px);
  grid-template-areas:
    "canvas creator"
    "meta creator";
  align-items: start;
  column-gap: 24px;
  row-gap: 0;
  margin-top: 2px;
}

.work-canvas {
  grid-area: canvas;
  display: grid;
  min-width: 0;
  gap: 18px;
  padding: 0;
  background: transparent;
  border: 0;
  box-shadow: none;
}

.work-canvas__summary {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  color: var(--sos-text-secondary);
  font: 800 10px/1.3 ui-monospace, SFMono-Regular, Menlo, monospace;
  letter-spacing: 0.08em;
  background: color-mix(in srgb, var(--sos-bg-surface) 78%, transparent);
  border: 1px solid var(--sos-border-subtle);
}

.work-canvas__summary small {
  color: var(--sos-text-tertiary);
  font: inherit;
  letter-spacing: 0;
}

.work-image { min-width: 0; margin: 0; }
.work-image__open {
  position: relative;
  display: flex;
  width: 100%;
  min-height: 0;
  align-items: center;
  justify-content: center;
  padding: 0;
  overflow: hidden;
  cursor: zoom-in;
  background: transparent;
  border: 0;
}

.work-image__open img {
  display: block;
  width: 100%;
  max-width: none;
  height: auto;
  max-height: none;
  object-fit: contain;
}

.work-image__hint {
  position: absolute;
  right: 12px;
  bottom: 12px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 10px;
  color: white;
  font-size: 11px;
  font-weight: 800;
  background: rgba(5, 15, 19, 0.72);
  border: 1px solid rgba(255, 255, 255, 0.24);
  border-radius: 999px;
  opacity: 0;
  transform: translateY(5px);
  transition: 0.2s ease;
}

.work-image__open:hover .work-image__hint { opacity: 1; transform: none; }
.work-image figcaption {
  display: flex;
  align-items: center;
  gap: 6px;
  padding-top: 8px;
  color: var(--sos-text-tertiary);
  font: 800 10px/1 ui-monospace, SFMono-Regular, Menlo, monospace;
  letter-spacing: 0.08em;
}

.work-image figcaption a { margin-left: auto; color: var(--sos-text-secondary); }

.work-meta {
  grid-area: meta;
  min-width: 0;
  padding: 25px 4px 10px;
}

.work-meta h1 {
  margin: 0;
  font-size: clamp(24px, 2.7vw, 36px);
  font-weight: 950;
  line-height: 1.25;
  letter-spacing: -0.035em;
  overflow-wrap: anywhere;
}

.work-meta__description {
  margin: 13px 0 0;
  color: var(--sos-text-secondary);
  font-size: 14px;
  font-weight: 600;
  line-height: 1.72;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.work-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 10px;
  margin-top: 15px;
}

.work-tags button {
  padding: 0;
  color: color-mix(in srgb, var(--sos-accent) 84%, var(--sos-text-primary));
  font: inherit;
  font-size: 12px;
  font-weight: 750;
  line-height: 1.6;
  cursor: pointer;
  background: transparent;
  border: 0;
}

.work-meta__activity {
  display: flex;
  min-height: 38px;
  align-items: center;
  gap: 18px;
  margin-top: 13px;
}

.work-metric {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 0;
  color: var(--sos-text-tertiary);
  font: inherit;
  font-size: 11px;
  font-weight: 750;
  text-decoration: none;
  cursor: default;
  background: transparent;
  border: 0;
}

button.work-metric { cursor: pointer; }
.work-metric span { font-variant-numeric: tabular-nums; }
.work-metric.is-liked { color: #d44957; }
.work-metric:disabled { cursor: wait; opacity: 0.62; }

.work-meta__tools {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  margin-left: auto;
}

.work-meta__tools a,
.work-meta__tools button {
  display: grid;
  width: 36px;
  height: 36px;
  place-items: center;
  padding: 0;
  color: var(--sos-text-secondary);
  font: inherit;
  text-decoration: none;
  cursor: pointer;
  background: transparent;
  border: 0;
  border-radius: 50%;
  transition: color 0.18s ease, background 0.18s ease;
}

.work-meta__tools a:hover,
.work-meta__tools button:hover {
  color: var(--sos-text-primary);
  background: color-mix(in srgb, var(--sos-bg-surface) 72%, transparent);
}

.work-meta__tools .work-favorite:hover { background: transparent; }

.work-meta__tools button:disabled { cursor: wait; opacity: 0.62; }
.work-meta__tools .work-favorite.is-favorited { color: #e2a621; }

.work-license {
  position: relative;
  display: inline-flex;
}

.work-license__popover {
  position: absolute;
  right: -41px;
  bottom: calc(100% + 10px);
  z-index: 20;
  width: min(320px, calc(100vw - 32px));
  box-sizing: border-box;
  padding: 14px;
  color: var(--sos-text-primary);
  background: color-mix(in srgb, var(--sos-bg-surface) 94%, transparent);
  border: 1px solid color-mix(in srgb, var(--sos-text-primary) 18%, transparent);
  border-top: 2px solid var(--sos-accent);
  border-radius: 5px;
  box-shadow: 0 14px 36px rgba(20, 39, 46, 0.18);
  backdrop-filter: blur(18px) saturate(1.12);
}

.work-license__popover::after {
  position: absolute;
  right: 52px;
  bottom: -6px;
  width: 10px;
  height: 10px;
  content: '';
  background: color-mix(in srgb, var(--sos-bg-surface) 94%, transparent);
  border-right: 1px solid color-mix(in srgb, var(--sos-text-primary) 18%, transparent);
  border-bottom: 1px solid color-mix(in srgb, var(--sos-text-primary) 18%, transparent);
  transform: rotate(45deg);
}

.work-license__popover header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
  padding: 0 2px 10px;
  border-bottom: 1px solid color-mix(in srgb, var(--sos-text-primary) 11%, transparent);
}

.work-license__popover header strong {
  font-size: 13px;
  font-weight: 900;
}

.work-license__popover header small {
  color: var(--sos-text-tertiary);
  font-size: 9px;
  font-weight: 650;
  white-space: nowrap;
}

.work-license__popover ul {
  display: grid;
  gap: 0;
  margin: 6px 0 0;
  padding: 0;
  list-style: none;
}

.work-license__popover li {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 20px;
  align-items: center;
  gap: 12px;
  min-height: 34px;
  color: var(--sos-text-secondary);
  font-size: 11px;
  font-weight: 650;
  line-height: 1.45;
}

.work-license__popover li + li {
  border-top: 1px solid color-mix(in srgb, var(--sos-text-primary) 8%, transparent);
}

.work-license__popover li svg {
  justify-self: end;
  color: #b54d57;
}

.work-license__popover li.is-granted svg { color: #2c9d7f; }

.work-license-popover-enter-active,
.work-license-popover-leave-active {
  transition: opacity 0.16s ease, transform 0.16s ease;
  transform-origin: calc(100% - 52px) 100%;
}

.work-license-popover-enter-from,
.work-license-popover-leave-to {
  opacity: 0;
  transform: translateY(5px) scale(0.98);
}

.work-meta__time {
  display: block;
  margin-top: 8px;
  color: var(--sos-text-tertiary);
  font-size: 10px;
  font-weight: 650;
}

.work-meta__rule {
  display: block;
  width: min(50%, 310px);
  margin-top: 19px;
  border-top: 1px solid color-mix(in srgb, var(--sos-text-primary) 16%, transparent);
}

.work-creator {
  position: sticky;
  top: 86px;
  grid-area: creator;
  min-width: 0;
  padding: 17px;
  overflow: hidden;
  background: color-mix(in srgb, var(--sos-bg-surface) 76%, transparent);
  border-top: 1px solid color-mix(in srgb, var(--sos-text-primary) 16%, transparent);
  backdrop-filter: blur(14px);
}

.work-creator__head {
  display: flex;
  min-width: 0;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 15px;
}

.work-creator__author {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 10px;
  padding: 0;
  color: var(--sos-text-primary);
  font: inherit;
  text-align: left;
  cursor: pointer;
  background: transparent;
  border: 0;
}

.work-creator__author.is-static { cursor: default; }
.work-creator__author > span:last-child { display: grid; min-width: 0; gap: 2px; }
.work-creator__author b { overflow: hidden; font-size: 13px; text-overflow: ellipsis; white-space: nowrap; }
.work-creator__author small { color: var(--sos-text-tertiary); font-size: 10px; }

.work-follow {
  display: inline-flex;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  min-width: 68px;
  min-height: 32px;
  padding: 0 15px;
  color: white;
  font: inherit;
  font-size: 12px;
  font-weight: 850;
  cursor: pointer;
  background: var(--sos-accent);
  border: 1px solid var(--sos-accent);
  border-radius: 999px;
}

.work-follow.is-following {
  color: var(--sos-text-secondary);
  background: color-mix(in srgb, var(--sos-bg-surface) 72%, transparent);
  border-color: var(--sos-border-subtle);
}

.work-follow:disabled { cursor: wait; opacity: 0.64; }

.work-creator__works {
  display: flex;
  gap: 8px;
  padding: 2px 2px 7px;
  overflow-x: auto;
  overscroll-behavior-inline: contain;
  scrollbar-width: thin;
  scroll-padding-inline: 2px;
  scroll-snap-type: x mandatory;
  touch-action: pan-x pan-y;
}

.work-creator__works button {
  position: relative;
  display: block;
  flex: 0 0 clamp(112px, 44%, 142px);
  aspect-ratio: 4 / 5;
  padding: 0;
  overflow: hidden;
  cursor: pointer;
  background: color-mix(in srgb, var(--sos-bg-muted) 74%, transparent);
  border: 1px solid color-mix(in srgb, var(--sos-accent) 28%, var(--sos-border-subtle));
  border-radius: 4px;
  scroll-snap-align: center;
}

.work-creator__works button.is-current {
  border-color: color-mix(in srgb, var(--sos-accent) 76%, white);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--sos-accent) 18%, transparent);
}

.work-creator__works button > span {
  position: absolute;
  inset: -12%;
  background-position: center;
  background-size: cover;
  filter: blur(12px) saturate(0.8);
  opacity: 0.4;
  transform: scale(1.08);
}

.work-creator__works img {
  position: relative;
  display: block;
  width: 100%;
  height: 100%;
  object-fit: contain;
  transition: transform 0.22s ease;
}

.work-creator__works button:hover img { transform: scale(1.025); }

.work-muted { color: var(--sos-text-tertiary) !important; font-size: 12px !important; }

.work-discovery,
.work-comments {
  margin-top: 38px;
  padding: 32px 34px 38px;
  background: color-mix(in srgb, var(--sos-bg-surface) 80%, transparent);
  border: 1px solid color-mix(in srgb, var(--sos-border-subtle) 82%, transparent);
  backdrop-filter: blur(14px);
}

.work-comments { scroll-margin-top: 88px; }
.work-discovery--related { margin-top: 28px; }
.work-section-head {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 20px;
  margin-bottom: 24px;
  padding-bottom: 19px;
  border-bottom: 1px solid var(--sos-border-subtle);
}

.work-section-head h2 { margin: 6px 0 0; font-size: clamp(23px, 2.4vw, 35px); font-weight: 950; letter-spacing: -0.03em; }
.work-section-head button,
.work-section-head > b,
.work-section-head > p {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  margin: 0;
  padding: 0;
  color: var(--sos-text-secondary);
  font: inherit;
  font-size: 12px;
  font-weight: 800;
  background: transparent;
  border: 0;
}
.work-section-head button { cursor: pointer; }

.work-comments__layout { display: grid; grid-template-columns: minmax(0, 1fr) minmax(300px, 390px); gap: 30px; align-items: start; }
.work-comment-list { display: grid; gap: 12px; }
.work-comment {
  display: grid;
  grid-template-columns: 38px minmax(0, 1fr);
  gap: 12px;
  padding: 17px 0;
  border-bottom: 1px solid var(--sos-border-subtle);
}
.work-comment__avatar {
  display: grid;
  width: 38px;
  height: 38px;
  place-items: center;
  color: var(--sos-text-secondary);
  font-size: 13px;
  font-weight: 900;
  background: color-mix(in srgb, var(--sos-accent) 11%, var(--sos-bg-muted));
  border-radius: 50%;
}
.work-comment__body header { display: flex; flex-wrap: wrap; align-items: baseline; gap: 8px; }
.work-comment__body header b { font-size: 13px; }
.work-comment__body time { color: var(--sos-text-tertiary); font-size: 10px; }
.work-comment__body p { margin: 7px 0 10px; color: var(--sos-text-secondary); font-size: 13px; line-height: 1.7; white-space: pre-wrap; }
.work-comment__body button { display: inline-flex; align-items: center; gap: 4px; padding: 0; color: var(--sos-text-tertiary); font: inherit; font-size: 11px; cursor: pointer; background: transparent; border: 0; }

.work-comment-empty { display: grid; min-height: 160px; place-items: center; align-content: center; gap: 5px; color: var(--sos-text-tertiary); text-align: center; border: 1px dashed var(--sos-border-subtle); }
.work-comment-empty strong { color: var(--sos-text-secondary); font-size: 13px; }
.work-comment-empty span { font-size: 11px; }

.work-compose {
  position: sticky;
  top: 92px;
  display: grid;
  gap: 11px;
  padding: 20px;
  background: color-mix(in srgb, var(--sos-bg-muted) 68%, var(--sos-bg-surface));
  border: 1px solid var(--sos-border-subtle);
  border-top: 3px solid var(--sos-accent);
}
.work-compose label { color: var(--sos-text-secondary); font-size: 12px; font-weight: 750; }
.work-compose textarea {
  width: 100%;
  box-sizing: border-box;
  padding: 12px;
  color: var(--sos-text-primary);
  font: inherit;
  font-size: 13px;
  line-height: 1.65;
  resize: vertical;
  background: var(--sos-bg-surface);
  border: 1px solid var(--sos-border-subtle);
  border-radius: 0;
  outline: none;
}
.work-compose textarea:focus { border-color: var(--sos-accent); box-shadow: 0 0 0 2px color-mix(in srgb, var(--sos-accent) 15%, transparent); }
.work-compose > div { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
.work-compose small { color: var(--sos-text-tertiary); font-size: 10px; }
.work-compose__notice { margin: 0; color: var(--sos-text-secondary); font-size: 11px; }
.work-compose--login strong { margin-top: 3px; font-size: 16px; }
.work-compose--login p { margin: 0; color: var(--sos-text-secondary); font-size: 12px; line-height: 1.6; }
.work-compose--login .work-button { margin-top: 5px; }

.work-state {
  display: grid;
  min-height: 58vh;
  place-items: center;
  align-content: center;
  gap: 8px;
  text-align: center;
  background: color-mix(in srgb, var(--sos-bg-surface) 90%, transparent);
  border: 1px solid var(--sos-border-subtle);
}
.work-state strong { font-size: 18px; }
.work-state small { color: var(--sos-text-tertiary); }
.work-state .work-button { margin-top: 12px; }
.work-state__scanner { width: 34px; height: 34px; margin-bottom: 8px; border: 2px solid color-mix(in srgb, var(--sos-accent) 20%, transparent); border-top-color: var(--sos-accent); border-radius: 50%; animation: work-scan 0.8s linear infinite; }
@keyframes work-scan { to { transform: rotate(1turn); } }

.image-viewer {
  position: fixed;
  inset: 0;
  z-index: 2000;
  display: grid;
  grid-template-columns: 64px minmax(0, 1fr) 64px;
  grid-template-rows: 58px minmax(0, 1fr);
  color: white;
  background: rgba(4, 9, 12, 0.97);
}
.image-viewer__bar { grid-column: 1 / -1; display: flex; align-items: center; justify-content: space-between; padding: 0 18px; border-bottom: 1px solid rgba(255, 255, 255, 0.12); }
.image-viewer__bar > span { font: 800 11px/1 ui-monospace, monospace; letter-spacing: 0.12em; }
.image-viewer__bar > div { display: flex; align-items: center; gap: 4px; }
.image-viewer__bar button,
.image-viewer__bar a,
.image-viewer__nav { display: grid; width: 40px; height: 40px; place-items: center; padding: 0; color: white; cursor: pointer; background: transparent; border: 0; }
.image-viewer__bar div > span { width: 50px; color: rgba(255, 255, 255, 0.62); font-size: 10px; text-align: center; }
.image-viewer__stage { grid-column: 2; grid-row: 2; overflow: auto; display: grid; place-items: center; padding: 24px; }
.image-viewer__stage img { display: block; max-width: 100%; max-height: calc(100vh - 106px); object-fit: contain; transform-origin: center; transition: transform 0.18s ease; }
.image-viewer__nav { align-self: center; justify-self: center; border: 1px solid rgba(255, 255, 255, 0.16); border-radius: 50%; }
.image-viewer__nav.is-prev { grid-column: 1; grid-row: 2; }
.image-viewer__nav.is-next { grid-column: 3; grid-row: 2; }

:global(html.art-image-viewer-open) { overflow: hidden; }

@media (max-width: 980px) {
  .work-page { width: min(100% - 36px, 820px); }
  .work-layout {
    position: static;
    grid-template-columns: 1fr;
    grid-template-areas:
      "canvas"
      "meta"
      "creator";
  }
  .work-creator { position: static; margin-top: 18px; }
  .work-comments__layout { grid-template-columns: 1fr; }
  .work-compose { position: static; }
}

@media (max-width: 680px) {
  .work-page { width: 100%; padding-top: 0; padding-bottom: 76px; }
  .work-path { min-height: 39px; padding-inline: 14px; font-size: 10px; }
  .work-button { min-height: 42px; padding-inline: 14px; }
  .work-layout { margin-top: 0; }
  .work-canvas { gap: 11px; padding: 0; border-inline: 0; }
  .work-canvas__summary { padding: 8px 12px; }
  .work-image__hint { display: none; }
  .work-meta { padding: 21px 16px 5px; }
  .work-meta h1 { font-size: clamp(23px, 7vw, 29px); }
  .work-meta__description { margin-top: 11px; font-size: 13px; }
  .work-tags { margin-top: 13px; }
  .work-meta__activity { gap: 14px; margin-top: 11px; }
  .work-meta__rule { margin-top: 17px; }
  .work-creator { width: calc(100% - 24px); box-sizing: border-box; margin: 18px auto 0; padding: 16px; }
  .work-creator__head { margin-bottom: 13px; }
  .work-discovery,
  .work-comments { margin: 30px 0 0; padding: 24px 12px 28px; border-inline: 0; }
  .work-discovery--related { margin-top: 12px; }
  .work-section-head { align-items: flex-start; margin-bottom: 17px; padding-bottom: 14px; }
  .work-section-head h2 { font-size: 24px; }
  .work-section-head > p { display: none; }
  .work-section-head button { max-width: 120px; justify-content: flex-end; text-align: right; }
  .work-comments__layout { gap: 20px; }
  .work-compose { padding: 17px; }
  .work-compose .work-button { min-width: 120px; }
  .image-viewer { grid-template-columns: 46px minmax(0, 1fr) 46px; grid-template-rows: 54px minmax(0, 1fr); }
  .image-viewer__bar { padding: 0 7px 0 12px; }
  .image-viewer__bar div > span,
  .image-viewer__bar button:nth-of-type(1),
  .image-viewer__bar button:nth-of-type(2) { display: none; }
  .image-viewer__stage { padding: 8px; }
  .image-viewer__stage img { max-height: calc(100vh - 70px); }
  .image-viewer__nav { width: 34px; height: 34px; background: rgba(0, 0, 0, 0.4); }
}

@media (max-width: 360px) {
  .work-meta__activity { gap: 10px; }
  .work-meta__tools a,
  .work-meta__tools button { width: 32px; }
  .work-follow { min-width: 62px; padding-inline: 12px; }
}

@media (prefers-reduced-motion: reduce) {
  .work-state__scanner { animation-duration: 1.8s; }
  .work-image__hint,
  .image-viewer__stage img { transition: none; }
}
</style>
