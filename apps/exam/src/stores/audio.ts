import { defineStore } from 'pinia';
import { ref } from 'vue';

// 定义回退尝试的后缀列表
const FALLBACK_EXTS = [
  '.mp3',
  '.m4a',
  '.aac',
  '.webm',
  '.ogg',
  '.opus',
  '.wav',
  '.flac'
];

export const useAudioStore = defineStore('audio', () => {
  const currentId = ref<string | null>(null);
  const playing = ref(false);
  const instance = ref<HTMLAudioElement | null>(null);

  // 进度状态
  const progress = ref(0);
  const currentTime = ref(0);
  const duration = ref(0);

  /**
   * 会话令牌：每次 stop / toggle 都会推进，用于“取消旧回退链”
   * 任何异步回调只要发现 token 不匹配，就直接退出，避免超多请求/逻辑复活
   */
  const sessionToken = ref(0);

  // 彻底清理一个 audio：停止播放 + 取消网络加载 + 清空回调
  const cleanupAudio = (audio: HTMLAudioElement | null) => {
    if (!audio) return;

    try {
      audio.pause();
      // 取消后续事件回调
      audio.onloadedmetadata = null;
      audio.ontimeupdate = null;
      audio.onended = null;
      audio.onerror = null;

      // 关键：取消正在进行的资源加载（避免旧 audio 继续触发 error/重试）
      audio.removeAttribute('src');
      audio.load();
    } catch {
      // ignore
    }
  };

  // 停止播放并重置
  const stop = () => {
    // 先推进 token，立即“熄火”所有旧链路回调
    sessionToken.value++;

    if (instance.value) {
      cleanupAudio(instance.value);
      instance.value = null;
    }

    currentId.value = null;
    playing.value = false;
    progress.value = 0;
    currentTime.value = 0;
    duration.value = 0;
  };

  // 切换播放状态
  const toggle = (src: string, id: string) => {
    // 如果点击的是当前正在播放的音频，则暂停（toggle off）
    if (currentId.value === id && playing.value) {
      stop();
      return;
    }

    stop(); // 停止其他正在播放的音频（也会推进 token）

    if (src.startsWith('tone:')) {
      console.warn('Synthesized tone is deprecated and removed.');
      return;
    }

    currentId.value = id;
    playing.value = true;

    // 生成候选列表并开始播放
    const candidates = generateCandidates(src);

    // 用当前 token 作为本次播放会话
    const token = sessionToken.value;
    playWithFallback(candidates, id, token, 0);
  };

  /**
   * 更稳健的候选生成：
   * - 保留 ?query / #hash
   * - 只对 pathname 的扩展名动刀（不会把 query 里的点号当扩展名）
   * - 去重，保证每个候选只出现一次
   */
  const generateCandidates = (originalSrc: string): string[] => {
    const list: string[] = [];

    // 允许相对路径：用当前页面作为 base
    let url: URL | null = null;
    try {
      url = new URL(originalSrc, window.location.href);
    } catch {
      url = null;
    }

    // 如果 URL 解析失败，就退回原始逻辑（但尽量少出错）
    if (!url) {
      const lastDotIndex = originalSrc.lastIndexOf('.');
      const basePath = lastDotIndex > -1 ? originalSrc.substring(0, lastDotIndex) : originalSrc;
      const originalExt = lastDotIndex > -1 ? originalSrc.substring(lastDotIndex) : '';

      const raw = [originalSrc];
      FALLBACK_EXTS.forEach(ext => {
        if (ext !== originalExt) raw.push(`${basePath}${ext}`);
      });

      // 去重
      return Array.from(new Set(raw));
    }

    const pathname = url.pathname; // 不含 query/hash
    const search = url.search || '';
    const hash = url.hash || '';

    // 扩展名只看最后一个 / 后面的部分
    const lastSlash = pathname.lastIndexOf('/');
    const filePart = lastSlash >= 0 ? pathname.slice(lastSlash + 1) : pathname;
    const dotInFile = filePart.lastIndexOf('.');

    const hasExt = dotInFile > 0; // ".bashrc" 这种不当作扩展名
    const originalExt = hasExt ? filePart.slice(dotInFile) : '';

    const basePathname = hasExt ? pathname.slice(0, pathname.length - originalExt.length) : pathname;

    // 1) 原始 URL 优先
    list.push(url.toString());

    // 2) 追加其他格式：basePathname + ext + 原 query/hash
    for (const ext of FALLBACK_EXTS) {
      if (ext === originalExt) continue;

      const u = new URL(url.toString());
      u.pathname = `${basePathname}${ext}`;
      u.search = search;
      u.hash = hash;
      list.push(u.toString());
    }

    // 去重并返回
    return Array.from(new Set(list));
  };

  const isNotSupportedPlayError = (e: unknown) => {
    const name = (e as any)?.name || '';
    const msg = String((e as any)?.message || '');
    return (
      name === 'NotSupportedError' ||
      msg.includes('NotSupportedError') ||
      msg.includes('no supported sources') ||
      msg.includes('The element has no supported sources')
    );
  };

  /**
   * 顺序回退播放（严格单链路、单次推进）
   * @param candidates URL 候选列表
   * @param targetId 当前期望播放的 ID
   * @param token 本次播放会话 token（用于取消旧链路）
   * @param index 当前尝试的索引
   */
  const playWithFallback = (
    candidates: string[],
    targetId: string,
    token: number,
    index = 0
  ) => {
    // 会话已失效（stop 或切歌）
    if (sessionToken.value !== token) return;

    // 用户切歌/停止
    if (currentId.value !== targetId) return;

    // 尝试完毕
    if (index >= candidates.length) {
      console.error('All audio candidates failed for ID:', targetId);
      if (currentId.value === targetId && sessionToken.value === token) {
        playing.value = false;
        currentId.value = null;
        instance.value = null;
        progress.value = 0;
        currentTime.value = 0;
        duration.value = 0;
      }
      return;
    }

    const src = candidates[index];

    // 启动新尝试前，清理旧实例（避免旧 onerror 继续触发造成“超多请求”）
    if (instance.value) {
      cleanupAudio(instance.value);
      instance.value = null;
    }

    // 每次尝试先重置进度（避免 UI 混乱）
    progress.value = 0;
    currentTime.value = 0;
    duration.value = 0;

    const audio = new Audio(src);
    audio.preload = 'metadata';
    instance.value = audio;

    // 只允许推进一次（防止 onerror + play.catch 双触发）
    let advanced = false;
    const advance = () => {
      if (advanced) return;
      advanced = true;

      // 会话已失效/用户切歌就不推进
      if (sessionToken.value !== token) return;
      if (currentId.value !== targetId) return;

      console.warn(`Audio load/play failed for ${src}, trying next format...`);
      cleanupAudio(audio);

      playWithFallback(candidates, targetId, token, index + 1);
    };

    audio.onloadedmetadata = () => {
      if (sessionToken.value !== token) return;
      if (currentId.value === targetId) {
        duration.value = audio.duration || 0;
      }
    };

    audio.ontimeupdate = () => {
      if (sessionToken.value !== token) return;
      if (currentId.value !== targetId || !audio.duration) return;
      currentTime.value = audio.currentTime;
      progress.value = (audio.currentTime / audio.duration) * 100;
    };

    audio.onended = () => {
      if (sessionToken.value !== token) return;
      if (currentId.value === targetId) {
        playing.value = false;
        currentId.value = null;
        instance.value = null;
        progress.value = 0;
        currentTime.value = 0;
      }
    };

    // 失败回退：资源错误
    audio.onerror = () => {
      advance();
    };

    // 启动播放
    audio.play().catch((e) => {
      // 典型：用户手势限制（NotAllowedError）——不要疯狂回退
      if (sessionToken.value !== token) return;
      if (currentId.value !== targetId) return;

      const name = (e as any)?.name || '';
      if (name === 'NotAllowedError') {
        console.warn('Playback blocked by browser policy:', src, e);
        // 这类情况不应回退尝试（否则会造成多余请求），直接停止播放态
        playing.value = false;
        return;
      }

      // 不支持格式时，有些浏览器只在这里报错，不触发 onerror
      if (isNotSupportedPlayError(e) || audio.error) {
        advance();
        return;
      }

      console.warn('Playback failed (non-retryable):', src, e);
      playing.value = false;
    });
  };

  return {
    currentId,
    playing,
    instance,
    progress,
    currentTime,
    duration,
    stop,
    toggle
  };
});
