//! Cache Example
//! 缓存示例
//!
//! Demonstrates the use of Nexus cache annotations equivalent to Spring's @Cacheable, @CachePut, and @CacheEvict.
//! 演示Nexus缓存注解的使用，等同于Spring的@Cacheable、@CachePut和@CacheEvict。

use hiver_cache::{
    Cache, CacheConfig, CacheEvictExec, CachePutExec, Cached, MemoryCache,
};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    id: String,
    name: String,
    email: String,
}

#[derive(Clone)]
pub struct UserService {
    cache: Arc<MemoryCache<String, User>>,
}

impl UserService {
    /// Create a new user service with cache
    /// 创建带缓存的新用户服务
    pub fn new() -> Self {
        let config = CacheConfig::new("users").max_capacity(100).ttl_secs(3600); // 1 hour TTL
        Self {
            cache: Arc::new(MemoryCache::new(config)),
        }
    }

    /// Get user by ID - cached (@Cacheable equivalent)
    /// 通过ID获取用户 - 缓存（@Cacheable等价物）
    ///
    /// # Spring Equivalent / Spring等价物
    /// ```java
    /// @Cacheable("users")
    /// public User getUser(String id) {
    ///     return userRepository.findById(id);
    /// }
    /// ```
    pub async fn get_user(&self, id: &str) -> Option<User> {
        Cached::get_or_fetch(self.cache.as_ref(), &id.to_string(), || async {
            // Simulate database lookup
            // 模拟数据库查找
            println!("[Database] Fetching user: {}", id);
            self.fetch_from_database(id).await
        })
        .await
    }

    /// Update user - always executes and updates cache (@CachePut equivalent)
    /// 更新用户 - 始终执行并更新缓存（@CachePut等价物）
    ///
    /// # Spring Equivalent / Spring等价物
    /// ```java
    /// @CachePut(value = "users", key = "#user.id")
    /// public User updateUser(User user) {
    ///     return userRepository.save(user);
    /// }
    /// ```
    pub async fn update_user(&self, user: User) -> User {
        CachePutExec::execute_and_update(self.cache.as_ref(), user.id.clone(), || async {
            // Simulate database update
            // 模拟数据库更新
            println!("[Database] Updating user: {}", user.id);
            user.clone()
        })
        .await
    }

    /// Delete user - removes from cache (@CacheEvict equivalent)
    /// 删除用户 - 从缓存中移除（@CacheEvict等价物）
    ///
    /// # Spring Equivalent / Spring等价物
    /// ```java
    /// @CacheEvict(value = "users", key = "#id")
    /// public void deleteUser(String id) {
    ///     userRepository.deleteById(id);
    /// }
    /// ```
    pub async fn delete_user(&self, id: &str) {
        CacheEvictExec::execute_and_evict(self.cache.as_ref(), &id.to_string(), || async {
            // Simulate database deletion
            // 模拟数据库删除
            println!("[Database] Deleting user: {}", id);
        })
        .await
    }

    /// Delete all users - clears entire cache
    /// 删除所有用户 - 清除整个缓存
    ///
    /// # Spring Equivalent / Spring等价物
    /// ```java
    /// @CacheEvict(value = "users", allEntries = true)
    /// public void deleteAllUsers() {
    ///     userRepository.deleteAll();
    /// }
    /// ```
    pub async fn delete_all_users(&self) {
        CacheEvictExec::execute_and_evict_all(self.cache.as_ref(), || async {
            // Simulate database deletion of all users
            // 模拟删除所有用户的数据库操作
            println!("[Database] Deleting all users");
        })
        .await
    }

    /// Simulate database fetch
    /// 模拟数据库获取
    async fn fetch_from_database(&self, id: &str) -> Option<User> {
        // Simulate database latency
        // 模拟数据库延迟
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        match id {
            "1" => Some(User {
                id: "1".to_string(),
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
            }),
            "2" => Some(User {
                id: "2".to_string(),
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
            }),
            _ => None,
        }
    }

    /// Get cache statistics
    /// 获取缓存统计
    pub async fn get_stats(&self) -> (u64, u64, f64, usize) {
        let stats = self.cache.stats().await;
        (stats.hits, stats.misses, stats.hit_rate, stats.size)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Nexus Cache Example / Nexus缓存示例 ===\n");

    let service = UserService::new();

    // Example 1: Cache miss - fetches from database
    // 示例1: 缓存未命中 - 从数据库获取
    println!("1. First call - Cache Miss / 第一次调用 - 缓存未命中");
    let user1 = service.get_user("1").await;
    println!("   Result: {:?}\n", user1);

    // Example 2: Cache hit - returns from cache
    // 示例2: 缓存命中 - 从缓存返回
    println!("2. Second call - Cache Hit / 第二次调用 - 缓存命中");
    let user2 = service.get_user("1").await;
    println!("   Result: {:?}\n", user2);

    // Check stats
    // 检查统计
    let (hits, misses, rate, size) = service.get_stats().await;
    println!(
        "   Stats: Hits={}, Misses={}, Hit Rate={:.2}%, Size={}\n",
        hits,
        misses,
        rate * 100.0,
        size
    );

    // Example 3: CachePut - update user and update cache
    // 示例3: CachePut - 更新用户并更新缓存
    println!("3. Update user - CachePut / 更新用户 - CachePut");
    let updated_user = User {
        id: "1".to_string(),
        name: "Alice Updated".to_string(),
        email: "alice.updated@example.com".to_string(),
    };
    service.update_user(updated_user.clone()).await;
    println!("   Updated: {:?}\n", updated_user);

    // Get updated user from cache
    // 从缓存获取更新后的用户
    let user3 = service.get_user("1").await;
    println!("   From cache: {:?}\n", user3);

    // Example 4: CacheEvict - delete user from cache
    // 示例4: CacheEvict - 从缓存删除用户
    println!("4. Delete user - CacheEvict / 删除用户 - CacheEvict");
    service.delete_user("1").await;
    println!("   User deleted from cache\n");

    // Check stats again
    // 再次检查统计
    let (hits, misses, rate, size) = service.get_stats().await;
    println!(
        "   Stats: Hits={}, Misses={}, Hit Rate={:.2}%, Size={}\n",
        hits,
        misses,
        rate * 100.0,
        size
    );

    // Example 5: Fetch again after eviction - cache miss
    // 示例5: 驱逐后再次获取 - 缓存未命中
    println!("5. Fetch after eviction - Cache Miss / 驱逐后获取 - 缓存未命中");
    let user4 = service.get_user("1").await;
    println!("   Result: {:?}\n", user4);

    // Example 6: allEntries = true - clear all cache
    // 示例6: allEntries = true - 清除所有缓存
    println!("6. Add multiple users to cache / 添加多个用户到缓存");
    service.get_user("1").await;
    service.get_user("2").await;
    let (hits, misses, rate, size) = service.get_stats().await;
    println!(
        "   Before clear: Hits={}, Misses={}, Hit Rate={:.2}%, Size={}\n",
        hits,
        misses,
        rate * 100.0,
        size
    );

    println!("7. Clear all cache entries / 清除所有缓存条目");
    service.delete_all_users().await;
    let (hits, misses, rate, size) = service.get_stats().await;
    println!(
        "   After clear: Hits={}, Misses={}, Hit Rate={:.2}%, Size={}\n",
        hits,
        misses,
        rate * 100.0,
        size
    );

    println!("=== Example Complete / 示例完成 ===");

    Ok(())
}
