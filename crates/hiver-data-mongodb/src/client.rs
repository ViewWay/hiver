//! MongoDB client management
//! MongoDB 客户端管理

use std::sync::Arc;

use mongodb::{Client, Database};

use crate::{MongoError, MongoResult};

/// MongoDB client wrapper / MongoDB 客户端包装器
#[derive(Debug, Clone)]
pub struct MongoClient
{
    /// Inner MongoDB client / 内部 MongoDB 客户端
    client: Arc<Client>,
    /// Database name / 数据库名称
    database_name: Arc<String>,
}

impl MongoClient
{
    /// Create a new MongoDB client / 创建新的 MongoDB 客户端
    ///
    /// # Arguments / 参数
    ///
    /// * `connection_string` - MongoDB connection string / MongoDB 连接字符串
    /// * `database_name` - Default database name / 默认数据库名称
    pub async fn new(connection_string: &str, database_name: &str) -> MongoResult<Self>
    {
        let client = Client::with_uri_str(connection_string).await?;

        Ok(Self {
            client: Arc::new(client),
            database_name: Arc::new(database_name.to_string()),
        })
    }

    /// Create from existing MongoDB client / 从现有的 MongoDB 客户端创建
    pub fn from_client(client: Client, database_name: &str) -> Self
    {
        Self {
            client: Arc::new(client),
            database_name: Arc::new(database_name.to_string()),
        }
    }

    /// Get the inner client / 获取内部客户端
    pub fn inner(&self) -> &Client
    {
        &self.client
    }

    /// Get the database name / 获取数据库名称
    pub fn database_name(&self) -> &str
    {
        &self.database_name
    }

    /// Get a database / 获取数据库
    pub fn database(&self) -> Database
    {
        self.client.database(&self.database_name)
    }

    /// Get a collection / 获取集合
    pub fn collection<T: Send + Sync>(&self, name: &str) -> mongodb::Collection<T>
    {
        self.database().collection(name)
    }

    /// Ping the database / 检查数据库连接
    pub async fn ping(&self) -> MongoResult<()>
    {
        self.database()
            .run_command(mongodb::bson::doc! {"ping": 1})
            .await
            .map(|_| ())
            .map_err(MongoError::from)
    }

    /// List all collections in the database / 列出数据库中的所有集合
    pub async fn list_collection_names(&self) -> MongoResult<Vec<String>>
    {
        self.database()
            .list_collection_names()
            .await
            .map_err(MongoError::from)
    }

    /// Drop a collection / 删除集合
    pub async fn drop_collection(&self, name: &str) -> MongoResult<()>
    {
        self.database()
            .collection::<mongodb::bson::Document>(name)
            .drop()
            .await
            .map_err(MongoError::from)
    }

    /// Create a collection / 创建集合
    pub async fn create_collection(&self, name: &str) -> MongoResult<()>
    {
        self.database()
            .create_collection(name)
            .await
            .map_err(MongoError::from)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[tokio::test]
    async fn test_client_creation()
    {
        let client = Client::with_uri_str("mongodb://localhost:27017")
            .await
            .unwrap();
        let mc = MongoClient::from_client(client, "test_db");
        assert_eq!(mc.database_name(), "test_db");
    }
}
