use std::ffi::{c_char, CStr, CString};
use std::fs;
use tree_sitter::{Language, Parser};

pub fn parse_ast(file_path: *const c_char, language: Language) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(file_path) };
    let file_path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    match parse_file_with_language(file_path_str, language) {
        Ok(result) => match CString::new(result) {
            Ok(c_string) => c_string.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

fn parse_file_with_language(
    file_path: &str,
    language: Language,
) -> Result<String, Box<dyn std::error::Error>> {
    let source_code = fs::read_to_string(file_path)?;

    let mut parser = Parser::new();
    parser.set_language(&language)?;

    let tree = parser.parse(&source_code, None).unwrap();
    let root_node = tree.root_node();

    let ast_string = format_node(&root_node, &source_code, 0);
    Ok(ast_string)
}

fn format_node(node: &tree_sitter::Node, source: &str, depth: usize) -> String {
    let indent = "  ".repeat(depth);
    let mut result = format!("{}({}", indent, node.kind());

    if node.child_count() == 0 {
        // Leaf node - include the text
        let text = node.utf8_text(source.as_bytes()).unwrap_or("");
        if !text.trim().is_empty() {
            result.push_str(&format!(" \"{}\"", text.replace('\n', "\\n")));
        }
    }
    result.push(')');

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            result.push('\n');
            result.push_str(&format_node(&child, source, depth + 1));
        }
    }

    result
}