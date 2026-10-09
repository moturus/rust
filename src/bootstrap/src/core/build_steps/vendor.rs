//! Handles the vendoring process for the bootstrap system.
//!
//! This module ensures that all required Cargo dependencies are gathered
//! and stored in the `<src>/<VENDOR_DIR>` directory.
use std::env;
use std::path::PathBuf;

use crate::core::build_steps::tool::SUBMODULES_FOR_RUSTBOOK;
use crate::core::builder::{Builder, CommandLineStep, RunConfig, ShouldRun};
use crate::utils::exec::command;

/// The name of the directory where vendored dependencies are stored.
pub const VENDOR_DIR: &str = "vendor";

/// Returns the cargo workspaces to vendor for `x vendor` and dist tarballs.
///
/// Returns a `Vec` of `(path_to_manifest, submodules_required)` where
/// `path_to_manifest` is the cargo workspace, and `submodules_required` is
/// the set of submodules that must be available.
pub fn default_paths_to_vendor(builder: &Builder<'_>) -> Vec<(PathBuf, Vec<&'static str>)> {
    [
        ("src/tools/cargo/Cargo.toml", vec!["src/tools/cargo"]),
        ("src/tools/clippy/clippy_test_deps/Cargo.toml", vec![]),
        ("src/tools/rust-analyzer/Cargo.toml", vec![]),
        ("compiler/rustc_codegen_cranelift/Cargo.toml", vec![]),
        ("compiler/rustc_codegen_gcc/Cargo.toml", vec![]),
        ("library/Cargo.toml", vec![]),
        ("library/stdarch/Cargo.toml", vec![]),
        ("src/bootstrap/Cargo.toml", vec![]),
        ("src/tools/rustbook/Cargo.toml", SUBMODULES_FOR_RUSTBOOK.into()),
        ("src/tools/rustc-perf/Cargo.toml", vec!["src/tools/rustc-perf"]),
        ("src/tools/opt-dist/Cargo.toml", vec![]),
        ("src/doc/book/packages/trpl/Cargo.toml", vec![]),
    ]
    .into_iter()
    .map(|(path, submodules)| (builder.src.join(path), submodules))
    .collect()
}

/// Defines the vendoring step in the bootstrap process.
///
/// This step executes `cargo vendor` to collect all dependencies
/// and store them in the `<src>/<VENDOR_DIR>` directory.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct Vendor {
    /// Additional paths to synchronize during vendoring.
    pub(crate) sync_args: Vec<PathBuf>,
    /// Determines whether vendored dependencies use versioned directories.
    pub(crate) versioned_dirs: bool,
    /// The root directory of the source code.
    ///
    /// Vendored dependencies will be stored in <root_dir>/vendor and
    /// <root_dir>/library/vendor unless overridden by `output_dir`.
    pub(crate) root_dir: PathBuf,
    /// The root directory for storing vendored dependencies in <output_dir>/vendor
    /// and <output_dir>/library/vendor.
    pub(crate) output_dir: Option<PathBuf>,
    /// Only vendor crates necessary by the library workspace.
    pub(crate) only_library_workspace: bool,
}

impl CommandLineStep for Vendor {
    type Output = VendorOutput;
    const IS_HOST: bool = true;

    fn should_run(run: ShouldRun<'_>) -> ShouldRun<'_> {
        run.alias("placeholder")
    }

    fn is_default_step(_builder: &Builder<'_>) -> bool {
        true
    }

    fn make_run(run: RunConfig<'_>) {
        run.builder.ensure(Vendor {
            sync_args: run.builder.config.cmd.vendor_sync_args(),
            versioned_dirs: run.builder.config.cmd.vendor_versioned_dirs(),
            root_dir: run.builder.src.clone(),
            output_dir: None,
            only_library_workspace: false,
        });
    }

    /// Executes the vendoring process.
    ///
    /// This function runs `cargo vendor` and ensures all required submodules
    /// are initialized before vendoring begins.
    fn run(self, builder: &Builder<'_>) -> Self::Output {
        let _guard = builder.group(&format!("Vendoring sources to {:?}", self.root_dir));

        // Motor: the rust-analyzer lockfile resolves two crates to patched sources that only
        // its workspace-scoped Cargo config names. Vendor it alone with that config, and never
        // let a Motor build re-resolve a lockfile.
        let motor_config = env::var_os("MOTOR_RUST_ANALYZER_CARGO_CONFIG");
        let analyzer = builder.src.join("src/tools/rust-analyzer/Cargo.toml");

        let config = if self.only_library_workspace {
            String::new()
        } else {
            let mut cmd = command(&builder.initial_cargo);
            cmd.arg("vendor");
            if motor_config.is_some() {
                cmd.arg("--locked");
            }

            if self.versioned_dirs {
                cmd.arg("--versioned-dirs");
            }

            let to_vendor = default_paths_to_vendor(builder);
            // These submodules must be present for `x vendor` to work.
            for (_, submodules) in &to_vendor {
                for submodule in submodules {
                    builder.build.require_submodule(submodule, None);
                }
            }

            // Sync these paths by default.
            for (p, _) in &to_vendor {
                if motor_config.is_none() || *p != analyzer {
                    cmd.arg("--sync").arg(p);
                }
            }

            // Also sync explicitly requested paths.
            for sync_arg in self.sync_args {
                cmd.arg("--sync").arg(sync_arg);
            }

            // Reuse vendored dependencies when building source tarball for offline support.
            if builder.config.vendor {
                cmd.arg("--respect-source-config")
                    .arg("--config")
                    .arg(builder.src.join(".cargo").join("config.toml"));
            }

            // Will read the libstd Cargo.toml
            // which uses the unstable `public-dependency` feature.
            cmd.env("RUSTC_BOOTSTRAP", "1");
            cmd.env("RUSTC", &builder.initial_rustc);

            cmd.current_dir(&self.root_dir);
            match &self.output_dir {
                None => cmd.arg(VENDOR_DIR),
                Some(output_dir) => cmd.arg(output_dir.join(VENDOR_DIR)),
            };

            let config = cmd.run_capture_stdout(builder).stdout();

            if let Some(motor_config) = &motor_config {
                // `--no-delete` keeps what the run above vendored.
                let mut cmd = command(&builder.initial_cargo);
                cmd.args(["vendor", "--locked", "--no-delete"]);
                if self.versioned_dirs {
                    cmd.arg("--versioned-dirs");
                }
                if builder.config.vendor {
                    cmd.arg("--respect-source-config")
                        .arg("--config")
                        .arg(builder.src.join(".cargo").join("config.toml"));
                }
                cmd.arg("--config").arg(motor_config).arg("--manifest-path").arg(&analyzer);
                cmd.env("RUSTC_BOOTSTRAP", "1");
                cmd.env("RUSTC", &builder.initial_rustc);
                cmd.current_dir(&self.root_dir);
                match &self.output_dir {
                    None => cmd.arg(VENDOR_DIR),
                    Some(output_dir) => cmd.arg(output_dir.join(VENDOR_DIR)),
                };
                cmd.run_capture_stdout(builder);
            }

            config
        };

        let mut cmd = command(&builder.initial_cargo);
        cmd.arg("vendor");
        if motor_config.is_some() {
            cmd.arg("--locked");
        }

        if self.versioned_dirs {
            cmd.arg("--versioned-dirs");
        }

        // Reuse vendored dependencies when building source tarball for offline support.
        if builder.config.vendor {
            cmd.arg("--respect-source-config")
                .arg("--config")
                .arg(builder.src.join("library").join(".cargo").join("config.toml"));
        }

        // Will read the libstd Cargo.toml
        // which uses the unstable `public-dependency` feature.
        cmd.env("RUSTC_BOOTSTRAP", "1");
        cmd.env("RUSTC", &builder.initial_rustc);

        cmd.current_dir(self.root_dir.join("library"));
        match &self.output_dir {
            None => cmd.arg(VENDOR_DIR),
            Some(output_dir) => cmd.arg(output_dir.join("library").join(VENDOR_DIR)),
        };

        let config_library = cmd.run_capture_stdout(builder).stdout();

        VendorOutput { config, config_library }
    }
}

/// Stores the result of the vendoring step.
#[derive(Debug, Clone)]
pub(crate) struct VendorOutput {
    pub(crate) config: String,
    pub(crate) config_library: String,
}
