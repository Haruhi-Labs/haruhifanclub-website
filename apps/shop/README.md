# @haruhi/shop · 春日商城前端

春日应援团的周边电商前端：商品浏览 / 购物车 / 下单 / 在线支付 / 订单查询的前台，外加一套订单·商品·优惠券·留言·统计·站点配置的后台管理台。是本仓库 6 个前端 app 中业务最复杂的一个（预售、子订单分组发货、CSV 物流导入导出、优惠券批次等）。开发端口 `5205`，构建后部署在子路径 `/shop/`。

## 技术栈与关键依赖

- **Vue 3**（`^3.5`，纯 `<script setup>` + Composition API）+ **vue-router 4**（`createWebHistory`）。
- **状态管理：不用 Pinia**，而是单例 `reactive()` store（`src/stores/shopStore.js`），通过 `useShopStore()` 暴露 state、computed 与一组 async action。
- **chart.js**（`^4.4`，`chart.js/auto`）：后台统计趋势图（`views/admin/StatsView.vue`）。
- **cropperjs**（`^1.6`）：后台商品图裁剪（`views/admin/ProductsView.vue`）。
- **@haruhi/api-client**（`workspace:*`）：共享鉴权与上传地址解析，见下文「与共享层 / 后端的关系」。
- 构建：**Vite 7** + `@vitejs/plugin-vue`，别名 `@` → `src`。无 TypeScript（纯 JS）、无独立 CSS 框架（手写 `assets/shop.css` 与 `assets/admin.css`）。

## 目录结构要点

```
src/
  main.js                应用入口（createApp + router）
  App.vue                根组件
  router/index.js        路由表 + 守卫（后台路由统一 JWT + shop 权限校验）+ 前台埋点
  stores/shopStore.js    全局 reactive store：购物车 / 商品 / 订单 / 优惠券 / 留言 / 站点配置
  layouts/
    ShopLayout.vue       前台外壳
    AdminLayout.vue      后台外壳
  views/
    shop/                前台：Home / ProductDetail / Cart / Checkout / Payment / Success / OrderQuery / Contact
    admin/               后台：AdminLogin / Dashboard / Orders / Messages / Products / Coupons / Stats / Settings
  utils/
    runtimePaths.js      子路径 / API 前缀解析（resolveAppPath / resolveApiPath，统一前缀 /api/shop）
    adminAuth.js         后台鉴权薄封装，委托 createAdminAuth('shop')
    analytics.js         前台埋点（sendBeacon → /api/shop/analytics/event）
    chinaDivision.js     省市区联动数据（从 CDN 拉 pcas-code.json）
  components/TheFooter.vue
```

## 本地开发

需要先在仓库根启动统一后端：`cargo run -p haruhi-server`（监听 `127.0.0.1:17777`）。

```bash
pnpm --filter @haruhi/shop dev       # 启动 dev，端口 5205
pnpm --filter @haruhi/shop build     # 生产构建到 dist/
pnpm --filter @haruhi/shop preview    # 预览构建产物
```

- `vite.config.js` 中 `base: '/shop/'`，dev server 端口 `5205`，并把 `/api`、`/uploads` 代理到 `http://127.0.0.1:17777`，所以本地无需配置跨域。
- 访问前台 `http://localhost:5205/shop/`，后台 `http://localhost:5205/shop/admin`（未登录会跳转 `/shop/admin/login`）。

## 关键特性与约定

- **runtimePaths 统一路径**：所有 API 走 `resolveApiPath()`，模块前缀固定 `/api/shop`（旧 `/shop-api` 已废弃）；前端路由 / 跳转走 `resolveAppPath()`，自动带上 `BASE_URL` 子路径。改子路径只需改 Vite `base`，代码无需动。
- **金额按「分」运算**：store 内 `toCents` / `fromCents` 全程整数运算，规避浮点误差；前台不存价，价格随商品快照进购物车。
- **运费规则**：按 `shippingTag` 分组取组内最大运费再累加，满 `150` 元包邮（`FREE_SHIPPING_THRESHOLD`，store 以 `freeShippingThreshold` 导出）。
- **预售**：商品支持 `presaleMode` = `none` / `goal`（凑单目标）/ `fixed`（固定发货时间，月初 / 月底 / 具体日期）；store 提供 `isPresaleProduct` / `getPresaleProgress` / `formatFixedPresaleDate`。后台可单独 `adjustProductMetrics` 微调库存与预售已付数。
- **订单状态流**：前台下单（`createOrderBackend`）→ 支付页扫码后点「我已完成支付」提交（`submitOrderPayment`，POST `/api/shop/orders/{id}/payment`，待付款 → 待确认）→ 后台 `updateOrderStatus` 流转 / 改收货信息 / 删除。
- **子订单分组发货**：一单可拆为现货 / 预售等多个子订单，`shipSubOrder(orderId, subKey, tracking)` 按子订单维度发货。
- **CSV 物流闭环**：后台导出待发货数据、`importTracking` 批量导入快递单号，并用 `markOrdersExported` / `markOrdersSpotExported` / `markPresaleExported` 标记导出状态，避免重复导出。
- **优惠券**：批次创建（`createCouponBatch`）、按状态 / 批次号 / 关键字筛选分页、启停与删除；前台结算页 `previewCoupon` 实时校验可用性与抵扣金额。
- **后台图片上传**：商品图等在浏览器侧用 canvas 转 **WebP** 并按 `maxDimension` 压缩（`store.uploadImage`，二维码 / 原图等 `convertToWebp: false` 的 `purpose` 例外不转），再传后端 `/api/shop/upload`。
- **支付收款**：前台支付页（`PaymentView`）展示后台在「站点配置」上传的微信 / 支付宝 / 好友收款二维码，买家扫码后手动确认完成支付。
- **埋点**：前台路由切换通过 `analytics.js` 以 `navigator.sendBeacon` 上报，匿名 session id 存 localStorage。
- **留言板**：前台 `Contact` 提交留言，后台 `Messages` 分页处理与改状态。

## 与共享层 / 后端的关系

- **鉴权全部委托共享层**：`utils/adminAuth.js` 仅薄封装 `@haruhi/api-client` 的 `createAdminAuth('shop')`，统一走 `/api/auth/login` 单点 JWT，登录后校验是否具备 shop 管理权限（超管或被授予 shop 角色）。token 由 api-client 的 `getToken/setToken/clearToken` 统一存取；后台请求头由 `buildAdminAuthHeaders()` 注入 `Authorization: Bearer`。
- **路由守卫**：`router/index.js` 对 `/admin/*` 路由先做本地 token 有效性判断，再异步 `verifyShopAccess()`（命中 `/api/auth/me`）确认权限，失效则跳登录页并带 `redirect`。
- **后端契约**：对接统一后端 `haruhi-server` 的 `/api/shop/*`（商品 / 订单 / 优惠券 / 站点配置 / 留言 / 埋点 / 上传）；图片等静态资源走 `/uploads/shop/*`。下单 / 留言触发的邮件通知由后端 `haruhi-mail` 的邮件队列异步发送，前端只负责提交收件邮箱。

## 更多

- 整体架构、工具链与提交规范见仓库根 [README](../../README.md)。
- 协作流程与贡献规范见 [CONTRIBUTING](../../CONTRIBUTING.md) 与 [docs/COLLABORATION.md](../../docs/COLLABORATION.md)。
- 提交 scope 用 `shop`。
