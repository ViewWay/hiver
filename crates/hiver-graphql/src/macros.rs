//! Helper macros for GraphQL / GraphQL辅助宏

#[macro_export]
macro_rules! graphql_endpoint {
    ($router:expr, $path:expr, $handler:expr) => {{
        let _h = $handler.clone();
        let _p = $path;
        $router
    }};
}

#[macro_export]
macro_rules! gql {
    ($handler:expr, $query:expr) => {{
        $handler.query($query)
    }};
}
