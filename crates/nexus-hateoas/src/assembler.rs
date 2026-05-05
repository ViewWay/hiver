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
