//! This module is in charge of managing the implicit dependencies

use std::fs;
use std::path::PathBuf;

use move_package::source_package::parsed_manifest::{
    Dependency, DependencyKind, GitInfo, InternalDependency, SubstOrRename, Substitution,
};

use crate::Move;

pub fn inject_implicit_dependencies(move_args: &mut Move) {
    move_args.build_config.implicit_dependencies.insert(
        "MoveStdlib".into(),
        Dependency::Internal(InternalDependency {
            kind: DependencyKind::Git(GitInfo {
                git_url: "https://github.com/MystenLabs/sui.git".into(),
                subdir: "crates/sui-framework/packages/move-stdlib".into(),
                git_rev: "framework/mainnet".into(),
            }),
            subst: None,
            digest: None,
            dep_override: true,
        }),
    );

    /*

    let stdlib_path =
        fs::canonicalize(PathBuf::from("./packages/move-stdlib")).expect("unable to locate stdlib");

    println!("===========> {stdlib_path:?}");
    move_args.build_config.implicit_dependencies.insert(
        "MoveStdLib".into(),
        Dependency::Internal(InternalDependency {
            kind: DependencyKind::Local(stdlib_path),
            subst: Some(Substitution::from([(
                "std".into(),
                SubstOrRename::Assign(
                    [
                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0, 0, 0, 0, 1,
                    ]
                    .into(),
                ),
            )])),
            digest: None,
            dep_override: true,
        }),
    );*/
}
