use std::{collections::BTreeMap, path::PathBuf};

use clap::Parser;
use move_compiler::editions::Flavor;
use move_package::{LintFlag, source_package::parsed_manifest::Dependencies};
use move_packages_build::implicit_dependencies;

#[derive(Debug, Parser, Clone, Eq, PartialEq, PartialOrd, Default)]
#[clap(about)]
pub struct BuildConfig {
    /// Compile in 'dev' mode. The 'dev-addresses' and 'dev-dependencies' fields will be used if
    /// this flag is set. This flag is useful for development of packages that expose named
    /// addresses that are not set to a specific value.
    #[clap(name = "dev-mode", short = 'd', long = "dev", global = true)]
    pub dev_mode: bool,

    /// Installation directory for compiled artifacts. Defaults to current directory.
    #[clap(long = "install-dir", global = true)]
    pub install_dir: Option<PathBuf>,

    /// Force recompilation of all packages
    #[clap(name = "force-recompilation", long = "force", global = true)]
    pub force_recompilation: bool,

    /// Optional location to save the lock file to, if package resolution succeeds.
    #[clap(skip)]
    pub lock_file: Option<PathBuf>,

    /// Only fetch dependency repos to MOVE_HOME
    #[clap(long = "fetch-deps-only", global = true)]
    pub fetch_deps_only: bool,

    /// Skip fetching latest git dependencies
    #[clap(long = "skip-fetch-latest-git-deps", global = true)]
    pub skip_fetch_latest_git_deps: bool,

    /// If set, ignore any compiler warnings
    #[clap(long = move_compiler::command_line::SILENCE_WARNINGS, global = true)]
    pub silence_warnings: bool,

    /// If set, warnings become errors
    #[clap(long = move_compiler::command_line::WARNINGS_ARE_ERRORS, global = true)]
    pub warnings_are_errors: bool,

    /// If set, reports errors at JSON
    #[clap(long = move_compiler::command_line::JSON_ERRORS, global = true)]
    pub json_errors: bool,

    #[clap(flatten)]
    pub lint_flag: LintFlag,

    /// Additional dependencies to be automatically included in every package
    #[clap(skip)]
    pub implicit_dependencies: Dependencies,
}

impl From<BuildConfig> for move_package::BuildConfig {
    fn from(value: BuildConfig) -> Self {
        move_package::BuildConfig {
            dev_mode: value.dev_mode,
            test_mode: false,
            generate_docs: false,
            save_disassembly: false,
            install_dir: value.install_dir,
            force_recompilation: value.force_recompilation,
            lock_file: value.lock_file,
            fetch_deps_only: value.fetch_deps_only,
            skip_fetch_latest_git_deps: value.skip_fetch_latest_git_deps,
            default_flavor: Some(Flavor::Core),
            default_edition: None,
            deps_as_root: false,
            silence_warnings: value.silence_warnings,
            warnings_are_errors: value.warnings_are_errors,
            json_errors: value.json_errors,
            additional_named_addresses: BTreeMap::new(),
            lint_flag: value.lint_flag,
            implicit_dependencies: implicit_dependencies(),
        }
    }
}
