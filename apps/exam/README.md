# @haruhi/exam · 在线考试平台

春日应援团「凉宫春日入团测试」风格的在线考试平台：用户在线作答、批改、按分数评定等级并把成绩/试卷导出为图片分享；同时提供试卷在线编辑（带 `edit_token` 免登录编辑链接）与管理员审核后台。本 app 是全仓唯一使用 **TypeScript** 的前端样板。

## 功能特性

- **在线作答与批改** — 单选 / 多选 / 判断 / 填空题，作答本地存档（刷新可恢复），交卷自动批改
- **等级评定** — 按总分落入配置的分数区间评定等级（名称 / 颜色 / 头像 / 评语）
- **成绩与试卷分享** — 成绩单及完整试卷（正反双页拼版）用 `html-to-image` 导出高清 PNG，并生成二维码分享链接
- **多媒体题目** — 题干 / 解析支持文字 + 图片（WebP）+ 音频（后端 ffmpeg 转 MP3）内容块自由组合
- **在线编辑** — 可视化编辑试卷（配置 / 题目 / 等级三大面板），`edit_token` 免登录编辑链接，支持 JSON 导入 / 导出
- **管理后台** — 试卷审核（通过 / 锁定 / 删除）与统计（统一 JWT 登录 + RBAC）

## 技术栈与关键依赖

- Vue 3（`<script setup>`）+ Vue Router 4 + Pinia 3
- **TypeScript** + **SCSS**（Sass `modern-compiler` API）
- 构建工具 Vite 7，构建前置类型检查 `vue-tsc --noEmit`
- [`qrcode`](https://www.npmjs.com/package/qrcode)：把试卷/分享链接生成二维码（`QRCode.toDataURL`）
- [`html-to-image`](https://www.npmjs.com/package/html-to-image)：把成绩单 / 完整试卷 DOM 导出为 PNG 图片（`toPng`）
- `@haruhi/api-client`（workspace）：统一 JWT 与管理员鉴权、与后端约定

## 目录结构要点

```
apps/exam/
├── index.html               # 入口 HTML
├── env.d.ts                 # *.vue 与 html-to-image 的类型声明
├── tsconfig.json            # 别名 @ → src，strict
├── vite.config.ts           # base /exam/、端口 5202、/api 与 /uploads 代理
└── src/
    ├── main.ts              # 装配 Pinia + Router
    ├── router/index.ts      # 路由表（见下）
    ├── views/
    │   ├── HomeView.vue     # 试卷列表（分页/搜索）+ 入口
    │   ├── ExamPaper.vue    # 答题/批改/导出分享主视图（qrcode + html-to-image）
    │   ├── EditorView.vue   # 试卷在线编辑（创建/edit_token 校验编辑/导入 JSON）
    │   └── AdminView.vue    # 管理员后台（审核：通过/锁定/删除 + 统计）
    ├── components/
    │   ├── QuestionRenderer.vue   # 渲染题干内容块（文本/图片/音频）
    │   ├── TheFooter.vue
    │   └── editor/                # 编辑器三大面板
    │       ├── ConfigPanel.vue        # 试卷配置/必填校验
    │       ├── LevelsPanel.vue        # 等级评定配置（图片上传）
    │       └── QuestionsPanel.vue     # 题目编辑（图片/音频上传）
    ├── stores/
    │   ├── exam.ts          # 试卷数据、答题/批改、本地存档、上传文件 GC
    │   └── audio.ts         # 音频播放器（多格式回退、会话令牌防并发）
    ├── composables/
    │   ├── useExamAdmin.ts  # 管理员门禁（封装 createAdminAuth('exam')）
    │   └── useCameraRig.ts  # 试卷"双页拼版"拖拽平移视口
    ├── services/api.ts      # 封装 /api/exam/* 端点
    ├── types/exam.ts        # ExamDatabase / Question / LevelConfig 等类型
    ├── utils/               # file.ts(图片→WebP 压缩)、index.ts(日期/考号/文本)
    └── data/mock-exam.ts    # 内置「凉宫春日」样例卷（/haruhi 路由）
```

## 本地开发

```bash
pnpm --filter @haruhi/exam dev       # 启动 dev 服务，端口 5202
pnpm --filter @haruhi/exam build     # vue-tsc --noEmit + vite build
pnpm --filter @haruhi/exam preview   # 预览构建产物
```

- 部署子路径：`/exam/`（`vite.config.ts` 的 `base` 与 Router 的 `createWebHistory('/exam/')`）。
- dev 下 `/api` 与 `/uploads` 代理到后端 `http://127.0.0.1:17777`，需先在仓库根运行 `cargo run -p haruhi-server`。

### 路由

| 路径 | 视图 | 说明 |
|------|------|------|
| `/` | HomeView | 已发布试卷列表（分页 + 搜索） |
| `/create` | EditorView | 新建试卷；带 `?id=&token=` 时进入编辑模式 |
| `/haruhi` | ExamPaper | 内置「凉宫春日」样例卷 |
| `/exam/:id` | ExamPaper | 按 id 作答某张试卷 |
| `/admin` | AdminView | 管理员审核后台 |

## 关键特性与约定

- **试卷编辑（edit_token）**：创建后由后端返回 `editToken`，编辑链接形如 `/exam/create?id=<id>&token=<token>`；`EditorView` 调 `api.verifyExam` 用 token 免登录校验并加载完整试卷，编辑 token 同时存入 `localStorage`。
- **媒体处理**：图片在前端用 Canvas 压缩为 **WebP**（`utils/file.ts` `compressImage`，默认质量 0.75）再上传；音频上传后由后端用 ffmpeg 转码为 MP3。`audio.ts` 播放时对 `.mp3/.m4a/.aac/...` 等后缀做顺序回退，并用 `sessionToken` 取消旧回退链、避免并发请求。
- **上传文件垃圾回收**：编辑会话内 `recordUpload` 记录所有 `/uploads/` 路径，保存时与最终试卷实际引用做差集，调 `/api/exam/cleanup` 清理未使用文件。
- **导出与分享**：`ExamPaper.vue` 用 `html-to-image` 的 `toPng` 把成绩单 / 完整试卷（正反双页拼版，`useCameraRig` 提供拖拽平移）导出为高分辨率 PNG，并用 `qrcode` 生成分享二维码。
- **作答本地存档**：考生姓名、答案、批改结果、考号（`BK+日期+随机`）按 `haruhi_exam_<id>` 持久化到 `localStorage`，刷新可恢复。
- **访问控制**：后端返回 `403 EXAM_UNAVAILABLE` 时按 `status`（`pending`/`locked`）显示对应不可访问提示，而非报错。
- **题型**：单选 / 多选（答案逗号分隔、顺序无关）/ 判断 / 填空，批改时文本经 `normalizeText` 归一化比较。

### JSON 试卷导入格式

编辑器的「导入 JSON」可批量建卷，文件分三段：

```jsonc
{
  "config":    { "title": "...", "paperTitle": "...", "author": "...", "contact": "..." }, // 试卷基本信息
  "questions": [ /* 题目列表 */ ],
  "levels":    [ { "id": "level1", "min": 0, "max": 30, "name": "不及格", "color": "#ef4444", "comment": "..." } ] // 等级配置
}
```

每道题由 `id / no / column(C1–C4) / type / score / stemBlocks / answer / analysisBlocks` 组成；题干（`stemBlocks`）与解析（`analysisBlocks`）都是**内容块数组**，块类型 `text` / `image`（`src` 指向 `/uploads/exam/*.webp`）/ `audio`（`/uploads/exam/*.mp3`）可任意组合。答案格式随题型而定：

| 题型（`type`） | `options` | `answer` 格式 |
| --- | --- | --- |
| 单选 `choice` | A/B/C/D | 单个字母，如 `"C"` |
| 多选 `multiple` | A/B/C/D | 逗号分隔、顺序无关，如 `"A,C"` |
| 判断 `judgment` | 固定 `true` / `false` | `"true"` 或 `"false"` |
| 填空 `fill` | 无 | 直接写答案，多空逗号分隔，如 `"2,北京"` |

`levels` 的分数区间不可重叠、需连续覆盖试卷满分。导入时编辑器会校验必填字段与答案格式。

## 与共享层 / 后端的关系

- 业务接口统一前缀 `/api/exam/*`（`services/api.ts`），静态资源在 `/uploads/exam/*`。
- 鉴权统一走 `@haruhi/api-client`：
  - 公开作答/列表接口无需登录；
  - 「导入 JSON」「管理后台」需管理员权限，由 `useExamAdmin`（`createAdminAuth('exam')`）负责登录/会话恢复，`createApiClient('/api/exam')` 自动带 `Authorization: Bearer <jwt>`。
- 管理后台数据走 `/api/exam/admin/*`（`stats` / `list` / `exams/:id/status` / 删除），对应后端 `haruhi_auth::authorize` 的 exam 模块权限校验。

## 更多

- 仓库总体架构与启动：见根 [`README.md`](../../README.md)
- 贡献规范与提交约定（scope `exam`）：见 [`CONTRIBUTING.md`](../../CONTRIBUTING.md)
- 新增模块流程：见 [`docs/ADDING_MODULE.md`](../../docs/ADDING_MODULE.md)
