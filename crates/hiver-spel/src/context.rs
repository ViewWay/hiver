//! SpEL evaluation context holding variables, roles, authorities, and authentication state.
//! SpEL 求值上下文，持有变量、角色、权限和认证状态。

use std::collections::{HashMap, HashSet};

use serde_json::Value;

/// Evaluation context for SpEL expressions.
/// SpEL 表达式求值上下文。
///
/// Supports variables, roles, authorities, and authentication state
/// for Spring Security-style expression evaluation.
///
/// 支持变量、角色、权限和认证状态，
/// 用于 Spring Security 风格的表达式求值。
pub struct SpelContext
{
    variables: HashMap<String, Value>,
    roles: HashSet<String>,
    authorities: HashSet<String>,
    authenticated: bool,
    principal: Option<String>,
}

impl SpelContext
{
    /// Creates a new empty evaluation context.
    /// 创建空的求值上下文。
    pub fn new() -> Self
    {
        Self {
            variables: HashMap::new(),
            roles: HashSet::new(),
            authorities: HashSet::new(),
            authenticated: false,
            principal: None,
        }
    }

    /// Sets a variable in the context.
    /// 在上下文中设置变量。
    pub fn set_variable(&mut self, name: &str, value: Value)
    {
        self.variables.insert(name.to_string(), value);
    }

    /// Adds a role to the context.
    /// 在上下文中添加角色。
    pub fn add_role(&mut self, role: &str)
    {
        self.roles.insert(role.to_string());
    }

    /// Adds multiple roles to the context.
    /// 在上下文中添加多个角色。
    pub fn add_roles(&mut self, roles: impl IntoIterator<Item = impl AsRef<str>>)
    {
        for role in roles
        {
            self.roles.insert(role.as_ref().to_string());
        }
    }

    /// Adds an authority to the context.
    /// 在上下文中添加权限。
    pub fn add_authority(&mut self, authority: &str)
    {
        self.authorities.insert(authority.to_string());
    }

    /// Adds multiple authorities to the context.
    /// 在上下文中添加多个权限。
    pub fn add_authorities(&mut self, authorities: impl IntoIterator<Item = impl AsRef<str>>)
    {
        for auth in authorities
        {
            self.authorities.insert(auth.as_ref().to_string());
        }
    }

    /// Sets the authentication state.
    /// 设置认证状态。
    pub fn set_authenticated(&mut self, authenticated: bool)
    {
        self.authenticated = authenticated;
    }

    /// Sets the principal (current user) name.
    /// 设置主体（当前用户）名称。
    pub fn set_principal(&mut self, principal: impl Into<String>)
    {
        let name = principal.into();
        self.principal = Some(name.clone());
        self.authenticated = true;
        self.variables
            .insert("principal".to_string(), Value::String(name));
    }

    pub(crate) fn get_variable(&self, name: &str) -> Option<&Value>
    {
        self.variables.get(name)
    }

    pub(crate) fn has_role(&self, role: &str) -> bool
    {
        self.roles.contains(role)
    }

    pub(crate) fn has_authority(&self, authority: &str) -> bool
    {
        self.authorities.contains(authority)
    }

    pub(crate) fn has_any_role(&self, roles: &[String]) -> bool
    {
        roles.iter().any(|r| self.roles.contains(r.as_str()))
    }

    pub(crate) fn is_authenticated(&self) -> bool
    {
        self.authenticated
    }

    pub(crate) fn is_anonymous(&self) -> bool
    {
        !self.authenticated
    }

    pub(crate) fn principal(&self) -> Option<&str>
    {
        self.principal.as_deref()
    }
}

impl Default for SpelContext
{
    fn default() -> Self
    {
        Self::new()
    }
}
