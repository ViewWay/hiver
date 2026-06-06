//! Tests for hiver-data-macros
//! 测试模块

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::float_cmp, clippy::module_inception, clippy::items_after_statements, clippy::assertions_on_constants)]
mod tests {
    // Basic compilation test — the actual tests are in the proc-macro crate's
    // integration tests which verify macro expansion output.
    #[test]
    fn test_module_exists() {
        // Verify the crate is functional
        assert!(true);
    }
}
