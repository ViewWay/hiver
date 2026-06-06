//! LDAP mapper traits — mapping LDAP entries to Rust objects
//! `LDAP映射器trait` — `将LDAP条目映射到Rust对象`
//!
//! Equivalent to Spring LDAP's `AttributesMapper` and `ContextMapper`
//! 等价于 Spring LDAP 的 `AttributesMapper` 和 `ContextMapper`

/// Maps LDAP attributes to a Rust type / `将LDAP属性映射到Rust类型`
pub trait AttributesMapper<T>: Send + Sync
{
    /// Map raw attribute pairs to a Rust type / 将原始属性对映射到Rust类型
    fn map_attributes(&self, attrs: &[(&str, &[&str])]) -> T;
    /// Map attributes with default delegation / 使用默认委托映射属性
    fn map_from_attributes(&self, attrs: &[(&str, &[&str])]) -> T
    {
        self.map_attributes(attrs)
    }
}

/// Maps an LDAP entry context to a Rust type / `将LDAP条目上下文映射到Rust类型`
pub trait ContextMapper<T>: Send + Sync
{
    /// Map from a raw LDAP context string / 从原始LDAP上下文字符串映射
    fn map_from_context(&self, ctx: &str) -> T;
}

/// Simple attribute map / 简单属性映射
#[derive(Debug, Clone)]
pub struct AttrMap
{
    attrs: Vec<(String, Vec<String>)>,
}

impl Default for AttrMap
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl AttrMap
{
    /// Create an empty attribute map / 创建空的属性映射
    pub fn new() -> Self
    {
        Self { attrs: Vec::new() }
    }

    /// Add a key-value pair to the map / 向映射中添加键值对
    pub fn add(&mut self, key: &str, values: &[&str])
    {
        self.attrs
            .push((key.to_string(), values.iter().map(ToString::to_string).collect()));
    }

    /// Get all values for a key / 获取某个键的所有值
    pub fn get(&self, key: &str) -> Option<&[String]>
    {
        self.attrs
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_slice())
    }

    /// Get the first value for a key / 获取某个键的第一个值
    pub fn get_first(&self, key: &str) -> Option<&str>
    {
        self.get(key).and_then(|v| v.first().map(String::as_str))
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests
{
    use super::*;

    #[test]
    fn test_attr_map()
    {
        let mut map = AttrMap::new();
        map.add("cn", &["John"]);
        assert_eq!(map.get_first("cn"), Some("John"));
        assert_eq!(map.get_first("sn"), None);
    }
}
