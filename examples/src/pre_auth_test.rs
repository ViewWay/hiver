//! Smoke test for #[pre_authorize] macro — verifies it generates enforcement code.
//! #[pre_authorize] 宏的冒烟测试 —— 验证它生成强制执行代码。

use hiver_macros::pre_authorize;

/// A handler that requires ADMIN role. With an empty SecurityContext (no auth),
/// this should panic with "access denied".
/// 需要 ADMIN 角色的处理程序。在空 SecurityContext（无认证）下应 panic
/// 并显示 "access denied"。
#[pre_authorize("hasRole('ADMIN')")]
async fn admin_only() -> &'static str
{
    "you are admin"
}

fn main()
{
    let mut rt = hiver_runtime::Runtime::new().unwrap();
    // This should panic because the SecurityContext has no authentication,
    // so hasRole('ADMIN') evaluates to false.
    // 这应该 panic,因为 SecurityContext 无认证,故 hasRole('ADMIN') 为 false。
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rt.block_on(async { admin_only().await })
    }));
    assert!(result.is_err(), "should panic: access denied");
    eprintln!("PASS: #[pre_authorize] correctly denied access for unauthenticated user");
}
