//! Access Control List (ACL) for domain-object-level permissions.
//! 访问控制列表（ACL），用于领域对象级别的权限控制。
//!
//! Equivalent to Spring Security's ACL module.
//! 等价于 Spring Security 的 ACL 模块。

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Bit-flag permissions for ACL entries.
/// ACL 条目的位标志权限。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AclPermission(u32);

impl AclPermission {
    /// Read permission.
    pub const READ: AclPermission = AclPermission(1);
    /// Write permission.
    pub const WRITE: AclPermission = AclPermission(1 << 1);
    /// Create permission.
    pub const CREATE: AclPermission = AclPermission(1 << 2);
    /// Delete permission.
    pub const DELETE: AclPermission = AclPermission(1 << 3);
    /// Administration permission (implies all others).
    pub const ADMIN: AclPermission = AclPermission(1 << 4);

    /// Create a permission from raw bits.
    pub fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// Check if this permission includes the given permission.
    pub fn includes(&self, other: AclPermission) -> bool {
        (self.0 & other.0) != 0
    }

    /// Combine two permissions.
    pub fn union(self, other: AclPermission) -> Self {
        Self(self.0 | other.0)
    }

    /// Raw bits value.
    pub fn bits(&self) -> u32 {
        self.0
    }
}

/// Security Identity — represents a principal or authority.
/// 安全身份 —— 表示主体或授权。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AclSid {
    /// A specific user principal.
    Principal(String),
    /// A granted authority (role/permission).
    Authority(String),
}

/// Identifies a domain object instance.
/// 标识领域对象实例。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AclObjectIdentity {
    /// Type of the domain object (e.g. "Document", "Order").
    pub object_type: String,
    /// Identifier of the specific instance.
    pub object_id: String,
}

impl AclObjectIdentity {
    /// Create a new object identity.
    pub fn new(object_type: impl Into<String>, object_id: impl Into<String>) -> Self {
        Self {
            object_type: object_type.into(),
            object_id: object_id.into(),
        }
    }
}

/// A single access control entry: grants or denies a permission to a SID.
/// 单个访问控制条目：授予或拒绝 SID 的权限。
#[derive(Debug, Clone)]
pub struct AclEntry {
    /// The SID this entry applies to.
    pub sid: AclSid,
    /// The permission mask.
    pub permission: AclPermission,
    /// `true` = grant, `false` = deny.
    pub granting: bool,
}

impl AclEntry {
    /// Create a granting entry.
    pub fn grant(sid: AclSid, permission: AclPermission) -> Self {
        Self { sid, permission, granting: true }
    }

    /// Create a denying entry.
    pub fn deny(sid: AclSid, permission: AclPermission) -> Self {
        Self { sid, permission, granting: false }
    }

    /// Check if this entry matches the given SID and permission.
    pub fn matches(&self, sid: &AclSid, permission: AclPermission) -> bool {
        &self.sid == sid && self.permission.includes(permission)
    }
}

/// Access Control List for a single domain object.
/// 单个领域对象的访问控制列表。
#[derive(Debug, Clone)]
pub struct Acl {
    /// The object this ACL protects.
    pub object_identity: AclObjectIdentity,
    /// The owner of this ACL.
    pub owner: AclSid,
    /// ACL entries (order matters: first match wins).
    pub entries: Vec<AclEntry>,
    /// Whether this ACL inherits entries from a parent.
    pub inherit: bool,
    /// Optional parent ACL object identity.
    pub parent: Option<AclObjectIdentity>,
}

impl Acl {
    /// Create a new ACL for an object owned by the given SID.
    pub fn new(object_identity: AclObjectIdentity, owner: AclSid) -> Self {
        Self {
            object_identity,
            owner,
            entries: Vec::new(),
            inherit: false,
            parent: None,
        }
    }

    /// Add an entry to this ACL.
    pub fn add_entry(&mut self, entry: AclEntry) {
        self.entries.push(entry);
    }

    /// Remove entries matching the given SID.
    pub fn remove_entries_for_sid(&mut self, sid: &AclSid) {
        self.entries.retain(|e| &e.sid != sid);
    }

    /// Check if the given SID is granted the requested permission.
    pub fn is_granted(&self, sid: &AclSid, permission: AclPermission) -> bool {
        if &self.owner == sid {
            return true;
        }
        let mut granted = false;
        for entry in &self.entries {
            if entry.matches(sid, permission) {
                if !entry.granting {
                    return false;
                }
                granted = true;
            }
        }
        granted
    }
}

/// In-memory ACL service for managing access control lists.
/// 内存型 ACL 服务，用于管理访问控制列表。
pub struct AclService {
    acls: Arc<RwLock<HashMap<(String, String), Acl>>>,
}

impl Default for AclService {
    fn default() -> Self {
        Self::new()
    }
}

impl AclService {
    /// Create a new empty ACL service.
    pub fn new() -> Self {
        Self {
            acls: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create or replace the ACL for an object.
    pub fn save_acl(&self, acl: Acl) {
        let key = (acl.object_identity.object_type.clone(), acl.object_identity.object_id.clone());
        self.acls.write().unwrap().insert(key, acl);
    }

    /// Retrieve the ACL for an object, if it exists.
    pub fn get_acl(&self, oid: &AclObjectIdentity) -> Option<Acl> {
        self.acls.read().unwrap()
            .get(&(oid.object_type.clone(), oid.object_id.clone()))
            .cloned()
    }

    /// Remove the ACL for an object.
    pub fn delete_acl(&self, oid: &AclObjectIdentity) -> bool {
        self.acls.write().unwrap()
            .remove(&(oid.object_type.clone(), oid.object_id.clone()))
            .is_some()
    }

    /// Check if a SID is granted the requested permission on an object.
    pub fn is_granted(&self, oid: &AclObjectIdentity, sid: &AclSid, permission: AclPermission) -> bool {
        self.acls.read().unwrap()
            .get(&(oid.object_type.clone(), oid.object_id.clone()))
            .map_or(false, |acl| acl.is_granted(sid, permission))
    }

    /// List all ACLs.
    pub fn list_acls(&self) -> Vec<Acl> {
        self.acls.read().unwrap().values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_flags() {
        let p = AclPermission::READ.union(AclPermission::WRITE);
        assert!(p.includes(AclPermission::READ));
        assert!(p.includes(AclPermission::WRITE));
        assert!(!p.includes(AclPermission::DELETE));
    }

    #[test]
    fn test_acl_owner_always_granted() {
        let oid = AclObjectIdentity::new("Document", "1");
        let owner = AclSid::Principal("alice".into());
        let acl = Acl::new(oid, owner.clone());
        assert!(acl.is_granted(&owner, AclPermission::DELETE));
    }

    #[test]
    fn test_acl_grant_entry() {
        let oid = AclObjectIdentity::new("Document", "1");
        let owner = AclSid::Principal("alice".into());
        let mut acl = Acl::new(oid, owner);
        let bob = AclSid::Principal("bob".into());
        acl.add_entry(AclEntry::grant(bob.clone(), AclPermission::READ));
        assert!(acl.is_granted(&bob, AclPermission::READ));
        assert!(!acl.is_granted(&bob, AclPermission::WRITE));
    }

    #[test]
    fn test_acl_deny_entry() {
        let oid = AclObjectIdentity::new("Document", "1");
        let owner = AclSid::Principal("alice".into());
        let mut acl = Acl::new(oid, owner);
        let bob = AclSid::Principal("bob".into());
        acl.add_entry(AclEntry::grant(bob.clone(), AclPermission::READ.union(AclPermission::WRITE)));
        acl.add_entry(AclEntry::deny(bob.clone(), AclPermission::WRITE));
        assert!(acl.is_granted(&bob, AclPermission::READ));
        assert!(!acl.is_granted(&bob, AclPermission::WRITE));
    }

    #[test]
    fn test_acl_service_crud() {
        let service = AclService::new();
        let oid = AclObjectIdentity::new("Order", "42");
        let owner = AclSid::Principal("alice".into());
        service.save_acl(Acl::new(oid.clone(), owner.clone()));
        assert!(service.get_acl(&oid).is_some());
        assert!(service.is_granted(&oid, &owner, AclPermission::ADMIN));
        assert!(service.delete_acl(&oid));
        assert!(service.get_acl(&oid).is_none());
    }

    #[test]
    fn test_acl_service_authority_sid() {
        let service = AclService::new();
        let oid = AclObjectIdentity::new("Report", "r1");
        let owner = AclSid::Principal("admin".into());
        let mut acl = Acl::new(oid.clone(), owner);
        let editors = AclSid::Authority("ROLE_EDITOR".into());
        acl.add_entry(AclEntry::grant(editors.clone(), AclPermission::READ.union(AclPermission::WRITE)));
        service.save_acl(acl);
        assert!(service.is_granted(&oid, &editors, AclPermission::READ));
        assert!(!service.is_granted(&oid, &editors, AclPermission::DELETE));
    }
}
