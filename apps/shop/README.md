# @haruhi/shop

商城 app，包含商品浏览、购物车、下单、支付确认、订单查询，以及商品、订单、优惠券、留言、统计和站点配置后台。

## 入口

- 子路径：`/shop/`
- dev 端口：`5205`
- 后端接口：`/api/shop/*`
- 上传资源：`/uploads/shop/*`
- 技术栈：Vue 3、Vue Router、Vite、Chart.js、Cropper.js、`@haruhi/api-client`

## 本地运行

```bash
bash deploy/gen-secrets.sh
pnpm dev:backend

pnpm --filter @haruhi/shop dev
pnpm --filter @haruhi/shop build
pnpm --filter @haruhi/shop preview
```

访问：

- 前台：`http://localhost:5205/shop/`
- 后台：`http://localhost:5205/shop/admin`

## 目录

```text
src/
  main.js
  router/index.js
  stores/shopStore.js       全局 reactive store
  layouts/
    ShopLayout.vue
    AdminLayout.vue
  views/
    shop/                   前台页面
    admin/                  后台页面
  utils/
    runtimePaths.js         子路径和 API 前缀
    adminAuth.js            shop 后台鉴权
    analytics.js            前台埋点
    chinaDivision.js        省市区数据
  assets/
    shop.css
    admin.css
```

## 功能范围

- 商品列表、详情、多图展示。
- 购物车、结算、优惠券预览、收货地址。
- 微信/支付宝二维码支付展示，用户手动确认已支付。
- 订单查询和物流信息。
- 预售：目标制和固定日期两种模式。
- 一单拆分现货/预售等子订单，按子订单发货。
- 后台商品、订单、优惠券、留言、站点配置管理。
- CSV 导出待发货数据，CSV 导入快递单号。
- Chart.js 统计销售趋势、排行和转化率。
- 下单/留言邮件由后端队列异步发送。

## 后端契约

- 所有业务接口使用 `/api/shop` 前缀。
- `runtimePaths.js` 负责 `resolveApiPath()` 和 `resolveAppPath()`。
- 后台登录使用 `createAdminAuth('shop')`，封装在 `utils/adminAuth.js`。
- 后台路由守卫会先检查本地 token，再调用 `/api/auth/me` 确认权限。
- 上传图片走 `/api/shop/upload`，静态访问走 `/uploads/shop/*`。

## 业务规则

- 金额在 store 内按“分”做整数运算，避免浮点误差。
- 运费按 `shippingTag` 分组，组内取最大运费再累加。
- 包邮门槛来自后端配置 `SHOP_FREE_SHIPPING_THRESHOLD`，默认 150 元。
- 订单状态流：下单 -> 待付款 -> 用户确认付款 -> 待确认 -> 后台流转。
- 优惠券支持金额减免和百分比折扣，前台结算时实时预览。

## 维护注意

- 本 app 不用 Pinia，状态在 `stores/shopStore.js` 的单例 `reactive()` store 中。
- 改子路径时优先改 Vite `base`，业务代码通过 `runtimePaths.js` 取路径。
- 商品图上传前浏览器侧可转 WebP；收款二维码等场景可禁用转换。
- 邮件模板、队列和重试在后端 shop 模块，前端只提交邮箱和业务动作。
