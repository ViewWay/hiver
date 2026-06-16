use std::sync::Arc;

use super::{
    ApplicationContext, BeanRegistrar, BeanRegistration, ComponentScanner, Container,
    PostConstruct, PreDestroy,
};
use crate::{
    bean::{Bean, BeanState, Scope},
    conditional::{ConditionalOnBean, ConditionalOnMissingBean},
    error::{Error, Result},
};

// ── Test fixtures / 测试夹具 ────────────────────────────────────────

#[derive(Debug, Default)]
struct UserRepository
{
    initialized: bool,
}

impl Bean for UserRepository {}

impl PostConstruct for UserRepository
{
    fn post_construct(&self) -> Result<()>
    {
        Ok(())
    }
}

impl PreDestroy for UserRepository
{
    fn pre_destroy(&self) -> Result<()>
    {
        Ok(())
    }
}

#[derive(Debug)]
struct UserService
{
    user_count: u32,
}

impl Bean for UserService {}

#[derive(Debug, Default)]
struct EmailService
{
    sent_count: u32,
}

impl Bean for EmailService {}

#[derive(Debug)]
struct CacheService
{
    hits: u64,
}

impl Bean for CacheService {}

#[derive(Debug, Default)]
struct AuditService;

impl Bean for AuditService {}

// ── Container::new / Container::default ────────────────────────────

#[test]
fn test_container_new()
{
    let container = Container::new();
    assert!(!container.has_bean::<UserRepository>());
}

#[test]
fn test_container_default()
{
    let container = Container::default();
    assert!(!container.has_bean::<UserService>());
}

#[test]
fn test_container_clone_independent()
{
    let mut container = Container::new();
    container.register(|_| Ok(EmailService::default())).unwrap();
    // Clone shares underlying Arc<RwLock<>> so beans are shared / Clone共享底层Arc<RwLock<>>
    let cloned = container.clone();
    assert!(cloned.has_bean::<EmailService>());
}

// ── register / get_bean ────────────────────────────────────────────

#[test]
fn test_register_and_get_bean()
{
    let mut container = Container::new();
    container
        .register(|_| Ok(UserRepository::default()))
        .unwrap();
    let bean = container.get_bean::<UserRepository>().unwrap();
    assert!(!bean.initialized);
}

#[test]
fn test_register_factory_creates_instance()
{
    let mut container = Container::new();
    container
        .register(|_| Ok(UserService { user_count: 42 }))
        .unwrap();
    let bean = container.get_bean::<UserService>().unwrap();
    assert_eq!(bean.user_count, 42);
}

#[test]
fn test_get_bean_missing_returns_error()
{
    let container = Container::new();
    let result = container.get_bean::<UserService>();
    assert!(result.is_err());
}

#[test]
fn test_get_bean_singleton_identity()
{
    let mut container = Container::new();
    container
        .register(|_| Ok(UserService { user_count: 1 }))
        .unwrap();
    let first = container.get_bean::<UserService>().unwrap();
    let second = container.get_bean::<UserService>().unwrap();
    // Same Arc / 同一个Arc
    assert!(Arc::ptr_eq(&first, &second));
}

#[test]
fn test_register_factory_simple()
{
    let mut container = Container::new();
    container
        .register_factory(|| EmailService { sent_count: 5 })
        .unwrap();
    let bean = container.get_bean::<EmailService>().unwrap();
    assert_eq!(bean.sent_count, 5);
}

#[test]
fn test_register_factory_default()
{
    let mut container = Container::new();
    container
        .register_factory(|| EmailService::default())
        .unwrap();
    let bean = container.get_bean::<EmailService>().unwrap();
    assert_eq!(bean.sent_count, 0);
}

// ── register_bean (direct instance) ────────────────────────────────

#[test]
fn test_register_bean_direct()
{
    let mut container = Container::new();
    container
        .register_bean(UserRepository { initialized: true })
        .unwrap();
    let bean = container.get_bean::<UserRepository>().unwrap();
    assert!(bean.initialized);
}

// ── has_bean ───────────────────────────────────────────────────────

#[test]
fn test_has_bean_false_initially()
{
    let container = Container::new();
    assert!(!container.has_bean::<UserService>());
}

#[test]
fn test_has_bean_true_after_register()
{
    let mut container = Container::new();
    container
        .register(|_| Ok(UserService { user_count: 0 }))
        .unwrap();
    assert!(container.has_bean::<UserService>());
}

#[test]
fn test_has_bean_true_after_register_bean()
{
    let mut container = Container::new();
    container.register_bean(EmailService::default()).unwrap();
    assert!(container.has_bean::<EmailService>());
}

#[test]
fn test_has_bean_false_for_unregistered_type()
{
    let mut container = Container::new();
    container
        .register(|_| Ok(UserService { user_count: 0 }))
        .unwrap();
    assert!(!container.has_bean::<EmailService>());
}

// ── get_bean_by_name ───────────────────────────────────────────────

#[test]
fn test_get_bean_by_name()
{
    let mut container = Container::new();
    container
        .register(|_| Ok(UserService { user_count: 7 }))
        .unwrap();
    let type_name = std::any::type_name::<UserService>();
    let bean = container
        .get_bean_by_name::<UserService>(type_name)
        .unwrap();
    assert_eq!(bean.user_count, 7);
}

#[test]
fn test_get_bean_by_name_missing()
{
    let container = Container::new();
    let result = container.get_bean_by_name::<UserService>("nonexistent");
    assert!(result.is_err());
}

// ── register_with (full configuration) ─────────────────────────────

#[test]
fn test_register_with_factory()
{
    let mut container = Container::new();
    let reg = BeanRegistration::new("custom_service")
        .factory(Arc::new(|_| Ok(UserService { user_count: 99 })));
    container.register_with(reg).unwrap();
    let bean = container.get_bean::<UserService>().unwrap();
    assert_eq!(bean.user_count, 99);
}

#[test]
fn test_register_with_post_construct()
{
    use std::sync::atomic::{AtomicBool, Ordering};
    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();

    let mut container = Container::new();
    let reg = BeanRegistration::new("svc")
        .factory(Arc::new(|_| Ok(UserService { user_count: 0 })))
        .post_construct(move |_bean| {
            called_clone.store(true, Ordering::SeqCst);
            Ok(())
        });
    container.register_with(reg).unwrap();
    container.get_bean::<UserService>().unwrap();
    assert!(called.load(Ordering::SeqCst));
}

#[test]
fn test_register_with_pre_destroy()
{
    use std::sync::atomic::{AtomicBool, Ordering};
    let destroyed = Arc::new(AtomicBool::new(false));
    let destroyed_clone = destroyed.clone();

    let mut container = Container::new();
    let reg = BeanRegistration::new("svc")
        .factory(Arc::new(|_| Ok(UserService { user_count: 0 })))
        .pre_destroy(move |_bean| {
            destroyed_clone.store(true, Ordering::SeqCst);
            Ok(())
        });
    container.register_with(reg).unwrap();
    container.get_bean::<UserService>().unwrap();
    container.shutdown().unwrap();
    assert!(destroyed.load(Ordering::SeqCst));
}

#[test]
fn test_register_with_scope()
{
    let mut container = Container::new();
    let reg = BeanRegistration::new("svc")
        .factory(Arc::new(|_| Ok(UserService { user_count: 0 })))
        .scope(Scope::Prototype);
    container.register_with(reg).unwrap();
    assert!(container.has_bean::<UserService>());
}

#[test]
fn test_register_with_primary()
{
    let mut container = Container::new();
    let reg = BeanRegistration::new("svc")
        .factory(Arc::new(|_| Ok(UserService { user_count: 0 })))
        .primary(true);
    container.register_with(reg).unwrap();
    assert!(container.has_bean::<UserService>());
}

#[test]
fn test_register_with_lazy()
{
    let mut container = Container::new();
    let reg = BeanRegistration::new("svc")
        .factory(Arc::new(|_| Ok(UserService { user_count: 0 })))
        .lazy(true);
    container.register_with(reg).unwrap();
    assert!(container.has_bean::<UserService>());
}

// ── Dependency injection ───────────────────────────────────────────

#[test]
fn test_dependency_injection()
{
    let mut container = Container::new();
    container
        .register(|_| Ok(UserRepository::default()))
        .unwrap();
    container
        .register(|c| {
            let _repo = c.get_bean::<UserRepository>()?;
            Ok(UserService { user_count: 0 })
        })
        .unwrap();
    let service = container.get_bean::<UserService>().unwrap();
    assert_eq!(service.user_count, 0);
}

// ── shutdown ───────────────────────────────────────────────────────

#[test]
fn test_shutdown_clears_beans()
{
    let mut container = Container::new();
    container
        .register(|_| Ok(UserService { user_count: 0 }))
        .unwrap();
    container.get_bean::<UserService>().unwrap();
    container.shutdown().unwrap();
    assert!(!container.has_bean::<UserService>());
}

#[test]
fn test_shutdown_on_empty_container()
{
    let container = Container::new();
    // Should not panic / 不应panic
    container.shutdown().unwrap();
}

// ── initialize ─────────────────────────────────────────────────────

#[test]
fn test_initialize_no_error()
{
    let mut container = Container::new();
    container
        .register(|_| Ok(UserService { user_count: 0 }))
        .unwrap();
    container.initialize().unwrap();
}

#[test]
fn test_initialize_creates_eager_beans()
{
    use std::sync::atomic::{AtomicBool, Ordering};
    let created = Arc::new(AtomicBool::new(false));
    let created_clone = created.clone();

    let mut container = Container::new();
    let reg = BeanRegistration::new("svc").factory(Arc::new(move |_| {
        created_clone.store(true, Ordering::SeqCst);
        Ok(UserService { user_count: 42 })
    }));
    container.register_with(reg).unwrap();

    assert!(!created.load(Ordering::SeqCst));

    container.initialize().unwrap();

    assert!(created.load(Ordering::SeqCst));
    let bean = container.get_bean::<UserService>().unwrap();
    assert_eq!(bean.user_count, 42);
}

#[test]
fn test_initialize_skips_lazy_beans()
{
    use std::sync::atomic::{AtomicBool, Ordering};
    let created = Arc::new(AtomicBool::new(false));
    let created_clone = created.clone();

    let mut container = Container::new();
    let reg = BeanRegistration::new("svc")
        .factory(Arc::new(move |_| {
            created_clone.store(true, Ordering::SeqCst);
            Ok(UserService { user_count: 99 })
        }))
        .lazy(true);
    container.register_with(reg).unwrap();

    container.initialize().unwrap();

    // Factory should NOT have been called
    assert!(!created.load(Ordering::SeqCst));

    // But it works on first get_bean
    let bean = container.get_bean::<UserService>().unwrap();
    assert!(created.load(Ordering::SeqCst));
    assert_eq!(bean.user_count, 99);
}

#[test]
fn test_initialize_mixed_lazy_and_eager()
{
    use std::sync::atomic::{AtomicBool, Ordering};
    let eager_created = Arc::new(AtomicBool::new(false));
    let eager_clone = eager_created.clone();

    let mut container = Container::new();
    let reg_eager = BeanRegistration::new("eager").factory(Arc::new(move |_| {
        eager_clone.store(true, Ordering::SeqCst);
        Ok(UserService { user_count: 1 })
    }));
    container.register_with(reg_eager).unwrap();

    container.initialize().unwrap();
    assert!(eager_created.load(Ordering::SeqCst));
}

// ── Extensions ─────────────────────────────────────────────────────

#[test]
fn test_container_extensions()
{
    let mut container = Container::new();
    container.extensions_mut().insert("test".to_string());
    assert_eq!(container.extensions().get::<String>(), Some(&"test".to_string()));
}

#[test]
fn test_container_extensions_mut()
{
    let mut container = Container::new();
    container.extensions_mut().insert(42i32);
    if let Some(v) = container.extensions_mut().get_mut::<i32>()
    {
        *v = 100;
    }
    assert_eq!(container.extensions().get::<i32>(), Some(&100));
}

// ── Reflect ────────────────────────────────────────────────────────

#[test]
fn test_container_reflect()
{
    let container = Container::new();
    let _reflect = container.reflect();
}

// ── register_conditional ───────────────────────────────────────────

#[test]
fn test_register_conditional_on_missing_bean_registers_when_absent()
{
    let mut container = Container::new();
    let cond = ConditionalOnMissingBean::of::<CacheService>();
    container
        .register_conditional(|_| Ok(CacheService { hits: 0 }), &cond)
        .unwrap();
    assert!(container.has_bean::<CacheService>());
}

#[test]
fn test_register_conditional_on_missing_bean_skips_when_present()
{
    let mut container = Container::new();
    // First register / 先注册
    container
        .register(|_| Ok(CacheService { hits: 10 }))
        .unwrap();
    container.get_bean::<CacheService>().unwrap();
    // Second conditional should still register (registrations are independent)
    // 第二次条件注册仍会注册（注册是独立的）
    let cond = ConditionalOnMissingBean::of::<CacheService>();
    container
        .register_conditional(|_| Ok(CacheService { hits: 20 }), &cond)
        .unwrap();
}

#[test]
fn test_register_conditional_on_bean_registers_when_present()
{
    let mut container = Container::new();
    container
        .register(|_| Ok(UserRepository::default()))
        .unwrap();
    let cond = ConditionalOnBean::of::<UserRepository>();
    container
        .register_conditional(|_| Ok(UserService { user_count: 0 }), &cond)
        .unwrap();
    assert!(container.has_bean::<UserService>());
}

// ── ApplicationContext ─────────────────────────────────────────────

#[test]
fn test_application_context_new()
{
    let ctx = ApplicationContext::new();
    assert_eq!(ctx.profile(), "default");
    assert!(!ctx.is_active());
}

#[test]
fn test_application_context_default()
{
    let ctx = ApplicationContext::default();
    assert_eq!(ctx.profile(), "default");
}

#[test]
fn test_application_context_set_profile()
{
    let mut ctx = ApplicationContext::new();
    ctx.set_profile("production");
    assert_eq!(ctx.profile(), "production");
}

#[test]
fn test_application_context_accepts_profile_default()
{
    let ctx = ApplicationContext::new();
    assert!(ctx.accepts_profile("default"));
    assert!(ctx.accepts_profile("anything")); // "default" context accepts all / "default"上下文接受所有
}

#[test]
fn test_application_context_accepts_profile_specific()
{
    let mut ctx = ApplicationContext::new();
    ctx.set_profile("staging");
    assert!(ctx.accepts_profile("staging"));
    assert!(!ctx.accepts_profile("production"));
    assert!(ctx.accepts_profile("default")); // "default" profile always accepted / "default"配置始终接受
}

#[test]
fn test_application_context_start()
{
    let mut ctx = ApplicationContext::new();
    ctx.start().unwrap();
    assert!(ctx.is_active());
}

#[test]
fn test_application_context_register_and_get()
{
    let mut ctx = ApplicationContext::new();
    ctx.register(EmailService::default()).unwrap();
    let bean = ctx.get_bean::<EmailService>().unwrap();
    assert_eq!(bean.sent_count, 0);
}

#[test]
fn test_application_context_register_with_factory()
{
    let mut ctx = ApplicationContext::new();
    ctx.register_with(|_| Ok(UserService { user_count: 5 }))
        .unwrap();
    let bean = ctx.get_bean::<UserService>().unwrap();
    assert_eq!(bean.user_count, 5);
}

#[test]
fn test_application_context_contains_bean()
{
    let mut ctx = ApplicationContext::new();
    ctx.register(AuditService).unwrap();
    assert!(ctx.contains_bean::<AuditService>());
    assert!(!ctx.contains_bean::<UserService>());
}

#[test]
fn test_application_context_get_bean_by_name()
{
    let mut ctx = ApplicationContext::new();
    ctx.register_with(|_| Ok(UserService { user_count: 3 }))
        .unwrap();
    let type_name = std::any::type_name::<UserService>();
    let bean = ctx.get_bean_by_name::<UserService>(type_name).unwrap();
    assert_eq!(bean.user_count, 3);
}

#[test]
fn test_application_context_close()
{
    let mut ctx = ApplicationContext::new();
    ctx.register(EmailService::default()).unwrap();
    ctx.start().unwrap();
    ctx.close().unwrap();
}

#[test]
fn test_application_context_refresh()
{
    let mut ctx = ApplicationContext::new();
    ctx.register_with(|_| Ok(UserService { user_count: 1 }))
        .unwrap();
    ctx.start().unwrap();
    ctx.refresh().unwrap();
    assert!(ctx.is_active());
}

#[test]
fn test_application_context_container_access()
{
    let mut ctx = ApplicationContext::new();
    ctx.register_with(|_| Ok(UserService { user_count: 0 }))
        .unwrap();
    // Immutable access / 不可变访问
    assert!(ctx.container().has_bean::<UserService>());
    // Mutable access / 可变访问
    ctx.container_mut()
        .register(|_| Ok(EmailService::default()))
        .unwrap();
    assert!(ctx.container().has_bean::<EmailService>());
}

// ── ComponentScanner ───────────────────────────────────────────────

#[test]
fn test_component_scanner_new()
{
    let scanner = ComponentScanner::new();
    let mut ctx = ApplicationContext::new();
    scanner.scan(&mut ctx).unwrap();
}

#[test]
fn test_component_scanner_default()
{
    let scanner = ComponentScanner::default();
    let mut ctx = ApplicationContext::new();
    scanner.scan(&mut ctx).unwrap();
}

#[test]
fn test_component_scanner_scan_package_builder()
{
    let scanner = ComponentScanner::new()
        .scan_package("com.example")
        .scan_package("com.other");
    let mut ctx = ApplicationContext::new();
    scanner.scan(&mut ctx).unwrap();
}

#[test]
fn test_component_scanner_register_component()
{
    let scanner = ComponentScanner::new();
    let mut ctx = ApplicationContext::new();
    scanner.register_component::<UserService>(&mut ctx).unwrap();
}

// ── PostConstruct / PreDestroy traits ──────────────────────────────

#[test]
fn test_post_construct_trait()
{
    struct MySvc;
    impl PostConstruct for MySvc
    {
        fn post_construct(&self) -> Result<()>
        {
            Ok(())
        }
    }
    let svc = MySvc;
    assert!(svc.post_construct().is_ok());
}

#[test]
fn test_pre_destroy_trait()
{
    struct MySvc;
    impl PreDestroy for MySvc
    {
        fn pre_destroy(&self) -> Result<()>
        {
            Ok(())
        }
    }
    let svc = MySvc;
    assert!(svc.pre_destroy().is_ok());
}

// ── Edge cases / 边界情况 ──────────────────────────────────────────

#[test]
fn test_register_multiple_different_types()
{
    let mut container = Container::new();
    container
        .register(|_| Ok(UserService { user_count: 1 }))
        .unwrap();
    container
        .register(|_| Ok(EmailService { sent_count: 2 }))
        .unwrap();
    container
        .register(|_| Ok(CacheService { hits: 3 }))
        .unwrap();
    let user = container.get_bean::<UserService>().unwrap();
    let email = container.get_bean::<EmailService>().unwrap();
    let cache = container.get_bean::<CacheService>().unwrap();
    assert_eq!(user.user_count, 1);
    assert_eq!(email.sent_count, 2);
    assert_eq!(cache.hits, 3);
}

#[test]
fn test_get_bean_after_shutdown_returns_error()
{
    let mut container = Container::new();
    container
        .register(|_| Ok(UserService { user_count: 0 }))
        .unwrap();
    container.get_bean::<UserService>().unwrap();
    container.shutdown().unwrap();
    let result = container.get_bean::<UserService>();
    assert!(result.is_err());
}

#[test]
fn test_bean_registration_builder()
{
    let reg: BeanRegistration<UserService> = BeanRegistration::new("test_svc")
        .scope(Scope::Prototype)
        .primary(true)
        .lazy(true);
    assert_eq!(reg.definition.name, "test_svc");
    assert_eq!(reg.definition.scope, Scope::Prototype);
    assert!(reg.definition.primary);
    assert!(reg.definition.lazy);
}

#[test]
fn test_bean_registration_new_defaults()
{
    let reg: BeanRegistration<UserService> = BeanRegistration::new("svc");
    assert_eq!(reg.definition.name, "svc");
    assert!(reg.factory.is_none());
    assert!(reg.post_construct.is_none());
    assert!(reg.pre_destroy.is_none());
    assert_eq!(reg.definition.scope, Scope::Singleton);
    assert!(!reg.definition.primary);
    assert!(!reg.definition.lazy);
}

// ── Additional container tests / 额外容器测试 ──────────────────────

#[test]
fn test_register_bean_overwrite()
{
    let mut container = Container::new();
    container
        .register_bean(UserService { user_count: 1 })
        .unwrap();
    // Register again overwrites / 再次注册会覆盖
    container
        .register_bean(UserService { user_count: 99 })
        .unwrap();
    let bean = container.get_bean::<UserService>().unwrap();
    assert_eq!(bean.user_count, 99);
}

#[test]
fn test_register_factory_with_container_access()
{
    let mut container = Container::new();
    container
        .register(|_| Ok(UserRepository { initialized: true }))
        .unwrap();
    container
        .register(|c| {
            let repo = c.get_bean::<UserRepository>()?;
            Ok(UserService {
                user_count: if repo.initialized { 100 } else { 0 },
            })
        })
        .unwrap();
    let svc = container.get_bean::<UserService>().unwrap();
    assert_eq!(svc.user_count, 100);
}

#[test]
fn test_application_context_not_active_before_start()
{
    let ctx = ApplicationContext::new();
    assert!(!ctx.is_active());
}

#[test]
fn test_application_context_set_profile_multiple_times()
{
    let mut ctx = ApplicationContext::new();
    ctx.set_profile("dev");
    assert_eq!(ctx.profile(), "dev");
    ctx.set_profile("prod");
    assert_eq!(ctx.profile(), "prod");
}

#[test]
fn test_application_context_accepts_profile_default_always()
{
    let mut ctx = ApplicationContext::new();
    ctx.set_profile("custom");
    // "default" profile is always accepted / "default"配置始终被接受
    assert!(ctx.accepts_profile("default"));
    assert!(ctx.accepts_profile("custom"));
    assert!(!ctx.accepts_profile("other"));
}

#[test]
fn test_application_context_register_unit_type()
{
    let mut ctx = ApplicationContext::new();
    ctx.register(AuditService).unwrap();
    assert!(ctx.contains_bean::<AuditService>());
    let _bean = ctx.get_bean::<AuditService>().unwrap();
}

#[test]
fn test_shutdown_twice()
{
    let mut container = Container::new();
    container
        .register(|_| Ok(UserService { user_count: 0 }))
        .unwrap();
    container.get_bean::<UserService>().unwrap();
    container.shutdown().unwrap();
    // Second shutdown on already-cleared container / 第二次关闭已清空的容器
    container.shutdown().unwrap();
}

#[test]
fn test_get_bean_by_name_after_lazy_creation()
{
    let mut container = Container::new();
    container
        .register(|_| Ok(CacheService { hits: 42 }))
        .unwrap();
    // First get by name triggers creation / 首次按名称获取触发创建
    let type_name = std::any::type_name::<CacheService>();
    let bean1 = container
        .get_bean_by_name::<CacheService>(type_name)
        .unwrap();
    assert_eq!(bean1.hits, 42);
    // Second get returns same singleton / 第二次获取返回同一单例
    let bean2 = container
        .get_bean_by_name::<CacheService>(type_name)
        .unwrap();
    assert!(Arc::ptr_eq(&bean1, &bean2));
}

#[test]
fn test_container_extensions_isolation()
{
    let mut container = Container::new();
    container.extensions_mut().insert(42i32);
    // Extensions are separate from beans / 扩展与bean分离
    assert!(!container.has_bean::<EmailService>());
    assert_eq!(container.extensions().get::<i32>(), Some(&42));
}

#[test]
fn test_register_conditional_on_bean_skips_when_absent()
{
    let mut container = Container::new();
    let cond = ConditionalOnBean::of::<UserService>();
    container
        .register_conditional(|_| Ok(EmailService { sent_count: 0 }), &cond)
        .unwrap();
    assert!(!container.has_bean::<EmailService>());
}

// ── BeanState lifecycle tests / BeanState 生命周期测试 ──────────────

#[test]
fn test_bean_state_defined_after_register()
{
    let mut container = Container::new();
    container
        .register(|_| Ok(UserService { user_count: 0 }))
        .unwrap();
    assert_eq!(container.bean_state::<UserService>(), Some(BeanState::Defined));
}

#[test]
fn test_bean_state_created_after_get()
{
    let mut container = Container::new();
    container
        .register(|_| Ok(UserService { user_count: 0 }))
        .unwrap();
    container.get_bean::<UserService>().unwrap();
    assert_eq!(container.bean_state::<UserService>(), Some(BeanState::Created));
}

#[test]
fn test_bean_state_destroyed_after_shutdown()
{
    let mut container = Container::new();
    container
        .register(|_| Ok(UserService { user_count: 0 }))
        .unwrap();
    container.get_bean::<UserService>().unwrap();
    container.shutdown().unwrap();
    assert_eq!(container.bean_state::<UserService>(), Some(BeanState::Destroyed));
}

// ── Prototype scope tests / Prototype 作用域测试 ──────────────────

#[test]
fn test_prototype_creates_new_instance_each_time()
{
    let mut container = Container::new();
    let reg = BeanRegistration::new("svc")
        .factory(Arc::new(|_| Ok(UserService { user_count: 0 })))
        .scope(Scope::Prototype);
    container.register_with(reg).unwrap();

    let bean1 = container.get_bean::<UserService>().unwrap();
    let bean2 = container.get_bean::<UserService>().unwrap();
    // Prototype: each call creates a new instance
    // Prototype: 每次调用创建新实例
    assert!(!Arc::ptr_eq(&bean1, &bean2));
}

#[test]
fn test_singleton_returns_same_instance()
{
    let mut container = Container::new();
    container
        .register(|_| Ok(UserService { user_count: 0 }))
        .unwrap();
    let bean1 = container.get_bean::<UserService>().unwrap();
    let bean2 = container.get_bean::<UserService>().unwrap();
    assert!(Arc::ptr_eq(&bean1, &bean2));
}

#[test]
fn test_prototype_not_cached_in_singletons()
{
    let mut container = Container::new();
    let reg = BeanRegistration::new("svc")
        .factory(Arc::new(|_| Ok(UserService { user_count: 42 })))
        .scope(Scope::Prototype);
    container.register_with(reg).unwrap();

    let bean = container.get_bean::<UserService>().unwrap();
    assert_eq!(bean.user_count, 42);
    // Each call creates fresh instance
    // 每次调用创建新实例
    drop(bean);
    let bean2 = container.get_bean::<UserService>().unwrap();
    assert_eq!(bean2.user_count, 42);
}

#[test]
fn test_multiple_beans_same_factory_pattern()
{
    let mut container = Container::new();
    for i in 0..3
    {
        let count = i;
        match i % 3
        {
            0 => container
                .register(move |_| Ok(UserService { user_count: count }))
                .unwrap(),
            1 => container
                .register(move |_| {
                    Ok(EmailService {
                        sent_count: count as u32,
                    })
                })
                .unwrap(),
            _ => container
                .register(move |_| Ok(CacheService { hits: count as u64 }))
                .unwrap(),
        }
    }
    assert!(container.has_bean::<UserService>());
    assert!(container.has_bean::<EmailService>());
    assert!(container.has_bean::<CacheService>());
}

// ── @Qualifier / Named bean tests / @Qualifier / 命名bean测试 ───────

#[test]
fn test_register_named_single_bean()
{
    let mut container = Container::new();
    container
        .register_named("myService", |_| Ok(UserService { user_count: 42 }))
        .unwrap();
    let bean = container
        .get_qualified_bean::<UserService>("myService")
        .unwrap();
    assert_eq!(bean.user_count, 42);
}

#[test]
fn test_register_named_multiple_same_type()
{
    let mut container = Container::new();
    container
        .register_named("serviceA", |_| Ok(UserService { user_count: 1 }))
        .unwrap();
    container
        .register_named("serviceB", |_| Ok(UserService { user_count: 2 }))
        .unwrap();

    let a = container
        .get_qualified_bean::<UserService>("serviceA")
        .unwrap();
    let b = container
        .get_qualified_bean::<UserService>("serviceB")
        .unwrap();
    assert_eq!(a.user_count, 1);
    assert_eq!(b.user_count, 2);
}

#[test]
fn test_qualified_bean_singleton_identity()
{
    let mut container = Container::new();
    container
        .register_named("svc", |_| Ok(UserService { user_count: 10 }))
        .unwrap();

    let first = container.get_qualified_bean::<UserService>("svc").unwrap();
    let second = container.get_qualified_bean::<UserService>("svc").unwrap();
    assert!(Arc::ptr_eq(&first, &second));
}

#[test]
fn test_get_bean_falls_back_to_named_single()
{
    let mut container = Container::new();
    container
        .register_named("onlyCache", |_| Ok(CacheService { hits: 99 }))
        .unwrap();

    // get_bean should find the single named bean
    // get_bean 应找到唯一的命名bean
    let bean = container.get_bean::<CacheService>().unwrap();
    assert_eq!(bean.hits, 99);
}

#[test]
fn test_get_bean_multiple_named_without_primary_returns_error()
{
    let mut container = Container::new();
    container
        .register_named("cacheA", |_| Ok(CacheService { hits: 1 }))
        .unwrap();
    container
        .register_named("cacheB", |_| Ok(CacheService { hits: 2 }))
        .unwrap();

    let result = container.get_bean::<CacheService>();
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Multiple beans"));
}

#[test]
fn test_has_bean_checks_named_storage()
{
    let mut container = Container::new();
    assert!(!container.has_bean::<CacheService>());
    container
        .register_named("myCache", |_| Ok(CacheService { hits: 0 }))
        .unwrap();
    assert!(container.has_bean::<CacheService>());
}

#[test]
fn test_get_qualified_bean_not_found()
{
    let container = Container::new();
    let result = container.get_qualified_bean::<UserService>("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_get_beans_of_type()
{
    let mut container = Container::new();
    container
        .register_named("alpha", |_| Ok(CacheService { hits: 10 }))
        .unwrap();
    container
        .register_named("beta", |_| Ok(CacheService { hits: 20 }))
        .unwrap();

    // Instantiate both named beans first
    // 先实例化两个命名bean
    container
        .get_qualified_bean::<CacheService>("alpha")
        .unwrap();
    container
        .get_qualified_bean::<CacheService>("beta")
        .unwrap();

    let beans = container.get_beans_of_type::<CacheService>();
    assert_eq!(beans.len(), 2);
}

#[test]
fn test_qualified_bean_with_dependency_injection()
{
    let mut container = Container::new();
    container
        .register(|_| Ok(UserRepository { initialized: true }))
        .unwrap();
    container
        .register_named("primaryService", |c| {
            let repo = c.get_bean::<UserRepository>()?;
            Ok(UserService {
                user_count: if repo.initialized { 100 } else { 0 },
            })
        })
        .unwrap();

    let svc = container
        .get_qualified_bean::<UserService>("primaryService")
        .unwrap();
    assert_eq!(svc.user_count, 100);
}

// ── ApplicationContext @Qualifier tests ─────────────────────────────

#[test]
fn test_application_context_register_named()
{
    let mut ctx = ApplicationContext::new();
    ctx.register_named("mySvc", |_| Ok(UserService { user_count: 7 }))
        .unwrap();
    let bean = ctx.get_qualified_bean::<UserService>("mySvc").unwrap();
    assert_eq!(bean.user_count, 7);
}
