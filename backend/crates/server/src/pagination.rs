//! 分页公共助手：抽取 shop 多个列表端点（coupons/contact-messages/orders）重复的
//! 总页数计算与 `pagination` 响应对象，消除三处逐字节相同的样板。
//!
//! 注意：exam 列表有「非搜索首页预留 1 位」的特殊分页语义，仍保留各自实现，
//! 不强行套用本助手——共享只覆盖真正一致的形状。

use serde_json::{json, Value};

/// 总页数：向上取整；`total<=0`（或非法 page_size）按 1 页，与既有 shop 行为一致。
pub fn total_pages(total: i64, page_size: i64) -> i64 {
    if total > 0 && page_size > 0 {
        (total + page_size - 1) / page_size
    } else {
        1
    }
}

/// 标准分页元信息对象 `{ page, pageSize, total, totalPages }`（shop 列表统一形状）。
pub fn page_meta(page: i64, page_size: i64, total: i64) -> Value {
    json!({
        "page": page,
        "pageSize": page_size,
        "total": total,
        "totalPages": total_pages(total, page_size),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_pages_ceil_division_and_floor_one() {
        assert_eq!(total_pages(0, 10), 1, "空结果也算 1 页");
        assert_eq!(total_pages(1, 10), 1);
        assert_eq!(total_pages(10, 10), 1);
        assert_eq!(total_pages(11, 10), 2);
        assert_eq!(total_pages(25, 10), 3);
        assert_eq!(total_pages(100, 100), 1);
        // 非法 page_size 不 panic，回退 1 页
        assert_eq!(total_pages(5, 0), 1);
    }

    #[test]
    fn page_meta_shape_is_stable() {
        let m = page_meta(2, 10, 25);
        assert_eq!(m["page"], 2);
        assert_eq!(m["pageSize"], 10);
        assert_eq!(m["total"], 25);
        assert_eq!(m["totalPages"], 3);
    }
}
