//! MongoDB query builders
//! MongoDB 查询构建器

use mongodb::bson::{doc, Document};

/// MongoDB filter builder / MongoDB 过滤器构建器
#[derive(Debug, Clone, Default)]
pub struct MongoFilter {
    filters: Vec<Document>,
}

impl MongoFilter {
    /// Create a new filter builder / 创建新的过滤器构建器
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    /// Add equality filter / 添加相等过滤器
    pub fn eq(mut self, field: &str, value: impl Into<mongodb::bson::Bson>) -> Self {
        self.filters.push(doc! { field: value.into() });
        self
    }

    /// Add not equal filter / 添加不等过滤器
    pub fn ne(mut self, field: &str, value: impl Into<mongodb::bson::Bson>) -> Self {
        self.filters.push(doc! { field: { "$ne": value.into() } });
        self
    }

    /// Add greater than filter / 添加大于过滤器
    pub fn gt(mut self, field: &str, value: impl Into<mongodb::bson::Bson>) -> Self {
        self.filters.push(doc! { field: { "$gt": value.into() } });
        self
    }

    /// Add greater than or equal filter / 添加大于等于过滤器
    pub fn gte(mut self, field: &str, value: impl Into<mongodb::bson::Bson>) -> Self {
        self.filters.push(doc! { field: { "$gte": value.into() } });
        self
    }

    /// Add less than filter / 添加小于过滤器
    pub fn lt(mut self, field: &str, value: impl Into<mongodb::bson::Bson>) -> Self {
        self.filters.push(doc! { field: { "$lt": value.into() } });
        self
    }

    /// Add less than or equal filter / 添加小于等于过滤器
    pub fn lte(mut self, field: &str, value: impl Into<mongodb::bson::Bson>) -> Self {
        self.filters.push(doc! { field: { "$lte": value.into() } });
        self
    }

    /// Add in filter / 添加 in 过滤器
    pub fn in_(mut self, field: &str, values: Vec<mongodb::bson::Bson>) -> Self {
        self.filters.push(doc! { field: { "$in": values } });
        self
    }

    /// Add and filter / 添加 and 过滤器
    pub fn and(mut self, filters: Vec<Document>) -> Self {
        self.filters.push(doc! { "$and": filters });
        self
    }

    /// Add or filter / 添加 or 过滤器
    pub fn or(mut self, filters: Vec<Document>) -> Self {
        self.filters.push(doc! { "$or": filters });
        self
    }

    /// Build the filter document / 构建过滤器文档
    pub fn build(&self) -> Document {
        if self.filters.is_empty() {
            doc! {}
        } else if self.filters.len() == 1 {
            self.filters[0].clone()
        } else {
            doc! { "$and": self.filters.clone() }
        }
    }
}

/// MongoDB query options / MongoDB 查询选项
#[derive(Debug, Clone, Default)]
pub struct MongoQueryOptions {
    /// Limit number of results / 限制结果数量
    pub limit: Option<u64>,
    /// Skip number of results / 跳过结果数量
    pub skip: Option<u64>,
    /// Sort specification / 排序规范
    pub sort: Option<Document>,
}

impl MongoQueryOptions {
    /// Create new query options / 创建新的查询选项
    pub fn new() -> Self {
        Self::default()
    }

    /// Set limit / 设置限制
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set skip / 设置跳过
    pub fn skip(mut self, skip: u64) -> Self {
        self.skip = Some(skip);
        self
    }

    /// Set sort / 设置排序
    pub fn sort(mut self, sort: Document) -> Self {
        self.sort = Some(sort);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_eq() {
        let filter = MongoFilter::new().eq("name", "test").build();
        assert_eq!(filter.get_str("name").unwrap(), "test");
    }

    #[test]
    fn test_filter_gt() {
        let filter = MongoFilter::new().gt("age", 18).build();
        assert!(filter.contains_key("age"));
    }

    #[test]
    fn test_query_options_limit() {
        let opts = MongoQueryOptions::new().limit(10);
        assert_eq!(opts.limit, Some(10));
    }
}
