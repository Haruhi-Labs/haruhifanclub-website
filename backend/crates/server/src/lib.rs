//! haruhi-server 库目标：把路由/状态/seed/限流等暴露出来，供 main 二进制与
//! `tests/` 集成测试复用。二进制入口仍是 `src/main.rs`。
//
// 各模块 handler 直接用元组承接 sqlx 行（忠实移植旧后端，避免为每个查询造 DTO），
// type_complexity 是有意取舍而非疏忽，crate 级放行该 lint。
#![allow(clippy::type_complexity)]

pub mod admin_routes;
pub mod auth_routes;
pub mod modules;
pub mod notify;
pub mod ratelimit;
pub mod routes;
pub mod seed;
pub mod state;
