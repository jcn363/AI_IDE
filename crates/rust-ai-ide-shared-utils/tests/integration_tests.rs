use std::path::Path;

use rust_ai_ide_shared_utils::{get_extension, is_code_file, normalize_path};

#[tokio::test]
async fn test_file_operations_integration() {
    // Test basic file operations work together
    let test_path = Path::new("test.rs");

    // Extension detection
    assert_eq!(get_extension(test_path), Some("rs"));

    // Code file detection (this would fail if the file doesn't exist, but tests the logic)
    // Note: We can't create actual files in unit tests easily, so we test the logic

    // Path normalization
    let relative_path = Path::new("./src/main.rs");
    let normalized = normalize_path(relative_path);
    assert!(normalized.ends_with("main.rs"));
}

#[tokio::test]
async fn test_caching_integration() {
    // Test that caching layer works with filesystem operations
    use rust_ai_ide_shared_utils::get_file_size_cached;

    let test_path = Path::new("src/lib.rs");

    if test_path.exists() {
        // First call should compute and cache
        let size1 = get_file_size_cached(test_path).await;
        assert!(size1.is_some());

        // Second call should use cache
        let size2 = get_file_size_cached(test_path).await;
        assert_eq!(size1, size2);
    }
}

#[test]
fn test_multiple_language_support() {
    // Test that our file type detection supports multiple programming languages
    let languages = vec![
        ("main.rs", "rs"),
        ("script.py", "py"),
        ("app.js", "js"),
        ("component.tsx", "tsx"),
        ("program.go", "go"),
        ("Main.java", "java"),
        ("Program.cs", "cs"),
        ("module.rb", "rb"),
        ("lib.php", "php"),
        ("config.toml", "toml"),
        ("readme.md", "md"),
    ];

    for (filename, expected_ext) in languages {
        let path = Path::new(filename);
        assert_eq!(get_extension(path), Some(expected_ext), "Failed for {}", filename);
    }
}

#[test]
fn test_code_file_detection() {
    // Test comprehensive code file detection
    let code_files = vec![
        "main.rs", "lib.rs", "mod.rs",
        "script.py", "app.py",
        "index.js", "server.js",
        "component.tsx", "App.tsx",
        "program.go", "main.go",
        "Main.java", "App.java",
        "Program.cs", "Library.cs",
        "config.json", "package.json",
    ];

    let non_code_files = vec![
        "readme.txt", "readme.md", // md is documentation, not code in this context
        "image.png", "photo.jpg",
        "document.pdf", "archive.zip",
        "music.mp3", "video.mp4",
    ];

    for filename in code_files {
        let path = Path::new(filename);
        assert!(is_code_file(path), "{} should be detected as code file", filename);
    }

    for filename in non_code_files {
        let path = Path::new(filename);
        assert!(!is_code_file(path), "{} should NOT be detected as code file", filename);
    }
}