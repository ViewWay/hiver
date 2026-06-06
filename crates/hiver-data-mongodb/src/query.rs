//! MongoDB query builders
//! MongoDB 查询构建器

use mongodb::bson::{Document, doc};

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

    /// Add regex filter / 添加正则过滤器
    pub fn regex(mut self, field: &str, pattern: &str, options: Option<&str>) -> Self {
        let mut regex_doc = mongodb::bson::doc! { "$regex": pattern };
        if let Some(opts) = options {
            regex_doc.insert("$options", opts);
        }
        self.filters.push(doc! { field: regex_doc });
        self
    }

    /// Add exists filter (field exists or not) / 添加存在性过滤器
    pub fn exists(mut self, field: &str, exists: bool) -> Self {
        self.filters.push(doc! { field: { "$exists": exists } });
        self
    }

    /// Add `$elemMatch` filter for array elements / 添加数组元素的 `$elemMatch` 过滤器
    pub fn elem_match(mut self, field: &str, condition: Document) -> Self {
        self.filters
            .push(doc! { field: { "$elemMatch": condition } });
        self
    }

    /// Add `$mod` (modulo) filter / 添加 `$mod`（模运算）过滤器
    pub fn mod_(mut self, field: &str, divisor: i64, remainder: i64) -> Self {
        self.filters
            .push(doc! { field: { "$mod": [divisor, remainder] } });
        self
    }

    /// Add type filter (by BSON type number) / 添加类型过滤器（按 BSON 类型编号）
    pub fn type_(mut self, field: &str, bson_type: i32) -> Self {
        self.filters.push(doc! { field: { "$type": bson_type } });
        self
    }

    /// Add `$all` filter (array contains all specified values) / 添加 `$all` 过滤器
    pub fn all(mut self, field: &str, values: Vec<mongodb::bson::Bson>) -> Self {
        self.filters.push(doc! { field: { "$all": values } });
        self
    }

    /// Add `$size` filter (array has specified length) / 添加 `$size` 过滤器
    pub fn size(mut self, field: &str, size: i32) -> Self {
        self.filters.push(doc! { field: { "$size": size } });
        self
    }

    /// Add not filter / 添加 not 过滤器
    pub fn not(mut self, field: &str, value: Document) -> Self {
        self.filters.push(doc! { field: { "$not": value } });
        self
    }

    /// Add nor filter / 添加 nor 过滤器
    pub fn nor(mut self, filters: Vec<Document>) -> Self {
        self.filters.push(doc! { "$nor": filters });
        self
    }

    /// Add where filter (JavaScript expression, use with caution) / 添加 where 过滤器
    pub fn where_(mut self, expression: &str) -> Self {
        self.filters.push(doc! { "$where": expression });
        self
    }

    /// Add text search filter / 添加文本搜索过滤器
    pub fn text_search(mut self, search: &str, language: Option<&str>) -> Self {
        let mut text_doc = mongodb::bson::doc! { "$search": search };
        if let Some(lang) = language {
            text_doc.insert("$language", lang);
        }
        self.filters.push(doc! { "$text": text_doc });
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

    #[test]
    fn test_filter_ne() {
        let filter = MongoFilter::new().ne("status", "deleted").build();
        let inner = filter.get_document("status").unwrap();
        assert_eq!(inner.get_str("$ne").unwrap(), "deleted");
    }

    #[test]
    fn test_filter_gte_lte() {
        let filter = MongoFilter::new().gte("age", 18).lte("age", 65).build();
        assert!(filter.contains_key("$and"));
    }

    #[test]
    fn test_filter_in() {
        let values = vec![
            mongodb::bson::Bson::String("a".to_string()),
            mongodb::bson::Bson::String("b".to_string()),
        ];
        let filter = MongoFilter::new().in_("status", values).build();
        assert!(filter.get_document("status").unwrap().contains_key("$in"));
    }

    #[test]
    fn test_filter_regex() {
        let filter = MongoFilter::new()
            .regex("email", r"@example\.com$", Some("i"))
            .build();
        let inner = filter.get_document("email").unwrap();
        assert_eq!(inner.get_str("$regex").unwrap(), r"@example\.com$");
        assert_eq!(inner.get_str("$options").unwrap(), "i");
    }

    #[test]
    fn test_filter_exists() {
        let filter = MongoFilter::new().exists("deletedAt", false).build();
        let inner = filter.get_document("deletedAt").unwrap();
        assert!(!inner.get_bool("$exists").unwrap());
    }

    #[test]
    fn test_filter_mod() {
        let filter = MongoFilter::new().mod_("qty", 5, 0).build();
        let inner = filter.get_document("qty").unwrap();
        assert!(inner.contains_key("$mod"));
    }

    #[test]
    fn test_filter_type() {
        let filter = MongoFilter::new().type_("name", 2).build();
        let inner = filter.get_document("name").unwrap();
        assert_eq!(inner.get_i32("$type").unwrap(), 2);
    }

    #[test]
    fn test_filter_all() {
        let values = vec![
            mongodb::bson::Bson::String("ssh".to_string()),
            mongodb::bson::Bson::String("ssl".to_string()),
        ];
        let filter = MongoFilter::new().all("tags", values).build();
        assert!(filter.get_document("tags").unwrap().contains_key("$all"));
    }

    #[test]
    fn test_filter_size() {
        let filter = MongoFilter::new().size("tags", 3).build();
        let inner = filter.get_document("tags").unwrap();
        assert_eq!(inner.get_i32("$size").unwrap(), 3);
    }

    #[test]
    fn test_filter_nor() {
        let conditions = vec![
            mongodb::bson::doc! { "status": "deleted" },
            mongodb::bson::doc! { "status": "archived" },
        ];
        let filter = MongoFilter::new().nor(conditions).build();
        assert!(filter.contains_key("$nor"));
    }

    #[test]
    fn test_filter_text_search() {
        let filter = MongoFilter::new().text_search("coffee", Some("en")).build();
        let inner = filter.get_document("$text").unwrap();
        assert_eq!(inner.get_str("$search").unwrap(), "coffee");
        assert_eq!(inner.get_str("$language").unwrap(), "en");
    }

    #[test]
    fn test_filter_and_or() {
        let filter = MongoFilter::new()
            .eq("active", true)
            .or(vec![
                mongodb::bson::doc! { "age": { "$lt": 30 } },
                mongodb::bson::doc! { "role": "admin" },
            ])
            .build();
        assert!(filter.contains_key("$and"));
    }

    #[test]
    fn test_filter_where() {
        let filter = MongoFilter::new()
            .where_("this.name == this.nickname")
            .build();
        assert_eq!(filter.get_str("$where").unwrap(), "this.name == this.nickname");
    }
}
