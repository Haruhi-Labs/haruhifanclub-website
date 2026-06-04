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
 *   - scope **可选、不限定取值**：用它点明本次改动的范围（受影响的 app / crate / 跨领域区域）。
 *     具体写法建议见 CONTRIBUTING.md——**不在代码里维护一套封闭 scope 集合**。
 *
 * type（唯一受约束项）：feat fix perf refactor docs style test build ci chore revert
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

/** @type {import('@commitlint/types').UserConfig} */
export default {
  extends: ['@commitlint/config-conventional'],
  rules: {
    // type 锁定到本仓库合法集合
    'type-enum': [2, 'always', types],
    // scope 自由：不校验取值（只在 CONTRIBUTING 里建议用 scope 点明范围）
    'scope-enum': [0],
    // 中文友好：subject 不限大小写、不强制句号
    'subject-case': [0],
    'subject-full-stop': [0],
    // 放宽 header 长度到 100，body/footer 行长不限（中文友好）
    'header-max-length': [2, 'always', 100],
    'body-max-line-length': [0],
    'footer-max-line-length': [0],
  },
};
