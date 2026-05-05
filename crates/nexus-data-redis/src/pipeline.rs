//! Redis Pipeline — Batch Command Execution
//! Redis 管道 — 批量命令执行

use crate::{RedisError, RedisResult};

/// A pipelined batch of Redis commands.
/// 一批管道的 Redis 命令。
#[derive(Debug, Clone, Default)]
pub struct RedisPipeline {
    commands: Vec<(String, Vec<Vec<u8>>)>,
}

impl RedisPipeline {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn cmd(mut self, command: impl Into<String>, args: Vec<impl Into<Vec<u8>>>) -> Self {
        self.commands.push((
            command.into().to_uppercase(),
            args.into_iter().map(|a| a.into()).collect(),
        ));
        self
    }

    #[must_use]
    pub fn set(mut self, key: impl Into<String>, value: impl Into<Vec<u8>>) -> Self {
        self.commands.push((
            "SET".to_string(),
            vec![key.into().into_bytes(), value.into()],
        ));
        self
    }

    #[must_use]
    pub fn set_ex(mut self, key: impl Into<String>, value: impl Into<Vec<u8>>, seconds: u64) -> Self {
        self.commands.push((
            "SETEX".to_string(),
            vec![
                key.into().into_bytes(),
                seconds.to_string().into_bytes(),
                value.into(),
            ],
        ));
        self
    }

    #[must_use]
    pub fn get(mut self, key: impl Into<String>) -> Self {
        self.commands
            .push(("GET".to_string(), vec![key.into().into_bytes()]));
        self
    }

    #[must_use]
    pub fn del(mut self, keys: Vec<impl Into<String>>) -> Self {
        let key_bytes: Vec<Vec<u8>> = keys.into_iter().map(|k| k.into().into_bytes()).collect();
        self.commands.push(("DEL".to_string(), key_bytes));
        self
    }

    #[must_use]
    pub fn exists(mut self, key: impl Into<String>) -> Self {
        self.commands
            .push(("EXISTS".to_string(), vec![key.into().into_bytes()]));
        self
    }

    #[must_use]
    pub fn expire(mut self, key: impl Into<String>, seconds: u64) -> Self {
        self.commands.push((
            "EXPIRE".to_string(),
            vec![key.into().into_bytes(), seconds.to_string().into_bytes()],
        ));
        self
    }

    #[must_use]
    pub fn incr(mut self, key: impl Into<String>) -> Self {
        self.commands
            .push(("INCR".to_string(), vec![key.into().into_bytes()]));
        self
    }

    #[must_use]
    pub fn decr(mut self, key: impl Into<String>) -> Self {
        self.commands
            .push(("DECR".to_string(), vec![key.into().into_bytes()]));
        self
    }

    #[must_use]
    pub fn hset(mut self, key: impl Into<String>, field: impl Into<String>, value: impl Into<Vec<u8>>) -> Self {
        self.commands.push((
            "HSET".to_string(),
            vec![key.into().into_bytes(), field.into().into_bytes(), value.into()],
        ));
        self
    }

    #[must_use]
    pub fn hget(mut self, key: impl Into<String>, field: impl Into<String>) -> Self {
        self.commands.push((
            "HGET".to_string(),
            vec![key.into().into_bytes(), field.into().into_bytes()],
        ));
        self
    }

    #[must_use]
    pub fn sadd(mut self, key: impl Into<String>, member: impl Into<String>) -> Self {
        self.commands.push((
            "SADD".to_string(),
            vec![key.into().into_bytes(), member.into().into_bytes()],
        ));
        self
    }

    #[must_use]
    pub fn zadd(mut self, key: impl Into<String>, score: f64, member: impl Into<String>) -> Self {
        self.commands.push((
            "ZADD".to_string(),
            vec![
                key.into().into_bytes(),
                score.to_string().into_bytes(),
                member.into().into_bytes(),
            ],
        ));
        self
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Execute the pipeline and return raw results.
    /// 执行管道并返回原始结果。
    pub async fn execute(
        &self,
        conn: &mut redis::aio::MultiplexedConnection,
    ) -> RedisResult<PipelineResult> {
        if self.commands.is_empty() {
            return Ok(PipelineResult::default());
        }

        let mut pipe = redis::pipe();
        for (cmd, args) in &self.commands {
            let mut c = redis::Cmd::new();
            c.arg(cmd.as_str());
            for arg in args {
                c.arg(arg.as_slice());
            }
            pipe.add_command(c);
        }

        let results: Vec<redis::Value> =
            pipe.query_async(conn).await.map_err(RedisError::from)?;

        Ok(PipelineResult {
            results,
            command_count: self.commands.len(),
        })
    }
}

/// Result of a pipeline execution.
/// 管道执行的结果。
#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub results: Vec<redis::Value>,
    pub command_count: usize,
}

impl Default for PipelineResult {
    fn default() -> Self {
        Self {
            results: Vec::new(),
            command_count: 0,
        }
    }
}

impl PipelineResult {
    /// Get result as string at index.
    pub fn get_string(&self, index: usize) -> RedisResult<String> {
        self.results
            .get(index)
            .and_then(|v| match v {
                redis::Value::BulkString(data) => String::from_utf8(data.clone()).ok(),
                redis::Value::SimpleString(s) => Some(s.clone()),
                redis::Value::Okay => Some("OK".to_string()),
                _ => None,
            })
            .ok_or_else(|| RedisError::type_mismatch(format!(
                "Expected string at index {index}"
            )))
    }

    /// Get result as i64 at index.
    pub fn get_i64(&self, index: usize) -> RedisResult<i64> {
        self.results
            .get(index)
            .and_then(|v| match v {
                redis::Value::Int(i) => Some(*i),
                _ => None,
            })
            .ok_or_else(|| RedisError::type_mismatch(format!(
                "Expected integer at index {index}"
            )))
    }

    /// Get result as optional string at index.
    pub fn get_optional_string(&self, index: usize) -> RedisResult<Option<String>> {
        match self.results.get(index) {
            Some(redis::Value::Nil) => Ok(None),
            Some(redis::Value::BulkString(data)) => {
                Ok(Some(String::from_utf8(data.clone()).map_err(|e| {
                    RedisError::type_mismatch(e.to_string())
                })?))
            }
            Some(redis::Value::SimpleString(s)) => Ok(Some(s.clone())),
            _ => Err(RedisError::type_mismatch(format!(
                "Expected optional string at index {index}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_builder() {
        let pipe = RedisPipeline::new()
            .set("key1", "value1")
            .set("key2", "value2")
            .get("key1");
        assert_eq!(pipe.len(), 3);
        assert!(!pipe.is_empty());
    }

    #[test]
    fn test_pipeline_empty() {
        let pipe = RedisPipeline::new();
        assert!(pipe.is_empty());
        assert_eq!(pipe.len(), 0);
    }

    #[test]
    fn test_pipeline_with_expiry() {
        let pipe = RedisPipeline::new()
            .set_ex("session", "data", 3600)
            .get("session");
        assert_eq!(pipe.len(), 2);
    }

    #[test]
    fn test_pipeline_hash_ops() {
        let pipe = RedisPipeline::new()
            .hset("user:1", "name", b"Alice".to_vec())
            .hset("user:1", "email", b"alice@example.com".to_vec())
            .hget("user:1", "name");
        assert_eq!(pipe.len(), 3);
    }

    #[test]
    fn test_pipeline_del() {
        let pipe = RedisPipeline::new().del(vec!["key1", "key2", "key3"]);
        assert_eq!(pipe.len(), 1);
    }

    #[test]
    fn test_pipeline_result_default() {
        let result = PipelineResult::default();
        assert_eq!(result.command_count, 0);
        assert!(result.results.is_empty());
    }

    #[test]
    fn test_pipeline_sorted_set_counter() {
        let pipe = RedisPipeline::new()
            .zadd("leaderboard", 100.0, "player1")
            .incr("counter")
            .decr("counter");
        assert_eq!(pipe.len(), 3);
    }
}
