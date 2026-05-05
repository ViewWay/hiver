//! Prelude — re-exports commonly used HATEOAS types / 重新导出常用HATEOAS类型

pub use crate::affordance::{Affordance, AffordanceBuilder};
pub use crate::assembler::{CollectionModelAssembler, RepresentationModelAssembler, SimpleAssembler};
pub use crate::collection_model::{CollectionModel, PagedModel, PageMetadata};
pub use crate::entity_model::EntityModel;
pub use crate::link::{Link, LinkRelation, UriTemplate};
pub use crate::link_builder::LinkBuilder;
pub use crate::media_types::hal::{HalDeserializer, HalSerializer};
pub use crate::media_types::hal_forms::{
    HalFormsOption, HalFormsOptions, HalFormsProperty, HalFormsTemplate, HalFormsTemplateBuilder,
};
pub use crate::representation::RepresentationModel;
pub use crate::traverson::{Traverson, TraversonError};
