# Rust特有挑战解决方案
# Rust-Specific Challenges Solutions

**生成日期 / Generated Date**: 2026-01-24  
**目标 / Objective**: 使用Rust特性和开源库解决Spring Framework移植中的挑战

---

## 目录 / Table of Contents

1. [反射机制缺失解决方案](#1-反射机制缺失解决方案)
2. [AOP实现困难解决方案](#2-aop实现困难解决方案)
3. [循环依赖处理解决方案](#3-循环依赖处理解决方案)
4. [异步上下文传递解决方案](#4-异步上下文传递解决方案)
5. [实现示例代码](#5-实现示例代码)

---

## 1. 反射机制缺失解决方案 / Reflection Mechanism Solutions

### 1.1 问题分析 / Problem Analysis

**Spring方式**:
```java
// 使用反射动态创建Bean
Class<?> clazz = Class.forName(beanClassName);
Constructor<?> constructor = clazz.getConstructor();
Object bean = constructor.newInstance();

// 动态调用方法
Method method = clazz.getMethod("methodName", String.class);
method.invoke(bean, "arg");
```

**Rust挑战**:
- ❌ 无运行时反射
- ❌ 无法动态创建类型
- ❌ 无法动态调用方法

### 1.2 解决方案1: 使用bevy_reflect / Solution 1: Using bevy_reflect

**bevy_reflect** 是Bevy游戏引擎使用的反射库，功能强大且活跃维护。

#### 依赖添加 / Dependencies

```toml
# Cargo.toml
[dependencies]
bevy_reflect = { version = "0.13", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
```

#### 实现示例 / Implementation Example

```rust
// hiver-core/src/reflect.rs

use bevy_reflect::{Reflect, TypeRegistry, TypeInfo, StructInfo};
use std::any::{Any, TypeId};

/// 类型注册表（类似Spring的BeanDefinitionRegistry）
/// Type registry (similar to Spring's BeanDefinitionRegistry)
pub struct ReflectTypeRegistry {
    registry: TypeRegistry,
    bean_factories: HashMap<TypeId, Box<dyn BeanFactory>>,
}

impl ReflectTypeRegistry {
    pub fn new() -> Self {
        let mut registry = TypeRegistry::default();
        
        // 注册常用类型
        registry.register::<String>();
        registry.register::<i32>();
        registry.register::<bool>();
        
        Self {
            registry,
            bean_factories: HashMap::new(),
        }
    }
    
    /// 注册类型（等价于Spring的@Component扫描）
    /// Register type (equivalent to Spring's @Component scanning)
    pub fn register_type<T: Reflect + 'static>(&mut self) {
        self.registry.register::<T>();
    }
    
    /// 注册Bean工厂
    /// Register bean factory
    pub fn register_bean_factory<T: Reflect + 'static>(
        &mut self,
        factory: impl BeanFactory + 'static,
    ) {
        let type_id = TypeId::of::<T>();
        self.bean_factories.insert(type_id, Box::new(factory));
    }
    
    /// 动态创建Bean（使用反射）
    /// Dynamically create bean (using reflection)
    pub fn create_bean_dynamic(&self, type_name: &str) -> Result<Box<dyn Reflect>> {
        // 查找类型信息
        let type_info = self.registry.get_type_info_by_name(type_name)
            .ok_or_else(|| Error::new("Type not found"))?;
        
        // 如果是结构体，尝试创建实例
        if let Some(struct_info) = type_info.as_struct() {
            // 使用工厂创建
            if let Some(factory) = self.bean_factories.get(&type_info.type_id()) {
                return factory.create();
            }
            
            // 尝试使用默认值创建
            return self.create_default_instance(struct_info);
        }
        
        Err(Error::new("Cannot create instance"))
    }
    
    /// 获取字段值（动态访问）
    /// Get field value (dynamic access)
    pub fn get_field_value(&self, instance: &dyn Reflect, field_name: &str) -> Option<Box<dyn Reflect>> {
        if let Some(struct_reflect) = instance.as_struct() {
            return struct_reflect.field(field_name).map(|f| f.clone_value());
        }
        None
    }
    
    /// 设置字段值（动态设置）
    /// Set field value (dynamic set)
    pub fn set_field_value(&mut self, instance: &mut dyn Reflect, field_name: &str, value: Box<dyn Reflect>) -> Result<()> {
        if let Some(struct_reflect) = instance.as_struct_mut() {
            struct_reflect.field_mut(field_name)
                .map(|f| *f = value)
                .ok_or_else(|| Error::new("Field not found"))?;
            return Ok(());
        }
        Err(Error::new("Not a struct"))
    }
}

/// Bean工厂trait
/// Bean factory trait
pub trait BeanFactory: Send + Sync {
    fn create(&self) -> Result<Box<dyn Reflect>>;
}

/// 使用示例
/// Usage example
#[derive(Reflect, Debug)]
struct UserService {
    #[reflect(ignore)]
    repository: Arc<UserRepository>,
}

impl UserService {
    fn new(repository: Arc<UserRepository>) -> Self {
        Self { repository }
    }
}

// 注册类型
let mut registry = ReflectTypeRegistry::new();
registry.register_type::<UserService>();

// 注册工厂
registry.register_bean_factory::<UserService>(UserServiceFactory {
    repository: Arc::new(UserRepository::new()),
});

// 动态创建
let bean = registry.create_bean_dynamic("UserService")?;
```

#### bevy_reflect的优势 / Advantages

- ✅ **类型安全**: 编译时类型检查
- ✅ **性能**: 零成本抽象（大部分操作在编译时完成）
- ✅ **功能完整**: 支持字段访问、方法调用、序列化
- ✅ **活跃维护**: Bevy项目使用，持续更新

### 1.3 解决方案2: 使用typetag / Solution 2: Using typetag

**typetag** 提供类型擦除的序列化，适合需要序列化的场景。

```rust
// hiver-core/src/typetag_bean.rs

use typetag::{serde::Serialize, Deserialize};

/// 可序列化的Bean trait
/// Serializable bean trait
#[typetag::serde(tag = "type")]
pub trait SerializableBean: Send + Sync {
    fn bean_name(&self) -> &str;
    fn bean_type(&self) -> &str;
}

#[derive(Serialize, Deserialize)]
struct UserService {
    name: String,
}

#[typetag::serde(name = "UserService")]
impl SerializableBean for UserService {
    fn bean_name(&self) -> &str {
        "userService"
    }
    
    fn bean_type(&self) -> &str {
        "UserService"
    }
}

// 序列化和反序列化
let service = UserService { name: "test".to_string() };
let json = serde_json::to_string(&service as &dyn SerializableBean)?;
let deserialized: Box<dyn SerializableBean> = serde_json::from_str(&json)?;
```

### 1.4 解决方案3: 使用Rust特性替代反射 / Solution 3: Using Rust Features

**最佳实践**: 使用trait、泛型和宏替代反射。

#### Trait对象 / Trait Objects

```rust
// hiver-core/src/trait_bean.rs

/// Bean trait（所有Bean必须实现）
/// Bean trait (all beans must implement)
pub trait Bean: Send + Sync + 'static {
    fn bean_name(&self) -> &str;
    fn bean_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

/// Bean工厂trait
/// Bean factory trait
pub trait BeanFactory<T: Bean>: Send + Sync {
    fn create(&self, container: &Container) -> Result<Arc<T>>;
}

/// 类型擦除的Bean工厂
/// Type-erased bean factory
pub struct ErasedBeanFactory {
    type_id: TypeId,
    factory: Box<dyn Fn(&Container) -> Result<Arc<dyn Bean>> + Send + Sync>,
}

impl ErasedBeanFactory {
    pub fn new<T: Bean, F>(factory: F) -> Self
    where
        F: Fn(&Container) -> Result<Arc<T>> + Send + Sync + 'static,
    {
        Self {
            type_id: TypeId::of::<T>(),
            factory: Box::new(move |c| {
                factory(c).map(|bean| bean as Arc<dyn Bean>)
            }),
        }
    }
    
    pub fn create(&self, container: &Container) -> Result<Arc<dyn Bean>> {
        (self.factory)(container)
    }
}

// 使用示例
let factory = ErasedBeanFactory::new(|c| {
    let repo: Arc<UserRepository> = c.get_bean()?;
    Ok(Arc::new(UserService::new(repo)))
});
```

#### 宏生成 / Macro Generation

```rust
// hiver-macros/src/bean.rs

/// #[bean]宏自动实现Bean trait
/// #[bean] macro automatically implements Bean trait
#[proc_macro_attribute]
pub fn bean(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;
    
    let expanded = quote! {
        #input
        
        impl Bean for #name {
            fn bean_name(&self) -> &str {
                stringify!(#name)
            }
        }
    };
    
    TokenStream::from(expanded)
}

// 使用
#[bean]
struct UserService {
    repository: Arc<UserRepository>,
}
```

### 1.5 推荐方案 / Recommended Solution

**混合方案**:
1. **基础Bean管理**: 使用trait和泛型（零成本）
2. **动态类型操作**: 使用bevy_reflect（需要时）
3. **序列化场景**: 使用typetag（配置持久化）

---

## 2. AOP实现困难解决方案 / AOP Implementation Solutions

### 2.1 问题分析 / Problem Analysis

**Spring方式**:
```java
// 使用JDK动态代理或CGLIB
@Aspect
@Component
public class LoggingAspect {
    @Around("execution(* com.example.service.*.*(..))")
    public Object log(ProceedingJoinPoint pjp) throws Throwable {
        System.out.println("Before: " + pjp.getSignature());
        Object result = pjp.proceed();
        System.out.println("After: " + result);
        return result;
    }
}
```

**Rust挑战**:
- ❌ 无运行时代理
- ❌ 无法动态拦截方法调用

### 2.2 解决方案1: 使用过程宏实现AOP / Solution 1: Procedural Macros

#### 基础实现 / Basic Implementation

```rust
// hiver-macros/src/aop.rs

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Expr};

/// #[around]宏实现环绕通知
/// #[around] macro implements around advice
#[proc_macro_attribute]
pub fn around(attr: TokenStream, item: TokenStream) -> TokenStream {
    let pointcut = parse_macro_input!(attr as Expr);
    let function = parse_macro_input!(item as ItemFn);
    
    let fn_name = &function.sig.ident;
    let fn_vis = &function.vis;
    let fn_async = &function.sig.asyncness;
    let fn_inputs = &function.sig.inputs;
    let fn_output = &function.sig.output;
    let fn_block = &function.block;
    
    let expanded = quote! {
        #fn_vis #fn_async fn #fn_name(#fn_inputs) #fn_output {
            // Before advice
            println!("Before: {}", stringify!(#fn_name));
            
            // 执行原方法
            // Execute original method
            let result = async move {
                #fn_block
            };
            
            // After advice
            let result = result.await;
            println!("After: {:?}", result);
            result
        }
    };
    
    TokenStream::from(expanded)
}

// 使用示例
#[around]
async fn create_user(user: User) -> Result<User> {
    // 原方法逻辑
    Ok(user)
}
```

#### 高级实现：支持切点表达式 / Advanced: Pointcut Expression

```rust
// hiver-macros/src/aop_advanced.rs

use syn::{parse_macro_input, ItemFn, LitStr};

/// #[aspect]宏定义切面
/// #[aspect] macro defines aspect
#[proc_macro_attribute]
pub fn aspect(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    // 生成切面结构体
    TokenStream::from(quote! { #input })
}

/// #[before]宏实现前置通知
/// #[before] macro implements before advice
#[proc_macro_attribute]
pub fn before(attr: TokenStream, item: TokenStream) -> TokenStream {
    let pointcut = parse_macro_input!(attr as LitStr);
    let function = parse_macro_input!(item as ItemFn);
    
    // 解析切点表达式（简化版）
    // Parse pointcut expression (simplified)
    let pointcut_str = pointcut.value();
    
    // 生成包装代码
    // Generate wrapper code
    let fn_name = &function.sig.ident;
    let fn_vis = &function.vis;
    let fn_async = &function.sig.asyncness;
    let fn_inputs = &function.sig.inputs;
    let fn_output = &function.sig.output;
    let fn_block = &function.block;
    
    let expanded = quote! {
        #fn_vis #fn_async fn #fn_name(#fn_inputs) #fn_output {
            // Before advice logic
            tracing::info!("Before: {} - {}", stringify!(#fn_name), #pointcut_str);
            
            // Execute original method
            #fn_block
        }
    };
    
    TokenStream::from(expanded)
}

/// #[after]宏实现后置通知
/// #[after] macro implements after advice
#[proc_macro_attribute]
pub fn after(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 类似实现
}

/// #[around]宏实现环绕通知
/// #[around] macro implements around advice
#[proc_macro_attribute]
pub fn around(attr: TokenStream, item: TokenStream) -> TokenStream {
    let pointcut = parse_macro_input!(attr as LitStr);
    let function = parse_macro_input!(item as ItemFn);
    
    let fn_name = &function.sig.ident;
    let fn_vis = &function.vis;
    let fn_async = &function.sig.asyncness;
    let fn_inputs = &function.sig.inputs;
    let fn_output = &function.sig.output;
    let fn_block = &function.block;
    
    let expanded = quote! {
        #fn_vis #fn_async fn #fn_name(#fn_inputs) #fn_output {
            // Before
            let start = std::time::Instant::now();
            tracing::info!("Entering: {}", stringify!(#fn_name));
            
            // Execute
            let result = async move {
                #fn_block
            }.await;
            
            // After
            let duration = start.elapsed();
            tracing::info!("Exiting: {} - took {:?}", stringify!(#fn_name), duration);
            
            result
        }
    };
    
    TokenStream::from(expanded)
}

// 使用示例
#[aspect]
struct LoggingAspect;

impl LoggingAspect {
    #[before("execution(*Service::*")]
    async fn log_before() {
        tracing::info!("Before method execution");
    }
    
    #[around("execution(*Service::*")]
    async fn log_around() {
        // 环绕逻辑
    }
}
```

### 2.3 解决方案2: 使用trait和组合模式 / Solution 2: Trait and Composition

```rust
// hiver-core/src/aop_trait.rs

/// 可拦截的trait
/// Interceptable trait
pub trait Interceptable {
    type Input;
    type Output;
    
    async fn execute(&self, input: Self::Input) -> Self::Output;
}

/// 拦截器trait
/// Interceptor trait
pub trait Interceptor<T: Interceptable>: Send + Sync {
    async fn intercept(
        &self,
        target: &T,
        input: T::Input,
        chain: &mut InterceptorChain<T>,
    ) -> T::Output;
}

/// 拦截器链
/// Interceptor chain
pub struct InterceptorChain<T: Interceptable> {
    interceptors: Vec<Arc<dyn Interceptor<T>>>,
    current: usize,
}

impl<T: Interceptable> InterceptorChain<T> {
    pub async fn proceed(&mut self, target: &T, input: T::Input) -> T::Output {
        if self.current < self.interceptors.len() {
            let interceptor = &self.interceptors[self.current];
            self.current += 1;
            interceptor.intercept(target, input, self).await
        } else {
            target.execute(input).await
        }
    }
}

// 使用示例
struct LoggingInterceptor;

impl<T: Interceptable> Interceptor<T> for LoggingInterceptor {
    async fn intercept(
        &self,
        target: &T,
        input: T::Input,
        chain: &mut InterceptorChain<T>,
    ) -> T::Output {
        tracing::info!("Before execution");
        let result = chain.proceed(target, input).await;
        tracing::info!("After execution");
        result
    }
}
```

### 2.4 解决方案3: 使用aspect-rs库 / Solution 3: Using aspect-rs

**aspect-rs** 是专门的Rust AOP库。

```rust
// Cargo.toml
[dependencies]
aspect-rs = "0.1"

// 使用
use aspect_rs::{Aspect, Pointcut, Advice};

#[derive(Aspect)]
struct LoggingAspect;

impl LoggingAspect {
    #[Pointcut("execution(*Service::*")]
    fn service_methods() {}
    
    #[Advice(OnEnter)]
    fn log_enter(&self) {
        tracing::info!("Entering method");
    }
    
    #[Advice(OnResult)]
    fn log_result(&self, result: &dyn std::fmt::Debug) {
        tracing::info!("Result: {:?}", result);
    }
}
```

### 2.5 推荐方案 / Recommended Solution

**混合方案**:
1. **简单场景**: 使用过程宏（`#[transactional]`, `#[cacheable]`）
2. **复杂场景**: 使用trait和组合模式
3. **通用AOP**: 考虑集成aspect-rs

---

## 3. 循环依赖处理解决方案 / Circular Dependency Solutions

### 3.1 问题分析 / Problem Analysis

**Spring方式**:
```java
// 三级缓存解决循环依赖
// Level 1: singletonObjects (完全初始化)
// Level 2: earlySingletonObjects (提前暴露)
// Level 3: singletonFactories (工厂对象)
```

**Rust挑战**:
- Rust的所有权系统使循环依赖更复杂
- 需要使用`Arc`和`Weak`引用

### 3.2 解决方案1: 使用Arc和Weak / Solution 1: Arc and Weak

```rust
// hiver-core/src/circular_dependency.rs

use std::sync::{Arc, Weak};
use std::collections::HashMap;
use std::any::{Any, TypeId};

/// Bean创建状态
/// Bean creation state
enum BeanState<T> {
    /// 正在创建（用于检测循环）
    /// Creating (for cycle detection)
    Creating,
    
    /// 已创建（单例）
    /// Created (singleton)
    Created(Arc<T>),
    
    /// 提前暴露的引用（Weak）
    /// Early exposed reference (Weak)
    EarlyExposed(Weak<T>),
}

/// 支持循环依赖的容器
/// Container supporting circular dependencies
pub struct CircularAwareContainer {
    /// Bean存储
    /// Bean storage
    singletons: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    
    /// 创建状态（用于检测循环）
    /// Creation state (for cycle detection)
    creating: std::cell::RefCell<HashSet<TypeId>>,
    
    /// 提前暴露的Bean（Weak引用）
    /// Early exposed beans (Weak references)
    early_exposed: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl CircularAwareContainer {
    pub fn new() -> Self {
        Self {
            singletons: HashMap::new(),
            creating: std::cell::RefCell::new(HashSet::new()),
            early_exposed: HashMap::new(),
        }
    }
    
    /// 获取Bean（支持循环依赖）
    /// Get bean (supporting circular dependencies)
    pub fn get_bean<T: Bean + Send + Sync + 'static>(&self) -> Result<Arc<T>> {
        let type_id = TypeId::of::<T>();
        
        // 检查是否正在创建（循环依赖检测）
        // Check if currently creating (circular dependency detection)
        {
            let mut creating = self.creating.borrow_mut();
            if creating.contains(&type_id) {
                // 循环依赖：返回提前暴露的引用
                // Circular dependency: return early exposed reference
                if let Some(early) = self.early_exposed.get(&type_id) {
                    if let Ok(weak) = early.downcast_ref::<Weak<T>>() {
                        return weak.upgrade()
                            .ok_or_else(|| Error::new("Bean was dropped during creation"));
                    }
                }
                return Err(Error::new("Circular dependency detected"));
            }
            creating.insert(type_id);
        }
        
        // 检查是否已存在
        // Check if already exists
        if let Some(bean) = self.singletons.get(&type_id) {
            if let Ok(arc) = bean.downcast_ref::<Arc<T>>() {
                return Ok(Arc::clone(arc));
            }
        }
        
        // 创建Bean
        // Create bean
        let factory = self.get_factory::<T>()?;
        
        // 提前暴露Weak引用
        // Early expose Weak reference
        let (arc, weak) = {
            // 先创建Weak引用占位
            // Create Weak reference placeholder first
            let placeholder = Arc::new(std::mem::MaybeUninit::<T>::uninit());
            let weak = Arc::downgrade(&placeholder);
            
            // 存储Weak引用
            // Store Weak reference
            self.early_exposed.insert(type_id, Box::new(weak.clone()));
            
            // 创建实际Bean
            // Create actual bean
            let bean = factory.create(self)?;
            let arc = Arc::new(bean);
            
            // 更新Weak引用指向实际Bean
            // Update Weak reference to point to actual bean
            (arc.clone(), Arc::downgrade(&arc))
        };
        
        // 存储单例
        // Store singleton
        self.singletons.insert(type_id, Box::new(arc.clone()));
        
        // 移除创建标记
        // Remove creation marker
        self.creating.borrow_mut().remove(&type_id);
        
        Ok(arc)
    }
}

// 使用示例
struct ServiceA {
    service_b: Arc<ServiceB>,
}

struct ServiceB {
    service_a: Weak<ServiceA>,  // 使用Weak避免循环
}

let container = CircularAwareContainer::new();
let service_a: Arc<ServiceA> = container.get_bean()?;
```

### 3.3 解决方案2: 延迟初始化 / Solution 2: Lazy Initialization

```rust
// hiver-core/src/lazy_bean.rs

use std::sync::OnceLock;

/// 延迟初始化的Bean
/// Lazy initialized bean
pub struct LazyBean<T> {
    value: OnceLock<Arc<T>>,
    factory: Box<dyn Fn() -> Result<Arc<T>> + Send + Sync>,
}

impl<T> LazyBean<T> {
    pub fn new<F>(factory: F) -> Self
    where
        F: Fn() -> Result<Arc<T>> + Send + Sync + 'static,
    {
        Self {
            value: OnceLock::new(),
            factory: Box::new(factory),
        }
    }
    
    pub fn get(&self) -> Result<Arc<T>> {
        self.value.get_or_try_init(|| (self.factory)())
            .map(Arc::clone)
    }
}

// 使用示例
struct ServiceA {
    service_b: LazyBean<ServiceB>,
}

struct ServiceB {
    service_a: LazyBean<ServiceA>,
}

// 延迟初始化避免循环依赖
// Lazy initialization avoids circular dependencies
```

### 3.4 解决方案3: 依赖注入重构 / Solution 3: Dependency Injection Refactoring

**最佳实践**: 重构代码避免循环依赖。

```rust
// 反模式：循环依赖
// Anti-pattern: circular dependency
struct UserService {
    order_service: Arc<OrderService>,
}

struct OrderService {
    user_service: Arc<UserService>,  // 循环依赖
}

// 解决方案1：提取共同依赖
// Solution 1: Extract common dependency
struct UserService {
    repository: Arc<UserRepository>,
}

struct OrderService {
    repository: Arc<OrderRepository>,
    user_repository: Arc<UserRepository>,  // 直接依赖Repository
}

// 解决方案2：使用事件/消息
// Solution 2: Use events/messages
struct UserService {
    event_bus: Arc<EventBus>,
}

struct OrderService {
    event_bus: Arc<EventBus>,
}

// 通过事件通信，避免直接依赖
// Communicate through events, avoid direct dependency
```

### 3.5 推荐方案 / Recommended Solution

**混合方案**:
1. **首选**: 重构代码避免循环依赖
2. **必须时**: 使用`Arc`和`Weak`引用
3. **复杂场景**: 使用延迟初始化

---

## 4. 异步上下文传递解决方案 / Async Context Propagation Solutions

### 4.1 问题分析 / Problem Analysis

**Spring方式**:
```java
// ThreadLocal存储
SecurityContextHolder.getContext().setAuthentication(auth);
TransactionSynchronizationManager.getCurrentTransactionName();
```

**Rust挑战**:
- 异步任务可能在不同线程恢复
- ThreadLocal无法跨await点传递

### 4.2 解决方案1: 使用tokio::task_local / Solution 1: Using tokio::task_local

```rust
// hiver-security/src/async_context.rs

use tokio::task_local;

/// 任务本地SecurityContext
/// Task-local SecurityContext
task_local! {
    static SECURITY_CONTEXT: Arc<RwLock<Option<Authentication>>>;
}

/// 设置SecurityContext
/// Set SecurityContext
pub async fn set_security_context(auth: Authentication) {
    SECURITY_CONTEXT.scope(Arc::new(RwLock::new(Some(auth))), async {
        // 在这个作用域内，SecurityContext可用
        // Within this scope, SecurityContext is available
        get_current_user().await;
    }).await;
}

/// 获取当前认证
/// Get current authentication
pub async fn get_current_authentication() -> Option<Authentication> {
    SECURITY_CONTEXT.try_with(|ctx| {
        ctx.read().await.clone()
    }).ok().flatten()
}

// 使用示例
async fn handler() -> Result<Response> {
    let auth = authenticate().await?;
    
    SECURITY_CONTEXT.scope(Arc::new(RwLock::new(Some(auth))), async {
        // 在这个作用域内可以访问SecurityContext
        // Can access SecurityContext within this scope
        let user = get_current_user().await;
        Ok(Response::json(user))
    }).await
}
```

### 4.3 解决方案2: 使用async-local库 / Solution 2: Using async-local

```rust
// Cargo.toml
[dependencies]
async-local = "1.0"

// hiver-security/src/async_local_context.rs

use async_local::LocalRef;
use std::sync::Arc;
use tokio::sync::RwLock;

/// SecurityContext的本地引用
/// Local reference to SecurityContext
thread_local! {
    static SECURITY_CONTEXT: Arc<RwLock<Option<Authentication>>> = Arc::new(RwLock::new(None));
}

/// 获取SecurityContext的LocalRef
/// Get LocalRef to SecurityContext
pub fn security_context_ref() -> LocalRef<Arc<RwLock<Option<Authentication>>>> {
    LocalRef::new(&SECURITY_CONTEXT)
}

/// 设置SecurityContext
/// Set SecurityContext
pub async fn set_security_context(auth: Authentication) {
    SECURITY_CONTEXT.with(|ctx| {
        let mut guard = ctx.write().await;
        *guard = Some(auth);
    });
}

/// 获取当前认证（跨await点）
/// Get current authentication (across await points)
pub async fn get_current_authentication() -> Option<Authentication> {
    let ctx_ref = security_context_ref();
    ctx_ref.with(|ctx| {
        ctx.read().await.clone()
    }).await
}
```

### 4.4 解决方案3: 使用Request扩展 / Solution 3: Using Request Extensions

**推荐方案**: 通过Request传递上下文。

```rust
// hiver-http/src/request_context.rs

use std::sync::Arc;
use tokio::sync::RwLock;

/// Request扩展中的SecurityContext
/// SecurityContext in Request extensions
pub struct SecurityContextExtension {
    authentication: Arc<RwLock<Option<Authentication>>>,
}

impl SecurityContextExtension {
    pub fn new() -> Self {
        Self {
            authentication: Arc::new(RwLock::new(None)),
        }
    }
    
    pub async fn set_authentication(&self, auth: Authentication) {
        *self.authentication.write().await = Some(auth);
    }
    
    pub async fn get_authentication(&self) -> Option<Authentication> {
        self.authentication.read().await.clone()
    }
}

/// 中间件：从Request提取SecurityContext
/// Middleware: Extract SecurityContext from Request
pub struct SecurityContextMiddleware;

impl Middleware for SecurityContextMiddleware {
    async fn handle(&self, req: Request, next: Next) -> Response {
        // 从Request获取SecurityContext
        // Get SecurityContext from Request
        let ctx = req.extensions()
            .get::<SecurityContextExtension>()
            .cloned()
            .unwrap_or_else(SecurityContextExtension::new);
        
        // 设置到全局（如果需要）
        // Set to global (if needed)
        set_global_security_context(ctx.clone()).await;
        
        next.run(req).await
    }
}

// 使用示例
async fn handler(req: Request) -> Result<Response> {
    // 从Request获取SecurityContext
    // Get SecurityContext from Request
    let ctx = req.extensions()
        .get::<SecurityContextExtension>()
        .ok_or_else(|| Error::new("No security context"))?;
    
    let auth = ctx.get_authentication().await;
    Ok(Response::json(auth))
}
```

### 4.5 解决方案4: 使用Context变量（类似Python） / Solution 4: Using Context Variables

```rust
// hiver-core/src/context_var.rs

use std::cell::RefCell;
use std::sync::Arc;

/// 上下文变量（类似Python的contextvars）
/// Context variable (similar to Python's contextvars)
pub struct ContextVar<T> {
    default: Arc<T>,
    local: RefCell<Option<Arc<T>>>,
}

impl<T: Clone> ContextVar<T> {
    pub fn new(default: T) -> Self {
        Self {
            default: Arc::new(default),
            local: RefCell::new(None),
        }
    }
    
    pub fn get(&self) -> Arc<T> {
        self.local.borrow()
            .as_ref()
            .cloned()
            .unwrap_or_else(|| Arc::clone(&self.default))
    }
    
    pub fn set(&self, value: T) {
        *self.local.borrow_mut() = Some(Arc::new(value));
    }
    
    pub fn reset(&self) {
        *self.local.borrow_mut() = None;
    }
}

// 全局SecurityContext
// Global SecurityContext
thread_local! {
    static SECURITY_CONTEXT_VAR: ContextVar<Option<Authentication>> = 
        ContextVar::new(None);
}

pub fn get_security_context() -> Option<Authentication> {
    SECURITY_CONTEXT_VAR.with(|var| var.get().as_ref().clone())
}

pub fn set_security_context(auth: Authentication) {
    SECURITY_CONTEXT_VAR.with(|var| var.set(Some(auth)));
}
```

### 4.6 推荐方案 / Recommended Solution

**混合方案**:
1. **Request级别**: 使用Request扩展（最简单、最清晰）
2. **任务级别**: 使用`tokio::task_local`（需要任务隔离时）
3. **全局访问**: 使用async-local（需要全局访问时）

**Hiver当前实现改进**:

```rust
// hiver-security/src/context.rs (改进版)

use std::sync::Arc;
use tokio::sync::RwLock;

/// 改进的SecurityContext（使用Request扩展）
/// Improved SecurityContext (using Request extensions)
pub struct SecurityContext {
    authentication: Arc<RwLock<Option<Authentication>>>,
}

impl SecurityContext {
    pub fn new() -> Self {
        Self {
            authentication: Arc::new(RwLock::new(None)),
        }
    }
    
    /// 从Request获取SecurityContext
    /// Get SecurityContext from Request
    pub fn from_request(req: &Request) -> Option<Arc<Self>> {
        req.extensions().get::<Arc<SecurityContext>>().cloned()
    }
    
    /// 设置到Request
    /// Set to Request
    pub fn set_to_request(req: &mut Request) {
        req.extensions_mut().insert(Arc::new(Self::new()));
    }
}

/// 中间件：自动管理SecurityContext
/// Middleware: Automatically manage SecurityContext
pub struct SecurityContextMiddleware;

impl Middleware for SecurityContextMiddleware {
    async fn handle(&self, mut req: Request, next: Next) -> Response {
        // 创建SecurityContext
        // Create SecurityContext
        let ctx = Arc::new(SecurityContext::new());
        req.extensions_mut().insert(ctx.clone());
        
        // 执行下一个中间件
        // Execute next middleware
        let response = next.run(req).await;
        
        response
    }
}

/// 便捷函数：从Request获取认证
/// Convenience function: Get authentication from Request
pub async fn get_authentication_from_request(req: &Request) -> Option<Authentication> {
    SecurityContext::from_request(req)
        .and_then(|ctx| {
            // 使用blocking方式获取（简化示例）
            // Use blocking way to get (simplified example)
            Some(ctx.authentication.blocking_read().clone()?)
        })
}
```

---

## 5. 实现示例代码 / Implementation Examples

### 5.1 完整的Bean容器实现（支持反射和循环依赖）/ Complete Bean Container

```rust
// hiver-core/src/advanced_container.rs

use bevy_reflect::{Reflect, TypeRegistry};
use std::sync::{Arc, Weak};
use std::collections::{HashMap, HashSet};
use std::any::{Any, TypeId};

/// 高级Bean容器
/// Advanced bean container
pub struct AdvancedContainer {
    /// 类型注册表
    /// Type registry
    type_registry: TypeRegistry,
    
    /// Bean存储
    /// Bean storage
    singletons: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    
    /// Bean工厂
    /// Bean factories
    factories: HashMap<TypeId, Box<dyn BeanFactory>>,
    
    /// 创建状态（循环依赖检测）
    /// Creation state (circular dependency detection)
    creating: std::cell::RefCell<HashSet<TypeId>>,
    
    /// 提前暴露的Bean
    /// Early exposed beans
    early_exposed: HashMap<TypeId, Weak<dyn Any + Send + Sync>>,
}

impl AdvancedContainer {
    pub fn new() -> Self {
        Self {
            type_registry: TypeRegistry::default(),
            singletons: HashMap::new(),
            factories: HashMap::new(),
            creating: std::cell::RefCell::new(HashSet::new()),
            early_exposed: HashMap::new(),
        }
    }
    
    /// 注册类型（使用bevy_reflect）
    /// Register type (using bevy_reflect)
    pub fn register_type<T: Reflect + 'static>(&mut self) {
        self.type_registry.register::<T>();
    }
    
    /// 注册Bean工厂
    /// Register bean factory
    pub fn register_factory<T: Bean + Send + Sync + 'static>(
        &mut self,
        factory: impl BeanFactory + 'static,
    ) {
        let type_id = TypeId::of::<T>();
        self.factories.insert(type_id, Box::new(factory));
    }
    
    /// 获取Bean（支持循环依赖）
    /// Get bean (supporting circular dependencies)
    pub fn get_bean<T: Bean + Send + Sync + 'static>(&self) -> Result<Arc<T>> {
        let type_id = TypeId::of::<T>();
        
        // 检查循环依赖
        // Check circular dependency
        {
            let mut creating = self.creating.borrow_mut();
            if creating.contains(&type_id) {
                // 循环依赖：返回提前暴露的引用
                // Circular dependency: return early exposed reference
                if let Some(weak) = self.early_exposed.get(&type_id) {
                    // 尝试升级Weak引用
                    // Try to upgrade Weak reference
                    // 注意：这里需要类型转换，简化示例
                    // Note: Type conversion needed here, simplified example
                    return Err(Error::new("Circular dependency - use Weak reference"));
                }
                return Err(Error::new("Circular dependency detected"));
            }
            creating.insert(type_id);
        }
        
        // 检查是否已存在
        // Check if already exists
        if let Some(bean) = self.singletons.get(&type_id) {
            if let Ok(typed) = bean.clone().downcast::<T>() {
                self.creating.borrow_mut().remove(&type_id);
                return Ok(typed);
            }
        }
        
        // 创建Bean
        // Create bean
        let factory = self.factories.get(&type_id)
            .ok_or_else(|| Error::new("Factory not found"))?;
        
        // 提前暴露Weak引用占位
        // Early expose Weak reference placeholder
        let (arc, weak) = {
            // 创建占位符
            // Create placeholder
            let placeholder: Arc<T> = factory.create(self)?;
            let weak = Arc::downgrade(&placeholder);
            
            // 存储Weak引用
            // Store Weak reference
            self.early_exposed.insert(type_id, weak.clone());
            
            (placeholder.clone(), weak)
        };
        
        // 存储单例
        // Store singleton
        self.singletons.insert(type_id, arc.clone());
        
        // 移除创建标记
        // Remove creation marker
        self.creating.borrow_mut().remove(&type_id);
        
        Ok(arc)
    }
}
```

### 5.2 完整的AOP实现 / Complete AOP Implementation

```rust
// hiver-macros/src/complete_aop.rs

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitStr};

/// 完整的#[transactional]宏实现
/// Complete #[transactional] macro implementation
#[proc_macro_attribute]
pub fn transactional(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as syn::AttributeArgs);
    let function = parse_macro_input!(item as ItemFn);
    
    // 解析属性（propagation, isolation等）
    // Parse attributes (propagation, isolation, etc.)
    let propagation = parse_propagation(&attrs);
    let isolation = parse_isolation(&attrs);
    
    let fn_name = &function.sig.ident;
    let fn_vis = &function.vis;
    let fn_async = &function.sig.asyncness;
    let fn_inputs = &function.sig.inputs;
    let fn_output = &function.sig.output;
    let fn_block = &function.block;
    
    let expanded = quote! {
        #fn_vis #fn_async fn #fn_name(#fn_inputs) #fn_output {
            use hiver_tx::{TransactionManager, TransactionDefinition, Propagation, IsolationLevel};
            
            // 获取事务管理器
            // Get transaction manager
            let tx_manager = get_transaction_manager();
            
            // 创建事务定义
            // Create transaction definition
            let definition = TransactionDefinition::new(stringify!(#fn_name))
                .propagation(#propagation)
                .isolation(#isolation);
            
            // 开始事务
            // Begin transaction
            let status = tx_manager.begin(&definition).await?;
            
            // 执行原方法
            // Execute original method
            let result = async move {
                #fn_block
            }.await;
            
            // 根据结果提交或回滚
            // Commit or rollback based on result
            match &result {
                Ok(_) => tx_manager.commit(status).await?,
                Err(_) => tx_manager.rollback(status).await?,
            }
            
            result
        }
    };
    
    TokenStream::from(expanded)
}

// 使用示例
#[transactional(propagation = "REQUIRED", isolation = "READ_COMMITTED")]
async fn create_user(user: User) -> Result<User> {
    // 方法逻辑
    Ok(user)
}
```

### 5.3 完整的异步上下文实现 / Complete Async Context Implementation

```rust
// hiver-security/src/complete_async_context.rs

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task_local;

/// 任务本地SecurityContext
/// Task-local SecurityContext
task_local! {
    static TASK_SECURITY_CONTEXT: Arc<RwLock<Option<Authentication>>>;
}

/// SecurityContext管理器
/// SecurityContext manager
pub struct SecurityContextManager;

impl SecurityContextManager {
    /// 在作用域内设置SecurityContext
    /// Set SecurityContext within scope
    pub async fn with_context<F, R>(auth: Authentication, f: F) -> R
    where
        F: std::future::Future<Output = R>,
    {
        TASK_SECURITY_CONTEXT.scope(
            Arc::new(RwLock::new(Some(auth))),
            f
        ).await
    }
    
    /// 获取当前认证
    /// Get current authentication
    pub async fn get_current() -> Option<Authentication> {
        TASK_SECURITY_CONTEXT.try_with(|ctx| {
            ctx.read().await.clone()
        }).ok().flatten()
    }
    
    /// 设置当前认证
    /// Set current authentication
    pub async fn set_current(auth: Authentication) -> Result<()> {
        TASK_SECURITY_CONTEXT.try_with(|mut ctx| {
            *ctx.write().await = Some(auth);
            Ok(())
        }).ok().flatten().unwrap_or(Err(Error::new("No context")))
    }
}

// 使用示例
async fn handler() -> Result<Response> {
    let auth = authenticate().await?;
    
    SecurityContextManager::with_context(auth, async {
        // 在这个作用域内可以访问SecurityContext
        // Can access SecurityContext within this scope
        let user = SecurityContextManager::get_current().await;
        Ok(Response::json(user))
    }).await
}
```

---

## 6. 总结与建议 / Summary and Recommendations

### 6.1 技术选型总结 / Technology Selection Summary

| 挑战 | 推荐方案 | 备选方案 | 理由 |
|------|---------|---------|------|
| **反射** | bevy_reflect | typetag | 功能完整，性能好 |
| **AOP** | 过程宏 | aspect-rs | 零成本，编译时优化 |
| **循环依赖** | Arc + Weak | 重构代码 | Rust原生支持 |
| **异步上下文** | Request扩展 | tokio::task_local | 最简单清晰 |

### 6.2 实施优先级 / Implementation Priority

1. **P0 - 立即实施**:
   - ✅ 使用Request扩展实现异步上下文传递
   - ✅ 使用过程宏实现基础AOP（`#[transactional]`, `#[cacheable]`）

2. **P1 - Phase 2实施**:
   - ✅ 集成bevy_reflect支持动态Bean操作
   - ✅ 实现循环依赖检测和处理

3. **P2 - Phase 3实施**:
   - ✅ 完善AOP功能（支持切点表达式）
   - ✅ 优化异步上下文性能

### 6.3 代码迁移建议 / Code Migration Recommendations

**从Spring迁移到Hiver**:

1. **Bean定义**: 使用`#[bean]`宏替代`@Component`
2. **依赖注入**: 使用构造函数注入（Rust推荐方式）
3. **AOP**: 使用`#[transactional]`等宏替代`@Transactional`
4. **上下文**: 使用Request扩展替代ThreadLocal

---

**文档生成时间 / Document Generated**: 2026-01-24  
**更新建议 / Update Recommendation**: 实施时根据实际情况调整方案
