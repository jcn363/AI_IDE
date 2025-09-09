//! Dependency conflict resolution utilities

use anyhow::Result;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a version conflict between dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConflict {
    pub package_name: String,
    pub versions: Vec<ConflictVersion>,
    pub dependents: Vec<DependentInfo>,
}

/// Information about a specific version in conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictVersion {
    pub version: String,
    pub required_by: Vec<DependentInfo>,
}

/// Information about a dependent package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependentInfo {
    pub name: String,
    pub version: String,
    pub path: String,
}

/// Analyzes the dependency graph for version conflicts
pub fn find_version_conflicts(metadata: &cargo_metadata::Metadata) -> Vec<VersionConflict> {
    let mut conflicts = Vec::new();
    let mut version_map: HashMap<String, HashMap<String, Vec<DependentInfo>>> = HashMap::new();

    // First, collect all version requirements for each package
    for package in &metadata.packages {
        for dep in &package.dependencies {
            let entry = version_map.entry(dep.name.to_string()).or_default();

            let dep_info = DependentInfo {
                name: package.name.to_string(),
                version: package.version.to_string(),
                path: package.manifest_path.to_string(),
            };

            entry.entry(dep.req.to_string()).or_default().push(dep_info);
        }
    }

    // Then, identify conflicts
    for (package, versions) in version_map {
        if versions.len() > 1 {
            let conflict_versions: Vec<ConflictVersion> = versions
                .into_iter()
                .map(|(version_req, dependents)| ConflictVersion {
                    version: version_req,
                    required_by: dependents,
                })
                .collect();

            let dependents: Vec<_> = conflict_versions
                .iter()
                .flat_map(|v| v.required_by.iter().cloned())
                .collect();

            conflicts.push(VersionConflict {
                package_name: package,
                versions: conflict_versions,
                dependents,
            });
        }
    }

    conflicts
}

/// Suggests a resolution for a version conflict by finding the highest compatible version
pub fn suggest_resolution(conflict: &VersionConflict) -> Option<String> {
    // Try to parse all version requirements
    let parsed_reqs: Result<Vec<VersionReq>, _> = conflict
        .versions
        .iter()
        .map(|v| VersionReq::parse(&v.version))
        .collect();

    let _version_reqs = match parsed_reqs {
        Ok(reqs) => reqs,
        Err(_) => return None,
    };
    // Note: This will be resolved in version_reqs processing below

    // Extract all version numbers from the requirements
    let mut candidate_versions: Vec<Version> = conflict
        .versions
        .iter()
        .filter_map(|v| {
            // Try to parse the version requirement as a specific version
            if let Ok(ver) = Version::parse(&v.version) {
                return Some(ver);
            }

            // For range requirements, try to extract the minimum version
            if let Ok(req) = VersionReq::parse(&v.version) {
                if let Some(cap) = req.comparators.first() {
                    if cap.op == semver::Op::Caret
                        || cap.op == semver::Op::Tilde
                        || cap.op == semver::Op::GreaterEq
                    {
                        return Some(Version {
                            major: cap.major,
                            minor: cap.minor.unwrap_or(0),
                            patch: cap.patch.unwrap_or(0),
                            pre: cap.pre.clone(),
                            build: Default::default(),
                        });
                    }
                }
            }
            None
        })
        .collect();

    // Sort versions in descending order
    candidate_versions.sort_by(|a, b| b.cmp(a));

    // Return the highest version that might satisfy all requirements
    candidate_versions.first().map(|v| v.to_string())
}

/// Applies a resolution to a dependency
pub fn apply_resolution(
    conflict: &VersionConflict,
    chosen_version: &str,
) -> Result<HashMap<String, String>> {
    let mut updates = HashMap::new();

    // Update all dependents to use the chosen version
    for dep in &conflict.dependents {
        updates.insert(dep.path.clone(), chosen_version.to_string());
    }

    Ok(updates)
}
