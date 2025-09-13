use rust_ai_ide_cargo::dependency::conflict::{self, ConflictVersion, DependentInfo, VersionConflict};

#[test]
fn test_conflict_resolution_logic() {
    // Create a simple test conflict
    let conflict = VersionConflict {
        package_name: "test_pkg".to_string(),
        versions:     vec![
            ConflictVersion {
                version:     "^1.0.0".to_string(),
                required_by: vec![DependentInfo {
                    name:    "test_dep1".to_string(),
                    version: "0.1.0".to_string(),
                    path:    "test1/Cargo.toml".to_string(),
                }],
            },
            ConflictVersion {
                version:     "^2.0.0".to_string(),
                required_by: vec![DependentInfo {
                    name:    "test_dep2".to_string(),
                    version: "0.2.0".to_string(),
                    path:    "test2/Cargo.toml".to_string(),
                }],
            },
        ],
        dependents:   vec![
            DependentInfo {
                name:    "test_dep1".to_string(),
                version: "0.1.0".to_string(),
                path:    "test1/Cargo.toml".to_string(),
            },
            DependentInfo {
                name:    "test_dep2".to_string(),
                version: "0.2.0".to_string(),
                path:    "test2/Cargo.toml".to_string(),
            },
        ],
    };

    // Test that we can suggest a resolution
    let suggested = conflict::suggest_resolution(&conflict);
    assert!(suggested.is_some(), "Should suggest a resolution");

    // Test that the suggested version is valid semver
    if let Some(version) = suggested {
        assert!(
            semver::Version::parse(&version).is_ok(),
            "Suggested version should be valid semver"
        );
    }
}

#[test]
fn test_suggest_resolution_with_compatible_versions() {
    let conflict = VersionConflict {
        package_name: "test_pkg".to_string(),
        versions:     vec![
            ConflictVersion {
                version:     ">=1.0.0, <2.0.0".to_string(),
                required_by: vec![DependentInfo {
                    name:    "dep1".to_string(),
                    version: "1.0.0".to_string(),
                    path:    "test/Cargo.toml".to_string(),
                }],
            },
            ConflictVersion {
                version:     "^1.2.0".to_string(),
                required_by: vec![DependentInfo {
                    name:    "dep2".to_string(),
                    version: "1.0.0".to_string(),
                    path:    "test/Cargo.toml".to_string(),
                }],
            },
        ],
        dependents:   vec![
            DependentInfo {
                name:    "dep1".to_string(),
                version: "1.0.0".to_string(),
                path:    "test/Cargo.toml".to_string(),
            },
            DependentInfo {
                name:    "dep2".to_string(),
                version: "1.0.0".to_string(),
                path:    "test/Cargo.toml".to_string(),
            },
        ],
    };

    let suggested = conflict::suggest_resolution(&conflict);
    assert!(suggested.is_some());
    // The resolver should pick the highest version that satisfies all constraints
    // In this case, any version >=1.0.0 and <2.0.0 that also satisfies ^1.2.0
    let version = suggested.unwrap();
    assert!(
        semver::VersionReq::parse("^1.2.0")
            .unwrap()
            .matches(&semver::Version::parse(&version).unwrap()),
        "Version {} should satisfy ^1.2.0",
        version
    );
    assert!(
        semver::VersionReq::parse(">=1.0.0, <2.0.0")
            .unwrap()
            .matches(&semver::Version::parse(&version).unwrap()),
        "Version {} should satisfy >=1.0.0, <2.0.0",
        version
    );
}
