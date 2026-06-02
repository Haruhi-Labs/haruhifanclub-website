/**
 * 压缩图片为 WebP 格式
 * @param file 原始 File 对象
 * @param quality 压缩质量 0-1, 默认 0.75
 * @returns Promise<File> 返回压缩后的 File 对象
 */
export async function compressImage(file: File, quality = 0.75): Promise<File> {
  // 如果不是图片，直接返回
  if (!file.type.startsWith('image/')) return file;

  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.readAsDataURL(file);
    reader.onload = (e) => {
      const img = new Image();
      img.src = e.target?.result as string;
      img.onload = () => {
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');
        if (!ctx) return reject('Canvas context unsupported');

        // 保持原始尺寸
        canvas.width = img.width;
        canvas.height = img.height;

        // 绘制图片
        ctx.drawImage(img, 0, 0, canvas.width, canvas.height);

        // 导出为 WebP
        canvas.toBlob((blob) => {
          if (!blob) return reject('Compression failed');
          // 创建新文件，修改后缀
          const newName = file.name.replace(/\.[^/.]+$/, "") + ".webp";
          const newFile = new File([blob], newName, { type: 'image/webp' });
          resolve(newFile);
        }, 'image/webp', quality);
      };
      img.onerror = (err) => reject(err);
    };
    reader.onerror = (err) => reject(err);
  });
}