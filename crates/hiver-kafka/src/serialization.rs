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
pub trait Serializer: Send + Sync
{
    /// Serialize data to bytes
    /// 序列化数据为字节
    fn serialize(&self, data: &dyn SerializeData) -> Result<Vec<u8>, String>;
}

/// Data to serialize
/// 要序列化的数据
pub trait SerializeData
{
    /// Get as bytes
    /// 获取字节表示
    fn as_bytes(&self) -> Option<&[u8]>;

    /// Get as string
    /// 获取字符串表示
    fn as_string(&self) -> Option<&str>;
}

impl SerializeData for str
{
    fn as_bytes(&self) -> Option<&[u8]>
    {
        Some(self.as_bytes())
    }

    fn as_string(&self) -> Option<&str>
    {
        Some(self)
    }
}

impl SerializeData for String
{
    fn as_bytes(&self) -> Option<&[u8]>
    {
        Some(self.as_bytes())
    }

    fn as_string(&self) -> Option<&str>
    {
        Some(self)
    }
}

impl SerializeData for [u8]
{
    fn as_bytes(&self) -> Option<&[u8]>
    {
        Some(self)
    }

    fn as_string(&self) -> Option<&str>
    {
        None
    }
}

impl SerializeData for Vec<u8>
{
    fn as_bytes(&self) -> Option<&[u8]>
    {
        Some(self)
    }

    fn as_string(&self) -> Option<&str>
    {
        None
    }
}

/// Bytes serializer
/// 字节序列化器
#[derive(Clone, Default)]
pub struct BytesSerializer;

impl Serializer for BytesSerializer
{
    fn serialize(&self, data: &dyn SerializeData) -> Result<Vec<u8>, String>
    {
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

impl Serializer for JsonSerializer
{
    fn serialize(&self, data: &dyn SerializeData) -> Result<Vec<u8>, String>
    {
        // Try to get string representation and serialize as JSON
        // 尝试获取字符串表示并序列化为JSON
        if let Some(s) = data.as_string()
        {
            serde_json::to_vec(s).map_err(|e| format!("Failed to serialize JSON: {}", e))
        }
        else if let Some(b) = data.as_bytes()
        {
            // Try to serialize bytes as JSON string
            // 尝试将字节序列化为JSON字符串
            let s = String::from_utf8(b.to_vec()).map_err(|e| format!("Invalid UTF-8: {}", e))?;
            serde_json::to_vec(&s).map_err(|e| format!("Failed to serialize JSON: {}", e))
        }
        else
        {
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
pub trait Deserializer: Send + Sync
{
    /// Deserialize bytes to data
    /// 反序列化字节数据
    fn deserialize<'a, T: serde::Deserialize<'a>>(&self, bytes: &'a [u8]) -> Result<T, String>;
}

/// JSON deserializer
/// JSON反序列化器
#[derive(Clone, Default)]
pub struct JsonDeserializer;

impl Deserializer for JsonDeserializer
{
    fn deserialize<'a, T: serde::Deserialize<'a>>(&self, bytes: &'a [u8]) -> Result<T, String>
    {
        serde_json::from_slice(bytes).map_err(|e| format!("Failed to deserialize JSON: {}", e))
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
#[derive(Clone)]
pub struct KeySerializer
{
    /// Use string keys
    /// 使用字符串键
    pub use_string: bool,
}

impl Default for KeySerializer
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl KeySerializer
{
    /// Create new key serializer
    /// 创建新的键序列化器
    pub fn new() -> Self
    {
        Self { use_string: true }
    }

    /// Set to use string keys
    /// 设置使用字符串键
    pub fn with_string(mut self, use_string: bool) -> Self
    {
        self.use_string = use_string;
        self
    }
}

impl Serializer for KeySerializer
{
    fn serialize(&self, data: &dyn SerializeData) -> Result<Vec<u8>, String>
    {
        if self.use_string
        {
            if let Some(s) = data.as_string()
            {
                Ok(s.as_bytes().to_vec())
            }
            else
            {
                Err("Key must be a string".to_string())
            }
        }
        else
        {
            BytesSerializer.serialize(data)
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    // ── SerializeData tests ───────────────────────────────────────────

    /// Test SerializeData for &str
    /// 测试 &str 的 SerializeData
    #[test]
    fn test_serialize_data_str()
    {
        let data: &str = "hello";
        assert_eq!(SerializeData::as_bytes(data), Some(&b"hello"[..]));
        assert_eq!(data.as_string(), Some("hello"));
    }

    /// Test SerializeData for String
    /// 测试 String 的 SerializeData
    #[test]
    fn test_serialize_data_string()
    {
        let data = String::from("world");
        assert_eq!(SerializeData::as_bytes(&data), Some(&b"world"[..]));
        assert_eq!(data.as_string(), Some("world"));
    }

    /// Test SerializeData for &[u8]
    /// 测试 &[u8] 的 SerializeData
    #[test]
    fn test_serialize_data_bytes_slice()
    {
        let data: &[u8] = &[1, 2, 3];
        assert_eq!(data.as_bytes(), Some(&[1u8, 2, 3][..]));
        assert!(data.as_string().is_none());
    }

    /// Test SerializeData for Vec<u8>
    /// 测试 Vec<u8> 的 SerializeData
    #[test]
    fn test_serialize_data_bytes_vec()
    {
        let data = vec![10, 20, 30];
        assert_eq!(data.as_bytes(), Some(&[10u8, 20, 30][..]));
        assert!(data.as_string().is_none());
    }

    // ── BytesSerializer tests ─────────────────────────────────────────

    /// Test BytesSerializer with string data
    /// 测试 BytesSerializer 序列化字符串数据
    #[test]
    fn test_bytes_serializer_string()
    {
        let serializer = BytesSerializer;
        let result = serializer.serialize(&"hello".to_string()).unwrap();
        assert_eq!(result, b"hello".to_vec());
    }

    /// Test BytesSerializer with bytes data
    /// 测试 BytesSerializer 序列化字节数据
    #[test]
    fn test_bytes_serializer_bytes()
    {
        let serializer = BytesSerializer;
        let data = vec![1u8, 2, 3];
        let result = serializer.serialize(&data).unwrap();
        assert_eq!(result, vec![1u8, 2, 3]);
    }

    // ── JsonSerializer tests ──────────────────────────────────────────

    /// Test JsonSerializer with string data
    /// 测试 JsonSerializer 序列化字符串数据
    #[test]
    fn test_json_serializer_string()
    {
        let serializer = JsonSerializer;
        let result = serializer.serialize(&"test-value".to_string()).unwrap();
        // JSON-serialized string includes quotes
        assert_eq!(result, b"\"test-value\"".to_vec());
    }

    /// Test JsonSerializer with valid UTF-8 bytes
    /// 测试 JsonSerializer 序列化有效UTF-8字节
    #[test]
    fn test_json_serializer_utf8_bytes()
    {
        let serializer = JsonSerializer;
        let data: Vec<u8> = b"hello".to_vec();
        let result = serializer.serialize(&data).unwrap();
        assert_eq!(result, b"\"hello\"".to_vec());
    }

    /// Test JsonSerializer with invalid UTF-8 bytes fails
    /// 测试 JsonSerializer 序列化无效UTF-8字节失败
    #[test]
    fn test_json_serializer_invalid_utf8_fails()
    {
        let serializer = JsonSerializer;
        let data: Vec<u8> = vec![0xFF, 0xFE];
        let result = serializer.serialize(&data);
        assert!(result.is_err());
    }

    // ── JsonDeserializer tests ────────────────────────────────────────

    /// Test JsonDeserializer with string
    /// 测试 JsonDeserializer 反序列化字符串
    #[test]
    fn test_json_deserializer_string()
    {
        let deserializer = JsonDeserializer;
        let bytes = b"\"hello\"";
        let result: String = deserializer.deserialize(bytes).unwrap();
        assert_eq!(result, "hello");
    }

    /// Test JsonDeserializer with struct
    /// 测试 JsonDeserializer 反序列化结构体
    #[test]
    fn test_json_deserializer_struct()
    {
        let deserializer = JsonDeserializer;
        #[derive(serde::Deserialize)]
        struct TestData
        {
            name: String,
            value: i32,
        }
        let bytes = br#"{"name":"test","value":42}"#;
        let result: TestData = deserializer.deserialize(bytes).unwrap();
        assert_eq!(result.name, "test");
        assert_eq!(result.value, 42);
    }

    /// Test JsonDeserializer with invalid JSON fails
    /// 测试 JsonDeserializer 反序列化无效JSON失败
    #[test]
    fn test_json_deserializer_invalid_json()
    {
        let deserializer = JsonDeserializer;
        let result: Result<String, String> = deserializer.deserialize(b"not json");
        assert!(result.is_err());
    }

    // ── KeySerializer tests ───────────────────────────────────────────

    /// Test KeySerializer in string mode with string data
    /// 测试字符串模式下 KeySerializer 序列化字符串
    #[test]
    fn test_key_serializer_string_mode()
    {
        let serializer = KeySerializer::new();
        assert!(serializer.use_string);
        let result = serializer.serialize(&"my-key".to_string()).unwrap();
        assert_eq!(result, b"my-key".to_vec());
    }

    /// Test KeySerializer in string mode rejects bytes
    /// 测试字符串模式下 KeySerializer 拒绝字节
    #[test]
    fn test_key_serializer_string_mode_rejects_bytes()
    {
        let serializer = KeySerializer::new();
        let data = vec![1u8, 2, 3];
        let result = serializer.serialize(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Key must be a string"));
    }

    /// Test KeySerializer in bytes mode
    /// 测试字节模式下 KeySerializer
    #[test]
    fn test_key_serializer_bytes_mode()
    {
        let serializer = KeySerializer::new().with_string(false);
        assert!(!serializer.use_string);
        let data = vec![10u8, 20];
        let result = serializer.serialize(&data).unwrap();
        assert_eq!(result, vec![10u8, 20]);
    }

    /// Test KeySerializer default
    /// 测试 KeySerializer 默认值
    #[test]
    fn test_key_serializer_default()
    {
        let serializer = KeySerializer::default();
        assert!(serializer.use_string);
    }
}
