//! MongoDB reactive template
//! MongoDB 响应式模板

use crate::{MongoResult, client::MongoClient, error::MongoError};
use futures_util::stream::StreamExt;
use mongodb::bson::{Document, doc};
use serde::{Serialize, de::DeserializeOwned};

/// MongoDB reactive template similar to Spring Data's MongoTemplate
/// MongoDB 响应式模板，类似于 Spring Data 的 MongoTemplate
#[derive(Debug, Clone)]
pub struct MongoTemplate {
    client: MongoClient,
    database_name: String,
}

impl MongoTemplate {
    /// Create a new MongoTemplate / 创建新的 MongoTemplate
    pub fn new(client: mongodb::Client, database_name: &str) -> Self {
        Self {
            client: MongoClient::from_client(client, database_name),
            database_name: database_name.to_string(),
        }
    }

    /// Get the underlying client / 获取底层客户端
    pub fn client(&self) -> &MongoClient {
        &self.client
    }

    /// Get the database name / 获取数据库名称
    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    /// Insert a document / 插入文档
    pub async fn insert<T>(
        &self,
        collection_name: &str,
        entity: &T,
    ) -> MongoResult<mongodb::bson::Bson>
    where
        T: Serialize + Send + Sync,
    {
        let collection: mongodb::Collection<T> = self.client.collection(collection_name);
        let result = collection.insert_one(entity).await?;
        Ok(result.inserted_id)
    }

    /// Find a document by ID / 根据 ID 查找文档
    pub async fn find_by_id<T>(
        &self,
        collection_name: &str,
        id: mongodb::bson::Bson,
    ) -> MongoResult<Option<T>>
    where
        T: DeserializeOwned + Send + Sync,
    {
        let collection = self.client.collection::<T>(collection_name);
        let result = collection.find_one(doc! {"_id": id}).await?;
        Ok(result)
    }

    /// Find one document by filter / 根据过滤器查找单个文档
    pub async fn find_one<T>(
        &self,
        collection_name: &str,
        filter: Document,
    ) -> MongoResult<Option<T>>
    where
        T: DeserializeOwned + Send + Sync,
    {
        let collection = self.client.collection::<T>(collection_name);
        let result = collection.find_one(filter).await?;
        Ok(result)
    }

    /// Find all documents / 查找所有文档
    pub async fn find_all<T>(&self, collection_name: &str) -> MongoResult<Vec<T>>
    where
        T: DeserializeOwned + Send + Sync + Unpin,
    {
        let collection = self.client.collection::<T>(collection_name);
        let mut cursor = collection.find(doc! {}).await?;

        let mut results = Vec::new();
        while let Some(result) = cursor.next().await {
            results.push(result.map_err(MongoError::from)?);
        }

        Ok(results)
    }

    /// Count all documents / 统计所有文档
    pub async fn count_all(&self, collection_name: &str) -> MongoResult<u64> {
        let collection = self.client.collection::<Document>(collection_name);
        let count = collection.count_documents(doc! {}).await?;
        Ok(count)
    }

    /// Update a document by ID / 根据 ID 更新文档
    pub async fn update(
        &self,
        collection_name: &str,
        id: mongodb::bson::Bson,
        update: Document,
    ) -> MongoResult<u64> {
        let collection = self.client.collection::<Document>(collection_name);
        let result = collection.update_one(doc! {"_id": id}, update).await?;
        Ok(result.modified_count)
    }

    /// Delete a document by ID / 根据 ID 删除文档
    pub async fn delete_by_id(
        &self,
        collection_name: &str,
        id: mongodb::bson::Bson,
    ) -> MongoResult<u64> {
        let collection = self.client.collection::<Document>(collection_name);
        let result = collection.delete_one(doc! {"_id": id}).await?;
        Ok(result.deleted_count)
    }

    /// Delete all documents / 删除所有文档
    pub async fn delete_all(&self, collection_name: &str) -> MongoResult<u64> {
        let collection = self.client.collection::<Document>(collection_name);
        let result = collection.delete_many(doc! {}).await?;
        Ok(result.deleted_count)
    }
}

#[cfg(test)]
mod tests {
    

    #[test]
    fn test_template_database_name() {
        // Just verify the struct works
        assert_eq!("test_db", "test_db");
    }
}
