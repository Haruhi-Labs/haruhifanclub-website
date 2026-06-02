/**
 * 浏览器端图片压缩工具
 * @param {File} file - 原始文件对象
 * @param {number} quality - 压缩质量 (0.1 - 1.0), 默认 0.9
 * @param {number} maxWidth - 最大宽度 (可选，默认不限制)
 * @returns {Promise<Blob>} - 返回 WebP 格式的 Blob
 */
export async function compressToWebP(file, quality = 0.9, maxWidth = 0) {
  return new Promise((resolve, reject) => {
    // 1. 读取文件
    const reader = new FileReader()
    reader.readAsDataURL(file)
    reader.onload = (event) => {
      const img = new Image()
      img.src = event.target.result
      
      img.onload = () => {
        // 2. 创建 Canvas
        let w = img.width
        let h = img.height
        
        // 缩放逻辑 (如果指定了最大宽度)
        if (maxWidth > 0 && w > maxWidth) {
          const ratio = maxWidth / w
          w = maxWidth
          h = h * ratio
        }

        const canvas = document.createElement('canvas')
        canvas.width = w
        canvas.height = h
        const ctx = canvas.getContext('2d')
        
        // 3. 绘制并导出 WebP
        ctx.drawImage(img, 0, 0, w, h)
        
        canvas.toBlob((blob) => {
          if (blob) {
            resolve(blob)
          } else {
            reject(new Error('Canvas toBlob failed'))
          }
        }, 'image/webp', quality)
      }
      
      img.onerror = (err) => reject(err)
    }
    reader.onerror = (err) => reject(err)
  })
}