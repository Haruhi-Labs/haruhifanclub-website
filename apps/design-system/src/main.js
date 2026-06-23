import '@haruhi/design-system/components.css'
import designMarkdown from '../../../docs/DESIGN_SYSTEM.md?raw'
import './style.css'

const svgData = (svg) => `data:image/svg+xml;charset=UTF-8,${encodeURIComponent(svg)}`
const logoUrl = `${import.meta.env.BASE_URL}haruhi-logo-192.png`

const productImage = svgData(`
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 900 900">
  <rect width="900" height="900" fill="#eef4ff"/>
  <rect x="210" y="116" width="480" height="640" rx="72" fill="#fff" stroke="#dde1e6" stroke-width="12"/>
  <rect x="270" y="196" width="360" height="360" rx="48" fill="#ffd9dc"/>
  <circle cx="450" cy="376" r="132" fill="#ffc83d"/>
  <circle cx="404" cy="350" r="18" fill="#171a22"/>
  <circle cx="496" cy="350" r="18" fill="#171a22"/>
  <path d="M384 420q66 54 132 0" fill="none" stroke="#171a22" stroke-width="18" stroke-linecap="round"/>
  <path d="M328 638h244" stroke="#3478f6" stroke-width="30" stroke-linecap="round"/>
</svg>`)

const artImage = svgData(`
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1200 900">
  <rect width="1200" height="900" fill="#bfe7ee"/>
  <circle cx="910" cy="190" r="92" fill="#fff4c5"/>
  <path d="M0 620q190-154 386-24t384-18q174-112 430 26v296H0z" fill="#159a90" opacity=".68"/>
  <path d="M0 724q244-132 472-34t418-28q164-92 310 24v214H0z" fill="#0f1b2e"/>
  <path d="M560 552c36-112 88-190 156-260" stroke="#171a22" stroke-width="22" stroke-linecap="round" fill="none"/>
  <circle cx="726" cy="274" r="52" fill="#171a22"/>
  <path d="M676 374l-108-72M680 386l126 28M624 466l-74 128M660 468l104 104" stroke="#171a22" stroke-width="20" stroke-linecap="round"/>
</svg>`)

const modes = [
  {
    id: 'news',
    label: 'news',
    name: '春日团报',
    color: '#171a22',
    note: '中性灰白、墨色主行动。黄色只作为线索、专题编号和重要标记。',
    sample: `
      <article class="sos-card sos-news-card sos-card--interactive">
        <div class="sos-card__body">
          <span class="sos-news-card__type">NEWS</span>
          <h3 class="sos-news-card__title">凉宫春日动画台词匹配站发布</h3>
          <p class="sos-news-card__summary">支持凉宫与京阿尼作品台词查询，强调语义理解和清晰的信息来源。</p>
          <div class="sos-card__footer sos-news-card__meta">
            <div class="ds-inline">
              <span class="sos-badge sos-badge--outline">#技术</span>
              <span class="sos-badge sos-badge--outline">#工具</span>
            </div>
            <span class="ds-meta">2026-06-23</span>
          </div>
        </div>
      </article>`,
  },
  {
    id: 'shop',
    label: 'shop',
    name: '春日商城',
    color: '#3478f6',
    note: '蓝灰浅底、行动蓝主 CTA。交易数字必须清晰，品牌黄只作标签。',
    sample: `
      <article class="sos-card sos-product-card sos-card--interactive">
        <div class="sos-product-card__media"><img alt="团长推荐商品示意" src="${productImage}"></div>
        <div class="sos-card__body">
          <div class="sos-product-card__title-row">
            <h3 class="sos-product-card__title">朝比奈实玖瑠 fufu</h3>
            <strong class="sos-product-card__price">¥ 147</strong>
          </div>
          <p class="sos-product-card__description">达到目标后进入统一排产，库存和进度常驻显示。</p>
          <div class="sos-progress">
            <div class="sos-progress__meta"><span>进度预售</span><strong>126/200</strong></div>
            <div class="sos-progress__track"><span class="sos-progress__fill" style="width: 63%"></span></div>
          </div>
          <div class="sos-card__footer ds-card-footer-gap">
            <span class="sos-badge sos-badge--accent">fufu</span>
            <span class="ds-meta">预售商品</span>
          </div>
        </div>
      </article>`,
  },
  {
    id: 'art',
    label: 'art',
    name: '春日美术部',
    color: '#159a90',
    note: '作品是主角。允许一层磨砂和柔和悬浮，禁止背景抢作品。',
    sample: `
      <article class="sos-art-card sos-card--interactive">
        <div class="sos-art-card__frame">
          <img alt="美术部作品示意" src="${artImage}">
          <div class="sos-art-card__veil">
            <span class="sos-badge sos-badge--signal">插画</span>
            <h3 class="sos-art-card__title">放学后的未知信号</h3>
          </div>
        </div>
        <div class="sos-art-card__meta"><span>SOS 团美术部</span><span>查看作品</span></div>
      </article>`,
  },
  {
    id: 'library',
    label: 'library',
    name: '长门有希的书架',
    color: '#9d5d16',
    note: '纸张、书脊和阅读行高优先。装饰颗粒不能影响正文对比。',
    sample: `
      <article class="sos-book-card sos-card--interactive">
        <div class="sos-book-card__cover" style="--book-color: #f4ecdc">
          <span class="sos-book-card__vertical">消失的长门书签</span>
        </div>
        <div>
          <h3 class="sos-book-card__title">长门有希的书架</h3>
          <p class="sos-book-card__author">阅读 · 1.9 行高</p>
        </div>
      </article>`,
  },
  {
    id: 'exam',
    label: 'exam',
    name: '春日试卷中心',
    color: '#c8171e',
    note: '木桌和纸张可以有舞台感，题目、选项、倒计时必须稳定。',
    sample: `
      <article class="sos-exam-card sos-card--interactive">
        <div class="sos-exam-card__paper">
          <div class="sos-exam-card__content">
            <span class="sos-stamp">Official</span>
            <strong class="sos-exam-card__score">92</strong>
            <h3 class="sos-exam-card__title">SOS 团入团资格综合测验</h3>
            <p class="sos-exam-card__meta">正式试卷 · 25 题 · 预计 18 分钟</p>
            <hr class="sos-exam-card__rule">
            <div class="sos-progress">
              <div class="sos-progress__meta"><span>答题完成度</span><strong>36%</strong></div>
              <div class="sos-progress__track"><span class="sos-progress__fill" style="width: 36%"></span></div>
            </div>
            <div class="sos-card__footer ds-card-footer-gap">
              <span class="sos-badge sos-badge--outline">单选 · 多选 · 简答</span>
              <button class="sos-button sos-button--primary sos-button--sm">开始答题</button>
            </div>
          </div>
        </div>
      </article>`,
  },
]

const navItems = [
  ['overview', '总览'],
  ['architecture', '接入结构'],
  ['foundations', '基础规范'],
  ['layout', '布局原语'],
  ['components', '基础组件'],
  ['contracts', '组件契约'],
  ['expressions', '表达模式'],
  ['voice', '组件变声'],
  ['quality', '状态响应式'],
  ['migration', '迁移验收'],
]

const palette = [
  ['Signal Yellow', '--sos-yellow-500', '#ffc83d', '品牌信号、重点标签、全局提示'],
  ['Sky', '--sos-sky-500', '#4b9fe8', '导航、轻快链接、默认焦点'],
  ['Action Blue', '--sos-blue-500', '#3478f6', '商城主行动、交易链接'],
  ['Gallery Teal', '--sos-teal-500', '#159a90', '美术部主行动、成功语义'],
  ['Book Amber', '--sos-amber-600', '#9d5d16', '书架、档案、历史内容'],
  ['Teacher Red', '--sos-red-600', '#c8171e', '考试、危险、强提醒'],
  ['Ink', '--sos-ink-950', '#171a22', '标题、深色表面、团报主行动'],
  ['Paper', '--sos-paper-100', '#f4ecdc', '书籍、便签、试卷纸张'],
]

const spacingScale = [
  ['4 / 8', '细节间距', '图标与文字、Badge 内部、紧凑列表行内元素。'],
  ['12 / 16', '控件节奏', '输入框、按钮组、表单 label 与 help text。'],
  ['20 / 24', '卡片内部', '标题、说明、媒体、状态和操作之间的局部留白。'],
  ['32 / 40', '组合间距', '筛选区、列表组、详情块和并列模块之间。'],
  ['48 / 64', '页面模块', '频道头部、内容区、表单大段落之间的主要分隔。'],
  ['80 / 96', '大区段', '只用于首页或专题页首屏级区段，后台和密集页慎用。'],
]

const layoutPrimitives = [
  ['Stack', '.sos-stack', '纵向信息组。标题、说明、表单、状态列表默认用 Stack 管理间距。'],
  ['Inline', '.sos-inline', '同行工具组。按钮、Badge、短操作集合可换行，不拉伸子项。'],
  ['Cluster', '.sos-cluster', '两端或多组对齐。卡片 footer、标题栏和统计行优先用 Cluster。'],
  ['Grid', '.sos-grid', '自适应卡片网格。只定义最小列宽和 gap，不在业务页硬写列数。'],
  ['Surface', '.sos-surface', '页面内有边界的承载面。用于控制边框、圆角、背景和 elevation。'],
  [
    'MediaFrame',
    '.sos-media-frame',
    '媒体比例容器。商品 1:1、作品 4:3、书封 2:3 必须通过 ratio 声明。',
  ],
]

const componentContracts = [
  [
    'Button',
    'button / a.sos-button',
    'primary · secondary · ghost · danger · sm · lg',
    'hover · active · focus-visible · disabled · loading',
    '只承载明确命令；图标按钮以后单独封装为 IconButton。',
  ],
  [
    'Badge',
    'span.sos-badge',
    'default · accent · solid · outline · signal',
    'default · selected · disabled by parent',
    '只表达状态、分类或短标签；不能作为段落标题或按钮替代品。',
  ],
  [
    'Field',
    'label.sos-field > label/input/help',
    'input · textarea · select',
    'focus · disabled · error · help · required',
    'Label 不被 placeholder 替代；错误态必须有文字证据。',
  ],
  [
    'Card',
    '.sos-card + body/footer/media',
    'flat · raised · interactive · composition recipe',
    'hover · focus-within · selected · loading · empty',
    '卡片内部节奏由 body/footer/media anatomy 管理，页面不得临时改 padding。',
  ],
  [
    'Notice',
    '.sos-notice > icon/content/action',
    'info · warning · success · danger',
    'dismissible · action · compact',
    '用于系统提示，不用于普通营销文案。',
  ],
  [
    'Progress',
    '.sos-progress > meta/track/fill',
    'default · success · danger · compact later',
    '0 · in-progress · complete · error',
    '进度数字常驻显示，不能只依赖颜色或 hover。',
  ],
  [
    'EmptyState',
    '.sos-empty-state > icon/title/copy/action',
    'default',
    'empty · no-result · permission-missing',
    '必须说明为空原因，并提供清晰下一步。',
  ],
]

const stateRows = [
  ['Default', '信息完整、操作可用、无悬浮依赖。'],
  ['Hover', '只增强可点击感；不能出现新关键信息。'],
  ['Focus-visible', '键盘焦点必须比边框清晰，不能只靠阴影。'],
  ['Disabled', '降低可操作性但保留 label，必要时说明原因。'],
  ['Loading', '锁定重复提交，保留当前上下文和进度反馈。'],
  ['Empty / Error', '给出下一步动作；错误不能只用红色表达。'],
]

const componentStateExamples = [
  {
    name: 'Button',
    rule: '命令状态要保留标签和尺寸；loading 锁定重复提交，disabled 仍能读出动作含义。',
    cases: [
      ['Default', '<button class="sos-button sos-button--primary">提交审核</button>'],
      [
        'Hover',
        '<button class="sos-button sos-button--primary" data-state="hover">悬停预览</button>',
      ],
      [
        'Focus',
        '<button class="sos-button sos-button--secondary" data-state="focus">键盘焦点</button>',
      ],
      [
        'Loading',
        '<button class="sos-button sos-button--primary" aria-busy="true" disabled>提交中</button>',
      ],
      ['Disabled', '<button class="sos-button sos-button--secondary" disabled>不可提交</button>'],
    ],
  },
  {
    name: 'Badge',
    rule: '短标签可以表达分类、选中和不可用，但不能替代按钮或段落标题。',
    cases: [
      ['Default', '<span class="sos-badge">普通</span>'],
      ['Accent', '<span class="sos-badge sos-badge--accent">分类</span>'],
      ['Selected', '<span class="sos-badge" aria-selected="true">已选</span>'],
      ['Signal', '<span class="sos-badge sos-badge--signal">重点</span>'],
      ['Disabled', '<span class="sos-badge sos-badge--outline" aria-disabled="true">已过期</span>'],
    ],
  },
  {
    name: 'Field',
    rule: '输入状态必须同时有 label、控件边界和帮助/错误文字，placeholder 不承担 label。',
    cases: [
      [
        'Default',
        '<label class="sos-field"><span class="sos-field__label">标题</span><input class="sos-input" value="北高校园祭专题"><span class="sos-field__help">可在发布前修改。</span></label>',
      ],
      [
        'Focus',
        '<label class="sos-field"><span class="sos-field__label">搜索</span><input class="sos-input" data-state="focus" value="长门"><span class="sos-field__help">焦点外环可截图验收。</span></label>',
      ],
      [
        'Error',
        '<label class="sos-field sos-field--error"><span class="sos-field__label">库存</span><input class="sos-input" aria-invalid="true" value="-1"><span class="sos-field__help">库存不能小于 0。</span></label>',
      ],
      [
        'Disabled',
        '<label class="sos-field"><span class="sos-field__label">审核编号</span><input class="sos-input" value="AUTO-042" disabled><span class="sos-field__help">由系统生成。</span></label>',
      ],
    ],
  },
  {
    name: 'Notice',
    rule: '系统提示用 tone 表达语义，并始终保留标题和正文证据。',
    cases: [
      [
        'Info',
        '<div class="sos-notice"><span class="sos-notice__icon">i</span><div><h4 class="sos-notice__title">保存成功</h4><p class="sos-notice__copy">草稿已进入待审核列表。</p></div></div>',
      ],
      [
        'Success',
        '<div class="sos-notice sos-notice--success"><span class="sos-notice__icon">✓</span><div><h4 class="sos-notice__title">流程完成</h4><p class="sos-notice__copy">可以继续发布下一条内容。</p></div></div>',
      ],
      [
        'Warning',
        '<div class="sos-notice sos-notice--warning"><span class="sos-notice__icon">!</span><div><h4 class="sos-notice__title">库存偏低</h4><p class="sos-notice__copy">继续售卖前请确认补货计划。</p></div></div>',
      ],
      [
        'Danger',
        '<div class="sos-notice sos-notice--danger"><span class="sos-notice__icon">!</span><div><h4 class="sos-notice__title">提交失败</h4><p class="sos-notice__copy">网络超时，请保留当前内容后重试。</p></div></div>',
      ],
    ],
  },
  {
    name: 'Progress',
    rule: '进度数字常驻显示；完成和错误不能只靠色彩表达。',
    cases: [
      [
        'Active',
        '<div class="sos-progress"><div class="sos-progress__meta"><span>预售进度</span><strong>63%</strong></div><div class="sos-progress__track"><span class="sos-progress__fill" style="width:63%"></span></div></div>',
      ],
      [
        'Complete',
        '<div class="sos-progress sos-progress--success"><div class="sos-progress__meta"><span>上传完成</span><strong>100%</strong></div><div class="sos-progress__track"><span class="sos-progress__fill" style="width:100%"></span></div></div>',
      ],
      [
        'Error',
        '<div class="sos-progress sos-progress--danger"><div class="sos-progress__meta"><span>导入失败</span><strong>第 7 行错误</strong></div><div class="sos-progress__track"><span class="sos-progress__fill" style="width:72%"></span></div></div>',
      ],
      [
        'Zero',
        '<div class="sos-progress"><div class="sos-progress__meta"><span>尚未开始</span><strong>0%</strong></div><div class="sos-progress__track"><span class="sos-progress__fill" style="width:0%"></span></div></div>',
      ],
    ],
  },
  {
    name: 'Card',
    rule: '卡片状态只增强容器反馈；标题、状态和下一步动作必须常驻。',
    cases: [
      [
        'Default',
        '<article class="sos-card"><div class="sos-card__body"><h4 class="ds-state-card-title">待审核稿件</h4><p class="ds-state-card-copy">标题、摘要和日期常驻。</p></div></article>',
      ],
      [
        'Hover',
        '<article class="sos-card sos-card--interactive" data-state="hover"><div class="sos-card__body"><h4 class="ds-state-card-title">可打开卡片</h4><p class="ds-state-card-copy">hover 只增强可点击感。</p></div></article>',
      ],
      [
        'Selected',
        '<article class="sos-card" aria-selected="true"><div class="sos-card__body"><h4 class="ds-state-card-title">已选内容</h4><p class="ds-state-card-copy">选中态有边界和色块证据。</p></div></article>',
      ],
      [
        'Loading',
        '<article class="sos-card" aria-busy="true"><div class="sos-card__body"><h4 class="ds-state-card-title">同步状态</h4><p class="ds-state-card-copy">上下文仍在原位。</p></div></article>',
      ],
    ],
  },
  {
    name: 'EmptyState',
    rule: '空状态解释为什么为空，并给出一个真实下一步，不能只是插画或口号。',
    cases: [
      [
        'No Data',
        '<section class="sos-empty-state"><span class="sos-empty-state__icon">0</span><h4 class="sos-empty-state__title">暂无投稿</h4><p class="sos-empty-state__copy">当前筛选条件下没有作品。</p><div class="sos-empty-state__actions"><button class="sos-button sos-button--secondary sos-button--sm">清除筛选</button></div></section>',
      ],
      [
        'No Result',
        '<section class="sos-empty-state"><span class="sos-empty-state__icon">?</span><h4 class="sos-empty-state__title">没有匹配结果</h4><p class="sos-empty-state__copy">换一个关键词，或回到全部列表。</p></section>',
      ],
      [
        'Permission',
        '<section class="sos-empty-state"><span class="sos-empty-state__icon">!</span><h4 class="sos-empty-state__title">需要登录</h4><p class="sos-empty-state__copy">登录后才能查看订单和投稿记录。</p><div class="sos-empty-state__actions"><button class="sos-button sos-button--primary sos-button--sm">去登录</button></div></section>',
      ],
    ],
  },
]

const responsiveRules = [
  ['320-390', 'Phone', '单列；核心操作 44px 以上；所有卡片不能依赖 hover。'],
  ['640', 'Large Phone', '工具组允许换行；FilterBar 可折叠为抽屉或段落。'],
  ['768', 'Tablet', '列表与详情可以并排；卡片网格最小列宽驱动。'],
  ['1024', 'Laptop', '常规后台和内容页进入双栏；侧栏允许 sticky。'],
  ['1280', 'Desktop', '内容容器锁定 1248px；大网格使用 1472px。'],
  ['1440+', 'Wide', '只增加留白和列数，不放大字体或卡片内部 padding。'],
]

const app = document.querySelector('#app')
const markdownUrl = URL.createObjectURL(
  new Blob([designMarkdown], { type: 'text/markdown;charset=utf-8' })
)

app.innerHTML = `
  <main class="sos-scope ds-app">
    <header class="ds-topbar">
      <a class="ds-brand sos-brand-lockup sos-brand-lockup--compact" href="#overview" aria-label="返回设计规范总览">
        <span class="ds-brand__mark sos-brand-lockup__mark"><img src="${logoUrl}" alt="" aria-hidden="true"></span>
        <span class="sos-brand-lockup__text">
          <strong>SOS / Parallel Design System</strong>
          <small>项目内正式设计规范 · v0.2.0</small>
        </span>
      </a>
      <a class="sos-button sos-button--secondary sos-button--sm" href="${markdownUrl}" download="DESIGN_SYSTEM.md">Markdown 规范</a>
    </header>

    <div class="ds-layout">
      <aside class="ds-sidebar" aria-label="设计规范目录">
        <nav class="ds-nav">
          ${navItems.map(([id, label]) => `<a href="#${id}" class="ds-nav__item">${label}</a>`).join('')}
        </nav>
        <div class="ds-note">
          <strong>接入原则</strong>
          <p>外部规范和既有工程都不是最终答案。正式落地以本仓库的包结构、部署方式和迁移成本为准。</p>
        </div>
      </aside>

      <div class="ds-content">
        <section class="ds-hero" id="overview">
          <p class="sos-eyebrow">One Spine, Five Worlds</p>
          <h1>一套骨架，五个平行世界。</h1>
          <p class="ds-hero__lead">v0.2 把设计系统从“可看的规范页”推进到“可执行的工程契约”：Token、布局原语、组件 anatomy、状态矩阵和响应式规则先稳定，再进入 Vue UI 库。</p>
          <div class="ds-hero__actions">
            <a class="sos-button sos-button--primary sos-button--lg" href="#architecture">查看接入结构</a>
            <a class="sos-button sos-button--secondary sos-button--lg" href="#expressions">浏览表达模式</a>
          </div>
          <dl class="ds-metrics" aria-label="设计系统关键数字">
            <div><dt>3</dt><dd>Primitive / Semantic / Expression</dd></div>
            <div><dt>5</dt><dd>业务表达模式</dd></div>
            <div><dt>6</dt><dd>layout primitives</dd></div>
          </dl>
        </section>

        <section class="ds-section" id="architecture">
          <div class="ds-section__header">
            <p class="sos-eyebrow">Monorepo Contract</p>
            <h2>接入结构</h2>
            <p>规范页和共享样式是新增能力，不把现有业务 app 立即改造成同一套组件库。v0.2 先稳定 CSS 契约，并启动 <code>@haruhi/ui</code> 的 Vue 基础组件 MVP。</p>
          </div>
          <div class="ds-architecture">
            <article>
              <span>Package</span>
              <h3>@haruhi/design-system</h3>
              <p>导出 <code>tokens.css</code>、<code>components.css</code>、<code>bridges.css</code>。业务 app 可以按需导入，不绑定 Vue 或 React。</p>
            </article>
            <article>
              <span>App</span>
              <h3>apps/design-system</h3>
              <p>部署到 <code>/design-system/</code> 的单页规范，承载 token、组件、表达模式、迁移策略和验收清单。</p>
            </article>
            <article>
              <span>Docs</span>
              <h3>docs/DESIGN_SYSTEM.md</h3>
              <p>正式文字规范，记录项目约束、Do / Don't、迁移阶段和上线检查项。</p>
            </article>
            <article>
              <span>Vue MVP</span>
              <h3>packages/ui</h3>
              <p>导出 Button、Badge、Field、Notice、Progress、Card、EmptyState、HeaderBrand 和布局 wrapper。它只输出既有 class，不重新定义视觉。</p>
            </article>
          </div>
          <pre class="ds-code"><code>// CSS-first 接入
import '@haruhi/design-system/tokens.css'
import '@haruhi/design-system/components.css'

&lt;section class="sos-scope" data-sos-site="shop"&gt;
  &lt;button class="sos-button sos-button--primary"&gt;加入购物车&lt;/button&gt;
&lt;/section&gt;

// Vue wrapper MVP
import { SosButton, SosField, SosStack } from '@haruhi/ui'</code></pre>
        </section>

        <section class="ds-section" id="foundations">
          <div class="ds-section__header">
            <p class="sos-eyebrow">Foundations</p>
            <h2>基础规范</h2>
            <p>业务代码消费 Semantic Token，Expression Mode 只负责映射。不得在业务 CSS 中随手新增 Hex、阴影、圆角和非 4px 网格间距。</p>
          </div>
          <div class="ds-token-grid">
            ${palette
              .map(
                ([name, token, color, usage]) => `
              <article class="ds-token">
                <span class="ds-token__swatch" style="background:${color}"></span>
                <h3>${name}</h3>
                <code>${token}</code>
                <p>${usage}</p>
              </article>
            `
              )
              .join('')}
          </div>
          <div class="ds-foundation-grid">
            <article>
              <h3>排版</h3>
              <p>UI 使用系统 sans，长文和书架可使用衬线阅读栈。价格、分数、进度启用等宽数字。</p>
              <div class="ds-type-stack">
                <strong>频道标题 Display</strong>
                <span>卡片标题 Title</span>
                <small>日期、库存、帮助和元信息 Caption</small>
              </div>
            </article>
            <article>
              <h3>间距</h3>
              <p>间距只从 4px 网格取值，不用视觉上“差不多”的临时值。选择间距先看信息关系，再看页面气质。</p>
              <div class="ds-spacing-scale" aria-label="Spacing scale usage">
                ${spacingScale
                  .map(
                    ([value, role, guidance]) => `
                  <div>
                    <code>${value}px</code>
                    <strong>${role}</strong>
                    <span>${guidance}</span>
                  </div>
                `
                  )
                  .join('')}
              </div>
            </article>
            <article>
              <h3>形状</h3>
              <p>圆角只使用 4 / 8 / 12 / 18 / 24 / full。表达模式映射语义，不新增临时半径。</p>
              <div class="ds-radius-row">
                ${[
                  ['4', 4],
                  ['8', 8],
                  ['12', 12],
                  ['18', 18],
                  ['24', 24],
                  ['full', 999],
                ]
                  .map(
                    ([label, radius]) => `<span style="border-radius:${radius}px">${label}</span>`
                  )
                  .join('')}
              </div>
            </article>
            <article>
              <h3>Header 标识组合</h3>
              <p>Header 使用 <code>haruhi-logo-192.png</code> 加标题文字。图形保持一致，文字可以按站点气质调整，但尺寸、间距和行高要稳定。</p>
              <div class="ds-lockup-demo">
                <div class="ds-lockup sos-brand-lockup">
                  <img src="${logoUrl}" alt="">
                  <div class="sos-brand-lockup__text">
                    <strong>Haruhi Fan Club</strong>
                    <span>统一站群入口</span>
                  </div>
                </div>
                <div class="ds-lockup ds-lockup--compact sos-brand-lockup sos-brand-lockup--compact">
                  <img src="${logoUrl}" alt="">
                  <div class="sos-brand-lockup__text">
                    <strong>春日团报</strong>
                  </div>
                </div>
                <div class="ds-lockup ds-lockup--site sos-brand-lockup" data-sos-site="shop">
                  <img src="${logoUrl}" alt="">
                  <div class="sos-brand-lockup__text">
                    <strong>春日商城</strong>
                    <span>预售、订单与周边</span>
                  </div>
                </div>
              </div>
            </article>
          </div>
        </section>

        <section class="ds-section" id="layout">
          <div class="ds-section__header">
            <p class="sos-eyebrow">Layout Primitives</p>
            <h2>先稳定间距，再封装组件</h2>
            <p>v0.2 新增框架无关布局原语。它们不替代业务组件，而是让卡片内部、工具组、媒体比例和页面网格不再靠局部 CSS 猜测。</p>
          </div>
          <div class="ds-primitive-grid">
            ${layoutPrimitives
              .map(
                ([name, className, usage]) => `
              <article class="ds-primitive-card">
                <code>${className}</code>
                <h3>${name}</h3>
                <p>${usage}</p>
              </article>
            `
              )
              .join('')}
          </div>
          <div class="ds-layout-lab sos-surface">
            <div class="sos-stack">
              <span class="sos-badge sos-badge--signal">Stack</span>
              <h3>内容组只处理垂直节奏</h3>
              <p>标题、正文、帮助文字和状态之间使用 Stack，而不是在每个子元素上追加 margin。</p>
            </div>
            <div class="sos-stack sos-stack--tight">
              <span class="sos-badge sos-badge--accent">MediaFrame</span>
              <div class="sos-grid" style="--sos-grid-min: 7rem; --sos-grid-gap: var(--sos-space-3)">
                <div class="sos-media-frame" data-ratio="1:1"><img alt="" src="${productImage}"></div>
                <div class="sos-media-frame" data-ratio="4:3"><img alt="" src="${artImage}"></div>
                <div class="sos-media-frame" data-ratio="2:3"><span class="ds-book-mini">书</span></div>
              </div>
              <p>比例写在容器上，图片只负责 cover。页面不再通过固定高度猜测卡片形态。</p>
            </div>
          </div>
        </section>

        <section class="ds-section" id="components">
          <div class="ds-section__header">
            <p class="sos-eyebrow">Components</p>
            <h2>基础组件契约</h2>
            <p>基础组件先定义 class contract。Vue 或 React 包装层以后只负责产出相同 anatomy、variant 和 state。</p>
          </div>
          <div class="ds-component-grid">
            <article>
              <h3>Button</h3>
              <div class="ds-inline">
                <button class="sos-button sos-button--primary">提交审核</button>
                <button class="sos-button sos-button--secondary">保存草稿</button>
                <button class="sos-button sos-button--ghost">取消</button>
                <button class="sos-button sos-button--danger">删除</button>
              </div>
            </article>
            <article>
              <h3>Badge</h3>
              <div class="ds-inline">
                <span class="sos-badge">普通</span>
                <span class="sos-badge sos-badge--accent">分类</span>
                <span class="sos-badge sos-badge--solid">选中</span>
                <span class="sos-badge sos-badge--signal">团长推荐</span>
              </div>
            </article>
            <article>
              <h3>Field</h3>
              <label class="sos-field">
                <span class="sos-field__label">页面标题</span>
                <input class="sos-input" value="北高校园祭专题" aria-label="页面标题">
                <span class="sos-field__help">Label 不被 placeholder 替代；错误态要同时有文字、图标和颜色。</span>
              </label>
            </article>
            <article>
              <h3>Notice + Progress</h3>
              <div class="sos-notice">
                <span class="sos-notice__icon">!</span>
                <div>
                  <h4 class="sos-notice__title">迁移前先建立视觉基线</h4>
                  <p class="sos-notice__copy">Bridge 是过渡层，不是新变量命名空间。</p>
                </div>
              </div>
              <div class="sos-progress ds-progress-demo">
                <div class="sos-progress__meta"><span>组件迁移</span><strong>40%</strong></div>
                <div class="sos-progress__track"><span class="sos-progress__fill" style="width:40%"></span></div>
              </div>
            </article>
          </div>
        </section>

        <section class="ds-section" id="contracts">
          <div class="ds-section__header">
            <p class="sos-eyebrow">Component Contract</p>
            <h2>UI 库先封装稳定 anatomy</h2>
            <p><code>@haruhi/ui</code> 已启用 Vue MVP，只封装这些已经稳定的基础组件。业务卡片先作为 recipe 验证，不急着抽成跨站组件。</p>
          </div>
          <div class="ds-ui-mvp sos-surface sos-surface--padded">
            <div>
              <span class="sos-badge sos-badge--signal">UI MVP</span>
              <h3>先抽基础件，不抽业务卡片</h3>
              <p>Button、Badge、Field、Notice、Progress、Card、EmptyState、HeaderBrand、Stack、Inline、Surface 和 MediaFrame 已进入 <code>@haruhi/ui</code>。每个 wrapper 只负责 props、slot、可访问性和 class 组合。</p>
            </div>
            <pre class="ds-code ds-code--compact"><code>&lt;SosStack gap="loose" data-sos-site="shop"&gt;
  &lt;SosField label="商品标题" help="Label 不被 placeholder 替代"&gt;
    &lt;input class="sos-input" value="SOS 团限定徽章" /&gt;
  &lt;/SosField&gt;
  &lt;SosButton&gt;保存商品&lt;/SosButton&gt;
&lt;/SosStack&gt;</code></pre>
          </div>
          <div class="ds-contract-table" role="table" aria-label="Component contract matrix">
            <div role="row" class="ds-contract-row ds-contract-row--head">
              <span>组件</span>
              <span>Anatomy</span>
              <span>Variants</span>
              <span>States</span>
              <span>规则</span>
            </div>
            ${componentContracts
              .map(
                ([name, anatomy, variants, states, rule]) => `
              <div role="row" class="ds-contract-row">
                <strong>${name}</strong>
                <code>${anatomy}</code>
                <span>${variants}</span>
                <span>${states}</span>
                <p>${rule}</p>
              </div>
            `
              )
              .join('')}
          </div>
        </section>

        <section class="ds-section" id="expressions">
          <div class="ds-section__header">
            <p class="sos-eyebrow">Expression Modes</p>
            <h2>五个平行世界</h2>
            <p>同一组件在不同 <code>data-sos-site</code> 下切换语义映射。变化范围被限制在颜色、材质、圆角、媒体比例和阅读字体。</p>
          </div>
          <div class="ds-mode-picker" role="list">
            ${modes
              .map(
                (mode) => `
              <button class="ds-mode-option" data-mode="${mode.id}" role="listitem">
                <span style="background:${mode.color}"></span>
                <strong>${mode.label}</strong>
                <small>${mode.name}</small>
              </button>
            `
              )
              .join('')}
          </div>
          <div class="ds-mode-stage" data-sos-site="news">
            <div>
              <span class="sos-badge sos-badge--solid" id="mode-label">news</span>
              <h3 id="mode-name">春日团报</h3>
              <p id="mode-note">中性灰白、墨色主行动。黄色只作为线索、专题编号和重要标记。</p>
            </div>
            <div id="mode-sample">${modes[0].sample}</div>
          </div>
        </section>

        <section class="ds-section" id="voice">
          <div class="ds-section__header">
            <p class="sos-eyebrow">Contextual Components</p>
            <h2>同一组件，按上下文自然变声</h2>
            <p>组件 anatomy 和状态保持一致，Expression Mode 只改变语义色、圆角、表面质感和局部节奏。这样可以统一基础体验，同时让每个站点保留自己的气质。</p>
          </div>
          <div class="ds-mode-picker ds-voice-picker" role="list">
            ${modes
              .map(
                (mode) => `
              <button class="ds-mode-option ds-voice-option" data-voice="${mode.id}" role="listitem">
                <span style="background:${mode.color}"></span>
                <strong>${mode.label}</strong>
                <small>${mode.name}</small>
              </button>
            `
              )
              .join('')}
          </div>
          <article class="ds-voice-stage" data-sos-site="news">
            <header>
              <span class="sos-badge sos-badge--solid" id="voice-label">news</span>
              <div>
                <h3 id="voice-title">春日团报</h3>
                <p id="voice-note">中性灰白、墨色主行动。黄色只作为线索、专题编号和重要标记。</p>
              </div>
            </header>
            <div class="ds-voice-surface">
              <div class="ds-voice-controls">
                <button class="sos-button sos-button--primary">发布内容</button>
                <button class="sos-button sos-button--secondary">保存草稿</button>
                <span class="sos-badge sos-badge--signal">重点</span>
              </div>
              <label class="sos-field">
                <span class="sos-field__label">标题</span>
                <input class="sos-input" value="北高校园祭专题更新" aria-label="组件变声标题示例">
                <span class="sos-field__help">同一个 Field 结构，只读取当前上下文的语义 Token。</span>
              </label>
              <div class="sos-notice">
                <span class="sos-notice__icon">i</span>
                <div>
                  <h4 class="sos-notice__title">上下文提示</h4>
                  <p class="sos-notice__copy">Notice、按钮、徽章和进度条使用同一套 class，在不同站点里自然变声。</p>
                </div>
              </div>
              <div class="sos-progress">
                <div class="sos-progress__meta"><span>迁移进度</span><strong>64%</strong></div>
                <div class="sos-progress__track"><span class="sos-progress__fill" style="width:64%"></span></div>
              </div>
            </div>
          </article>
        </section>

        <section class="ds-section" id="quality">
          <div class="ds-section__header">
            <p class="sos-eyebrow">States & Responsive</p>
            <h2>状态和断点必须成为验收条件</h2>
            <p>成熟设计系统不是只有默认态。每个进入 UI 库的组件都必须有状态矩阵和响应式行为，预览页、截图和 PR 描述都要能证明。</p>
          </div>
          <div class="ds-quality-grid">
            <article class="ds-quality-panel">
              <h3>组件状态矩阵</h3>
              <div class="ds-state-list">
                ${stateRows
                  .map(
                    ([state, rule]) => `
                  <div>
                    <strong>${state}</strong>
                    <span>${rule}</span>
                  </div>
                `
                  )
                  .join('')}
              </div>
            </article>
            <article class="ds-quality-panel">
              <h3>响应式断点</h3>
              <div class="ds-responsive-list">
                ${responsiveRules
                  .map(
                    ([width, name, rule]) => `
                  <div>
                    <code>${width}</code>
                    <strong>${name}</strong>
                    <span>${rule}</span>
                  </div>
                `
                  )
                  .join('')}
              </div>
            </article>
          </div>
          <div class="ds-state-matrix" aria-label="基础组件状态矩阵">
            ${componentStateExamples
              .map(
                (component) => `
              <article class="ds-state-component">
                <header>
                  <h3>${component.name}</h3>
                  <p>${component.rule}</p>
                </header>
                <div class="ds-state-cases">
                  ${component.cases
                    .map(
                      ([label, sample]) => `
                    <div class="ds-state-case">
                      <strong>${label}</strong>
                      <div class="ds-state-preview">${sample}</div>
                    </div>
                  `
                    )
                    .join('')}
                </div>
              </article>
            `
              )
              .join('')}
          </div>
        </section>

        <section class="ds-section" id="migration">
          <div class="ds-section__header">
            <p class="sos-eyebrow">Migration</p>
            <h2>迁移证据和上线门槛</h2>
            <p>迁移不是把组件换成同名 class 就结束。每个阶段都必须留下可复查的页面证据、交互证据和回滚边界。</p>
          </div>
          <ol class="ds-timeline">
            <li><strong>Phase 0 · Inventory</strong><span>盘点旧变量、组件状态、页面模式和视觉回归基线。</span></li>
            <li><strong>Phase 1 · Token Bridge</strong><span>引入 <code>tokens.css</code>，必要时加载 <code>bridges.css</code>，第一阶段不改变外观。</span></li>
            <li><strong>Phase 2 · Shared Primitives</strong><span>按 Button、Badge、Input、Tabs、Notice、Progress 的顺序迁移。</span></li>
            <li><strong>Phase 3 · Business Compositions</strong><span>每个业务站点迁移一张代表性卡片，用真实数据验收。</span></li>
            <li><strong>Phase 4 · Page Patterns</strong><span>统一 Header、Container、FilterBar、DetailLayout、StickyAction 和系统状态。</span></li>
            <li><strong>Phase 5 · QA and Removal</strong><span>完成视觉回归、键盘、读屏、响应式、Reduced Motion 后删除旧变量。</span></li>
          </ol>
          <div class="ds-acceptance">
            ${[
              ['变更范围可解释', 'PR 说明列出接入的 app、页面、旧变量桥接范围和未迁移项。'],
              [
                '真实流程可完成',
                'news 发布/阅读、shop 下单、art 上传/审核、library 阅读、exam 答题按涉及范围至少走通一个核心路径。',
              ],
              [
                '视觉回归有证据',
                '为被改页面保存 390、768、1280px 截图；对比接入前后信息层级和关键操作位置。',
              ],
              [
                'CSS 债务没有扩散',
                '新增样式只使用设计系统 token；bridge 变量必须有删除计划，不能成为新接口。',
              ],
              [
                '交互状态完整',
                'hover、focus-visible、loading、disabled、empty、error 至少在受影响组件中可见且文案明确。',
              ],
              ['无障碍能操作', '键盘可到达主要操作，焦点不丢失；状态变化不能只靠颜色表达。'],
              [
                '移动端可用',
                '320px 无横向溢出；核心按钮和输入控件不小于 44px；购买、提交、继续答题等动作不依赖 hover。',
              ],
              [
                '回滚成本清楚',
                '能通过移除 app 入口 import 或 bridge 文件回滚视觉接入，不影响后端 API 和数据结构。',
              ],
            ]
              .map(([title, body]) => `<article><strong>${title}</strong><p>${body}</p></article>`)
              .join('')}
          </div>
        </section>
      </div>
    </div>
  </main>
`

const modeStage = document.querySelector('.ds-mode-stage')
const modeLabel = document.querySelector('#mode-label')
const modeName = document.querySelector('#mode-name')
const modeNote = document.querySelector('#mode-note')
const modeSample = document.querySelector('#mode-sample')
const modeButtons = [...document.querySelectorAll('.ds-mode-option[data-mode]')]
const voiceStage = document.querySelector('.ds-voice-stage')
const voiceLabel = document.querySelector('#voice-label')
const voiceTitle = document.querySelector('#voice-title')
const voiceNote = document.querySelector('#voice-note')
const voiceButtons = [...document.querySelectorAll('.ds-voice-option')]

function setMode(id) {
  const mode = modes.find((item) => item.id === id) || modes[0]
  modeStage.dataset.sosSite = mode.id
  modeLabel.textContent = mode.label
  modeName.textContent = mode.name
  modeNote.textContent = mode.note
  modeSample.innerHTML = mode.sample
  modeButtons.forEach((button) => {
    button.setAttribute('aria-pressed', String(button.dataset.mode === mode.id))
  })
}

modeButtons.forEach((button) => {
  button.addEventListener('click', () => setMode(button.dataset.mode))
})

setMode('news')

function setVoice(id) {
  const mode = modes.find((entry) => entry.id === id) || modes[0]
  voiceStage.dataset.sosSite = mode.id
  voiceLabel.textContent = mode.label
  voiceTitle.textContent = mode.name
  voiceNote.textContent = mode.note
  voiceButtons.forEach((button) => {
    button.setAttribute('aria-pressed', String(button.dataset.voice === mode.id))
  })
}

voiceButtons.forEach((button) => {
  button.addEventListener('click', () => setVoice(button.dataset.voice))
})

setVoice('news')

const navLinks = [...document.querySelectorAll('.ds-nav__item')]
const sections = navLinks
  .map((link) => document.querySelector(link.getAttribute('href')))
  .filter(Boolean)

const observer = new IntersectionObserver(
  (entries) => {
    const current = entries
      .filter((entry) => entry.isIntersecting)
      .sort((a, b) => b.intersectionRatio - a.intersectionRatio)[0]

    if (!current) return
    navLinks.forEach((link) => {
      link.toggleAttribute('aria-current', link.getAttribute('href') === `#${current.target.id}`)
    })
  },
  { rootMargin: '-20% 0px -70% 0px', threshold: [0.1, 0.4, 0.8] }
)

sections.forEach((section) => observer.observe(section))
