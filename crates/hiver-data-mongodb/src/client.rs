//! MongoDB client management
//! MongoDB 客户端管理

use std::sync::Arc;

use mongodb::{Client, Database};

use crate::{MongoError, MongoResult};
use futures_util::StreamExt;

/// MongoDB client wrapper / MongoDB 客户端包装器
#[derive(Debug, Clone)]
pub struct MongoClient {
    /// Inner MongoDB client / 内部 MongoDB 客户端
    client: Arc<Client>,
    /// Database name / 数据库名称
    database_name: Arc<String>,
}

impl MongoClient {
    /// Create a new MongoDB client / 创建新的 MongoDB 客户端
    ///
    /// # Arguments / 参数
    ///
    /// * `connection_string` - MongoDB connection string / MongoDB 连接字符串
    /// * `database_name` - Default database name / 默认数据库名称
    pub async fn new(connection_string: &str, database_name: &str) -> MongoResult<Self> {
        let client = Client::with_uri_str(connection_string).await?;

        Ok(Self {
            client: Arc::new(client),
            database_name: Arc::new(database_name.to_string()),
        })
    }

    /// Create from existing MongoDB client / 从现有的 MongoDB 客户端创建
    pub fn from_client(client: Client, database_name: &str) -> Self {
        Self {
            client: Arc::new(client),
            database_name: Arc::new(database_name.to_string()),
        }
    }

    /// Get the inner client / 获取内部客户端
    pub fn inner(&self) -> &Client {
        &self.client
    }

    /// Get the database name / 获取数据库名称
    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    /// Get a database / 获取数据库
    pub fn database(&self) -> Database {
        self.client.database(&self.database_name)
    }

    /// Get a collection / 获取集合
    pub fn collection<T: Send + Sync>(&self, name: &str) -> mongodb::Collection<T> {
        self.database().collection(name)
    }

    /// Ping the database / 检查数据库连接
    pub async fn ping(&self) -> MongoResult<()> {
        self.database()
            .run_command(mongodb::bson::doc! {"ping": 1})
            .await
            .map(|_| ())
            .map_err(MongoError::from)
    }

    /// List all collections in the database / 列出数据库中的所有集合
    pub async fn list_collection_names(&self) -> MongoResult<Vec<String>> {
        self.database()
            .list_collection_names()
            .await
            .map_err(MongoError::from)
    }

    /// Drop a collection / 删除集合
    pub async fn drop_collection(&self, name: &str) -> MongoResult<()> {
        self.database()
            .collection::<mongodb::bson::Document>(name)
            .drop()
            .await
            .map_err(MongoError::from)
    }

    /// Create a collection / 创建集合
    pub async fn create_collection(&self, name: &str) -> MongoResult<()> {
        self.database()
            .create_collection(name)
            .await
            .map_err(MongoError::from)
    }

    /// Insert a document into a collection.
    /// 向集合中插入文档。
    pub async fn insert(
        &self,
        database: &str,
        collection: &str,
        doc: serde_json::Value,
    ) -> MongoResult<mongodb::results::InsertOneResult> {
        let bson_doc = mongodb::bson::to_document(&doc).map_err(|e| {
            MongoError::data_conversion(format!("Failed to convert to BSON: {}", e))
        })?;
        self.client
            .database(database)
            .collection::<mongodb::bson::Document>(collection)
            .insert_one(bson_doc)
            .await
            .map_err(MongoError::from)
    }

    /// Find a single document matching the filter.
    /// 查找匹配过滤器的单个文档。
    pub async fn find_one(
        &self,
        database: &str,
        collection: &str,
        filter: serde_json::Value,
    ) -> MongoResult<Option<serde_json::Value>> {
        let bson_filter = mongodb::bson::to_document(&filter).map_err(|e| {
            MongoError::data_conversion(format!("Failed to convert filter to BSON: {}", e))
        })?;
        let result: Option<mongodb::bson::Document> = self
            .client
            .database(database)
            .collection::<mongodb::bson::Document>(collection)
            .find_one(bson_filter)
            .await
            .map_err(MongoError::from)?;
        match result {
            Some(doc) => mongodb::bson::from_document::<serde_json::Value>(doc)
                .map(Some)
                .map_err(|e| {
                    MongoError::data_conversion(format!("Failed to convert from BSON: {}", e))
                }),
            None => Ok(None),
        }
    }

    /// Delete documents matching the filter.
    /// 删除匹配过滤器的文档。
    pub async fn delete(
        &self,
        database: &str,
        collection: &str,
        filter: serde_json::Value,
    ) -> MongoResult<mongodb::results::DeleteResult> {
        let bson_filter = mongodb::bson::to_document(&filter).map_err(|e| {
            MongoError::data_conversion(format!("Failed to convert filter to BSON: {}", e))
        })?;
        self.client
            .database(database)
            .collection::<mongodb::bson::Document>(collection)
            .delete_one(bson_filter)
            .await
            .map_err(MongoError::from)
    }

    /// Find all documents matching the filter.
    /// 查找匹配过滤器的所有文档。
    pub async fn find(
        &self,
        database: &str,
        collection: &str,
        filter: serde_json::Value,
    ) -> MongoResult<Vec<serde_json::Value>> {
        let bson_filter = mongodb::bson::to_document(&filter).map_err(|e| {
            MongoError::data_conversion(format!("Failed to convert filter to BSON: {}", e))
        })?;
        let mut cursor = self
            .client
            .database(database)
            .collection::<mongodb::bson::Document>(collection)
            .find(bson_filter)
            .await
            .map_err(MongoError::from)?;
        let mut results = Vec::new();
        while let Some(doc) = cursor.next().await {
            let doc = doc.map_err(MongoError::from)?;
            let json: serde_json::Value = mongodb::bson::from_document(doc).map_err(|e| {
                MongoError::data_conversion(format!("Failed to convert from BSON: {}", e))
            })?;
            results.push(json);
        }
        Ok(results)
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
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = Client::with_uri_str("mongodb://localhost:27017")
            .await
            .unwrap();
        let mc = MongoClient::from_client(client, "test_db");
        assert_eq!(mc.database_name(), "test_db");
    }
}
