import { DatabaseSync } from 'node:sqlite'
import { resolve } from 'node:path'

const sourceOrigin = String(process.env.ART_PUBLIC_SOURCE || 'https://haruyuki.cn').replace(/\/$/, '')
const databasePath = resolve(process.env.HARUHI_ART_DB || 'data/art.db')
const pageSize = 60

function relativeUploadPath(value) {
  return String(value || '')
    .replace(/^https?:\/\/[^/]+\/uploads\//i, '')
    .replace(/^\/?uploads\//i, '')
    .replace(/^\/+/, '')
}

function normalizedImages(artwork) {
  const source = Array.isArray(artwork.images) && artwork.images.length
    ? artwork.images
    : [{ image_url: artwork.image_url, original_url: artwork.original_url }]
  return source
    .map(image => ({
      path: relativeUploadPath(image.image_url || image.path),
      original: relativeUploadPath(image.original_url || image.original || image.image_url || image.path),
      ...(Number(image.width) > 0 ? { width: Number(image.width) } : {}),
      ...(Number(image.height) > 0 ? { height: Number(image.height) } : {}),
    }))
    .filter(image => image.path)
}

async function fetchPublicArtworks() {
  const artworks = []
  let page = 1
  let total = Number.POSITIVE_INFINITY
  while (artworks.length < total) {
    const url = new URL('/api/art/artworks', sourceOrigin)
    url.searchParams.set('status', 'approved')
    url.searchParams.set('sort', 'time')
    url.searchParams.set('page', String(page))
    url.searchParams.set('pageSize', String(pageSize))
    const response = await fetch(url)
    if (!response.ok) throw new Error(`公开画廊读取失败：HTTP ${response.status}`)
    const payload = await response.json()
    const items = Array.isArray(payload.data) ? payload.data : []
    artworks.push(...items)
    total = Number(payload.total || artworks.length)
    if (!items.length || items.length < pageSize) break
    page += 1
  }
  return artworks
}

function syncSnapshot(artworks) {
  const database = new DatabaseSync(databasePath)
  database.exec('PRAGMA foreign_keys=ON')
  const migrationReady = database
    .prepare("SELECT COUNT(1) AS count FROM sqlite_master WHERE type='table' AND name='artwork_favorites'")
    .get()
  if (!migrationReady?.count) {
    database.close()
    throw new Error('本地画廊迁移尚未完成，请先启动后端后再同步。')
  }

  const upsertArtwork = database.prepare(`
    INSERT INTO artworks(
      id, title, description, uploader_name, uploader_uid, source_type, content_type,
      tags_json, tags_norm, origin_url, file_path, file_path_original, status,
      review_note, reviewed_at, created_at, licenses_json, like_total, images_json,
      ai_reason, random_key, exhibit_enabled
    ) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)
    ON CONFLICT(id) DO UPDATE SET
      title=excluded.title,
      description=excluded.description,
      uploader_name=excluded.uploader_name,
      uploader_uid=excluded.uploader_uid,
      source_type=excluded.source_type,
      content_type=excluded.content_type,
      tags_json=excluded.tags_json,
      tags_norm=excluded.tags_norm,
      origin_url=excluded.origin_url,
      file_path=excluded.file_path,
      file_path_original=excluded.file_path_original,
      status=excluded.status,
      reviewed_at=excluded.reviewed_at,
      created_at=excluded.created_at,
      licenses_json=excluded.licenses_json,
      like_total=excluded.like_total,
      images_json=excluded.images_json,
      random_key=excluded.random_key
  `)
  const upsertCreator = database.prepare(`
    INSERT INTO creators(uid, avatar_url, created_at)
    VALUES(?,?,?)
    ON CONFLICT(uid) DO UPDATE SET
      avatar_url=CASE WHEN excluded.avatar_url<>'' THEN excluded.avatar_url ELSE creators.avatar_url END
  `)

  database.exec('BEGIN IMMEDIATE')
  try {
    for (const artwork of artworks) {
      const images = normalizedImages(artwork)
      const first = images[0] || { path: '', original: '' }
      const tags = Array.isArray(artwork.tags) ? artwork.tags.map(String) : []
      const licenses = Array.isArray(artwork.licenses) ? artwork.licenses.map(String) : []
      const uploaderUid = String(artwork.uploader_uid || '').trim()
      upsertArtwork.run(
        Number(artwork.id),
        artwork.title || null,
        artwork.description || null,
        artwork.uploader_name || null,
        uploaderUid || null,
        artwork.source_type || null,
        artwork.content_type || null,
        JSON.stringify(tags),
        tags.length ? ` ${tags.join(' ')} ` : '',
        artwork.origin_url || null,
        first.path || null,
        first.original || first.path || null,
        'approved',
        null,
        artwork.reviewed_at || null,
        artwork.created_at || null,
        JSON.stringify(licenses),
        Number(artwork.like_total || 0),
        JSON.stringify(images),
        null,
        Math.abs(Math.imul(Number(artwork.id), 1103515245)) % 2147483647,
        artwork.exhibit_enabled == null ? null : Number(Boolean(artwork.exhibit_enabled)),
      )
      if (uploaderUid) {
        upsertCreator.run(
          uploaderUid,
          relativeUploadPath(artwork.uploader_avatar),
          artwork.created_at || new Date().toISOString(),
        )
      }
    }
    database.exec('COMMIT')
  } catch (error) {
    database.exec('ROLLBACK')
    database.close()
    throw error
  }
  const approved = database.prepare("SELECT COUNT(1) AS count FROM artworks WHERE status='approved'").get()
  database.close()
  return Number(approved?.count || 0)
}

try {
  const artworks = await fetchPublicArtworks()
  const localCount = syncSnapshot(artworks)
  process.stdout.write(`✓ 已同步 ${artworks.length} 件真实公开作品；本地可交互画廊共 ${localCount} 件作品。\n`)
} catch (error) {
  process.stderr.write(`✗ ${error?.message || error}\n`)
  process.exitCode = 1
}
