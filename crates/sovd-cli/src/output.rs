use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Text => write!(f, "text"),
            OutputFormat::Json => write!(f, "json"),
        }
    }
}

/// Print output in the requested format.
#[allow(dead_code)]
pub fn print_output<T: Serialize + fmt::Display>(value: &T, format: &OutputFormat) {
    match format {
        OutputFormat::Text => println!("{value}"),
        OutputFormat::Json => {
            if let Ok(json) = serde_json::to_string_pretty(value) {
                println!("{json}");
            } else {
                println!("{value}");
            }
        }
    }
}

/// Print a simple status message.
pub fn print_status(ok: bool, message: &str, format: &OutputFormat) {
    match format {
        OutputFormat::Text => {
            let icon = if ok { "✓" } else { "✗" };
            println!("{icon} {message}");
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({ "ok": ok, "message": message })
            );
        }
    }
}

/// Print an error message.
pub fn print_error(message: &str, format: &OutputFormat) {
    match format {
        OutputFormat::Text => eprintln!("Error: {message}"),
        OutputFormat::Json => {
            eprintln!(
                "{}",
                serde_json::json!({ "error": message })
            );
        }
    }
}

/// Display helper for component lists, etc.
#[allow(dead_code)]
pub fn print_table(headers: &[&str], rows: &[Vec<String>], _format: &OutputFormat) {
    // Calculate column widths
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    // Print header
    let header_line: Vec<String> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| format!("{:<width$}", h, width = widths[i]))
        .collect();
    println!("{}", header_line.join("  "));
    println!("{}", widths.iter().map(|w| "-".repeat(*w)).collect::<Vec<_>>().join("  "));

    // Print rows
    for row in rows {
        let line: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let w = widths.get(i).copied().unwrap_or(cell.len());
                format!("{cell:<w$}")
            })
            .collect();
        println!("{}", line.join("  "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_format_display_text() {
        assert_eq!(format!("{}", OutputFormat::Text), "text");
    }

    #[test]
    fn output_format_display_json() {
        assert_eq!(format!("{}", OutputFormat::Json), "json");
    }

    #[test]
    fn output_format_debug() {
        let text = format!("{:?}", OutputFormat::Text);
        assert_eq!(text, "Text");
        let json = format!("{:?}", OutputFormat::Json);
        assert_eq!(json, "Json");
    }

    #[test]
    fn output_format_clone() {
        let a = OutputFormat::Json;
        let b = a.clone();
        assert_eq!(format!("{b}"), "json");
    }
}
