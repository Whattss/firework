// Hot reload functionality for development
// 
// Use the firework-dev binary for automatic hot reloading:
//
//   cargo install --path firework --features hot-reload
//   firework-dev                    # watches src/ and runs cargo run
//   firework-dev hello_world        # watches examples/ and runs cargo run --example hello_world
//
// Features:
// - Automatically rebuilds on file changes
// - Ignores editor temp files (vim, neovim, etc.)
// - Kills old process before starting new one
// - Graceful shutdown on Ctrl+C
// - State preservation support

use std::path::PathBuf;

pub struct HotReload {
    _marker: std::marker::PhantomData<()>,
}

impl HotReload {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }

    pub fn enable(self) -> Self {
        self
    }

    pub fn project_root<P: Into<PathBuf>>(self, _path: P) -> Self {
        self
    }

    pub fn watch_path<P: Into<PathBuf>>(self, _path: P) -> Self {
        self
    }

    pub fn ignore_pattern<S: Into<String>>(self, _pattern: S) -> Self {
        self
    }

    pub fn command<S: Into<String>>(self, _cmd: S) -> Self {
        self
    }

    pub async fn start(self) -> crate::Result<()> {
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!("  Hot reload should be used via firework-dev CLI");
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!();
        eprintln!("Install:");
        eprintln!("  cargo install --path firework --features hot-reload");
        eprintln!();
        eprintln!("Usage:");
        eprintln!("  firework-dev                    # main app");
        eprintln!("  firework-dev hello_world        # example");
        eprintln!();
        eprintln!("Features:");
        eprintln!("  - Auto-rebuild on .rs/.toml changes");
        eprintln!("  - Ignores editor temp files");
        eprintln!("  - Process management");
        eprintln!("  - Graceful shutdown");
        eprintln!();
        Ok(())
    }
}

impl Default for HotReload {
    fn default() -> Self {
        Self::new()
    }
}

/// Simplified hot reload macro - just use firework-dev CLI instead
#[macro_export]
macro_rules! hot_reload {
    () => {
        $crate::HotReload::new()
    };
    (enabled) => {
        $crate::HotReload::new().enable()
    };
    (enabled, watch = [$($path:expr),*]) => {
        $crate::HotReload::new().enable()
    };
    (enabled, watch = [$($path:expr),*], ignore = [$($pattern:expr),*]) => {
        $crate::HotReload::new().enable()
    };
}

/// Development helper macro - prints instructions for hot reload
#[macro_export]
macro_rules! dev {
    () => {
        if cfg!(debug_assertions) {
            eprintln!("\n💡 Tip: Use 'firework-dev' for automatic hot reloading\n");
        }
    };
}
