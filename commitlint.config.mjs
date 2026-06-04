/**
 * commitlint 配置（凉宫春日应援团 monorepo）
 *
 * 用途：
 *   - CI（.github/workflows/pr-checks.yml 的 commitlint job）逐条校验 PR 范围内的每个提交。
 *   - 本地手动自查：`pnpm exec commitlint --from=HEAD~1 --to=HEAD` 或 `pnpm run lint:commit`。
 *   - 本仓库不安装本地 git hooks，规范完全靠 CI 把关。
 *
 * 提交格式：type(scope): subject
 *   - subject 允许中文、不限大小写、不强制句号。
 *   - scope 可省略；若给出则必须取自下方 scope 集合。
 *
 * type（唯一合法集合）：
 *   feat     新功能
 *   fix      缺陷修复
 *   perf     性能优化
 *   refactor 重构（既非新功能也非修复）
 *   docs     文档
 *   style    代码风格/格式（不影响逻辑）
 *   test     测试
 *   build    构建系统或外部依赖（产物构建相关）
 *   ci       持续集成配置与脚本
 *   chore    杂务（不修改 src/test 的其它改动）
 *   revert   回滚此前的提交
 *
 * scope（可选；含义）：
 *   —— 前端 app ——
 *   news     新闻站（/news/）
 *   art      美术站（/art/）
 *   exam     考试站（/exam/，TS）
 *   novel    小说站（/library/）
 *   shop     商店站（/shop/）
 *   console  超管台（/console/，TS，RBAC）
 *   —— 共享前端 ——
 *   api-client  前端共享 API 客户端（被全部 6 个 app 依赖）
 *   —— 后端 crate ——
 *   server   单二进制后端服务
 *   core     核心通用 crate
 *   db       数据库 crate
 *   auth     鉴权 crate
 *   media    媒体处理 crate
 *   ai       AI crate
 *   mail     邮件 crate
 *   —— 跨领域 ——
 *   deploy   部署
 *   ci       持续集成
 *   docs     文档
 *   deps     依赖
 *   repo     仓库级（配置、元信息等）
 */

// type 唯一合法集合
const types = [
  'feat',
  'fix',
  'perf',
  'refactor',
  'docs',
  'style',
  'test',
  'build',
  'ci',
  'chore',
  'revert',
];

// scope 集合（可空，但给出则必须在集合内）
const scopes = [
  // 前端 app
  'news',
  'art',
  'exam',
  'novel',
  'shop',
  'console',
  // 共享前端
  'api-client',
  // 后端 crate
  'server',
  'core',
  'db',
  'auth',
  'media',
  'ai',
  'mail',
  // 跨领域
  'deploy',
  'ci',
  'docs',
  'deps',
  'repo',
];

/** @type {import('@commitlint/types').UserConfig} */
export default {
  extends: ['@commitlint/config-conventional'],
  rules: {
    // type 锁定到本仓库合法集合
    'type-enum': [2, 'always', types],
    // scope 可空；给出则必须在集合内
    'scope-enum': [2, 'always', scopes],
    // 中文友好：subject 不限大小写、不强制句号
    'subject-case': [0],
    'subject-full-stop': [0],
    // 放宽 header 长度到 100，body/footer 行长不限（中文友好）
    'header-max-length': [2, 'always', 100],
    'body-max-line-length': [0],
    'footer-max-line-length': [0],
  },
};
