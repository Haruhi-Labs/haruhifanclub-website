# @haruhi/exam

在线考试 app，包含试卷列表、答题批改、成绩/试卷图片导出、试卷编辑器和审核后台。

## 入口

- 子路径：`/exam/`
- dev 端口：`5202`
- 后端接口：`/api/exam/*`
- 上传资源：`/uploads/exam/*`
- 技术栈：Vue 3、TypeScript、Vue Router、Pinia、SCSS、Vite、`qrcode`、`html-to-image`

## 本地运行

```bash
bash deploy/gen-secrets.sh
pnpm dev:backend

pnpm --filter @haruhi/exam dev
pnpm --filter @haruhi/exam build
pnpm --filter @haruhi/exam preview
```

访问 `http://localhost:5202/exam/`。`build` 会先跑 `vue-tsc --noEmit`。

## 路由

| 路径        | 视图         | 说明                                        |
| ----------- | ------------ | ------------------------------------------- |
| `/`         | `HomeView`   | 试卷列表、分页、搜索                        |
| `/create`   | `EditorView` | 新建试卷；带 `id` 和 `token` 时编辑已有试卷 |
| `/haruhi`   | `ExamPaper`  | 内置样例卷                                  |
| `/exam/:id` | `ExamPaper`  | 作答指定试卷                                |
| `/admin`    | `AdminView`  | 审核后台                                    |

## 目录

```text
src/
  main.ts
  router/index.ts
  services/api.ts
  stores/
    exam.ts              试卷、答题、批改、本地存档、上传清理
    audio.ts             音频播放与多格式回退
  composables/
    useExamAdmin.ts
    useCameraRig.ts
  components/
    QuestionRenderer.vue
    editor/
      ConfigPanel.vue
      LevelsPanel.vue
      QuestionsPanel.vue
  views/
    HomeView.vue
    ExamPaper.vue
    EditorView.vue
    AdminView.vue
  types/exam.ts
  utils/
```

## 功能范围

- 单选、多选、判断、填空题。
- 考生答案、批改结果和考号存入 `localStorage`，刷新可恢复。
- 试卷编辑器支持配置、题目、等级三部分。
- 图片上传前在浏览器压缩为 WebP。
- 音频上传后由后端 ffmpeg 转 MP3。
- `html-to-image` 导出成绩单和整张试卷 PNG。
- `qrcode` 生成分享二维码。
- 后台审核通过、锁定、删除试卷，并查看统计。

## 后端契约

- 公开接口和编辑接口在 `src/services/api.ts`。
- 后台登录通过 `useExamAdmin`，内部使用 `createAdminAuth('exam')`。
- 编辑已有试卷使用 `edit_token`，链接形如 `/exam/create?id=<id>&token=<token>`。
- 后端返回 `403 EXAM_UNAVAILABLE` 时，前端按 `pending` 或 `locked` 显示不可访问状态。
- 编辑器保存后会对会话内上传但未被最终试卷引用的文件调用清理接口。

## JSON 导入

编辑器导入 JSON 时，顶层结构为：

```json
{
  "config": {},
  "questions": [],
  "levels": []
}
```

题目支持 `choice`、`multiple`、`judgment`、`fill`。题干和解析是内容块数组，块类型为 `text`、`image`、`audio`。等级区间需要覆盖试卷总分且不能重叠。

## 维护注意

- Vite `base` 是 `/exam/`，Router 也按该子路径创建 history。
- 改题目类型、导入格式或等级规则时，同步更新 `types/exam.ts`、编辑器校验和后端解析。
- 音频播放有回退链和会话令牌，修播放问题时先看 `stores/audio.ts`。
