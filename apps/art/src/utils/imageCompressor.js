/**
 * 浏览器端图片压缩工具：Canvas 负责解码和缩放，Web Worker 中的 Squoosh
 * libwebp WASM 负责编码，避免压缩大图时阻塞页面主线程。
 * @param {File} file - 原始文件对象
 * @param {number} quality - 压缩质量 (0.1 - 1.0), 默认 0.9
 * @param {number} maxWidth - 最大宽度 (可选，默认不限制)
 * @returns {Promise<Blob>} - 返回 WebP 格式的 Blob
 */
export async function compressToWebP(file, quality = 0.9, maxWidth = 0) {
  const source = await loadImageSource(file)
  try {
    let w = source.width
    let h = source.height

    if (!w || !h) throw new Error('图片尺寸读取失败')

    if (maxWidth > 0 && w > maxWidth) {
      const ratio = maxWidth / w
      w = maxWidth
      h = Math.round(h * ratio)
    }

    const canvas = createCanvas(w, h)
    const ctx = canvas.getContext('2d')
    if (!ctx) throw new Error('Canvas 初始化失败')

    ctx.drawImage(source.image, 0, 0, w, h)
    const imageData = ctx.getImageData(0, 0, w, h)
    const encoded = await encodeWithSquoosh(imageData, normalizeQuality(quality))
    return new Blob([encoded], { type: 'image/webp' })
  } finally {
    source.cleanup()
  }
}

let encoderWorker = null
let nextJobId = 0
const pendingJobs = new Map()
const ENCODE_TIMEOUT_MS = 30_000

function encodeWithSquoosh(imageData, quality) {
  const worker = getEncoderWorker()
  const id = ++nextJobId
  const pixels = imageData.data.buffer

  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      if (!pendingJobs.has(id)) return
      stopEncoderWorker(new Error('WebP 编码超时'))
    }, ENCODE_TIMEOUT_MS)
    const job = {
      resolve(value) {
        clearTimeout(timer)
        resolve(value)
      },
      reject(error) {
        clearTimeout(timer)
        reject(error)
      },
    }

    pendingJobs.set(id, job)
    try {
      worker.postMessage(
        {
          id,
          pixels,
          width: imageData.width,
          height: imageData.height,
          quality,
        },
        [pixels]
      )
    } catch (error) {
      pendingJobs.delete(id)
      job.reject(error instanceof Error ? error : new Error(String(error)))
    }
  })
}

function getEncoderWorker() {
  if (encoderWorker) return encoderWorker

  const worker = new Worker(new URL('./webpEncoder.worker.js', import.meta.url), {
    type: 'module',
  })
  encoderWorker = worker
  worker.addEventListener('message', ({ data }) => {
    const job = pendingJobs.get(data.id)
    if (!job) return

    pendingJobs.delete(data.id)
    if (data.error) {
      job.reject(new Error(data.error))
    } else {
      job.resolve(data.encoded)
    }
  })
  worker.addEventListener('error', (event) => {
    if (encoderWorker !== worker) return
    const error = new Error(event.message || 'WebP 编码线程异常')
    stopEncoderWorker(error)
  })

  return worker
}

function stopEncoderWorker(error) {
  for (const job of pendingJobs.values()) job.reject(error)
  pendingJobs.clear()
  encoderWorker?.terminate()
  encoderWorker = null
}

export function disposeEncoderWorker() {
  stopEncoderWorker(new Error('WebP 编码线程已释放'))
}

function normalizeQuality(quality) {
  const normalized = Number.isFinite(quality) ? quality : 0.9
  return Math.round(Math.min(1, Math.max(0, normalized)) * 100)
}

async function loadImageSource(file) {
  if (typeof createImageBitmap === 'function') {
    try {
      const bitmap = await createImageBitmap(file, { imageOrientation: 'from-image' })
      return imageBitmapSource(bitmap)
    } catch {
      try {
        const bitmap = await createImageBitmap(file)
        return imageBitmapSource(bitmap)
      } catch {
        // 部分浏览器对 HEIC/特殊 JPEG 的 ImageBitmap 支持不完整，继续走 object URL 回退。
      }
    }
  }

  const url = URL.createObjectURL(file)
  try {
    const img = new Image()
    img.decoding = 'async'
    await new Promise((resolve, reject) => {
      img.onload = resolve
      img.onerror = reject
      img.src = url
    })
    return {
      image: img,
      width: img.naturalWidth || img.width,
      height: img.naturalHeight || img.height,
      cleanup: () => URL.revokeObjectURL(url),
    }
  } catch (err) {
    URL.revokeObjectURL(url)
    throw err
  }
}

function imageBitmapSource(bitmap) {
  return {
    image: bitmap,
    width: bitmap.width,
    height: bitmap.height,
    cleanup: () => bitmap.close?.(),
  }
}

function createCanvas(width, height) {
  if (typeof OffscreenCanvas !== 'undefined') {
    return new OffscreenCanvas(width, height)
  }
  const canvas = document.createElement('canvas')
  canvas.width = width
  canvas.height = height
  return canvas
}
