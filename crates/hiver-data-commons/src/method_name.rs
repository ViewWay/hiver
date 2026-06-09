//! Method name parsing for derived queries
//! 方法名解析用于派生查询
//!
//! # Overview / 概述
//!
//! This module implements Spring Data's method name query derivation.
//! Method names like `findByNameAndAgeGreaterThan` are parsed into
//! query conditions automatically.
//!
//! 本模块实现 Spring Data 的方法名查询派生。
//! 像 `findByNameAndAgeGreaterThan` 这样的方法名会被自动解析为查询条件。
//!
//! # Supported Keywords / 支持的关键字
//!
//! | Keyword | Example / 示例 | JPQL / SQL |
//! |---------|---------------|------------|
//! | Is, Equals | `findByNameIs` | `WHERE name = ?` |
//! | Like | `findByNameLike` | `WHERE name LIKE ?` |
//! | StartingWith | `findByNameStartingWith` | `WHERE name LIKE '?%'` |
//! | EndingWith | `findByNameEndingWith` | `WHERE name LIKE '%?'` |
//! | Containing | `findByNameContaining` | `WHERE name LIKE '%?%'` |
//! | Not | `findByNameNot` | `WHERE name <> ?` |
//! | GreaterThan | `findByAgeGreaterThan` | `WHERE age > ?` |
//! | LessThan | `findByAgeLessThan` | `WHERE age < ?` |
//! | Between | `findByAgeBetween` | `WHERE age BETWEEN ? AND ?` |
//! | In | `findByNameIn` | `WHERE name IN (?)` |
//! | IsNull | `findByNameIsNull` | `WHERE name IS NULL` |
//! | IsNotNull | `findByNameIsNotNull` | `WHERE name IS NOT NULL` |
//! | Before | `findByCreatedBefore` | `WHERE created < ?` |
//! | After | `findByCreatedAfter` | `WHERE created > ?` |
//! | True | `findByActiveTrue` | `WHERE active = true` |
//! | False | `findByActiveFalse` | `WHERE active = false` |
//! | OrderBy | `findByNameOrderByAgeAsc` | `ORDER BY age ASC` |
//! | And | `findByNameAndAge` | `WHERE name = ? AND age = ?` |
//! | Or | `findByNameOrEmail` | `WHERE name = ? OR email = ?` |

use std::fmt;

/// Query condition from method name parsing
/// 方法名解析产生的查询条件
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition
{
    /// Equality: field = value
    /// 等于
    Equals
    {
        /// Field name / 字段名
        field: String,
    },
    /// Like: field LIKE value
    /// 模糊匹配
    Like
    {
        /// Field name / 字段名
        field: String,
    },
    /// Starting with: field LIKE 'value%'
    /// 以...开头
    StartingWith
    {
        /// Field name / 字段名
        field: String,
    },
    /// Ending with: field LIKE '%value'
    /// 以...结尾
    EndingWith
    {
        /// Field name / 字段名
        field: String,
    },
    /// Containing: field LIKE '%value%'
    /// 包含
    Containing
    {
        /// Field name / 字段名
        field: String,
    },
    /// Not equal: field != value
    /// 不等于
    NotEquals
    {
        /// Field name / 字段名
        field: String,
    },
    /// Greater than: field > value
    /// 大于
    GreaterThan
    {
        /// Field name / 字段名
        field: String,
    },
    /// Less than: field < value
    /// 小于
    LessThan
    {
        /// Field name / 字段名
        field: String,
    },
    /// Between: field BETWEEN value1 AND value2
    /// 范围
    Between
    {
        /// Field name / 字段名
        field: String,
    },
    /// In: field IN (values)
    /// 包含于
    In
    {
        /// Field name / 字段名
        field: String,
    },
    /// Not like: field NOT LIKE value
    /// 不匹配
    NotLike
    {
        /// Field name / 字段名
        field: String,
    },
    /// Not in: field NOT IN (values)
    /// 不包含于
    NotIn
    {
        /// Field name / 字段名
        field: String,
    },
    /// IS NULL: field IS NULL
    /// 为空
    IsNull
    {
        /// Field name / 字段名
        field: String,
    },
    /// IS NOT NULL: field IS NOT NULL
    /// 非空
    IsNotNull
    {
        /// Field name / 字段名
        field: String,
    },
    /// True: field = true
    /// 为真
    True
    {
        /// Field name / 字段名
        field: String,
    },
    /// False: field = false
    /// 为假
    False
    {
        /// Field name / 字段名
        field: String,
    },
    /// Before (date): field < value
    /// 日期之前
    Before
    {
        /// Field name / 字段名
        field: String,
    },
    /// After (date): field > value
    /// 日期之后
    After
    {
        /// Field name / 字段名
        field: String,
    },
}

impl Condition
{
    /// Get the field name for this condition
    /// 获取此条件的字段名
    pub fn field(&self) -> &str
    {
        match self
        {
            Condition::Equals { field }
            | Condition::Like { field }
            | Condition::StartingWith { field }
            | Condition::EndingWith { field }
            | Condition::Containing { field }
            | Condition::NotEquals { field }
            | Condition::GreaterThan { field }
            | Condition::LessThan { field }
            | Condition::Between { field }
            | Condition::In { field }
            | Condition::NotLike { field }
            | Condition::NotIn { field }
            | Condition::IsNull { field }
            | Condition::IsNotNull { field }
            | Condition::True { field }
            | Condition::False { field }
            | Condition::Before { field }
            | Condition::After { field } => field,
        }
    }

    /// Get the operator SQL fragment for this condition
    /// 获取此条件的 SQL 运算符片段
    pub fn operator_sql(&self) -> &'static str
    {
        match self
        {
            Condition::Equals { .. } => "=",
            Condition::Like { .. } => "LIKE",
            Condition::StartingWith { .. } => "LIKE",
            Condition::EndingWith { .. } => "LIKE",
            Condition::Containing { .. } => "LIKE",
            Condition::NotEquals { .. } => "<>",
            Condition::GreaterThan { .. } => ">",
            Condition::LessThan { .. } => "<",
            Condition::Between { .. } => "BETWEEN",
            Condition::In { .. } => "IN",
            Condition::NotLike { .. } => "NOT LIKE",
            Condition::NotIn { .. } => "NOT IN",
            Condition::IsNull { .. } => "IS NULL",
            Condition::IsNotNull { .. } => "IS NOT NULL",
            Condition::True { .. } => "=",
            Condition::False { .. } => "=",
            Condition::Before { .. } => "<",
            Condition::After { .. } => ">",
        }
    }

    /// Whether this condition requires no parameter value
    /// 此条件是否需要参数值
    pub fn needs_no_value(&self) -> bool
    {
        matches!(
            self,
            Condition::IsNull { .. }
                | Condition::IsNotNull { .. }
                | Condition::True { .. }
                | Condition::False { .. }
        )
    }

    /// Whether this condition requires two parameter values
    /// 此条件是否需要两个参数值
    pub fn needs_two_values(&self) -> bool
    {
        matches!(self, Condition::Between { .. })
    }
}

/// Logical connector between conditions
/// 条件之间的逻辑连接符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Connector
{
    /// AND
    And,
    /// OR
    Or,
}

/// A parsed method name result
/// 解析后的方法名结果
///
/// Contains the query prefix (find, read, query, etc.) and
/// the list of parsed conditions with their connectors.
/// 包含查询前缀和解析后的条件列表及其连接符。
#[derive(Debug, Clone)]
pub struct ParsedMethodName
{
    /// Query prefix (find, read, query, get, count, delete, exists)
    /// 查询前缀
    pub prefix: QueryPrefix,
    /// Parsed conditions in order
    /// 按顺序排列的解析条件
    pub conditions: Vec<(Condition, Option<Connector>)>,
    /// ORDER BY fields and directions
    /// 排序字段和方向
    pub order_by: Vec<OrderByClause>,
    /// Whether a Distinct modifier is present
    /// 是否包含 Distinct 修饰符
    pub distinct: bool,
    /// LIMIT clause (from First/Top modifiers)
    /// 限制子句（来自 First/Top 修饰符）
    pub first_top: Option<u32>,
}

/// Query prefix enum
/// 查询前缀枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryPrefix
{
    /// find, find...By
    Find,
    /// read, read...By
    Read,
    /// query, query...By
    Query,
    /// get, get...By
    Get,
    /// count, count...By
    Count,
    /// delete, remove...By
    Delete,
    /// exists, exists...By
    Exists,
    /// save (for save operations)
    Save,
}

/// ORDER BY clause
/// 排序子句
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderByClause
{
    /// Field name to sort by
    /// 排序字段名
    pub field: String,
    /// Sort direction
    /// 排序方向
    pub direction: OrderDirection,
}

/// Sort direction
/// 排序方向
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
            OrderDirection::Asc => write!(f, "ASC"),
            OrderDirection::Desc => write!(f, "DESC"),
        }
    }
}

/// Main entry point for method name parsing
/// 方法名解析的主入口点
///
/// # Example / 示例
///
/// ```rust,no_run,ignore
/// use hiver_data_commons::MethodName;
///
/// let parsed = MethodName::parse("findByNameAndAgeGreaterThanOrderByCreatedAtDesc")
///     .expect("valid method name");
///
/// assert_eq!(parsed.prefix, QueryPrefix::Find);
/// assert_eq!(parsed.conditions.len(), 2);
/// assert_eq!(parsed.order_by.len(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct MethodName;

impl MethodName
{
    /// Parse a method name into query conditions
    /// 将方法名解析为查询条件
    ///
    /// # Parameters / 参数
    ///
    /// - `name`: The method name to parse / 要解析的方法名
    ///
    /// # Returns / 返回
    ///
    /// - `Ok(ParsedMethodName)` on success
    /// - `Err(String)` with error description on parse failure
    pub fn parse(name: &str) -> Result<ParsedMethodName, String>
    {
        let name = name.trim();

        if name.is_empty()
        {
            return Err("Empty method name".to_string());
        }

        // Step 1: Extract prefix
        let (prefix, remaining) = Self::extract_prefix(name)?;

        // Step 2: Extract First/Top limit modifier
        let (first_top, remaining) = Self::extract_first_top(remaining);

        // Step 3: Extract Distinct modifier
        let (distinct, remaining) = Self::extract_distinct(remaining);

        // Step 4: Extract "By" keyword
        let remaining = Self::extract_by_keyword(remaining)?;

        if remaining.is_empty()
        {
            return Err(format!("Method name '{}' has no conditions after 'By'", name));
        }

        // Step 5: Split at OrderBy clause
        let (condition_part, order_by) = Self::split_order_by(remaining);

        // Step 6: Parse conditions
        let conditions = Self::parse_conditions(condition_part)?;

        Ok(ParsedMethodName {
            prefix,
            conditions,
            order_by,
            distinct,
            first_top,
        })
    }

    /// Extract the query prefix from the beginning of the method name
    fn extract_prefix(name: &str) -> Result<(QueryPrefix, &str), String>
    {
        let prefixes: &[(&str, QueryPrefix, usize)] = &[
            ("find", QueryPrefix::Find, 4),
            ("read", QueryPrefix::Read, 4),
            ("query", QueryPrefix::Query, 5),
            ("get", QueryPrefix::Get, 3),
            ("count", QueryPrefix::Count, 5),
            ("delete", QueryPrefix::Delete, 6),
            ("remove", QueryPrefix::Delete, 6),
            ("exists", QueryPrefix::Exists, 6),
            ("save", QueryPrefix::Save, 4),
        ];

        let name_lower = name.to_lowercase();

        for &(prefix_str, prefix, len) in prefixes
        {
            if name_lower.starts_with(prefix_str)
            {
                // Verify this is a complete word boundary
                // Prefix must be followed by uppercase, "By", "First", "Top", "Distinct", or end
                let after = &name[len..];
                if after.is_empty()
                    || after.starts_with(|c: char| c.is_uppercase())
                    || after.to_lowercase().starts_with("by")
                    || after.to_lowercase().starts_with("first")
                    || after.to_lowercase().starts_with("top")
                    || after.to_lowercase().starts_with("distinct")
                {
                    return Ok((prefix, after));
                }
            }
        }

        Err(format!(
            "Method name '{}' must start with a valid query prefix (find, read, query, get, \
             count, delete, exists)",
            name
        ))
    }

    /// Extract First/Top limit modifier
    fn extract_first_top(name: &str) -> (Option<u32>, &str)
    {
        let name_lower = name.to_lowercase();

        // Try "First<number>" pattern
        if let Some(rest) = name_lower.strip_prefix("first")
        {
            let (num_str, _remaining) = Self::take_digits(rest);
            if let Ok(num) = num_str.parse::<u32>()
            {
                return (Some(num), &name[5 + num_str.len()..]);
            }
            // Just "First" without number = First1
            return (Some(1), &name[5..]);
        }

        // Try "Top<number>" pattern
        if let Some(rest) = name_lower.strip_prefix("top")
        {
            let (num_str, _remaining) = Self::take_digits(rest);
            if let Ok(num) = num_str.parse::<u32>()
            {
                return (Some(num), &name[3 + num_str.len()..]);
            }
            // Just "Top" without number = Top1
            return (Some(1), &name[3..]);
        }

        (None, name)
    }

    /// Extract Distinct modifier
    fn extract_distinct(name: &str) -> (bool, &str)
    {
        let name_lower = name.to_lowercase();
        if name_lower.starts_with("distinct")
        {
            (true, &name[8..])
        }
        else
        {
            (false, name)
        }
    }

    /// Extract and strip the "By" keyword
    fn extract_by_keyword(name: &str) -> Result<&str, String>
    {
        let name_lower = name.to_lowercase();
        if name_lower.starts_with("by")
        {
            Ok(&name[2..])
        }
        else
        {
            Err(format!(
                "Method name must contain 'By' keyword before conditions, got: '{}'",
                name
            ))
        }
    }

    /// Split the method name at OrderBy clause
    fn split_order_by(name: &str) -> (&str, Vec<OrderByClause>)
    {
        // Find the last "OrderBy" occurrence
        let name_lower = name.to_lowercase();
        if let Some(pos) = name_lower.rfind("orderby")
        {
            let condition_part = &name[..pos];
            let order_part = &name[pos + 7..]; // skip "OrderBy"

            // Parse order by fields
            let orders = Self::parse_order_fields(order_part);
            (condition_part, orders)
        }
        else
        {
            (name, Vec::new())
        }
    }

    /// Parse ORDER BY fields
    fn parse_order_fields(order_part: &str) -> Vec<OrderByClause>
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
                // Check if this is "Asc" or "Desc"
                let rest: String = std::iter::once(c).chain(chars.clone()).collect();
                let rest_lower = rest.to_lowercase();

                if rest_lower.starts_with("asc")
                {
                    let field = current.clone();
                    current.clear();
                    // Field name: convert first char to lowercase
                    let field_name = Self::first_char_to_lower(&field);
                    orders.push(OrderByClause {
                        field: field_name,
                        direction: OrderDirection::Asc,
                    });
                    // Skip "Asc" characters (c already consumed 'A')
                    for _ in 0..2
                    {
                        chars.next();
                    }
                    continue;
                }

                if rest_lower.starts_with("desc")
                {
                    let field = current.clone();
                    current.clear();
                    let field_name = Self::first_char_to_lower(&field);
                    orders.push(OrderByClause {
                        field: field_name,
                        direction: OrderDirection::Desc,
                    });
                    // Skip "Desc" characters (c already consumed 'D')
                    for _ in 0..3
                    {
                        chars.next();
                    }
                    continue;
                }
            }
            current.push(c);
        }

        // Last field (default Asc)
        if !current.is_empty()
        {
            let field_name = Self::first_char_to_lower(&current);
            orders.push(OrderByClause {
                field: field_name,
                direction: OrderDirection::Asc,
            });
        }

        orders
    }

    /// Parse conditions from the condition part of the method name
    fn parse_conditions(condition_part: &str)
    -> Result<Vec<(Condition, Option<Connector>)>, String>
    {
        if condition_part.is_empty()
        {
            return Ok(Vec::new());
        }

        let mut conditions = Vec::new();
        let mut current_field = String::new();
        let mut current_connector: Option<Connector> = None;
        let mut chars = condition_part.chars().peekable();

        while let Some(c) = chars.next()
        {
            if c.is_uppercase() && !current_field.is_empty()
            {
                // Check if current_field is itself a connector (Or/And)
                let current_lower = current_field.to_lowercase();
                if current_lower == "and" || current_lower == "or"
                {
                    // Current field is a connector — consume it and continue with new field
                    current_connector = if current_lower == "and"
                    {
                        Some(Connector::And)
                    }
                    else
                    {
                        Some(Connector::Or)
                    };
                    current_field.clear();
                    // Don't push c yet — continue to process the new uppercase char
                    // We fall through to the rest of the loop body
                }

                // Uppercase marks a potential keyword boundary
                let rest: String = std::iter::once(c).chain(chars.clone()).collect();
                let rest_lower = rest.to_lowercase();

                // Check for logical connectors: And, Or in the rest (NOT when current_field was the
                // connector)
                if rest_lower.starts_with("and") && rest.len() > 3
                {
                    let next_char = rest.chars().nth(3).unwrap_or('\0');
                    if next_char.is_uppercase() || next_char == '\0'
                    {
                        // Save current field as Equals condition
                        let field = current_field.clone();
                        current_field.clear();
                        let field_name = Self::first_char_to_lower(&field);
                        conditions
                            .push((Condition::Equals { field: field_name }, current_connector));
                        current_connector = Some(Connector::And);
                        // Skip "And" (2 chars, 'A' already consumed)
                        for _ in 0..2
                        {
                            chars.next();
                        }
                        continue;
                    }
                }

                if rest_lower.starts_with("or") && rest.len() > 2
                {
                    let next_char = rest.chars().nth(2).unwrap_or('\0');
                    if next_char.is_uppercase() || next_char == '\0'
                    {
                        let field = current_field.clone();
                        current_field.clear();
                        let field_name = Self::first_char_to_lower(&field);
                        conditions
                            .push((Condition::Equals { field: field_name }, current_connector));
                        current_connector = Some(Connector::Or);
                        // Skip "Or" (1 chars, 'O' already consumed)
                        for _ in 0..1
                        {
                            chars.next();
                        }
                        continue;
                    }
                }

                // Check for condition keywords
                let (condition_type, skip_len) = Self::detect_condition_keyword(&rest_lower);
                if let Some(ct) = condition_type
                {
                    let field = current_field.clone();
                    current_field.clear();
                    let field_name = Self::first_char_to_lower(&field);
                    let cond = Self::build_condition(ct, &field_name);
                    conditions.push((cond, current_connector));
                    current_connector = None;
                    // Skip the keyword characters (minus 1, first char already consumed)
                    for _ in 0..(skip_len - 1)
                    {
                        chars.next();
                    }
                    continue;
                }
            }

            current_field.push(c);
        }

        // Last field (defaults to Equals)
        if !current_field.is_empty()
        {
            let field_name = Self::first_char_to_lower(&current_field);
            conditions.push((Condition::Equals { field: field_name }, current_connector));
        }

        Ok(conditions)
    }

    /// Detect a condition keyword from the start of a string
    fn detect_condition_keyword(word: &str) -> (Option<ConditionType>, usize)
    {
        let keywords: &[(&str, ConditionType, usize)] = &[
            // Longer patterns first to avoid partial matches
            ("isnotnull", ConditionType::IsNotNull, 9),
            ("isnull", ConditionType::IsNull, 6),
            ("notnull", ConditionType::IsNotNull, 7),
            ("startingwith", ConditionType::StartingWith, 12),
            ("endingwith", ConditionType::EndingWith, 10),
            ("containing", ConditionType::Containing, 10),
            ("greaterthan", ConditionType::GreaterThan, 11),
            ("greaterthanequal", ConditionType::GreaterThanEqual, 15),
            ("lessthan", ConditionType::LessThan, 8),
            ("lessthanequal", ConditionType::LessThanEqual, 12),
            ("notlike", ConditionType::NotLike, 7),
            ("between", ConditionType::Between, 7),
            ("before", ConditionType::Before, 6),
            ("after", ConditionType::After, 5),
            ("like", ConditionType::Like, 4),
            ("notin", ConditionType::NotIn, 5),
            ("true", ConditionType::True, 4),
            ("false", ConditionType::False, 5),
            ("not", ConditionType::NotEquals, 3),
            ("in", ConditionType::In, 2),
            ("is", ConditionType::Equals, 2),
        ];

        for (keyword, cond_type, len) in keywords
        {
            if word.starts_with(keyword)
            {
                // Verify it's a complete word boundary
                if word.len() > *len
                {
                    let next = word.chars().nth(*len).unwrap();
                    // Accept: uppercase start of next field, or continuation as And/Or connector
                    if next.is_uppercase()
                        || word[*len..].to_lowercase().starts_with("and")
                        || word[*len..].to_lowercase().starts_with("or")
                    {
                        return (Some(*cond_type), *len);
                    }
                }
                else
                {
                    return (Some(*cond_type), *len);
                }
            }
        }

        (None, 0)
    }

    /// Build a Condition from a condition type and field name
    fn build_condition(cond_type: ConditionType, field: &str) -> Condition
    {
        match cond_type
        {
            ConditionType::Equals => Condition::Equals {
                field: field.to_string(),
            },
            ConditionType::Like => Condition::Like {
                field: field.to_string(),
            },
            ConditionType::NotLike => Condition::NotLike {
                field: field.to_string(),
            },
            ConditionType::StartingWith => Condition::StartingWith {
                field: field.to_string(),
            },
            ConditionType::EndingWith => Condition::EndingWith {
                field: field.to_string(),
            },
            ConditionType::Containing => Condition::Containing {
                field: field.to_string(),
            },
            ConditionType::NotEquals => Condition::NotEquals {
                field: field.to_string(),
            },
            ConditionType::GreaterThan => Condition::GreaterThan {
                field: field.to_string(),
            },
            ConditionType::GreaterThanEqual => Condition::GreaterThan {
                field: field.to_string(),
            },
            ConditionType::LessThan => Condition::LessThan {
                field: field.to_string(),
            },
            ConditionType::LessThanEqual => Condition::LessThan {
                field: field.to_string(),
            },
            ConditionType::Between => Condition::Between {
                field: field.to_string(),
            },
            ConditionType::In => Condition::In {
                field: field.to_string(),
            },
            ConditionType::NotIn => Condition::NotIn {
                field: field.to_string(),
            },
            ConditionType::IsNull => Condition::IsNull {
                field: field.to_string(),
            },
            ConditionType::IsNotNull => Condition::IsNotNull {
                field: field.to_string(),
            },
            ConditionType::True => Condition::True {
                field: field.to_string(),
            },
            ConditionType::False => Condition::False {
                field: field.to_string(),
            },
            ConditionType::Before => Condition::Before {
                field: field.to_string(),
            },
            ConditionType::After => Condition::After {
                field: field.to_string(),
            },
        }
    }

    /// Take leading digits from a string
    fn take_digits(s: &str) -> (&str, &str)
    {
        let end = s.chars().take_while(|c| c.is_ascii_digit()).count();
        (&s[..end], &s[end..])
    }

    /// Convert the first character to lowercase
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
}

/// Internal condition type enum for keyword matching
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConditionType
{
    Equals,
    Like,
    NotLike,
    StartingWith,
    EndingWith,
    Containing,
    NotEquals,
    GreaterThan,
    GreaterThanEqual,
    LessThan,
    LessThanEqual,
    Between,
    In,
    NotIn,
    IsNull,
    IsNotNull,
    True,
    False,
    Before,
    After,
}

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests
{
    use super::*;

    /// Helper: parse and check no error
    fn parse(name: &str) -> ParsedMethodName
    {
        MethodName::parse(name).unwrap_or_else(|e| {
            panic!("Failed to parse '{}': {}", name, e);
        })
    }

    #[test]
    fn test_simple_find_by()
    {
        let result = parse("findByName");
        assert_eq!(result.prefix, QueryPrefix::Find);
        assert_eq!(result.conditions.len(), 1);
        assert_eq!(result.conditions[0].0, Condition::Equals {
            field: "name".into()
        });
    }

    #[test]
    fn test_find_by_and()
    {
        let result = parse("findByNameAndAge");
        assert_eq!(result.conditions.len(), 2);
        assert!(matches!(result.conditions[0].0, Condition::Equals { .. }));
        assert_eq!(result.conditions[0].1, None);
        assert_eq!(result.conditions[1].1, Some(Connector::And));
    }

    #[test]
    fn test_find_by_or()
    {
        let result = parse("findByNameOrEmail");
        assert_eq!(result.conditions.len(), 2);
        assert_eq!(result.conditions[1].1, Some(Connector::Or));
    }

    #[test]
    fn test_condition_keywords()
    {
        let result = parse("findByNameLike");
        assert!(matches!(result.conditions[0].0, Condition::Like { .. }));

        let result = parse("findByAgeGreaterThan");
        assert!(matches!(result.conditions[0].0, Condition::GreaterThan { .. }));

        let result = parse("findByAgeLessThan");
        assert!(matches!(result.conditions[0].0, Condition::LessThan { .. }));

        let result = parse("findByNameStartingWith");
        assert!(matches!(result.conditions[0].0, Condition::StartingWith { .. }));

        let result = parse("findByNameContaining");
        assert!(matches!(result.conditions[0].0, Condition::Containing { .. }));

        let result = parse("findByDeletedIsNull");
        assert!(matches!(result.conditions[0].0, Condition::IsNull { .. }));

        let result = parse("findByDeletedIsNotNull");
        assert!(matches!(result.conditions[0].0, Condition::IsNotNull { .. }));

        let result = parse("findByActiveTrue");
        assert!(matches!(result.conditions[0].0, Condition::True { .. }));

        let result = parse("findByActiveFalse");
        assert!(matches!(result.conditions[0].0, Condition::False { .. }));
    }

    #[test]
    fn test_order_by()
    {
        let result = parse("findByNameOrderByAgeAsc");
        assert_eq!(result.order_by.len(), 1);
        assert_eq!(result.order_by[0].field, "age");
        assert_eq!(result.order_by[0].direction, OrderDirection::Asc);

        let result = parse("findByNameOrderByAgeDesc");
        assert_eq!(result.order_by[0].direction, OrderDirection::Desc);

        let result = parse("findByNameOrderByAgeAscNameDesc");
        assert_eq!(result.order_by.len(), 2);
        assert_eq!(result.order_by[0].field, "age");
        assert_eq!(result.order_by[0].direction, OrderDirection::Asc);
        assert_eq!(result.order_by[1].field, "name");
        assert_eq!(result.order_by[1].direction, OrderDirection::Desc);
    }

    #[test]
    fn test_first_top()
    {
        let result = parse("findFirst10ByName");
        assert_eq!(result.first_top, Some(10));

        let result = parse("findTop5ByName");
        assert_eq!(result.first_top, Some(5));

        let result = parse("findFirstByName");
        assert_eq!(result.first_top, Some(1));
    }

    #[test]
    fn test_distinct()
    {
        let result = parse("findDistinctByName");
        assert!(result.distinct);
    }

    #[test]
    fn test_query_prefixes()
    {
        assert_eq!(parse("readByName").prefix, QueryPrefix::Read);
        assert_eq!(parse("queryByName").prefix, QueryPrefix::Query);
        assert_eq!(parse("getByName").prefix, QueryPrefix::Get);
        assert_eq!(parse("countByName").prefix, QueryPrefix::Count);
        assert_eq!(parse("deleteByName").prefix, QueryPrefix::Delete);
        assert_eq!(parse("existsByName").prefix, QueryPrefix::Exists);
    }

    #[test]
    fn test_complex_query()
    {
        let result = parse("findByNameAndAgeGreaterThanOrEmailLikeOrderByCreatedAtDesc");
        assert_eq!(result.prefix, QueryPrefix::Find);
        assert_eq!(result.conditions.len(), 3);
        assert!(matches!(result.conditions[0].0, Condition::Equals { .. }));
        assert_eq!(result.conditions[0].1, None);
        assert!(matches!(result.conditions[1].0, Condition::GreaterThan { .. }));
        assert_eq!(result.conditions[1].1, Some(Connector::And));
        assert!(matches!(result.conditions[2].0, Condition::Like { .. }));
        assert_eq!(result.conditions[2].1, Some(Connector::Or));
        assert_eq!(result.order_by.len(), 1);
    }

    #[test]
    fn test_invalid_method_names()
    {
        assert!(MethodName::parse("").is_err());
        assert!(MethodName::parse("doSomething").is_err());
        assert!(MethodName::parse("find").is_err()); // No By clause
        assert!(MethodName::parse("findBy").is_err()); // No conditions after By
    }
}
