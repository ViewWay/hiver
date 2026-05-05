//! Kafka serialization
//! Kafka序列化

/// Serializer trait
/// 序列化器trait
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface Serializer<T> {
///     byte[] serialize(String topic, T data);
/// }
///
/// @Bean
/// public Serializer<String> keySerializer() {
///     return new StringSerializer();
/// }
///
/// @Bean
/// public Serializer<String> valueSerializer() {
///     return new JsonSerializer();
/// }
/// ```
pub trait Serializer: Send + Sync {
    /// Serialize data to bytes
    /// 序列化数据为字节
    fn serialize(&self, data: &dyn SerializeData) -> Result<Vec<u8>, String>;
}

/// Data to serialize
/// 要序列化的数据
pub trait SerializeData {
    /// Get as bytes
    /// 获取字节表示
    fn as_bytes(&self) -> Option<&[u8]>;

    /// Get as string
    /// 获取字符串表示
    fn as_string(&self) -> Option<&str>;
}

impl SerializeData for str {
    fn as_bytes(&self) -> Option<&[u8]> {
        Some(self.as_bytes())
    }

    fn as_string(&self) -> Option<&str> {
        Some(self)
    }
}

impl SerializeData for String {
    fn as_bytes(&self) -> Option<&[u8]> {
        Some(self.as_bytes())
    }

    fn as_string(&self) -> Option<&str> {
        Some(self)
    }
}

impl SerializeData for [u8] {
    fn as_bytes(&self) -> Option<&[u8]> {
        Some(self)
    }

    fn as_string(&self) -> Option<&str> {
        None
    }
}

impl SerializeData for Vec<u8> {
    fn as_bytes(&self) -> Option<&[u8]> {
        Some(self)
    }

    fn as_string(&self) -> Option<&str> {
        None
    }
}

/// Bytes serializer
/// 字节序列化器
#[derive(Clone, Default)]
pub struct BytesSerializer;

impl Serializer for BytesSerializer {
    fn serialize(&self, data: &dyn SerializeData) -> Result<Vec<u8>, String> {
        data.as_bytes()
            .map(<[u8]>::to_vec)
            .ok_or_else(|| "Cannot serialize to bytes".to_string())
    }
}

/// JSON serializer
/// JSON序列化器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public JsonSerializer jsonSerializer() {
///     return new JsonSerializer();
/// }
/// ```
#[derive(Clone, Default)]
pub struct JsonSerializer;

impl Serializer for JsonSerializer {
    fn serialize(&self, data: &dyn SerializeData) -> Result<Vec<u8>, String> {
        // Try to get string representation and serialize as JSON
        // 尝试获取字符串表示并序列化为JSON
        if let Some(s) = data.as_string() {
            serde_json::to_vec(s)
                .map_err(|e| format!("Failed to serialize JSON: {}", e))
        } else if let Some(b) = data.as_bytes() {
            // Try to serialize bytes as JSON string
            // 尝试将字节序列化为JSON字符串
            let s = String::from_utf8(b.to_vec())
                .map_err(|e| format!("Invalid UTF-8: {}", e))?;
            serde_json::to_vec(&s)
                .map_err(|e| format!("Failed to serialize JSON: {}", e))
        } else {
            Err("Cannot serialize to JSON".to_string())
        }
    }
}

/// Deserializer trait
/// 反序列化器trait
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// public interface Deserializer<T> {
///     T deserialize(String topic, byte[] data);
/// }
///
/// @Bean
/// public Deserializer<String> valueDeserializer() {
///     return new JsonDeserializer<>(String.class);
/// }
/// ```
pub trait Deserializer: Send + Sync {
    /// Deserialize bytes to data
    /// 反序列化字节数据
    fn deserialize<'a, T: serde::Deserialize<'a>>(&self, bytes: &'a [u8]) -> Result<T, String>;
}

/// JSON deserializer
/// JSON反序列化器
#[derive(Clone, Default)]
pub struct JsonDeserializer;

impl Deserializer for JsonDeserializer {
    fn deserialize<'a, T: serde::Deserialize<'a>>(&self, bytes: &'a [u8]) -> Result<T, String> {
        serde_json::from_slice(bytes)
            .map_err(|e| format!("Failed to deserialize JSON: {}", e))
    }
}

/// Key serializer
/// 键序列化器
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Bean
/// public Serializer<String> keySerializer() {
///     return new StringSerializer();
/// }
/// ```
#[derive(Clone, Default)]
pub struct KeySerializer {
    /// Use string keys
    /// 使用字符串键
    pub use_string: bool,
}

impl KeySerializer {
    /// Create new key serializer
    /// 创建新的键序列化器
    pub fn new() -> Self {
        Self {
            use_string: true,
        }
    }

    /// Set to use string keys
    /// 设置使用字符串键
    pub fn with_string(mut self, use_string: bool) -> Self {
        self.use_string = use_string;
        self
    }
}

impl Serializer for KeySerializer {
    fn serialize(&self, data: &dyn SerializeData) -> Result<Vec<u8>, String> {
        if self.use_string {
            if let Some(s) = data.as_string() {
                Ok(s.as_bytes().to_vec())
            } else {
                Err("Key must be a string".to_string())
            }
        } else {
            BytesSerializer.serialize(data)
        }
    }
}
