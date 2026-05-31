# Spring Boot 进阶篇 - 第9-12章
# Spring Boot Advanced - Chapters 9-12

> 安全认证、API 文档、日志监控
> Security, API Documentation, Logging & Monitoring

---

## 目录 / Table of Contents

1. [第9章：Spring Security 与认证授权基础](#第9章spring-security-与认证授权基础)
2. [第10章：Spring Security JWT 进阶](#第10章spring-security-jwt-进阶)
3. [第11章：接口文档生成](#第11章接口文档生成)
4. [第12章：日志系统配置与项目监控](#第12章日志系统配置与项目监控)

---

## 第9章：Spring Security 与认证授权基础

### 安全认证架构对比 / Security Architecture Comparison

#### Spring Boot - Spring Security 架构

```
┌─────────────────────────────────────────────────────────┐
│                    Spring Security                      │
├─────────────────────────────────────────────────────────┤
│  Web Security (Filter Chain)                           │
│  ┌───────────────────────────────────────────────────┐ │
│  │  SecurityContextPersistenceFilter                 │ │
│  │  → LogoutFilter                                  │ │
│  │  → UsernamePasswordAuthenticationFilter          │ │
│  │  → JwtAuthenticationFilter (Custom)              │ │
│  │  → ExceptionTranslationFilter                    │ │
│  │  → FilterSecurityInterceptor                     │ │
│  └───────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────┤
│  Authentication Manager                                 │
│  ┌───────────────────────────────────────────────────┐ │
│  │  ProviderManager                                  │ │
│  │    → DaoAuthenticationProvider                   │ │
│  │    → JwtAuthenticationProvider (Custom)          │ │
│  └───────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────┤
│  Access Decision Manager                                │
│  ┌───────────────────────────────────────────────────┐ │
│  │  AffirmativeBased                                 │ │
│  │    → RoleVoter                                    │ │
│  │    → CustomPermissionVoter                       │ │
│  └───────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

#### Hiver - Hiver Security 架构

```
┌─────────────────────────────────────────────────────────┐
│                    Hiver Security                       │
├─────────────────────────────────────────────────────────┤
│  Middleware Chain                                      │
│  ┌───────────────────────────────────────────────────┐ │
│  │  SecurityMiddleware                               │ │
│  │    → AuthenticationMiddleware                    │ │
│  │    → AuthorizationMiddleware                     │ │
│  │    → CsrfMiddleware                              │ │
│  └───────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────┤
│  Authentication Manager                                 │
│  ┌───────────────────────────────────────────────────┐ │
│  │  AuthenticationManager                            │ │
│  │    → DaoAuthenticationProvider                   │ │
│  │    → JwtAuthenticationProvider                    │ │
│  └───────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────┤
│  Access Control                                         │
│  ┌───────────────────────────────────────────────────┐ │
│  │  AccessDecisionManager                            │ │
│  │    → RoleVoter                                    │ │
│  │    → PermissionVoter                             │ │
│  └───────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### 基本认证配置对比 / Basic Authentication Configuration

#### Spring Boot - Security Config

```java
@Configuration
@EnableWebSecurity
@EnableGlobalMethodSecurity(prePostEnabled = true)
public class SecurityConfig {

    @Autowired
    private UserDetailsService userDetailsService;

    @Autowired
    private JwtAuthenticationEntryPoint jwtAuthenticationEntryPoint;

    @Autowired
    private JwtAuthenticationFilter jwtAuthenticationFilter;

    // 1. 密码编码器
    @Bean
    public PasswordEncoder passwordEncoder() {
        return new BCryptPasswordEncoder();
    }

    // 2. 用户详情服务
    @Bean
    public UserDetailsService userDetailsService() {
        return new CustomUserDetailsService();
    }

    // 3. 认证管理器
    @Bean
    public AuthenticationManager authenticationManager(
        AuthenticationConfiguration config
    ) throws Exception {
        return config.getAuthenticationManager();
    }

    // 4. 安全过滤器链
    @Bean
    public SecurityFilterChain securityFilterChain(HttpSecurity http) throws Exception {
        http
            // 禁用 CSRF (使用 JWT 时)
            .csrf(csrf -> csrf.disable())

            // 禁用 CORS (全局配置中处理)
            .cors(cors -> cors.disable())

            // 会话管理：无状态
            .sessionManagement(session -> session
                .sessionCreationPolicy(SessionCreationPolicy.STATELESS)
            )

            // 异常处理
            .exceptionHandling(exception -> exception
                .authenticationEntryPoint(jwtAuthenticationEntryPoint)
                .accessDeniedHandler(new CustomAccessDeniedHandler())
            )

            // 授权规则
            .authorizeHttpRequests(auth -> auth
                // 公开端点
                .requestMatchers(
                    "/api/auth/**",
                    "/api/public/**",
                    "/swagger-ui/**",
                    "/v3/api-docs/**"
                ).permitAll()

                // 需要认证
                .anyRequest().authenticated()
            )

            // 添加 JWT 过滤器
            .authenticationProvider(authenticationProvider())
            .addFilterBefore(jwtAuthenticationFilter,
                UsernamePasswordAuthenticationFilter.class);

        return http.build();
    }

    // 5. 认证提供者
    @Bean
    public AuthenticationProvider authenticationProvider() {
        DaoAuthenticationProvider provider = new DaoAuthenticationProvider();
        provider.setUserDetailsService(userDetailsService);
        provider.setPasswordEncoder(passwordEncoder());
        return provider;
    }
}
```

#### Hiver - Security Config

```rust
use hiver_security::{
    SecurityConfig, PasswordEncoder, AuthenticationManager,
    SecurityMiddleware, JwtAuthenticationProvider
};
use hiver_macros::{config, service};

#[config(prefix = "security")]
pub struct SecurityConfig {
    #[config(default = "true")]
    pub enabled: bool,

    #[config(nested)]
    pub jwt: JwtSecurityConfig,

    #[config(nested)]
    pub cors: CorsConfig,
}

#[config(prefix = "security.jwt")]
pub struct JwtSecurityConfig {
    pub secret: String,
    #[config(default = "86400")]
    pub expiration: u64,
    #[config(default = "Authorization")]
    pub header_name: String,
    #[config(default = "Bearer ")]
    pub token_prefix: String,
}

#[service]
pub struct SecurityService {
    #[autowired]
    auth_manager: Arc<AuthenticationManager>,

    #[autowired]
    password_encoder: Arc<dyn PasswordEncoder>,

    #[config]
    security_config: Arc<SecurityConfig>,
}

impl SecurityService {
    // 创建安全中间件
    pub fn create_middleware(&self) -> SecurityMiddleware {
        SecurityMiddleware::new(self.auth_manager.clone())
            .with_jwt_config(self.security_config.jwt.clone())
            .with_public_paths(&[
                "/api/auth/*",
                "/api/public/*",
                "/health",
                "/swagger-ui/*",
                "/api-docs/*",
            ])
    }
}

// 使用中间件
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let security_service = get_security_service();

    let app = Router::new()
        .get("/health", health_check)
        .nest("/api", api_routes())
        .middleware(Arc::new(security_service.create_middleware()));

    Server::bind("127.0.0.1:8080")
        .serve(app)
        .await?;

    Ok(())
}
```

### 用户认证流程对比 / User Authentication Flow

#### Spring Boot - Login Flow

```java
// 1. 登录请求 DTO
@Data
public class LoginRequest {
    @NotBlank
    private String username;

    @NotBlank
    private String password;
}

// 2. 登录响应 DTO
@Data
public class LoginResponse {
    private String token;
    private String type = "Bearer";
    private UserDto user;
}

// 3. 认证服务
@Service
@Slf4j
public class AuthenticationService {

    @Autowired
    private AuthenticationManager authenticationManager;

    @Autowired
    private UserDetailsService userDetailsService;

    @Autowired
    private JwtTokenProvider jwtTokenProvider;

    @Autowired
    private PasswordEncoder passwordEncoder;

    // 登录
    public LoginResponse login(LoginRequest request) {
        // 1. 认证
        Authentication authentication = authenticationManager.authenticate(
            new UsernamePasswordAuthenticationToken(
                request.getUsername(),
                request.getPassword()
            )
        );

        // 2. 获取用户
        UserDetails userDetails = userDetailsService
            .loadUserByUsername(request.getUsername());

        // 3. 生成 Token
        String token = jwtTokenProvider.generateToken(userDetails);

        // 4. 返回响应
        User user = ((CustomUserDetails) userDetails).getUser();
        return LoginResponse.builder()
            .token(token)
            .user(UserDto.from(user))
            .build();
    }

    // 注册
    public User register(RegisterRequest request) {
        // 检查用户是否存在
        if (userRepository.existsByUsername(request.getUsername())) {
            throw new BusinessException("用户名已存在");
        }

        // 创建用户
        User user = new User();
        user.setUsername(request.getUsername());
        user.setPassword(passwordEncoder.encode(request.getPassword()));
        user.setEmail(request.getEmail());
        user.setRoles(Set.of(Role.ROLE_USER));

        return userRepository.save(user);
    }
}

// 4. 认证控制器
@RestController
@RequestMapping("/api/auth")
public class AuthController {

    @Autowired
    private AuthenticationService authService;

    @PostMapping("/login")
    public Result<LoginResponse> login(@RequestBody @Valid LoginRequest request) {
        LoginResponse response = authService.login(request);
        return Result.success(response);
    }

    @PostMapping("/register")
    public Result<UserDto> register(@RequestBody @Valid RegisterRequest request) {
        User user = authService.register(request);
        return Result.success(UserDto.from(user));
    }

    @PostMapping("/logout")
    public Result<Void> logout() {
        // JWT 无状态，客户端删除 Token 即可
        return Result.success();
    }
}
```

#### Hiver - Login Flow

```rust
use hiver_security::{AuthenticationManager, JwtTokenProvider, PasswordEncoder};
use hiver_macros::{controller, post};
use serde::{Deserialize, Serialize};

// 1. 登录请求
#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

// 2. 登录响应
#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    #[serde(rename = "type")]
    pub token_type: String,
    pub user: UserDto,
}

// 3. 认证服务
#[service]
pub struct AuthenticationService {
    #[autowired]
    auth_manager: Arc<AuthenticationManager>,

    #[autowired]
    user_service: Arc<UserDetailsService>,

    #[autowired]
    jwt_provider: Arc<JwtTokenProvider>,

    #[autowired]
    password_encoder: Arc<dyn PasswordEncoder>,
}

impl AuthenticationService {
    // 登录
    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse, AuthError> {
        // 1. 认证
        let auth = self.auth_manager
            .authenticate(&request.username, &request.password)
            .await?;

        // 2. 获取用户
        let user = self.user_service
            .load_user_by_username(&request.username)
            .await?;

        // 3. 生成 Token
        let token = self.jwt_provider.generate_token(&user)?;

        // 4. 返回响应
        Ok(LoginResponse {
            token,
            token_type: "Bearer".to_string(),
            user: UserDto::from(user),
        })
    }

    // 注册
    pub async fn register(&self, request: RegisterRequest) -> Result<User, AuthError> {
        // 检查用户是否存在
        if self.user_service.exists_by_username(&request.username).await? {
            return Err(AuthError::UsernameExists);
        }

        // 创建用户
        let user = User {
            id: 0,
            username: request.username,
            password: self.password_encoder.encode(&request.password)?,
            email: request.email,
            roles: vec!["ROLE_USER".to_string()],
        };

        self.user_service.create_user(user).await
    }
}

// 4. 认证控制器
#[controller]
struct AuthController;

#[post("/api/auth/login")]
async fn login(
    #[request_body] request: LoginRequest,
    #[state] auth_service: Arc<AuthenticationService>,
) -> Result<Json<LoginResponse>, Error> {
    auth_service.login(request).await
        .map(Json)
        .map_err(|e| Error::unauthorized(&e.to_string()))
}

#[post("/api/auth/register")]
async fn register(
    #[request_body] request: RegisterRequest,
    #[state] auth_service: Arc<AuthenticationService>,
) -> Result<Json<UserDto>, Error> {
    let user = auth_service.register(request).await?;
    Ok(Json(UserDto::from(user)))
}

#[post("/api/auth/logout")]
async fn logout() -> Result<Status, Error> {
    // JWT 无状态，客户端删除 Token 即可
    Ok(Status::OK)
}
```

### JWT 实现对比 / JWT Implementation

#### Spring Boot - JwtTokenProvider

```java
@Component
@Slf4j
public class JwtTokenProvider {

    @Value("${app.security.jwt.secret}")
    private String jwtSecret;

    @Value("${app.security.jwt.expiration}")
    private long jwtExpiration;

    // 生成 Token
    public String generateToken(UserDetails userDetails) {
        Date now = new Date();
        Date expiryDate = new Date(now.getTime() + jwtExpiration * 1000);

        return Jwts.builder()
            .setSubject(userDetails.getUsername())
            .setIssuedAt(now)
            .setExpiration(expiryDate)
            .claim("roles", userDetails.getAuthorities())
            .signWith(SignatureAlgorithm.HS512, jwtSecret)
            .compact();
    }

    // 从 Token 获取用户名
    public String getUsernameFromToken(String token) {
        Claims claims = Jwts.parser()
            .setSigningKey(jwtSecret)
            .parseClaimsJws(token)
            .getBody();
        return claims.getSubject();
    }

    // 验证 Token
    public boolean validateToken(String token) {
        try {
            Jwts.parser()
                .setSigningKey(jwtSecret)
                .parseClaimsJws(token);
            return true;
        } catch (SignatureException ex) {
            log.error("Invalid JWT signature");
        } catch (MalformedJwtException ex) {
            log.error("Invalid JWT token");
        } catch (ExpiredJwtException ex) {
            log.error("Expired JWT token");
        } catch (UnsupportedJwtException ex) {
            log.error("Unsupported JWT token");
        } catch (IllegalArgumentException ex) {
            log.error("JWT claims string is empty");
        }
        return false;
    }

    // 获取 Token 剩余有效时间
    public long getExpirationTime(String token) {
        Claims claims = Jwts.parser()
            .setSigningKey(jwtSecret)
            .parseClaimsJws(token)
            .getBody();
        return claims.getExpiration().getTime() - System.currentTimeMillis();
    }
}

// JWT 过滤器
@Component
@Slf4j
public class JwtAuthenticationFilter extends OncePerRequestFilter {

    @Autowired
    private JwtTokenProvider jwtTokenProvider;

    @Autowired
    private UserDetailsService userDetailsService;

    @Override
    protected void doFilterInternal(
        HttpServletRequest request,
        HttpServletResponse response,
        FilterChain filterChain
    ) throws ServletException, IOException {

        // 1. 从请求头获取 Token
        String token = resolveToken(request);

        // 2. 验证 Token
        if (token != null && jwtTokenProvider.validateToken(token)) {
            // 3. 获取用户名
            String username = jwtTokenProvider.getUsernameFromToken(token);

            // 4. 加载用户详情
            UserDetails userDetails = userDetailsService
                .loadUserByUsername(username);

            // 5. 创建认证对象
            UsernamePasswordAuthenticationToken authentication =
                new UsernamePasswordAuthenticationToken(
                    userDetails,
                    null,
                    userDetails.getAuthorities()
                );

            // 6. 设置到安全上下文
            SecurityContextHolder.getContext()
                .setAuthentication(authentication);
        }

        filterChain.doFilter(request, response);
    }

    private String resolveToken(HttpServletRequest request) {
        String bearerToken = request.getHeader("Authorization");
        if (bearerToken != null && bearerToken.startsWith("Bearer ")) {
            return bearerToken.substring(7);
        }
        return null;
    }
}
```

#### Hiver - JwtTokenProvider

```rust
use jsonwebtoken::{encode, decode, Validation, EncodingKey, DecodingKey};
use jsonwebtoken::{Header, TokenData};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};

#[service]
pub struct JwtTokenProvider {
    #[config]
    secret: String,

    #[config(default = "86400")]
    expiration: u64,

    #[config(default = "Authorization")]
    header_name: String,

    #[config(default = "Bearer ")]
    token_prefix: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub roles: Vec<String>,
}

impl JwtTokenProvider {
    // 生成 Token
    pub fn generate_token(&self, user: &User) -> Result<String, JwtError> {
        let now = Utc::now();
        let expiration = now + Duration::seconds(self.expiration as i64);

        let claims = Claims {
            sub: user.username.clone(),
            exp: expiration.timestamp(),
            iat: now.timestamp(),
            roles: user.roles.clone(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )?;

        Ok(token)
    }

    // 从 Token 获取用户名
    pub fn get_username_from_token(&self, token: &str) -> Result<String, JwtError> {
        let claims = self.decode_token(token)?;
        Ok(claims.claims.sub)
    }

    // 验证 Token
    pub fn validate_token(&self, token: &str) -> bool {
        match self.decode_token(token) {
            Ok(_) => true,
            Err(e) => {
                log::warn!("Invalid JWT token: {}", e);
                false
            }
        }
    }

    // 解码 Token
    fn decode_token(&self, token: &str) -> Result<TokenData<Claims>, JwtError> {
        let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &validation,
        )?;
        Ok(token_data)
    }

    // 获取 Token 剩余有效时间
    pub fn get_expiration_time(&self, token: &str) -> Result<Duration, JwtError> {
        let token_data = self.decode_token(token)?;
        let now = Utc::now().timestamp();
        let exp = token_data.claims.exp;
        Ok(Duration::seconds(exp - now))
    }
}

// JWT 认证中间件
pub struct JwtAuthenticationMiddleware {
    token_provider: Arc<JwtTokenProvider>,
    user_service: Arc<UserDetailsService>,
}

impl Middleware for JwtAuthenticationMiddleware {
    async fn call(&self, req: Request, next: Next) -> Result<Response, Error> {
        // 1. 从请求头获取 Token
        let token = self.resolve_token(&req)?;

        // 2. 验证 Token
        if let Some(token) = token {
            if self.token_provider.validate_token(&token) {
                // 3. 获取用户名
                let username = self.token_provider.get_username_from_token(&token)?;

                // 4. 加载用户
                let user = self.user_service.load_user_by_username(&username).await?;

                // 5. 设置认证上下文
                req.set_auth_context(AuthContext::new(user));
            }
        }

        // 6. 继续处理
        next.run(req).await
    }
}

impl JwtAuthenticationMiddleware {
    fn resolve_token(&self, req: &Request) -> Result<Option<String>, Error> {
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok());

        if let Some(auth) = auth_header {
            if auth.starts_with("Bearer ") {
                return Ok(Some(auth[7..].to_string()));
            }
        }

        Ok(None)
    }
}
```

---

## 第10章：Spring Security JWT 进阶

### 权限控制模型对比 / Permission Control Model

#### Spring Boot - RBAC 权限模型

```java
// 1. 用户实体
@Entity
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
        name = "user_roles",
        joinColumns = @JoinColumn(name = "user_id"),
        inverseJoinColumns = @JoinColumn(name = "role_id")
    )
    private Set<Role> roles;

    @ManyToMany(fetch = FetchType.EAGER)
    @JoinTable(
        name = "user_permissions",
        joinColumns = @JoinColumn(name = "user_id"),
        inverseJoinColumns = @JoinColumn(name = "permission_id")
    )
    private Set<Permission> permissions;
}

// 2. 角色实体
@Entity
@Data
public class Role {
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(unique = true)
    private String name; // ROLE_ADMIN, ROLE_USER

    @ManyToMany(fetch = FetchType.EAGER)
    @JoinTable(
        name = "role_permissions",
        joinColumns = @JoinColumn(name = "role_id"),
        inverseJoinColumns = @JoinColumn(name = "permission_id")
    )
    private Set<Permission> permissions;
}

// 3. 权限实体
@Entity
@Data
public class Permission {
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(unique = true)
    private String name; // user:read, user:write, user:delete

    private String description;
}

// 4. 权限注解使用
@RestController
@RequestMapping("/api/users")
public class UserController {

    // 角色检查
    @PreAuthorize("hasRole('ADMIN')")
    @GetMapping
    public List<User> listUsers() {
        return userService.findAll();
    }

    // 权限检查
    @PreAuthorize("hasAuthority('user:read')")
    @GetMapping("/{id}")
    public User getUser(@PathVariable Long id) {
        return userService.findById(id);
    }

    @PreAuthorize("hasAuthority('user:write')")
    @PostMapping
    public User createUser(@RequestBody CreateUserRequest request) {
        return userService.create(request);
    }

    @PreAuthorize("hasAuthority('user:delete')")
    @DeleteMapping("/{id}")
    public void deleteUser(@PathVariable Long id) {
        userService.delete(id);
    }

    // 复杂表达式
    @PreAuthorize("hasRole('ADMIN') or #id == authentication.principal.id")
    @PutMapping("/{id}")
    public User updateUser(@PathVariable Long id, @RequestBody UpdateUserRequest request) {
        return userService.update(id, request);
    }
}
```

#### Hiver - RBAC 权限模型

```rust
use hiver_security::{HasRole, HasPermission, AuthContext};
use hiver_macros::{controller, get, post, put, delete};

// 1. 用户实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

// 2. 权限检查注解
pub trait HasRole {
    fn has_role(&self, role: &str) -> bool;
}

pub trait HasPermission {
    fn has_permission(&self, permission: &str) -> bool;
}

impl HasRole for AuthContext {
    fn has_role(&self, role: &str) -> bool {
        self.user().roles.iter().any(|r| r == role)
    }
}

impl HasPermission for AuthContext {
    fn has_permission(&self, permission: &str) -> bool {
        self.user().permissions.iter().any(|p| p == permission)
    }
}

// 3. 权限中间件
pub struct AuthorizationMiddleware;

impl Middleware for AuthorizationMiddleware {
    async fn call(&self, req: Request, next: Next) -> Result<Response, Error> {
        // 获取路由所需权限
        let required_permission = req.get_required_permission();

        if let Some(permission) = required_permission {
            let auth = req.get_auth_context()
                .ok_or_else(|| Error::unauthorized("未登录"))?;

            if !auth.has_permission(&permission) {
                return Err(Error::forbidden("权限不足"));
            }
        }

        next.run(req).await
    }
}

// 4. 使用权限注解
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
        return Err(Error::forbidden("需要读取权限"));
    }
    Ok(Json(service.find_by_id(id).await?))
}

#[post("/api/users")]
#[require_permission("user:write")]
async fn create_user(
    #[request_body] request: CreateUserRequest,
    #[auth] auth: &AuthContext,
    #[state] service: Arc<UserService>,
) -> Result<Json<User>, Error> {
    if !auth.has_permission("user:write") {
        return Err(Error::forbidden("需要写入权限"));
    }
    Ok(Json(service.create(request).await?))
}

// 5. 复杂权限检查
#[put("/api/users/:id")]
async fn update_user(
    id: i64,
    #[request_body] request: UpdateUserRequest,
    #[auth] auth: &AuthContext,
    #[state] service: Arc<UserService>,
) -> Result<Json<User>, Error> {
    // ADMIN 或 用户本人可以修改
    let can_update = auth.has_role("ADMIN") || auth.user_id() == id;

    if !can_update {
        return Err(Error::forbidden("权限不足"));
    }

    Ok(Json(service.update(id, request).await?))
}
```

### 登录拦截器对比 / Login Interceptor

#### Spring Boot - Interceptor + Aspect

```java
// 1. 登录日志切面
@Aspect
@Component
@Slf4j
public class LoginLogAspect {

    @Autowired
    private LoginLogService loginLogService;

    @Pointcut("execution(* com.example.service.AuthenticationService.login(..))")
    public void loginPointcut() {}

    @AfterReturning(pointcut = "loginPointcut()", returning = "result")
    public void logSuccessfulLogin(JoinPoint joinPoint, Object result) {
        LoginRequest request = (LoginRequest) joinPoint.getArgs()[0];
        LoginResponse response = (LoginResponse) result;

        LoginLog log = LoginLog.builder()
            .username(request.getUsername())
            .ip(getClientIp())
            .userAgent(getUserAgent())
            .status(LoginStatus.SUCCESS)
            .loginTime(LocalDateTime.now())
            .build();

        loginLogService.save(log);
        log.info("User {} logged in successfully from {}", request.getUsername(), log.getIp());
    }

    @AfterThrowing(pointcut = "loginPointcut()", throwing = "exception")
    public void logFailedLogin(JoinPoint joinPoint, Exception exception) {
        LoginRequest request = (LoginRequest) joinPoint.getArgs()[0];

        LoginLog log = LoginLog.builder()
            .username(request.getUsername())
            .ip(getClientIp())
            .userAgent(getUserAgent())
            .status(LoginStatus.FAILED)
            .failureReason(exception.getMessage())
            .loginTime(LocalDateTime.now())
            .build();

        loginLogService.save(log);
        log.warn("User {} failed to login from {}: {}", request.getUsername(), log.getIp(), exception.getMessage());
    }
}

// 2. 登录尝试限制
@Component
@Slf4j
public class LoginAttemptService {
    private final int MAX_ATTEMPTS = 5;
    private final long BLOCK_TIME_MINUTES = 30;

    @Autowired
    private RedisTemplate<String, String> redisTemplate;

    public void loginFailed(String username) {
        String key = "login_attempt:" + username;
        Long attempts = redisTemplate.opsForValue().increment(key);

        if (attempts == 1) {
            redisTemplate.expire(key, BLOCK_TIME_MINUTES, TimeUnit.MINUTES);
        }

        if (attempts >= MAX_ATTEMPTS) {
            log.warn("User {} is blocked for {} minutes due to too many failed attempts",
                username, BLOCK_TIME_MINUTES);
        }
    }

    public boolean isBlocked(String username) {
        String key = "login_attempt:" + username;
        String attempts = redisTemplate.opsForValue().get(key);
        return attempts != null && Integer.parseInt(attempts) >= MAX_ATTEMPTS;
    }

    public void loginSucceeded(String username) {
        String key = "login_attempt:" + username;
        redisTemplate.delete(key);
    }
}
```

#### Hiver - 登录拦截与限制

```rust
use hiver_resilience::rate_limit::RateLimiter;
use hiver_observability::log::Logger;
use std::sync::Arc;
use tokio::sync::RwLock;

// 1. 登录日志服务
#[service]
pub struct LoginLogService {
    #[autowired]
    logger: Arc<Logger>,

    #[autowired]
    repository: Arc<LoginLogRepository>,
}

impl LoginLogService {
    pub async fn log_success(&self, username: &str, ip: &str, user_agent: &str) {
        let log = LoginLog {
            id: 0,
            username: username.to_string(),
            ip: ip.to_string(),
            user_agent: user_agent.to_string(),
            status: LoginStatus::Success,
            login_time: Utc::now(),
            failure_reason: None,
        };

        self.repository.save(log).await;

        self.logger.info()
            .field("username", username)
            .field("ip", ip)
            .field("event", "login_success")
            .message("User logged in successfully")
            .log();
    }

    pub async fn log_failure(&self, username: &str, ip: &str, user_agent: &str, reason: &str) {
        let log = LoginLog {
            id: 0,
            username: username.to_string(),
            ip: ip.to_string(),
            user_agent: user_agent.to_string(),
            status: LoginStatus::Failed,
            login_time: Utc::now(),
            failure_reason: Some(reason.to_string()),
        };

        self.repository.save(log).await;

        self.logger.warn()
            .field("username", username)
            .field("ip", ip)
            .field("event", "login_failed")
            .field("reason", reason)
            .message("User login failed")
            .log();
    }
}

// 2. 登录尝试限制服务
#[service]
pub struct LoginAttemptService {
    attempts: Arc<RwLock<HashMap<String, LoginAttempts>>>,
    max_attempts: u32,
    block_duration: Duration,
}

#[derive(Clone)]
struct LoginAttempts {
    count: u32,
    last_attempt: DateTime<Utc>,
    blocked_until: Option<DateTime<Utc>>,
}

impl LoginAttemptService {
    pub fn new() -> Self {
        Self {
            attempts: Arc::new(RwLock::new(HashMap::new())),
            max_attempts: 5,
            block_duration: Duration::minutes(30),
        }
    }

    pub async fn login_failed(&self, username: &str) {
        let mut attempts = self.attempts.write().await;
        let entry = attempts.entry(username.to_string()).or_insert_with(|| LoginAttempts {
            count: 0,
            last_attempt: Utc::now(),
            blocked_until: None,
        });

        entry.count += 1;
        entry.last_attempt = Utc::now();

        if entry.count >= self.max_attempts {
            entry.blocked_until = Some(Utc::now() + self.block_duration);
            log::warn!(
                "User {} is blocked for {} minutes due to too many failed attempts",
                username, self.block_duration.num_minutes()
            );
        }
    }

    pub async fn is_blocked(&self, username: &str) -> bool {
        let attempts = self.attempts.read().await;
        if let Some(entry) = attempts.get(username) {
            if let Some(blocked_until) = entry.blocked_until {
                if Utc::now() < blocked_until {
                    return true;
                }
            }
        }
        false
    }

    pub async fn login_succeeded(&self, username: &str) {
        let mut attempts = self.attempts.write().await;
        attempts.remove(username);
    }
}

// 3. 登录拦截器
pub struct LoginInterceptor {
    attempt_service: Arc<LoginAttemptService>,
    log_service: Arc<LoginLogService>,
}

impl LoginInterceptor {
    pub async fn intercept_login(
        &self,
        username: &str,
        password: &str,
        ip: &str,
        user_agent: &str,
    ) -> Result<LoginResponse, AuthError> {
        // 检查是否被阻止
        if self.attempt_service.is_blocked(username).await {
            self.log_service.log_failure(username, ip, user_agent, "账户已锁定").await;
            return Err(AuthError::AccountBlocked);
        }

        // 执行认证
        match self.authenticate(username, password).await {
            Ok(response) => {
                self.attempt_service.login_succeeded(username).await;
                self.log_service.log_success(username, ip, user_agent).await;
                Ok(response)
            }
            Err(e) => {
                self.attempt_service.login_failed(username).await;
                self.log_service.log_failure(username, ip, user_agent, &e.to_string()).await;
                Err(e)
            }
        }
    }

    async fn authenticate(&self, username: &str, password: &str) -> Result<LoginResponse, AuthError> {
        // 实际认证逻辑
        // ...
    }
}
```

---

## 第11章：接口文档生成

### Swagger/OpenAPI 对比 / Swagger/OpenAPI Comparison

#### Spring Boot - Swagger 3 配置

```java
// 1. 添加依赖
// implementation 'org.springdoc:springdoc-openapi-starter-webmvc-ui:2.0.2'

// 2. OpenAPI 配置
@Configuration
public class OpenApiConfig {

    @Bean
    public OpenAPI openAPI() {
        return new OpenAPI()
            .info(new Info()
                .title("My API Documentation")
                .description("API 接口文档")
                .version("v1.0.0")
                .contact(new Contact()
                    .name("API Support")
                    .email("support@example.com"))
                .license(new License()
                    .name("Apache 2.0")
                    .url("https://www.apache.org/licenses/LICENSE-2.0.html")))
            .externalDocs(new ExternalDocumentation()
                .description("更多文档")
                .url("https://example.com/docs"))
            .addSecurityItem(new SecurityRequirement()
                .addList("bearerAuth"))
            .components(new Components()
                .addSecuritySchemes("bearerAuth",
                    new SecurityScheme()
                        .type(SecurityScheme.Type.HTTP)
                        .scheme("bearer")
                        .bearerFormat("JWT")))
            .addServersItem(new Server()
                .url("http://localhost:8080")
                .description("开发环境"))
            .addServersItem(new Server()
                .url("https://api.example.com")
                .description("生产环境"));
    }
}

// 3. 接口文档注解
@RestController
@RequestMapping("/api/users")
@Tag(name = "用户管理", description = "用户相关接口")
public class UserController {

    @Operation(summary = "获取用户列表", description = "分页获取所有用户")
    @ApiResponses({
        @ApiResponse(responseCode = "200", description = "成功",
            content = @Content(schema = @Schema(implementation = PageResult.class))),
        @ApiResponse(responseCode = "401", description = "未认证"),
        @ApiResponse(responseCode = "403", description = "无权限")
    })
    @Parameters({
        @Parameter(name = "page", description = "页码", example = "0"),
        @Parameter(name = "size", description = "每页数量", example = "10"),
        @Parameter(name = "sort", description = "排序字段", example = "id,desc")
    })
    @GetMapping
    public PageResult<User> listUsers(
        @Parameter(description = "页码") @RequestParam(defaultValue = "0") int page,
        @Parameter(description = "每页数量") @RequestParam(defaultValue = "10") int size
    ) {
        return userService.findAll(page, size);
    }

    @Operation(summary = "获取用户详情", description = "根据 ID 获取用户")
    @ApiResponses({
        @ApiResponse(responseCode = "200", description = "成功"),
        @ApiResponse(responseCode = "404", description = "用户不存在")
    })
    @GetMapping("/{id}")
    public User getUser(@Parameter(description = "用户ID") @PathVariable Long id) {
        return userService.findById(id);
    }

    @Operation(summary = "创建用户", description = "创建新用户")
    @PostMapping
    public User createUser(
        @io.swagger.v3.oas.annotations.parameters.RequestBody(
            description = "用户信息",
            required = true,
            content = @Content(schema = @Schema(implementation = CreateUserRequest.class))
        )
        @RequestBody @Valid CreateUserRequest request
    ) {
        return userService.create(request);
    }
}

// 4. 模型文档
@Schema(description = "用户信息")
@Data
public class User {
    @Schema(description = "用户ID", example = "1")
    private Long id;

    @Schema(description = "用户名", example = "alice", required = true)
    @NotBlank
    private String username;

    @Schema(description = "邮箱", example = "alice@example.com", required = true)
    @Email
    private String email;

    @Schema(description = "创建时间", example = "2023-01-01T00:00:00Z")
    private LocalDateTime createdAt;
}

@Schema(description = "创建用户请求")
@Data
public class CreateUserRequest {
    @Schema(description = "用户名", example = "alice", required = true)
    @NotBlank
    @Size(min = 3, max = 20)
    private String username;

    @Schema(description = "邮箱", example = "alice@example.com", required = true)
    @NotBlank
    @Email
    private String email;

    @Schema(description = "密码", example = "password123", required = true)
    @NotBlank
    @Size(min = 6, max = 20)
    private String password;
}
```

#### Hiver - OpenAPI 实现

```rust
use utoipa::{OpenApi, OpenApiSpec, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use hiver_macros::{controller, get, post};

// 1. OpenAPI 配置
#[derive(OpenApi)]
#[openapi(
    info(
        title = "My API Documentation",
        version = "1.0.0",
        description = "API 接口文档",
        contact(
            name = "API Support",
            email = "support@example.com"
        ),
        license(
            name = "Apache 2.0",
            url = "https://www.apache.org/licenses/LICENSE-2.0.html"
        )
    ),
    paths(
        list_users,
        get_user,
        create_user,
    ),
    components(
        schemas(User, CreateUserRequest, Error)
    ),
    tags(
        (name = "users", description = "用户管理"),
    ),
    servers(
        (url = "http://localhost:8080", description = "开发环境"),
        (url = "https://api.example.com", description = "生产环境")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
struct ApiDoc;

// 2. 安全方案定义
#[derive(utoipa::ToSchema)]
struct BearerAuth {
    #[schema(format = "Bearer {token}")]
    token: String}

// 3. 模型文档
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(description = "用户信息")]
pub struct User {
    #[schema(description = "用户ID", example = "1")]
    pub id: i64,

    #[schema(description = "用户名", example = "alice")]
    pub username: String,

    #[schema(description = "邮箱", example = "alice@example.com")]
    pub email: String,

    #[schema(description = "创建时间", example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[schema(description = "创建用户请求")]
pub struct CreateUserRequest {
    #[schema(description = "用户名", example = "alice", min_length = 3, max_length = 20)]
    pub username: String,

    #[schema(description = "邮箱", example = "alice@example.com", format = "email")]
    pub email: String,

    #[schema(description = "密码", example = "password123", min_length = 6, max_length = 20)]
    pub password: String,
}

// 4. 接口文档注解
#[controller]
struct UserController;

/// 获取用户列表
///
/// 分页获取所有用户
#[utoipa::path(
    get,
    path = "/api/users",
    tag = "users",
    params(
        ("page" = u32, Query, description = "页码", default = "0"),
        ("size" = u32, Query, description = "每页数量", default = "10"),
    ),
    responses(
        (status = 200, description = "成功", body = PageResult<User>),
        (status = 401, description = "未认证"),
        (status = 403, description = "无权限")
    ),
    security(("bearer_auth" = []))
)]
#[get("/api/users")]
async fn list_users(
    #[query] page: Option<u32>,
    #[query] size: Option<u32>,
    #[state] service: Arc<UserService>,
) -> Json<PageResult<User>> {
    let page = page.unwrap_or(0);
    let size = size.unwrap_or(10);
    Json(service.find_all(page, size).await)
}

/// 获取用户详情
///
/// 根据 ID 获取用户
#[utoipa::path(
    get,
    path = "/api/users/{id}",
    tag = "users",
    params(
        ("id" = i64, Path, description = "用户ID"),
    ),
    responses(
        (status = 200, description = "成功", body = User),
        (status = 404, description = "用户不存在")
    ),
    security(("bearer_auth" = []))
)]
#[get("/api/users/:id")]
async fn get_user(
    id: i64,
    #[state] service: Arc<UserService>,
) -> Result<Json<User>, Error> {
    Ok(Json(service.find_by_id(id).await?))
}

/// 创建用户
///
/// 创建新用户
#[utoipa::path(
    post,
    path = "/api/users",
    tag = "users",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "成功", body = User),
        (status = 400, description = "参数错误")
    ),
    security(("bearer_auth" = []))
)]
#[post("/api/users")]
async fn create_user(
    #[request_body] request: CreateUserRequest,
    #[state] service: Arc<UserService>,
) -> Result<Json<User>, Error> {
    Ok(Json(service.create(request).await?))
}

// 5. 添加 Swagger UI 路由
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        // API 路由
        .get("/api/users", list_users)
        .get("/api/users/:id", get_user)
        .post("/api/users", create_user)

        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", ApiDoc::openapi())
            .into_router());

    Server::bind("127.0.0.1:8080")
        .serve(app)
        .await?;

    Ok(())
}
```

---

## 第12章：日志系统配置与项目监控

### 日志系统对比 / Logging System Comparison

#### Spring Boot - Logback 配置

```xml
<!-- logback-spring.xml -->
<configuration>
    <!-- 控制台输出 -->
    <appender name="CONSOLE" class="ch.qos.logback.core.ConsoleAppender">
        <encoder>
            <pattern>%d{yyyy-MM-dd HH:mm:ss.SSS} [%thread] %-5level %logger{36} - %msg%n</pattern>
            <charset>UTF-8</charset>
        </encoder>
    </appender>

    <!-- 文件输出 -->
    <appender name="FILE" class="ch.qos.logback.core.rolling.RollingFileAppender">
        <file>logs/application.log</file>
        <encoder>
            <pattern>%d{yyyy-MM-dd HH:mm:ss.SSS} [%thread] %-5level %logger{36} - %msg%n</pattern>
            <charset>UTF-8</charset>
        </encoder>
        <rollingPolicy class="ch.qos.logback.core.rolling.TimeBasedRollingPolicy">
            <fileNamePattern>logs/application-%d{yyyy-MM-dd}.%i.log</fileNamePattern>
            <maxHistory>30</maxHistory>
            <timeBasedFileNamingAndTriggeringPolicy class="ch.qos.logback.core.rolling.SizeAndTimeBasedFNATP">
                <maxFileSize>100MB</maxFileSize>
            </timeBasedFileNamingAndTriggeringPolicy>
        </rollingPolicy>
    </appender>

    <!-- 错误日志单独输出 -->
    <appender name="ERROR_FILE" class="ch.qos.logback.core.rolling.RollingFileAppender">
        <file>logs/error.log</file>
        <filter class="ch.qos.logback.classic.filter.LevelFilter">
            <level>ERROR</level>
            <onMatch>ACCEPT</onMatch>
            <onMismatch>DENY</onMismatch>
        </filter>
        <encoder>
            <pattern>%d{yyyy-MM-dd HH:mm:ss.SSS} [%thread] %-5level %logger{36} - %msg%n</pattern>
        </encoder>
        <rollingPolicy class="ch.qos.logback.core.rolling.TimeBasedRollingPolicy">
            <fileNamePattern>logs/error-%d{yyyy-MM-dd}.log</fileNamePattern>
            <maxHistory>30</maxHistory>
        </rollingPolicy>
    </appender>

    <!-- 异步输出 -->
    <appender name="ASYNC_FILE" class="ch.qos.logback.classic.AsyncAppender">
        <appender-ref ref="FILE"/>
        <queueSize>512</queueSize>
        <discardingThreshold>0</discardingThreshold>
    </appender>

    <!-- Logger 配置 -->
    <logger name="com.example" level="DEBUG"/>
    <logger name="org.springframework" level="INFO"/>
    <logger name="org.hibernate" level="INFO"/>

    <!-- 根 Logger -->
    <root level="INFO">
        <appender-ref ref="CONSOLE"/>
        <appender-ref ref="ASYNC_FILE"/>
        <appender-ref ref="ERROR_FILE"/>
    </root>
</configuration>
```

#### Hiver - 日志配置

```rust
use hiver_observability::log::{Logger, LoggerFactory, Level, LogConfig};
use hiver_macros::config;

#[config(prefix = "logging")]
pub struct LoggingConfig {
    #[config(default = "INFO")]
    pub level: String,

    #[config(default = "logs/application.log")]
    pub file: String,

    #[config(default = "30")]
    pub max_history: i32,

    #[config(default = "100MB")]
    pub max_file_size: String,

    #[config(default = "true")]
    pub console_enabled: bool,

    #[config(default = "true")]
    pub async_enabled: bool,
}

impl LoggerFactory {
    pub fn configure(config: &LoggingConfig) -> Self {
        let level = match config.level.to_uppercase().as_str() {
            "TRACE" => Level::Trace,
            "DEBUG" => Level::Debug,
            "INFO" => Level::Info,
            "WARN" => Level::Warn,
            "ERROR" => Level::Error,
            _ => Level::Info,
        };

        let log_config = LogConfig::builder()
            .level(level)
            .console_output(config.console_enabled)
            .file_output(&config.file)
            .max_history(config.max_history)
            .max_file_size(parse_size(&config.max_file_size))
            .async_logging(config.async_enabled)
            .build();

        LoggerFactory::new(log_config)
    }
}

// 使用日志
#[service]
pub struct UserService {
    #[autowired]
    logger: Arc<Logger>,
}

impl UserService {
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User, Error> {
        self.logger.info()
            .field("username", &request.username)
            .field("email", &request.email)
            .message("Creating user")
            .log();

        match self.repository.create(&request).await {
            Ok(user) => {
                self.logger.info()
                    .field("user_id", user.id)
                    .message("User created successfully")
                    .log();
                Ok(user)
            }
            Err(e) => {
                self.logger.error()
                    .field("error", &e.to_string())
                    .message("Failed to create user")
                    .log();
                Err(Error::internal("Failed to create user"))
            }
        }
    }
}
```

### 监控端点对比 / Monitoring Endpoints Comparison

#### Spring Boot - Actuator

```yaml
# application.yml
management:
  endpoints:
    web:
      exposure:
        include: health,info,metrics,prometheus,env,beans
      base-path: /actuator
  endpoint:
    health:
      show-details: when-authorized
      show-components: always
      probes:
        enabled: true
  health:
    redis:
      enabled: true
    db:
      enabled: true
  metrics:
    export:
      prometheus:
        enabled: true
    tags:
      application: ${spring.application.name}
    distribution:
      percentiles-histogram:
        http.server.requests: true
      percentiles:
        http.server.requests: 0.5,0.95,0.99
```

```java
// 自定义健康检查
@Component
public class CustomHealthIndicator implements HealthIndicator {

    @Autowired
    private ExternalServiceClient externalServiceClient;

    @Override
    public Health health() {
        try {
            // 检查外部服务
            boolean isHealthy = externalServiceClient.ping();

            if (isHealthy) {
                return Health.up()
                    .withDetail("externalService", "Available")
                    .build();
            } else {
                return Health.down()
                    .withDetail("externalService", "Unavailable")
                    .build();
            }
        } catch (Exception e) {
            return Health.down()
                .withDetail("error", e.getMessage())
                .build();
        }
    }
}

// 自定义指标
@Component
public class CustomMetrics {

    private final Counter userCreatedCounter;
    private final Timer userCreationTimer;

    public CustomMetrics(MeterRegistry registry) {
        this.userCreatedCounter = Counter.builder("users.created")
            .description("Total number of users created")
            .tag("type", "regular")
            .register(registry);

        this.userCreationTimer = Timer.builder("users.creation.time")
            .description("Time taken to create a user")
            .publishPercentiles(0.5, 0.95, 0.99)
            .register(registry);
    }

    public void recordUserCreation() {
        userCreatedCounter.increment();
    }

    public void recordCreationTime(Runnable creation) {
        userCreationTimer.record(creation);
    }
}
```

#### Hiver - Observability 模块

```rust
use hiver_observability::{
    metrics::{Counter, Histogram, Gauge},
    health::{HealthChecker, HealthStatus},
    trace::{Tracer, Span},
};
use prometheus::{Encoder, TextEncoder};

// 1. 健康检查端点
#[get("/health")]
async fn health_check(
    #[state] health_checkers: Arc<Vec<Arc<dyn HealthChecker>>>,
) -> Json<serde_json::Value> {
    let mut health = serde_json::Map::new();

    for checker in health_checkers.iter() {
        let status = checker.check().await;
        health.insert(
            checker.name(),
            serde_json::json!({
                "status": match status {
                    HealthStatus::Healthy => "UP",
                    HealthStatus::Unhealthy => "DOWN",
                    HealthStatus::Degraded => "DEGRADED",
                },
                "details": status.details(),
            })
        );
    }

    Json(serde_json::json!({
        "status": if health.values().all(|v| v["status"] == "UP") { "UP" } else { "DOWN" },
        "components": health
    }))
}

// 2. 指标端点 (Prometheus 格式)
#[get("/metrics")]
async fn metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

// 3. 自定义健康检查
#[service]
pub struct DatabaseHealthChecker {
    #[autowired]
    db: Arc<Database>,
}

#[async_trait]
impl HealthChecker for DatabaseHealthChecker {
    fn name(&self) -> &str {
        "database"
    }

    async fn check(&self) -> HealthStatus {
        match self.db.ping().await {
            Ok(_) => HealthStatus::new_healthy()
                .with_detail("connection", "OK"),
            Err(e) => HealthStatus::new_unhealthy()
                .with_detail("error", e.to_string()),
        }
    }
}

// 4. 自定义指标
#[service]
pub struct UserMetrics {
    users_created: Arc<Counter>,
    creation_time: Arc<Histogram>,
    active_users: Arc<Gauge>,
}

impl UserMetrics {
    pub fn new() -> Self {
        let users_created = Arc::new(
            Counter::new("users_created_total", "Total users created")
        );

        let creation_time = Arc::new(
            Histogram::with_opts(HistogramOpts {
                name: "user_creation_seconds".to_string(),
                help: "User creation time".to_string(),
                buckets: vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
            })
        );

        let active_users = Arc::new(
            Gauge::new("active_users", "Number of active users")
        );

        Self {
            users_created,
            creation_time,
            active_users,
        }
    }

    pub fn record_user_created(&self) {
        self.users_created.inc();
    }

    pub fn record_creation_time<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.creation_time.observe_closure_duration(f)
    }

    pub fn set_active_users(&self, count: f64) {
        self.active_users.set(count);
    }
}

// 5. 分布式追踪
#[service]
pub struct UserService {
    #[autowired]
    tracer: Arc<Tracer>,
}

impl UserService {
    #[span(name = "create_user")]
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User, Error> {
        // 自动创建 span
        let user = self.repository.create(&request).await?;

        // 添加事件
        self.tracer.current_span().add_event("user_created", vec![
            ("user_id", user.id.to_string()),
            ("username", &user.username),
        ]);

        Ok(user)
    }
}
```

---

## 功能对比总结 / Summary

### 进阶功能对比 / Advanced Features Comparison

| 功能 / Feature | Spring Boot | Hiver | 完成度 |
|----------------|-------------|-------|--------|
| **JWT 认证 / JWT Auth** | Spring Security | hiver-security | ✅ 90% |
| **权限控制 / RBAC** | @PreAuthorize | #[require_permission] | ⚠️ 75% |
| **登录拦截 / Login Intercept** | @Aspect | Interceptor | ⚠️ 70% |
| **登录限制 / Rate Limit** | 自定义 | hiver-resilience | ✅ 90% |
| **API 文档 / Swagger** | springdoc-openapi | utoipa | ⚠️ 80% |
| **日志系统 / Logging** | Logback | hiver-observability | ✅ 85% |
| **健康检查 / Health Check** | Actuator | HealthChecker | ✅ 85% |
| **指标收集 / Metrics** | Micrometer | Prometheus | ✅ 85% |
| **分布式追踪 / Tracing** | Spring Cloud Sleuth | hiver-observability | ✅ 85% |

### 待补充功能 / Features to Add

1. **完善权限控制**
   - 实现更细粒度的权限检查
   - 支持数据权限过滤
   - 实现动态权限加载

2. **增强 API 文档**
   - 自动生成请求示例
   - 支持多语言文档
   - 集成 API 测试功能

3. **完善监控体系**
   - 添加更多开箱即用的指标
   - 实现告警规则
   - 支持日志聚合查询
