//! Cron expression parsing and next-fire-time calculation.
//! Cron表达式解析和下次触发时间计算。

use std::fmt;

use chrono::{DateTime, Datelike, Timelike, Utc};

use super::field::{CronField, FieldError, FieldKind};

/// Error type for cron expression parsing.
/// Cron表达式解析错误类型。
#[derive(Debug, Clone)]
pub enum CronError
{
    /// Wrong number of whitespace-separated fields.
    /// 空格分隔的字段数量错误。
    InvalidFieldCount(usize),
    /// Error in a specific field (0-indexed).
    /// 特定字段的错误（从0开始索引）。
    FieldError(usize, FieldError),
}

impl fmt::Display for CronError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::InvalidFieldCount(n) =>
            {
                write!(f, "expected 5, 6, or 7 fields, got {}", n)
            }
            Self::FieldError(idx, e) =>
            {
                write!(f, "field {}: {}", idx, e)
            }
        }
    }
}

impl std::error::Error for CronError {}

/// A parsed cron expression with next-fire-time calculation.
/// 解析后的cron表达式，支持下次触发时间计算。
///
/// # Supported formats / 支持的格式
///
/// | Fields | Layout |
/// |--------|--------|
/// | 5 | `minute hour day month weekday` |
/// | 6 | `second minute hour day month weekday` |
/// | 7 | `second minute hour day month weekday year` |
///
/// # Spring Equivalent / Spring等价物
///
/// ```java
/// @Scheduled(cron = "0 0 9-17 * * MON-FRI")
/// public void workHoursTask() { }
/// ```
///
/// # Rust Advantage / Rust优势
///
/// Can be validated at compile time via proc-macro — Spring can only detect
/// bad cron expressions at runtime.
/// 可通过过程宏在编译时验证 — Spring 只能在运行时检测错误的cron表达式。
#[derive(Debug, Clone)]
pub struct CronExpression
{
    seconds: CronField,
    minutes: CronField,
    hours: CronField,
    days_of_month: CronField,
    months: CronField,
    days_of_week: CronField,
    years: Option<CronField>,
    source: String,
}

impl CronExpression
{
    /// Parse a cron expression string.
    /// 解析cron表达式字符串。
    ///
    /// Automatically detects 5, 6, or 7 field format.
    /// 自动检测5、6或7字段格式。
    pub fn parse(expr: &str) -> Result<Self, CronError>
    {
        let fields: Vec<&str> = expr.split_whitespace().collect();

        let parsed = match fields.len()
        {
            5 => Self::parse_5(&fields)?,
            6 => Self::parse_6(&fields)?,
            7 => Self::parse_7(&fields)?,
            n => return Err(CronError::InvalidFieldCount(n)),
        };

        Ok(Self {
            source: expr.to_string(),
            ..parsed
        })
    }

    fn parse_5(f: &[&str]) -> Result<Self, CronError>
    {
        // minute hour day month weekday  →  seconds default to 0
        Ok(Self {
            seconds: pf("0", FieldKind::Second, 0)?,
            minutes: pf(f[0], FieldKind::Minute, 1)?,
            hours: pf(f[1], FieldKind::Hour, 2)?,
            days_of_month: pf(f[2], FieldKind::DayOfMonth, 3)?,
            months: pf(f[3], FieldKind::Month, 4)?,
            days_of_week: pf(f[4], FieldKind::DayOfWeek, 5)?,
            years: None,
            source: String::new(),
        })
    }

    fn parse_6(f: &[&str]) -> Result<Self, CronError>
    {
        Ok(Self {
            seconds: pf(f[0], FieldKind::Second, 0)?,
            minutes: pf(f[1], FieldKind::Minute, 1)?,
            hours: pf(f[2], FieldKind::Hour, 2)?,
            days_of_month: pf(f[3], FieldKind::DayOfMonth, 3)?,
            months: pf(f[4], FieldKind::Month, 4)?,
            days_of_week: pf(f[5], FieldKind::DayOfWeek, 5)?,
            years: None,
            source: String::new(),
        })
    }

    fn parse_7(f: &[&str]) -> Result<Self, CronError>
    {
        Ok(Self {
            seconds: pf(f[0], FieldKind::Second, 0)?,
            minutes: pf(f[1], FieldKind::Minute, 1)?,
            hours: pf(f[2], FieldKind::Hour, 2)?,
            days_of_month: pf(f[3], FieldKind::DayOfMonth, 3)?,
            months: pf(f[4], FieldKind::Month, 4)?,
            days_of_week: pf(f[5], FieldKind::DayOfWeek, 5)?,
            years: Some(pf(f[6], FieldKind::Year, 6)?),
            source: String::new(),
        })
    }

    /// The original cron expression string.
    /// 原始cron表达式字符串。
    pub fn source(&self) -> &str
    {
        &self.source
    }

    // ── matching ─────────────────────────────────────────────────────

    /// Check if a datetime matches this cron expression.
    /// 检查日期时间是否匹配此cron表达式。
    pub fn matches(&self, dt: &DateTime<Utc>) -> bool
    {
        if !self.seconds.matches(dt.second())
        {
            return false;
        }
        if !self.minutes.matches(dt.minute())
        {
            return false;
        }
        if !self.hours.matches(dt.hour())
        {
            return false;
        }
        if !self.months.matches(dt.month())
        {
            return false;
        }
        if let Some(ref y) = self.years
        {
            if !y.matches(dt.year() as u32)
            {
                return false;
            }
        }

        // Day matching: when both DOM and DOW are restricted, the cron standard
        // uses union (either match is OK). When only one is restricted it must match.
        // 日期匹配：当 DOM 和 DOW 都受限制时，cron 标准使用并集（任一匹配即可）。
        // 当只有一个受限制时，它必须匹配。
        let dom_ok = self.days_of_month.matches(dt.day());
        let dow = dt.weekday().num_days_from_sunday(); // 0 = Sun
        let dow_ok = self.days_of_week.matches(dow);

        if !self.days_of_month.is_any() && !self.days_of_week.is_any()
        {
            dom_ok || dow_ok
        }
        else
        {
            dom_ok && dow_ok
        }
    }

    // ── next-fire-time ───────────────────────────────────────────────

    /// Calculate the next fire time strictly after `from`.
    /// 计算严格在 `from` 之后的下次触发时间。
    ///
    /// Returns `None` if no matching time is found within ~4 years.
    /// 如果在约4年内找不到匹配时间，则返回 `None`。
    pub fn next_after(&self, from: &DateTime<Utc>) -> Option<DateTime<Utc>>
    {
        // Start one second after `from`, aligned to whole seconds.
        let mut c = from.with_nanosecond(0).unwrap_or(*from)
            + chrono::Duration::seconds(1);

        // Safety limit: ~4 years of minute-level iterations.
        let max_iters = 4 * 366 * 24 * 60;

        for _ in 0..max_iters
        {
            // ── Year ──────────────────────────────────────────────
            if let Some(ref yf) = self.years
            {
                if let Some(y) = yf.next_value(c.year() as u32)
                {
                    if y as i32 != c.year()
                    {
                        c = reset_to_year(c, y as i32);
                        continue;
                    }
                }
                else
                {
                    return None;
                }
            }

            // ── Month ─────────────────────────────────────────────
            if let Some(m) = self.months.next_value(c.month())
            {
                if m != c.month()
                {
                    c = reset_to_month(c, m);
                    continue;
                }
            }
            else
            {
                c = reset_to_year(c, c.year() + 1);
                continue;
            }

            // ── Day ───────────────────────────────────────────────
            let dim = days_in_month(c.year(), c.month());
            if let Some(d) = self.next_day(c.year(), c.month(), c.day(), dim)
            {
                if d != c.day()
                {
                    c = reset_to_day(c, d);
                    continue;
                }
            }
            else
            {
                // Roll to next month
                if c.month() == 12
                {
                    c = reset_to_year(c, c.year() + 1);
                }
                else
                {
                    c = reset_to_month(c, c.month() + 1);
                }
                continue;
            }

            // ── Hour ──────────────────────────────────────────────
            if let Some(h) = self.hours.next_value(c.hour())
            {
                if h != c.hour()
                {
                    c = reset_to_hour(c, h);
                    continue;
                }
            }
            else
            {
                c = reset_to_day(c, c.day() + 1);
                continue;
            }

            // ── Minute ────────────────────────────────────────────
            if let Some(m) = self.minutes.next_value(c.minute())
            {
                if m != c.minute()
                {
                    c = reset_to_minute(c, m);
                    continue;
                }
            }
            else
            {
                c = reset_to_hour(c, c.hour() + 1);
                continue;
            }

            // ── Second ────────────────────────────────────────────
            if let Some(s) = self.seconds.next_value(c.second())
            {
                if s != c.second()
                {
                    c = c.with_second(s).unwrap_or(c);
                    continue;
                }
            }
            else
            {
                c = reset_to_minute(c, c.minute() + 1);
                continue;
            }

            // All fields matched.
            return Some(c);
        }

        None
    }

    /// Find the next matching day-of-month, respecting the DOM/DOW union rule.
    /// 查找下一个匹配的日期，遵循 DOM/DOW 并集规则。
    fn next_day(&self, year: i32, month: u32, from_day: u32, dim: u32) -> Option<u32>
    {
        let dom_any = self.days_of_month.is_any();
        let dow_any = self.days_of_week.is_any();

        for d in from_day..=dim
        {
            let dom_ok = self.days_of_month.matches(d);

            let dow = {
                let dt = chrono::NaiveDate::from_ymd_opt(year, month, d)?;
                dt.weekday().num_days_from_sunday()
            };
            let dow_ok = self.days_of_week.matches(dow);

            let matched = if !dom_any && !dow_any
            {
                dom_ok || dow_ok
            }
            else
            {
                dom_ok && dow_ok
            };

            if matched
            {
                return Some(d);
            }
        }
        None
    }
}

// ── Reset helpers ────────────────────────────────────────────────────────
// Reset to the start of a larger unit when a smaller unit rolls over.
// 当较小单位溢出时，重置到较大单位的起始。

fn reset_to_year(c: DateTime<Utc>, year: i32) -> DateTime<Utc>
{
    c.with_year(year)
        .unwrap_or(c)
        .with_month(1)
        .unwrap()
        .with_day(1)
        .unwrap()
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
}

fn reset_to_month(c: DateTime<Utc>, month: u32) -> DateTime<Utc>
{
    if month <= 12
    {
        c.with_month(month)
            .unwrap_or(c)
            .with_day(1)
            .unwrap()
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
    }
    else
    {
        c
    }
}

fn reset_to_day(c: DateTime<Utc>, day: u32) -> DateTime<Utc>
{
    c.with_day(day)
        .unwrap_or(c)
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
}

fn reset_to_hour(c: DateTime<Utc>, hour: u32) -> DateTime<Utc>
{
    c.with_hour(hour)
        .unwrap_or(c)
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap()
}

fn reset_to_minute(c: DateTime<Utc>, minute: u32) -> DateTime<Utc>
{
    c.with_minute(minute)
        .unwrap_or(c)
        .with_second(0)
        .unwrap()
}

/// Days in the given month (handles leap years).
/// 给定月份的天数（处理闰年）。
fn days_in_month(year: i32, month: u32) -> u32
{
    match month
    {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 =>
        {
            if is_leap_year(year)
            {
                29
            }
            else
            {
                28
            }
        }
        _ => 31,
    }
}

fn is_leap_year(year: i32) -> bool
{
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Shorthand for parsing a field.
fn pf(expr: &str, kind: FieldKind, idx: usize) -> Result<CronField, CronError>
{
    CronField::parse(expr, kind).map_err(|e| CronError::FieldError(idx, e))
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests
{
    use super::*;
    use chrono::TimeZone;

    fn dt(y: i32, m: u32, d: u32, h: u32, mi: u32, s: u32) -> DateTime<Utc>
    {
        Utc.with_ymd_and_hms(y, m, d, h, mi, s).unwrap()
    }

    // ── parse ───────────────────────────────────────────────────────

    #[test]
    fn test_parse_5_field()
    {
        let expr = CronExpression::parse("0 * * * *").unwrap();
        assert_eq!(expr.source(), "0 * * * *");
    }

    #[test]
    fn test_parse_6_field()
    {
        let expr = CronExpression::parse("0 0 * * * *").unwrap();
        assert_eq!(expr.source(), "0 0 * * * *");
    }

    #[test]
    fn test_parse_7_field()
    {
        let expr = CronExpression::parse("0 0 0 * * * 2025").unwrap();
        assert_eq!(expr.source(), "0 0 0 * * * 2025");
    }

    #[test]
    fn test_parse_bad_field_count()
    {
        assert!(CronExpression::parse("0 0 0").is_err());
        assert!(CronExpression::parse("").is_err());
    }

    #[test]
    fn test_parse_bad_value()
    {
        assert!(CronExpression::parse("60 * * * *").is_err());
        assert!(CronExpression::parse("* 25 * * *").is_err());
    }

    // ── matches ─────────────────────────────────────────────────────

    #[test]
    fn test_every_minute()
    {
        let expr = CronExpression::parse("0 * * * * *").unwrap();
        assert!(expr.matches(&dt(2025, 1, 15, 10, 30, 0)));
        assert!(!expr.matches(&dt(2025, 1, 15, 10, 30, 1))); // second=1 not allowed
    }

    #[test]
    fn test_specific_hour()
    {
        // At second 0 of every minute of hour 9
        let expr = CronExpression::parse("0 * 9 * * *").unwrap();
        assert!(expr.matches(&dt(2025, 6, 1, 9, 0, 0)));
        assert!(expr.matches(&dt(2025, 6, 1, 9, 30, 0)));
        assert!(!expr.matches(&dt(2025, 6, 1, 10, 0, 0)));
    }

    #[test]
    fn test_named_weekday()
    {
        // Every Monday at midnight
        // 2025-06-02 is a Monday
        let expr = CronExpression::parse("0 0 0 ? * MON").unwrap();
        assert!(expr.matches(&dt(2025, 6, 2, 0, 0, 0)));
        // 2025-06-03 is Tuesday
        assert!(!expr.matches(&dt(2025, 6, 3, 0, 0, 0)));
    }

    #[test]
    fn test_named_month()
    {
        let expr = CronExpression::parse("0 0 0 1 JAN ?").unwrap();
        assert!(expr.matches(&dt(2025, 1, 1, 0, 0, 0)));
        assert!(!expr.matches(&dt(2025, 2, 1, 0, 0, 0)));
    }

    // ── next_after ──────────────────────────────────────────────────

    #[test]
    fn test_next_every_minute()
    {
        let expr = CronExpression::parse("0 * * * * *").unwrap();
        let from = dt(2025, 1, 1, 0, 0, 30);
        let next = expr.next_after(&from).unwrap();
        assert_eq!(next, dt(2025, 1, 1, 0, 1, 0));
    }

    #[test]
    fn test_next_specific_second()
    {
        let expr = CronExpression::parse("30 * * * * *").unwrap();
        let from = dt(2025, 1, 1, 0, 0, 0);
        let next = expr.next_after(&from).unwrap();
        assert_eq!(next, dt(2025, 1, 1, 0, 0, 30));
    }

    #[test]
    fn test_next_quarter_hour()
    {
        // Every 15th minute
        let expr = CronExpression::parse("0 */15 * * * *").unwrap();
        let from = dt(2025, 1, 1, 0, 10, 0);
        let next = expr.next_after(&from).unwrap();
        assert_eq!(next, dt(2025, 1, 1, 0, 15, 0));
    }

    #[test]
    fn test_next_work_hours()
    {
        // Seconds 0, minutes 0, hours 9-17, any day
        let expr = CronExpression::parse("0 0 9-17 * * *").unwrap();
        let from = dt(2025, 6, 2, 8, 30, 0);
        let next = expr.next_after(&from).unwrap();
        assert_eq!(next, dt(2025, 6, 2, 9, 0, 0));
    }

    #[test]
    fn test_next_cross_day()
    {
        // Every day at 00:00:00
        let expr = CronExpression::parse("0 0 0 * * *").unwrap();
        let from = dt(2025, 6, 1, 23, 59, 59);
        let next = expr.next_after(&from).unwrap();
        assert_eq!(next, dt(2025, 6, 2, 0, 0, 0));
    }

    #[test]
    fn test_next_cross_month()
    {
        let expr = CronExpression::parse("0 0 0 1 * *").unwrap();
        let from = dt(2025, 6, 15, 0, 0, 0);
        let next = expr.next_after(&from).unwrap();
        assert_eq!(next, dt(2025, 7, 1, 0, 0, 0));
    }

    #[test]
    fn test_next_cross_year()
    {
        let expr = CronExpression::parse("0 0 0 1 JAN *").unwrap();
        let from = dt(2025, 6, 1, 0, 0, 0);
        let next = expr.next_after(&from).unwrap();
        assert_eq!(next, dt(2026, 1, 1, 0, 0, 0));
    }

    #[test]
    fn test_next_with_year_field()
    {
        let expr = CronExpression::parse("0 0 0 1 1 * 2026").unwrap();
        let from = dt(2025, 6, 1, 0, 0, 0);
        let next = expr.next_after(&from).unwrap();
        assert_eq!(next, dt(2026, 1, 1, 0, 0, 0));
    }

    #[test]
    fn test_next_february_29()
    {
        // Leap year: 2028 is a leap year
        let expr = CronExpression::parse("0 0 0 29 2 *").unwrap();
        let from = dt(2025, 1, 1, 0, 0, 0);
        let next = expr.next_after(&from).unwrap();
        assert_eq!(next, dt(2028, 2, 29, 0, 0, 0));
    }

    #[test]
    fn test_5_field_no_seconds()
    {
        // 5-field: minute=every, hour=9, rest=any
        let expr = CronExpression::parse("* 9 * * *").unwrap();
        let from = dt(2025, 6, 2, 8, 59, 0);
        let next = expr.next_after(&from).unwrap();
        // seconds default to 0, so first match is 09:00:00
        assert_eq!(next, dt(2025, 6, 2, 9, 0, 0));
    }

    #[test]
    fn test_next_chain()
    {
        // Verify consecutive next_after calls produce monotonically increasing times.
        let expr = CronExpression::parse("0 */5 * * * *").unwrap();
        let mut t = dt(2025, 1, 1, 0, 0, 0);
        for _ in 0..10
        {
            let next = expr.next_after(&t).unwrap();
            assert!(next > t, "next {} must be > current {}", next, t);
            t = next;
        }
    }
}
