//! Integration-level tests for hiver-ldap
//! hiver-ldap 的集成级测试
//!
//! These tests exercise the public API contract without requiring a live LDAP server.
//! When the `ldap` feature is disabled, the template returns safe stub values.
//!
//! 这些测试在不要求实时LDAP服务器的情况下验证公共API契约。
//! 当 `ldap` feature 禁用时，模板返回安全的存根值。

#[cfg(test)]
#[allow(
    clippy::indexing_slicing,
    clippy::float_cmp,
    clippy::module_inception,
    clippy::items_after_statements,
    clippy::assertions_on_constants
)]
mod tests {
    use crate::{
        context::LdapContextSourceBuilder,
        mapper::AttrMap,
        odm::{AttributeMapping, Dn, ObjectDirectoryMapper, build_dn, parse_rdn_value},
        pool::{LdapPool, PoolConfig},
        query::LdapQueryBuilder,
        repository::{EntryMapper, EntrySerializer, IdExtractor, TypedLdapRepository},
        *,
    };

    // ============================================================
    // Domain types for testing / 用于测试的领域类型
    // ============================================================

    #[derive(Debug, Clone)]
    struct Person {
        uid: String,
        cn: String,
        mail: String,
    }

    impl OdmEntry for Person {
        fn base_dn() -> &'static str {
            "ou=people,dc=example,dc=com"
        }

        fn rdn_attribute() -> &'static str {
            "uid"
        }

        fn object_classes() -> &'static [&'static str] {
            &["top", "person", "inetOrgPerson"]
        }

        fn attribute_names() -> &'static [&'static str] {
            &["uid", "cn", "mail"]
        }
    }

    #[derive(Debug, Clone)]
    struct Group {
        cn: String,
        description: String,
        member_count: usize,
    }

    impl OdmEntry for Group {
        fn base_dn() -> &'static str {
            "ou=groups,dc=example,dc=com"
        }

        fn rdn_attribute() -> &'static str {
            "cn"
        }

        fn object_classes() -> &'static [&'static str] {
            &["top", "groupOfNames"]
        }
    }

    struct PersonMapper;
    impl EntryMapper<Person> for PersonMapper {
        fn map_entry(&self, attrs: &AttrMap) -> Person {
            Person {
                uid: attrs.get_first("uid").unwrap_or_default().to_string(),
                cn: attrs.get_first("cn").unwrap_or_default().to_string(),
                mail: attrs.get_first("mail").unwrap_or_default().to_string(),
            }
        }
    }

    struct PersonSerializer;
    impl EntrySerializer<Person> for PersonSerializer {
        fn serialize(&self, p: &Person) -> Vec<(String, Vec<String>)> {
            vec![
                ("uid".into(), vec![p.uid.clone()]),
                ("cn".into(), vec![p.cn.clone()]),
                ("mail".into(), vec![p.mail.clone()]),
            ]
        }

        fn rdn_value(&self, p: &Person) -> String {
            p.uid.clone()
        }
    }

    struct PersonIdExtractor;
    impl IdExtractor<Person, String> for PersonIdExtractor {
        fn extract_id(&self, p: &Person) -> String {
            p.uid.clone()
        }
    }

    struct GroupMapper;
    impl EntryMapper<Group> for GroupMapper {
        fn map_entry(&self, attrs: &AttrMap) -> Group {
            Group {
                cn: attrs.get_first("cn").unwrap_or_default().to_string(),
                description: attrs
                    .get_first("description")
                    .unwrap_or_default()
                    .to_string(),
                member_count: 0,
            }
        }
    }

    struct GroupSerializer;
    impl EntrySerializer<Group> for GroupSerializer {
        fn serialize(&self, g: &Group) -> Vec<(String, Vec<String>)> {
            vec![
                ("cn".into(), vec![g.cn.clone()]),
                ("description".into(), vec![g.description.clone()]),
            ]
        }

        fn rdn_value(&self, g: &Group) -> String {
            g.cn.clone()
        }
    }

    struct GroupIdExtractor;
    impl IdExtractor<Group, String> for GroupIdExtractor {
        fn extract_id(&self, g: &Group) -> String {
            g.cn.clone()
        }
    }

    // Helper to create a template / 创建模板的辅助函数
    fn test_template() -> LdapTemplate {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        LdapTemplate::new(ctx)
    }

    fn person_repo()
    -> TypedLdapRepository<Person, String, PersonMapper, PersonSerializer, PersonIdExtractor> {
        TypedLdapRepository::new(
            test_template(),
            "ou=people,dc=example,dc=com",
            PersonMapper,
            PersonSerializer,
            PersonIdExtractor,
        )
    }

    fn group_repo()
    -> TypedLdapRepository<Group, String, GroupMapper, GroupSerializer, GroupIdExtractor> {
        TypedLdapRepository::new(
            test_template(),
            "ou=groups,dc=example,dc=com",
            GroupMapper,
            GroupSerializer,
            GroupIdExtractor,
        )
    }

    // ============================================================
    // Context source tests / 上下文源测试
    // ============================================================

    #[test]
    fn test_context_source_builder_full() {
        let ctx = LdapContextSourceBuilder::default()
            .url("ldap://localhost:389")
            .base_dn("dc=example,dc=com")
            .username("cn=admin,dc=example,dc=com")
            .password("secret")
            .build()
            .unwrap();
        assert_eq!(ctx.url(), "ldap://localhost:389");
        assert_eq!(ctx.base_dn(), "dc=example,dc=com");
    }

    #[test]
    fn test_context_source_builder_missing_url() {
        let result = LdapContextSourceBuilder::default()
            .base_dn("dc=example,dc=com")
            .build();
        assert!(result.is_err());
        match result.unwrap_err() {
            LdapError::Connection(msg) => assert!(msg.contains("URL")),
            other => panic!("Expected Connection error, got: {:?}", other),
        }
    }

    #[test]
    fn test_context_source_builder_missing_base_dn() {
        let result = LdapContextSourceBuilder::default()
            .url("ldap://localhost:389")
            .build();
        assert!(result.is_err());
        match result.unwrap_err() {
            LdapError::Connection(msg) => assert!(msg.contains("Base DN")),
            other => panic!("Expected Connection error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_context_source_get_context() {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let conn = ctx.get_context().await.unwrap();
        assert!(conn.is_connected());
    }

    #[tokio::test]
    async fn test_context_source_anonymous_context() {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let conn = ctx.get_anonymous_context().await.unwrap();
        assert!(conn.is_connected());
    }

    #[tokio::test]
    async fn test_connection_unbind() {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let mut conn = ctx.get_context().await.unwrap();
        assert!(conn.is_connected());
        conn.unbind().await.unwrap();
        assert!(!conn.is_connected());
    }

    #[tokio::test]
    async fn test_connection_simple_bind() {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let mut conn = ctx.get_context().await.unwrap();
        let result = conn.simple_bind("cn=admin", "password").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_with_credentials() {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com")
            .with_credentials("cn=admin", "secret");
        let conn = ctx.get_context().await.unwrap();
        assert!(conn.is_connected());
    }

    // ============================================================
    // Template tests / 模板测试
    // ============================================================

    #[test]
    fn test_template_exposes_context_source() {
        let template = test_template();
        assert_eq!(template.context_source().url(), "ldap://localhost:389");
        assert_eq!(template.context_source().base_dn(), "dc=example,dc=com");
    }

    #[tokio::test]
    async fn test_template_authenticate() {
        let template = test_template();
        let ok = template.authenticate("cn=admin", "secret").await.unwrap();
        assert!(ok);
    }

    #[tokio::test]
    async fn test_template_search_empty() {
        let template = test_template();
        let results = template
            .search_attrs("dc=example,dc=com", "(objectClass=*)")
            .await
            .unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_template_lookup_none() {
        let template = test_template();
        struct CtxMapper;
        impl ContextMapper<String> for CtxMapper {
            fn map_from_context(&self, ctx: &str) -> String {
                ctx.to_string()
            }
        }
        let result = template
            .lookup("cn=missing,dc=example,dc=com", &CtxMapper)
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_template_bind_ok() {
        let template = test_template();
        let result = template
            .bind("cn=new,dc=example,dc=com", &[("objectClass", &["person"] as &[&str])])
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_template_unbind_ok() {
        let template = test_template();
        let result = template.unbind("cn=user,dc=example,dc=com").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_template_modify_ok() {
        let template = test_template();
        let result = template
            .modify("cn=user,dc=example,dc=com", &[("sn", &["newValue"] as &[&str])])
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_template_exists_false() {
        let template = test_template();
        let exists = template
            .exists("cn=missing,dc=example,dc=com")
            .await
            .unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_template_count_zero() {
        let template = test_template();
        let count = template
            .count("dc=example,dc=com", "(objectClass=*)")
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    // ============================================================
    // Repository tests / 仓库测试
    // ============================================================

    #[test]
    fn test_person_repo_base() {
        let repo = person_repo();
        assert_eq!(repo.base(), "ou=people,dc=example,dc=com");
    }

    #[test]
    fn test_person_repo_build_entry_dn() {
        let repo = person_repo();
        let person = Person {
            uid: "alice".into(),
            cn: "Alice".into(),
            mail: "alice@example.com".into(),
        };
        assert_eq!(repo.build_entry_dn(&person), "uid=alice,ou=people,dc=example,dc=com");
    }

    #[test]
    fn test_person_repo_build_id_dn() {
        let repo = person_repo();
        assert_eq!(repo.build_id_dn(&"bob".to_string()), "uid=bob,ou=people,dc=example,dc=com");
    }

    #[tokio::test]
    async fn test_person_repo_find_all_empty() {
        let repo = person_repo();
        let results = repo.find_all().await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_person_repo_find_by_id_none() {
        let repo = person_repo();
        let result = repo.find_by_id(&"nobody".to_string()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_person_repo_exists_false() {
        let repo = person_repo();
        let exists = repo.exists_by_id(&"ghost".to_string()).await.unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_person_repo_count_zero() {
        let repo = person_repo();
        let count = repo.count().await.unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_group_repo_build_entry_dn() {
        let repo = group_repo();
        let group = Group {
            cn: "admins".into(),
            description: "Admins".into(),
            member_count: 5,
        };
        assert_eq!(repo.build_entry_dn(&group), "cn=admins,ou=groups,dc=example,dc=com");
    }

    // ============================================================
    // ODM tests / ODM 测试
    // ============================================================

    #[test]
    fn test_dn_equality() {
        let a = Dn::new("cn=user,dc=example,dc=com");
        let b = Dn::new("cn=user,dc=example,dc=com");
        assert_eq!(a, b);
    }

    #[test]
    fn test_dn_components() {
        let dn = Dn::new("cn=user,ou=people,dc=example,dc=com");
        assert_eq!(dn.components(), vec!["cn=user", "ou=people", "dc=example", "dc=com"]);
    }

    #[test]
    fn test_dn_rdn() {
        let dn = Dn::new("uid=john,ou=people,dc=example,dc=com");
        assert_eq!(dn.rdn(), Some("uid=john"));
    }

    #[test]
    fn test_dn_parent() {
        let dn = Dn::new("uid=john,ou=people,dc=example,dc=com");
        let parent = dn.parent().unwrap();
        assert_eq!(parent.as_str(), "ou=people,dc=example,dc=com");
    }

    #[test]
    fn test_dn_parent_of_root_is_none() {
        let dn = Dn::new("dc=com");
        assert!(dn.parent().is_none());
    }

    #[test]
    fn test_dn_is_descendant() {
        let dn = Dn::new("cn=user,ou=people,dc=example,dc=com");
        let base = Dn::new("dc=example,dc=com");
        assert!(dn.is_descendant_of(&base));
        assert!(!base.is_descendant_of(&dn));
    }

    #[test]
    fn test_dn_case_insensitive_descendant() {
        let dn = Dn::new("cn=user,DC=EXAMPLE,DC=COM");
        let base = Dn::new("dc=example,dc=com");
        assert!(dn.is_descendant_of(&base));
    }

    #[test]
    fn test_dn_display() {
        let dn = Dn::new("cn=user,dc=example,dc=com");
        assert_eq!(format!("{}", dn), "cn=user,dc=example,dc=com");
    }

    #[test]
    fn test_build_dn_helper() {
        assert_eq!(
            build_dn("uid", "john", "ou=people,dc=example,dc=com"),
            "uid=john,ou=people,dc=example,dc=com"
        );
    }

    #[test]
    fn test_parse_rdn_value_found() {
        assert_eq!(
            parse_rdn_value("uid=john,ou=people,dc=example,dc=com", "uid"),
            Some("john".to_string())
        );
    }

    #[test]
    fn test_parse_rdn_value_not_found() {
        assert_eq!(parse_rdn_value("uid=john,ou=people,dc=example,dc=com", "cn"), None);
    }

    // ============================================================
    // ObjectDirectoryMapper tests / 对象目录映射器测试
    // ============================================================

    #[test]
    fn test_odm_mapper_full_roundtrip() {
        let mut odm = ObjectDirectoryMapper::new();
        odm.add_mapping(AttributeMapping::new("uid", "uid").id());
        odm.add_mapping(AttributeMapping::new("cn", "name"));
        odm.add_mapping(AttributeMapping::new("mail", "email"));

        let mut attrs = AttrMap::new();
        attrs.add("uid", &["jdoe"]);
        attrs.add("cn", &["John Doe"]);
        attrs.add("mail", &["jdoe@example.com"]);

        let mapped = odm.map_from_attrs(&attrs);
        assert_eq!(mapped.get("uid").unwrap(), "jdoe");
        assert_eq!(mapped.get("name").unwrap(), "John Doe");
        assert_eq!(mapped.get("email").unwrap(), "jdoe@example.com");
    }

    #[test]
    fn test_odm_mapper_skips_readonly_on_serialize() {
        let mut odm = ObjectDirectoryMapper::new();
        odm.add_mapping(AttributeMapping::new("cn", "name"));
        odm.add_mapping(AttributeMapping::new("createTimestamp", "created").readonly());

        let mut fields = std::collections::HashMap::new();
        fields.insert("name".to_string(), "John".to_string());
        fields.insert("created".to_string(), "2024-01-01".to_string());

        let result = odm.map_to_attrs(&fields);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "cn");
    }

    // ============================================================
    // AttrMap tests / 属性映射测试
    // ============================================================

    #[test]
    fn test_attr_map_add_and_get() {
        let mut map = AttrMap::new();
        map.add("cn", &["John", "Johnny"]);
        assert_eq!(map.get_first("cn"), Some("John"));
        let values = map.get("cn").unwrap();
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_attr_map_missing_key() {
        let map = AttrMap::new();
        assert!(map.get("cn").is_none());
        assert_eq!(map.get_first("cn"), None);
    }

    #[test]
    fn test_attr_map_default() {
        let map = AttrMap::default();
        assert!(map.get("anything").is_none());
    }

    // ============================================================
    // Pool tests / 连接池测试
    // ============================================================

    #[test]
    fn test_pool_default_config() {
        let config = PoolConfig::default();
        assert_eq!(config.max_size, 8);
        assert_eq!(config.max_idle, 4);
        assert_eq!(config.min_idle, 1);
    }

    #[test]
    fn test_pool_borrow_and_return() {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let pool = LdapPool::new(ctx, PoolConfig::default());
        let stats = pool.stats();
        assert_eq!(stats.idle, 1);
        assert_eq!(stats.active, 0);

        let conn = pool.borrow().unwrap();
        let stats = pool.stats();
        assert_eq!(stats.active, 1);

        pool.return_connection(conn);
        let stats = pool.stats();
        assert_eq!(stats.active, 0);
    }

    #[test]
    fn test_pool_exhaustion() {
        let ctx = LdapContextSource::new("ldap://localhost:389", "dc=example,dc=com");
        let config = PoolConfig {
            max_size: 2,
            ..PoolConfig::default()
        };
        let pool = LdapPool::new(ctx, config);

        let c1 = pool.borrow().unwrap();
        let c2 = pool.borrow().unwrap();
        let c3 = pool.borrow(); // should fail

        assert!(c3.is_err());
        pool.return_connection(c1);
        pool.return_connection(c2);
    }

    // ============================================================
    // Query builder tests / 查询构建器测试
    // ============================================================

    #[test]
    fn test_query_wildcard() {
        assert_eq!(LdapQueryBuilder::new().build(), "(objectClass=*)");
    }

    #[test]
    fn test_query_single_eq() {
        assert_eq!(
            LdapQueryBuilder::new().eq("objectClass", "person").build(),
            "(objectClass=person)"
        );
    }

    #[test]
    fn test_query_and() {
        let filter = LdapQueryBuilder::new()
            .where_attr("objectClass")
            .is("person")
            .and()
            .where_attr("sn")
            .is("Smith")
            .build();
        assert_eq!(filter, "(&(objectClass=person)(sn=Smith))");
    }

    #[test]
    fn test_query_or() {
        let filter = LdapQueryBuilder::or_query()
            .where_attr("cn")
            .is("John")
            .or()
            .where_attr("cn")
            .is("Jane")
            .build();
        assert_eq!(filter, "(|(cn=John)(cn=Jane))");
    }

    #[test]
    fn test_query_not_equal() {
        let filter = LdapQueryBuilder::new()
            .where_attr("status")
            .is_not("inactive")
            .build();
        assert_eq!(filter, "(!(status=inactive))");
    }

    #[test]
    fn test_query_present() {
        assert_eq!(LdapQueryBuilder::new().present("mail").build(), "(mail=*)");
    }

    #[test]
    fn test_query_like() {
        assert_eq!(
            LdapQueryBuilder::new()
                .where_attr("cn")
                .like("John")
                .build(),
            "(cn=*John*)"
        );
    }

    #[test]
    fn test_query_starts_with() {
        assert_eq!(
            LdapQueryBuilder::new()
                .where_attr("cn")
                .starts_with("J")
                .build(),
            "(cn=J*)"
        );
    }

    #[test]
    fn test_query_ends_with() {
        assert_eq!(
            LdapQueryBuilder::new()
                .where_attr("mail")
                .ends_with("@example.com")
                .build(),
            "(mail=*@example.com)"
        );
    }

    #[test]
    fn test_query_gte_lte() {
        let filter = LdapQueryBuilder::new()
            .gte("uidNumber", "1000")
            .lte("uidNumber", "2000")
            .build();
        assert_eq!(filter, "(&(uidNumber>=1000)(uidNumber<=2000))");
    }

    #[test]
    fn test_query_complex_nested() {
        let filter = LdapQueryBuilder::new()
            .eq("objectClass", "person")
            .raw("(|(cn=John)(cn=Jane))")
            .build();
        assert_eq!(filter, "(&(objectClass=person)(|(cn=John)(cn=Jane)))");
    }

    #[test]
    fn test_query_exists_not_exists() {
        let filter = LdapQueryBuilder::new()
            .where_attr("mail")
            .exists()
            .and()
            .where_attr("password")
            .not_exists()
            .build();
        assert_eq!(filter, "(&(mail=*)(!(password=*)))");
    }

    // ============================================================
    // Error type tests / 错误类型测试
    // ============================================================

    #[test]
    fn test_ldap_error_display() {
        let err = LdapError::Connection("refused".into());
        assert!(err.to_string().contains("refused"));

        let err = LdapError::Authentication("bad creds".into());
        assert!(err.to_string().contains("bad creds"));

        let err = LdapError::Operation("timeout".into());
        assert!(err.to_string().contains("timeout"));

        let err = LdapError::NotFound("cn=missing".into());
        assert!(err.to_string().contains("cn=missing"));

        let err = LdapError::SchemaViolation("violation".into());
        assert!(err.to_string().contains("violation"));
    }
}
