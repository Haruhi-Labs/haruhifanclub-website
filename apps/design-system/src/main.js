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
  ['tokens', 'Token 映射'],
  ['layout', '布局原语'],
  ['components', '基础组件'],
  ['contracts', '组件契约'],
  ['api', 'UI 路线'],
  ['usage', '组件用法'],
  ['expressions', '表达模式'],
  ['voice', '组件变声'],
  ['patterns', '页面模式'],
  ['a11y', '可访问性'],
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

const semanticTokens = [
  ['Page', '--sos-bg-page', '页面底色', '只用于页面根背景，不用于卡片内部。'],
  ['Surface', '--sos-bg-surface', '承载面', '卡片、表单、弹层和 Notice 的默认表面。'],
  ['Subtle', '--sos-bg-subtle', '弱承载面', '筛选条、空状态、状态格和局部衬底。'],
  ['Text Primary', '--sos-text-primary', '主文本', '标题、强数字、按钮内文和可点击主体。'],
  ['Text Secondary', '--sos-text-secondary', '辅助文本', '摘要、说明、meta、help text。'],
  [
    'Border',
    '--sos-border-subtle / default / strong',
    '边界',
    '分隔信息关系；焦点态不能只靠边框变浅。',
  ],
  ['Accent', '--sos-accent', '主行动', '每个表达模式只有一个主行动色。'],
  ['Signal', '--sos-signal', '品牌信号', '少量 Badge、编号和重点标记，不承担主 CTA。'],
  ['State', '--sos-danger / --sos-success', '状态语义', '错误和成功跨站点保持意义一致。'],
]

const expressionTokenMap = [
  ['news', '墨色主行动', '8px', 'none', '列表、长文和后台审核优先清晰扫读。'],
  ['shop', '行动蓝 CTA', '18px', '柔和交易卡片阴影', '商品图 1:1，价格和库存常驻。'],
  ['art', '画廊青主行动', '24px', '轻磨砂悬浮', '作品占视觉主位，界面只做承载。'],
  ['library', '书脊琥珀', '8px', '纸张阴影', '阅读栈、书封比例和目录连续性优先。'],
  ['exam', '批改红', '12px', '试卷纸张阴影', '题目、选择、倒计时和批阅状态稳定。'],
]

const tokenBoundaries = [
  ['Do', '业务 CSS 使用 Semantic Token：背景、文字、边框、主行动和状态都从语义变量读取。'],
  ['Do', '需要站点气质时改 Expression Mapping，不在组件里按站点写选择器覆盖物理色值。'],
  ['Don’t', '不要在业务卡片里新增 hex、临时阴影、13px 圆角或非 4px 网格间距。'],
  ['Don’t', '不要把 Signal Yellow 当主按钮色、价格色或错误色；它只负责品牌线索。'],
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

const uiMaturityLevels = [
  [
    'L0',
    'Token / Class Contract',
    'CSS 包',
    '颜色、间距、圆角、布局原语和基础 class。只要业务还在迁移，就允许先以 CSS-first 接入。',
  ],
  [
    'L1',
    'Primitive Wrapper',
    '@haruhi/ui',
    'Button、Badge、Field、Notice、Progress、Card anatomy 和布局 wrapper。只封装稳定 props、slot、状态和可访问性。',
  ],
  [
    'L2',
    'Composition Recipe',
    '规范页 + 业务 app',
    '新闻卡、商品卡、作品卡、书封卡、试卷卡先作为 recipe 验证。它们共享基础件，但不急着跨站抽象。',
  ],
  [
    'L3',
    'Product Component',
    '未来评估',
    '至少三个页面反复出现同一信息结构、状态和数据契约后，才考虑从 recipe 升级为共享业务组件。',
  ],
]

const uiApiRules = [
  ['Props', '只暴露稳定语义：variant、size、tone、ratio、gap、selected、loading。'],
  ['Slots', 'Slot 对应 anatomy 槽位；不能用任意 slot 绕过结构约束。'],
  ['Styling', '组件不接受 color、shadow、radius 等视觉 props；这些由 Semantic Token 决定。'],
  ['State', '状态 props 必须同步 aria、disabled、aria-busy、aria-invalid 等可访问性证据。'],
  ['Upgrade', '新增组件或 variant 前，必须先更新规范页、状态矩阵和响应式截图证据。'],
]

const uiDecisionMatrix = [
  [
    'SosButton / SosBadge / SosField',
    'L1 · UI Wrapper',
    '跨站重复、语义稳定、状态明确，可以由 Vue wrapper 输出统一 class。',
  ],
  [
    'SosCard / SosMediaFrame',
    'L1 · Anatomy Wrapper',
    '只封装边界、媒体比例和 body/footer 槽位，不决定新闻、商品或作品信息结构。',
  ],
  [
    'SosStack / Inline / Grid / Split',
    'L1 · Layout Wrapper',
    '允许 gap、min、ratio 等布局参数；不暴露颜色和材质。',
  ],
  [
    'ShopProductCard / ArtworkCard',
    'L2 · Recipe',
    '需要真实数据、媒体比例和流程状态继续验证，暂不进入共享 UI 包。',
  ],
  [
    'CheckoutRail / ExamQuestion',
    'L3 · Candidate',
    '只有当多页面共享同一数据契约和状态机时，才进入产品组件评估。',
  ],
]

const componentGuides = [
  {
    id: 'button',
    label: 'Button',
    title: 'Button 命令按钮',
    role: '用于提交、保存、购买、删除等明确命令。',
    summary:
      'Button 的责任是让用户完成一个动作。它不承担状态标签、普通导航或长段说明；状态必须通过 disabled、aria-busy、focus-visible 等可复查线索呈现。',
    anatomy: [
      'Root：button / a.sos-button',
      'Variant：primary / secondary / ghost / danger',
      'Size：sm / md / lg',
      'State：hover / active / focus / loading / disabled',
    ],
    do: [
      '按钮文案用动词开头。',
      'Loading 保留当前标签和尺寸。',
      '同一视口只保留一个最高行动层级。',
    ],
    dont: [
      '不要用 Badge 或 Card 替代按钮。',
      '不要让关键动作只在 hover 出现。',
      '不要把危险操作做成普通 primary。',
    ],
    sample: `
      <div class="ds-guide-button-row">
        <button class="sos-button sos-button--primary">提交审核</button>
        <button class="sos-button sos-button--secondary">保存草稿</button>
        <button class="sos-button sos-button--danger">删除</button>
      </div>
    `,
    code: `<SosButton variant="primary">提交审核</SosButton>
<SosButton variant="secondary">保存草稿</SosButton>`,
  },
  {
    id: 'badge',
    label: 'Badge',
    title: 'Badge 短状态',
    role: '用于分类、筛选命中、短状态和少量品牌信号。',
    summary:
      'Badge 是短标签，不是标题也不是按钮。它必须保持小面积、短文案和明确语义；selected 和 disabled 只能作为当前上下文状态，不制造新交互模式。',
    anatomy: [
      'Root：span.sos-badge',
      'Variant：default / accent / solid / outline / signal',
      'State：selected / disabled by parent',
      'Content：1-6 个字优先',
    ],
    do: [
      '一屏内 Signal Badge 控制数量。',
      '用 selected 表示当前筛选或选择。',
      '用 outline 表示低强调分类。',
    ],
    dont: ['不要承载句子或段落标题。', '不要把 Badge 当按钮用。', '不要让彩色 Badge 铺满卡片。'],
    sample: `
      <div class="sos-inline">
        <span class="sos-badge">普通</span>
        <span class="sos-badge sos-badge--accent">分类</span>
        <span class="sos-badge" aria-selected="true">已选</span>
        <span class="sos-badge sos-badge--signal">重点</span>
      </div>
    `,
    code: `<SosBadge variant="accent">分类</SosBadge>
<SosBadge selected>已选</SosBadge>`,
  },
  {
    id: 'field',
    label: 'Field',
    title: 'Field 表单字段',
    role: '用于把 label、control、help/error 绑定成稳定输入单元。',
    summary:
      'Field 先保证信息关系，再讨论视觉。Label 不被 placeholder 替代；错误态必须给出文字证据，帮助文字不能在错误时消失。',
    anatomy: [
      'Root：.sos-field',
      'Label：.sos-field__label',
      'Control：.sos-input / .sos-textarea / .sos-select',
      'Evidence：.sos-field__help / error text',
    ],
    do: ['Label 永远可见。', 'Error 同时使用文字和边界。', '移动端使用合适 input type。'],
    dont: [
      '不要只用 placeholder 说明字段。',
      '不要只把边框改红。',
      '不要把多个无关控件塞进一个 Field。',
    ],
    sample: `
      <label class="sos-field">
        <span class="sos-field__label">页面标题</span>
        <input class="sos-input" value="北高校园祭专题">
        <span class="sos-field__help">标题会显示在列表、分享卡片和详情页。</span>
      </label>
    `,
    code: `<SosField label="页面标题" help="标题会显示在列表、分享卡片和详情页。">
  <input class="sos-input" v-model="title" />
</SosField>`,
  },
  {
    id: 'notice',
    label: 'Notice',
    title: 'Notice 系统提示',
    role: '用于页面内常驻提示、流程反馈和可复查的系统状态。',
    summary:
      'Notice 保留上下文，不抢走用户当前任务。它和 Toast、Dialog 分工不同：需要长期阅读或比较的信息不能放进自动消失的 Toast。',
    anatomy: [
      'Root：.sos-notice',
      'Icon：.sos-notice__icon',
      'Title：.sos-notice__title',
      'Copy：.sos-notice__copy',
      'Optional action slot',
    ],
    do: [
      '标题先说明结果或风险。',
      '正文给出影响和下一步。',
      'Tone 使用 info / success / warning / danger。',
    ],
    dont: [
      '不要用 Notice 写普通营销文案。',
      '不要只用颜色表达风险。',
      '不要用 Toast 承载需要复制的信息。',
    ],
    sample: `
      <div class="sos-notice sos-notice--warning">
        <span class="sos-notice__icon">!</span>
        <div>
          <h4 class="sos-notice__title">库存偏低</h4>
          <p class="sos-notice__copy">继续售卖前请确认补货计划。</p>
        </div>
      </div>
    `,
    code: `<SosNotice tone="warning" title="库存偏低">
  继续售卖前请确认补货计划。
</SosNotice>`,
  },
  {
    id: 'progress',
    label: 'Progress',
    title: 'Progress 进度反馈',
    role: '用于预售、上传、答题完成度等可量化流程。',
    summary:
      'Progress 必须让用户知道当前进度、单位和结果。颜色可以增强语义，但完成、错误、零进度都要有文字或数字证据。',
    anatomy: [
      'Root：.sos-progress',
      'Meta：label + value',
      'Track：.sos-progress__track',
      'Fill：.sos-progress__fill',
      'Tone：default / success / danger',
    ],
    do: ['常驻显示数字或可读描述。', '错误进度说明失败位置。', '预售类数字使用等宽数字。'],
    dont: [
      '不要只有一条彩色线。',
      '不要让颜色含义随站点变化。',
      '不要把不可估算加载伪装成精确百分比。',
    ],
    sample: `
      <div class="sos-progress">
        <div class="sos-progress__meta"><span>预售进度</span><strong>126/200</strong></div>
        <div class="sos-progress__track"><span class="sos-progress__fill" style="width:63%"></span></div>
      </div>
    `,
    code: `<SosProgress :value="126" :max="200" label="预售进度" value-label="126/200" />`,
  },
  {
    id: 'card',
    label: 'Card',
    title: 'Card 内容容器',
    role: '用于承载一组可扫读内容、状态和操作。',
    summary:
      'Card 只负责边界、层级和 anatomy，不决定业务结构。新闻、商品、作品、书籍、试卷卡片先作为 recipe 验证，不直接抽成统一业务组件。',
    anatomy: [
      'Root：.sos-card',
      'Media slot：image / frame / cover',
      'Body：.sos-card__body',
      'Footer：.sos-card__footer',
      'State：interactive / selected / loading',
    ],
    do: [
      '标题、状态和主信息常驻。',
      '媒体比例交给 MediaFrame。',
      'Footer 用 Cluster 思路处理操作。',
    ],
    dont: [
      '不要靠 hover 才显示价格或库存。',
      '不要在页面临时改内部 padding。',
      '不要把所有业务卡片强行做成同一比例。',
    ],
    sample: `
      <article class="sos-card sos-card--interactive">
        <div class="sos-card__body">
          <h4 class="ds-state-card-title">待审核稿件</h4>
          <p class="ds-state-card-copy">标题、摘要、日期和状态常驻显示。</p>
          <footer class="sos-card__footer">
            <span class="sos-badge sos-badge--outline">待处理</span>
            <button class="sos-button sos-button--secondary sos-button--sm">查看</button>
          </footer>
        </div>
      </article>
    `,
    code: `<SosCard interactive>
  <h3>待审核稿件</h3>
  <template #footer>
    <SosBadge variant="outline">待处理</SosBadge>
  </template>
</SosCard>`,
  },
  {
    id: 'empty',
    label: 'EmptyState',
    title: 'EmptyState 空状态',
    role: '用于无数据、无结果、权限缺失等系统状态。',
    summary:
      '空状态不是插画展位。它要解释为什么为空、用户还能做什么，以及下一步是否会改变当前状态。',
    anatomy: [
      'Root：.sos-empty-state',
      'Icon：optional status mark',
      'Title：原因',
      'Copy：解释和影响',
      'Actions：一个主行动优先',
    ],
    do: ['说明为空原因。', '给出清除筛选、登录、返回全部等真实动作。', '保持语气具体，避免口号。'],
    dont: ['不要只放插画和“暂无数据”。', '不要塞多个竞争 CTA。', '不要把空状态做成营销首页。'],
    sample: `
      <section class="sos-empty-state">
        <span class="sos-empty-state__icon">0</span>
        <h4 class="sos-empty-state__title">暂无投稿</h4>
        <p class="sos-empty-state__copy">当前筛选条件下没有作品。</p>
        <div class="sos-empty-state__actions">
          <button class="sos-button sos-button--secondary sos-button--sm">清除筛选</button>
        </div>
      </section>
    `,
    code: `<SosEmptyState title="暂无投稿" copy="当前筛选条件下没有作品。">
  <template #actions>
    <SosButton variant="secondary" size="sm">清除筛选</SosButton>
  </template>
</SosEmptyState>`,
  },
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

const qaEvidenceGroups = [
  [
    'Scope',
    '变更范围',
    '列出 app、路由、组件、旧变量 bridge、未迁移项和回滚入口。',
    'PR 描述 + diff 链接',
  ],
  [
    'Visual',
    '视觉回归',
    '保存 390 / 768 / 1280px 前后截图，检查首屏主操作、关键数据和信息层级。',
    'Playwright 截图',
  ],
  [
    'Interaction',
    '交互状态',
    '键盘路径、hover、focus-visible、loading、disabled、empty、error 都有可见证据。',
    '状态矩阵截图',
  ],
  [
    'A11y',
    '可访问性',
    '验证触控目标、读屏名称、aria-live、Reduced Motion、Forced Colors 和 200% Zoom。',
    '检查记录 + 截图',
  ],
  [
    'CSS Debt',
    '样式债务',
    '新增样式不写 raw hex、临时阴影、临时圆角；bridge 变量有 owner 和删除计划。',
    'rg 输出 + 说明',
  ],
  [
    'Rollback',
    '回滚边界',
    '说明如何移除入口 import 或 bridge 文件回滚视觉接入，不影响 API 和数据结构。',
    '回滚步骤',
  ],
]

const qaGateLevels = [
  ['Pass', '可以合并', '所有核心路径可完成；无横向溢出；截图、状态、a11y 和 token 证据齐全。'],
  ['Review', '需要设计复核', '存在轻微视觉差异，但不影响核心任务、状态证据、响应式和回滚边界。'],
  [
    'Block',
    '不得上线',
    '核心路径断裂、移动端依赖 hover、状态只靠颜色、关键数据消失或新增 CSS 债务。',
  ],
]

const pagePatterns = [
  [
    'AppShell',
    'BrandLockup / Nav / Account / Footer',
    '全站 Header 使用同一 logo + 标题组合；站点差异体现在文字和语义色，不重新造壳。',
  ],
  [
    'ChannelHeader',
    'Title / Description / Primary action / Meta',
    '频道名必须成为首屏信号；说明文字解释当前页面任务，不写营销口号。',
  ],
  [
    'FilterBar',
    'Search / Category / Sort / Result count',
    '筛选和结果数量在同一区域，移动端可换行或折叠，但顺序不反转。',
  ],
  [
    'ContentGrid',
    'Grid / Card recipe / Empty / Loading',
    '列数由 `--sos-grid-min` 决定，不能为了塞列数压缩标题、价格或按钮。',
  ],
  [
    'DetailLayout',
    'Main content / Rail / Related action',
    '双栏只在空间足够时出现；窄屏 Rail 下移，不遮挡正文或媒体。',
  ],
  [
    'StickyAction',
    'Current state / Primary action / Safe area',
    '移动端核心动作可 sticky，但不能盖住内容，也不能替代页面内状态说明。',
  ],
  [
    'SystemState',
    'Notice / EmptyState / Progress / Error',
    '系统状态跨站统一体验，只让 tone 和上下文文案变化。',
  ],
]

const a11yGates = [
  ['Keyboard', 'Tab 顺序与视觉顺序一致；focus-visible 明确且不被 sticky 或浮层遮挡。'],
  ['Touch', '核心点击目标不小于 44px；主 CTA 建议 48px；相邻按钮保留 8px 以上距离。'],
  ['State Evidence', '错误、成功、禁用和加载状态不能只靠颜色，必须有文字、图标、位置或形状证据。'],
  ['ARIA', 'Toast 使用 aria-live；Dialog 管理焦点；状态变化有可读名称。'],
  ['Reduced Motion', '位移、旋转、视差降级；信息时序和操作反馈不改变。'],
  ['Forced Colors', '边框、文字、焦点和可点击区域在强制颜色模式仍可辨认。'],
  ['Zoom', '200% 缩放不遮挡正文、表单、按钮和关键状态。'],
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

        <section class="ds-section" id="tokens">
          <div class="ds-section__header">
            <p class="sos-eyebrow">Token Map</p>
            <h2>语义 Token 先于组件抽象</h2>
            <p>成熟 UI 库不能建立在散落的颜色、阴影和间距上。v0.2 先把业务可消费的语义变量固定下来，组件只读取语义变量，站点差异只通过 Expression Mapping 变声。</p>
          </div>
          <div class="ds-token-map">
            ${semanticTokens
              .map(
                ([name, token, role, usage]) => `
              <article class="ds-token-row">
                <span>${name}</span>
                <code>${token}</code>
                <strong>${role}</strong>
                <p>${usage}</p>
              </article>
            `
              )
              .join('')}
          </div>
          <div class="ds-token-workbench">
            <article class="ds-expression-map">
              <h3>Expression Mapping 只改表达，不改组件结构</h3>
              ${expressionTokenMap
                .map(
                  ([mode, accent, radius, shadow, rule]) => `
                <div class="ds-expression-row" data-sos-site="${mode}">
                  <span class="sos-badge sos-badge--solid">${mode}</span>
                  <strong>${accent}</strong>
                  <code>${radius}</code>
                  <code>${shadow}</code>
                  <p>${rule}</p>
                </div>
              `
                )
                .join('')}
            </article>
            <article class="ds-token-code-card">
              <h3>业务代码读取语义变量</h3>
              <pre class="ds-code ds-code--compact"><code>.product-card {
  background: var(--sos-bg-surface);
  color: var(--sos-text-primary);
  border: 1px solid var(--sos-border-subtle);
  border-radius: var(--sos-card-radius);
  box-shadow: var(--sos-card-shadow);
}

.product-card__price {
  color: var(--sos-link);
  font-variant-numeric: tabular-nums;
}</code></pre>
              <div class="ds-token-boundaries">
                ${tokenBoundaries
                  .map(
                    ([label, copy]) => `
                  <div>
                    <span>${label}</span>
                    <p>${copy}</p>
                  </div>
                `
                  )
                  .join('')}
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

        <section class="ds-section" id="api">
          <div class="ds-section__header">
            <p class="sos-eyebrow">UI Library Roadmap</p>
            <h2>组件进入 UI 库前先分级</h2>
            <p>UI 库不是把所有重复 UI 都收进去。v0.2 用四个成熟度层级管理抽象边界：先稳定 CSS 契约，再封装基础 wrapper，业务组合继续作为 recipe 验证，最后才评估产品组件。</p>
          </div>
          <div class="ds-maturity-grid">
            ${uiMaturityLevels
              .map(
                ([level, title, owner, copy]) => `
              <article class="ds-maturity-card">
                <span>${level}</span>
                <h3>${title}</h3>
                <strong>${owner}</strong>
                <p>${copy}</p>
              </article>
            `
              )
              .join('')}
          </div>
          <div class="ds-api-lab">
            <article class="ds-api-rules">
              <h3>API 设计守则</h3>
              ${uiApiRules
                .map(
                  ([term, description]) => `
                <div>
                  <strong>${term}</strong>
                  <p>${description}</p>
                </div>
              `
                )
                .join('')}
            </article>
            <article class="ds-ui-decision">
              <h3>当前去留判断</h3>
              ${uiDecisionMatrix
                .map(
                  ([name, level, reason]) => `
                <div>
                  <code>${name}</code>
                  <span>${level}</span>
                  <p>${reason}</p>
                </div>
              `
                )
                .join('')}
            </article>
          </div>
        </section>

        <section class="ds-section" id="usage">
          <div class="ds-section__header">
            <p class="sos-eyebrow">Component Usage</p>
            <h2>基础组件必须能被说明和复用</h2>
            <p>成熟设计系统不只展示默认态，还要说明组件边界、Anatomy、使用条件和禁止用法。这里的示例直接对应 <code>@haruhi/ui</code> MVP，不把业务卡片提前抽象。</p>
          </div>
          <div class="ds-guide-shell">
            <div class="ds-guide-picker" role="list" aria-label="基础组件用法目录">
              ${componentGuides
                .map(
                  (guide) => `
                <button class="ds-guide-option" data-guide="${guide.id}" role="listitem">
                  <strong>${guide.label}</strong>
                  <span>${guide.role}</span>
                </button>
              `
                )
                .join('')}
            </div>
            <article class="ds-guide-panel">
              <div class="ds-guide-content">
                <span class="sos-badge sos-badge--signal" id="guide-label">${componentGuides[0].label}</span>
                <h3 id="guide-title">${componentGuides[0].title}</h3>
                <p id="guide-summary">${componentGuides[0].summary}</p>
                <div class="ds-guide-anatomy">
                  <strong>Anatomy</strong>
                  <ul id="guide-anatomy">
                    ${componentGuides[0].anatomy.map((item) => `<li>${item}</li>`).join('')}
                  </ul>
                </div>
                <div class="ds-guide-rules">
                  <div>
                    <strong>Do</strong>
                    <ul id="guide-do">${componentGuides[0].do.map((item) => `<li>${item}</li>`).join('')}</ul>
                  </div>
                  <div>
                    <strong>Don't</strong>
                    <ul id="guide-dont">${componentGuides[0].dont.map((item) => `<li>${item}</li>`).join('')}</ul>
                  </div>
                </div>
              </div>
              <div class="ds-guide-live">
                <div class="ds-guide-sample" id="guide-sample">${componentGuides[0].sample}</div>
                <pre class="ds-code ds-code--compact"><code id="guide-code">${componentGuides[0].code}</code></pre>
              </div>
            </article>
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

        <section class="ds-section" id="patterns">
          <div class="ds-section__header">
            <p class="sos-eyebrow">Page Patterns</p>
            <h2>页面模式把组件变成真实流程</h2>
            <p>页面模式不是新组件库，而是迁移时的结构协议。它要求 Header、频道头、筛选、网格、详情 Rail、Sticky Action 和系统状态在不同站点中保持一致的信息顺序。</p>
          </div>
          <div class="ds-pattern-grid">
            ${pagePatterns
              .map(
                ([name, anatomy, rule]) => `
              <article class="ds-pattern-card">
                <span>${name}</span>
                <strong>${anatomy}</strong>
                <p>${rule}</p>
              </article>
            `
              )
              .join('')}
          </div>
          <article class="ds-page-lab" data-sos-site="shop">
            <header class="ds-page-lab__bar">
              <div class="sos-brand-lockup sos-brand-lockup--compact">
                <span class="sos-brand-lockup__mark"><img src="${logoUrl}" alt=""></span>
                <span class="sos-brand-lockup__text">
                  <strong>春日商城</strong>
                  <small>预售、订单与周边</small>
                </span>
              </div>
              <div class="sos-inline">
                <button class="sos-button sos-button--ghost sos-button--sm">订单</button>
                <button class="sos-button sos-button--primary sos-button--sm">发布预售</button>
              </div>
            </header>
            <section class="ds-page-lab__hero">
              <div>
                <span class="sos-badge sos-badge--signal">ChannelHeader</span>
                <h3>本周预售清单</h3>
                <p>用真实信息结构验收页面模式：标题、筛选、结果、状态和操作都必须在窄屏可用。</p>
              </div>
              <button class="sos-button sos-button--secondary">导出订单</button>
            </section>
            <section class="ds-page-lab__filters" aria-label="页面模式筛选示例">
              <label class="sos-field">
                <span class="sos-field__label">搜索商品</span>
                <input class="sos-input" value="SOS 团限定徽章" aria-label="搜索商品">
              </label>
              <div class="sos-inline">
                <span class="sos-badge" aria-selected="true">全部</span>
                <span class="sos-badge sos-badge--outline">预售中</span>
                <span class="sos-badge sos-badge--outline">待补货</span>
              </div>
              <strong>24 个结果</strong>
            </section>
            <div class="ds-page-lab__body">
              <div class="ds-page-lab__grid">
                ${[
                  ['限定徽章', '预售 126/200', '63%'],
                  ['团长臂章', '库存 18', '92%'],
                  ['活动票根套装', '待补货', '0%'],
                ]
                  .map(
                    ([title, status, progress]) => `
                  <article class="sos-card">
                    <div class="sos-card__body">
                      <h4 class="ds-state-card-title">${title}</h4>
                      <p class="ds-state-card-copy">${status}</p>
                      <div class="sos-progress">
                        <div class="sos-progress__meta"><span>状态</span><strong>${progress}</strong></div>
                        <div class="sos-progress__track"><span class="sos-progress__fill" style="width:${progress}"></span></div>
                      </div>
                    </div>
                  </article>
                `
                  )
                  .join('')}
              </div>
              <aside class="ds-page-lab__rail">
                <div class="sos-notice">
                  <span class="sos-notice__icon">i</span>
                  <div>
                    <h4 class="sos-notice__title">迁移验收点</h4>
                    <p class="sos-notice__copy">筛选、结果数、进度和主操作在 390px 下仍保持顺序。</p>
                  </div>
                </div>
                <section class="sos-empty-state">
                  <span class="sos-empty-state__icon">?</span>
                  <h4 class="sos-empty-state__title">无匹配库存</h4>
                  <p class="sos-empty-state__copy">切换筛选后必须说明原因，并提供返回动作。</p>
                </section>
              </aside>
            </div>
            <footer class="ds-page-lab__sticky">
              <span>已选 3 件商品</span>
              <button class="sos-button sos-button--primary sos-button--sm">继续处理</button>
            </footer>
          </article>
        </section>

        <section class="ds-section" id="a11y">
          <div class="ds-section__header">
            <p class="sos-eyebrow">Accessibility Gates</p>
            <h2>可访问性是上线门槛，不是最后修饰</h2>
            <p>设计系统接入必须证明键盘、触控、状态证据、读屏公告、Reduced Motion、Forced Colors 和 200% 缩放都可用。这里的验收样例用于 PR 截图和回归检查。</p>
          </div>
          <div class="ds-a11y-grid">
            ${a11yGates
              .map(
                ([name, rule]) => `
              <article class="ds-a11y-card">
                <strong>${name}</strong>
                <p>${rule}</p>
              </article>
            `
              )
              .join('')}
          </div>
          <article class="ds-a11y-lab">
            <section>
              <span class="sos-badge sos-badge--signal">Keyboard</span>
              <h3>焦点路径清楚</h3>
              <div class="ds-a11y-focus-row" aria-label="焦点路径示例">
                <button class="sos-button sos-button--secondary" data-state="focus">1 保存草稿</button>
                <button class="sos-button sos-button--primary">2 提交审核</button>
                <a class="sos-button sos-button--ghost" href="#migration">3 查看迁移门槛</a>
              </div>
            </section>
            <section>
              <span class="sos-badge sos-badge--signal">Touch</span>
              <h3>触控目标稳定</h3>
              <div class="ds-touch-targets">
                <button class="sos-button sos-button--sm">36px 次要</button>
                <button class="sos-button sos-button--secondary">44px 默认</button>
                <button class="sos-button sos-button--primary sos-button--lg">48px 主行动</button>
              </div>
            </section>
            <section>
              <span class="sos-badge sos-badge--signal">Evidence</span>
              <h3>状态不只靠颜色</h3>
              <div class="sos-notice sos-notice--danger">
                <span class="sos-notice__icon">!</span>
                <div>
                  <h4 class="sos-notice__title">提交失败</h4>
                  <p class="sos-notice__copy">网络超时，请保留当前内容后重试。错误有图标、标题、正文和边框证据。</p>
                </div>
              </div>
            </section>
            <section>
              <span class="sos-badge sos-badge--signal">Live Region</span>
              <h3>读屏公告有语义</h3>
              <div class="ds-live-region" aria-live="polite">
                <strong>草稿已保存</strong>
                <span>aria-live="polite" 用于非阻断结果；需要决策时改用 Dialog。</span>
              </div>
            </section>
            <section class="ds-a11y-wide">
              <span class="sos-badge sos-badge--signal">Fallback</span>
              <h3>降级模式要保留信息</h3>
              <div class="ds-fallback-grid">
                <div>
                  <strong>Reduced Motion</strong>
                  <p>去掉位移和旋转，保留加载、完成、错误文案。</p>
                </div>
                <div>
                  <strong>Forced Colors</strong>
                  <p>边框和焦点仍可见，不依赖背景渐变。</p>
                </div>
                <div>
                  <strong>200% Zoom</strong>
                  <p>文本换行后不遮挡按钮、输入框和状态区域。</p>
                </div>
              </div>
            </section>
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
          <div class="ds-qa-board">
            <article class="ds-qa-evidence">
              <h3>PR 证据包</h3>
              <div class="ds-qa-evidence-grid">
                ${qaEvidenceGroups
                  .map(
                    ([label, title, copy, proof]) => `
                  <div>
                    <span>${label}</span>
                    <strong>${title}</strong>
                    <p>${copy}</p>
                    <code>${proof}</code>
                  </div>
                `
                  )
                  .join('')}
              </div>
            </article>
            <article class="ds-qa-gates">
              <h3>合并门槛</h3>
              ${qaGateLevels
                .map(
                  ([level, title, copy]) => `
                <div>
                  <span>${level}</span>
                  <strong>${title}</strong>
                  <p>${copy}</p>
                </div>
              `
                )
                .join('')}
            </article>
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
const guideButtons = [...document.querySelectorAll('.ds-guide-option')]
const guideLabel = document.querySelector('#guide-label')
const guideTitle = document.querySelector('#guide-title')
const guideSummary = document.querySelector('#guide-summary')
const guideAnatomy = document.querySelector('#guide-anatomy')
const guideDo = document.querySelector('#guide-do')
const guideDont = document.querySelector('#guide-dont')
const guideSample = document.querySelector('#guide-sample')
const guideCode = document.querySelector('#guide-code')
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

function setGuide(id) {
  const guide = componentGuides.find((item) => item.id === id) || componentGuides[0]
  guideLabel.textContent = guide.label
  guideTitle.textContent = guide.title
  guideSummary.textContent = guide.summary
  guideAnatomy.innerHTML = guide.anatomy.map((item) => `<li>${item}</li>`).join('')
  guideDo.innerHTML = guide.do.map((item) => `<li>${item}</li>`).join('')
  guideDont.innerHTML = guide.dont.map((item) => `<li>${item}</li>`).join('')
  guideSample.innerHTML = guide.sample
  guideCode.textContent = guide.code
  guideButtons.forEach((button) => {
    const isActive = button.dataset.guide === guide.id
    button.setAttribute('aria-pressed', String(isActive))
    if (isActive) button.scrollIntoView({ block: 'nearest', inline: 'nearest' })
  })
}

guideButtons.forEach((button) => {
  button.addEventListener('click', () => setGuide(button.dataset.guide))
})

setGuide('button')

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
