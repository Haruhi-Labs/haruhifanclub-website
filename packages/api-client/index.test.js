// @haruhi/api-client 纯逻辑单元测试（vitest）。
// 覆盖被全部 6 个 app 依赖的共享层中、最易出错的两块：
// 上传 URL 拼接（曾出过双前缀 404）与前端 RBAC 镜像（hasScope 向上继承 / hasPerm 向下含子作用域）。
import { describe, it, expect } from 'vitest'
import { resolveUploadUrl, hasScope, createAdminAuth } from './index.js'

describe('resolveUploadUrl', () => {
  it('空值 → 空串', () => {
    expect(resolveUploadUrl('')).toBe('')
    expect(resolveUploadUrl(null)).toBe('')
    expect(resolveUploadUrl(undefined)).toBe('')
  })
  it('绝对 URL / blob / data 原样返回', () => {
    expect(resolveUploadUrl('http://x/y.png')).toBe('http://x/y.png')
    expect(resolveUploadUrl('https://a.b/c.webp')).toBe('https://a.b/c.webp')
    expect(resolveUploadUrl('blob:abc')).toBe('blob:abc')
    expect(resolveUploadUrl('data:image/png;base64,xx')).toBe('data:image/png;base64,xx')
  })
  it('站内绝对路径原样返回', () => {
    expect(resolveUploadUrl('/uploads/a.webp')).toBe('/uploads/a.webp')
    expect(resolveUploadUrl('/other/p.png')).toBe('/other/p.png')
  })
  it('去掉前导 uploads/ 防 /uploads/uploads 双前缀', () => {
    expect(resolveUploadUrl('uploads/art/2025-11/x.webp')).toBe('/uploads/art/2025-11/x.webp')
  })
  it('相对路径拼成 /uploads/<rel>', () => {
    expect(resolveUploadUrl('novel/covers/001.png')).toBe('/uploads/novel/covers/001.png')
    expect(resolveUploadUrl('art/x.webp')).toBe('/uploads/art/x.webp')
  })
  it('自定义 base', () => {
    expect(resolveUploadUrl('a/b.png', '/cdn')).toBe('/cdn/a/b.png')
  })
})

describe('hasScope（拥有作用域或其任一父级或超管即有权）', () => {
  it('null / 无 apps → false', () => {
    expect(hasScope(null, 'news')).toBe(false)
    expect(hasScope(undefined, 'news')).toBe(false)
    expect(hasScope({}, 'news')).toBe(false)
    expect(hasScope({ apps: {} }, 'news')).toBe(false)
  })
  it('超管恒 true', () => {
    expect(hasScope({ isSuperAdmin: true }, 'anything')).toBe(true)
    expect(hasScope({ isSuperAdmin: true, apps: {} }, 'news.activity')).toBe(true)
  })
  it('精确命中', () => {
    expect(hasScope({ apps: { news: {} } }, 'news')).toBe(true)
  })
  it('父作用域继承到子（持有 news → 对 news.activity 有效）', () => {
    expect(hasScope({ apps: { news: {} } }, 'news.activity')).toBe(true)
    expect(hasScope({ apps: { 'a.b': {} } }, 'a.b.c')).toBe(true)
  })
  it('子不授父、兄弟隔离', () => {
    expect(hasScope({ apps: { 'news.activity': {} } }, 'news')).toBe(false)
    expect(hasScope({ apps: { 'news.activity': {} } }, 'news.blog')).toBe(false)
  })
})

describe('createAdminAuth(app).hasPerm（拥有该 app、其任一子作用域或超管即可进后台）', () => {
  const auth = createAdminAuth('news')
  it('null → false', () => {
    expect(auth.hasPerm(null)).toBe(false)
    expect(auth.hasPerm({})).toBe(false)
  })
  it('超管 → true', () => {
    expect(auth.hasPerm({ isSuperAdmin: true })).toBe(true)
  })
  it('精确 app → true', () => {
    expect(auth.hasPerm({ apps: { news: {} } })).toBe(true)
  })
  it('持有任一子作用域 → true（news.activity 可进 news 后台）', () => {
    expect(auth.hasPerm({ apps: { 'news.activity': {} } })).toBe(true)
  })
  it('无关 app → false', () => {
    expect(auth.hasPerm({ apps: { art: {} } })).toBe(false)
  })
})
