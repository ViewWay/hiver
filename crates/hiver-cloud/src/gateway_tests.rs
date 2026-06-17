use std::sync::Arc;
use std::time::Duration;

use super::{
    Filter, Gateway, GatewayCbState, GatewayCircuitBreaker, GatewayConfig,
    GatewayRequest, GatewayResponse, GatewayRoute, GatewayRouter, InMemoryRouteLocator,
    Predicate, Route, RouteLocator, SimpleGateway,
    TokenBucketRateLimiter, glob_match,
};

#[tokio::test]
async fn test_gateway_route()
{
    let route = GatewayRoute::new("test", "/api", "http://backend:8080");
    assert_eq!(route.id, "test");
    assert_eq!(route.path, "/api");
}

#[tokio::test]
async fn test_simple_gateway()
{
    let gateway = SimpleGateway::new();
    let route = GatewayRoute::new("users", "/users", "http://user-service");

    gateway.add_route(route).await.unwrap();

    let routes = gateway.routes().await;
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].id, "users");
}

// --- New enhanced types tests ---

#[test]
fn test_predicate_path()
{
    let pred = Predicate::Path("/api".to_string());
    let req = GatewayRequest::new(http::Method::GET, "/api/users");
    assert!(pred.matches(&req));

    let req2 = GatewayRequest::new(http::Method::GET, "/other");
    assert!(!pred.matches(&req2));
}

#[test]
fn test_predicate_method()
{
    let pred = Predicate::Method(vec!["GET".to_string(), "POST".to_string()]);
    let req_get = GatewayRequest::new(http::Method::GET, "/any");
    assert!(pred.matches(&req_get));

    let req_delete = GatewayRequest::new(http::Method::DELETE, "/any");
    assert!(!pred.matches(&req_delete));
}

#[test]
fn test_predicate_header()
{
    let pred = Predicate::Header("X-Custom".to_string(), ".*".to_string());
    let mut req = GatewayRequest::new(http::Method::GET, "/any");
    req.headers
        .insert("X-Custom".to_string(), "value".to_string());
    assert!(pred.matches(&req));

    let req2 = GatewayRequest::new(http::Method::GET, "/any");
    assert!(!pred.matches(&req2));
}

#[test]
fn test_predicate_query()
{
    let pred = Predicate::Query("page".to_string());
    let mut req = GatewayRequest::new(http::Method::GET, "/search");
    req.query = Some("page=1&size=10".to_string());
    assert!(pred.matches(&req));

    let req2 = GatewayRequest::new(http::Method::GET, "/search");
    assert!(!pred.matches(&req2));
}

#[test]
fn test_predicate_weight()
{
    let pred = Predicate::Weight(50);
    let req = GatewayRequest::new(http::Method::GET, "/any");
    // Weight predicates always return true at request level
    assert!(pred.matches(&req));
}

#[test]
fn test_filter_add_header_to_response()
{
    let filter = Filter::AddHeader("X-Custom".to_string(), "value".to_string());
    let mut resp = GatewayResponse::new(http::StatusCode::OK);
    filter.apply_to_response(&mut resp);
    assert_eq!(resp.headers.get("X-Custom"), Some(&"value".to_string()));
}

#[test]
fn test_filter_remove_header_from_request()
{
    let filter = Filter::RemoveHeader("Authorization".to_string());
    let mut req = GatewayRequest::new(http::Method::GET, "/api");
    req.headers
        .insert("Authorization".to_string(), "Bearer token".to_string());
    filter.apply_to_request(&mut req);
    assert!(!req.headers.contains_key("Authorization"));
}

#[test]
fn test_filter_rewrite_path()
{
    let filter = Filter::RewritePath("/api/v1".to_string(), "/v2".to_string());
    let mut req = GatewayRequest::new(http::Method::GET, "/api/v1/users");
    filter.apply_to_request(&mut req);
    assert_eq!(req.path, "/v2/users");
}

#[test]
fn test_filter_rewrite_path_no_match()
{
    let filter = Filter::RewritePath("/api/v1".to_string(), "/v2".to_string());
    let mut req = GatewayRequest::new(http::Method::GET, "/other/path");
    filter.apply_to_request(&mut req);
    assert_eq!(req.path, "/other/path");
}

#[test]
fn test_route_builder()
{
    let route = Route::new("users", "http://user-service:8080")
        .predicate(Predicate::Path("/users".to_string()))
        .predicate(Predicate::Method(vec!["GET".to_string()]))
        .filter(Filter::AddHeader("X-Source".to_string(), "gateway".to_string()))
        .filter(Filter::RateLimit(100))
        .order(1);

    assert_eq!(route.id, "users");
    assert_eq!(route.uri, "http://user-service:8080");
    assert_eq!(route.predicates.len(), 2);
    assert_eq!(route.filters.len(), 2);
    assert_eq!(route.order, 1);
}

#[test]
fn test_route_matches()
{
    let route = Route::new("api", "http://backend:8080")
        .predicate(Predicate::Path("/api".to_string()))
        .predicate(Predicate::Method(vec!["GET".to_string()]));

    let req = GatewayRequest::new(http::Method::GET, "/api/users");
    assert!(route.matches(&req));

    let req_post = GatewayRequest::new(http::Method::POST, "/api/users");
    assert!(!route.matches(&req_post));

    let req_other = GatewayRequest::new(http::Method::GET, "/other");
    assert!(!route.matches(&req_other));
}

#[tokio::test]
async fn test_in_memory_route_locator()
{
    let locator = InMemoryRouteLocator::new();

    locator
        .add_route(
            Route::new("route-a", "http://a:8080")
                .predicate(Predicate::Path("/a".to_string()))
                .order(2),
        )
        .await;
    locator
        .add_route(
            Route::new("route-b", "http://b:8080")
                .predicate(Predicate::Path("/b".to_string()))
                .order(1),
        )
        .await;

    let routes = locator.get_routes().await;
    assert_eq!(routes.len(), 2);
    // Should be sorted by order
    assert_eq!(routes[0].id, "route-b");
    assert_eq!(routes[1].id, "route-a");

    assert!(locator.remove_route("route-a").await);
    let routes = locator.get_routes().await;
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].id, "route-b");

    assert!(!locator.remove_route("nonexistent").await);
}

#[tokio::test]
async fn test_gateway_router_match()
{
    let locator = Arc::new(InMemoryRouteLocator::new());
    locator
        .add_route(
            Route::new("users", "http://user-service:8080")
                .predicate(Predicate::Path("/users".to_string()))
                .filter(Filter::AddHeader("X-Routed".to_string(), "users".to_string())),
        )
        .await;
    locator
        .add_route(
            Route::new("orders", "http://order-service:8080")
                .predicate(Predicate::Path("/orders".to_string()))
                .order(1),
        )
        .await;

    let router = GatewayRouter::new(locator);

    let req = GatewayRequest::new(http::Method::GET, "/users/123");
    let matched = router.match_route(&req).await;
    assert!(matched.is_some());
    assert_eq!(matched.unwrap().id, "users");

    let req2 = GatewayRequest::new(http::Method::GET, "/orders/456");
    let matched2 = router.match_route(&req2).await;
    assert!(matched2.is_some());
    assert_eq!(matched2.unwrap().id, "orders");

    let req3 = GatewayRequest::new(http::Method::GET, "/unknown");
    let matched3 = router.match_route(&req3).await;
    assert!(matched3.is_none());
}

#[tokio::test]
async fn test_gateway_router_apply_filters()
{
    let locator = Arc::new(InMemoryRouteLocator::new());
    locator
        .add_route(
            Route::new("api", "http://backend:8080")
                .predicate(Predicate::Path("/api".to_string()))
                .filter(Filter::RemoveHeader("Secret".to_string()))
                .filter(Filter::RewritePath("/api/v1".to_string(), "/v2".to_string())),
        )
        .await;

    let config = GatewayConfig::with_defaults()
        .global_filter(Filter::AddHeader("X-Gateway".to_string(), "hiver".to_string()));

    let router = GatewayRouter::with_config(locator, config);

    let mut req = GatewayRequest::new(http::Method::GET, "/api/v1/resource");
    req.headers
        .insert("Secret".to_string(), "should-be-removed".to_string());

    let target = router.route(&mut req).await.unwrap();
    assert_eq!(target, Some("http://backend:8080".to_string()));
    assert_eq!(req.path, "/v2/resource");
    assert!(!req.headers.contains_key("Secret"));
}

#[test]
fn test_gateway_config_with_defaults()
{
    let config = GatewayConfig::with_defaults();
    assert_eq!(config.global_filters.len(), 1);
    assert!(config.default_routes.is_empty());
}

#[tokio::test]
async fn test_gateway_router_default_routes()
{
    let locator = Arc::new(InMemoryRouteLocator::new());
    let config = GatewayConfig::new().default_route(
        Route::new("fallback", "http://fallback:8080")
            .predicate(Predicate::Path("/".to_string()))
            .order(999),
    );

    let router = GatewayRouter::with_config(locator, config);
    let req = GatewayRequest::new(http::Method::GET, "/");
    let matched = router.match_route(&req).await;
    assert!(matched.is_some());
    assert_eq!(matched.unwrap().id, "fallback");
}

#[test]
fn test_filter_roundtrip_on_request_and_response()
{
    let filters = vec![
        Filter::AddHeader("X-Added".to_string(), "yes".to_string()),
        Filter::RemoveHeader("X-Removed".to_string()),
    ];

    let mut req = GatewayRequest::new(http::Method::GET, "/api");
    req.headers
        .insert("X-Removed".to_string(), "gone".to_string());

    let mut resp = GatewayResponse::new(http::StatusCode::OK);

    for f in &filters
    {
        f.apply_to_request(&mut req);
        f.apply_to_response(&mut resp);
    }

    assert!(!req.headers.contains_key("X-Removed"));
    assert_eq!(resp.headers.get("X-Added"), Some(&"yes".to_string()));
}

// =======================================================================
// TokenBucketRateLimiter tests
// =======================================================================

#[test]
fn test_rate_limiter_burst_allows_initial_burst()
{
    // 5 tokens/sec, burst of 10
    let limiter = TokenBucketRateLimiter::new(5, 10);
    assert_eq!(limiter.available_tokens(), 10);

    // Should be able to acquire all 10 burst tokens
    let mut acquired = 0;
    for _ in 0..15
    {
        if limiter.try_acquire()
        {
            acquired += 1;
        }
    }
    assert_eq!(acquired, 10, "Burst should allow exactly 10 requests");
    assert_eq!(limiter.available_tokens(), 0);
}

#[test]
fn test_rate_limiter_rejects_after_burst_exhausted()
{
    let limiter = TokenBucketRateLimiter::new(1, 3);

    // Consume all 3 burst tokens
    assert!(limiter.try_acquire());
    assert!(limiter.try_acquire());
    assert!(limiter.try_acquire());

    // 4th should fail immediately (no time for refill)
    assert!(!limiter.try_acquire());
}

#[test]
fn test_rate_limiter_debug_format()
{
    let limiter = TokenBucketRateLimiter::new(10, 50);
    let debug_str = format!("{:?}", limiter);
    assert!(debug_str.contains("TokenBucketRateLimiter"));
    assert!(debug_str.contains("available: 50"));
    assert!(debug_str.contains("max_tokens: 50"));
}

#[test]
fn test_rate_limiter_available_tokens_after_consume()
{
    let limiter = TokenBucketRateLimiter::new(100, 5);
    assert_eq!(limiter.available_tokens(), 5);

    limiter.try_acquire();
    limiter.try_acquire();
    assert_eq!(limiter.available_tokens(), 3);
}

#[test]
fn test_rate_limiter_zero_rate_only_burst()
{
    // Zero refill rate: only the initial burst is available.
    let limiter = TokenBucketRateLimiter::new(0, 2);
    assert!(limiter.try_acquire());
    assert!(limiter.try_acquire());
    assert!(!limiter.try_acquire());
}

// =======================================================================
// GatewayCircuitBreaker tests
// =======================================================================

#[test]
fn test_cb_starts_closed()
{
    let cb = GatewayCircuitBreaker::new(3, 2, Duration::from_secs(30));
    assert_eq!(cb.state(), GatewayCbState::Closed);
    assert!(cb.allow_request());
}

#[test]
fn test_cb_closed_to_open_transition()
{
    let cb = GatewayCircuitBreaker::new(3, 2, Duration::from_secs(30));
    assert_eq!(cb.state(), GatewayCbState::Closed);

    // 3 failures should trigger the transition to Open.
    cb.record_failure();
    assert_eq!(cb.state(), GatewayCbState::Closed);
    cb.record_failure();
    assert_eq!(cb.state(), GatewayCbState::Closed);
    cb.record_failure();
    // After the 3rd failure, threshold is met → Open.
    assert_eq!(cb.state(), GatewayCbState::Open);

    // Requests should be rejected.
    assert!(!cb.allow_request());
}

#[test]
fn test_cb_open_to_half_open_after_timeout()
{
    // Very short timeout for testing.
    let cb = GatewayCircuitBreaker::new(1, 1, Duration::from_millis(10));
    cb.record_failure(); // 1 failure → Open.
    assert_eq!(cb.state(), GatewayCbState::Open);

    // Wait for timeout to elapse.
    std::thread::sleep(Duration::from_millis(20));

    // allow_request should transition to HalfOpen and return true.
    assert!(cb.allow_request());
    assert_eq!(cb.state(), GatewayCbState::HalfOpen);
}

#[test]
fn test_cb_half_open_to_closed_on_success_threshold()
{
    let cb = GatewayCircuitBreaker::new(1, 2, Duration::from_millis(10));
    // Trip it open.
    cb.record_failure();
    assert_eq!(cb.state(), GatewayCbState::Open);

    // Wait for timeout → HalfOpen.
    std::thread::sleep(Duration::from_millis(20));
    assert!(cb.allow_request());
    assert_eq!(cb.state(), GatewayCbState::HalfOpen);

    // Record 2 successes → should close.
    cb.record_success();
    assert_eq!(cb.state(), GatewayCbState::HalfOpen);
    cb.record_success();
    assert_eq!(cb.state(), GatewayCbState::Closed);
}

#[test]
fn test_cb_half_open_to_open_on_failure()
{
    let cb = GatewayCircuitBreaker::new(1, 2, Duration::from_millis(10));
    cb.record_failure(); // → Open
    std::thread::sleep(Duration::from_millis(20));
    assert!(cb.allow_request()); // → HalfOpen
    assert_eq!(cb.state(), GatewayCbState::HalfOpen);

    // A single failure in HalfOpen should immediately re-open.
    cb.record_failure();
    assert_eq!(cb.state(), GatewayCbState::Open);
}

#[test]
fn test_cb_success_resets_failure_count_in_closed()
{
    let cb = GatewayCircuitBreaker::new(3, 1, Duration::from_secs(30));
    cb.record_failure();
    cb.record_failure();
    // 2 failures so far, not yet open.
    assert_eq!(cb.state(), GatewayCbState::Closed);

    // A success resets the counter.
    cb.record_success();

    // Now it should take 3 more failures to open.
    cb.record_failure();
    cb.record_failure();
    assert_eq!(cb.state(), GatewayCbState::Closed);
}

#[test]
fn test_cb_full_lifecycle()
{
    // Closed → Open → HalfOpen → Closed
    let cb = GatewayCircuitBreaker::new(2, 1, Duration::from_millis(10));

    // Closed phase
    assert_eq!(cb.state(), GatewayCbState::Closed);
    cb.record_failure();
    cb.record_failure();
    assert_eq!(cb.state(), GatewayCbState::Open);

    // Wait for timeout → HalfOpen
    std::thread::sleep(Duration::from_millis(20));
    assert!(cb.allow_request());
    assert_eq!(cb.state(), GatewayCbState::HalfOpen);

    // Success in HalfOpen → Closed
    cb.record_success();
    assert_eq!(cb.state(), GatewayCbState::Closed);
}

#[test]
fn test_cb_debug_format()
{
    let cb = GatewayCircuitBreaker::new(5, 3, Duration::from_secs(10));
    let debug = format!("{:?}", cb);
    assert!(debug.contains("GatewayCircuitBreaker"));
    assert!(debug.contains("state: Closed"));
}

// =======================================================================
// Filter::is_infrastructure_filter tests
// =======================================================================

#[test]
fn test_filter_is_infrastructure()
{
    assert!(Filter::RateLimit(100).is_infrastructure_filter());
    assert!(Filter::CircuitBreaker("svc".to_string()).is_infrastructure_filter());
    assert!(!Filter::AddHeader("k".to_string(), "v".to_string()).is_infrastructure_filter());
    assert!(!Filter::RemoveHeader("k".to_string()).is_infrastructure_filter());
    assert!(!Filter::RewritePath("/a".to_string(), "/b".to_string()).is_infrastructure_filter());
}

// =======================================================================
// Integration: GatewayRouter + RateLimiter + CircuitBreaker
// =======================================================================

#[tokio::test]
async fn test_router_rate_limit_rejects_excess()
{
    let locator = Arc::new(InMemoryRouteLocator::new());
    locator
        .add_route(
            Route::new("api", "http://backend:8080")
                .predicate(Predicate::Path("/api".to_string()))
                .filter(Filter::RateLimit(10)), // key = "10"
        )
        .await;

    // Rate limiter with burst of 3.
    let limiter = Arc::new(TokenBucketRateLimiter::new(10, 3));
    let router = GatewayRouter::new(locator).with_rate_limiter("10", limiter);

    // First 3 requests should pass.
    for i in 0..3
    {
        let mut req = GatewayRequest::new(http::Method::GET, "/api/test");
        let result = router.route(&mut req).await;
        assert!(result.is_ok(), "request {} should be accepted", i);
    }

    // 4th request should be rate-limited.
    let mut req = GatewayRequest::new(http::Method::GET, "/api/test");
    let result = router.route(&mut req).await;
    assert!(result.is_err());
    let resp = result.unwrap_err();
    assert_eq!(resp.status, http::StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn test_router_circuit_breaker_rejects_when_open()
{
    let locator = Arc::new(InMemoryRouteLocator::new());
    locator
        .add_route(
            Route::new("svc", "http://svc:8080")
                .predicate(Predicate::Path("/svc".to_string()))
                .filter(Filter::CircuitBreaker("my-cb".to_string())),
        )
        .await;

    let cb = Arc::new(GatewayCircuitBreaker::new(1, 1, Duration::from_secs(60)));
    // Trip the breaker open.
    cb.record_failure();
    assert_eq!(cb.state(), GatewayCbState::Open);

    let router = GatewayRouter::new(locator).with_circuit_breaker("my-cb", cb);

    let mut req = GatewayRequest::new(http::Method::GET, "/svc/test");
    let result = router.route(&mut req).await;
    assert!(result.is_err());
    let resp = result.unwrap_err();
    assert_eq!(resp.status, http::StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn test_router_circuit_breaker_allows_when_closed()
{
    let locator = Arc::new(InMemoryRouteLocator::new());
    locator
        .add_route(
            Route::new("svc", "http://svc:8080")
                .predicate(Predicate::Path("/svc".to_string()))
                .filter(Filter::CircuitBreaker("my-cb".to_string())),
        )
        .await;

    let cb = Arc::new(GatewayCircuitBreaker::new(5, 2, Duration::from_secs(30)));
    let router = GatewayRouter::new(locator).with_circuit_breaker("my-cb", cb);

    let mut req = GatewayRequest::new(http::Method::GET, "/svc/test");
    let result = router.route(&mut req).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some("http://svc:8080".to_string()));
}

#[tokio::test]
async fn test_router_record_response_success_closes_cb()
{
    let locator = Arc::new(InMemoryRouteLocator::new());
    let route = Route::new("svc", "http://svc:8080")
        .predicate(Predicate::Path("/svc".to_string()))
        .filter(Filter::CircuitBreaker("cb".to_string()));
    locator.add_route(route).await;

    let cb = Arc::new(GatewayCircuitBreaker::new(1, 1, Duration::from_millis(10)));
    // Trip open.
    cb.record_failure();
    assert_eq!(cb.state(), GatewayCbState::Open);

    // Wait for timeout → HalfOpen.
    std::thread::sleep(Duration::from_millis(20));
    assert!(cb.allow_request());
    assert_eq!(cb.state(), GatewayCbState::HalfOpen);

    let router = GatewayRouter::new(locator).with_circuit_breaker("cb", cb.clone());

    // Simulate a successful response.
    let resp = GatewayResponse::new(http::StatusCode::OK);
    let matched = router
        .match_route(&GatewayRequest::new(http::Method::GET, "/svc"))
        .await
        .unwrap();
    router.record_response(&resp, &matched);
    assert_eq!(cb.state(), GatewayCbState::Closed);
}

#[tokio::test]
async fn test_router_record_response_failure_trips_cb()
{
    let locator = Arc::new(InMemoryRouteLocator::new());
    let route = Route::new("svc", "http://svc:8080")
        .predicate(Predicate::Path("/svc".to_string()))
        .filter(Filter::CircuitBreaker("cb".to_string()));
    locator.add_route(route).await;

    let cb = Arc::new(GatewayCircuitBreaker::new(3, 1, Duration::from_secs(30)));
    let router = GatewayRouter::new(locator).with_circuit_breaker("cb", cb.clone());

    // Record 3 failures via the response recording mechanism.
    let fail_resp = GatewayResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR);
    let matched = router
        .match_route(&GatewayRequest::new(http::Method::GET, "/svc"))
        .await
        .unwrap();
    for _ in 0..3
    {
        router.record_response(&fail_resp, &matched);
    }
    assert_eq!(cb.state(), GatewayCbState::Open);
}

#[tokio::test]
async fn test_router_passthrough_when_no_limiter_registered()
{
    // A route has Filter::RateLimit(100) but no limiter is registered.
    // The request should pass through (passthrough mode).
    let locator = Arc::new(InMemoryRouteLocator::new());
    locator
        .add_route(
            Route::new("api", "http://backend:8080")
                .predicate(Predicate::Path("/api".to_string()))
                .filter(Filter::RateLimit(100)),
        )
        .await;

    let router = GatewayRouter::new(locator);
    let mut req = GatewayRequest::new(http::Method::GET, "/api/test");
    let result = router.route(&mut req).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some("http://backend:8080".to_string()));
}

#[tokio::test]
async fn test_router_no_match_returns_none()
{
    let locator = Arc::new(InMemoryRouteLocator::new());
    let router = GatewayRouter::new(locator);
    let mut req = GatewayRequest::new(http::Method::GET, "/nonexistent");
    let result = router.route(&mut req).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None);
}

#[tokio::test]
async fn test_router_combined_rate_limit_and_circuit_breaker()
{
    let locator = Arc::new(InMemoryRouteLocator::new());
    locator
        .add_route(
            Route::new("svc", "http://svc:8080")
                .predicate(Predicate::Path("/svc".to_string()))
                .filter(Filter::RateLimit(10))
                .filter(Filter::CircuitBreaker("svc-cb".to_string())),
        )
        .await;

    let limiter = Arc::new(TokenBucketRateLimiter::new(10, 5));
    let cb = Arc::new(GatewayCircuitBreaker::new(3, 1, Duration::from_secs(60)));

    let router = GatewayRouter::new(locator)
        .with_rate_limiter("10", limiter)
        .with_circuit_breaker("svc-cb", cb);

    // 5 requests should pass (burst = 5, CB is closed).
    for i in 0..5
    {
        let mut req = GatewayRequest::new(http::Method::GET, "/svc/test");
        let result = router.route(&mut req).await;
        assert!(result.is_ok(), "request {} should succeed", i);
    }

    // 6th should be rate-limited (burst exhausted).
    let mut req = GatewayRequest::new(http::Method::GET, "/svc/test");
    let result = router.route(&mut req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().status, http::StatusCode::TOO_MANY_REQUESTS);
}

// =======================================================================
// Glob pattern matching tests
// =======================================================================

#[test]
fn test_glob_exact_match()
{
    assert!(glob_match("/api/users", "/api/users"));
    assert!(!glob_match("/api/users", "/api/orders"));
}

#[test]
fn test_glob_double_star()
{
    assert!(glob_match("/api/**", "/api"));
    assert!(glob_match("/api/**", "/api/users"));
    assert!(glob_match("/api/**", "/api/users/123"));
    assert!(glob_match("/api/**", "/api/users/123/orders"));
    assert!(!glob_match("/api/**", "/other"));
}

#[test]
fn test_glob_single_star()
{
    assert!(glob_match("/api/*", "/api/users"));
    assert!(!glob_match("/api/*", "/api/users/123"));
    assert!(glob_match("/api/*/orders", "/api/users/orders"));
    assert!(!glob_match("/api/*/orders", "/api/users/items/orders"));
}

#[test]
fn test_glob_path_variable()
{
    assert!(glob_match("/users/{id}", "/users/123"));
    assert!(glob_match("/users/{id}", "/users/abc"));
    assert!(!glob_match("/users/{id}", "/users/123/orders"));
    assert!(glob_match("/users/{id}/orders/{oid}", "/users/123/orders/456"));
}

#[test]
fn test_glob_combined_patterns()
{
    assert!(glob_match("/api/{version}/**", "/api/v1/users/123"));
    assert!(glob_match("/api/{version}/**", "/api/v2"));
    assert!(!glob_match("/api/{version}/**", "/other/v1/users"));
}

#[test]
fn test_glob_empty_paths()
{
    assert!(glob_match("", ""));
    assert!(!glob_match("/api", ""));
}

#[test]
fn test_predicate_path_uses_glob()
{
    let pred = Predicate::Path("/api/**".to_string());
    let req = GatewayRequest::new(http::Method::GET, "/api/users/123");
    assert!(pred.matches(&req));

    let req2 = GatewayRequest::new(http::Method::GET, "/other");
    assert!(!pred.matches(&req2));
}

#[test]
fn test_filter_timeout_and_retry_are_infrastructure()
{
    assert!(Filter::Timeout(5000).is_infrastructure_filter());
    assert!(
        Filter::Retry {
            max_attempts: 3,
            statuses: vec![500, 502]
        }
        .is_infrastructure_filter()
    );
}
