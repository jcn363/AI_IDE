//! Testing macros for common assertions and utilities

#[macro_export]
macro_rules! assert_test_file_exists {
    ($workspace:expr, $path:expr) => {
        assert!(
            $workspace.file_exists($path),
            "Expected file {:?} to exist in workspace",
            $path
        );
    };
}

#[macro_export]
macro_rules! assert_file_contains {
    ($workspace:expr, $path:expr, $content:expr) => {
        let actual_content = $workspace
            .read_file($path)
            .expect(concat!("Failed to read file: ", stringify!($path)));
        assert!(
            actual_content.contains($content),
            "Expected file {:?} to contain '{}', but got: {}",
            $path,
            $content,
            actual_content
        );
    };
}

#[macro_export]
macro_rules! assert_test_eq {
    ($left:expr, $right:expr) => {
        assert_eq!($left, $right, "Test assertion failed at {}:{}", file!(), line!());
    };
    ($left:expr, $right:expr, $msg:expr) => {
        assert_eq!($left, $right, "{} (at {}:{})", $msg, file!(), line!());
    };
}

#[macro_export]
macro_rules! assert_test_result {
    ($result:expr) => {
        $result.expect_test(concat!("Test failed at ", file!(), ":", line!()));
    };
    ($result:expr, $msg:expr) => {
        $result.expect(concat!("Test failed: ", $msg));
    };
}

#[macro_export]
macro_rules! setup_test_workspace {
    () => {
        $crate::filesystem::TempWorkspace::new().expect_test("Failed to create test workspace")
    };
    ($scenario:expr) => {{
        let workspace = $crate::filesystem::TempWorkspace::new().expect_test("Failed to create test workspace");
        workspace.setup_scenario($scenario).expect_test("Failed to setup test scenario");
        workspace
    }};
}

#[macro_export]
macro_rules! with_test_fixture {
    ($fixture:expr) => {{
        let workspace = setup_test_workspace!();
        let fixture = $fixture.build(&workspace).expect_test("Failed to build fixture");
        (workspace, fixture)
    }};
}

#[macro_export]
macro_rules! assert_timeout {
    ($future:expr, $duration:expr) => {
        $crate::async_utils::with_timeout($future, $duration)
            .await
            .expect_test(concat!("Async operation timed out at ", file!(), ":", line!()));
    };
}

#[macro_export]
macro_rules! assert_concurrent_completion {
    ($tasks:expr) => {
        $crate::async_utils::run_concurrent($tasks)
            .await
            .expect_test("Concurrent tasks failed");
    };
    ($tasks:expr, $timeout:expr) => {
        $crate::async_utils::wait_all_timeout($tasks, $timeout)
            .await
            .expect_test("Concurrent tasks failed or timed out");
    };
}

/// Attribute macro for test functions that require clean setup
#[macro_export]
macro_rules! test_fn {
    ($name:ident, $body:block) => {
        #[test]
        fn $name() {
            // Initialize test logging if needed
            let _guard = if std::env::var("TEST_LOG").is_ok() {
                env_logger::init()
            } else {
                let _ = env_logger::builder().is_test(true).try_init();
            };

            $body
        }
    };
}

/// Macro for creating property-based tests (requires proptest)
#[macro_export]
macro_rules! proptest_test {
    ($name:ident, $($strategy:tt => $body:expr),*) => {
        #[cfg(test)]
        proptest::proptest! {
            #[$name]($($strategy),*) {
                $body
            }
        }
    };
}

/// Macro for cleanup in tests
#[macro_export]
macro_rules! with_cleanup {
    ($setup:expr, $test:expr) => {{
        let setup_result = $setup;
        let test_result = $test(setup_result);
        // Add cleanup logic here if needed
        test_result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filesystem::TempWorkspace;

    #[test]
    fn test_assert_test_file_exists() {
        let workspace = TempWorkspace::new().expect("Failed to create workspace");
        workspace.create_file(std::path::Path::new("test.txt"), "test").expect("Failed to create file");

        assert_test_file_exists!(workspace, std::path::Path::new("test.txt"));
    }

    #[test]
    fn test_assert_file_contains() {
        let workspace = TempWorkspace::new().expect("Failed to create workspace");
        workspace.create_file(std::path::Path::new("test.txt"), "Hello World").expect("Failed to create file");

        assert_file_contains!(workspace, std::path::Path::new("test.txt"), "Hello");
    }
}