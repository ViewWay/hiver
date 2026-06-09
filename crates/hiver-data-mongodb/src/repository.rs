//! MongoDB repository implementation
//! MongoDB 仓储实现

use std::{marker::PhantomData, sync::Arc};

use futures_util::stream::StreamExt;
use mongodb::bson::{Bson, Document, doc};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    client::MongoClient,
    error::{MongoError, MongoResult},
    template::MongoTemplate,
};

/// MongoDB repository base implementation
/// MongoDB 仓储基础实现
///
/// # Type Parameters / 类型参数
///
/// * `T` - Entity type / 实体类型
/// * `ID` - ID type / ID 类型
#[derive(Debug, Clone)]
pub struct MongoRepository<T, ID = Bson>
where
    T: Serialize + DeserializeOwned + Send + Sync,
    ID: Clone + Into<Bson> + Send + Sync,
{
    template: Arc<MongoTemplate>,
    collection_name: Arc<String>,
    _phantom: PhantomData<(T, ID)>,
}

impl<T, ID> MongoRepository<T, ID>
where
    T: Serialize + DeserializeOwned + Send + Sync + Unpin,
    ID: Clone + Into<Bson> + Send + Sync,
{
    /// Create a new repository / 创建新的仓储
    pub fn new(client: MongoClient, collection_name: &str) -> Self {
        Self {
            template: Arc::new(MongoTemplate::new(client.inner().clone(), client.database_name())),
            collection_name: Arc::new(collection_name.to_string()),
            _phantom: PhantomData,
        }
    }

    /// Create from template / 从模板创建
    pub fn from_template(template: MongoTemplate, collection_name: &str) -> Self {
        Self {
            template: Arc::new(template),
            collection_name: Arc::new(collection_name.to_string()),
            _phantom: PhantomData,
        }
    }

    /// Get the collection name / 获取集合名称
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Get the template / 获取模板
    pub fn template(&self) -> &MongoTemplate {
        &self.template
    }

    /// Find documents by filter / 根据过滤器查找文档
    pub async fn find(&self, filter: Document) -> MongoResult<Vec<T>> {
        let collection = self
            .template
            .client()
            .collection::<T>(&self.collection_name);
        let mut cursor = collection.find(filter).await?;

        let mut results = Vec::new();
        while let Some(result) = cursor.next().await {
            results.push(result.map_err(MongoError::from)?);
        }

        Ok(results)
    }

    /// Find one document by filter / 根据过滤器查找单个文档
    pub async fn find_one(&self, filter: Document) -> MongoResult<Option<T>> {
        let collection = self
            .template
            .client()
            .collection::<T>(&self.collection_name);
        let result = collection.find_one(filter).await?;
        Ok(result)
    }

    /// Save an entity / 保存实体
    pub async fn save(&self, entity: &T) -> MongoResult<()> {
        let collection = self.template.client().collection(&self.collection_name);

        let doc = mongodb::bson::to_document(entity)?;
        let id = doc.get("_id");

        if let Some(id_value) = id {
            collection.replace_one(doc! {"_id": id_value}, doc).await?;
        } else {
            collection.insert_one(doc).await?;
        }

        Ok(())
    }

    /// Update documents by filter / 根据过滤器更新文档
    pub async fn update(&self, filter: Document, update: Document) -> MongoResult<u64> {
        let collection = self
            .template
            .client()
            .collection::<Document>(&self.collection_name);
        let result = collection.update_many(filter, update).await?;
        Ok(result.modified_count)
    }

    /// Delete documents by filter / 根据过滤器删除文档
    pub async fn delete(&self, filter: Document) -> MongoResult<u64> {
        let collection = self
            .template
            .client()
            .collection::<Document>(&self.collection_name);
        let result = collection.delete_many(filter).await?;
        Ok(result.deleted_count)
    }

    /// Count documents by filter / 根据过滤器统计文档
    pub async fn count(&self, filter: Document) -> MongoResult<u64> {
        let collection = self
            .template
            .client()
            .collection::<Document>(&self.collection_name);
        let count = collection.count_documents(filter).await?;
        Ok(count)
    }

    /// Check if document exists by filter / 根据过滤器检查文档是否存在
    pub async fn exists(&self, filter: Document) -> MongoResult<bool> {
        let count = self.count(filter).await?;
        Ok(count > 0)
    }
}

/// Builder for creating MongoRepository / 创建 MongoRepository 的构建器
pub struct MongoRepositoryBuilder<T, ID = Bson>
where
    T: Serialize + DeserializeOwned + Send + Sync + Unpin,
    ID: Clone + Into<Bson> + Send + Sync,
{
    client: Option<MongoClient>,
    template: Option<MongoTemplate>,
    collection_name: Option<String>,
    _phantom: PhantomData<(T, ID)>,
}

impl<T, ID> Default for MongoRepositoryBuilder<T, ID>
where
    T: Serialize + DeserializeOwned + Send + Sync + Unpin,
    ID: Clone + Into<Bson> + Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, ID> MongoRepositoryBuilder<T, ID>
where
    T: Serialize + DeserializeOwned + Send + Sync + Unpin,
    ID: Clone + Into<Bson> + Send + Sync,
{
    /// Create a new builder / 创建新的构建器
    pub fn new() -> Self {
        Self {
            client: None,
            template: None,
            collection_name: None,
            _phantom: PhantomData,
        }
    }

    /// Set the client / 设置客户端
    pub fn client(mut self, client: MongoClient) -> Self {
        self.client = Some(client);
        self
    }

    /// Set the template / 设置模板
    pub fn template(mut self, template: MongoTemplate) -> Self {
        self.template = Some(template);
        self
    }

    /// Set the collection name / 设置集合名称
    pub fn collection_name(mut self, name: &str) -> Self {
        self.collection_name = Some(name.to_string());
        self
    }

    /// Build the repository / 构建仓储
    pub fn build(self) -> MongoResult<MongoRepository<T, ID>> {
        let collection_name = self
            .collection_name
            .ok_or_else(|| MongoError::validation("Collection name is required"))?;

        if let Some(template) = self.template {
            Ok(MongoRepository::from_template(template, &collection_name))
        } else if let Some(client) = self.client {
            Ok(MongoRepository::new(client, &collection_name))
        } else {
            Err(MongoError::validation("Either client or template is required"))
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
mod tests {
    use super::*;

    #[test]
    fn test_repository_builder_no_client() {
        // Test that builder fails without client
        let result = MongoRepositoryBuilder::<String, String>::new()
            .collection_name("test_entities")
            .build();

        assert!(result.is_err());
    }
}
