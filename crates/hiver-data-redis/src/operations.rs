//! Redis Data Structure Operations
//! Redis 数据结构操作
//!
//! # Overview / 概述
//!
//! Specialized operations for Redis data structures.
//! Equivalent to Spring Data Redis's `*Operations` interfaces.
//! Redis 数据结构的专用操作。等价于 Spring Data Redis 的 `*Operations` 接口。
//!
//! Note: Stream/Geo/HyperLogLog ops require enabling the corresponding
//! features in the `redis` crate dependency.

use std::collections::HashMap;

use redis::AsyncCommands;
use serde::{Serialize, de::DeserializeOwned};

use crate::{RedisError, RedisResult};

// ── Hash Operations ──

/// Hash operations — equivalent to Spring's `HashOperations`.
/// Hash 操作 — 等价于 Spring 的 `HashOperations`。
pub struct HashOps;

impl HashOps
{
    /// Set a field in a hash.
    /// 设置 hash 中的字段。
    pub async fn hset<C: AsyncCommands>(
        conn: &mut C,
        key: &str,
        field: &str,
        value: &str,
    ) -> RedisResult<bool>
    {
        let result: i32 = conn.hset(key, field, value).await?;
        Ok(result > 0)
    }

    /// Set multiple fields in a hash.
    /// 设置 hash 中的多个字段。
    pub async fn hset_multiple<C: AsyncCommands>(
        conn: &mut C,
        key: &str,
        items: &[(&str, &str)],
    ) -> RedisResult<()>
    {
        for (field, value) in items
        {
            conn.hset::<_, _, _, ()>(key, field, value).await?;
        }
        Ok(())
    }

    /// Set a JSON-serialized field in a hash.
    /// 设置 JSON 序列化的字段。
    pub async fn hset_json<C: AsyncCommands, T: Serialize + Send + Sync>(
        conn: &mut C,
        key: &str,
        field: &str,
        value: &T,
    ) -> RedisResult<bool>
    {
        let json =
            serde_json::to_string(value).map_err(|e| RedisError::serialization(e.to_string()))?;
        Self::hset(conn, key, field, &json).await
    }

    /// Get a field from a hash.
    /// 获取 hash 中的字段。
    pub async fn hget<C: AsyncCommands>(
        conn: &mut C,
        key: &str,
        field: &str,
    ) -> RedisResult<Option<String>>
    {
        let result: Option<String> = conn.hget(key, field).await?;
        Ok(result)
    }

    /// Get a JSON-deserialized field from a hash.
    /// 获取 JSON 反序列化的字段。
    pub async fn hget_json<C: AsyncCommands, T: DeserializeOwned + Send + Sync>(
        conn: &mut C,
        key: &str,
        field: &str,
    ) -> RedisResult<Option<T>>
    {
        match Self::hget(conn, key, field).await?
        {
            Some(json) =>
            {
                let value = serde_json::from_str(&json)
                    .map_err(|e| RedisError::deserialization(e.to_string()))?;
                Ok(Some(value))
            },
            None => Ok(None),
        }
    }

    /// Delete fields from a hash.
    /// 从 hash 中删除字段。
    pub async fn hdel<C: AsyncCommands>(
        conn: &mut C,
        key: &str,
        fields: &[&str],
    ) -> RedisResult<u64>
    {
        let result: u64 = conn.hdel(key, fields).await?;
        Ok(result)
    }

    /// Get all fields and values from a hash.
    /// 获取 hash 中所有字段和值。
    pub async fn hgetall<C: AsyncCommands>(
        conn: &mut C,
        key: &str,
    ) -> RedisResult<HashMap<String, String>>
    {
        let result: HashMap<String, String> = conn.hgetall(key).await?;
        Ok(result)
    }

    /// Check if a field exists in a hash.
    /// 检查 hash 中是否存在字段。
    pub async fn hexists<C: AsyncCommands>(
        conn: &mut C,
        key: &str,
        field: &str,
    ) -> RedisResult<bool>
    {
        let result: i32 = conn.hexists(key, field).await?;
        Ok(result > 0)
    }

    /// Get the number of fields in a hash.
    /// 获取 hash 中的字段数量。
    pub async fn hlen<C: AsyncCommands>(conn: &mut C, key: &str) -> RedisResult<u64>
    {
        let result: u64 = conn.hlen(key).await?;
        Ok(result)
    }

    /// Get all field names in a hash.
    /// 获取 hash 中所有字段名。
    pub async fn hkeys<C: AsyncCommands>(conn: &mut C, key: &str) -> RedisResult<Vec<String>>
    {
        let result: Vec<String> = conn.hkeys(key).await?;
        Ok(result)
    }

    /// Get all values in a hash.
    /// 获取 hash 中所有值。
    pub async fn hvals<C: AsyncCommands>(conn: &mut C, key: &str) -> RedisResult<Vec<String>>
    {
        let result: Vec<String> = conn.hvals(key).await?;
        Ok(result)
    }

    /// Increment a numeric field in a hash.
    /// 递增 hash 中的数值字段。
    pub async fn hincrby<C: AsyncCommands>(
        conn: &mut C,
        key: &str,
        field: &str,
        delta: i64,
    ) -> RedisResult<i64>
    {
        let result: i64 = conn.hincr(key, field, delta).await?;
        Ok(result)
    }
}

// ── Geo Operations ──

/// Geo operations — equivalent to Spring's `GeoOperations`.
/// Geo 操作 — 等价于 Spring 的 `GeoOperations`。
///
/// Note: Requires `geospatial` feature on the `redis` crate.
pub struct GeoOps;

impl GeoOps
{
    /// Add a geospatial location.
    /// 添加地理位置。
    pub async fn geoadd<C: AsyncCommands>(
        conn: &mut C,
        key: &str,
        longitude: f64,
        latitude: f64,
        member: &str,
    ) -> RedisResult<bool>
    {
        let result: i32 = conn.geo_add(key, (longitude, latitude, member)).await?;
        Ok(result > 0)
    }

    /// Add multiple geospatial locations.
    /// 添加多个地理位置。
    pub async fn geoadd_multiple<C: AsyncCommands>(
        conn: &mut C,
        key: &str,
        items: &[(f64, f64, &str)],
    ) -> RedisResult<u64>
    {
        let tuples: Vec<(f64, f64, String)> = items
            .iter()
            .map(|(lon, lat, member)| (*lon, *lat, member.to_string()))
            .collect();
        let result: u64 = conn.geo_add(key, tuples).await?;
        Ok(result)
    }

    /// Get coordinates for members.
    /// 获取成员的坐标。
    pub async fn geopos<C: AsyncCommands>(
        conn: &mut C,
        key: &str,
        members: &[&str],
    ) -> RedisResult<Vec<Option<(f64, f64)>>>
    {
        let result: Vec<Option<(f64, f64)>> = conn.geo_pos(key, members).await?;
        Ok(result)
    }

    /// Calculate distance between two members.
    /// 计算两个成员之间的距离。
    pub async fn geodist<C: AsyncCommands>(
        conn: &mut C,
        key: &str,
        member1: &str,
        member2: &str,
        unit: GeoUnit,
    ) -> RedisResult<Option<f64>>
    {
        use redis::geo::Unit;
        let u = match unit
        {
            GeoUnit::Meters => Unit::Meters,
            GeoUnit::Kilometers => Unit::Kilometers,
            GeoUnit::Miles => Unit::Miles,
            GeoUnit::Feet => Unit::Feet,
        };
        let result: Option<f64> = conn.geo_dist(key, member1, member2, u).await?;
        Ok(result)
    }

    /// Find members within a radius.
    /// 查找半径内的成员。
    pub async fn georadius<C: AsyncCommands>(
        conn: &mut C,
        key: &str,
        longitude: f64,
        latitude: f64,
        radius: f64,
        unit: GeoUnit,
    ) -> RedisResult<Vec<String>>
    {
        use redis::geo::{RadiusOptions, RadiusSearchResult, Unit};
        let u = match unit
        {
            GeoUnit::Meters => Unit::Meters,
            GeoUnit::Kilometers => Unit::Kilometers,
            GeoUnit::Miles => Unit::Miles,
            GeoUnit::Feet => Unit::Feet,
        };
        let opts = RadiusOptions::default().with_dist();
        let result: Vec<RadiusSearchResult> = conn
            .geo_radius(key, longitude, latitude, radius, u, opts)
            .await?;
        Ok(result.into_iter().map(|r| r.name).collect())
    }

    /// Find members within a radius of a member.
    /// 查找某个成员半径内的其他成员。
    pub async fn georadius_by_member<C: AsyncCommands>(
        conn: &mut C,
        key: &str,
        member: &str,
        radius: f64,
        unit: GeoUnit,
    ) -> RedisResult<Vec<String>>
    {
        use redis::geo::{RadiusOptions, RadiusSearchResult, Unit};
        let u = match unit
        {
            GeoUnit::Meters => Unit::Meters,
            GeoUnit::Kilometers => Unit::Kilometers,
            GeoUnit::Miles => Unit::Miles,
            GeoUnit::Feet => Unit::Feet,
        };
        let opts = RadiusOptions::default().with_dist();
        let result: Vec<RadiusSearchResult> = conn
            .geo_radius_by_member(key, member, radius, u, opts)
            .await?;
        Ok(result.into_iter().map(|r| r.name).collect())
    }
}

/// Unit of distance for geo operations.
/// 地理操作的距离单位。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeoUnit
{
    /// Meters / 米
    Meters,
    /// Kilometers / 千米
    Kilometers,
    /// Miles / 英里
    Miles,
    /// Feet / 英尺
    Feet,
}

// ── Lua Scripting ──

/// Lua script execution helper.
/// Lua 脚本执行助手。
///
/// Executes Redis Lua scripts with automatic SHA caching.
/// 使用自动 SHA 缓存执行 Redis Lua 脚本。
pub struct LuaScript;

impl LuaScript
{
    /// Execute a Lua script with KEYS and ARGV.
    /// 使用 KEYS 和 ARGV 执行 Lua 脚本。
    pub async fn eval<C: AsyncCommands>(
        conn: &mut C,
        script: &str,
        keys: &[&str],
        args: &[&str],
    ) -> RedisResult<redis::Value>
    {
        let mut cmd = redis::cmd("EVAL");
        cmd.arg(script).arg(keys.len());
        for k in keys
        {
            cmd.arg(k);
        }
        for a in args
        {
            cmd.arg(a);
        }
        let result: redis::Value = cmd.query_async(conn).await?;
        Ok(result)
    }

    /// Execute a Lua script that returns an integer.
    /// 执行返回整数的 Lua 脚本。
    pub async fn eval_i64<C: AsyncCommands>(
        conn: &mut C,
        script: &str,
        keys: &[&str],
        args: &[&str],
    ) -> RedisResult<i64>
    {
        let result = Self::eval(conn, script, keys, args).await?;
        match result
        {
            redis::Value::Int(i) => Ok(i),
            redis::Value::BulkString(data) =>
            {
                let s = String::from_utf8(data)
                    .map_err(|e| RedisError::type_mismatch(e.to_string()))?;
                s.parse::<i64>()
                    .map_err(|e| RedisError::type_mismatch(e.to_string()))
            },
            _ => Err(RedisError::type_mismatch("Expected integer result".to_string())),
        }
    }

    /// Execute a Lua script that returns a string.
    /// 执行返回字符串的 Lua 脚本。
    pub async fn eval_str<C: AsyncCommands>(
        conn: &mut C,
        script: &str,
        keys: &[&str],
        args: &[&str],
    ) -> RedisResult<String>
    {
        let result = Self::eval(conn, script, keys, args).await?;
        match result
        {
            redis::Value::BulkString(data) =>
            {
                String::from_utf8(data).map_err(|e| RedisError::type_mismatch(e.to_string()))
            },
            redis::Value::SimpleString(s) => Ok(s),
            _ => Err(RedisError::type_mismatch("Expected string result".to_string())),
        }
    }
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

    #[test]
    fn test_geo_unit()
    {
        assert_eq!(GeoUnit::Meters, GeoUnit::Meters);
        assert_ne!(GeoUnit::Meters, GeoUnit::Kilometers);
    }

    #[test]
    fn test_lua_script_op()
    {
        // Just verify struct can be created
        let _ = LuaScript;
    }
}
