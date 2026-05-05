//! `OpenAPI` schema definitions
//! `OpenAPI模式定义`

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schema type
/// 模式类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    /// String
    /// 字符串
    String,

    /// Number (float or double)
    /// 数字（浮点或双精度）
    Number,

    /// Integer
    /// 整数
    Integer,

    /// Boolean
    /// 布尔值
    Boolean,

    /// Array
    /// 数组
    Array,

    /// Object
    /// 对象
    Object,
}

/// Schema format
/// 模式格式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SchemaFormat {
    /// Float
    /// 浮点
    Float,

    /// Double
    /// 双精度
    Double,

    /// Int32
    /// 32位整数
    Int32,

    /// Int64
    /// 64位整数
    Int64,

    /// Date
    /// 日期
    Date,

    /// Date-time
    /// 日期时间
    DateTime,

    /// Password
    /// 密码
    Password,

    /// Byte (base64)
    /// 字节（base64）
    Byte,

    /// Binary
    /// 二进制
    Binary,

    /// UUID
    /// UUID
    Uuid,
}

/// `OpenAPI` Schema
/// `OpenAPI模式`
///
/// Equivalent to Spring's `@Schema` annotation.
/// 等价于Spring的`@Schema`注解。
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Schema(description = "User object", example = "{\"id\": 1}")
/// public class User {
///     @Schema(description = "User ID", required = true)
///     private Long id;
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct Schema {
    /// Schema type
    /// 模式类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<SchemaType>,

    /// Schema format
    /// 模式格式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<SchemaFormat>,

    /// Description
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Example value
    /// 示例值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,

    /// Properties (for objects)
    /// 属性（对象）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, SchemaProperty>>,

    /// Required properties
    /// 必需属性
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,

    /// Array items
    /// 数组项
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,

    /// Reference to another schema
    /// 引用其他模式
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub ref_: Option<String>,

    /// Enum values
    /// 枚举值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<serde_json::Value>>,

    /// Minimum value (for numbers)
    /// 最小值（数字）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,

    /// Maximum value (for numbers)
    /// 最大值（数字）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,

    /// Min length (for strings)
    /// 最小长度（字符串）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,

    /// Max length (for strings)
    /// 最大长度（字符串）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,

    /// Pattern (regex for strings)
    /// 模式（字符串的正则）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    /// Default value
    /// 默认值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    /// Nullable
    /// 可空
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,
}

impl Schema {
    /// Create a new schema
    /// 创建新模式
    pub fn new() -> Self {
        Self::default()
    }

    /// Set type
    /// 设置类型
    pub fn with_type(mut self, schema_type: SchemaType) -> Self {
        self.schema_type = Some(schema_type);
        self
    }

    /// Set format
    /// 设置格式
    pub fn with_format(mut self, format: SchemaFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Set description
    /// 设置描述
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set example
    /// 设置示例
    pub fn example(mut self, example: impl Into<serde_json::Value>) -> Self {
        self.example = Some(example.into());
        self
    }

    /// Add property
    /// 添加属性
    pub fn add_property(
        mut self,
        name: impl Into<String>,
        property: SchemaProperty,
    ) -> Self {
        self.properties
            .get_or_insert_with(HashMap::new)
            .insert(name.into(), property);
        self
    }

    /// Add required property
    /// 添加必需属性
    pub fn add_required(mut self, name: impl Into<String>) -> Self {
        self.required.push(name.into());
        self
    }

    /// Set array items
    /// 设置数组项
    pub fn items(mut self, items: Schema) -> Self {
        self.items = Some(Box::new(items));
        self
    }

    /// Set enum values
    /// 设置枚举值
    pub fn enum_values(mut self, values: Vec<serde_json::Value>) -> Self {
        self.enum_values = Some(values);
        self
    }

    /// String schema
    /// 字符串模式
    pub fn string() -> Self {
        Self {
            schema_type: Some(SchemaType::String),
            ..Default::default()
        }
    }

    /// Integer schema
    /// 整数模式
    pub fn integer() -> Self {
        Self {
            schema_type: Some(SchemaType::Integer),
            format: Some(SchemaFormat::Int32),
            ..Default::default()
        }
    }

    /// Long schema
    /// 长整数模式
    pub fn long() -> Self {
        Self {
            schema_type: Some(SchemaType::Integer),
            format: Some(SchemaFormat::Int64),
            ..Default::default()
        }
    }

    /// Float schema
    /// 浮点模式
    pub fn float() -> Self {
        Self {
            schema_type: Some(SchemaType::Number),
            format: Some(SchemaFormat::Float),
            ..Default::default()
        }
    }

    /// Double schema
    /// 双精度模式
    pub fn double() -> Self {
        Self {
            schema_type: Some(SchemaType::Number),
            format: Some(SchemaFormat::Double),
            ..Default::default()
        }
    }

    /// Boolean schema
    /// 布尔模式
    pub fn boolean() -> Self {
        Self {
            schema_type: Some(SchemaType::Boolean),
            ..Default::default()
        }
    }

    /// Array schema
    /// 数组模式
    pub fn array(items: Schema) -> Self {
        Self {
            schema_type: Some(SchemaType::Array),
            items: Some(Box::new(items)),
            ..Default::default()
        }
    }

    /// Object schema
    /// 对象模式
    pub fn object() -> Self {
        Self {
            schema_type: Some(SchemaType::Object),
            properties: Some(HashMap::new()),
            ..Default::default()
        }
    }

    /// Reference schema
    /// 引用模式
    pub fn reference(ref_: impl Into<String>) -> Self {
        Self {
            ref_: Some(ref_.into()),
            ..Default::default()
        }
    }
}


/// Schema property
/// 模式属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaProperty {
    /// Property schema
    /// 属性模式
    #[serde(flatten)]
    pub schema: Schema,
}

impl SchemaProperty {
    /// Create a new property
    /// 创建新属性
    pub fn new(schema: Schema) -> Self {
        Self { schema }
    }
}

impl From<Schema> for SchemaProperty {
    fn from(schema: Schema) -> Self {
        Self::new(schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_string() {
        let schema = Schema::string().description("User name");
        assert_eq!(schema.schema_type, Some(SchemaType::String));
        assert_eq!(schema.description, Some("User name".to_string()));
    }

    #[test]
    fn test_schema_integer() {
        let schema = Schema::integer();
        assert_eq!(schema.schema_type, Some(SchemaType::Integer));
        assert_eq!(schema.format, Some(SchemaFormat::Int32));
    }

    #[test]
    fn test_schema_array() {
        let schema = Schema::array(Schema::string());
        assert_eq!(schema.schema_type, Some(SchemaType::Array));
        assert!(schema.items.is_some());
    }

    #[test]
    fn test_schema_object() {
        let schema = Schema::object()
            .add_property("id", Schema::integer().description("User ID").into())
            .add_property("name", Schema::string().description("User name").into())
            .add_required("id");

        assert_eq!(schema.schema_type, Some(SchemaType::Object));
        assert!(schema.properties.is_some());
        assert_eq!(schema.required, vec!["id"]);
    }

    #[test]
    fn test_schema_reference() {
        let schema = Schema::reference("#/components/schemas/User");
        assert_eq!(schema.ref_, Some("#/components/schemas/User".to_string()));
    }
}
