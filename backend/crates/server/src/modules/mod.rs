//! 各业务模块路由，按迁移阶段逐个接入到 /api/<module>。

use axum::Router;

use crate::state::AppState;

/// 把所有已上线的模块路由挂到 api 路由器上。
/// 迁移每个模块时在此 nest 一行，例如：
///   let api = api.nest("/novel", novel::router());
pub fn mount(api: Router<AppState>) -> Router<AppState> {
    api
}
