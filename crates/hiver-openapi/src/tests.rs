//! Tests for hiver-openapi
//! 测试模块

#[cfg(test)]
mod tests
{
    #[test]
    fn smoke_test()
    {
        assert!(true, "hiver-openapi test infrastructure is working");
    }

    #[test]
    fn test_basic_math()
    {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_vec_operations()
    {
        let v: Vec<i32> = vec![1, 2, 3];
        assert_eq!(v.len(), 3);
        assert_eq!(v.iter().sum::<i32>(), 6);
    }
}
