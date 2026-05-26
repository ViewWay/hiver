//! Representation model assemblers
//! 表示模型装配器
//! Equivalent to Spring HATEOAS RepresentationModelAssembler + CollectionModelAssembler

use crate::collection_model::CollectionModel;
use crate::entity_model::EntityModel;
use crate::link::Link;

/// Assembles a single entity into an EntityModel with links
/// 将单个实体装配为带链接的EntityModel
///
/// Equivalent to Spring HATEOAS `RepresentationModelAssembler<T>`.
pub trait RepresentationModelAssembler<T> {
    /// Convert an entity to an EntityModel with hypermedia links
    fn to_model(&self, entity: &T) -> EntityModel<T>
    where
        T: Clone + serde::Serialize;

    /// Add links to an existing EntityModel (override for custom link logic)
    fn add_links(&self, model: &mut EntityModel<T>, entity: &T)
    where
        T: Clone + serde::Serialize,
    {
        let _ = (model, entity);
    }
}

/// Assembles a collection into a CollectionModel with links
/// 将集合装配为带链接的CollectionModel
///
/// Equivalent to Spring HATEOAS `CollectionModelAssembler<T>`.
pub trait CollectionModelAssembler<T> {
    /// Convert a collection to a CollectionModel with hypermedia links
    fn to_collection_model(&self, entities: &[T]) -> CollectionModel<T>
    where
        T: Clone + serde::Serialize;

    /// Add links to an existing CollectionModel
    fn add_links(&self, model: &mut CollectionModel<T>, entities: &[T])
    where
        T: Clone + serde::Serialize,
    {
        let _ = (model, entities);
    }

    /// Build self link for the collection
    fn collection_link(&self) -> Link {
        Link::new("/").with_rel(crate::link::LinkRelation::Self_)
    }
}

/// Simple assembler that wraps each entity without adding links
/// 简单装配器，包装每个实体而不添加链接
pub struct SimpleAssembler;

impl<T> RepresentationModelAssembler<T> for SimpleAssembler
where
    T: Clone + serde::Serialize,
{
    fn to_model(&self, entity: &T) -> EntityModel<T> {
        EntityModel::from(entity.clone())
    }
}

impl<T> CollectionModelAssembler<T> for SimpleAssembler
where
    T: Clone + serde::Serialize,
{
    fn to_collection_model(&self, entities: &[T]) -> CollectionModel<T> {
        CollectionModel::from(entities.to_vec())
    }
}

/// Typed resource assembler that auto-generates self and collection links.
/// 类型化资源装配器，自动生成 self 和 collection 链接。
pub struct TypedResourceAssembler<T, F>
where
    F: Fn(&T) -> String + Send + Sync,
{
    collection_path: String,
    self_link_fn: F,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, F> TypedResourceAssembler<T, F>
where
    F: Fn(&T) -> String + Send + Sync,
{
    /// Create a new typed assembler.
    pub fn new(collection_path: impl Into<String>, self_link_fn: F) -> Self {
        Self {
            collection_path: collection_path.into(),
            self_link_fn,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, F> RepresentationModelAssembler<T> for TypedResourceAssembler<T, F>
where
    T: Clone + serde::Serialize,
    F: Fn(&T) -> String + Send + Sync,
{
    fn to_model(&self, entity: &T) -> EntityModel<T> {
        let mut model = EntityModel::from(entity.clone());
        let self_href = (self.self_link_fn)(entity);
        model.add_link(Link::new(&self_href).with_rel(crate::link::LinkRelation::Self_));
        model.add_link(
            Link::new(&self.collection_path)
                .with_rel(crate::link::LinkRelation::Custom("collection".into())),
        );
        model
    }
}

impl<T, F> CollectionModelAssembler<T> for TypedResourceAssembler<T, F>
where
    T: Clone + serde::Serialize,
    F: Fn(&T) -> String + Send + Sync,
{
    fn to_collection_model(&self, entities: &[T]) -> CollectionModel<T> {
        let items: Vec<EntityModel<T>> = entities.iter().map(|e| self.to_model(e)).collect();
        let mut collection = CollectionModel::from(items);
        collection.add_link(
            Link::new(&self.collection_path).with_rel(crate::link::LinkRelation::Self_),
        );
        collection
    }

    fn collection_link(&self) -> Link {
        Link::new(&self.collection_path).with_rel(crate::link::LinkRelation::Self_)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, serde::Serialize)]
    struct Item {
        id: u64,
        name: String,
    }

    #[test]
    fn test_typed_assembler_single() {
        let asm = TypedResourceAssembler::new("/api/items", |item: &Item| {
            format!("/api/items/{}", item.id)
        });
        let item = Item { id: 42, name: "test".into() };
        let model = asm.to_model(&item);
        let links = model.links();
        assert!(links.iter().any(|l| l.rel() == &LinkRelation::Self_));
        assert!(links.iter().any(|l| l.rel().to_string() == "collection"));
    }

    #[test]
    fn test_typed_assembler_collection() {
        let asm = TypedResourceAssembler::new("/api/items", |item: &Item| {
            format!("/api/items/{}", item.id)
        });
        let items = vec![
            Item { id: 1, name: "a".into() },
            Item { id: 2, name: "b".into() },
        ];
        let collection = asm.to_collection_model(&items);
        assert!(collection.links().iter().any(|l| l.rel() == &LinkRelation::Self_));
    }

    #[test]
    fn test_simple_assembler() {
        let asm = SimpleAssembler;
        let item = Item { id: 1, name: "test".into() };
        let model: EntityModel<Item> = asm.to_model(&item);
        assert_eq!(model.links().len(), 0);
    }
}
