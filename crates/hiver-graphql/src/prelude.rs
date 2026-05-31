//! Prelude — re-exports commonly used types / 重新导出常用类型

pub use crate::context::GraphQLContext;
pub use crate::dataloader::{BatchLoader, DataLoader, DataLoaderRegistry};
pub use crate::error::{GraphQLError, SourceLocation};
pub use crate::resolver::{
    FieldResolver, MutationResolver, QueryResolver, ResolverRegistry, ResolverResult,
    SubscriptionResolver,
};
pub use crate::schema::{IntrospectionResult, Schema, SchemaBuilder, TypeDef, TypeRegistry};
pub use crate::transport::{GraphQLRequest, GraphQLResponse, GraphQLHandler, graphiql_html};

#[cfg(feature = "client")]
pub use crate::client::GraphQlClient;

pub type Result<T> = std::result::Result<T, GraphQLError>;
