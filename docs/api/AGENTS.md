# Nexus Framework — API Reference for LLM Context Injection

> Structured reference covering all 62 crates. No prose, no examples — pure API surface.
> Last updated: 2025-05-31

---

## 1. Runtime & Core

### nexus-runtime

Spring equivalent: `Spring WebFlux` reactor / `ScheduledExecutorService`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `Runtime` | `fn new() -> Result<Self>`, `fn block_on<F: Future>(&self, fut: F) -> F::Output` |
| struct | `RuntimeBuilder` | Builder for Runtime |
| struct | `RuntimeConfig` | Runtime configuration |
| struct | `DriverConfig` | Driver configuration |
| struct | `DriverConfigBuilder` | Builder for DriverConfig |
| trait | `DriverFactory` | Creates Driver instances |
| enum | `DriverType` | `IoUring`, `Epoll`, `Kqueue` |
| struct | `Driver` | Low-level I/O driver |
| struct | `Scheduler` | Thread-per-core scheduler |
| struct | `SchedulerConfig` | Scheduler configuration |
| struct | `WorkStealingScheduler` | Work-stealing variant |
| struct | `WorkStealingConfig` | Work-stealing configuration |
| fn | `spawn<F: Future + Send + 'static>(task: F) -> JoinHandle<F::Output>` | Async task spawn |
| fn | `sleep(duration: Duration) -> Sleep` | Async sleep |
| fn | `sleep_until(deadline: Instant) -> Sleep` | Sleep until instant |
| fn | `bounded<T>(cap: usize) -> (Sender<T>, Receiver<T>)` | Bounded MPSC channel |
| fn | `unbounded<T>() -> (Sender<T>, Receiver<T>)` | Unbounded MPSC channel |
| struct | `JoinHandle<T>` | Task join handle |
| struct | `JoinError` | Task join error |
| struct | `Sender<T>` / `Receiver<T>` | Channel halves |
| enum | `SendError<T>` / `RecvError` | Channel errors |
| fn | `select_two<F1, F2>(f1: F1, f2: F2) -> SelectTwoOutput` | Select on 2 futures |
| fn | `select_multiple(futures) -> SelectMultipleOutput` | Select on N futures |
| struct | `Duration` / `Instant` | Time primitives (re-exported) |

### nexus-core

Spring equivalent: `Spring Core` (IoC Container, `BeanFactory`, `ApplicationContext`)

| Kind | Symbol | Notes |
|------|--------|-------|
| trait | `Bean` | Core bean trait |
| struct | `BeanDefinition` | Bean metadata |
| trait | `BeanFactory` | IoC container abstraction |
| enum | `Scope` | `Singleton`, `Prototype`, `Request`, `Session` |
| struct | `ApplicationContext` | Full IoC container |
| struct | `Container` | Lightweight DI container |
| struct | `Extensions` | Type-map for request/response extensions |
| struct | `Error` | Framework error type |
| enum | `ErrorKind` | Error classification |
| type | `Result<T>` | `std::result::Result<T, Error>` |
| struct | `Mono<T>` | Reactive single-value (Spring WebFlux) |
| struct | `Flux<T>` | Reactive multi-value stream (Spring WebFlux) |
| trait | `ReflectContainer` | Runtime reflection on container |
| trait | `ContainerReflectExt` | Extension trait for reflection |

**Conditional beans:**

| Kind | Symbol | Notes |
|------|--------|-------|
| trait | `Condition` | Conditional evaluation |
| struct | `ConditionContext` | Context for condition evaluation |
| struct | `ConditionalOnBean` | Bean-present condition |
| struct | `ConditionalOnMissingBean` | Bean-absent condition |
| struct | `ConditionalOnProperty` | Property condition |
| struct | `ProfileCondition` | Active profile condition |
| struct | `AllConditions` / `AnyCondition` / `NotCondition` | Combinators |

### nexus-macros `[proc-macro]`

| Macro | Spring equivalent | Target |
|-------|-------------------|--------|
| `#[controller]` | `@Controller` | Struct |
| `#[service]` | `@Service` | Struct |
| `#[repository]` | `@Repository` | Struct |
| `#[component]` | `@Component` | Struct |
| `#[autowired]` | `@Autowired` | Field/param |
| `#[configuration]` | `@Configuration` | Struct |
| `#[bean]` | `@Bean` | Fn in `#[configuration]` |
| `#[transactional]` | `@Transactional` | Method |
| `#[get(path)]` | `@GetMapping` | Handler fn |
| `#[post(path)]` | `@PostMapping` | Handler fn |
| `#[put(path)]` | `@PutMapping` | Handler fn |
| `#[delete(path)]` | `@DeleteMapping` | Handler fn |
| `#[patch(path)]` | `@PatchMapping` | Handler fn |
| `#[head(path)]` | `@HeadMapping` | Handler fn |
| `#[options(path)]` | `@OptionsMapping` | Handler fn |
| `#[trace(path)]` | N/A | Handler fn |
| `#[request_mapping(path, method)]` | `@RequestMapping` | Handler fn |
| `#[rest_controller]` | `@RestController` | Struct |
| `#[cross_origin]` | `@CrossOrigin` | Struct/method |
| `#[nexus_main]` | `@SpringBootApplication` | Struct |
| `#[handler]` | N/A | Handler fn |
| `#[from_request_derive]` | N/A | Struct (FromRequest) |
| `#[into_response_derive]` | N/A | Struct (IntoResponse) |
| `#[profile]` | `@Profile` | Struct |
| `#[value(expr)]` | `@Value("${...}")` | Field |

---

## 2. Web Layer

### nexus-http

Spring equivalent: `Spring Web` / `Spring WebFlux` / `Spring MVC`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `Request` | HTTP request |
| struct | `Response` | HTTP response; `fn builder() -> ResponseBuilder`, `fn new(status: StatusCode) -> Self` |
| struct | `BodyBuilder` | Response builder |
| struct | `Json<T>` | JSON response wrapper; `fn new(value: T)`, `fn into_inner() -> T`, `fn get() -> &T` |
| trait | `IntoResponse` | `fn into_response(self) -> Response` |
| trait | `FromRequest` | `fn from_request(req: &Request) -> Result<Self>` (async) |
| trait | `HttpBody` | Body abstraction |
| struct | `Body` | Concrete body; `fn from(bytes)`, `fn empty()` |
| struct | `EmptyBody` / `FullBody` | Body variants |
| struct | `Server` | HTTP server; `fn new()`, `fn bind(addr)`, `fn run() -> impl Future` |
| struct | `HttpService` | Service abstraction |
| struct | `Connection` | HTTP connection |
| enum | `ConnectionState` | Connection lifecycle states |
| struct | `Uri` / `UriBuilder` | URI handling |
| struct | `ApiResponse<T>` | Generic API response |
| struct | `PageResponse<T>` | Paginated response |
| struct | `ResultCode` | Business result code |
| struct | `IntoApiResponse` | Trait for API response conversion |
| enum | `Method` | `GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `HEAD`, `OPTIONS`, `TRACE` |
| enum | `StatusCode` | HTTP status codes (1xx–5xx) |
| struct | `Error` | HTTP error type |
| type | `Result<T>` | `std::result::Result<T, Error>` |

**SSE:**

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `Sse` | Server-sent events stream |
| struct | `Event` | SSE event |
| struct | `SseKeepAlive` | Keep-alive configuration |

**WebSocket:**

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `WebSocket` | WebSocket connection |
| struct | `WebSocketConfig` | WS configuration |
| struct | `WebSocketUpgrade` | WS upgrade handler |
| struct | `WebSocketError` | WS error |
| enum | `Message` | `Text`, `Binary`, `Close`, `Ping`, `Pong` |
| struct | `CloseFrame` | Close frame payload |

**HTTP/2:**

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `Http2Config` | H2 configuration |
| enum | `ErrorCode` | H2 error codes |
| enum | `FrameType` | `Data`, `Headers`, `Settings`, `Ping`, `GoAway`, `WindowUpdate` |
| struct | `SettingsParameter` | SETTINGS frame params |
| struct | `StreamId` | Stream identifier |
| struct | `StreamState` / `StreamReset` | Stream lifecycle |
| enum | `Priority` | Stream priority |
| enum | `Http2ConnectionState` | Connection lifecycle |

**Exception handling:**

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `ControllerAdvice` | Global exception handler |
| struct | `ControllerAdviceBuilder` | Builder for ControllerAdvice |
| trait | `ExceptionHandler` | Exception handler trait |
| struct | `ErrorResponse` | Error response body |
| struct | `FieldError` | Field-level validation error |
| trait | `IntoErrorResponse` | Convert to ErrorResponse |
| struct | `ApplicationException` | Base application exception |
| struct | `ValidationException` | Validation error response |
| struct | `ResourceNotFoundException` | 404 handler |
| struct | `ExceptionHandlerRegistry` | Registry for handlers |
| struct | `ControllerErrorResponse` | Controller-level error |
| struct | `ForbiddenHandler` / `UnauthorizedHandler` / `NotFoundHandler` / `InternalErrorHandler` / `ValidationHandler` | Built-in handlers |

**Validation:**

| Kind | Symbol | Notes |
|------|--------|-------|
| trait | `Validatable` | Self-validation trait |
| trait | `ValidatableExtractor` | Extraction + validation |
| struct | `Validated<T>` | Wrapper for validated values |
| struct | `ValidationError` / `ValidationErrors` | Validation error types |
| struct | `ValidationHelpers` | Validation utility functions |
| struct | `ValidationMiddleware` | Middleware-level validation |

**Multipart (inline):**

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `MultipartData` | Multipart form data |
| struct | `MultipartFile` | Uploaded file |
| struct | `MultipartForm` | Form data extractor |
| struct | `FileSizeLimits` | Upload size limits |
| trait | `FromMultipart` | Multipart extraction trait |
| fn | `media_type_for_extension(ext)` | Extension to MIME |
| fn | `validate_content_type(ct)` / `validate_extension(ext)` | Content validation |

**Constants:**

| Kind | Symbol | Value |
|------|--------|-------|
| const | `content_type::JSON` | `"application/json"` |
| const | `content_type::HTML` | `"text/html"` |
| const | `content_type::TEXT` | `"text/plain"` |
| const | `content_type::FORM` | `"application/x-www-form-urlencoded"` |
| const | `content_type::MULTIPART_FORM` | `"multipart/form-data"` |
| const | `header::CONTENT_TYPE` | `"content-type"` |
| const | `header::CONTENT_LENGTH` | `"content-length"` |
| const | `header::AUTHORIZATION` | `"authorization"` |
| const | `header::ACCEPT` | `"accept"` |
| const | `header::USER_AGENT` | `"user-agent"` |
| const | `header::LOCATION` | `"location"` |

### nexus-router

Spring equivalent: `RequestMappingHandlerMapping`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `Router` | `fn new() -> Self`, `fn get(path, handler)`, `fn post(path, handler)`, `fn route(path, method, handler)`, `fn nest(path, router)`, `fn layer(middleware)` |
| struct | `Route` | Single route definition |
| struct | `Path` | Path parameter extractor; `fn get(name) -> &str`, `fn param(name) -> &str` |
| trait | `Handler` | Route handler trait |
| type | `BoxedAsyncHandler` | Type-erased async handler |
| struct | `AsyncHandlerFn` | Function-based handler |
| trait | `Middleware` | `fn handle(req, next) -> Response` |
| struct | `Next` | Middleware chain continuation |
| trait | `Stateful` | State injection trait |
| struct | `TrieRouter` | Radix tree router (high performance) |
| struct | `Handler as RouteHandler` | Re-exported handler |

### nexus-extractors

Spring equivalent: `@PathVariable`, `@RequestParam`, `@RequestBody`, `@RequestHeader`, `@CookieValue`, `@MatrixVariable`

| Kind | Symbol | Notes |
|------|--------|-------|
| trait | `FromRequest` | `fn from_request(req: &Request) -> ExtractorFuture<Self>` |
| type | `ExtractorFuture<T>` | `Pin<Box<dyn Future<Output = Result<T, ExtractorError>> + Send>>` |
| enum | `ExtractorError` | `Missing(String)`, `Invalid(String)`, `Io`, `Json`, `Other(String)` |
| struct | `Path<T: DeserializeOwned>` | `@PathVariable` — path params |
| struct | `Query<T: DeserializeOwned>` | `@RequestParam` — query string |
| struct | `Json<T: DeserializeOwned>` | `@RequestBody` — JSON body |
| struct | `Form<T: DeserializeOwned>` | Form data extraction |
| struct | `Header<T>` | `@RequestHeader` |
| struct | `HeaderOption<T>` | Optional header |
| struct | `NamedHeader` | Named header access |
| struct | `Cookie<T>` | `@CookieValue` |
| struct | `CookieOption<T>` | Optional cookie |
| struct | `NamedCookie` | Named cookie access |
| struct | `State<S>` | `@Autowired` — application state injection |
| struct | `RequestAttribute<T>` | `@RequestAttribute` |
| struct | `NamedRequestAttribute` | Named request attribute |
| struct | `MatrixPath` / `MatrixVariable` / `MatrixVariables` | `@MatrixVariable` |
| struct | `ModelAttribute<T>` | `@ModelAttribute` (query + form) |
| struct | `QueryParams<T>` | Query parameters only |
| struct | `Multipart` | Multipart form (feature-gated) |
| struct | `MultipartParser` | Multipart parsing |
| struct | `UploadedFile` | Uploaded file reference |
| struct | `UploadConfig` | Upload configuration |
| enum | `UploadError` | Upload error types |

### nexus-middleware

Spring equivalent: `Filter`, `HandlerInterceptor`, `CorsConfiguration`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `CorsMiddleware` | CORS handling |
| struct | `CorsConfig` | CORS configuration builder |
| struct | `CompressionMiddleware` | Gzip/Brotli compression |
| struct | `JwtAuthenticationMiddleware` | JWT auth middleware |
| struct | `JwtRequestExt` | JWT data on request |
| struct | `LoggerMiddleware` | Request/response logging |
| struct | `TimeoutMiddleware` | Request timeout |
| struct | `SecurityHeadersMiddleware` | Security header injection |
| struct | `SecurityHeadersConfig` | Security header config |
| struct | `StaticFiles` | Static file serving |
| struct | `MiddlewareStack` | Ordered middleware chain |
| trait | `Middleware` | (re-export from nexus-router) `fn handle(req, next) -> Response` |
| struct | `Next` | (re-export from nexus-router) Chain continuation |

### nexus-response

Spring equivalent: `ResponseEntity`, `@ResponseBody`, `ResponseAdvice`

| Kind | Symbol | Notes |
|------|--------|-------|
| trait | `IntoResponse` | Convert to HTTP response |
| struct | `Json<T>` | JSON response |
| struct | `Html<T>` | HTML response |
| struct | `PageResult<T>` | Paginated result |
| struct | `ResultCode` | Business status code |
| struct | `ApiResponse<T>` | Standard API response |
| struct | `ResponseAdvice` | Response post-processing |

### nexus-hateoas

Spring equivalent: `Spring HATEOAS` / `RepresentationModel`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `Link` | Hypermedia link |
| struct | `LinkBuilder` | Link builder |
| struct | `EntityModel<T>` | Single resource with links |
| struct | `CollectionModel<T>` | Collection with links |
| struct | `RepresentationModel` | Base model with links |

### nexus-multipart

Spring equivalent: `MultipartFile`, `MultipartResolver`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `Multipart` | Multipart form parser |
| struct | `MultipartFile` | Uploaded file |
| struct | `FileValidator` | File validation rules |
| struct | `MultipartConfig` | Upload config (size limits, temp dir) |

### nexus-openapi

Spring equivalent: `springdoc-openapi` / Swagger

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `OpenApi` | OpenAPI spec root |
| struct | `OpenApiBuilder` | Spec builder |
| struct | `OpenApiHandler` | HTTP handler serving spec |
| struct | `OpenApiRouter` | Router with OpenAPI endpoint |
| struct | `SwaggerUi` | Swagger UI handler |
| struct | `Schema` | JSON Schema definition |
| struct | `Operation` | API operation |
| struct | `Parameter` | Operation parameter |
| trait | `GenerateOpenApi` | Auto-generate spec from handlers |
| struct | `PostmanGenerator` | Postman collection export |

### nexus-graphql

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `context` | GraphQL context |
| struct | `dataloader` | Batch data loading |
| struct | `engine` | Query execution engine |
| struct | `resolver` | Field resolver |
| struct | `subscription` | Subscription support |

---

## 3. Data Layer

### nexus-data-commons

Spring equivalent: `Spring Data Commons` (`Repository`, `CrudRepository`, `Pageable`)

| Kind | Symbol | Notes |
|------|--------|-------|
| trait | `Repository<T, ID>` | Marker trait |
| trait | `CrudRepository<T, ID>` | `save`, `delete`, `find_by_id`, `exists_by_id`, `count`, `find_all` |
| trait | `PagingAndSortingRepository<T, ID>` | Extends CrudRepository; `find_all(pageable)`, `find_all(sort)` |
| struct | `Page<T>` | Paginated result; `fn content() -> &[T]`, `fn total_elements() -> u64`, `fn total_pages() -> u32`, `fn number() -> u32`, `fn size() -> u32`, `fn is_empty() -> bool` |
| struct | `PageRequest` | `fn new(page, size)`, `fn with_sort(sort)`, `fn offset() -> u64` |
| struct | `Slice<T>` | Slice of data (no total count) |
| struct | `List<T>` | Simple list result |
| struct | `Sort` | `fn by(field)` / `fn by_desc(field)`, `fn and(sort)` |
| struct | `Order` | Single sort order |
| enum | `Direction` | `Asc`, `Desc` |
| enum | `NullHandling` | `Native`, `NullsFirst`, `NullsLast` |
| struct | `MethodName` | `fn parse(name: &str) -> Result<ParsedQuery>` — parses `findByXxxAndYyy` |
| trait | `Specification` (as `Spec`) | Type-safe query predicate |
| struct | `SpecPredicate` / `SpecValue` / `SpecBuilder` | Specification combinators |
| struct | `BuiltSpecification` | Compiled specification |
| struct | `AlwaysSpec` / `SimpleSpec` / `AndSpec` / `OrSpec` / `NotSpec` | Specification factories |
| struct | `JpaSpecificationExecutor` | Specification-based query execution |
| trait | `Projection` | DTO projection |
| struct | `ProjectionField` / `ProjectionTransformer` / `DtoProjection` / `ClosedProjection` | Projection types |
| trait | `AggregateRoot` | Domain aggregate marker |
| trait | `Identifier` | Entity ID trait |
| trait | `Auditable` | Audit fields |
| trait | `Versioned` | Optimistic lock version |
| trait | `SoftDeletable` | Soft delete support |
| trait | `EntityWithLifecycle` | Lifecycle callbacks |
| enum | `LifecycleEvent` | `PrePersist`, `PostPersist`, `PreUpdate`, `PostUpdate`, `PreRemove`, `PostLoad` |
| trait | `TableName` / `ColumnName` | Metadata traits |
| trait | `Entity` | Base entity trait |
| struct | `CreatedDate` / `LastModifiedDate` / `CreatedBy` / `LastModifiedBy` | Audit annotations |
| trait | `AuditorAware` | Auditor provider |
| struct | `AuditingEntity` / `AuditingHandler` | Auditing support |
| struct | `OptimisticLockError` | Version conflict |
| trait | `Version` / `VersionCheckedUpdate` | Optimistic lock |
| struct | `PartTree` | Parsed query from method name |
| struct | `PartTreeOrderDirection` / `AndOr` / `Part` / `PartType` / `Subject` / `Keyword` / `OrderBy` | Part tree components |
| trait | `ToSql` | Convert Rust type to SQL literal |
| struct | `Error` | Data layer error |
| type | `Result<T>` | Data result type |

**Query wrapper (feature = "query"):**

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `QueryWrapper` | Type-safe query builder |
| struct | `UpdateWrapper` | Update query builder |
| struct | `LambdaQueryWrapper` | Lambda-based query |
| struct | `Condition` / `QueryOrder` / `Value` / `ToValue` / `Predicate` | Query primitives |

### nexus-data-rdbc

Spring equivalent: `Spring Data R2DBC` / `JdbcTemplate`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `DatabaseClient` | Async database client |
| struct | `QueryParam` | Typed query parameter |
| trait | `ToSql` | SQL literal conversion |
| struct | `Connection` | Database connection |
| struct | `PoolConfig` | Connection pool config |
| struct | `Transaction` | Transaction handle |
| struct | `TransactionManager` | Transaction lifecycle |
| enum | `IsolationLevel` | `ReadUncommitted`, `ReadCommitted`, `RepeatableRead`, `Serializable` |
| struct | `Row` | Database row |
| struct | `Column` | Column metadata |
| enum | `ColumnType` | Column type variants |
| trait | `FromRowValue` | Row value extraction |
| struct | `ColumnValue` | Cell value |
| trait | `RowMapper<T>` | Map Row -> T |
| trait | `ResultSetExtractor<T>` | Map ResultSet -> T |
| struct | `Rows` | Multiple rows result |
| fn | `row_mapper` | Utility for creating RowMapper |
| struct | `QueryExecutor` | Execute parameterized queries |
| struct | `BaseMapper` | CRUD mapper base |
| struct | `R2dbcBaseRepository` / `R2dbcCrudRepository` / `R2dbcRepository` | R2DBC repository |
| struct | `SqlxRepository` | SQLx-based repository |
| struct | `AnnotatedQueryExecutor` | Query from method annotations |
| struct | `QueryMetadata` / `QueryType` / `ParamStyle` | Query metadata |
| struct | `PgPoolClient` / `SqlxPoolClient` | Pool clients |
| struct | `MySqlPoolClient` | MySQL pool (feature-gated) |
| struct | `SqlitePoolClient` | SQLite pool (feature-gated) |
| enum | `DatabaseType` | `PostgreSQL`, `MySQL`, `SQLite`, `H2` |
| struct | `DatabaseConfig` | Database configuration |
| struct | `PostgresConfig` / `MySqlConfig` / `SqliteConfig` | DB-specific configs |
| enum | `SslMode` | SSL mode for connections |
| struct | `Error` / `R2dbcError` / `Result` / `R2dbcResult` | Error types |

### nexus-data-orm

Spring equivalent: `JPA` / `Hibernate` / `Criteria API`

| Kind | Symbol | Notes |
|------|--------|-------|
| trait | `Model` | Entity model trait (derive via `#[derive(Model)]`) |
| struct | `ModelMeta` | Model reflection metadata |
| struct | `Column` | Column definition |
| enum | `ColumnType` | Column type variants |
| enum | `SqlDialect` | SQL dialect variants |
| trait | `ActiveRecord` | CRUD on entity directly: `save()`, `delete()`, `refresh()`, `count()` |
| trait | `Save` / `Delete` / `Refresh` / `Count` / `OptimisticLock` | ActiveRecord sub-traits |
| struct | `QueryBuilder` | Type-safe query builder: `where_()`, `order_by()`, `limit()`, `all()`, `one()`, `count()` |
| struct | `WhereClause` / `OrderBy` / `Limit` | Query components |
| struct | `OrmRepository` | ORM-based repository |
| struct | `DefaultOrmRepository` | Default implementation |
| struct | `Connection` | ORM connection wrapper |

**Relationships:**

| Kind | Symbol | Notes |
|------|--------|-------|
| trait | `HasMany<T>` | One-to-many relation |
| trait | `HasOne<T>` | One-to-one relation |
| trait | `BelongsTo<T>` | Many-to-one (belongs to) |
| trait | `BelongsToMany<T>` | Many-to-many |
| struct | `EagerLoad` / `EagerQueryBuilder` | Eager loading config |
| trait | `WithRelations` | Load related entities |
| struct | `Relation` / `RelationType` / `OnDelete` | Relation metadata |

**Migrations:**

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `Migration` | Single migration step |
| struct | `Migrator` | Run/rollback migrations |
| enum | `MigrationDirection` | `Up`, `Down` |
| struct | `Schema` | Schema builder |

**Bridge crates (feature-gated):**

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `DieselSchema` / `DieselQuery` / `DieselColumnType` / `OrderDirection` | Diesel bridge |
| struct | `SqlxQuery` / `SqlxOrder` / `FromRow` / `VerifiedQuery` | SQLx bridge |
| (sea-orm) | — | SeaORM bridge |

### nexus-data-annotations `[proc-macro]`

Spring equivalent: `JPA` annotations (`@Entity`, `@Table`, `@Column`, etc.)

| Macro | Spring equivalent | Target |
|-------|-------------------|--------|
| `#[Entity]` | `@Entity` | Struct |
| `#[Table(name)]` | `@Table(name)` | Struct |
| `#[Id]` | `@Id` | Field |
| `#[GeneratedValue]` | `@GeneratedValue` | Field |
| `#[Column(name, type, nullable, unique, length)]` | `@Column` | Field |
| `#[OneToOne]` | `@OneToOne` | Field |
| `#[OneToMany]` | `@OneToMany` | Field |
| `#[ManyToOne]` | `@ManyToOne` | Field |
| `#[ManyToMany]` | `@ManyToMany` | Field |
| `#[Query(sql)]` | `@Query` | Method |
| `#[Insert]` | N/A | Method |
| `#[Update]` | N/A | Method |
| `#[Delete]` | N/A | Method |
| `#[Transactional]` | `@Transactional` | Method |
| `#[PreAuthorize(expr)]` | `@PreAuthorize` | Method |
| `#[CreatedDate]` | `@CreatedDate` | Field |
| `#[LastModifiedDate]` | `@LastModifiedDate` | Field |
| `#[CreatedBy]` | `@CreatedBy` | Field |
| `#[LastModifiedBy]` | `@LastModifiedBy` | Field |
| `#[PrePersist]` | `@PrePersist` | Method |
| `#[PostPersist]` | `@PostPersist` | Method |
| `#[PreUpdate]` | `@PreUpdate` | Method |
| `#[PostUpdate]` | `@PostUpdate` | Method |
| `#[PreRemove]` | `@PreRemove` | Method |
| `#[PostLoad]` | `@PostLoad` | Method |
| `#[Transient]` | `@Transient` | Field |
| `#[JoinColumn(name)]` | `@JoinColumn` | Field |
| `#[JoinTable(name)]` | `@JoinTable` | Field |

### nexus-data-macros `[proc-macro]`

| Macro | Target |
|-------|--------|
| `#[derive(Model)]` | Struct — generates Model trait impl |
| `#[model(table, primary_key, default)]` | Field attrs within Model derive |
| `#[repository]` | Struct — generates repository impl |

### nexus-data-redis

Spring equivalent: `Spring Data Redis` / `RedisTemplate`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `RedisTemplate` | Redis operations client |
| struct | `RedisCache` | Redis-backed cache implementation |
| struct | `RedisCacheManager` | Cache manager for Redis |
| struct | `RedisLock` | Distributed lock (reentrant + watchdog) |
| struct | `HashOps` | Redis Hash operations |
| struct | `LuaScript` | Lua script execution |
| struct | `RedisPipeline` | Pipelined commands |

### nexus-data-mongodb

Spring equivalent: `Spring Data MongoDB` / `MongoTemplate`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `MongoTemplate` | MongoDB operations |
| struct | `MongoRepository` | Repository for MongoDB |
| struct | `Aggregation` | Aggregation pipeline |
| struct | `BulkOperations` | Bulk write operations |
| struct | `IndexOperations` | Index management |
| struct | `MongoFilter` | Type-safe filter builder |

### nexus-flyway

Spring equivalent: `Flyway` database migrations

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `Flyway` | Migration runner |
| struct | `Migration` | Single migration |
| struct | `MigrationEntry` | Migration record |
| struct | `Info` | Migration status info |
| struct | `ConfigBuilder` | Flyway configuration builder |

---

## 4. Security

### nexus-security

Spring equivalent: `Spring Security`

| Kind | Symbol | Notes |
|------|--------|-------|
| trait | `UserDetails` | User details trait |
| struct | `User` | Default user implementation |
| struct | `InMemoryUserService` | In-memory user store |
| trait | `UserService` | User lookup service |
| struct | `Authentication` | Auth token/principal |
| trait | `AuthenticationManager` | Authenticate credentials |
| struct | `SecurityContext` | Thread-local security context |
| struct | `SecurityContextGuard` | RAII guard for context |
| struct | `SecurityContextExt` | Request extension for security |
| fn | `get_authentication_from_request(req)` | Extract auth from request |
| struct | `Role` / `Roles` | Role definition and enum |
| struct | `Permission` | Permission definition |
| trait | `GrantedAuthority` / `Authority` | Authority trait |
| struct | `PreAuthorize` | `@PreAuthorize` (attribute macro) |
| struct | `SecurityExpression` | SpEL security expression |
| struct | `PostAuthorize` | `@PostAuthorize` (attribute macro) |
| struct | `PostAuthorizeOptions` | Post-auth options |
| struct | `Secured` | `@Secured` (attribute macro) |
| struct | `SecuredHelper` / `SecurityMetadata` | Secured utilities |
| struct | `JwtTokenProvider` | JWT creation/validation |
| struct | `JwtClaims` / `JwtClaimsBuilder` | JWT payload |
| struct | `JwtUtil` | JWT utility functions |
| struct | `JwtAuthentication` | JWT-based auth |
| enum | `JwtAlgorithm` | HS256, RS256, etc. |
| struct | `PasswordEncoder` | Password hashing trait |
| struct | `BcryptPasswordEncoder` | BCrypt implementation |
| struct | `Pbkdf2PasswordEncoder` | PBKDF2 implementation |
| struct | `StandardPasswordEncoder` | Standard encoder |
| struct | `NoOpPasswordEncoder` | No-op (testing) |
| struct | `CsrfToken` / `CsrfTokenRepository` / `CsrfProtectionConfig` / `InMemoryCsrfTokenRepository` | CSRF protection |
| struct | `CookieCsrfTokenRepository` / `CsrfValidator` | CSRF cookie strategy |
| struct | `RbacManager` / `RbacConfig` | Role-based access control |
| struct | `RolePermission` / `UserRole` / `PermissionEntry` | RBAC entities |
| struct | `AuditLog` / `AuditLogger` / `ConsoleAuditLogger` | RBAC audit |
| struct | `PermissionRegistry` / `PermissionEvaluator` / `PermissionDef` | Permission system |
| struct | `PermissionAuditLog` / `PermissionAuditEntry` / `PermissionAuditLogger` | Permission audit |
| struct | `DataScope` / `DataScopeRule` / `DataScopeType` / `DataScopeContext` / `DataScopeApply` / `DataScopeMiddleware` | Data-level security |
| struct | `AuthorizationServer` / `AuthorizationServerBuilder` | OAuth2 authorization server |
| struct | `RegisteredClient` / `GrantType` | Client registration |
| struct | `OAuth2Client` / `OAuth2Config` / `TokenResponse` / `TokenResponseWithTimestamp` | OAuth2 client |
| struct | `OIDCDiscovery` / `OIDCDiscoveryDocument` / `UserInfo` / `IntrospectionResponse` | OIDC support |
| struct | `PkceParams` / `StateManager` / `TokenEndpointAuthMethod` | OAuth2 security |
| struct | `DeviceAuthorizationResponse` / `DeviceCodeStatus` / `IssuedTokenResponse` / `IntrospectionResult` | Device auth flow |
| struct | `EmailSender` / `SmtpEmailSender` / `EmailConfig` / `EmailMessage` / `EmailTemplate` / `EmailQueue` / `EmailResult` | Email integration |
| struct | `EmailError` / `Attachment` | Email types |
| struct | `SecurityError` / `SecurityResult` | Error types |
| const | `DEFAULT_ROLE_PREFIX` | `"ROLE_"` |
| const | `ANONYMOUS_USER` | `"anonymousUser"` |
| const | `REMEMBER_ME_KEY` | `"remember_me"` |

### nexus-session

Spring equivalent: `Spring Session`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `Session` | User session |
| trait | `SessionStore` | Session persistence |
| struct | `MemorySessionStore` | In-memory store |
| struct | `RedisSessionStore` | Redis store |
| struct | `MongoSessionStore` | MongoDB store |
| struct | `SessionConfig` | Session configuration |
| struct | `SessionStrategy` | Session creation strategy |
| struct | `SessionEvent` | Session lifecycle events |

---

## 5. Transactions & AOP

### nexus-tx

Spring equivalent: `Spring TX` (`@Transactional`, `TransactionTemplate`, `PlatformTransactionManager`)

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `Transaction` | Active transaction handle |
| trait | `TransactionManager` | `fn begin()`, `fn commit()`, `fn rollback()` |
| struct | `TransactionDefinition` | Transaction attributes |
| struct | `TransactionStatus` | Transaction state |
| struct | `TransactionTemplate` | Programmatic transaction control |
| struct | `Transactional` | `#[transactional]` attribute macro |
| struct | `TransactionalOptions` | `isolation`, `propagation`, `readonly`, `timeout`, `rollback_for` |
| enum | `IsolationLevel` | `Default`, `ReadUncommitted`, `ReadCommitted`, `RepeatableRead`, `Serializable` |
| enum | `Propagation` | `Required`, `RequiresNew`, `Nested`, `Mandatory`, `Supports`, `NotSupported`, `Never` |
| struct | `TransactionError` / `TransactionResult` | Error types |
| struct | `NoopTransactionManager` | No-op implementation |
| struct | `DelegatingTransactionManager` | Delegates to sub-managers |
| struct | `TransactionManagerRegistry` | Multi-manager registry |
| fn | `global_tx_manager()` / `set_global_tx_manager()` | Global manager |
| trait | `TransactionSynchronization` | Callbacks at commit/rollback |
| struct | `PhaseListener` / `SynchronizationRegistry` / `LoggingSynchronization` | Sync support |
| enum | `TransactionPhase` | `BeforeCommit`, `AfterCommit`, `AfterCompletion`, `AfterRollback` |
| struct | `TransactionContextExt` | Request extension for TX |
| fn | `get_transaction_from_request(req)` / `has_active_transaction_in_request(req)` | TX on request |
| const | `DEFAULT_TX_TIMEOUT_SECS` | `30` |
| const | `DEFAULT_TX_NAME` | `"default"` |

**SQLx bridge (feature = "sqlx"):**

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `SqlxTransactionManager` | SQLx-based TX manager |
| struct | `PostgresTransactionManager` | PostgreSQL TX |
| struct | `MySqlTransactionManager` | MySQL TX |
| struct | `SqliteTransactionManager` | SQLite TX |

### nexus-aop `[proc-macro]`

Spring equivalent: `Spring AOP` (`@Aspect`, `@Before`, `@After`, `@Around`)

| Macro | Target | Notes |
|-------|--------|-------|
| `#[before(pointcut)]` | Method | Before advice |
| `#[after(pointcut)]` | Method | After advice |
| `#[around(pointcut)]` | Method | Around advice |
| `#[after_returning(pointcut)]` | Method | After-returning advice |
| `#[after_throwing(pointcut)]` | Method | After-throwing advice |
| `#[pointcut(expr)]` | Method | Pointcut definition |

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `JoinPoint` | Method join point context |
| struct | `ProceedingJoinPoint` | Proceedable join point |
| struct | `AspectRegistry` | Register aspects |
| struct | `PointcutExpression` | Parsed pointcut expression |

---

## 6. Messaging

### nexus-events

Spring equivalent: `ApplicationEventPublisher`, `@EventListener`, `@TransactionalEventListener`

| Kind | Symbol | Notes |
|------|--------|-------|
| trait | `ApplicationEvent` | Event marker trait |
| trait | `Event` | Event payload |
| struct | `EventPayload` / `EventResult` | Event wrapper types |
| struct | `ApplicationEventPublisher` | `fn publish_event(event) -> impl Future` |
| enum | `PublishStrategy` | `Sync`, `Async`, `Conditional` |
| trait | `EventListener` | Sync event listener |
| trait | `AsyncEventListener` | Async event listener |
| struct | `EventConsumer` / `ListenerConfig` / `ListenerBuilder` | Listener configuration |
| struct | `ConditionFilter` | Listener condition filter |
| struct | `EventRegistry` / `EventSubscription` / `EventFilter` | Event registry |
| struct | `ContextRefreshedEvent` | Framework lifecycle event |
| struct | `PayloadApplicationEvent<T>` | Generic payload event |
| struct | `EventCondition` / `ConditionParser` | Event condition DSL |
| struct | `PropertyCondition` / `CompareOp` / `CompositeCondition` | Condition types |
| trait | `ConditionPropertyProvider` | Property source for conditions |
| struct | `AlwaysMatchCondition` / `NeverMatchCondition` | Constant conditions |
| fn | `evaluate_condition(condition, event)` | Evaluate condition |
| struct | `TransactionPhase` | `BeforeCommit`, `AfterCommit`, `AfterRollback`, `AfterCompletion` |
| struct | `TransactionalEventListener` / `TransactionalEventListenerConfig` | TX-bound listener |
| struct | `TransactionalEventPublisher` / `TransactionalEventBridge` | TX event bridge |
| const | `DEFAULT_EVENT_MODE` | `"async"` |

### nexus-events-macros `[proc-macro]`

| Macro | Spring equivalent |
|-------|-------------------|
| `#[EventListener]` | `@EventListener` |
| `#[TransactionalEventListener(phase)]` | `@TransactionalEventListener` |

### nexus-kafka

Spring equivalent: `Spring Kafka`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `KafkaProducer` | Produce messages |
| struct | `KafkaConsumer` | Consume messages |
| struct | `ConsumerGroup` | Consumer group config |
| struct | `KafkaMessage` | Message wrapper |
| struct | `TransactionalProducer` | Transactional Kafka producer |
| — | JSON/Avro/Protobuf serialization | Via feature flags |

### nexus-amqp

Spring equivalent: `Spring AMQP` / `Spring RabbitMQ`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `RabbitMqClient` | RabbitMQ client |
| struct | `Publisher` | Message publisher |
| struct | `ListenerContainer` | Message listener container |
| struct | `QueueBuilder` / `ExchangeBuilder` | Queue/exchange builders |
| struct | `JsonMessageConverter` | JSON message serialization |
| struct | `DeadLetterQueue` | Dead letter handling |
| enum | `AckMode` | Acknowledgment mode |

### nexus-integration

Spring equivalent: `Spring Integration`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `IntegrationFlow` | Integration flow builder |
| struct | `Channel` | Message channel |
| struct | `Transformer` | Message transformer |
| struct | `ContentBasedRouter` | Content-based routing |
| struct | `Filter` | Message filter |
| struct | `Splitter` | Message splitter |
| struct | `Aggregator` | Message aggregator |

### nexus-websocket-stomp

Spring equivalent: `Spring WebSocket STOMP`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `StompHandler` | STOMP protocol handler |
| struct | `StompFrame` | STOMP frame |
| struct | `StompConfig` | STOMP configuration |
| struct | `StompSession` | STOMP session |

---

## 7. Resilience & Observability

### nexus-resilience

Spring equivalent: `Resilience4j`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `CircuitBreaker` | `fn execute(fut) -> Result`, `fn is_open() -> bool` |
| struct | `CircuitBreakerConfig` | Threshold, timeout, half-open config |
| enum | `CircuitState` | `Closed`, `Open`, `HalfOpen` |
| struct | `CircuitBreakerRegistry` / `CircuitBreakerError` / `CircuitMetrics` | CB support |
| struct | `RateLimiter` | `fn check(key) -> Result` |
| struct | `RateLimiterConfig` | Rate, burst, window config |
| enum | `RateLimiterType` | `TokenBucket`, `SlidingWindow`, `FixedWindow` |
| struct | `RateLimiterRegistry` / `RateLimiterMetrics` / `RateLimitError` | RL support |
| struct | `RetryPolicy` | `fn retry(fut) -> Result` |
| enum | `BackoffType` | `Constant`, `Linear`, `Exponential` |
| struct | `RetryState` / `RetryError` / `ShouldRetry` / `RetryAll` / `RetryErrors` | Retry support |
| fn | `retry(fut, policy)` / `retry_with_predicate(fut, policy, pred)` | Retry functions |
| struct | `Timeout` | `fn wrap(fut, duration) -> Result` |
| struct | `TimeoutConfig` / `TimeoutMetrics` / `TimeoutRegistry` / `TimeoutError` | Timeout support |
| fn | `timeout(fut, duration)` | Timeout function |
| struct | `ServiceDiscovery` | Service registration and lookup |
| struct | `ServiceInstance` / `ServiceRegistry` / `SimpleServiceRegistry` | Service instances |
| struct | `DiscoveryError` | Discovery error |
| enum | `InstanceStatus` | `Up`, `Down`, `Starting`, `OutOfService` |
| enum | `LoadBalanceStrategy` | `RoundRobin`, `Random`, `LeastConnections` |

### nexus-observability

Spring equivalent: `Micrometer Tracing` + `Spring Boot Actuator`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `Tracer` | Distributed tracer |
| struct | `Span` | Active span |
| struct | `TraceId` / `SpanId` | Trace identifiers |
| struct | `TraceContext` | Trace propagation context |
| struct | `Counter` / `Gauge` / `Histogram` | Metric types |
| struct | `MetricsRegistry` | Global metrics registry |
| struct | `Logger` / `LoggerConfig` / `LoggerFactory` / `LoggerHandle` | Structured logging |
| enum | `LogLevel` | `Trace`, `Debug`, `Info`, `Warn`, `Error` |
| enum | `LogFormat` / `LogMode` / `LogRotation` | Log configuration |
| struct | `Banner` / `SimpleFormatter` / `StartupLogger` | Startup output (feature = "nexus-format") |
| fn | `info!` / `debug!` / `trace!` / `warn!` / `error!` | Logging macros (re-export tracing) |

### nexus-micrometer

Spring equivalent: `Micrometer`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `Counter` | Monotonic counter |
| struct | `Gauge` | Numeric gauge |
| struct | `Timer` | Timer histogram |
| struct | `LongTaskTimer` | Long-running task timer |
| struct | `MetricRegistry` | Global registry |
| fn | `global_registry()` | Access global registry |
| — | Prometheus export | Via feature flag |

### nexus-actuator

Spring equivalent: `Spring Boot Actuator`

| Kind | Symbol | Notes |
|------|--------|-------|
| trait | `HealthIndicator` | Health check implementation |
| struct | `MetricsRegistry` | Actuator metrics endpoint |
| struct | `InfoBuilder` | /info endpoint data |
| struct | `BeanDescriptor` | Bean description for /beans |
| struct | `Environment` | Environment info for /env |
| struct | `LoggerManager` | /loggers endpoint |

---

## 8. Cache

### nexus-cache

Spring equivalent: `Spring Cache` (`@Cacheable`, `CacheManager`)

| Kind | Symbol | Notes |
|------|--------|-------|
| trait | `Cache<K, V>` | `fn get(key)`, `fn put(key, value)`, `fn evict(key)`, `fn clear()` |
| struct | `MemoryCache<K, V>` | In-memory LRU cache |
| struct | `CacheBuilder` / `CacheConfig` / `CacheStats` | Cache configuration |
| struct | `Cacheable` | `#[cacheable("name")]` attribute macro |
| struct | `CacheableOptions` / `Cached` | Cacheable config |
| struct | `CachePut` / `CachePutOptions` / `CachePutExec` | `#[cache_put("name")]` |
| struct | `CacheEvict` / `CacheEvictOptions` / `CacheEvictExec` | `#[cache_evict("name")]` |
| enum | `EvictPolicy` | Eviction policy |
| struct | `Caching` / `CachingBuilder` / `CachingExec` | Multi-operation cache annotation |
| trait | `KeyGenerator` | Custom key generation |
| struct | `DefaultKeyGenerator` / `HashKeyGenerator` | Built-in key generators |
| fn | `evaluate_cache_condition(expr)` | Condition evaluation |
| struct | `CacheManager` / `CacheManagerBuilder` / `SimpleCacheManager` | Cache managers |
| struct | `CacheResolver` / `SimpleCacheResolver` | Cache name resolution |
| fn | `cache_get` / `cache_put` / `cache_evict_key` | Global cache operations |
| fn | `global_cache_manager()` | Access global cache manager |
| struct | `RedisCache` / `RedisCacheManager` / `RedisConfig` | Redis backend (feature = "redis") |
| enum | `SerializationFormat` | Redis serialization |
| const | `DEFAULT_CACHE` | `"default"` |
| const | `DEFAULT_TTL_SECS` | `600` (10 min) |
| const | `DEFAULT_MAX_CAPACITY` | `10_000` |

---

## 9. Configuration

### nexus-config

Spring equivalent: `@ConfigurationProperties`, `Environment`, `@Profile`, `spring.config`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `Config` | Configuration root |
| struct | `ConfigBuilder` | Configuration builder |
| struct | `Environment` | Property environment |
| struct | `Profile` | Active profile |
| struct | `ActiveProfiles` | Multiple profiles |
| struct | `PropertySource` | Property source (file, env, remote) |
| struct | `ConfigLoader` | Multi-source config loading |
| struct | `RefreshScope` | Runtime config refresh |
| struct | `ConfigEncryptor` | Encrypted property support |

### nexus-starter

Spring equivalent: `@SpringBootApplication`, auto-configuration

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `ApplicationContext` | Full application context |
| struct | `AutoConfiguration` | Auto-config marker |
| struct | `BeanDefinition` | Bean metadata |
| struct | `ComponentRegistry` | Component scanner |
| struct | `ConfigurationLoader` | Config loading |
| struct | `ConfigurationProperties` | `@ConfigurationProperties` |
| struct | `Environment` | Application environment |
| const | `DEFAULT_SERVER_PORT` | `8080` |
| const | `DEFAULT_SERVER_HOST` | `"127.0.0.1"` |
| const | `DEFAULT_WORKER_THREADS` | `4` |
| const | `APP_CONFIG_FILE` | `"application"` |
| const | `ENV_VAR_PREFIX` | `"NEXUS"` |
| const | `PROFILE_ENV_VAR` | `"NEXUS_PROFILE"` |

Feature-gated modules: `web`, `security`, `data`, `cache`, `schedule`, `actuator`

---

## 10. Cloud & External

### nexus-cloud

Spring equivalent: `Spring Cloud` (Netflix/Eureka, Ribbon, Zuul, Spring Cloud Config, Feign)

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `ServiceDiscovery` | Service registry/lookup |
| struct | `LoadBalancer` | Client-side load balancing |
| struct | `Gateway` | API gateway |
| struct | `ConfigServerClient` | Remote config client |
| struct | `FeignClient` | Declarative HTTP client |
| struct | `ConsulServiceRegistry` | Consul integration |

### nexus-ai

Spring equivalent: `Spring AI`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `ChatClient` | LLM chat client |
| struct | `ChatMessage` / `ChatRequest` / `ChatResponse` | Chat types |
| struct | `OpenAiChatModel` / `AnthropicChatModel` / `OllamaChatModel` | LLM providers |
| struct | `EmbeddingModel` | Vector embeddings |
| struct | `VectorStore` | Vector database |
| struct | `PromptTemplate` | Prompt templating |
| struct | `ToolRegistry` / `ToolExecutor` | Function calling |
| struct | `RAG` | Retrieval-augmented generation |

### nexus-agent

Spring equivalent: `Spring AI Agents`

| Kind | Symbol | Notes |
|------|--------|-------|
| trait | `Agent` | `fn run(input) -> Output`, `fn run_stream(input) -> Stream` |
| struct | `ReActAgent` | Reasoning + Acting agent |
| struct | `AgentChain` | Sequential agent pipeline |
| struct | `MapReduceAgent` | Parallel map-reduce agents |
| struct | `RouterAgent` | Dynamic agent routing |
| struct | `AgentPromptTemplate` | Agent prompt templates |

### nexus-web3

Spring equivalent: `Web3j` / custom

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `ChainConfig` / `ChainId` | Blockchain chain config |
| struct | `Contract` | Smart contract interaction |
| struct | `LocalWallet` / `HdWallet` / `MultiSigWallet` | Wallet types |
| struct | `RpcClient` | JSON-RPC client |
| struct | `TransactionBuilder` / `TxType` | Transaction construction |
| struct | `GasOracle` | Gas price oracle |
| struct | `ERC20` / `ERC721` / `ERC1155` | Token standard support |
| struct | `UniswapV2Router` | DeFi integration |

### nexus-vault

Spring equivalent: `Spring Vault`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `VaultClient` / `VaultConfig` | Vault connection |
| struct | `KV` | Key-Value secret engine |
| struct | `PKI` | PKI secret engine |
| struct | `Transit` | Encryption as service |
| struct | `JwtAuth` | JWT auth method |
| struct | `Lease` | Lease management |

### nexus-ldap

Spring equivalent: `Spring LDAP`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `LdapTemplate` | LDAP operations |
| struct | `LdapRepository` | LDAP repository |
| struct | `LdapPool` | Connection pool |
| struct | `ObjectDirectoryMapper` | LDAP object mapping |
| struct | `LdapQueryBuilder` | Query builder |
| — | LDIF support | Import/export |

### nexus-grpc

Spring equivalent: `gRPC Spring Boot Starter`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `GrpcError` | gRPC error type |
| — | client/server | gRPC client and server |
| — | interceptor | gRPC interceptors |
| — | metadata | gRPC metadata |
| — | retry | gRPC retry policy |
| — | TLS | TLS configuration |
| — | health | Health check service |

---

## 11. Processing & Scheduling

### nexus-batch

Spring equivalent: `Spring Batch`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `Job` | Batch job definition |
| struct | `Step` | Job step |
| trait | `ItemReader<T>` | Read items |
| trait | `ItemProcessor<I, O>` | Transform items |
| trait | `ItemWriter<T>` | Write items |
| struct | `JobLauncher` | Launch jobs |
| struct | `JobRepository` / `JobExecution` | Job persistence |
| struct | `AdvancedJobOperator` | Job control API |
| struct | `FaultTolerantStep` | Skip/retry support |

### nexus-async

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `AsyncTaskExecutor` | Async task executor |
| struct | `TaskExecutor` | Task executor trait |
| enum | `ExecutionMode` | Task execution mode |
| enum | `RejectionPolicy` | Task rejection policy |

### nexus-schedule

Spring equivalent: `@Scheduled`, `TaskScheduler`

| Kind | Symbol | Notes |
|------|--------|-------|
| macro | `#[Scheduled]` | `fixed_rate`, `cron`, `initial_delay` attributes |
| — | cron expression parser | Cron scheduling |

### nexus-state-machine

Spring equivalent: `Spring Statemachine`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `StateMachine<S, E>` | Generic state machine |
| struct | `StateMachineConfig<S, E>` | Configuration builder |
| struct | `State<S>` / `Transition<S, E>` | State and transition |
| struct | `Guard<S, E>` / `Action<S, E>` | Transition guards/actions |
| struct | `ForkJoinRegion` | Parallel regions |
| struct | `StateMachineVisualizer` | Graphviz visualization |
| trait | `Event` | Event trait |

### nexus-retry `[proc-macro]`

Spring equivalent: `@Retryable`, `@Recover`

| Macro | Notes |
|-------|-------|
| `#[retry(max, delay, backoff)]` | Retry on failure |
| `#[recover]` | Fallback method |
| struct | `RetryTemplate` | Programmatic retry |

### nexus-retry-macros `[proc-macro]`

Same macros as `nexus-retry` — dedicated proc-macro crate.

### nexus-modulith

Spring equivalent: `Spring Modulith`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `Module` | Module boundary |
| struct | `ModuleRegistry` | Module registry |
| struct | `DomainEvent` | Domain event |
| struct | `EventPublisher` | Domain event publisher |
| fn | `verify_modules()` | Verify module boundaries at compile time |

---

## 12. Validation & Error

### nexus-validation

Spring equivalent: `Spring Validation` / `javax.validation`

| Kind | Symbol | Notes |
|------|--------|-------|
| trait | `Validate` | Self-validation trait |
| struct | `ValidationError` | Single validation error |
| struct | `ValidationErrors` | Collection of errors |
| struct | `Valid<T>` | Extractor wrapper that validates |
| — | group validation | Validate specific groups |
| — | `Nested` | Nested object validation |
| — | custom validators | User-defined validators |
| — | `field_match` | Cross-field validation |

### nexus-validation-annotations `[proc-macro]`

| Macro | Spring equivalent |
|-------|-------------------|
| `#[derive(NotNull)]` | `@NotNull` |
| `#[derive(Email)]` | `@Email` |
| `#[derive(Size(min, max))]` | `@Size` |
| `#[derive(Min(value))]` | `@Min` |
| `#[derive(Max(value))]` | `@Max` |
| `#[derive(Pattern(regex))]` | `@Pattern` |
| `#[derive(Length(min, max))]` | `@Length` |

### nexus-exceptions

Spring equivalent: `@ControllerAdvice`, `@ExceptionHandler`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `ControllerAdvice` | Global exception handler |
| struct | `ExceptionHandler` | Route-specific handler |
| struct | `ErrorBody` | Error response body |
| struct | `ErrorResponse` | Full error response |

---

## 13. Tooling & DX

### nexus-test

Spring equivalent: `@SpringBootTest`, `MockBean`, `WebTestClient`

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `TestClient` | HTTP test client |
| struct | `TestApplication` | Test application context |
| struct | `MockBean` | Mock bean registration |
| struct | `WebTestClient` | Reactive test client |
| trait | `NexusTest` | Test trait marker |
| struct | `ContainerSet` | Testcontainers (Postgres, Redis, Kafka) |

### nexus-shell

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `Shell` / `Repl` | Interactive shell |
| struct | `CommandRegistry` | Register shell commands |
| struct | `Banner` / `PromptStyle` / `InputValidator` | Shell UI |

### nexus-shell-macros `[proc-macro]`

| Macro | Target |
|-------|--------|
| `#[shell_component]` | Struct — register as shell component |
| `#[shell_method]` | Method — register as shell command |

### nexus-lombok `[proc-macro]`

Spring equivalent: Lombok

| Macro | Notes |
|-------|-------|
| `#[derive(Getter)]` | Auto-generate getter methods |
| `#[derive(Setter)]` | Auto-generate setter methods |
| `#[derive(Data)]` | Getter + Setter + new() + Debug |
| `#[derive(Builder)]` | Builder pattern |
| `#[derive(Value)]` | Value object |
| `#[derive(With)]` | Clone-with-field methods |
| `#[derive(AllArgsConstructor)]` | Constructor with all fields |
| `#[derive(NoArgsConstructor)]` | Constructor with no fields |

### nexus-spel

Spring equivalent: `SpEL` (Spring Expression Language)

| Kind | Symbol | Notes |
|------|--------|-------|
| struct | `SpelContext` | Expression evaluation context |
| struct | `SpelEvaluator` | Evaluate SpEL expressions |
| struct | `SpelExpr` | Parsed expression |
| struct | `SpelError` | Evaluation error |
| — | `hasRole(x)` / `hasAuthority(x)` | Security expressions |

### nexus-benches

| Kind | Symbol | Notes |
|------|--------|-------|
| — | HTTP server benchmarks | Criterion benchmark suite |
| — | Router benchmarks | Route matching performance |
| — | Extractor benchmarks | Request extraction performance |

### nexus-i18n

| Kind | Symbol | Notes |
|------|--------|-------|
| — | Internationalization | Message source, locale resolution |

### nexus-ws

| Kind | Symbol | Notes |
|------|--------|-------|
| — | WebSocket | Low-level WebSocket support |
