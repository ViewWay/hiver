//! Distributed tracing module
//! 分布式追踪模块
//!
//! # Overview / 概述
//!
//! This module provides distributed tracing functionality compatible with `OpenTelemetry`.
//! It includes span management, trace context propagation, and W3C Trace Context format.
//!
//! 本模块提供与 `OpenTelemetry` 兼容的分布式追踪功能。
//! 包括 span 管理、追踪上下文传播和 W3C Trace Context 格式。
//!
//! # Equivalent to Spring Boot / 等价于 Spring Boot
//!
//! - Spring Cloud Sleuth + `OpenTelemetry` Tracing
//! - Micrometer Tracing
//! - @`NewSpan` annotation
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_observability::trace::{Tracer, SpanBuilder};
//!
//! let tracer = Tracer::new("my-service");
//!
//! // Create a root span
//! let span = tracer.span("handle_request")
//!     .with_attribute("http.method", "GET")
//!     .with_attribute("http.path", "/api/users")
//!     .start();
//!
//! // Do some work...
//! span.end();
//! ```

#![warn(missing_docs)]
#![warn(unreachable_pub)]

use std::{
    fmt,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

/// Global trace ID generator
/// 全局追踪ID生成器
static TRACE_ID_GENERATOR: std::sync::LazyLock<Arc<IdGenerator>> =
    std::sync::LazyLock::new(|| Arc::new(IdGenerator::new()));

/// ID generator for trace and span IDs
/// ID生成器，用于追踪和span ID
struct IdGenerator
{
    counter: AtomicU64,
}

impl IdGenerator
{
    fn new() -> Self
    {
        Self {
            counter: AtomicU64::new(1),
        }
    }

    /// Generate next trace ID (128-bit)
    /// 生成下一个追踪ID（128位）
    #[allow(clippy::expect_used)]
    fn next_trace_id(&self) -> TraceId
    {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX_EPOCH")
            .as_nanos() as u64;
        let random = self.counter.fetch_add(1, Ordering::Relaxed);
        TraceId::from_parts(timestamp, random)
    }

    /// Generate next span ID (64-bit)
    /// 生成下一个span ID（64位）
    fn next_span_id(&self) -> SpanId
    {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);
        SpanId::from_u64(id)
    }
}

/// Trace ID (128-bit)
/// 追踪ID（128位）
///
/// Uniquely identifies a distributed trace.
/// 唯一标识分布式追踪。
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TraceId([u8; 16]);

impl TraceId
{
    /// Create a new trace ID
    /// 创建新的追踪ID
    pub fn new() -> Self
    {
        TRACE_ID_GENERATOR.next_trace_id()
    }

    /// Create from high and low u64 parts
    /// 从高位和低位u64部分创建
    pub fn from_parts(high: u64, low: u64) -> Self
    {
        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&high.to_be_bytes());
        bytes[8..16].copy_from_slice(&low.to_be_bytes());
        Self(bytes)
    }

    /// Create from u64 (zero-extended)
    /// 从u64创建（零扩展）
    pub fn from_u64(value: u64) -> Self
    {
        Self::from_parts(0, value)
    }

    /// Parse from hex string
    /// 从十六进制字符串解析
    pub fn from_hex(hex: &str) -> Option<Self>
    {
        let bytes = if hex.len() == 32
        {
            hex.as_bytes()
                .chunks(2)
                .map(|chunk| u8::from_str_radix(std::str::from_utf8(chunk).ok()?, 16).ok())
                .collect::<Option<Vec<_>>>()?
        }
        else
        {
            return None;
        };
        Some(Self(bytes.try_into().ok()?))
    }

    /// Get high 64 bits
    /// 获取高64位
    #[allow(clippy::expect_used)]
    pub fn high(&self) -> u64
    {
        u64::from_be_bytes(self.0[0..8].try_into().expect("slice has correct length"))
    }

    /// Get low 64 bits
    /// 获取低64位
    #[allow(clippy::expect_used)]
    pub fn low(&self) -> u64
    {
        u64::from_be_bytes(self.0[8..16].try_into().expect("slice has correct length"))
    }

    /// Convert to hex string
    /// 转换为十六进制字符串
    #[allow(clippy::format_collect)]
    pub fn to_hex(&self) -> String
    {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

impl Default for TraceId
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl fmt::Debug for TraceId
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "TraceId({})", self.to_hex())
    }
}

impl fmt::Display for TraceId
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", self.to_hex())
    }
}

/// Span ID (64-bit)
/// Span ID（64位）
///
/// Identifies a single span within a trace.
/// 标识追踪中的单个span。
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpanId([u8; 8]);

impl SpanId
{
    /// Create a new span ID
    /// 创建新的span ID
    pub fn new() -> Self
    {
        TRACE_ID_GENERATOR.next_span_id()
    }

    /// Create from u64
    /// 从u64创建
    pub fn from_u64(value: u64) -> Self
    {
        Self(value.to_be_bytes())
    }

    /// Parse from hex string
    /// 从十六进制字符串解析
    pub fn from_hex(hex: &str) -> Option<Self>
    {
        if hex.len() == 16
        {
            let bytes = hex
                .as_bytes()
                .chunks(2)
                .map(|chunk| u8::from_str_radix(std::str::from_utf8(chunk).ok()?, 16).ok())
                .collect::<Option<Vec<_>>>()?;
            return Some(Self(bytes.try_into().ok()?));
        }
        None
    }

    /// Convert to u64
    /// 转换为u64
    pub fn as_u64(&self) -> u64
    {
        u64::from_be_bytes(self.0)
    }

    /// Convert to hex string
    /// 转换为十六进制字符串
    #[allow(clippy::format_collect)]
    pub fn to_hex(&self) -> String
    {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

impl Default for SpanId
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl fmt::Debug for SpanId
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "SpanId({})", self.to_hex())
    }
}

impl fmt::Display for SpanId
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", self.to_hex())
    }
}

/// Span flags
/// Span标志
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpanFlags(u8);

impl SpanFlags
{
    /// No flags
    /// 无标志
    pub const NONE: SpanFlags = SpanFlags(0x00);
    /// Sampled flag
    /// 采样标志
    pub const SAMPLED: SpanFlags = SpanFlags(0x01);

    /// Create new flags
    /// 创建新标志
    pub fn new(value: u8) -> Self
    {
        Self(value)
    }

    /// Check if sampled
    /// 检查是否采样
    pub fn is_sampled(&self) -> bool
    {
        self.0 & 0x01 != 0
    }

    /// Set sampled flag
    /// 设置采样标志
    pub fn with_sampled(mut self) -> Self
    {
        self.0 |= 0x01;
        self
    }
}

impl Default for SpanFlags
{
    fn default() -> Self
    {
        Self::SAMPLED
    }
}

/// Trace context for distributed tracing
/// 分布式追踪的追踪上下文
///
/// Contains the trace ID, span ID, and flags needed to propagate
/// trace information across service boundaries.
///
/// 包含跨服务边界传播追踪信息所需的追踪ID、span ID和标志。
#[derive(Clone, PartialEq, Eq)]
pub struct TraceContext
{
    /// Trace ID
    /// 追踪ID
    pub trace_id: TraceId,

    /// Current span ID
    /// 当前span ID
    pub span_id: SpanId,

    /// Parent span ID (if any)
    /// 父span ID（如果有）
    pub parent_span_id: Option<SpanId>,

    /// Trace flags
    /// 追踪标志
    pub flags: SpanFlags,

    /// Trace state (for W3C tracestate)
    /// 追踪状态（用于W3C tracestate）
    pub trace_state: Vec<(String, String)>,
}

impl TraceContext
{
    /// Create a new trace context (root span)
    /// 创建新的追踪上下文（根span）
    pub fn new() -> Self
    {
        Self {
            trace_id: TraceId::new(),
            span_id: SpanId::new(),
            parent_span_id: None,
            flags: SpanFlags::SAMPLED,
            trace_state: Vec::new(),
        }
    }

    /// Create a child context from this one
    /// 从此上下文创建子上下文
    pub fn child(&self) -> Self
    {
        Self {
            trace_id: self.trace_id,
            span_id: SpanId::new(),
            parent_span_id: Some(self.span_id),
            flags: self.flags,
            trace_state: self.trace_state.clone(),
        }
    }

    /// Create from W3C traceparent header value
    /// 从W3C traceparent头值创建
    ///
    /// Format: `version-trace_id-span_id-flags`
    /// Example: `00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01`
    pub fn from_traceparent(value: &str) -> Result<Self, TraceContextError>
    {
        let parts: Vec<&str> = value.split('-').collect();
        if parts.len() != 4
        {
            return Err(TraceContextError::InvalidFormat);
        }

        // SAFETY: length verified to be 4 above
        #[allow(clippy::indexing_slicing)]
        let version = parts[0];
        if version != "00"
        {
            return Err(TraceContextError::UnsupportedVersion(version.to_string()));
        }

        // SAFETY: length verified to be 4 above
        #[allow(clippy::indexing_slicing)]
        let trace_id = TraceId::from_hex(parts[1])
            .ok_or(TraceContextError::InvalidTraceId(parts[1].to_string()))?;

        // SAFETY: length verified to be 4 above
        #[allow(clippy::indexing_slicing)]
        let span_id = SpanId::from_hex(parts[2])
            .ok_or(TraceContextError::InvalidSpanId(parts[2].to_string()))?;

        // SAFETY: length verified to be 4 above
        #[allow(clippy::indexing_slicing)]
        let flags = u8::from_str_radix(parts[3], 16)
            .map_err(|_| TraceContextError::InvalidFlags(parts[3].to_string()))?;

        Ok(Self {
            trace_id,
            span_id,
            parent_span_id: None,
            flags: SpanFlags(flags),
            trace_state: Vec::new(),
        })
    }

    /// Convert to W3C traceparent header value
    /// 转换为W3C traceparent头值
    pub fn to_traceparent(&self) -> String
    {
        format!("00-{}-{}-{:02x}", self.trace_id.to_hex(), self.span_id.to_hex(), self.flags.0)
    }

    /// Check if this trace is sampled
    /// 检查此追踪是否被采样
    pub fn is_sampled(&self) -> bool
    {
        self.flags.is_sampled()
    }

    /// Add a trace state entry
    /// 添加追踪状态条目
    pub fn with_trace_state(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.trace_state.push((key.into(), value.into()));
        self
    }
}

impl Default for TraceContext
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl fmt::Debug for TraceContext
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("TraceContext")
            .field("trace_id", &self.trace_id)
            .field("span_id", &self.span_id)
            .field("parent_span_id", &self.parent_span_id)
            .field("flags", &self.flags)
            .finish()
    }
}

/// Trace context error
/// 追踪上下文错误
#[derive(Debug, Clone)]
pub enum TraceContextError
{
    /// Invalid traceparent format
    /// 无效的traceparent格式
    InvalidFormat,

    /// Unsupported traceparent version
    /// 不支持的traceparent版本
    UnsupportedVersion(String),

    /// Invalid trace ID
    /// 无效的追踪ID
    InvalidTraceId(String),

    /// Invalid span ID
    /// 无效的span ID
    InvalidSpanId(String),

    /// Invalid flags
    /// 无效的标志
    InvalidFlags(String),
}

impl fmt::Display for TraceContextError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::InvalidFormat => write!(f, "Invalid traceparent format"),
            Self::UnsupportedVersion(v) => write!(f, "Unsupported traceparent version: {}", v),
            Self::InvalidTraceId(id) => write!(f, "Invalid trace ID: {}", id),
            Self::InvalidSpanId(id) => write!(f, "Invalid span ID: {}", id),
            Self::InvalidFlags(flg) => write!(f, "Invalid flags: {}", flg),
        }
    }
}

impl std::error::Error for TraceContextError {}

/// Span kind
/// Span类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanKind
{
    /// Internal operation
    /// 内部操作
    Internal,

    /// Server request handler
    /// 服务器请求处理程序
    Server,

    /// Client request sender
    /// 客户端请求发送者
    Client,

    /// Producer
    /// 生产者
    Producer,

    /// Consumer
    /// 消费者
    Consumer,
}

impl fmt::Display for SpanKind
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::Internal => write!(f, "internal"),
            Self::Server => write!(f, "server"),
            Self::Client => write!(f, "client"),
            Self::Producer => write!(f, "producer"),
            Self::Consumer => write!(f, "consumer"),
        }
    }
}

/// Span status
/// Span状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanStatus
{
    /// Operation completed successfully
    /// 操作成功完成
    Ok,

    /// Operation cancelled
    /// 操作已取消
    Cancelled,

    /// Operation encountered an error
    /// 操作遇到错误
    UnknownError,
}

/// Span representing a single operation
/// 表示单个操作的span
///
/// A span represents a single operation within a trace. Spans can be nested
/// to form a trace tree representing a distributed transaction.
///
/// span表示追踪中的单个操作。span可以嵌套形成表示分布式事务的追踪树。
#[derive(Clone)]
pub struct Span
{
    /// Span context
    /// Span上下文
    context: TraceContext,

    /// Span name
    /// Span名称
    name: String,

    /// Span kind
    /// Span类型
    kind: SpanKind,

    /// Start time (nanoseconds since UNIX epoch)
    /// 开始时间（自UNIX纪元以来的纳秒数）
    start_time: u64,

    /// End time (None if still active)
    /// 结束时间（如果仍处于活动状态则为None）
    end_time: Option<u64>,

    /// Span attributes
    /// Span属性
    attributes: Vec<(String, String)>,

    /// Span events
    /// Span事件
    events: Vec<SpanEvent>,

    /// Span status
    /// Span状态
    status: Option<SpanStatus>,

    /// Whether this span is recording
    /// 此span是否正在记录
    is_recording: bool,
}

/// Span event
/// Span事件
#[derive(Debug, Clone)]
pub struct SpanEvent
{
    /// Event name
    /// 事件名称
    pub name: String,

    /// Event timestamp (nanoseconds since UNIX epoch)
    /// 事件时间戳（自UNIX纪元以来的纳秒数）
    pub timestamp: u64,

    /// Event attributes
    /// 事件属性
    pub attributes: Vec<(String, String)>,
}

impl Span
{
    /// Create a new span
    /// 创建新的span
    pub fn new(name: impl Into<String>) -> Self
    {
        Self::with_context(name, TraceContext::new())
    }

    /// Create a new span with a context
    /// 使用上下文创建新span
    #[allow(clippy::expect_used)]
    pub fn with_context(name: impl Into<String>, context: TraceContext) -> Self
    {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX_EPOCH")
            .as_nanos() as u64;

        Self {
            context,
            name: name.into(),
            kind: SpanKind::Internal,
            start_time: now,
            end_time: None,
            attributes: Vec::new(),
            events: Vec::new(),
            status: None,
            is_recording: true,
        }
    }

    /// Get the span's context
    /// 获取span的上下文
    pub fn context(&self) -> &TraceContext
    {
        &self.context
    }

    /// Get the span's name
    /// 获取span的名称
    pub fn name(&self) -> &str
    {
        &self.name
    }

    /// Get start time in nanoseconds since UNIX epoch
    /// 获取开始时间（自 UNIX 纪元以来的纳秒数）
    pub fn start_time_ns(&self) -> u64
    {
        self.start_time
    }

    /// Get duration in nanoseconds (None if not ended)
    /// 获取持续时间（纳秒，未结束则为 None）
    pub fn duration_ns(&self) -> Option<u64>
    {
        self.end_time.map(|end| end.saturating_sub(self.start_time))
    }

    /// Get the span status
    /// 获取span状态
    pub fn status(&self) -> Option<SpanStatus>
    {
        self.status
    }

    /// Get the span's kind
    /// 获取span的类型
    pub fn kind(&self) -> SpanKind
    {
        self.kind
    }

    /// Set the span kind
    /// 设置span类型
    pub fn with_kind(mut self, kind: SpanKind) -> Self
    {
        self.kind = kind;
        self
    }

    /// Add an attribute to the span
    /// 向span添加属性
    pub fn add_attribute(&mut self, key: impl Into<String>, value: impl Into<String>)
    {
        if self.is_recording
        {
            self.attributes.push((key.into(), value.into()));
        }
    }

    /// Add an attribute to the span (builder style)
    /// 向span添加属性（构建器样式）
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.add_attribute(key, value);
        self
    }

    /// Get span attributes
    /// 获取span属性
    pub fn attributes(&self) -> &[(String, String)]
    {
        &self.attributes
    }

    /// Add an event to the span
    /// 向span添加事件
    #[allow(clippy::unwrap_used)]
    pub fn add_event(&mut self, name: impl Into<String>)
    {
        if self.is_recording
        {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;

            self.events.push(SpanEvent {
                name: name.into(),
                timestamp: now,
                attributes: Vec::new(),
            });
        }
    }

    /// Add an event with attributes
    /// 添加带属性的事件
    #[allow(clippy::unwrap_used)]
    pub fn add_event_with_attrs(&mut self, name: impl Into<String>, attrs: Vec<(String, String)>)
    {
        if self.is_recording
        {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;

            self.events.push(SpanEvent {
                name: name.into(),
                timestamp: now,
                attributes: attrs,
            });
        }
    }

    /// Get span events
    /// 获取span事件
    pub fn events(&self) -> &[SpanEvent]
    {
        &self.events
    }

    /// Get span duration in nanoseconds (None if not ended)
    /// 获取span持续时间（以纳秒为单位，如果未结束则为None）
    pub fn duration(&self) -> Option<u64>
    {
        self.end_time.map(|end| end.saturating_sub(self.start_time))
    }

    /// Check if the span is active (not ended)
    /// 检查span是否处于活动状态（未结束）
    pub fn is_active(&self) -> bool
    {
        self.end_time.is_none()
    }

    /// Check if the span is recording
    /// 检查span是否正在记录
    pub fn is_recording(&self) -> bool
    {
        self.is_recording
    }

    /// End the span
    /// 结束span
    #[allow(clippy::unwrap_used)]
    pub fn end(&mut self)
    {
        if self.is_active()
        {
            self.end_time = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as u64,
            );
        }
    }

    /// End the span with a status
    /// 使用状态结束span
    pub fn end_with_status(&mut self, status: SpanStatus)
    {
        self.status = Some(status);
        self.end();
    }

    /// Create a child span
    /// 创建子span
    pub fn child(&self, name: impl Into<String>) -> Self
    {
        let child_ctx = self.context.child();
        Self::with_context(name, child_ctx)
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl fmt::Debug for Span
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("Span")
            .field("name", &self.name)
            .field("context", &self.context)
            .field("kind", &self.kind)
            .field("start_time", &self.start_time)
            .field("end_time", &self.end_time)
            .field("duration", &self.duration())
            .field("attributes", &self.attributes.len())
            .field("events", &self.events.len())
            .finish()
    }
}

/// Builder for creating spans
/// 用于创建span的构建器
pub struct SpanBuilder
{
    name: String,
    context: Option<TraceContext>,
    kind: SpanKind,
    attributes: Vec<(String, String)>,
}

impl SpanBuilder
{
    /// Create a new span builder
    /// 创建新的span构建器
    pub fn new(name: impl Into<String>) -> Self
    {
        Self {
            name: name.into(),
            context: None,
            kind: SpanKind::Internal,
            attributes: Vec::new(),
        }
    }

    /// Set the trace context
    /// 设置追踪上下文
    pub fn with_context(mut self, context: TraceContext) -> Self
    {
        self.context = Some(context);
        self
    }

    /// Set the span kind
    /// 设置span类型
    pub fn with_kind(mut self, kind: SpanKind) -> Self
    {
        self.kind = kind;
        self
    }

    /// Add an attribute
    /// 添加属性
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self
    {
        self.attributes.push((key.into(), value.into()));
        self
    }

    /// Build the span
    /// 构建span
    pub fn build(self) -> Span
    {
        let context = self.context.unwrap_or_default();
        let mut span = Span::with_context(self.name, context).with_kind(self.kind);

        for (key, value) in self.attributes
        {
            span.add_attribute(key, value);
        }

        span
    }

    /// Build and start the span
    /// 构建并启动span
    pub fn start(self) -> Span
    {
        self.build()
    }
}

/// Tracer for creating and managing spans
/// 用于创建和管理span的追踪器
///
/// The tracer is the entry point for creating spans and managing
/// distributed tracing.
///
/// 追踪器是创建span和管理分布式追踪的入口点。
#[derive(Clone)]
pub struct Tracer
{
    /// Service name
    /// 服务名称
    service_name: String,

    /// Whether to sample traces
    /// 是否对追踪进行采样
    sample_rate: f64,
}

impl Tracer
{
    /// Create a new tracer
    /// 创建新的追踪器
    pub fn new(service_name: impl Into<String>) -> Self
    {
        Self {
            service_name: service_name.into(),
            sample_rate: 1.0,
        }
    }

    /// Set the sample rate (0.0 to 1.0)
    /// 设置采样率（0.0到1.0）
    pub fn with_sample_rate(mut self, rate: f64) -> Self
    {
        self.sample_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Get the service name
    /// 获取服务名称
    pub fn service_name(&self) -> &str
    {
        &self.service_name
    }

    /// Create a span builder
    /// 创建span构建器
    pub fn span(&self, name: impl Into<String>) -> SpanBuilder
    {
        SpanBuilder::new(name)
    }

    /// Create a root span
    /// 创建根span
    pub fn root_span(&self, name: impl Into<String>) -> Span
    {
        self.span(name)
            .with_kind(SpanKind::Server)
            .with_attribute("service.name", self.service_name.clone())
            .start()
    }

    /// Create a child span from a context
    /// 从上下文创建子span
    pub fn child_span(&self, name: impl Into<String>, parent: &TraceContext) -> Span
    {
        let child_ctx = parent.child();
        Span::with_context(name, child_ctx)
    }

    /// Extract trace context from headers
    /// 从头中提取追踪上下文
    pub fn extract(&self, headers: &[(String, String)]) -> Result<TraceContext, TraceContextError>
    {
        // Look for traceparent header
        for (key, value) in headers
        {
            if key.eq_ignore_ascii_case("traceparent")
            {
                return TraceContext::from_traceparent(value);
            }
        }
        Err(TraceContextError::InvalidFormat)
    }

    /// Inject trace context into headers
    /// 将追踪上下文注入头中
    pub fn inject(&self, context: &TraceContext, headers: &mut Vec<(String, String)>)
    {
        headers.push(("traceparent".to_string(), context.to_traceparent()));
    }
}

impl fmt::Debug for Tracer
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("Tracer")
            .field("service_name", &self.service_name)
            .field("sample_rate", &self.sample_rate)
            .finish()
    }
}

/// Global tracer instance
/// 全局追踪器实例
static GLOBAL_TRACER: std::sync::LazyLock<std::sync::RwLock<Option<Tracer>>> =
    std::sync::LazyLock::new(|| std::sync::RwLock::new(None));

/// Initialize the global tracer
/// 初始化全局追踪器
#[allow(clippy::expect_used)]
pub fn init_tracer(service_name: impl Into<String>) -> Tracer
{
    let tracer = Tracer::new(service_name);
    *GLOBAL_TRACER.write().expect("lock poisoned") = Some(tracer.clone());
    tracer
}

/// Get the global tracer
/// 获取全局追踪器
#[allow(clippy::expect_used)]
pub fn global_tracer() -> Option<Tracer>
{
    GLOBAL_TRACER
        .read()
        .expect("lock poisoned")
        .as_ref()
        .cloned()
}

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests
{
    use super::*;

    #[test]
    fn test_trace_id_default()
    {
        let id = TraceId::default();
        assert!(!id.to_hex().is_empty());
    }

    #[test]
    fn test_trace_id_from_parts()
    {
        let id = TraceId::from_parts(0x0123456789ABCDEF, 0xFEDCBA9876543210);
        assert_eq!(id.high(), 0x0123456789ABCDEF);
        assert_eq!(id.low(), 0xFEDCBA9876543210);
    }

    #[test]
    fn test_trace_id_hex()
    {
        let id = TraceId::from_hex("4bf92f3577b34da6a3ce929d0e0e4736").unwrap();
        assert_eq!(id.to_hex(), "4bf92f3577b34da6a3ce929d0e0e4736");
    }

    #[test]
    fn test_span_id_from_u64()
    {
        let id = SpanId::from_u64(0x00f067aa0ba902b7);
        assert_eq!(id.as_u64(), 0x00f067aa0ba902b7);
        assert_eq!(id.to_hex(), "00f067aa0ba902b7");
    }

    #[test]
    fn test_span_id_hex()
    {
        let id = SpanId::from_hex("00f067aa0ba902b7").unwrap();
        assert_eq!(id.to_hex(), "00f067aa0ba902b7");
    }

    #[test]
    fn test_span_flags()
    {
        assert!(!SpanFlags::NONE.is_sampled());
        assert!(SpanFlags::SAMPLED.is_sampled());
        assert!(SpanFlags::NONE.with_sampled().is_sampled());
    }

    #[test]
    fn test_trace_context_new()
    {
        let ctx = TraceContext::new();
        assert!(!ctx.trace_id.to_hex().is_empty());
        assert!(ctx.is_sampled());
        assert!(ctx.parent_span_id.is_none());
    }

    #[test]
    fn test_trace_context_child()
    {
        let parent = TraceContext::new();
        let child = parent.child();

        assert_eq!(child.trace_id, parent.trace_id);
        assert_ne!(child.span_id, parent.span_id);
        assert_eq!(child.parent_span_id, Some(parent.span_id));
    }

    #[test]
    fn test_trace_context_traceparent()
    {
        let ctx = TraceContext::from_traceparent(
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
        )
        .unwrap();
        assert_eq!(ctx.trace_id.to_hex(), "4bf92f3577b34da6a3ce929d0e0e4736");
        assert_eq!(ctx.span_id.to_hex(), "00f067aa0ba902b7");
        assert!(ctx.is_sampled());

        let traceparent = ctx.to_traceparent();
        assert_eq!(traceparent, "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01");
    }

    #[test]
    fn test_trace_context_invalid()
    {
        assert!(matches!(
            TraceContext::from_traceparent("invalid"),
            Err(TraceContextError::InvalidFormat)
        ));
    }

    #[test]
    fn test_span_new()
    {
        let span = Span::new("test_operation");
        assert_eq!(span.name(), "test_operation");
        assert!(span.is_active());
        assert!(span.is_recording());
    }

    #[test]
    fn test_span_with_context()
    {
        let ctx = TraceContext::new();
        let span = Span::with_context("test", ctx.clone());
        assert_eq!(span.context().trace_id, ctx.trace_id);
    }

    #[test]
    fn test_span_attributes()
    {
        let mut span = Span::new("test");
        span.add_attribute("key1", "value1");
        span.add_attribute("key2", "value2");

        assert_eq!(span.attributes().len(), 2);
        assert_eq!(span.attributes()[0], ("key1".to_string(), "value1".to_string()));
    }

    #[test]
    fn test_span_builder()
    {
        let span = SpanBuilder::new("test")
            .with_kind(SpanKind::Server)
            .with_attribute("key", "value")
            .build();

        assert_eq!(span.name(), "test");
        assert_eq!(span.kind(), SpanKind::Server);
        assert_eq!(span.attributes().len(), 1);
    }

    #[test]
    fn test_span_duration()
    {
        let mut span = Span::new("test");
        assert!(span.duration().is_none());

        std::thread::sleep(std::time::Duration::from_millis(10));
        span.end();
        assert!(span.duration().unwrap() > 0);
        assert!(!span.is_active());
    }

    #[test]
    fn test_span_child()
    {
        let parent = Span::new("parent");
        let child = parent.child("child");

        assert_eq!(child.context().trace_id, parent.context().trace_id);
        assert_eq!(child.context().parent_span_id, Some(parent.context().span_id));
    }

    #[test]
    fn test_span_events()
    {
        let mut span = Span::new("test");
        span.add_event("event1");
        span.add_event_with_attrs("event2", vec![("key".to_string(), "value".to_string())]);

        assert_eq!(span.events().len(), 2);
        assert_eq!(span.events()[0].name, "event1");
        assert_eq!(span.events()[1].name, "event2");
    }

    #[test]
    fn test_tracer_new()
    {
        let tracer = Tracer::new("my-service");
        assert_eq!(tracer.service_name(), "my-service");
    }

    #[test]
    fn test_tracer_root_span()
    {
        let tracer = Tracer::new("my-service");
        let span = tracer.root_span("handle_request");

        assert_eq!(span.name(), "handle_request");
        assert_eq!(span.kind(), SpanKind::Server);
        assert!(
            span.attributes()
                .iter()
                .any(|(k, v)| k == "service.name" && v == "my-service")
        );
    }

    #[test]
    fn test_tracer_extract_inject()
    {
        let tracer = Tracer::new("my-service");
        let ctx = TraceContext::new();

        let mut headers = Vec::new();
        tracer.inject(&ctx, &mut headers);

        let extracted = tracer.extract(&headers).unwrap();
        assert_eq!(extracted.trace_id, ctx.trace_id);
        assert_eq!(extracted.span_id, ctx.span_id);
    }

    #[test]
    fn test_span_kind_display()
    {
        assert_eq!(SpanKind::Internal.to_string(), "internal");
        assert_eq!(SpanKind::Server.to_string(), "server");
        assert_eq!(SpanKind::Client.to_string(), "client");
    }

    #[test]
    fn test_global_tracer()
    {
        init_tracer("test-service");
        let tracer = global_tracer().unwrap();
        assert_eq!(tracer.service_name(), "test-service");
    }
}
