# Spring Boot 企业级篇 - 第20-24章
# Spring Boot Enterprise - Chapters 20-24

> 企业级系统设计、权限管理、部署实战
> Enterprise System Design, Permission Management, Deployment

---

## 目录 / Table of Contents

1. [第20章：系统基础模块统一设计](#第20章系统基础模块统一设计)
2. [第21章：接口文档生成与调试](#第21章接口文档生成与调试)
3. [第22章：权限控制与角色管理](#第22章权限控制与角色管理)
4. [第23章：日志、监控与部署](#第23章日志监控与部署)
5. [第24章：企业级项目综合实战](#第24章企业级项目综合实战)

---

## 第20章：系统基础模块统一设计

### 模块化架构对比 / Modular Architecture Comparison

#### Spring Boot - 模块分层

```
my-enterprise-app/
├── src/main/java/com/example/myapp/
│   ├── config/                    # 配置模块
│   │   ├── SecurityConfig.java
│   │   ├── DatabaseConfig.java
│   │   ├── RedisConfig.java
│   │   └── AsyncConfig.java
│   │
│   ├── common/                    # 公共模块
│   │   ├── constant/             # 常量定义
│   │   ├── enums/                # 枚举定义
│   │   ├── exception/            # 自定义异常
│   │   ├── response/             # 统一响应
│   │   └── util/                 # 工具类
│   │
│   ├── model/                     # 数据模型
│   │   ├── entity/               # 实体类
│   │   ├── dto/                  # 数据传输对象
│   │   └── vo/                   # 视图对象
│   │
│   ├── security/                  # 安全模块
│   │   ├── JwtTokenProvider.java
│   │   ├── UserDetailsServiceImpl.java
│   │   ├── JwtAuthenticationFilter.java
│   │   └── CustomAccessDeniedHandler.java
│   │
│   ├── repository/                # 数据访问层
│   │   ├── UserRepository.java
│   │   ├── RoleRepository.java
│   │   └── PermissionRepository.java
│   │
│   ├── service/                   # 业务逻辑层
│   │   ├── UserService.java
│   │   ├── RoleService.java
│   │   └── PermissionService.java
│   │
│   ├── controller/                # 控制器层
│   │   ├── UserController.java
│   │   ├── RoleController.java
│   │   └── AuthController.java
│   │
│   └── aspect/                    # 切面模块
│       ├── LoggingAspect.java
│       ├── ValidationAspect.java
│       └── CacheAspect.java
```

#### Hiver - 模块分层

```
my-enterprise-app/
├── src/
│   ├── config/                    # 配置模块
│   │   ├── mod.rs
│   │   ├── security.rs
│   │   ├── database.rs
│   │   └── redis.rs
│   │
│   ├── common/                    # 公共模块
│   │   ├── mod.rs
│   │   ├── constants.rs
│   │   ├── enums.rs
│   │   ├── error.rs
│   │   ├── response.rs
│   │   └── utils.rs
│   │
│   ├── models/                    # 数据模型
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── user.rs
│   │   │   └── role.rs
│   │   ├── dto/
│   │   │   ├── mod.rs
│   │   │   └── user_dto.rs
│   │   └── vo/
│   │       ├── mod.rs
│   │       └── user_vo.rs
│   │
│   ├── security/                  # 安全模块
│   │   ├── mod.rs
│   │   ├── jwt_provider.rs
│   │   ├── user_details.rs
│   │   └── auth_middleware.rs
│   │
│   ├── repository/                # 数据访问层
│   │   ├── mod.rs
│   │   ├── user_repository.rs
│   │   └── role_repository.rs
│   │
│   ├── service/                   # 业务逻辑层
│   │   ├── mod.rs
│   │   ├── user_service.rs
│   │   └── role_service.rs
│   │
│   └── controller/                # 控制器层
│       ├── mod.rs
│       ├── user_controller.rs
│       └── auth_controller.rs
│
└── Cargo.toml
```

### 统一异常处理对比 / Unified Exception Handling

#### Spring Boot - 异常体系

```java
// 1. 基础异常类
@Getter
public class BaseException extends RuntimeException {
    private final Integer code;
    private final String message;

    public BaseException(ResultCode resultCode) {
        super(resultCode.getMessage());
        this.code = resultCode.getCode();
        this.message = resultCode.getMessage();
    }

    public BaseException(Integer code, String message) {
        super(message);
        this.code = code;
        this.message = message;
    }
}

// 2. 业务异常
public class BusinessException extends BaseException {
    public BusinessException(ResultCode resultCode) {
        super(resultCode);
    }

    public BusinessException(Integer code, String message) {
        super(code, message);
    }
}

// 3. 资源未找到异常
public class NotFoundException extends BaseException {
    private final String resource;
    private final String id;

    public NotFoundException(String resource, String id) {
        super(ResultCode.NOT_FOUND);
        this.resource = resource;
        this.id = id;
    }

    @Override
    public String getMessage() {
        return String.format("%s not found: %s", resource, id);
    }
}

// 4. 全局异常处理器
@RestControllerAdvice
@Slf4j
public class GlobalExceptionHandler {

    // 业务异常
    @ExceptionHandler(BusinessException.class)
    public Result<Void> handleBusinessException(BusinessException e) {
        log.warn("Business exception: {}", e.getMessage());
        return Result.error(e.getCode(), e.getMessage());
    }

    // 资源未找到
    @ExceptionHandler(NotFoundException.class)
    public Result<Void> handleNotFoundException(NotFoundException e) {
        log.warn("Resource not found: {}", e.getMessage());
        return Result.error(ResultCode.NOT_FOUND.getCode(), e.getMessage());
    }

    // 参数校验异常
    @ExceptionHandler(MethodArgumentNotValidException.class)
    public Result<Map<String, String>> handleValidationException(
        MethodArgumentNotValidException e
    ) {
        Map<String, String> errors = new HashMap<>();
        e.getBindingResult().getFieldErrors().forEach(error -> {
            errors.put(error.getField(), error.getDefaultMessage());
        });
        return Result.error(400, "参数校验失败").withErrors(errors);
    }

    // 权限异常
    @ExceptionHandler(AccessDeniedException.class)
    public Result<Void> handleAccessDeniedException(AccessDeniedException e) {
        log.warn("Access denied: {}", e.getMessage());
        return Result.error(ResultCode.FORBIDDEN);
    }

    // 认证异常
    @ExceptionHandler(AuthenticationException.class)
    public Result<Void> handleAuthenticationException(AuthenticationException e) {
        log.warn("Authentication failed: {}", e.getMessage());
        return Result.error(ResultCode.UNAUTHORIZED);
    }

    // 数据完整性异常
    @ExceptionHandler(DataIntegrityViolationException.class)
    public Result<Void> handleDataIntegrityViolationException(
        DataIntegrityViolationException e
    ) {
        log.error("Data integrity violation: {}", e.getMessage());
        return Result.error(409, "数据冲突");
    }

    // 默认异常
    @ExceptionHandler(Exception.class)
    public Result<Void> handleException(Exception e) {
        log.error("Unexpected error: {}", e.getMessage(), e);
        return Result.error(ResultCode.INTERNAL_ERROR);
    }
}
```

#### Hiver - 异常体系

```rust
// 1. 基础错误 trait
pub trait AppError: std::error::Error + Send + Sync {
    fn code(&self) -> u16;
    fn message(&self) -> String;
    fn details(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

// 2. 基础错误类型
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String,
    details: HashMap<String, String>,
    cause: Option<Box<dyn std::error::Error>>,
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    InternalError,
    Business(u16),
}

// 3. 业务错误
#[derive(Debug)]
pub struct BusinessError {
    code: u16,
    message: String,
}

impl AppError for BusinessError {
    fn code(&self) -> u16 { self.code }
    fn message(&self) -> String { self.message.clone() }
}

impl std::error::Error for BusinessError {}
impl std::fmt::Display for BusinessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

// 4. 资源未找到错误
#[derive(Debug)]
pub struct NotFoundError {
    resource: String,
    id: String,
}

impl NotFoundError {
    pub fn new(resource: &str, id: &str) -> Self {
        Self {
            resource: resource.to_string(),
            id: id.to_string(),
        }
    }
}

impl AppError for NotFoundError {
    fn code(&self) -> u16 { 404 }

    fn message(&self) -> String {
        format!("{} not found: {}", self.resource, self.id)
    }

    fn details(&self) -> HashMap<String, String> {
        let mut details = HashMap::new();
        details.insert("resource".to_string(), self.resource.clone());
        details.insert("id".to_string(), self.id.clone());
        details
    }
}

// 5. 错误构造器
impl Error {
    pub fn bad_request(msg: &str) -> Self {
        Self {
            kind: ErrorKind::BadRequest,
            message: msg.to_string(),
            details: HashMap::new(),
            cause: None,
        }
    }

    pub fn unauthorized(msg: &str) -> Self {
        Self {
            kind: ErrorKind::Unauthorized,
            message: msg.to_string(),
            details: HashMap::new(),
            cause: None,
        }
    }

    pub fn forbidden(msg: &str) -> Self {
        Self {
            kind: ErrorKind::Forbidden,
            message: msg.to_string(),
            details: HashMap::new(),
            cause: None,
        }
    }

    pub fn not_found(resource: &str, id: &str) -> Self {
        Self {
            kind: ErrorKind::NotFound,
            message: format!("{} not found: {}", resource, id),
            details: {
                let mut map = HashMap::new();
                map.insert("resource".to_string(), resource.to_string());
                map.insert("id".to_string(), id.to_string());
                map
            },
            cause: None,
        }
    }

    pub fn conflict(msg: &str) -> Self {
        Self {
            kind: ErrorKind::Conflict,
            message: msg.to_string(),
            details: HashMap::new(),
            cause: None,
        }
    }

    pub fn internal(msg: &str) -> Self {
        Self {
            kind: ErrorKind::InternalError,
            message: msg.to_string(),
            details: HashMap::new(),
            cause: None,
        }
    }

    pub fn with_details(mut self, details: HashMap<String, String>) -> Self {
        self.details = details;
        self
    }

    pub fn with_cause(mut self, cause: Box<dyn std::error::Error>) -> Self {
        self.cause = Some(cause);
        self
    }
}

// 6. 全局错误处理器
pub struct GlobalErrorHandler;

impl GlobalErrorHandler {
    pub fn handle_error(&self, error: Box<dyn AppError>) -> Response {
        let status = StatusCode::from_u16(error.code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let body = Json(serde_json::json!({
            "code": error.code(),
            "message": error.message(),
            "details": error.details(),
            "timestamp": Utc::now().to_rfc3339(),
        }));

        (status, body).into_response()
    }
}

// 7. IntoResponse 实现
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self.kind {
            ErrorKind::BadRequest => StatusCode::BAD_REQUEST,
            ErrorKind::Unauthorized => StatusCode::UNAUTHORIZED,
            ErrorKind::Forbidden => StatusCode::FORBIDDEN,
            ErrorKind::NotFound => StatusCode::NOT_FOUND,
            ErrorKind::Conflict => StatusCode::CONFLICT,
            ErrorKind::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            ErrorKind::Business(code) => StatusCode::from_u16(code).unwrap_or(StatusCode::BAD_REQUEST),
        };

        let body = Json(serde_json::json!({
            "code": status.as_u16(),
            "message": self.message,
            "details": self.details,
            "timestamp": Utc::now().to_rfc3339(),
        }));

        (status, body).into_response()
    }
}
```

---

## 第21章：接口文档生成与调试

### API 文档工具对比 / API Documentation Tools

#### Spring Boot - Swagger + Postman

```java
// 1. Swagger 配置增强
@Configuration
public class OpenApiConfig {

    @Bean
    public GroupedOpenApi publicApi() {
        return GroupedOpenApi.builder()
            .group("public")
            .pathsToMatch("/api/public/**")
            .build();
    }

    @Bean
    public GroupedOpenApi userApi() {
        return GroupedOpenApi.builder()
            .group("users")
            .pathsToMatch("/api/users/**")
            .addOpenApiMethodFilter(method -> method.isAnnotationPresent(Operation.class))
            .build();
    }

    @Bean
    public GroupedOpenApi adminApi() {
        return GroupedOpenApi.builder()
            .group("admin")
            .pathsToMatch("/api/admin/**")
            .addOpenApiMethodFilter(method ->
                method.isAnnotationPresent(PreAuthorize.class) &&
                method.getAnnotation(PreAuthorize.class).value().contains("ADMIN")
            )
            .build();
    }
}

// 2. API 分组注解
@Tag(name = "用户管理", description = "用户增删改查接口")
@RestController
@RequestMapping("/api/users")
public class UserController {

    @Operation(summary = "获取用户列表", tags = {"用户管理"})
    @ApiResponses({
        @ApiResponse(responseCode = "200", description = "成功"),
        @ApiResponse(responseCode = "401", description = "未认证")
    })
    @GetMapping
    public List<User> list() {
        return userService.findAll();
    }
}

// 3. Postman Collection 导出
// 通过 Spring Boot Actuator 自动生成 Postman Collection
@Configuration
public class PostmanConfig {

    @Bean
    public PostmanCollectionExporter postmanExporter() {
        return new PostmanCollectionExporter()
            .withBaseUrl("http://localhost:8080")
            .withAuth("Bearer", "{{token}}")
            .withFolders(
                new Folder("认证").addRequest(
                    new Request("登录", "/api/auth/login")
                        .method("POST")
                        .body("{\"username\":\"\",\"password\":\"\"}")
                ),
                new Folder("用户").addRequests(/* ... */)
            );
    }
}
```

#### Hiver - OpenAPI + Postman

```rust
use utoipa::{OpenApi, OpenApiSpec, openapi::OpenApiBuilder};
use utoipa_swagger_ui::SwaggerUi;

// 1. API 分组配置
#[derive(OpenApi)]
#[openapi(
    info(
        title = "企业级 API 文档",
        version = "1.0.0",
        description = "企业级管理系统 API 接口文档",
        contact(
            name = "API 支持",
            email = "api@example.com"
        )
    ),
    paths(
        // 公开接口
        login,
        register,
        // 用户接口
        list_users,
        get_user,
        create_user,
        update_user,
        delete_user,
        // 管理员接口
        list_all_users,
        update_user_status,
    ),
    components(
        schemas(
            User,
            LoginRequest,
            RegisterRequest,
            CreateUserRequest,
            UpdateUserRequest,
            Error,
            Result,
            PageResult
        )
    ),
    tags(
        (name = "auth", description = "认证相关接口"),
        (name = "users", description = "用户管理接口"),
        (name = "admin", description = "管理员接口"),
    ),
    servers(
        (url = "http://localhost:8080", description = "开发环境"),
        (url = "https://api-dev.example.com", description = "测试环境"),
        (url = "https://api.example.com", description = "生产环境")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
struct ApiDoc;

// 2. 接口分组控制器
#[controller]
#[tags("users")]
pub struct UserController;

/// 获取用户列表
///
/// 返回分页的用户列表，支持关键词搜索
#[utoipa::path(
    get,
    path = "/api/users",
    tag = "users",
    params(
        ("page" = u32, Query, description = "页码", default = "0"),
        ("size" = u32, Query, description = "每页数量", default = "10"),
        ("keyword" = String, Query, description = "搜索关键词"),
    ),
    responses(
        (status = 200, description = "成功", body = PageResult<User>),
        (status = 401, description = "未认证"),
        (status = 403, description = "无权限"),
    ),
    security(("bearer_auth" = []))
)]
#[get("/api/users")]
async fn list_users(
    #[query] page: Option<u32>,
    #[query] size: Option<u32>,
    #[query] keyword: Option<String>,
) -> Result<Json<PageResult<User>>, Error> {
    // ...
}

// 3. Postman Collection 生成
use serde_json::json;

pub fn generate_postman_collection() -> serde_json::Value {
    json!({
        "info": {
            "name": "Enterprise API",
            "description": "企业级管理系统 API 集合",
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
        },
        "variable": [
            {
                "key": "baseUrl",
                "value": "http://localhost:8080",
                "type": "string"
            },
            {
                "key": "token",
                "value": "",
                "type": "string"
            }
        ],
        "item": [
            {
                "name": "认证",
                "item": [
                    {
                        "name": "登录",
                        "request": {
                            "method": "POST",
                            "header": [
                                {
                                    "key": "Content-Type",
                                    "value": "application/json"
                                }
                            ],
                            "body": {
                                "mode": "raw",
                                "raw": "{\n  \"username\": \"admin\",\n  \"password\": \"password\"\n}"
                            },
                            "url": {
                                "raw": "{{baseUrl}}/api/auth/login",
                                "host": ["{{baseUrl}}"],
                                "path": ["api", "auth", "login"]
                            }
                        }
                    }
                ]
            },
            {
                "name": "用户管理",
                "item": [
                    {
                        "name": "获取用户列表",
                        "request": {
                            "method": "GET",
                            "header": [
                                {
                                    "key": "Authorization",
                                    "value": "Bearer {{token}}"
                                }
                            ],
                            "url": {
                                "raw": "{{baseUrl}}/api/users?page=0&size=10",
                                "host": ["{{baseUrl}}"],
                                "path": ["api", "users"],
                                "query": [
                                    {
                                        "key": "page",
                                        "value": "0"
                                    },
                                    {
                                        "key": "size",
                                        "value": "10"
                                    }
                                ]
                            }
                        }
                    }
                ]
            }
        ]
    })
}

// 4. 导出 Postman Collection 端点
#[get("/api-docs/postman")]
async fn export_postman_collection() -> Json<serde_json::Value> {
    Json(generate_postman_collection())
}

// 5. 路由配置
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        // API 路由
        .get("/api/users", list_users)
        .post("/api/auth/login", login)

        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", ApiDoc::openapi())
            .into_router())

        // Postman Collection
        .get("/api-docs/postman", export_postman_collection);

    Server::bind("127.0.0.1:8080")
        .serve(app)
        .await?;

    Ok(())
}
```

---

## 第22章：权限控制与角色管理

### RBAC 权限模型对比 / RBAC Model Comparison

#### Spring Boot - RBAC 实现

```java
// 1. 数据模型
@Entity
@Table(name = "sys_user")
@Data
public class User {
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(unique = true, nullable = false)
    private String username;

    private String password;

    @ManyToMany(fetch = FetchType.EAGER)
    @JoinTable(
        name = "sys_user_role",
        joinColumns = @JoinColumn(name = "user_id"),
        inverseJoinColumns = @JoinColumn(name = "role_id")
    )
    private Set<Role> roles = new HashSet<>();

    @ManyToMany(fetch = FetchType.EAGER)
    @JoinTable(
        name = "sys_user_permission",
        joinColumns = @JoinColumn(name = "user_id"),
        inverseJoinColumns = @JoinColumn(name = "permission_id")
    )
    private Set<Permission> permissions = new HashSet<>();
}

@Entity
@Table(name = "sys_role")
@Data
public class Role {
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(unique = true)
    private String code;  // ROLE_ADMIN, ROLE_USER

    private String name;

    @ManyToMany(fetch = FetchType.EAGER)
    @JoinTable(
        name = "sys_role_permission",
        joinColumns = @JoinColumn(name = "role_id"),
        inverseJoinColumns = @JoinColumn(name = "permission_id")
    )
    private Set<Permission> permissions = new HashSet<>();
}

@Entity
@Table(name = "sys_permission")
@Data
public class Permission {
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(unique = true)
    private String code;  // user:read, user:write, user:delete

    private String name;
    private String description;

    private String resource;  // /api/users
    private String action;    // GET, POST, PUT, DELETE
}

// 2. 权限服务
@Service
public class PermissionService {

    @Autowired
    private UserRepository userRepository;

    @Autowired
    private RoleRepository roleRepository;

    @Autowired
    private PermissionRepository permissionRepository;

    // 获取用户所有权限
    public Set<String> getUserPermissions(Long userId) {
        User user = userRepository.findById(userId)
            .orElseThrow(() -> new NotFoundException("User", userId.toString()));

        Set<String> permissions = new HashSet<>();

        // 直接分配的权限
        user.getPermissions().forEach(p -> permissions.add(p.getCode()));

        // 角色权限
        user.getRoles().forEach(role -> {
            permissions.add("ROLE_" + role.getCode());
            role.getPermissions().forEach(p -> permissions.add(p.getCode()));
        });

        return permissions;
    }

    // 检查用户是否有权限
    public boolean hasPermission(Long userId, String permission) {
        return getUserPermissions(userId).contains(permission);
    }

    // 分配角色
    @Transactional
    public void assignRole(Long userId, Long roleId) {
        User user = userRepository.findById(userId)
            .orElseThrow(() -> new NotFoundException("User", userId.toString()));
        Role role = roleRepository.findById(roleId)
            .orElseThrow(() -> new NotFoundException("Role", roleId.toString()));

        user.getRoles().add(role);
        userRepository.save(user);
    }

    // 分配权限
    @Transactional
    public void assignPermission(Long userId, Long permissionId) {
        User user = userRepository.findById(userId)
            .orElseThrow(() -> new NotFoundException("User", userId.toString()));
        Permission permission = permissionRepository.findById(permissionId)
            .orElseThrow(() -> new NotFoundException("Permission", permissionId.toString()));

        user.getPermissions().add(permission);
        userRepository.save(user);
    }
}

// 3. 权限注解使用
@RestController
@RequestMapping("/api/users")
public class UserController {

    // 需要管理员角色
    @PreAuthorize("hasRole('ADMIN')")
    @GetMapping
    public List<User> listUsers() {
        return userService.findAll();
    }

    // 需要用户读取权限
    @PreAuthorize("hasAuthority('user:read')")
    @GetMapping("/{id}")
    public User getUser(@PathVariable Long id) {
        return userService.findById(id);
    }

    // 需要用户写入权限
    @PreAuthorize("hasAuthority('user:write')")
    @PostMapping
    public User createUser(@RequestBody CreateUserRequest request) {
        return userService.create(request);
    }

    // 需要用户删除权限 OR 是本人
    @PreAuthorize("hasAuthority('user:delete') or #id == authentication.principal.id")
    @DeleteMapping("/{id}")
    public void deleteUser(@PathVariable Long id) {
        userService.delete(id);
    }
}

// 4. 数据权限
@Target({ElementType.METHOD, ElementType.TYPE})
@Retention(RetentionPolicy.RUNTIME)
public @interface DataScope {
    String tableAlias() default "";
    String field() default "creator_id";
}

@Aspect
@Component
public class DataScopeAspect {

    @Before("@annotation(dataScope)")
    public void doBefore(JoinPoint point, DataScope dataScope) {
        // 获取当前用户
        User currentUser = SecurityContextHolder.getContext()
            .getAuthentication()
            .getPrincipal();

        // 根据用户角色添加数据权限过滤条件
        if (currentUser.hasRole("ADMIN")) {
            // 管理员可以看到所有数据
            return;
        }

        // 普通用户只能看到自己的数据
        String condition = String.format("%s.%s = %d",
            dataScope.tableAlias(),
            dataScope.field(),
            currentUser.getId()
        );

        DataScopeContext.setCondition(condition);
    }
}

@Service
public class UserService {

    @DataScope(tableAlias = "u", field = "creator_id")
    public List<User> findAll() {
        // SQL 会自动拼接数据权限条件
        return userRepository.findAll();
    }
}
```

#### Hiver - RBAC 实现

```rust
use hiver_security::{Role, Permission, DataScope};
use std::collections::{HashSet, HashMap};

// 1. 数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub roles: Vec<Role>,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Role {
    pub id: i64,
    pub code: String,  // ADMIN, USER
    pub name: String,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Permission {
    pub id: i64,
    pub code: String,  // user:read, user:write
    pub name: String,
    pub resource: String,
    pub action: String,
}

// 2. 权限服务
#[service]
pub struct PermissionService {
    #[autowired]
    repository: Arc<dyn PermissionRepository>,
}

impl PermissionService {
    /// 获取用户所有权限
    pub async fn get_user_permissions(&self, user_id: i64) -> HashSet<String> {
        let user = self.repository.find_user_with_permissions(user_id).await;

        let mut permissions = HashSet::new();

        // 直接分配的权限
        for p in &user.permissions {
            permissions.insert(p.code.clone());
        }

        // 角色权限
        for role in &user.roles {
            permissions.insert(format!("ROLE_{}", role.code));
            for p in &role.permissions {
                permissions.insert(p.code.clone());
            }
        }

        permissions
    }

    /// 检查用户是否有权限
    pub async fn has_permission(&self, user_id: i64, permission: &str) -> bool {
        self.get_user_permissions(user_id).await
            .contains(&permission.to_string())
    }

    /// 分配角色
    pub async fn assign_role(&self, user_id: i64, role_id: i64) -> Result<(), Error> {
        self.repository.add_role_to_user(user_id, role_id).await
    }

    /// 移除角色
    pub async fn remove_role(&self, user_id: i64, role_id: i64) -> Result<(), Error> {
        self.repository.remove_role_from_user(user_id, role_id).await
    }

    /// 分配权限
    pub async fn assign_permission(&self, user_id: i64, permission_id: i64) -> Result<(), Error> {
        self.repository.add_permission_to_user(user_id, permission_id).await
    }
}

// 3. 权限检查宏
#[controller]
struct UserController;

#[get("/api/users")]
#[require_role("ADMIN")]
async fn list_users(
    #[auth] auth: &AuthContext,
    #[state] service: Arc<UserService>,
) -> Result<Json<Vec<User>>, Error> {
    if !auth.has_role("ADMIN") {
        return Err(Error::forbidden("需要管理员权限"));
    }
    Ok(Json(service.find_all().await))
}

#[get("/api/users/:id")]
#[require_permission("user:read")]
async fn get_user(
    id: i64,
    #[auth] auth: &AuthContext,
    #[state] service: Arc<UserService>,
) -> Result<Json<User>, Error> {
    if !auth.has_permission("user:read") {
        return Err(Error::forbidden("需要用户读取权限"));
    }
    Ok(Json(service.find_by_id(id).await?))
}

// 4. 数据权限
pub struct DataScopeContext {
    thread_local: std::cell::RefCell<HashMap<String, String>>,
}

impl DataScopeContext {
    pub fn set_condition(&self, condition: String) {
        self.thread_local.borrow_mut()
            .insert("condition".to_string(), condition);
    }

    pub fn get_condition(&self) -> Option<String> {
        self.thread_local.borrow().get("condition").cloned()
    }

    pub fn clear(&self) {
        self.thread_local.borrow_mut().clear();
    }
}

pub struct DataScope {
    pub table_alias: &'static str,
    pub field: &'static str,
}

impl DataScope {
    pub fn check(&self, auth: &AuthContext) -> Option<String> {
        if auth.has_role("ADMIN") {
            return None;  // 管理员无限制
        }

        // 普通用户只能访问自己的数据
        Some(format!(
            "{}.{} = {}",
            self.table_alias,
            self.field,
            auth.user_id()
        ))
    }
}

// 5. 使用数据权限
#[service]
pub struct UserService {
    #[autowired]
    repository: Arc<UserRepository>,
}

impl UserService {
    pub async fn find_all(&self, auth: &AuthContext) -> Result<Vec<User>, Error> {
        let data_scope = DataScope {
            table_alias: "u",
            field: "creator_id",
        };

        let condition = data_scope.check(auth);
        self.repository.find_all_with_condition(condition).await
    }
}

// 6. 资源级权限控制
#[derive(Debug)]
pub struct ResourcePermission {
    pub resource: String,
    pub action: String,
}

impl ResourcePermission {
    pub fn check(&self, auth: &AuthContext) -> bool {
        let permission = format!("{}:{}", self.resource, self.action);
        auth.has_permission(&permission)
    }
}

// 使用示例
#[put("/api/users/:id")]
async fn update_user(
    id: i64,
    #[auth] auth: &AuthContext,
    #[request_body] request: UpdateUserRequest,
) -> Result<Json<User>, Error> {
    let perm = ResourcePermission {
        resource: "user".to_string(),
        action: "write".to_string(),
    };

    // 检查权限
    if !perm.check(auth) {
        return Err(Error::forbidden("无权限"));
    }

    // 或者检查是否是本人
    if auth.user_id() != id && !auth.has_role("ADMIN") {
        return Err(Error::forbidden("只能修改自己的信息"));
    }

    // ...
}
```

---

## 第23章：日志、监控与部署

### 日志聚合对比 / Log Aggregation

#### Spring Boot - ELK Stack

```yaml
# application-prod.yml
logging:
  level:
    root: INFO
    com.example: INFO
  pattern:
    console: "%d{yyyy-MM-dd HH:mm:ss.SSS} [%thread] %-5level [%X{traceId}] %logger{36} - %msg%n"
  file:
    name: /var/log/app/application.log
  logback:
    rollingpolicy:
      max-file-size: 100MB
      max-history: 30

# Logstash 配置
spring:
  elasticsearch:
    uris: http://elasticsearch:9200
  sleuth:
    zipkin:
      base-url: http://zipkin:9411
```

```java
// 结构化日志
@Service
@Slf4j
public class UserService {

    @Autowired
    private ObjectMapper objectMapper;

    public User createUser(CreateUserRequest request) {
        // 结构化日志
        Map<String, Object> logData = new HashMap<>();
        logData.put("event", "user_created");
        logData.put("username", request.getUsername());
        logData.put("timestamp", System.currentTimeMillis());

        try {
            log.info("{}", objectMapper.writeValueAsString(logData));

            User user = userRepository.save(/* ... */);

            // MDC 支持
            MDC.put("userId", user.getId().toString());
            MDC.put("traceId", UUID.randomUUID().toString());

            return user;
        } catch (JsonProcessingException e) {
            log.error("Failed to serialize log data", e);
            throw new RuntimeException(e);
        } finally {
            MDC.clear();
        }
    }
}
```

#### Hiver - 日志聚合

```rust
use hiver_observability::log::{Logger, LoggerFactory, LogConfig};
use serde_json::json;

#[service]
pub struct UserService {
    #[autowired]
    logger: Arc<Logger>,

    #[autowired]
    repository: Arc<UserRepository>,
}

impl UserService {
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User, Error> {
        // 结构化日志
        self.logger.info()
            .field("event", "user_created")
            .field("username", &request.username)
            .field("timestamp", Utc::now().to_rfc3339())
            .message("Creating user")
            .log();

        let user = self.repository.create(&request).await?;

        // 设置追踪上下文
        self.logger.set_context("user_id", user.id.to_string());
        self.logger.set_context("trace_id", Uuid::new_v4().to_string());

        Ok(user)
    }
}

// 配置
fn configure_logging() -> Logger {
    let config = LogConfig::builder()
        .level(Level::Info)
        .format(LogFormat::Json)  // JSON 格式输出
        .output_targets(vec![
            LogOutput::Stdout,
            LogOutput::File("/var/log/app/application.log".into()),
        ])
        .rolling(RollingConfig {
            max_size: 100 * 1024 * 1024,  // 100MB
            max_history: 30,
            compress: true,
        })
        .build();

    LoggerFactory::new(config).get_logger("UserService")
}
```

### Docker 部署对比 / Docker Deployment

#### Spring Boot - Dockerfile

```dockerfile
# Dockerfile
FROM maven:3.8-openjdk-17-slim AS build

WORKDIR /app
COPY pom.xml .
COPY src ./src

RUN mvn clean package -DskipTests

FROM openjdk:17-slim

WORKDIR /app

# 添加非 root 用户
RUN groupadd -r appuser && useradd -r -g appuser appuser

# 复制 JAR
COPY --from=build /app/target/*.jar app.jar

# 修改权限
RUN chown -R appuser:appuser /app

USER appuser

# 暴露端口
EXPOSE 8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s \
  CMD curl -f http://localhost:8080/actuator/health || exit 1

# JVM 优化
ENV JAVA_OPTS="-Xms512m -Xmx512m -XX:+UseG1GC -XX:+PrintGCDetails"

ENTRYPOINT ["sh", "-c", "java $JAVA_OPTS -jar app.jar"]
```

```yaml
# docker-compose.yml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "8080:8080"
    environment:
      - SPRING_PROFILES_ACTIVE=prod
      - SPRING_DATASOURCE_URL=jdbc:postgresql://db:5432/mydb
      - SPRING_REDIS_HOST=redis
    depends_on:
      - db
      - redis
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/actuator/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    networks:
      - app-network

  db:
    image: postgres:15-alpine
    environment:
      - POSTGRES_DB=mydb
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=pass
    volumes:
      - postgres_data:/var/lib/postgresql/data
    networks:
      - app-network

  redis:
    image: redis:7-alpine
    networks:
      - app-network

volumes:
  postgres_data:

networks:
  app-network:
    driver: bridge
```

#### Hiver - Dockerfile

```dockerfile
# Dockerfile
FROM rust:1.75-alpine AS builder

WORKDIR /app

# 安装依赖
RUN apk add --no-cache musl-dev postgresql-client

# 复制 Cargo 文件
COPY Cargo.toml Cargo.lock ./

# 创建虚拟 main.rs 用于缓存依赖
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制源代码
COPY src ./src

# 构建应用
RUN cargo build --release

# 运行时镜像
FROM alpine:latest

WORKDIR /app

# 安装运行时依赖
RUN apk add --no-cache ca-certificates

# 添加非 root 用户
RUN addgroup -g 1000 appuser && \
    adduser -D -u 1000 -G appuser appuser

# 复制二进制文件
COPY --from=builder /app/target/release/myapp /app/myapp

# 修改权限
RUN chown -R appuser:appuser /app

USER appuser

# 暴露端口
EXPOSE 8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s \
  CMD wget -q -O- http://localhost:8080/health || exit 1

ENTRYPOINT ["/app/myapp"]
```

```yaml
# docker-compose.yml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - DATABASE_URL=postgresql://user:pass@db:5432/mydb
      - REDIS_URL=redis://redis:6379
      - SERVER_PORT=8080
    depends_on:
      - db
      - redis
    healthcheck:
      test: ["CMD", "wget", "-q", "-O-", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    networks:
      - app-network

  db:
    image: postgres:15-alpine
    environment:
      - POSTGRES_DB=mydb
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=pass
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U user -d mydb"]
      interval: 10s
      timeout: 5s
      retries: 5
    networks:
      - app-network

  redis:
    image: redis:7-alpine
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5
    networks:
      - app-network

volumes:
  postgres_data:

networks:
  app-network:
    driver: bridge
```

---

## 第24章：企业级项目综合实战

### 完整系统架构对比 / Complete System Architecture

#### Spring Boot - 微服务架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        API Gateway                             │
│                    (Spring Cloud Gateway)                      │
├─────────────────────────────────────────────────────────────────┤
│                         服务层 / Services                       │
├─────────────────────────────────────────────────────────────────┤
│ ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│ │   User       │  │   Order      │  │   Product    │          │
│ │  Service     │  │  Service     │  │   Service    │          │
│ └──────────────┘  └──────────────┘  └──────────────┘          │
│ ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│ │ Payment      │  │ Notification │  │   Search     │          │
│ │  Service     │  │   Service    │  │   Service    │          │
│ └──────────────┘  └──────────────┘  └──────────────┘          │
├─────────────────────────────────────────────────────────────────┤
│                        基础设施 / Infrastructure                │
├─────────────────────────────────────────────────────────────────┤
│ ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│ │    Eureka    │  │    Config    │  │    Zipkin    │          │
│ │ (Discovery)  │  │  (Center)    │  │  (Tracing)   │          │
│ └──────────────┘  └──────────────┘  └──────────────┘          │
│ ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│ │ PostgreSQL   │  │    Redis     │  │   RabbitMQ   │          │
│ │  (Primary)   │  │   (Cache)    │  │  (Message)   │          │
│ └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

#### Hiver - 微服务架构

```
┌─────────────────────────────────────────────────────────────────┐
│                      API Gateway (Hiver)                       │
│                    + Load Balancer                             │
├─────────────────────────────────────────────────────────────────┤
│                         服务层 / Services                       │
├─────────────────────────────────────────────────────────────────┤
│ ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│ │   User       │  │   Order      │  │   Product    │          │
│ │  Service     │  │  Service     │  │   Service    │          │
│ └──────────────┘  └──────────────┘  └──────────────┘          │
│ ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│ │ Payment      │  │ Notification │  │   Search     │          │
│ │  Service     │  │   Service    │  │   Service    │          │
│ └──────────────┘  └──────────────┘  └──────────────┘          │
├─────────────────────────────────────────────────────────────────┤
│                        基础设施 / Infrastructure                │
├─────────────────────────────────────────────────────────────────┤
│ ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│ │  Consul     │  │    Config    │  │    Jaeger    │          │
│ │ (Discovery)  │  │  (Center)    │  │  (Tracing)   │          │
│ └──────────────┘  └──────────────┘  └──────────────┘          │
│ ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│ │ PostgreSQL   │  │    Redis     │  │    Kafka     │          │
│ │  (Primary)   │  │   (Cache)    │  │  (Message)   │          │
│ └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

### 服务间通信对比 / Service Communication

#### Spring Boot - OpenFeign

```java
// Feign 客户端
@FeignClient(name = "order-service", url = "${services.order}")
public interface OrderClient {

    @GetMapping("/api/orders/user/{userId}")
    List<Order> getOrdersByUser(@PathVariable Long userId);

    @PostMapping("/api/orders")
    Order createOrder(@RequestBody CreateOrderRequest request);

    @GetMapping("/api/orders/{orderId}")
    Order getOrder(@PathVariable Long orderId);
}

// 使用 Feign
@Service
public class UserService {

    @Autowired
    private OrderClient orderClient;

    public UserWithOrders getUserWithOrders(Long userId) {
        User user = userRepository.findById(userId)
            .orElseThrow(() -> new NotFoundException("User", userId.toString()));

        List<Order> orders = orderClient.getOrdersByUser(userId);

        return UserWithOrders.builder()
            .user(user)
            .orders(orders)
            .build();
    }
}
```

#### Hiver - HTTP 客户端

```rust
use reqwest::Client;
use hiver_macros::service;

#[service]
pub struct OrderClient {
    client: Client,
    base_url: String,
}

impl OrderClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn get_orders_by_user(&self, user_id: i64) -> Result<Vec<Order>, ClientError> {
        let url = format!("{}/api/orders/user/{}", self.base_url, user_id);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClientError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ClientError::InvalidResponse(response.status().as_u16()));
        }

        response.json::<Vec<Order>>().await
            .map_err(|e| ClientError::ParseError(e.to_string()))
    }

    pub async fn create_order(&self, request: CreateOrderRequest) -> Result<Order, ClientError> {
        let url = format!("{}/api/orders", self.base_url);

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ClientError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ClientError::InvalidResponse(response.status().as_u16()));
        }

        response.json::<Order>().await
            .map_err(|e| ClientError::ParseError(e.to_string()))
    }
}

// 使用
#[service]
pub struct UserService {
    #[autowired]
    order_client: Arc<OrderClient>,
}

impl UserService {
    pub async fn get_user_with_orders(&self, user_id: i64) -> Result<UserWithOrders, Error> {
        let user = self.repository.find_by_id(user_id).await?;
        let orders = self.order_client.get_orders_by_user(user_id).await?;

        Ok(UserWithOrders { user, orders })
    }
}
```

---

## 功能对比总结 / Summary

### 企业级功能对比 / Enterprise Features Comparison

| 功能 / Feature | Spring Boot | Hiver | 完成度 |
|----------------|-------------|-------|--------|
| **模块化架构 / Modular** | Multi-module Maven | Workspace | ✅ 100% |
| **统一异常 / Exception** | @ControllerAdvice | ErrorHandler trait | ✅ 90% |
| **RBAC 权限 / RBAC** | @PreAuthorize | #[require_permission] | ⚠️ 75% |
| **数据权限 / Data Scope** | @DataScope | DataScope trait | ⚠️ 60% |
| **API 文档 / Swagger** | springdoc-openapi | utoipa | ⚠️ 80% |
| **Postman 集成** | 自动导出 | JSON 生成 | ⚠️ 70% |
| **结构化日志 / JSON Log** | Logback | hiver-observability | ✅ 85% |
| **Docker 部署** | 标准支持 | 标准支持 | ✅ 100% |
| **服务发现 / Discovery** | Eureka | Consul | ⚠️ 60% |
| **配置中心 / Config** | Spring Cloud Config | Consul/etcd | ⚠️ 60% |
| **链路追踪 / Tracing** | Sleuth + Zipkin | OpenTelemetry + Jaeger | ✅ 85% |
| **服务通信 / Communication** | OpenFeign | reqwest | ✅ 90% |

### 待补充功能 / Features to Add

1. **完善权限系统**
   - 实现动态权限加载
   - 支持更复杂的数据权限
   - 添加权限审计日志

2. **增强微服务支持**
   - 服务注册与发现
   - 配置中心集成
   - 服务熔断与降级

3. **完善监控体系**
   - 分布式追踪增强
   - 实时告警规则
   - 性能指标分析

---

## 学习总结 / Conclusion

通过对比 Spring Boot 和 Hiver 框架的 24 章内容，我们可以看到：

### 已完成 / Completed
- ✅ 基础 CRUD 功能
- ✅ REST API 开发
- ✅ IoC 容器和依赖注入
- ✅ 中间件系统
- ✅ 日志和监控
- ✅ JWT 认证
- ✅ 分页查询
- ✅ CORS 处理

### 待完善 / In Progress
- ⚠️ 参数校验系统 (70%)
- ⚠️ 统一响应结构 (80%)
- ⚠️ API 文档生成 (80%)
- ⚠️ 权限控制 (75%)

### 待实现 / Planned
- ⏳ 文件上传下载
- ⏳ 邮件服务
- ⏳ 文件导出
- ⏳ 数据权限
- ⏳ 微服务治理
