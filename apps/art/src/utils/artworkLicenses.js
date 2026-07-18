export const PUBLIC_LICENSE_OPTIONS = Object.freeze([
  '可在b站、小红书等社交媒体转载',
  '允许用于视频等个人创作',
  '允许用于制作无料发放',
])

function licenseValues(artwork) {
  if (!Array.isArray(artwork?.licenses)) return []
  return artwork.licenses
    .filter((license) => typeof license === 'string')
    .map((license) => license.trim())
    .filter(Boolean)
}

export function publicLicenseLabels(artwork) {
  const labels = licenseValues(artwork).flatMap((license) => {
    if (license.startsWith('NET:')) {
      const label = license.slice('NET:'.length).trim()
      return label ? [label] : []
    }

    // 兼容引入 NET:/GROUP: 前保存的旧授权数据。
    return PUBLIC_LICENSE_OPTIONS.includes(license) ? [license] : []
  })

  return [...new Set(labels)]
}

export function hasPublicDownloadLicense(artwork) {
  return publicLicenseLabels(artwork).length > 0
}

export function groupLicenseLabels(artwork) {
  const labels = licenseValues(artwork).flatMap((license) => {
    if (!license.startsWith('GROUP:')) return []
    const label = license.slice('GROUP:'.length).trim()
    return label ? [label] : []
  })

  return [...new Set(labels)]
}
