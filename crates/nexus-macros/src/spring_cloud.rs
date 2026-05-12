use proc_macro::TokenStream;

pub fn query(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn native_query(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn read_only(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn modifying(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn jdbc_repository(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn r2dbc_repository(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn mongo_repository(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn redis_hash(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn elasticsearch_repository(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn configuration_properties(_attr: TokenStream, item: TokenStream) -> TokenStream {
    super::spring_stereotype::config(_attr, item)
}

pub fn enable_configuration_properties(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn configuration_properties_scan(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn ignore_unknown_properties(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn default_value(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn nested_configuration_property(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn endpoint_actuator(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn read_operation(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn write_operation(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn delete_operation(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_client(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_get(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_post(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_put(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_delete(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_path(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_query(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_header(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_body(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn circuit_breaker_name(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_timeout(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_retry(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_configuration(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_decoder(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_encoder(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_logger(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_error_decoder(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn feign_options(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn query_map_encoder(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn contract(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn circuit_breaker_config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn time_limiter_config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn bulkhead_config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn retry_config(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn fallback(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn circuit_breaker(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn bulkhead(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn time_limiter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn retry_attr(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn rate_limiter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn request_rate_limiter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn origin_rate_limiter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn user_rate_limiter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn throttling(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn gateway_filter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn gateway_predicate(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn gateway_route(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

pub fn gateway_configuration(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
