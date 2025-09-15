#[cfg(test)]
mod tests {
    use rust_ai_ide_derive_utils::*;

    #[derive(DefaultFromNew)]
    struct TestStruct {
        field1: String,
        field2: i32,
    }

    impl TestStruct {
        fn new() -> Self {
            Self {
                field1: "test".to_string(),
                field2: 42,
            }
        }
    }

    #[derive(DeriveClone)]
    struct TestCloneStruct {
        field1: String,
        field2: i32,
    }

    #[test]
    fn test_default_from_new() {
        let instance = TestStruct::default();
        assert_eq!(instance.field1, "test");
        assert_eq!(instance.field2, 42);
    }

    #[test]
    fn test_clone_derive() {
        let original = TestCloneStruct {
            field1: "hello".to_string(),
            field2: 123,
        };
        let cloned = original.clone();

        assert_eq!(original.field1, cloned.field1);
        assert_eq!(original.field2, cloned.field2);
        assert_ne!(original.field1.as_ptr(), cloned.field1.as_ptr()); // Ensure deep clone
    }

    #[derive(Clone)]
    struct TupleStruct(String, i32);

    #[test]
    fn test_clone_tuple_struct() {
        let original = TupleStruct("world".to_string(), 456);
        let cloned = original.clone();

        assert_eq!(original.0, cloned.0);
        assert_eq!(original.1, cloned.1);
        assert_ne!(original.0.as_ptr(), cloned.0.as_ptr()); // Ensure deep clone
    }

    #[derive(Clone)]
    struct UnitStruct;

    #[test]
    fn test_clone_unit_struct() {
        let original = UnitStruct;
        let cloned = original.clone();

        // Unit structs are always equal and don't have fields to compare
        assert!(matches!(cloned, UnitStruct));
    }

    #[test]
    fn test_derive_clone_macro_compilation() {
        // Test that the derive macro compiles correctly by using it on a simple struct
        // This verifies the macro expansion works without runtime errors
        #[derive(DeriveClone)]
        struct TestDeriveCloneStruct {
            field1: String,
            field2: i32,
        }

        let original = TestDeriveCloneStruct {
            field1: "macro".to_string(),
            field2: 789,
        };

        // Test that clone works (this verifies the macro generated the correct code)
        let cloned = original.clone();
        assert_eq!(original.field1, cloned.field1);
        assert_eq!(original.field2, cloned.field2);
        assert_ne!(original.field1.as_ptr(), cloned.field1.as_ptr()); // Ensure deep clone
    }
}
