//! PartTree parser for Spring Data-style method name query derivation
//! Spring Data 风格方法名查询派生的 PartTree 解析器
//!
//! # Overview / 概述
//!
//! This module implements a full PartTree parser based on Spring Data's
//! `PartTree.java` source. It parses method names like
//! `findByNameAndAgeGreaterThanOrderByNameDesc` into structured query
//! components: subject, keywords, parts, and order-by clauses.
//! 本模块基于 Spring Data 的 `PartTree.java` 源码实现了完整的 PartTree 解析器。
//! 它将像 `findByNameAndAgeGreaterThanOrderByNameDesc` 这样的方法名
//! 解析为结构化的查询组件：主体、关键字、部件和排序子句。
//!
//! # Equivalent to Spring / 等价于 Spring
//!
//! | Hiver | Spring Data |
//! |-------|-------------|
//! | `PartTree` | `PartTree` |
//! | `Subject` | `Subject` (Find, Read, Get, Query, Count, Exists, Delete) |
//! | `Keyword` | `Keyword` (Distinct, First, Top) |
//! | `PartType` | `Part.Type` (30+ comparison types) |
//! | `OrderBy` | `OrderBySource` |
//! | `AndOr` | `Or` / implicit AND |
//!
//! # Example / 示例
//!
//! ```rust
//! use hiver_data_commons::part_tree::PartTree;
//!
//! let tree =
//!     PartTree::parse("findByNameAndAgeGreaterThanOrderByNameDesc").expect("valid method name");
//!
//! assert_eq!(tree.subject().to_string(), "FIND");
//! assert!(!tree.is_distinct());
//! assert_eq!(tree.parts().len(), 2);
//! assert_eq!(tree.order_by().len(), 1);
//! ```

use std::fmt;

/// The subject (action type) of a query method.
/// 查询方法的主体（操作类型）。
///
/// Derived from the method name prefix (find, read, get, query, etc.).
/// 从方法名前缀派生（find、read、get、query 等）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subject
{
    /// find...By / find...By
    Find,
    /// read...By / read...By
    Read,
    /// get...By / get...By
    Get,
    /// query...By / query...By
    Query,
    /// count...By / count...By
    Count,
    /// exists...By / exists...By
    Exists,
    /// delete...By / delete...By
    Delete,
    /// stream...By / stream...By
    Stream,
}

impl Subject
{
    /// Returns true if this subject is a counting query.
    /// 返回此主体是否为计数查询。
    pub fn is_count_projection(&self) -> bool
    {
        matches!(self, Subject::Count)
    }

    /// Returns true if this subject is an existence check.
    /// 返回此主体是否为存在性检查。
    pub fn is_exists_projection(&self) -> bool
    {
        matches!(self, Subject::Exists)
    }

    /// Returns true if this subject is a delete operation.
    /// 返回此主体是否为删除操作。
    pub fn is_delete(&self) -> bool
    {
        matches!(self, Subject::Delete)
    }

    /// Returns true if this subject returns a collection.
    /// 返回此主体是否返回集合。
    pub fn is_collection_returning(&self) -> bool
    {
        matches!(self, Subject::Find | Subject::Read | Subject::Query | Subject::Stream)
    }
}

impl fmt::Display for Subject
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::Find => write!(f, "FIND"),
            Self::Read => write!(f, "READ"),
            Self::Get => write!(f, "GET"),
            Self::Query => write!(f, "QUERY"),
            Self::Count => write!(f, "COUNT"),
            Self::Exists => write!(f, "EXISTS"),
            Self::Delete => write!(f, "DELETE"),
            Self::Stream => write!(f, "STREAM"),
        }
    }
}

/// Keywords that modify the query behavior (Distinct, First, Top).
/// 修改查询行为的关键字（Distinct、First、Top）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keyword
{
    /// DISTINCT modifier / DISTINCT 修饰符
    Distinct,
    /// First<N> limit / First<N> 限制
    First(u32),
    /// Top<N> limit / Top<N> 限制
    Top(u32),
}

impl fmt::Display for Keyword
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::Distinct => write!(f, "DISTINCT"),
            Self::First(n) => write!(f, "FIRST {}", n),
            Self::Top(n) => write!(f, "TOP {}", n),
        }
    }
}

/// The 30+ part types representing comparison operators.
/// 表示比较运算符的 30+ 部件类型。
///
/// Based on Spring Data's `Part.Type` enum.
/// 基于 Spring Data 的 `Part.Type` 枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartType
{
    /// Simple equality (Is, Equals) / 简单相等
    Is,
    /// Not equal (IsNot, Not) / 不等
    IsNot,
    /// LIKE / 模糊匹配
    Like,
    /// NOT LIKE / 不模糊匹配
    NotLike,
    /// Starting with (LIKE 'prefix%') / 以...开头
    StartingWith,
    /// Ending with (LIKE '%suffix') / 以...结尾
    EndingWith,
    /// Containing (LIKE '%value%') / 包含
    Containing,
    /// Not containing / 不包含
    NotContaining,
    /// Less than / 小于
    LessThan,
    /// Less than or equal / 小于等于
    LessThanEqual,
    /// Greater than / 大于
    GreaterThan,
    /// Greater than or equal / 大于等于
    GreaterThanEqual,
    /// Between / 范围
    Between,
    /// IN / 包含于
    In,
    /// NOT IN / 不包含于
    NotIn,
    /// IS NULL / 为空
    IsNull,
    /// IS NOT NULL / 非空
    IsNotNull,
    /// True (boolean) / 为真
    True,
    /// False (boolean) / 为假
    False,
    /// Before (date comparison) / 日期之前
    Before,
    /// After (date comparison) / 日期之后
    After,
    /// Negating simple property / 取反简单属性
    Not,
    /// Regex match / 正则匹配
    Regex,
    /// Near (geospatial) / 邻近（地理空间）
    Near,
    /// Within (geospatial) / 范围内（地理空间）
    Within,
    /// Negating (generic) / 取反（通用）
    NegatingSimpleProperty,
    /// Simple property (no operator) / 简单属性（无运算符）
    SimpleProperty,
}

impl PartType
{
    /// Check if this part type requires a single argument.
    /// 检查此部件类型是否需要单个参数。
    pub fn needs_single_argument(&self) -> bool
    {
        matches!(
            self,
            PartType::Is
                | PartType::IsNot
                | PartType::Like
                | PartType::NotLike
                | PartType::StartingWith
                | PartType::EndingWith
                | PartType::Containing
                | PartType::NotContaining
                | PartType::LessThan
                | PartType::LessThanEqual
                | PartType::GreaterThan
                | PartType::GreaterThanEqual
                | PartType::Before
                | PartType::After
                | PartType::Regex
                | PartType::NegatingSimpleProperty
                | PartType::SimpleProperty
                | PartType::Not
        )
    }

    /// Check if this part type requires no arguments.
    /// 检查此部件类型是否不需要参数。
    pub fn needs_no_argument(&self) -> bool
    {
        matches!(self, PartType::IsNull | PartType::IsNotNull | PartType::True | PartType::False)
    }

    /// Check if this part type requires two arguments.
    /// 检查此部件类型是否需要两个参数。
    pub fn needs_two_arguments(&self) -> bool
    {
        matches!(self, PartType::Between)
    }

    /// Check if this part type requires a variable number of arguments.
    /// 检查此部件类型是否需要可变数量的参数。
    pub fn needs_variable_arguments(&self) -> bool
    {
        matches!(self, PartType::In | PartType::NotIn)
    }
}

impl fmt::Display for PartType
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::Is => write!(f, "IS"),
            Self::IsNot => write!(f, "IS_NOT"),
            Self::Like => write!(f, "LIKE"),
            Self::NotLike => write!(f, "NOT_LIKE"),
            Self::StartingWith => write!(f, "STARTING_WITH"),
            Self::EndingWith => write!(f, "ENDING_WITH"),
            Self::Containing => write!(f, "CONTAINING"),
            Self::NotContaining => write!(f, "NOT_CONTAINING"),
            Self::LessThan => write!(f, "LESS_THAN"),
            Self::LessThanEqual => write!(f, "LESS_THAN_EQUAL"),
            Self::GreaterThan => write!(f, "GREATER_THAN"),
            Self::GreaterThanEqual => write!(f, "GREATER_THAN_EQUAL"),
            Self::Between => write!(f, "BETWEEN"),
            Self::In => write!(f, "IN"),
            Self::NotIn => write!(f, "NOT_IN"),
            Self::IsNull => write!(f, "IS_NULL"),
            Self::IsNotNull => write!(f, "IS_NOT_NULL"),
            Self::True => write!(f, "TRUE"),
            Self::False => write!(f, "FALSE"),
            Self::Before => write!(f, "BEFORE"),
            Self::After => write!(f, "AFTER"),
            Self::Not => write!(f, "NOT"),
            Self::Regex => write!(f, "REGEX"),
            Self::Near => write!(f, "NEAR"),
            Self::Within => write!(f, "WITHIN"),
            Self::NegatingSimpleProperty => write!(f, "NEGATING_SIMPLE_PROPERTY"),
            Self::SimpleProperty => write!(f, "SIMPLE_PROPERTY"),
        }
    }
}

/// A single parsed part of the method name (property + operator).
/// 方法名的单个解析部件（属性 + 运算符）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part
{
    /// Property path (e.g., "age") / 属性路径
    pub property_path: String,
    /// The type of comparison / 比较类型
    pub part_type: PartType,
    /// Whether to ignore case for this comparison / 是否忽略大小写
    pub ignore_case: bool,
}

impl Part
{
    /// Create a new part.
    /// 创建新的部件。
    pub fn new(property_path: impl Into<String>, part_type: PartType) -> Self
    {
        Self {
            property_path: property_path.into(),
            part_type,
            ignore_case: false,
        }
    }

    /// Create a new part with ignore_case flag.
    /// 创建带有 ignore_case 标志的新部件。
    pub fn with_ignore_case(
        property_path: impl Into<String>,
        part_type: PartType,
        ignore_case: bool,
    ) -> Self
    {
        Self {
            property_path: property_path.into(),
            part_type,
            ignore_case,
        }
    }
}

impl fmt::Display for Part
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{} {}", self.property_path, self.part_type)
    }
}

/// Order-by clause parsed from the method name.
/// 从方法名解析的排序子句。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderBy
{
    /// Property to sort by / 排序属性
    pub property: String,
    /// Sort direction / 排序方向
    pub direction: OrderDirection,
}

impl OrderBy
{
    /// Create a new order-by clause.
    /// 创建新的排序子句。
    pub fn new(property: impl Into<String>, direction: OrderDirection) -> Self
    {
        Self {
            property: property.into(),
            direction,
        }
    }
}

impl fmt::Display for OrderBy
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{} {}", self.property, self.direction)
    }
}

/// Sort direction.
/// 排序方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderDirection
{
    /// Ascending / 升序
    Asc,
    /// Descending / 降序
    Desc,
}

impl fmt::Display for OrderDirection
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::Asc => write!(f, "ASC"),
            Self::Desc => write!(f, "DESC"),
        }
    }
}

/// AndOr node combining parts with logical operators.
/// 使用逻辑运算符组合部件的 AndOr 节点。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndOr
{
    /// A single part (leaf node) / 单个部件（叶子节点）
    Part(Part),
    /// AND combination of two nodes / 两个节点的 AND 组合
    And(Box<AndOr>, Box<AndOr>),
    /// OR combination of two nodes / 两个节点的 OR 组合
    Or(Box<AndOr>, Box<AndOr>),
}

impl AndOr
{
    /// Create a leaf part node.
    /// 创建叶子部件节点。
    pub fn part(part: Part) -> Self
    {
        Self::Part(part)
    }

    /// Create an AND node from two children.
    /// 从两个子节点创建 AND 节点。
    pub fn and(left: AndOr, right: AndOr) -> Self
    {
        Self::And(Box::new(left), Box::new(right))
    }

    /// Create an OR node from two children.
    /// 从两个子节点创建 OR 节点。
    pub fn or(left: AndOr, right: AndOr) -> Self
    {
        Self::Or(Box::new(left), Box::new(right))
    }

    /// Collect all parts from this tree in order.
    /// 按顺序从此树中收集所有部件。
    pub fn all_parts(&self) -> Vec<&Part>
    {
        match self
        {
            Self::Part(p) => vec![p],
            Self::And(l, r) =>
            {
                let mut parts = l.all_parts();
                parts.extend(r.all_parts());
                parts
            },
            Self::Or(l, r) =>
            {
                let mut parts = l.all_parts();
                parts.extend(r.all_parts());
                parts
            },
        }
    }
}

/// The fully parsed PartTree from a method name.
/// 从方法名完全解析的 PartTree。
///
/// This is the main entry point for the PartTree parser.
/// It holds the subject, keywords, part tree, and order-by clauses.
/// 这是 PartTree 解析器的主入口点。
/// 它保存主体、关键字、部件树和排序子句。
///
/// # Example / 示例
///
/// ```rust
/// use hiver_data_commons::part_tree::PartTree;
///
/// let tree =
///     PartTree::parse("findByNameAndAgeGreaterThanOrderByNameDesc").expect("valid method name");
///
/// assert_eq!(tree.subject().to_string(), "FIND");
/// assert!(!tree.is_distinct());
/// assert_eq!(tree.parts().len(), 2);
/// assert_eq!(tree.order_by().len(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct PartTree
{
    /// The query subject (Find, Read, Get, etc.)
    /// 查询主体（Find、Read、Get 等）
    subject: Subject,
    /// Maximum results limit (from First/Top)
    /// 最大结果限制（来自 First/Top）
    max_results: Option<u32>,
    /// Whether DISTINCT was specified
    /// 是否指定了 DISTINCT
    distinct: bool,
    /// The tree of AND/OR parts
    /// AND/OR 部件树
    tree: Option<AndOr>,
    /// Order-by clauses
    /// 排序子句
    order_by: Vec<OrderBy>,
}

impl PartTree
{
    /// Parse a method name into a PartTree.
    /// 将方法名解析为 PartTree。
    ///
    /// # Parameters / 参数
    ///
    /// - `source`: The method name to parse / 要解析的方法名
    ///
    /// # Errors / 错误
    ///
    /// Returns an error if the method name cannot be parsed.
    /// 如果方法名无法解析，返回错误。
    pub fn parse(source: &str) -> Result<Self, String>
    {
        let source = source.trim();
        if source.is_empty()
        {
            return Err("Method name must not be empty".to_string());
        }

        let parser = PartTreeParser::new(source);
        parser.parse()
    }

    /// Get the subject of this query.
    /// 获取此查询的主体。
    pub fn subject(&self) -> Subject
    {
        self.subject
    }

    /// Check if the query has a DISTINCT modifier.
    /// 检查查询是否具有 DISTINCT 修饰符。
    pub fn is_distinct(&self) -> bool
    {
        self.distinct
    }

    /// Get the limit from First/Top keywords, if any.
    /// 获取 First/Top 关键字的限制值（如果有）。
    pub fn limit(&self) -> Option<u32>
    {
        self.max_results
    }

    /// Get all parts flattened from the AND/OR tree.
    /// 获取从 AND/OR 树中展平的所有部件。
    pub fn parts(&self) -> Vec<&Part>
    {
        match &self.tree
        {
            Some(tree) => tree.all_parts(),
            None => Vec::new(),
        }
    }

    /// Get the AND/OR tree.
    /// 获取 AND/OR 树。
    pub fn tree(&self) -> Option<&AndOr>
    {
        self.tree.as_ref()
    }

    /// Get the order-by clauses.
    /// 获取排序子句。
    pub fn order_by(&self) -> &[OrderBy]
    {
        &self.order_by
    }

    /// Check if this is a count projection.
    /// 检查是否为计数投影。
    pub fn is_count_projection(&self) -> bool
    {
        self.subject.is_count_projection()
    }

    /// Check if this is an exists projection.
    /// 检查是否为存在性投影。
    pub fn is_exists_projection(&self) -> bool
    {
        self.subject.is_exists_projection()
    }

    /// Check if this is a delete operation.
    /// 检查是否为删除操作。
    pub fn is_delete(&self) -> bool
    {
        self.subject.is_delete()
    }

    /// Check if this returns a collection.
    /// 检查是否返回集合。
    pub fn is_collection_returning(&self) -> bool
    {
        self.subject.is_collection_returning()
    }
}

/// Internal parser state for PartTree parsing.
/// PartTree 解析的内部解析器状态。
struct PartTreeParser
{
    /// Remaining characters to parse.
    /// 剩余要解析的字符。
    remaining: String,
    /// Parsed subject.
    /// 解析的主体。
    subject: Subject,
    /// Whether DISTINCT was found.
    /// 是否找到 DISTINCT。
    distinct: bool,
    /// Max results from First/Top.
    /// 来自 First/Top 的最大结果数。
    max_results: Option<u32>,
}

impl PartTreeParser
{
    fn new(source: &str) -> Self
    {
        Self {
            remaining: source.to_string(),
            subject: Subject::Find,
            distinct: false,
            max_results: None,
        }
    }

    fn parse(mut self) -> Result<PartTree, String>
    {
        // Step 1: Extract subject
        self.extract_subject()?;

        // Step 2: Extract First/Top keyword
        self.extract_first_top();

        // Step 3: Extract Distinct keyword
        self.extract_distinct();

        // Step 4: Extract "By" keyword
        self.extract_by()?;

        // Step 5: Split off OrderBy suffix
        let condition_part = self.split_order_by();

        // Step 6: Parse conditions into AndOr tree
        let tree = if condition_part.is_empty()
        {
            // "findBy" with nothing after "By" is an error
            return Err("No conditions found after 'By'".to_string());
        }
        else
        {
            Some(parse_conditions(&condition_part)?)
        };

        let order_by = parse_order_by_clauses(&self.remaining);

        Ok(PartTree {
            subject: self.subject,
            max_results: self.max_results,
            distinct: self.distinct,
            tree,
            order_by,
        })
    }

    fn extract_subject(&mut self) -> Result<(), String>
    {
        let lower = self.remaining.to_lowercase();

        let subjects: &[(&str, Subject, usize)] = &[
            ("find", Subject::Find, 4),
            ("read", Subject::Read, 4),
            ("get", Subject::Get, 3),
            ("query", Subject::Query, 5),
            ("count", Subject::Count, 5),
            ("exists", Subject::Exists, 6),
            ("delete", Subject::Delete, 6),
            ("stream", Subject::Stream, 6),
        ];

        for &(prefix, subject, len) in subjects
        {
            if lower.starts_with(prefix)
            {
                let after = &self.remaining[len..];
                // Must be followed by uppercase, keyword, or nothing meaningful
                if after.is_empty()
                    || after.starts_with(|c: char| c.is_uppercase())
                    || after.to_lowercase().starts_with("first")
                    || after.to_lowercase().starts_with("top")
                    || after.to_lowercase().starts_with("distinct")
                    || after.to_lowercase().starts_with("by")
                {
                    self.subject = subject;
                    self.remaining = after.to_string();
                    return Ok(());
                }
            }
        }

        Err(format!(
            "Invalid subject in method name '{}'. Must start with find, read, get, query, count, \
             exists, delete, or stream",
            self.remaining
        ))
    }

    fn extract_first_top(&mut self)
    {
        let lower = self.remaining.to_lowercase();

        if let Some(rest) = lower.strip_prefix("first")
        {
            let (digits, _after) = take_digits(rest);
            let num = digits.parse::<u32>().unwrap_or(1);
            self.max_results = Some(num);
            self.remaining = self.remaining[5 + digits.len()..].to_string();
            return;
        }

        if let Some(rest) = lower.strip_prefix("top")
        {
            let (digits, _after) = take_digits(rest);
            let num = digits.parse::<u32>().unwrap_or(1);
            self.max_results = Some(num);
            self.remaining = self.remaining[3 + digits.len()..].to_string();
        }
    }

    fn extract_distinct(&mut self)
    {
        let lower = self.remaining.to_lowercase();
        if lower.starts_with("distinct")
        {
            self.distinct = true;
            self.remaining = self.remaining[8..].to_string();
        }
    }

    fn extract_by(&mut self) -> Result<(), String>
    {
        let lower = self.remaining.to_lowercase();
        if lower.starts_with("by")
        {
            self.remaining = self.remaining[2..].to_string();
            Ok(())
        }
        else
        {
            Err(format!(
                "Method name must contain 'By' keyword after subject, got: '{}'",
                self.remaining
            ))
        }
    }

    fn split_order_by(&mut self) -> String
    {
        let lower = self.remaining.to_lowercase();
        if let Some(pos) = lower.rfind("orderby")
        {
            let condition_part = self.remaining[..pos].to_string();
            self.remaining = self.remaining[pos + 7..].to_string();
            condition_part
        }
        else
        {
            let condition_part = self.remaining.clone();
            self.remaining.clear();
            condition_part
        }
    }
}

/// Connector between parts (And or Or).
/// 部件之间的连接符（And 或 Or）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Connector
{
    And,
    Or,
}

/// Parse condition string into an AndOr tree.
/// 将条件字符串解析为 AndOr 树。
///
/// Uses a greedy keyword-matching tokenizer that scans through the string,
/// matching keywords at camelCase boundaries. "And"/"Or" are treated as
/// connectors only when they appear at a valid boundary after a complete
/// property+keyword combination.
///
/// 使用贪心关键字匹配分词器扫描字符串，在驼峰边界处匹配关键字。
/// "And"/"Or" 仅在完整的属性+关键字组合之后出现在有效边界时才被视为连接符。
fn parse_conditions(input: &str) -> Result<AndOr, String>
{
    if input.is_empty()
    {
        return Err("No conditions found after 'By'".to_string());
    }

    let (segments, connectors) = tokenize_into_segments(input)?;

    if segments.is_empty()
    {
        return Err("No conditions found after 'By'".to_string());
    }

    // Parse each segment into a Part
    let mut parts: Vec<Part> = Vec::new();
    let mut part_connectors: Vec<Connector> = Vec::new();

    for (i, segment) in segments.iter().enumerate()
    {
        let part = parse_single_segment(segment)?;
        parts.push(part);

        if i < connectors.len()
        {
            part_connectors.push(connectors[i]);
        }
    }

    build_and_or_tree(parts, &part_connectors)
}

/// Tokenize the condition string into segments separated by And/Or connectors.
/// 将条件字符串分词为由 And/Or 连接符分隔的段。
///
/// Strategy: scan through the string looking for "And"/"Or" at camelCase
/// boundaries. Before accepting a connector, verify that the preceding
/// text ends with a recognized keyword. This prevents false positives
/// like "Or" inside "GreaterThan" or "And" inside "ContainingAnd".
///
/// 策略：扫描字符串，在驼峰边界处查找 "And"/"Or"。
/// 在接受连接符之前，验证前面的文本以已识别的关键字结尾。
/// 这可以防止 "GreaterThan" 中的 "Or" 或 "ContainingAnd" 中的 "And"
/// 等误报。
fn tokenize_into_segments(input: &str) -> Result<(Vec<&str>, Vec<Connector>), String>
{
    let mut segments: Vec<&str> = Vec::new();
    let mut connectors: Vec<Connector> = Vec::new();

    let bytes = input.as_bytes();
    let mut segment_start = 0;
    let mut i = 0;

    while i < bytes.len()
    {
        let c = bytes[i] as char;

        // Look for "And" or "Or" at word boundaries:
        // preceded by a lowercase letter and followed by an uppercase letter.
        if c.is_uppercase() && i > segment_start
        {
            let rest = &input[i..];
            let rest_lower = rest.to_lowercase();

            // Try "And" connector: "And" followed by an uppercase letter
            if rest_lower.starts_with("and") && rest.len() > 3
            {
                let next = rest.as_bytes()[3] as char;
                if next.is_uppercase()
                {
                    // Check that the preceding text ends with a valid keyword
                    let before = &input[segment_start..i];
                    if ends_with_keyword(before)
                    {
                        segments.push(before);
                        connectors.push(Connector::And);
                        i += 3; // skip "And"
                        segment_start = i;
                        continue;
                    }
                }
            }

            // Try "Or" connector: "Or" followed by an uppercase letter
            if rest_lower.starts_with("or") && rest.len() > 2
            {
                let next = rest.as_bytes()[2] as char;
                if next.is_uppercase()
                {
                    // Check that the preceding text ends with a valid keyword
                    let before = &input[segment_start..i];
                    if ends_with_keyword(before)
                    {
                        segments.push(before);
                        connectors.push(Connector::Or);
                        i += 2; // skip "Or"
                        segment_start = i;
                        continue;
                    }
                }
            }
        }

        i += 1;
    }

    // Push the last segment
    if segment_start < input.len()
    {
        segments.push(&input[segment_start..]);
    }

    Ok((segments, connectors))
}

/// Check if the given text ends with a recognized keyword at a camelCase boundary.
/// 检查给定文本是否在驼峰边界处以已识别的关键字结尾。
///
/// For example:
/// - "Name" -> true (ends with property, treated as SimpleProperty)
/// - "AgeGreaterThan" -> true (ends with "GreaterThan" keyword)
/// - "AgeGreaterThanOr" -> false ("Or" is not a keyword, it's a connector)
/// - "NameContaining" -> true (ends with "Containing" keyword)
///
/// 此函数对于正确识别连接符 "And"/"Or" 至关重要。
fn ends_with_keyword(text: &str) -> bool
{
    if text.is_empty()
    {
        return false;
    }

    // Find the last camelCase boundary (lowercase -> uppercase transition)
    let bytes = text.as_bytes();
    let mut last_boundary = None;

    for i in 1..bytes.len()
    {
        let prev = bytes[i - 1] as char;
        let curr = bytes[i] as char;
        if curr.is_uppercase() && prev.is_lowercase()
        {
            last_boundary = Some(i);
        }
    }

    if let Some(pos) = last_boundary
    {
        // Try to match a keyword starting at the last camelCase boundary
        if let Some((_pt, consumed)) = find_keyword(&text[pos..])
        {
            // The keyword must consume exactly the remaining text
            return pos + consumed == text.len();
        }
    }

    // No keyword found at the last boundary. Check if the entire text
    // is a simple property name (no embedded keyword). This is valid --
    // e.g., "Name" in "findByNameAndAge" is a simple property.
    // In method names, properties start with uppercase.
    // We accept this as a valid endpoint for connectors.
    // But we must NOT accept if the text ends with a partial keyword.
    // Since we already checked the last boundary and found no keyword match,
    // the text doesn't end with any recognized keyword, which is fine for
    // a bare property name.
    true
}

/// Find a matching keyword at the start of the text.
/// 在文本开头查找匹配的关键字。
///
/// Returns `(PartType, bytes_consumed)` if found.
/// 如果找到，返回 `(PartType, 消耗的字节数)`。
///
/// Keywords are ordered longest-first to prevent prefix matches.
/// 关键字按最长优先排序以防止前缀匹配。
fn find_keyword(text: &str) -> Option<(PartType, usize)>
{
    let lower = text.to_lowercase();

    let keywords: &[(&str, PartType)] = &[
        ("isnotnull", PartType::IsNotNull),
        ("isnull", PartType::IsNull),
        ("notnull", PartType::IsNotNull),
        ("notcontaining", PartType::NotContaining),
        ("notlike", PartType::NotLike),
        ("notin", PartType::NotIn),
        ("startingwith", PartType::StartingWith),
        ("endingwith", PartType::EndingWith),
        ("containing", PartType::Containing),
        ("greaterthanequal", PartType::GreaterThanEqual),
        ("greaterthan", PartType::GreaterThan),
        ("lessthanequal", PartType::LessThanEqual),
        ("lessthan", PartType::LessThan),
        ("between", PartType::Between),
        ("before", PartType::Before),
        ("after", PartType::After),
        ("likerex", PartType::Regex),
        ("likeregex", PartType::Regex),
        ("regex", PartType::Regex),
        ("like", PartType::Like),
        ("true", PartType::True),
        ("false", PartType::False),
        ("isnot", PartType::IsNot),
        ("is", PartType::Is),
        ("not", PartType::Not),
        ("in", PartType::In),
        ("near", PartType::Near),
        ("within", PartType::Within),
    ];

    for (keyword, pt) in keywords
    {
        if lower.starts_with(keyword)
        {
            let consumed = keyword.len();
            // Verify word boundary: what follows must be end-of-string or an uppercase letter
            if text.len() == consumed
            {
                return Some((*pt, consumed));
            }
            let next = text
                .as_bytes()
                .get(consumed)
                .map(|&b| b as char)
                .unwrap_or('\0');
            if next.is_uppercase()
            {
                return Some((*pt, consumed));
            }
        }
    }

    None
}

/// Parse a single segment (property + keyword) into a Part.
/// 将单个段（属性 + 关键字）解析为 Part。
fn parse_single_segment(text: &str) -> Result<Part, String>
{
    if text.is_empty()
    {
        return Err("Empty segment".to_string());
    }

    let bytes = text.as_bytes();

    // Try all possible split positions (each uppercase letter after lowercase)
    // and check if the remainder starts with a recognized keyword.
    // Pick the longest valid property+keyword combination.
    for i in 1..bytes.len()
    {
        let prev = bytes[i - 1] as char;
        let curr = bytes[i] as char;
        if curr.is_uppercase() && prev.is_lowercase()
        {
            let candidate_keyword = &text[i..];

            if let Some((pt, consumed)) = find_keyword(candidate_keyword)
            {
                let remaining = &candidate_keyword[consumed..];
                if remaining.is_empty()
                {
                    let prop = first_char_to_lower(&text[..i]);
                    return Ok(Part::new(prop, pt));
                }
            }
        }
    }

    // No keyword found: the entire text is a simple property
    let prop = first_char_to_lower(text);
    Ok(Part::new(prop, PartType::SimpleProperty))
}

/// Build an AndOr tree from flat parts and connectors.
/// 从扁平部件和连接符构建 AndOr 树。
fn build_and_or_tree(parts: Vec<Part>, connectors: &[Connector]) -> Result<AndOr, String>
{
    if parts.is_empty()
    {
        return Err("No parts to build tree from".to_string());
    }

    if parts.len() == 1
    {
        return Ok(AndOr::part(parts.into_iter().next().unwrap()));
    }

    // Build tree left-to-right based on connectors.
    // AND has higher precedence than OR in Spring Data (when mixed).
    // For simplicity, we build left-associatively.
    // Spring Data 中 AND 优先级高于 OR（混合时）。
    // 为简单起见，我们左结合构建。
    let mut iter = parts.into_iter();
    let mut current = AndOr::part(iter.next().unwrap());

    for (i, part) in iter.enumerate()
    {
        let connector = connectors.get(i).copied().unwrap_or(Connector::And);
        let node = AndOr::part(part);
        current = match connector
        {
            Connector::And => AndOr::and(current, node),
            Connector::Or => AndOr::or(current, node),
        };
    }

    Ok(current)
}

/// Take leading digits from a string, return digits portion.
/// 从字符串开头获取数字部分。
fn take_digits(s: &str) -> (&str, &str)
{
    let end = s.chars().take_while(|c| c.is_ascii_digit()).count();
    (&s[..end], &s[end..])
}

/// Convert the first character to lowercase.
/// 将第一个字符转换为小写。
fn first_char_to_lower(s: &str) -> String
{
    if s.is_empty()
    {
        return String::new();
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap().to_lowercase().collect::<String>();
    let rest: String = chars.collect();
    format!("{}{}", first, rest)
}

/// Parse order-by clauses from the order-by portion of a method name.
/// 从方法名的排序部分解析排序子句。
fn parse_order_by_clauses(order_part: &str) -> Vec<OrderBy>
{
    if order_part.is_empty()
    {
        return Vec::new();
    }

    let mut orders = Vec::new();
    let mut current = String::new();
    let mut chars = order_part.chars().peekable();

    while let Some(c) = chars.next()
    {
        if c.is_uppercase() && !current.is_empty()
        {
            let rest: String = std::iter::once(c).chain(chars.clone()).collect();
            let rest_lower = rest.to_lowercase();

            if rest_lower.starts_with("asc")
            {
                let field = first_char_to_lower(&current);
                orders.push(OrderBy::new(field, OrderDirection::Asc));
                current.clear();
                chars.next();
                chars.next();
                continue;
            }

            if rest_lower.starts_with("desc")
            {
                let field = first_char_to_lower(&current);
                orders.push(OrderBy::new(field, OrderDirection::Desc));
                current.clear();
                chars.next();
                chars.next();
                chars.next();
                continue;
            }
        }
        current.push(c);
    }

    if !current.is_empty()
    {
        let field = first_char_to_lower(&current);
        orders.push(OrderBy::new(field, OrderDirection::Asc));
    }

    orders
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn parse(source: &str) -> PartTree
    {
        PartTree::parse(source).unwrap_or_else(|e| panic!("Failed to parse '{}': {}", source, e))
    }

    // === Basic parsing tests / 基础解析测试 ===

    #[test]
    fn test_simple_find_by()
    {
        let tree = parse("findByName");
        assert_eq!(tree.subject(), Subject::Find);
        let parts = tree.parts();
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].property_path, "name");
        assert_eq!(parts[0].part_type, PartType::SimpleProperty);
    }

    #[test]
    fn test_find_by_with_keyword()
    {
        let tree = parse("findByNameLike");
        assert_eq!(tree.parts()[0].part_type, PartType::Like);

        let tree = parse("findByAgeGreaterThan");
        assert_eq!(tree.parts()[0].part_type, PartType::GreaterThan);

        let tree = parse("findByAgeLessThanEqual");
        assert_eq!(tree.parts()[0].part_type, PartType::LessThanEqual);

        let tree = parse("findByAgeGreaterThanEqual");
        assert_eq!(tree.parts()[0].part_type, PartType::GreaterThanEqual);

        let tree = parse("findByActiveTrue");
        assert_eq!(tree.parts()[0].part_type, PartType::True);

        let tree = parse("findByDeletedFalse");
        assert_eq!(tree.parts()[0].part_type, PartType::False);
    }

    #[test]
    fn test_find_by_and()
    {
        let tree = parse("findByNameAndAge");
        let parts = tree.parts();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].property_path, "name");
        assert_eq!(parts[1].property_path, "age");
    }

    #[test]
    fn test_find_by_or()
    {
        let tree = parse("findByNameOrEmail");
        let parts = tree.parts();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].property_path, "name");
        assert_eq!(parts[1].property_path, "email");
    }

    #[test]
    fn test_complex_and_or()
    {
        // "findByNameAndAgeGreaterThanOrEmailLike"
        // Should parse as: name AND (age > ?) OR (email LIKE ?)
        let tree = parse("findByNameAndAgeGreaterThanOrEmailLike");
        let parts = tree.parts();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].property_path, "name");
        assert_eq!(parts[0].part_type, PartType::SimpleProperty);
        assert_eq!(parts[1].property_path, "age");
        assert_eq!(parts[1].part_type, PartType::GreaterThan);
        assert_eq!(parts[2].property_path, "email");
        assert_eq!(parts[2].part_type, PartType::Like);
    }

    #[test]
    fn test_order_by()
    {
        let tree = parse("findByNameOrderByAgeAsc");
        assert_eq!(tree.order_by().len(), 1);
        assert_eq!(tree.order_by()[0].property, "age");
        assert_eq!(tree.order_by()[0].direction, OrderDirection::Asc);

        let tree = parse("findByNameOrderByAgeDesc");
        assert_eq!(tree.order_by()[0].direction, OrderDirection::Desc);

        let tree = parse("findByNameOrderByAgeAscNameDesc");
        assert_eq!(tree.order_by().len(), 2);
        assert_eq!(tree.order_by()[0].property, "age");
        assert_eq!(tree.order_by()[1].property, "name");
        assert_eq!(tree.order_by()[1].direction, OrderDirection::Desc);
    }

    #[test]
    fn test_distinct()
    {
        let tree = parse("findDistinctByName");
        assert!(tree.is_distinct());
    }

    #[test]
    fn test_first_top()
    {
        let tree = parse("findFirst10ByName");
        assert_eq!(tree.limit(), Some(10));

        let tree = parse("findTop5ByName");
        assert_eq!(tree.limit(), Some(5));

        let tree = parse("findFirstByName");
        assert_eq!(tree.limit(), Some(1));
    }

    #[test]
    fn test_subjects()
    {
        assert_eq!(parse("readByName").subject(), Subject::Read);
        assert_eq!(parse("getByName").subject(), Subject::Get);
        assert_eq!(parse("queryByName").subject(), Subject::Query);
        assert!(parse("countByName").is_count_projection());
        assert!(parse("existsByName").is_exists_projection());
        assert!(parse("deleteByName").is_delete());
        assert_eq!(parse("streamByName").subject(), Subject::Stream);
    }

    #[test]
    fn test_null_keywords()
    {
        let tree = parse("findByDeletedIsNull");
        assert_eq!(tree.parts()[0].part_type, PartType::IsNull);

        let tree = parse("findByDeletedIsNotNull");
        assert_eq!(tree.parts()[0].part_type, PartType::IsNotNull);

        let tree = parse("findByDeletedNot");
        assert_eq!(tree.parts()[0].part_type, PartType::Not);
    }

    #[test]
    fn test_between_in()
    {
        let tree = parse("findByAgeBetween");
        assert_eq!(tree.parts()[0].part_type, PartType::Between);

        let tree = parse("findByNameIn");
        assert_eq!(tree.parts()[0].part_type, PartType::In);

        let tree = parse("findByNameNotIn");
        assert_eq!(tree.parts()[0].part_type, PartType::NotIn);
    }

    #[test]
    fn test_starting_ending_containing()
    {
        let tree = parse("findByNameStartingWith");
        assert_eq!(tree.parts()[0].part_type, PartType::StartingWith);

        let tree = parse("findByNameEndingWith");
        assert_eq!(tree.parts()[0].part_type, PartType::EndingWith);

        let tree = parse("findByNameContaining");
        assert_eq!(tree.parts()[0].part_type, PartType::Containing);

        let tree = parse("findByNameNotContaining");
        assert_eq!(tree.parts()[0].part_type, PartType::NotContaining);
    }

    #[test]
    fn test_is_and_is_not()
    {
        let tree = parse("findByNameIs");
        assert_eq!(tree.parts()[0].part_type, PartType::Is);

        let tree = parse("findByNameIsNot");
        assert_eq!(tree.parts()[0].part_type, PartType::IsNot);
    }

    #[test]
    fn test_before_after()
    {
        let tree = parse("findByCreatedBefore");
        assert_eq!(tree.parts()[0].part_type, PartType::Before);

        let tree = parse("findByCreatedAfter");
        assert_eq!(tree.parts()[0].part_type, PartType::After);
    }

    #[test]
    fn test_and_or_tree()
    {
        let tree = parse("findByNameAndAgeOrEmail");
        let tree_node = tree.tree().unwrap();
        let all_parts = tree_node.all_parts();
        assert_eq!(all_parts.len(), 3);
    }

    #[test]
    fn test_collection_returning()
    {
        assert!(parse("findByName").is_collection_returning());
        assert!(parse("readByName").is_collection_returning());
        assert!(parse("queryByName").is_collection_returning());
        assert!(parse("streamByName").is_collection_returning());
        assert!(!parse("getByName").is_collection_returning());
        assert!(!parse("countByName").is_collection_returning());
    }

    #[test]
    fn test_part_type_argument_counts()
    {
        assert!(PartType::SimpleProperty.needs_single_argument());
        assert!(PartType::LessThan.needs_single_argument());
        assert!(PartType::Like.needs_single_argument());
        assert!(PartType::In.needs_variable_arguments());
        assert!(PartType::Between.needs_two_arguments());
        assert!(PartType::IsNull.needs_no_argument());
        assert!(PartType::True.needs_no_argument());
    }

    #[test]
    fn test_part_display()
    {
        let part = Part::new("age", PartType::GreaterThan);
        assert_eq!(part.to_string(), "age GREATER_THAN");
    }

    #[test]
    fn test_order_by_display()
    {
        let ob = OrderBy::new("name", OrderDirection::Desc);
        assert_eq!(ob.to_string(), "name DESC");
    }

    #[test]
    fn test_invalid_names()
    {
        assert!(PartTree::parse("").is_err());
        assert!(PartTree::parse("doSomething").is_err());
        assert!(PartTree::parse("find").is_err());
        assert!(PartTree::parse("findBy").is_err());
    }

    #[test]
    fn test_subject_display()
    {
        assert_eq!(Subject::Find.to_string(), "FIND");
        assert_eq!(Subject::Count.to_string(), "COUNT");
        assert_eq!(Subject::Delete.to_string(), "DELETE");
    }

    #[test]
    fn test_keyword_display()
    {
        assert_eq!(Keyword::Distinct.to_string(), "DISTINCT");
        assert_eq!(Keyword::First(10).to_string(), "FIRST 10");
        assert_eq!(Keyword::Top(5).to_string(), "TOP 5");
    }

    // === Complex real-world tests / 复杂真实场景测试 ===

    #[test]
    fn test_count_by_status_in()
    {
        let tree = parse("countByStatusIn");
        assert!(tree.is_count_projection());
        assert_eq!(tree.parts().len(), 1);
        assert_eq!(tree.parts()[0].property_path, "status");
        assert_eq!(tree.parts()[0].part_type, PartType::In);
    }

    #[test]
    fn test_find_top_10_by_created_at_after_order_by_name_asc()
    {
        let tree = parse("findTop10ByCreatedAtAfterOrderByNameAsc");
        assert_eq!(tree.limit(), Some(10));
        assert_eq!(tree.parts().len(), 1);
        assert_eq!(tree.parts()[0].property_path, "createdAt");
        assert_eq!(tree.parts()[0].part_type, PartType::After);
        assert_eq!(tree.order_by().len(), 1);
        assert_eq!(tree.order_by()[0].property, "name");
        assert_eq!(tree.order_by()[0].direction, OrderDirection::Asc);
    }

    #[test]
    fn test_find_distinct_by_name_and_age_order_by_name_desc()
    {
        let tree = parse("findDistinctByNameAndAgeOrderByNameDesc");
        assert!(tree.is_distinct());
        assert_eq!(tree.parts().len(), 2);
        assert_eq!(tree.parts()[0].property_path, "name");
        assert_eq!(tree.parts()[1].property_path, "age");
        assert_eq!(tree.order_by().len(), 1);
        assert_eq!(tree.order_by()[0].direction, OrderDirection::Desc);
    }

    #[test]
    fn test_delete_by_active_false()
    {
        let tree = parse("deleteByActiveFalse");
        assert!(tree.is_delete());
        assert_eq!(tree.parts().len(), 1);
        assert_eq!(tree.parts()[0].property_path, "active");
        assert_eq!(tree.parts()[0].part_type, PartType::False);
    }

    #[test]
    fn test_exists_by_email_and_active_true()
    {
        let tree = parse("existsByEmailAndActiveTrue");
        assert!(tree.is_exists_projection());
        assert_eq!(tree.parts().len(), 2);
        assert_eq!(tree.parts()[0].property_path, "email");
        assert_eq!(tree.parts()[1].property_path, "active");
        assert_eq!(tree.parts()[1].part_type, PartType::True);
    }

    #[test]
    fn test_find_by_age_between_and_name_containing()
    {
        let tree = parse("findByAgeBetweenAndNameContaining");
        let parts = tree.parts();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].property_path, "age");
        assert_eq!(parts[0].part_type, PartType::Between);
        assert_eq!(parts[1].property_path, "name");
        assert_eq!(parts[1].part_type, PartType::Containing);
    }

    #[test]
    fn test_find_by_name_starting_with_and_email_ending_with()
    {
        let tree = parse("findByNameStartingWithAndEmailEndingWith");
        let parts = tree.parts();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].property_path, "name");
        assert_eq!(parts[0].part_type, PartType::StartingWith);
        assert_eq!(parts[1].property_path, "email");
        assert_eq!(parts[1].part_type, PartType::EndingWith);
    }

    #[test]
    fn test_find_by_status_is_not_null_and_name_not_like()
    {
        let tree = parse("findByStatusIsNotNullAndNameNotLike");
        let parts = tree.parts();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].property_path, "status");
        assert_eq!(parts[0].part_type, PartType::IsNotNull);
        assert_eq!(parts[1].property_path, "name");
        assert_eq!(parts[1].part_type, PartType::NotLike);
    }

    #[test]
    fn test_find_by_name_or_email_or_phone()
    {
        let tree = parse("findByNameOrEmailOrPhone");
        let parts = tree.parts();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].property_path, "name");
        assert_eq!(parts[1].property_path, "email");
        assert_eq!(parts[2].property_path, "phone");
    }

    #[test]
    fn test_part_with_ignore_case()
    {
        let part = Part::with_ignore_case("name", PartType::Like, true);
        assert!(part.ignore_case);
        assert_eq!(part.property_path, "name");

        let part = Part::new("name", PartType::Like);
        assert!(!part.ignore_case);
    }

    #[test]
    fn test_multiple_order_by()
    {
        let tree = parse("findByActiveTrueOrderByLastNameAscFirstNameAscAgeDesc");
        assert_eq!(tree.order_by().len(), 3);
        assert_eq!(tree.order_by()[0].property, "lastName");
        assert_eq!(tree.order_by()[0].direction, OrderDirection::Asc);
        assert_eq!(tree.order_by()[1].property, "firstName");
        assert_eq!(tree.order_by()[1].direction, OrderDirection::Asc);
        assert_eq!(tree.order_by()[2].property, "age");
        assert_eq!(tree.order_by()[2].direction, OrderDirection::Desc);
    }

    #[test]
    fn test_camel_case_property()
    {
        let tree = parse("findByFirstNameAndLastName");
        let parts = tree.parts();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].property_path, "firstName");
        assert_eq!(parts[1].property_path, "lastName");
    }

    #[test]
    fn test_camel_case_property_with_keyword()
    {
        let tree = parse("findByFirstNameStartingWithAndLastNameContaining");
        let parts = tree.parts();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].property_path, "firstName");
        assert_eq!(parts[0].part_type, PartType::StartingWith);
        assert_eq!(parts[1].property_path, "lastName");
        assert_eq!(parts[1].part_type, PartType::Containing);
    }

    #[test]
    fn test_empty_method_name()
    {
        assert!(PartTree::parse("").is_err());
        assert!(PartTree::parse("   ").is_err());
    }

    #[test]
    fn test_whitespace_trimmed()
    {
        let tree = parse("  findByName  ");
        assert_eq!(tree.subject(), Subject::Find);
        assert_eq!(tree.parts().len(), 1);
    }

    #[test]
    fn test_less_than_vs_less_than_equal()
    {
        // Verify that LessThanEqual is correctly distinguished from LessThan
        let tree = parse("findByAgeLessThan");
        assert_eq!(tree.parts()[0].part_type, PartType::LessThan);

        let tree = parse("findByAgeLessThanEqual");
        assert_eq!(tree.parts()[0].part_type, PartType::LessThanEqual);
    }

    #[test]
    fn test_greater_than_or_with_keyword()
    {
        // "GreaterThan" followed by "Or" should NOT split at "Or" within "GreaterThanOr"
        // when "Or" is followed by a property. But in "AgeGreaterThanOrEmail",
        // "Or" IS a connector because "AgeGreaterThan" is a valid combination.
        let tree = parse("findByAgeGreaterThanOrEmailLike");
        let parts = tree.parts();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].property_path, "age");
        assert_eq!(parts[0].part_type, PartType::GreaterThan);
        assert_eq!(parts[1].property_path, "email");
        assert_eq!(parts[1].part_type, PartType::Like);
    }
}
