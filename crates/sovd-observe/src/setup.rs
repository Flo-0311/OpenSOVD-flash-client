use tracing_subscriber::{fmt, EnvFilter, prelude::*};

/// Initialize tracing/logging with sensible defaults.
///
/// Respects `RUST_LOG` for filtering. Defaults to `info` level.
/// If `level_override` is provided, it is used instead of the default
/// (but `RUST_LOG` still takes precedence if set).
/// Set `json = true` for structured JSON output (CI/automation).
pub fn init_tracing(json: bool) {
    init_tracing_with_level(json, None);
}

/// Initialize tracing with an optional programmatic level override.
///
/// This avoids the need for `std::env::set_var("RUST_LOG", ...)` which
/// is unsound in multi-threaded / async contexts (deprecated since Rust 1.66).
pub fn init_tracing_with_level(json: bool, level_override: Option<&str>) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(level_override.unwrap_or("info"))
    });

    if json {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_target(true).with_thread_ids(false))
            .init();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_filter_parses_info() {
        let filter = EnvFilter::new("info");
        let debug_str = format!("{filter}");
        assert!(debug_str.contains("info"));
    }

    #[test]
    fn env_filter_parses_debug() {
        let filter = EnvFilter::new("debug");
        let debug_str = format!("{filter}");
        assert!(debug_str.contains("debug"));
    }

    #[test]
    fn env_filter_parses_module_filter() {
        let filter = EnvFilter::new("sovd_core=debug,info");
        let debug_str = format!("{filter}");
        assert!(debug_str.contains("sovd_core"));
    }
}
