use std::env;
use std::ffi::CString;
use std::path::Path;
use std::process;
use rs::{
    analyze_go_code, analyze_js_code, analyze_rust_code, free_string, parse_c_ast, parse_cpp_ast,
    parse_java_ast, parse_js_ast, parse_rust_ast, parse_ts_ast, parse_zig_ast,
};

#[derive(Debug, PartialEq)]
enum Command {
    Parse,
    Analyze,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <command> <file_path>", args[0]);
        eprintln!("Commands:");
        eprintln!("  parse    - Parse file and output AST");
        eprintln!("  analyze  - Analyze code and provide metrics");
        eprintln!();
        eprintln!("Supported extensions:");
        eprintln!("  Parse: .rs, .java, .zig, .c, .h, .js, .jsx, .ts, .tsx, .cpp, .cc, .cxx");
        eprintln!("  Analyze: .rs, .go, .js, .jsx");
        process::exit(1);
    }

    let command = match args[1].to_lowercase().as_str() {
        "parse" => Command::Parse,
        "analyze" => Command::Analyze,
        _ => {
            eprintln!("Error: Unknown command '{}'", args[1]);
            eprintln!("Available commands: parse, analyze");
            process::exit(1);
        }
    };

    let file_path = &args[2];

    // Check if file exists
    if !Path::new(file_path).exists() {
        eprintln!("Error: File '{}' does not exist", file_path);
        process::exit(1);
    }

    // Infer language from file extension and validate for the command
    let language = match infer_language_from_path(file_path, &command) {
        Some(lang) => lang,
        None => {
            eprintln!(
                "Error: Unsupported file extension for '{}' with command '{:?}'",
                file_path, command
            );
            match command {
                Command::Parse => eprintln!("Parse supports: .rs, .java, .zig, .c, .h, .js, .jsx, .ts, .tsx, .cpp, .cc, .cxx"),
                Command::Analyze => eprintln!("Analyze supports: .rs, .go, .js, .jsx"),
            }
            process::exit(1);
        }
    };

    match command {
        Command::Parse => println!("Parsing {} file: {}", language, file_path),
        Command::Analyze => println!("Analyzing {} file: {}", language, file_path),
    }
    println!("----------------------------------------");

    // Convert file path to CString for C FFI
    let c_file_path = match CString::new(file_path.as_str()) {
        Ok(cstring) => cstring,
        Err(_) => {
            eprintln!("Error: Invalid file path contains null bytes");
            process::exit(1);
        }
    };

    // Call the appropriate function based on command and language
    let result_ptr = match command {
        Command::Parse => match language.as_str() {
            "Rust" => parse_rust_ast(c_file_path.as_ptr()),
            "Java" => parse_java_ast(c_file_path.as_ptr()),
            "Zig" => parse_zig_ast(c_file_path.as_ptr()),
            "C" => parse_c_ast(c_file_path.as_ptr()),
            "JavaScript" => parse_js_ast(c_file_path.as_ptr()),
            "TypeScript" => parse_ts_ast(c_file_path.as_ptr()),
            "C++" => parse_cpp_ast(c_file_path.as_ptr()),
            _ => {
                eprintln!("Error: Parsing not supported for language '{}'", language);
                process::exit(1);
            }
        },
        Command::Analyze => match language.as_str() {
            "Rust" => analyze_rust_code(c_file_path.as_ptr()),
            "Go" => analyze_go_code(c_file_path.as_ptr()),
            "JavaScript" => analyze_js_code(c_file_path.as_ptr()),
            _ => {
                eprintln!("Error: Analysis not supported for language '{}'", language);
                process::exit(1);
            }
        },
    };

    // Check if operation was successful
    if result_ptr.is_null() {
        let operation = match command {
            Command::Parse => "parse",
            Command::Analyze => "analyze",
        };
        eprintln!(
            "Error: Failed to {} the file. The file might be malformed or contain invalid syntax.",
            operation
        );
        process::exit(1);
    }

    // Convert the result back to a Rust string and print it
    unsafe {
        if let Ok(c_str) = std::ffi::CStr::from_ptr(result_ptr).to_str() {
            println!("{}", c_str);
        } else {
            eprintln!("Error: Failed to convert result to valid UTF-8");
        }

        // Clean up the allocated memory
        free_string(result_ptr);
    }
}

fn infer_language_from_path(file_path: &str, command: &Command) -> Option<String> {
    let path = Path::new(file_path);
    let extension = path.extension()?.to_str()?;

    match extension.to_lowercase().as_str() {
        "rs" => Some("Rust".to_string()),
        "java" => {
            // Java parsing is supported, but not analysis
            match command {
                Command::Parse => Some("Java".to_string()),
                Command::Analyze => None,
            }
        }
        "zig" => {
            // Zig parsing is supported, but not analysis
            match command {
                Command::Parse => Some("Zig".to_string()),
                Command::Analyze => None,
            }
        }
        "c" | "h" => {
            // C parsing is supported, but not analysis
            match command {
                Command::Parse => Some("C".to_string()),
                Command::Analyze => None,
            }
        }
        "js" | "jsx" => Some("JavaScript".to_string()),
        "ts" | "tsx" => {
            // TypeScript parsing is supported, but not analysis
            match command {
                Command::Parse => Some("TypeScript".to_string()),
                Command::Analyze => None,
            }
        }
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => {
            // C++ parsing is supported, but not analysis
            match command {
                Command::Parse => Some("C++".to_string()),
                Command::Analyze => None,
            }
        }
        "go" => {
            // Go analysis is supported, but not parsing
            match command {
                Command::Parse => None,
                Command::Analyze => Some("Go".to_string()),
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_inference_parse() {
        assert_eq!(
            infer_language_from_path("test.rs", &Command::Parse),
            Some("Rust".to_string())
        );
        assert_eq!(
            infer_language_from_path("Test.java", &Command::Parse),
            Some("Java".to_string())
        );
        assert_eq!(
            infer_language_from_path("main.zig", &Command::Parse),
            Some("Zig".to_string())
        );
        assert_eq!(
            infer_language_from_path("hello.c", &Command::Parse),
            Some("C".to_string())
        );
        assert_eq!(
            infer_language_from_path("script.js", &Command::Parse),
            Some("JavaScript".to_string())
        );
        assert_eq!(
            infer_language_from_path("app.ts", &Command::Parse),
            Some("TypeScript".to_string())
        );
        assert_eq!(
            infer_language_from_path("main.cpp", &Command::Parse),
            Some("C++".to_string())
        );
        assert_eq!(infer_language_from_path("main.go", &Command::Parse), None);
        assert_eq!(
            infer_language_from_path("unknown.txt", &Command::Parse),
            None
        );
    }

    #[test]
    fn test_language_inference_analyze() {
        assert_eq!(
            infer_language_from_path("test.rs", &Command::Analyze),
            Some("Rust".to_string())
        );
        assert_eq!(
            infer_language_from_path("script.js", &Command::Analyze),
            Some("JavaScript".to_string())
        );
        assert_eq!(
            infer_language_from_path("main.go", &Command::Analyze),
            Some("Go".to_string())
        );

        // These should not be supported for analysis
        assert_eq!(
            infer_language_from_path("Test.java", &Command::Analyze),
            None
        );
        assert_eq!(
            infer_language_from_path("main.zig", &Command::Analyze),
            None
        );
        assert_eq!(infer_language_from_path("hello.c", &Command::Analyze), None);
        assert_eq!(infer_language_from_path("app.ts", &Command::Analyze), None);
        assert_eq!(
            infer_language_from_path("main.cpp", &Command::Analyze),
            None
        );
    }
}
