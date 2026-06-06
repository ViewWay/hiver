#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::needless_pass_by_value
)]
//! Hiver Macros - Spring Boot Style Procedural Macros
//! Hiver宏 - Spring Boot风格的过程宏
//!
//! # Overview / 概述
//!
//! `hiver-macros` provides Spring Boot-style procedural macros for the Hiver framework.
//!
//! `hiver-macros` 为 Hiver 框架提供 Spring Boot 风格的过程宏。
//!
//! # Example / 示例
//!
//! ```rust,no_run,ignore
//! use hiver_macros::{hiver_main, controller, get};
//!
//! #[hiver_main]
//! struct Application;
//!
//! fn main() -> anyhow::Result<()> {
//!     Application::run()
//! }
//!
//! #[controller]
//! struct DemoController;
//!
//! #[get("/helloworld")]
//! fn hello() -> &'static str {
//!     "Hello World!"
//! }
//! ```

#![allow(missing_docs)]
#![allow(unreachable_pub)]

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::items_after_statements)]
mod tests;

mod bean_register;
mod cache;
mod derive;
mod exception;
mod handler;
mod pre_authorize;
mod routes;
mod scheduled;
mod spring_cloud;
mod spring_di;
mod spring_stereotype;
mod transactional;

use proc_macro::TokenStream;

// ============================================================================
// Handler
// ============================================================================

#[proc_macro_attribute]
pub fn handler(attr: TokenStream, item: TokenStream) -> TokenStream
{
    handler::handler_impl(attr, item)
}

// ============================================================================
// Transactional
// ============================================================================

#[proc_macro_attribute]
pub fn transactional(attr: TokenStream, item: TokenStream) -> TokenStream
{
    transactional::transactional_impl(&attr, item)
}

// ============================================================================
// Derive macros
// ============================================================================

#[proc_macro_derive(FromRequest)]
pub fn from_request_derive(input: TokenStream) -> TokenStream
{
    derive::from_request(input)
}

#[proc_macro_derive(IntoResponse)]
pub fn into_response_derive(input: TokenStream) -> TokenStream
{
    derive::into_response(input)
}

#[proc_macro_derive(Bean)]
pub fn bean_derive(input: TokenStream) -> TokenStream
{
    derive::bean_derive(input)
}

// ============================================================================
// Spring stereotype macros (hiver_main, main, controller, service, etc.)
// ============================================================================

#[proc_macro_attribute]
pub fn hiver_main(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_stereotype::hiver_main(attr, item)
}

#[proc_macro_attribute]
pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_stereotype::main(attr, item)
}

#[proc_macro_attribute]
pub fn controller(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_stereotype::controller(attr, item)
}

#[proc_macro_attribute]
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_stereotype::service(attr, item)
}

#[proc_macro_attribute]
pub fn repository(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_stereotype::repository(attr, item)
}

#[proc_macro_attribute]
pub fn config(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_stereotype::config(attr, item)
}

#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_stereotype::component(attr, item)
}

#[proc_macro_attribute]
pub fn autowired(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_stereotype::autowired(attr, item)
}

#[proc_macro_attribute]
pub fn configuration(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_stereotype::configuration(attr, item)
}

#[proc_macro_attribute]
pub fn bean(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_stereotype::bean(attr, item)
}

#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_stereotype::profile(attr, item)
}

#[proc_macro_attribute]
pub fn value(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_stereotype::value(attr, item)
}

// ============================================================================
// Route macros
// ============================================================================

#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream
{
    routes::get(attr, item)
}

#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream
{
    routes::post(attr, item)
}

#[proc_macro_attribute]
pub fn put(attr: TokenStream, item: TokenStream) -> TokenStream
{
    routes::put(attr, item)
}

#[proc_macro_attribute]
pub fn delete(attr: TokenStream, item: TokenStream) -> TokenStream
{
    routes::delete(attr, item)
}

#[proc_macro_attribute]
pub fn patch(attr: TokenStream, item: TokenStream) -> TokenStream
{
    routes::patch(attr, item)
}

#[proc_macro_attribute]
pub fn head(attr: TokenStream, item: TokenStream) -> TokenStream
{
    routes::head(attr, item)
}

#[proc_macro_attribute]
pub fn options(attr: TokenStream, item: TokenStream) -> TokenStream
{
    routes::options(attr, item)
}

#[proc_macro_attribute]
pub fn trace(attr: TokenStream, item: TokenStream) -> TokenStream
{
    routes::trace(attr, item)
}

#[proc_macro_attribute]
pub fn request_mapping(attr: TokenStream, item: TokenStream) -> TokenStream
{
    routes::request_mapping(attr, item)
}

#[proc_macro_attribute]
pub fn cross_origin(attr: TokenStream, item: TokenStream) -> TokenStream
{
    routes::cross_origin(attr, item)
}

#[proc_macro_attribute]
pub fn trace_method(attr: TokenStream, item: TokenStream) -> TokenStream
{
    routes::trace_method(attr, item)
}

#[proc_macro_attribute]
pub fn patch_route(attr: TokenStream, item: TokenStream) -> TokenStream
{
    routes::patch_route(attr, item)
}

#[proc_macro_attribute]
pub fn rest_controller(attr: TokenStream, item: TokenStream) -> TokenStream
{
    routes::rest_controller(attr, item)
}

#[proc_macro_attribute]
pub fn controller_view(attr: TokenStream, item: TokenStream) -> TokenStream
{
    routes::controller_view(attr, item)
}

#[proc_macro_attribute]
pub fn get_mapping(attr: TokenStream, item: TokenStream) -> TokenStream
{
    routes::get_mapping(attr, item)
}

#[proc_macro_attribute]
pub fn post_mapping(attr: TokenStream, item: TokenStream) -> TokenStream
{
    routes::post_mapping(attr, item)
}

#[proc_macro_attribute]
pub fn put_mapping(attr: TokenStream, item: TokenStream) -> TokenStream
{
    routes::put_mapping(attr, item)
}

#[proc_macro_attribute]
pub fn delete_mapping(attr: TokenStream, item: TokenStream) -> TokenStream
{
    routes::delete_mapping(attr, item)
}

#[proc_macro_attribute]
pub fn patch_mapping(attr: TokenStream, item: TokenStream) -> TokenStream
{
    routes::patch_mapping(attr, item)
}

// ============================================================================
// Scheduled / async / logging macros
// ============================================================================

#[proc_macro_attribute]
pub fn scheduled(attr: TokenStream, item: TokenStream) -> TokenStream
{
    scheduled::scheduled(attr, item)
}

#[proc_macro_attribute]
pub fn async_fn(attr: TokenStream, item: TokenStream) -> TokenStream
{
    scheduled::async_fn(attr, item)
}

#[proc_macro_attribute]
pub fn slf4j(attr: TokenStream, item: TokenStream) -> TokenStream
{
    scheduled::slf4j(attr, item)
}

#[proc_macro_attribute]
pub fn logger(attr: TokenStream, item: TokenStream) -> TokenStream
{
    scheduled::logger(attr, item)
}

// ============================================================================
// Cache macros
// ============================================================================

#[proc_macro_attribute]
pub fn cacheable(attr: TokenStream, item: TokenStream) -> TokenStream
{
    cache::cacheable(attr, item)
}

#[proc_macro_attribute]
pub fn cache_evict(attr: TokenStream, item: TokenStream) -> TokenStream
{
    cache::cache_evict(attr, item)
}

#[proc_macro_attribute]
pub fn cache_put(attr: TokenStream, item: TokenStream) -> TokenStream
{
    cache::cache_put(attr, item)
}

#[proc_macro_attribute]
pub fn cache_config(attr: TokenStream, item: TokenStream) -> TokenStream
{
    cache::cache_config(attr, item)
}

#[proc_macro_attribute]
pub fn caching(attr: TokenStream, item: TokenStream) -> TokenStream
{
    cache::caching(attr, item)
}

// ============================================================================
// Spring DI macros (conditional, enable, param extraction, lifecycle, etc.)
// ============================================================================

#[proc_macro_attribute]
pub fn conditional_on_class(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::conditional_on_class(attr, item)
}

#[proc_macro_attribute]
pub fn conditional_on_property(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::conditional_on_property(attr, item)
}

#[proc_macro_attribute]
pub fn conditional_on_missing_bean(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::conditional_on_missing_bean(attr, item)
}

#[proc_macro_attribute]
pub fn enable_auto_config(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::enable_auto_config(attr, item)
}

#[proc_macro_attribute]
pub fn enable_caching(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::enable_caching(attr, item)
}

#[proc_macro_attribute]
pub fn enable_scheduling(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::enable_scheduling(attr, item)
}

#[proc_macro_attribute]
pub fn enable_async_exec(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::enable_async_exec(attr, item)
}

#[proc_macro_attribute]
pub fn enable_transaction_management(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::enable_transaction_management(attr, item)
}

#[proc_macro_attribute]
pub fn enable_validating(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::enable_validating(attr, item)
}

#[proc_macro_attribute]
pub fn enable_web_mvc(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::enable_web_mvc(attr, item)
}

#[proc_macro_attribute]
pub fn import(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::import(attr, item)
}

#[proc_macro_attribute]
pub fn component_scan(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::component_scan(attr, item)
}

#[proc_macro_attribute]
pub fn path_variable(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::path_variable(attr, item)
}

#[proc_macro_attribute]
pub fn request_param(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::request_param(attr, item)
}

#[proc_macro_attribute]
pub fn request_header_attr(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::request_header_attr(attr, item)
}

#[proc_macro_attribute]
pub fn cookie_value(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::cookie_value(attr, item)
}

#[proc_macro_attribute]
pub fn request_body(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::request_body(attr, item)
}

#[proc_macro_attribute]
pub fn model_attribute(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::model_attribute(attr, item)
}

#[proc_macro_attribute]
pub fn request_attribute(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::request_attribute(attr, item)
}

#[proc_macro_attribute]
pub fn matrix_variable(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::matrix_variable(attr, item)
}

#[proc_macro_attribute]
pub fn session_attribute(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::session_attribute(attr, item)
}

#[proc_macro_attribute]
pub fn post_construct(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::post_construct(attr, item)
}

#[proc_macro_attribute]
pub fn pre_destroy(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::pre_destroy(attr, item)
}

#[proc_macro_attribute]
pub fn qualifier(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::qualifier(attr, item)
}

#[proc_macro_attribute]
pub fn primary(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::primary(attr, item)
}

#[proc_macro_attribute]
pub fn lazy_bean(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::lazy_bean(attr, item)
}

#[proc_macro_attribute]
pub fn lookup(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::lookup(attr, item)
}

#[proc_macro_attribute]
pub fn scope_prototype(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::scope_prototype(attr, item)
}

#[proc_macro_attribute]
pub fn scope_singleton(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::scope_singleton(attr, item)
}

#[proc_macro_attribute]
pub fn request_scope(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::request_scope(attr, item)
}

#[proc_macro_attribute]
pub fn session_scope(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::session_scope(attr, item)
}

#[proc_macro_attribute]
pub fn application_scope(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::application_scope(attr, item)
}

#[proc_macro_attribute]
pub fn transactional_event_listener(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::transactional_event_listener(attr, item)
}

#[proc_macro_attribute]
pub fn event_listener(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::event_listener(attr, item)
}

#[proc_macro_attribute]
pub fn retryable(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::retryable(attr, item)
}

#[proc_macro_attribute]
pub fn recover(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::recover(attr, item)
}

#[proc_macro_attribute]
pub fn valid(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::valid(attr, item)
}

#[proc_macro_attribute]
pub fn validated(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::validated(attr, item)
}

#[proc_macro_attribute]
pub fn not_null(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::not_null(attr, item)
}

#[proc_macro_attribute]
pub fn not_blank(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::not_blank(attr, item)
}

#[proc_macro_attribute]
pub fn not_empty(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::not_empty(attr, item)
}

#[proc_macro_attribute]
pub fn size(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::size(attr, item)
}

#[proc_macro_attribute]
pub fn length(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::length(attr, item)
}

#[proc_macro_attribute]
pub fn min(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::min(attr, item)
}

#[proc_macro_attribute]
pub fn max(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::max(attr, item)
}

#[proc_macro_attribute]
pub fn decimal_min(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::decimal_min(attr, item)
}

#[proc_macro_attribute]
pub fn decimal_max(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::decimal_max(attr, item)
}

#[proc_macro_attribute]
pub fn email(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::email(attr, item)
}

#[proc_macro_attribute]
pub fn pattern(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::pattern(attr, item)
}

#[proc_macro_attribute]
pub fn url(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::url(attr, item)
}

#[proc_macro_attribute]
pub fn assert_true(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::assert_true(attr, item)
}

#[proc_macro_attribute]
pub fn assert_false(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::assert_false(attr, item)
}

#[proc_macro_attribute]
pub fn past(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::past(attr, item)
}

#[proc_macro_attribute]
pub fn future(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::future(attr, item)
}

#[proc_macro_attribute]
pub fn range(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::range(attr, item)
}

#[proc_macro_attribute]
pub fn negative(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::negative(attr, item)
}

#[proc_macro_attribute]
pub fn positive(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::positive(attr, item)
}

#[proc_macro_attribute]
pub fn secured(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::secured(attr, item)
}

#[proc_macro_attribute]
pub fn pre_authorize(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::pre_authorize(attr, item)
}

#[proc_macro_attribute]
pub fn post_authorize(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::post_authorize(attr, item)
}

#[proc_macro_attribute]
pub fn pre_filter(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::pre_filter(attr, item)
}

#[proc_macro_attribute]
pub fn post_filter(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::post_filter(attr, item)
}

#[proc_macro_attribute]
pub fn roles_allowed(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::roles_allowed(attr, item)
}

#[proc_macro_attribute]
pub fn permit_all(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::permit_all(attr, item)
}

#[proc_macro_attribute]
pub fn deny_all(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::deny_all(attr, item)
}

#[proc_macro_attribute]
pub fn anonymous(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::anonymous(attr, item)
}

#[proc_macro_attribute]
pub fn require_role(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::require_role(attr, item)
}

#[proc_macro_attribute]
pub fn response_status(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::response_status(attr, item)
}

#[proc_macro_attribute]
pub fn bad_request(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::bad_request(attr, item)
}

#[proc_macro_attribute]
pub fn unauthorized(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::unauthorized(attr, item)
}

#[proc_macro_attribute]
pub fn forbidden(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::forbidden(attr, item)
}

#[proc_macro_attribute]
pub fn not_found(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::not_found(attr, item)
}

#[proc_macro_attribute]
pub fn internal_server_error(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::internal_server_error(attr, item)
}

#[proc_macro_attribute]
pub fn service_unavailable(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::service_unavailable(attr, item)
}

#[proc_macro_attribute]
pub fn cron(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::cron(attr, item)
}

#[proc_macro_attribute]
pub fn fixed_rate(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::fixed_rate(attr, item)
}

#[proc_macro_attribute]
pub fn fixed_delay(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::fixed_delay(attr, item)
}

#[proc_macro_attribute]
pub fn initial_delay(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_di::initial_delay(attr, item)
}

// ============================================================================
// Spring Cloud / JPA / Feign / Resilience / Gateway macros
// ============================================================================

#[proc_macro_attribute]
pub fn query(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::query(attr, item)
}

#[proc_macro_attribute]
pub fn native_query(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::native_query(attr, item)
}

#[proc_macro_attribute]
pub fn read_only(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::read_only(attr, item)
}

#[proc_macro_attribute]
pub fn modifying(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::modifying(attr, item)
}

#[proc_macro_attribute]
pub fn jdbc_repository(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::jdbc_repository(attr, item)
}

#[proc_macro_attribute]
pub fn r2dbc_repository(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::r2dbc_repository(attr, item)
}

#[proc_macro_attribute]
pub fn mongo_repository(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::mongo_repository(attr, item)
}

#[proc_macro_attribute]
pub fn redis_hash(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::redis_hash(attr, item)
}

#[proc_macro_attribute]
pub fn elasticsearch_repository(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::elasticsearch_repository(attr, item)
}

#[proc_macro_attribute]
pub fn configuration_properties(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::configuration_properties(attr, item)
}

#[proc_macro_attribute]
pub fn enable_configuration_properties(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::enable_configuration_properties(attr, item)
}

#[proc_macro_attribute]
pub fn configuration_properties_scan(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::configuration_properties_scan(attr, item)
}

#[proc_macro_attribute]
pub fn ignore_unknown_properties(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::ignore_unknown_properties(attr, item)
}

#[proc_macro_attribute]
pub fn default_value(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::default_value(attr, item)
}

#[proc_macro_attribute]
pub fn nested_configuration_property(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::nested_configuration_property(attr, item)
}

#[proc_macro_attribute]
pub fn endpoint_actuator(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::endpoint_actuator(attr, item)
}

#[proc_macro_attribute]
pub fn read_operation(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::read_operation(attr, item)
}

#[proc_macro_attribute]
pub fn write_operation(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::write_operation(attr, item)
}

#[proc_macro_attribute]
pub fn delete_operation(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::delete_operation(attr, item)
}

#[proc_macro_attribute]
pub fn feign_client(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::feign_client(attr, item)
}

#[proc_macro_attribute]
pub fn feign_get(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::feign_get(attr, item)
}

#[proc_macro_attribute]
pub fn feign_post(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::feign_post(attr, item)
}

#[proc_macro_attribute]
pub fn feign_put(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::feign_put(attr, item)
}

#[proc_macro_attribute]
pub fn feign_delete(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::feign_delete(attr, item)
}

#[proc_macro_attribute]
pub fn feign_path(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::feign_path(attr, item)
}

#[proc_macro_attribute]
pub fn feign_query(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::feign_query(attr, item)
}

#[proc_macro_attribute]
pub fn feign_header(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::feign_header(attr, item)
}

#[proc_macro_attribute]
pub fn feign_body(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::feign_body(attr, item)
}

#[proc_macro_attribute]
pub fn circuit_breaker_name(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::circuit_breaker_name(attr, item)
}

#[proc_macro_attribute]
pub fn feign_timeout(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::feign_timeout(attr, item)
}

#[proc_macro_attribute]
pub fn feign_retry(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::feign_retry(attr, item)
}

#[proc_macro_attribute]
pub fn feign_configuration(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::feign_configuration(attr, item)
}

#[proc_macro_attribute]
pub fn feign_decoder(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::feign_decoder(attr, item)
}

#[proc_macro_attribute]
pub fn feign_encoder(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::feign_encoder(attr, item)
}

#[proc_macro_attribute]
pub fn feign_logger(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::feign_logger(attr, item)
}

#[proc_macro_attribute]
pub fn feign_error_decoder(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::feign_error_decoder(attr, item)
}

#[proc_macro_attribute]
pub fn feign_options(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::feign_options(attr, item)
}

#[proc_macro_attribute]
pub fn query_map_encoder(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::query_map_encoder(attr, item)
}

#[proc_macro_attribute]
pub fn contract(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::contract(attr, item)
}

#[proc_macro_attribute]
pub fn circuit_breaker_config(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::circuit_breaker_config(attr, item)
}

#[proc_macro_attribute]
pub fn time_limiter_config(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::time_limiter_config(attr, item)
}

#[proc_macro_attribute]
pub fn bulkhead_config(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::bulkhead_config(attr, item)
}

#[proc_macro_attribute]
pub fn retry_config(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::retry_config(attr, item)
}

#[proc_macro_attribute]
pub fn fallback(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::fallback(attr, item)
}

#[proc_macro_attribute]
pub fn circuit_breaker(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::circuit_breaker(attr, item)
}

#[proc_macro_attribute]
pub fn bulkhead(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::bulkhead(attr, item)
}

#[proc_macro_attribute]
pub fn time_limiter(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::time_limiter(attr, item)
}

#[proc_macro_attribute]
pub fn retry_attr(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::retry_attr(attr, item)
}

#[proc_macro_attribute]
pub fn rate_limiter(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::rate_limiter(attr, item)
}

#[proc_macro_attribute]
pub fn request_rate_limiter(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::request_rate_limiter(attr, item)
}

#[proc_macro_attribute]
pub fn origin_rate_limiter(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::origin_rate_limiter(attr, item)
}

#[proc_macro_attribute]
pub fn user_rate_limiter(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::user_rate_limiter(attr, item)
}

#[proc_macro_attribute]
pub fn throttling(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::throttling(attr, item)
}

#[proc_macro_attribute]
pub fn gateway_filter(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::gateway_filter(attr, item)
}

#[proc_macro_attribute]
pub fn gateway_predicate(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::gateway_predicate(attr, item)
}

#[proc_macro_attribute]
pub fn gateway_route(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::gateway_route(attr, item)
}

#[proc_macro_attribute]
pub fn gateway_configuration(attr: TokenStream, item: TokenStream) -> TokenStream
{
    spring_cloud::gateway_configuration(attr, item)
}

// ============================================================================
// Exception handling macros
// ============================================================================

#[proc_macro_attribute]
pub fn controller_advice(attr: TokenStream, item: TokenStream) -> TokenStream
{
    exception::controller_advice(attr, item)
}

#[proc_macro_attribute]
pub fn rest_controller_advice(attr: TokenStream, item: TokenStream) -> TokenStream
{
    exception::rest_controller_advice(attr, item)
}

#[proc_macro_attribute]
pub fn exception_handler(attr: TokenStream, item: TokenStream) -> TokenStream
{
    exception::exception_handler(attr, item)
}
