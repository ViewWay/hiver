//! Redis integration tests using testcontainers.
//! 使用 testcontainers 的 Redis 集成测试。
//!
//! These tests require Docker to be running.
//! 这些测试需要 Docker 正在运行。
//!
//! Run with: cargo test --features integration-tests --test redis_integration

use std::time::Duration;

use redis::AsyncCommands;
use testcontainers::core::IntoContainerPort;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::redis::Redis;

/// Helper: start a Redis container and return connection + container.
/// 辅助函数：启动 Redis 容器并返回连接和容器。
async fn setup_redis() -> (redis::aio::MultiplexedConnection, testcontainers::ContainerAsync<Redis>)
{
    let container = Redis::default()
        .start()
        .await
        .expect("Failed to start Redis container");

    let host_port = container
        .get_host_port_ipv4(6379.tcp())
        .await
        .expect("Failed to get Redis port");

    let client = redis::Client::open(format!("redis://127.0.0.1:{host_port}"))
        .expect("Failed to create Redis client");

    let con = client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to get Redis connection");

    (con, container)
}

// ============================================================
// Test 1: Redis container starts and accepts connections (PING)
// 测试 1：Redis 容器启动并接受连接（PING）
// ============================================================
#[tokio::test]
async fn test_redis_connectivity() {
    let (mut con, _container) = setup_redis().await;

    let pong: String = redis::cmd("PING")
        .query_async(&mut con)
        .await
        .expect("Failed to PING Redis");

    assert_eq!(pong, "PONG", "Redis should respond with PONG");
}

// ============================================================
// Test 2: SET and GET string values
// 测试 2：SET 和 GET 字符串值
// ============================================================
#[tokio::test]
async fn test_redis_set_get_string() {
    let (mut con, _container) = setup_redis().await;

    let _: () = con
        .set("test:key", "hello_world")
        .await
        .expect("Failed to SET key");

    let value: String = con
        .get("test:key")
        .await
        .expect("Failed to GET key")
        .expect("Key should exist");

    assert_eq!(value, "hello_world");
}

// ============================================================
// Test 3: SET with expiration (TTL)
// 测试 3：带过期时间的 SET（TTL）
// ============================================================
#[tokio::test]
async fn test_redis_set_with_expiry() {
    let (mut con, _container) = setup_redis().await;

    let _: () = con
        .set_ex("test:expiring", "temp_value", 60)
        .await
        .expect("Failed to SET key with expiry");

    let ttl: i64 = con.ttl("test:expiring").await.expect("Failed to get TTL");

    assert!(ttl > 0 && ttl <= 60, "TTL should be between 1 and 60, got {ttl}");
}

// ============================================================
// Test 4: DELETE key
// 测试 4：DELETE 键
// ============================================================
#[tokio::test]
async fn test_redis_delete_key() {
    let (mut con, _container) = setup_redis().await;

    let _: () = con
        .set("test:to_delete", "value")
        .await
        .expect("Failed to SET key");

    let deleted: i64 = con.del("test:to_delete").await.expect("Failed to DEL key");

    assert_eq!(deleted, 1, "Should delete exactly 1 key");

    let result: Option<String> = con
        .get("test:to_delete")
        .await
        .expect("GET should not fail");
    assert!(result.is_none(), "Deleted key should return None");
}

// ============================================================
// Test 5: EXISTS check
// 测试 5：EXISTS 检查
// ============================================================
#[tokio::test]
async fn test_redis_exists() {
    let (mut con, _container) = setup_redis().await;

    let _: () = con
        .set("test:existing", "yes")
        .await
        .expect("Failed to SET key");

    let exists_existing: bool = con
        .exists("test:existing")
        .await
        .expect("Failed to check EXISTS");
    assert!(exists_existing, "Existing key should return true");

    let exists_missing: bool = con
        .exists("test:nonexistent")
        .await
        .expect("Failed to check EXISTS");
    assert!(!exists_missing, "Non-existent key should return false");
}

// ============================================================
// Test 6: INCR and DECR integer operations
// 测试 6：INCR 和 DECR 整数操作
// ============================================================
#[tokio::test]
async fn test_redis_incr_decr() {
    let (mut con, _container) = setup_redis().await;

    let _: () = con
        .set("test:counter", 10)
        .await
        .expect("Failed to SET counter");

    let incremented: i64 = con.incr("test:counter", 5).await.expect("Failed to INCR");
    assert_eq!(incremented, 15, "Counter should be 15 after INCR by 5");

    let decremented: i64 = con.decr("test:counter", 3).await.expect("Failed to DECR");
    assert_eq!(decremented, 12, "Counter should be 12 after DECR by 3");
}

// ============================================================
// Test 7: Hash operations (HSET, HGET, HGETALL)
// 测试 7：Hash 操作（HSET, HGET, HGETALL）
// ============================================================
#[tokio::test]
async fn test_redis_hash_operations() {
    let (mut con, _container) = setup_redis().await;

    let _: () = con
        .hset("test:user:1", "name", "Alice")
        .await
        .expect("Failed to HSET name");
    let _: () = con
        .hset("test:user:1", "age", "30")
        .await
        .expect("Failed to HSET age");
    let _: () = con
        .hset("test:user:1", "email", "alice@example.com")
        .await
        .expect("Failed to HSET email");

    let name: String = con
        .hget("test:user:1", "name")
        .await
        .expect("Failed to HGET")
        .expect("Field should exist");
    assert_eq!(name, "Alice");

    let all_fields: std::collections::HashMap<String, String> =
        con.hgetall("test:user:1").await.expect("Failed to HGETALL");
    assert_eq!(all_fields.len(), 3, "Hash should have 3 fields");
    assert_eq!(all_fields.get("age"), Some(&"30".to_string()));
}

// ============================================================
// Test 8: List operations (LPUSH, RPUSH, LRANGE, LLEN)
// 测试 8：List 操作（LPUSH, RPUSH, LRANGE, LLEN）
// ============================================================
#[tokio::test]
async fn test_redis_list_operations() {
    let (mut con, _container) = setup_redis().await;

    let _: () = con
        .lpush("test:tasks", "task_c")
        .await
        .expect("Failed to LPUSH");
    let _: () = con
        .rpush("test:tasks", "task_a")
        .await
        .expect("Failed to RPUSH");
    let _: () = con
        .rpush("test:tasks", "task_b")
        .await
        .expect("Failed to RPUSH");

    let len: i64 = con.llen("test:tasks").await.expect("Failed to LLEN");
    assert_eq!(len, 3, "List should have 3 elements");

    let items: Vec<String> = con
        .lrange("test:tasks", 0, -1)
        .await
        .expect("Failed to LRANGE");
    assert_eq!(items, vec!["task_c", "task_a", "task_b"]);
}

// ============================================================
// Test 9: Set operations (SADD, SMEMBERS, SISMEMBER, SCARD)
// 测试 9：Set 操作（SADD, SMEMBERS, SISMEMBER, SCARD）
// ============================================================
#[tokio::test]
async fn test_redis_set_operations() {
    let (mut con, _container) = setup_redis().await;

    let added: i64 = con.sadd("test:tags", "rust").await.expect("Failed to SADD");
    assert_eq!(added, 1);

    let _: () = con
        .sadd("test:tags", "async")
        .await
        .expect("Failed to SADD");
    let _: () = con.sadd("test:tags", "web").await.expect("Failed to SADD");

    let card: i64 = con.scard("test:tags").await.expect("Failed to SCARD");
    assert_eq!(card, 3);

    let is_member: bool = con
        .sismember("test:tags", "rust")
        .await
        .expect("Failed to SISMEMBER");
    assert!(is_member);

    let is_not_member: bool = con
        .sismember("test:tags", "java")
        .await
        .expect("Failed to SISMEMBER");
    assert!(!is_not_member);

    let mut members: Vec<String> = con.smembers("test:tags").await.expect("Failed to SMEMBERS");
    members.sort();
    assert_eq!(members, vec!["async", "rust", "web"]);
}

// ============================================================
// Test 10: Sorted Set operations (ZADD, ZRANGE, ZRANK, ZCARD)
// 测试 10：Sorted Set 操作（ZADD, ZRANGE, ZRANK, ZCARD）
// ============================================================
#[tokio::test]
async fn test_redis_sorted_set_operations() {
    let (mut con, _container) = setup_redis().await;

    let _: () = con
        .zadd("test:leaderboard", "alice", 100.0)
        .await
        .expect("Failed to ZADD");
    let _: () = con
        .zadd("test:leaderboard", "bob", 85.0)
        .await
        .expect("Failed to ZADD");
    let _: () = con
        .zadd("test:leaderboard", "charlie", 92.0)
        .await
        .expect("Failed to ZADD");

    let card: i64 = con
        .zcard("test:leaderboard")
        .await
        .expect("Failed to ZCARD");
    assert_eq!(card, 3);

    // ZRANGE returns members in ascending score order
    let members: Vec<String> = con
        .zrange("test:leaderboard", 0, -1)
        .await
        .expect("Failed to ZRANGE");
    assert_eq!(members, vec!["bob", "charlie", "alice"]);

    let rank: i64 = con
        .zrank("test:leaderboard", "alice")
        .await
        .expect("Failed to ZRANK")
        .expect("Member should exist");
    assert_eq!(rank, 2, "Alice should be rank 2 (0-indexed)");
}

// ============================================================
// Test 11: Multiple key operations (MSET, MGET)
// 测试 11：多键操作（MSET, MGET）
// ============================================================
#[tokio::test]
async fn test_redis_mset_mget() {
    let (mut con, _container) = setup_redis().await;

    let _: () = con
        .set_multiple(&[
            ("test:mk1", "value1"),
            ("test:mk2", "value2"),
            ("test:mk3", "value3"),
        ])
        .await
        .expect("Failed to MSET");

    let values: Vec<Option<String>> = con
        .mget(&["test:mk1", "test:mk2", "test:mk3"])
        .await
        .expect("Failed to MGET");

    assert_eq!(values.len(), 3);
    assert_eq!(values[0], Some("value1".to_string()));
    assert_eq!(values[1], Some("value2".to_string()));
    assert_eq!(values[2], Some("value3".to_string()));
}

// ============================================================
// Test 12: KEYS pattern matching
// 测试 12：KEYS 模式匹配
// ============================================================
#[tokio::test]
async fn test_redis_keys_pattern() {
    let (mut con, _container) = setup_redis().await;

    let _: () = con
        .set("test:session:a", "data_a")
        .await
        .expect("SET failed");
    let _: () = con
        .set("test:session:b", "data_b")
        .await
        .expect("SET failed");
    let _: () = con.set("test:cache:c", "data_c").await.expect("SET failed");

    let session_keys: Vec<String> = con.keys("test:session:*").await.expect("Failed to KEYS");
    assert_eq!(session_keys.len(), 2, "Should find 2 session keys");

    let all_test_keys: Vec<String> = con.keys("test:*").await.expect("Failed to KEYS");
    assert_eq!(all_test_keys.len(), 3, "Should find 3 test keys total");
}
