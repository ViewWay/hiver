//! SpEL evaluation context holding variables, roles, and authorities.
//! SpEL 求值上下文，持有变量、角色和权限。

use std::collections::{HashMap, HashSet};

use serde_json::Value;

/// Evaluation context for SpEL expressions.
/// SpEL 表达式求值上下文。
pub struct SpelContext {
    variables: HashMap<String, Value>,
    roles: HashSet<String>,
    authorities: HashSet<String>,
}

impl SpelContext {
    /// Creates a new empty evaluation context.
    /// 创建空的求值上下文。
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            roles: HashSet::new(),
            authorities: HashSet::new(),
        }
    }

    /// Sets a variable in the context.
    /// 在上下文中设置变量。
    pub fn set_variable(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    /// Adds a role to the context.
    /// 在上下文中添加角色。
    pub fn add_role(&mut self, role: &str) {
        self.roles.insert(role.to_string());
    }

    /// Adds an authority to the context.
    /// 在上下文中添加权限。
    pub fn add_authority(&mut self, authority: &str) {
        self.authorities.insert(authority.to_string());
    }

    pub(crate) fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub(crate) fn has_role(&self, role: &str) -> bool {
        self.roles.contains(role)
    }

    pub(crate) fn has_authority(&self, authority: &str) -> bool {
        self.authorities.contains(authority)
    }

    pub(crate) fn has_any_role(&self, roles: &[String]) -> bool {
        roles.iter().any(|r| self.roles.contains(r.as_str()))
    }
}

impl Default for SpelContext {
    fn default() -> Self {
        Self::new()
    }
}
