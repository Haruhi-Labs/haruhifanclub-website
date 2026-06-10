//! 让 cargo 在 migrations 目录变动时重建本 crate。
//!
//! `sqlx::migrate!` 在编译期把迁移文件嵌入二进制，但新增迁移文件默认不会让 cargo
//! 失效旧的编译产物——增量构建会静默漏掉新迁移（如新增列查询报 "no such column"）。
//! 显式声明对整个 migrations 目录的依赖，新增/修改迁移即触发重编译。
fn main() {
    println!("cargo:rerun-if-changed=../../migrations");
}
