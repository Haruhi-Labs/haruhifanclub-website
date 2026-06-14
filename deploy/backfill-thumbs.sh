#!/usr/bin/env bash
# ============================================================
# 画廊缩略图离线预生成（backfill）。
#
# 把 uploads/art/ 下的存量图片一次性生成三档宽度缩略图，落到
#   uploads/art/.thumbs/<w>/<sub>.<srcext>.webp
# 路径布局与后端 thumb_cache_path / 前端 thumbUrl 完全一致，因此预热后
# nginx 直接静态命中、后端 /api/art/thumb 几乎不再被回源。
#
# 用 libvips（vips thumbnail，流式 shrink-on-load、内存有界），串行执行，
# 即便存量里有几十 MB 的巨图也不会把内存打爆——这正是放在请求路径外做的意义。
#
# 用法（服务器上执行；幂等，已存在的缩略图自动跳过）：
#   HARUHI_ROOT=/var/www/haruhifanclub bash deploy/backfill-thumbs.sh
# 前置：apt-get install -y libvips-tools
# ============================================================
set -euo pipefail

ROOT="${HARUHI_ROOT:-/var/www/haruhifanclub}"
ART="$ROOT/uploads/art"
WIDTHS=(320 640 960)
QUALITY=82

command -v vips >/dev/null 2>&1 || {
  echo "✗ 未找到 vips，请先安装：apt-get install -y libvips-tools" >&2
  exit 1
}
[ -d "$ART" ] || { echo "✗ 目录不存在：$ART" >&2; exit 1; }

made=0 skipped=0 failed=0
# 源图：art/ 下的位图（webp/jpg/jpeg/png），排除 .thumbs 自身与 gif/svg（不转码）
while IFS= read -r -d '' src; do
  rel="${src#"$ART"/}" # 例 2026-06/x.webp
  for w in "${WIDTHS[@]}"; do
    dst="$ART/.thumbs/$w/$rel.webp"
    if [ -f "$dst" ]; then
      skipped=$((skipped + 1))
      continue
    fi
    mkdir -p "$(dirname "$dst")"
    # 原子落盘：先写临时文件再 mv，避免中断/失败留下半截 webp 被"存在即跳过"永久供损坏图
    tmp="${dst}.tmp.$$"
    if vips thumbnail "$src" "${tmp}[Q=${QUALITY},strip]" "$w" --size down 2>/dev/null \
      && mv -f "$tmp" "$dst"; then
      made=$((made + 1))
    else
      rm -f "$tmp"
      failed=$((failed + 1))
      echo "  ✗ 生成失败：$rel @ ${w}" >&2
    fi
  done
done < <(find "$ART" -type f \
  \( -iname '*.webp' -o -iname '*.jpg' -o -iname '*.jpeg' -o -iname '*.png' \) \
  -not -path '*/.thumbs/*' -print0)

echo "✓ backfill 完成：新生成 ${made}，跳过 ${skipped}（已存在），失败 ${failed}"
[ "$failed" -eq 0 ]
