//! shop 模块：春日商城（商品、订单、子订单、优惠券、留言、埋点、统计、站点设置、邮件队列）。
//! 忠实移植旧 haruhishop 后端（server/server.cjs ~3500 行 + email.cjs + db.cjs），统一挂载于 /api/shop。
//!
//! 与旧实现的差异（详见交付报告）：
//! - 鉴权：删除旧自研登录/JWT/限流（POST /admin/login、GET /admin/me、requireAdminAuth 中间件、
//!   登录限流 loginAttempts），统一走 /api/auth/login + RBAC（authorize/AuthUser，app="shop"）。
//!   分级：只读→Read；日常写（改商品/库存调整/改券状态/留言处理/订单改状态用 Moderate）→Write/Moderate；
//!   删除/批量生成/site-config 写→Manage。
//! - 事务：旧版部分跨表操作缺事务（如下单内多步 dbRun 串行无显式 BEGIN 在某些路径），
//!   这里下单/改状态/子订单发货/删除/批量生成券一律用显式 `pools.shop.begin()` 事务补强。
//! - 邮件：旧 email.cjs 内置 nodemailer/Resend/OAuth2 + worker；这里发送统一交给 haruhi_mail::Mailer，
//!   email_jobs 队列 + worker（spawn_email_worker）在本模块实现，模板文案忠实搬旧 buildOrderEmail。
//! - 上传：旧存 server/uploads 经 /shop-api/uploads/* 暴露；这里存 uploads/shop/，URL 统一 /uploads/shop/*。
//! - 松类型容错：旧 shop.db 数值列可能存成 TEXT，按 i64/f64 解码处用 CAST 或读 String 容错避免解码 500。
//! - 一次性数据迁移（migrateOrderTypeFlags/migrateSubOrders）未移植（属历史数据补全，非接口逻辑）。

use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::state::AppState;

mod admin_orders;
mod common;
mod coupons;
mod email;
mod misc;
mod orders;
mod pricing;
mod products;

// orders 子模块需要被 admin_orders/email 引用，已通过 super:: 路径访问。

pub use email::spawn_email_worker;

pub fn router() -> Router<AppState> {
    Router::new()
        // ---------- 站点设置 ----------
        .route("/site-config", get(misc::get_site_config))
        .route(
            "/admin/site-config",
            get(misc::admin_get_site_config).put(misc::admin_put_site_config),
        )
        // ---------- 优惠券 ----------
        .route("/coupons/preview", post(coupons::preview_coupon))
        .route("/admin/coupons", get(coupons::list_coupons))
        .route("/admin/coupons/batch", post(coupons::batch_coupons))
        .route(
            "/admin/coupons/{id}/status",
            put(coupons::set_coupon_status),
        )
        .route("/admin/coupons/{id}", delete(coupons::delete_coupon))
        // ---------- 联系留言 ----------
        .route("/contact/messages", post(misc::create_contact_message))
        .route("/admin/contact-messages", get(misc::list_contact_messages))
        .route(
            "/admin/contact-messages/{id}/status",
            put(misc::set_contact_message_status),
        )
        // ---------- 商品 ----------
        .route(
            "/products",
            get(products::list_products).post(products::create_product),
        )
        .route("/upload", post(products::upload))
        .route("/products/reorder", put(products::reorder_products))
        .route(
            "/products/{id}",
            put(products::update_product).delete(products::delete_product),
        )
        .route("/admin/products/{id}/adjust", put(products::adjust_product))
        // ---------- 订单（后台聚合/导出/发货；注意静态段需在 /orders/{id} 之前）----------
        .route("/orders/ids", get(admin_orders::list_order_ids))
        .route("/orders/mark-exported", put(admin_orders::mark_exported))
        .route(
            "/orders/mark-spot-exported",
            put(admin_orders::mark_spot_exported),
        )
        .route(
            "/orders/mark-presale-exported",
            put(admin_orders::mark_presale_exported),
        )
        .route(
            "/orders/spot-export-data",
            get(admin_orders::spot_export_data),
        )
        .route(
            "/orders/import-tracking",
            post(admin_orders::import_tracking),
        )
        .route(
            "/orders/{id}/sub-orders/{subKey}/ship",
            put(admin_orders::ship_sub_order),
        )
        .route("/admin/dashboard-summary", get(misc::dashboard_summary))
        // ---------- 订单（公开/通用）----------
        .route(
            "/orders",
            get(admin_orders::list_orders).post(orders::create_order),
        )
        .route(
            "/orders/{id}",
            get(orders::get_order).delete(admin_orders::delete_order),
        )
        .route("/orders/{id}/contact", put(orders::update_order_contact))
        .route("/orders/{id}/payment", post(orders::submit_payment))
        .route(
            "/orders/{id}/status",
            put(admin_orders::update_order_status),
        )
        // ---------- 埋点 / 统计 ----------
        .route("/analytics/event", post(misc::analytics_event))
        .route("/admin/stats/sales-trend", get(misc::sales_trend))
        .route("/admin/stats/product-sales", get(misc::product_sales))
        .route("/admin/stats/conversion", get(misc::conversion))
}
