/// Logging utilities for Firework that work correctly in async contexts
/// 
/// Standard println!/eprintln! may not show output immediately in async tasks
/// due to buffering. These functions ensure output is flushed.

use std::io::Write;

/// Print to stdout with immediate flush (works correctly in async contexts)
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        use std::io::Write;
        let _ = writeln!(std::io::stdout(), $($arg)*);
        let _ = std::io::stdout().flush();
    }};
}

/// Print to stderr with immediate flush (works correctly in async contexts)
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        use std::io::Write;
        let _ = writeln!(std::io::stderr(), $($arg)*);
        let _ = std::io::stderr().flush();
    }};
}

/// Print debug information to stderr with immediate flush
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        #[cfg(debug_assertions)]
        {
            use std::io::Write;
            let _ = writeln!(std::io::stderr(), "[DEBUG] {}", format!($($arg)*));
            let _ = std::io::stderr().flush();
        }
    }};
}
