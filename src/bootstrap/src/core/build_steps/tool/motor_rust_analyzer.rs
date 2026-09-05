//! Opt-in, workspace-scoped Cargo patches for the Motor build.

use std::ffi::OsStr;
use std::path::{Component, Path};

pub(super) fn cargo_config_args<'a>(
    tool: &Path,
    config: Option<&'a OsStr>,
) -> Option<[&'a OsStr; 2]> {
    // Bootstrap runs Cargo from the Rust root, so a config in the analyzer
    // workspace would not be discovered. Never apply its patches to other tools.
    if !tool.starts_with("src/tools/rust-analyzer")
        || tool.components().any(|part| !matches!(part, Component::Normal(_)))
    {
        return None;
    }
    config.map(|config| [OsStr::new("--config"), config])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyzer_and_proc_macro_server_receive_config() {
        let config = OsStr::new("/external state/patches.toml");
        for tool in ["src/tools/rust-analyzer", "src/tools/rust-analyzer/crates/proc-macro-srv-cli"]
        {
            assert_eq!(
                cargo_config_args(Path::new(tool), Some(config)),
                Some([OsStr::new("--config"), config])
            );
        }
    }

    #[test]
    fn unrelated_tools_and_parent_traversal_are_excluded() {
        for tool in [
            "src/tools/cargo",
            "src/tools/rust-analyzer-other",
            "src/tools/rust-analyzer/../cargo",
            "/src/tools/rust-analyzer",
        ] {
            assert_eq!(cargo_config_args(Path::new(tool), Some(OsStr::new("/config"))), None);
        }
    }

    #[test]
    fn unset_config_leaves_bootstrap_unchanged() {
        assert_eq!(cargo_config_args(Path::new("src/tools/rust-analyzer"), None), None);
    }

    #[test]
    fn config_is_a_single_unmodified_argument() {
        let config = OsStr::new("/external state/quote'\"; $literal.toml");
        let args = cargo_config_args(Path::new("src/tools/rust-analyzer"), Some(config)).unwrap();
        assert_eq!(args, [OsStr::new("--config"), config]);
    }
}
