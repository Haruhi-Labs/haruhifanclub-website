/**
 * 浏览器端图片压缩工具
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
    return await canvasToWebP(canvas, quality)
  } finally {
    source.cleanup()
  }
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

function canvasToWebP(canvas, quality) {
  if (typeof canvas.convertToBlob === 'function') {
    return canvas.convertToBlob({ type: 'image/webp', quality })
  }

  return new Promise((resolve, reject) => {
    canvas.toBlob((blob) => {
      if (blob) resolve(blob)
      else reject(new Error('Canvas 导出 WebP 失败'))
    }, 'image/webp', quality)
  })
}
