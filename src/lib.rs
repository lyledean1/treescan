mod analyzer;
mod ast;
use crate::analyzer::{analyze_code_with_analyzer, CodeAnalyzer};
use libc::c_char;
use std::ffi::CString;
use crate::ast::parse_ast;

/// # Safety
///
/// This function needs to be exported so strings can be derefenced for FFI;
#[no_mangle]
pub unsafe extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}

// Functions exported for FFF
#[no_mangle]
pub extern "C" fn parse_rust_ast(file_path: *const c_char) -> *mut c_char {
    let language = tree_sitter_rust::LANGUAGE;
    parse_ast(file_path, language.into())
}

#[no_mangle]
pub extern "C" fn parse_java_ast(file_path: *const c_char) -> *mut c_char {
    let language = tree_sitter_java::LANGUAGE;
    parse_ast(file_path, language.into())
}

#[no_mangle]
pub extern "C" fn parse_zig_ast(file_path: *const c_char) -> *mut c_char {
    let language = tree_sitter_zig::LANGUAGE;
    parse_ast(file_path, language.into())
}

#[no_mangle]
pub extern "C" fn parse_c_ast(file_path: *const c_char) -> *mut c_char {
    let language = tree_sitter_c::LANGUAGE;
    parse_ast(file_path, language.into())
}

#[no_mangle]
pub extern "C" fn parse_js_ast(file_path: *const c_char) -> *mut c_char {
    let language = tree_sitter_javascript::LANGUAGE;
    parse_ast(file_path, language.into())
}

#[no_mangle]
pub extern "C" fn parse_ts_ast(file_path: *const c_char) -> *mut c_char {
    let language = tree_sitter_typescript::LANGUAGE_TSX;
    parse_ast(file_path, language.into())
}

#[no_mangle]
pub extern "C" fn parse_cpp_ast(file_path: *const c_char) -> *mut c_char {
    let language = tree_sitter_cpp::LANGUAGE;
    parse_ast(file_path, language.into())
}
#[no_mangle]
pub extern "C" fn analyze_rust_code(file_path: *const c_char) -> *mut c_char {
    let language = tree_sitter_rust::LANGUAGE;
    let analyzer = CodeAnalyzer::new_rust_analyzer();
    analyze_code_with_analyzer(file_path, language.into(), analyzer)
}

#[no_mangle]
pub extern "C" fn analyze_go_code(file_path: *const c_char) -> *mut c_char {
    let language = tree_sitter_go::LANGUAGE;
    let analyzer = CodeAnalyzer::new_go_analyzer();
    analyze_code_with_analyzer(file_path, language.into(), analyzer)
}

#[no_mangle]
pub extern "C" fn analyze_js_code(file_path: *const c_char) -> *mut c_char {
    let language = tree_sitter_javascript::LANGUAGE;
    let analyzer = CodeAnalyzer::new_javascript_analyzer();
    analyze_code_with_analyzer(file_path, language.into(), analyzer)
}