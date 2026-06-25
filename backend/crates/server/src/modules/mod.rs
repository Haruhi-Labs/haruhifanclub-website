//! 各业务模块路由，按迁移阶段逐个接入到 /api/<module>。

use axum::Router;

use crate::state::AppState;

pub mod art;
pub mod art_guild;
pub mod exam;
pub mod news;
pub mod novel;
pub mod shop;

/// 把所有已上线的模块路由挂到 api 路由器上。
pub fn mount(api: Router<AppState>) -> Router<AppState> {
    api.nest("/novel", novel::router())
        .nest("/art", art::router())
        .nest("/news", news::router())
        .nest("/exam", exam::router())
        .nest("/shop", shop::router())
}
