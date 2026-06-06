//! Hiver GraphQL — GraphQL support for the Hiver framework
//! Hiver GraphQL — `Hiver框架的GraphQL支持`

#![warn(missing_docs)]
#![warn(unreachable_pub)]

pub mod context;
pub mod dataloader;
pub mod engine;
pub mod error;
pub mod resolver;
pub mod subscription;

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests;
