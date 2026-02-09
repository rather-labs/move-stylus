// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

//! This library is in charge of manage and return the list of implicit dependencies supported by
//! the Move to WASM compiler.
use implicit_dependency_info::ImplicitDepenencyInfo;

use move_package::source_package::parsed_manifest::{
    Dependencies, Dependency, DependencyKind, GitInfo, InternalDependency,
};

mod implicit_dependency_info;

/// Base git repository where dependencies are located
const GIT_BASE_REPOSITORY: &str = "https://github.com/rather-labs/move-stylus/";

/// List of implicit dependencies supported by the compiler
const DEPENDENCIES: [ImplicitDepenencyInfo; 2] = [
    ImplicitDepenencyInfo {
        name: "MoveStdlib",
        subdir: "move-packages/move-stdlib",
        rev: "feat/stylus-framework_as_implicit_dependency",
    },
    ImplicitDepenencyInfo {
        name: "StylusFramework",
        subdir: "move-packages/stylus-framework",
        rev: "feat/stylus-framework_as_implicit_dependency",
    },
];

/// Process the `DEPENDENCIES` table and return them ready to be injected
pub fn implicit_dependencies() -> Dependencies {
    let mut dependencies = Dependencies::new();
    for dependency in DEPENDENCIES {
        dependencies.insert(
            dependency.name.into(),
            Dependency::Internal(InternalDependency {
                kind: DependencyKind::Git(GitInfo {
                    git_url: GIT_BASE_REPOSITORY.into(),
                    subdir: dependency.subdir.into(),
                    git_rev: dependency.rev.into(),
                }),
                subst: None,
                digest: None,
                dep_override: true,
            }),
        );
    }

    dependencies
}
