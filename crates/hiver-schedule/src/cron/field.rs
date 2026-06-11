//! Cron field parsing — individual field resolution (second, minute, hour, etc.).
//! Cron字段解析 — 单个字段解析（秒、分、时等）。

use std::fmt;

/// Error type for cron field parsing.
/// Cron字段解析错误类型。
#[derive(Debug, Clone)]
pub enum FieldError
{
    /// Value out of range for this field kind.
    /// 值超出此字段类型的范围。
    InvalidValue(String),
    /// Invalid range expression (e.g. `5-1`).
    /// 无效的范围表达式（如 `5-1`）。
    InvalidRange(String),
    /// Invalid step expression (e.g. `*/0`).
    /// 无效的步长表达式（如 `*/0`）。
    InvalidStep(String),
    /// General parse error.
    /// 通用解析错误。
    InvalidExpression(String),
}

impl fmt::Display for FieldError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::InvalidValue(v) => write!(f, "invalid value: {}", v),
            Self::InvalidRange(r) => write!(f, "invalid range: {}", r),
            Self::InvalidStep(s) => write!(f, "invalid step: {}", s),
            Self::InvalidExpression(e) => write!(f, "invalid expression: {}", e),
        }
    }
}

impl std::error::Error for FieldError {}

// ── Field kind ──────────────────────────────────────────────────────────

/// The kind of a cron field, determining its valid value range.
/// Cron字段的类型，决定其有效值范围。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind
{
    /// Seconds field (0–59).
    /// 秒字段（0–59）。
    Second,
    /// Minutes field (0–59).
    /// 分钟字段（0–59）。
    Minute,
    /// Hours field (0–23).
    /// 小时字段（0–23）。
    Hour,
    /// Day-of-month field (1–31).
    /// 日期字段（1–31）。
    DayOfMonth,
    /// Month field (1–12, also JAN–DEC).
    /// 月份字段（1–12，也支持 JAN–DEC）。
    Month,
    /// Day-of-week field (0–6, 0 = Sunday, also SUN–SAT).
    /// 星期字段（0–6，0 = 周日，也支持 SUN–SAT）。
    DayOfWeek,
    /// Year field (1970–2099).
    /// 年份字段（1970–2099）。
    Year,
}

impl FieldKind
{
    /// Minimum allowed value for this kind.
    /// 此类型的最小允许值。
    pub fn min(self) -> u32
    {
        match self
        {
            Self::Second | Self::Minute => 0,
            Self::Hour => 0,
            Self::DayOfMonth => 1,
            Self::Month => 1,
            Self::DayOfWeek => 0,
            Self::Year => 1970,
        }
    }

    /// Maximum allowed value for this kind.
    /// 此类型的最大允许值。
    pub fn max(self) -> u32
    {
        match self
        {
            Self::Second | Self::Minute => 59,
            Self::Hour => 23,
            Self::DayOfMonth => 31,
            Self::Month => 12,
            Self::DayOfWeek => 6,
            Self::Year => 2099,
        }
    }
}

// ── CronField ───────────────────────────────────────────────────────────

/// A parsed cron field holding the set of allowed values.
/// 解析后的cron字段，保存允许值的集合。
#[derive(Debug, Clone)]
pub struct CronField
{
    /// Sorted list of distinct allowed values.
    /// 允许值的排序去重列表。
    allowed: Vec<u32>,

    /// `true` when every value in the range is allowed (i.e. `*` or `?`).
    /// 当范围内的所有值都被允许时为 `true`（即 `*` 或 `?`）。
    is_any: bool,

    /// The field kind.
    /// 字段类型。
    kind: FieldKind,
}

impl CronField
{
    /// Parse a cron field expression for the given kind.
    /// 解析给定类型的cron字段表达式。
    ///
    /// # Supported syntax / 支持的语法
    ///
    /// | Syntax | Meaning |
    /// |--------|---------|
    /// | `*` / `?` | any value |
    /// | `5` | exact value |
    /// | `1-5` | range |
    /// | `*/3` | every Nth value |
    /// | `1-10/2` | range with step |
    /// | `1,3,5` | list |
    /// | `MON`, `JAN` | named weekday / month |
    pub fn parse(expr: &str, kind: FieldKind) -> Result<Self, FieldError>
    {
        let expr = expr.trim();

        if expr == "*" || expr == "?"
        {
            return Ok(Self {
                allowed: (kind.min()..=kind.max()).collect(),
                is_any: true,
                kind,
            });
        }

        let mut values = Vec::new();
        for part in expr.split(',')
        {
            values.extend(Self::parse_part(part.trim(), kind)?);
        }

        values.sort_unstable();
        values.dedup();

        // Validate range
        for &v in &values
        {
            if v < kind.min() || v > kind.max()
            {
                return Err(FieldError::InvalidValue(format!(
                    "{} is out of range [{}, {}] for {:?}",
                    v,
                    kind.min(),
                    kind.max(),
                    kind
                )));
            }
        }

        let is_any = values.len() == (kind.max() - kind.min() + 1) as usize
            && values.first() == Some(&kind.min())
            && values.last() == Some(&kind.max());

        Ok(Self {
            allowed: values,
            is_any,
            kind,
        })
    }

    // ── private helpers ──────────────────────────────────────────────

    fn parse_part(part: &str, kind: FieldKind) -> Result<Vec<u32>, FieldError>
    {
        // Step: */N, X-Y/N, X/N
        if let Some(slash_pos) = part.find('/')
        {
            let base = &part[..slash_pos];
            let step: u32 = part[slash_pos + 1..]
                .parse()
                .map_err(|_| FieldError::InvalidStep(part.to_string()))?;

            if step == 0
            {
                return Err(FieldError::InvalidStep(part.to_string()));
            }

            let (start, end) = if base == "*" || base == "?"
            {
                (kind.min(), kind.max())
            }
            else if let Some(dash_pos) = base.find('-')
            {
                let s = Self::resolve_value(&base[..dash_pos], kind)?;
                let e = Self::resolve_value(&base[dash_pos + 1..], kind)?;
                (s, e)
            }
            else
            {
                let s = Self::resolve_value(base, kind)?;
                (s, kind.max())
            };

            Ok((start..=end).step_by(step as usize).collect())
        }
        // Range: X-Y
        else if let Some(dash_pos) = part.find('-')
        {
            let s = Self::resolve_value(&part[..dash_pos], kind)?;
            let e = Self::resolve_value(&part[dash_pos + 1..], kind)?;
            Ok((s..=e).collect())
        }
        // Single value
        else
        {
            let v = Self::resolve_value(part, kind)?;
            Ok(vec![v])
        }
    }

    /// Resolve a single value, handling named months and weekdays.
    /// 解析单个值，处理命名的月份和星期。
    fn resolve_value(s: &str, kind: FieldKind) -> Result<u32, FieldError>
    {
        match kind
        {
            FieldKind::Month => match s.to_uppercase().as_str()
            {
                "JAN" => Ok(1),
                "FEB" => Ok(2),
                "MAR" => Ok(3),
                "APR" => Ok(4),
                "MAY" => Ok(5),
                "JUN" => Ok(6),
                "JUL" => Ok(7),
                "AUG" => Ok(8),
                "SEP" => Ok(9),
                "OCT" => Ok(10),
                "NOV" => Ok(11),
                "DEC" => Ok(12),
                _ => s
                    .parse()
                    .map_err(|_| FieldError::InvalidValue(s.to_string())),
            },
            FieldKind::DayOfWeek => match s.to_uppercase().as_str()
            {
                "SUN" => Ok(0),
                "MON" => Ok(1),
                "TUE" => Ok(2),
                "WED" => Ok(3),
                "THU" => Ok(4),
                "FRI" => Ok(5),
                "SAT" => Ok(6),
                _ => s
                    .parse()
                    .map_err(|_| FieldError::InvalidValue(s.to_string())),
            },
            _ => s
                .parse()
                .map_err(|_| FieldError::InvalidValue(s.to_string())),
        }
    }

    // ── public query methods ─────────────────────────────────────────

    /// Check if a value matches this field.
    /// 检查值是否匹配此字段。
    pub fn matches(&self, value: u32) -> bool
    {
        self.allowed.binary_search(&value).is_ok()
    }

    /// Get the smallest allowed value that is `>= from`.
    /// 获取 `>= from` 的最小允许值。
    pub fn next_value(&self, from: u32) -> Option<u32>
    {
        match self.allowed.binary_search(&from)
        {
            Ok(_) => Some(from),
            Err(idx) => self.allowed.get(idx).copied(),
        }
    }

    /// Get the smallest allowed value.
    /// 获取最小允许值。
    pub fn first(&self) -> u32
    {
        self.allowed.first().copied().unwrap_or(self.kind.min())
    }

    /// Whether this field matches every possible value.
    /// 此字段是否匹配所有可能的值。
    pub fn is_any(&self) -> bool
    {
        self.is_any
    }

    /// A reference to the sorted allowed values.
    /// 排序后允许值的引用。
    pub fn allowed(&self) -> &[u32]
    {
        &self.allowed
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_star_matches_everything()
    {
        let f = CronField::parse("*", FieldKind::Minute).unwrap();
        assert!(f.is_any());
        for v in 0..=59
        {
            assert!(f.matches(v));
        }
    }

    #[test]
    fn test_question_mark()
    {
        let f = CronField::parse("?", FieldKind::DayOfMonth).unwrap();
        assert!(f.is_any());
    }

    #[test]
    fn test_single_value()
    {
        let f = CronField::parse("5", FieldKind::Hour).unwrap();
        assert!(!f.is_any());
        assert!(f.matches(5));
        assert!(!f.matches(4));
        assert!(!f.matches(6));
    }

    #[test]
    fn test_range()
    {
        let f = CronField::parse("1-5", FieldKind::DayOfWeek).unwrap();
        for v in 1..=5
        {
            assert!(f.matches(v), "{} should match", v);
        }
        assert!(!f.matches(0));
        assert!(!f.matches(6));
    }

    #[test]
    fn test_step()
    {
        let f = CronField::parse("*/15", FieldKind::Minute).unwrap();
        assert!(f.matches(0));
        assert!(f.matches(15));
        assert!(f.matches(30));
        assert!(f.matches(45));
        assert!(!f.matches(5));
    }

    #[test]
    fn test_range_step()
    {
        let f = CronField::parse("0-30/10", FieldKind::Minute).unwrap();
        assert!(f.matches(0));
        assert!(f.matches(10));
        assert!(f.matches(20));
        assert!(f.matches(30));
        assert!(!f.matches(40));
    }

    #[test]
    fn test_list()
    {
        let f = CronField::parse("1,3,5", FieldKind::Hour).unwrap();
        assert!(f.matches(1));
        assert!(f.matches(3));
        assert!(f.matches(5));
        assert!(!f.matches(2));
        assert!(!f.matches(4));
    }

    #[test]
    fn test_named_month()
    {
        let f = CronField::parse("JAN-MAR", FieldKind::Month).unwrap();
        assert!(f.matches(1));
        assert!(f.matches(2));
        assert!(f.matches(3));
        assert!(!f.matches(4));
    }

    #[test]
    fn test_named_weekday()
    {
        let f = CronField::parse("MON-FRI", FieldKind::DayOfWeek).unwrap();
        assert!(!f.matches(0)); // SUN
        assert!(f.matches(1)); // MON
        assert!(f.matches(5)); // FRI
        assert!(!f.matches(6)); // SAT
    }

    #[test]
    fn test_next_value()
    {
        let f = CronField::parse("0,15,30,45", FieldKind::Minute).unwrap();
        assert_eq!(f.next_value(0), Some(0));
        assert_eq!(f.next_value(1), Some(15));
        assert_eq!(f.next_value(16), Some(30));
        assert_eq!(f.next_value(46), None);
    }

    #[test]
    fn test_out_of_range()
    {
        let result = CronField::parse("60", FieldKind::Second);
        assert!(result.is_err());
    }
}
