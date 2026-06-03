#!/usr/bin/env bash
# ============================================================
# 据 deploy/env.sample 渲染一份填好强密钥的环境变量文件。
# 把三处 change-me 占位（HARUHI_JWT_SECRET / ART_COOKIE_SECRET /
# HARUHI_SUPERADMIN_PASSWORD）替换为 openssl 随机值，其余原样保留。
#
# 用法：
#   bash deploy/gen-secrets.sh                 # 写到仓库根 .env（已存在则拒绝覆盖）
#   bash deploy/gen-secrets.sh /etc/haruhifanclub.env   # 指定输出
#   bash deploy/gen-secrets.sh --force         # 允许覆盖已存在文件
#
# 生成后：本地放仓库根 .env；生产拷到 /etc/haruhifanclub.env（脚本已 chmod 600）。
# DASHSCOPE_API_KEY、邮件(RESEND/SMTP) 等可选项仍需按环境自行填写。
# ============================================================
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
template="$here/env.sample"

out=".env"
force=0
for a in "$@"; do
  case "$a" in
    --force) force=1 ;;
    -h | --help)
      sed -n '2,18p' "$0"
      exit 0
      ;;
    -*)
      echo "未知参数：$a" >&2
      exit 2
      ;;
    *) out="$a" ;;
  esac
done

command -v openssl >/dev/null 2>&1 || {
  echo "✗ 需要 openssl（用于生成随机密钥）" >&2
  exit 1
}
[ -f "$template" ] || {
  echo "✗ 找不到模板：$template" >&2
  exit 1
}
if [ -e "$out" ] && [ "$force" -ne 1 ]; then
  echo "✗ $out 已存在，未覆盖（避免覆盖现有密钥）。如确需重新生成请加 --force。" >&2
  exit 1
fi

# 密钥：hex32 = 64 位十六进制，纯 [0-9a-f]，对 dotenvy/systemd/awk 均安全。
jwt="$(openssl rand -hex 32)"
cookie="$(openssl rand -hex 32)"
# 超管口令：base64(24B) 去掉易混/特殊字符，取前 24 位。
adminpw="$(openssl rand -base64 24 | tr -dc 'A-Za-z0-9' | cut -c1-24)"

# 据模板渲染（按 key 锚定整行替换；生成值不含 awk 特殊字符，直接字符串拼接安全）。
awk -v jwt="$jwt" -v cookie="$cookie" -v pw="$adminpw" '
  /^HARUHI_JWT_SECRET=/          { print "HARUHI_JWT_SECRET=" jwt; next }
  /^ART_COOKIE_SECRET=/          { print "ART_COOKIE_SECRET=" cookie; next }
  /^HARUHI_SUPERADMIN_PASSWORD=/ { print "HARUHI_SUPERADMIN_PASSWORD=" pw; next }
  { print }
' "$template" >"$out"

chmod 600 "$out"

echo "✓ 已生成 ${out}（权限 600）"
echo "  · HARUHI_JWT_SECRET、ART_COOKIE_SECRET 已用 openssl rand -hex 32 填充"
echo "  · 超级管理员账号：${HARUHI_SUPERADMIN_USER:-admin}"
echo "  · 超级管理员口令（请立即妥善保存，仅首次启动 seed 时使用）：$adminpw"
echo ""
echo "下一步："
echo "  本地：保持在仓库根，cargo run -p haruhi-server 会经 dotenvy 自动加载 .env"
echo "  生产：拷到 /etc/haruhifanclub.env（已是 600），按需改 HARUHI_BIND / HARUHI_CORS_ORIGINS / 邮件等"
echo "  仍需按环境填写：DASHSCOPE_API_KEY（AI 审核）、RESEND_API_KEY 或 SMTP_*（邮件）"
