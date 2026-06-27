// 投稿草稿暂存：未登录用户在投稿页填到一半时，可点「登录 / 注册」去登录。
// art 的登录是同一 SPA 内的 /login 路由（不会整页刷新），因此组件虽然卸载，
// 已选图片的 File 对象与其 createObjectURL 预览地址在内存中仍然有效。
// 这里用一个模块级变量在「去登录 → 回跳投稿页」期间暂存整份草稿，回来后原样恢复。
// 仅存活于内存：整页刷新或关闭标签页即失效（与"出去登录再回来"的场景预期一致）。
let stash = null

export function saveUploadDraft(data) {
  stash = data
}

export function takeUploadDraft() {
  return stash
}

export function clearUploadDraft() {
  stash = null
}
