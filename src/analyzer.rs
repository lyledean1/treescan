use std::ffi::{CStr, CString};
use std::fs;
use libc::c_char;
use serde_json::{json, Value};
use tree_sitter::{Language, Parser, Query, QueryCursor, StreamingIterator};

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub rule_name: String,
    pub severity: Severity,
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub text: String,
    pub suggestion: Option<String>,
    pub score_impact: f64,
}

#[derive(Debug, Clone)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Style,
}

impl Severity {
    pub fn base_score_impact(&self) -> f64 {
        match self {
            Severity::Error => -3.0,   // Critical issues
            Severity::Warning => -1.5, // Important issues
            Severity::Info => -0.4,    // Minor issues
            Severity::Style => -0.2,   // Style preferences
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnalysisRule {
    pub name: String,
    pub query: String,
    pub severity: Severity,
    pub message_template: String,
    pub suggestion: Option<String>,
    pub weight_multiplier: f64, // Custom weight for specific rules
}

impl AnalysisRule {
    pub fn new(
        name: String,
        query: String,
        severity: Severity,
        message: String,
        suggestion: Option<String>,
    ) -> Self {
        Self {
            name,
            query,
            severity,
            message_template: message,
            suggestion,
            weight_multiplier: 1.0, // Default weight
        }
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight_multiplier = weight;
        self
    }
}

#[derive(Debug, Clone)]
pub struct CodeScore {
    pub overall_score: f64,
    pub max_score: f64,
    pub total_issues: usize,
    pub breakdown: ScoreBreakdown,
    pub rating: String,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct ScoreBreakdown {
    pub errors: usize,
    pub warnings: usize,
    pub info_issues: usize,
    pub style_issues: usize,
    pub error_deduction: f64,
    pub warning_deduction: f64,
    pub info_deduction: f64,
    pub style_deduction: f64,
    pub size_bonus: f64,
}

pub struct CodeAnalyzer {
    rules: Vec<AnalysisRule>,
}

impl CodeAnalyzer {
    pub fn new() -> Self {
        CodeAnalyzer { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: AnalysisRule) {
        self.rules.push(rule);
    }

    pub fn analyze(
        &self,
        source_code: &str,
        language: &Language,
    ) -> Result<Vec<AnalysisResult>, Box<dyn std::error::Error>> {
        let mut parser = Parser::new();
        parser.set_language(language)?;

        let tree = parser.parse(source_code, None).unwrap();
        let mut results = Vec::new();

        for rule in &self.rules {
            let query = Query::new(language, &rule.query)?;
            let mut cursor = QueryCursor::new();

            let mut matches = cursor.matches(&query, tree.root_node(), source_code.as_bytes());
            while let Some(match_) = matches.next() {
                for capture in match_.captures {
                    let node = capture.node;
                    let start = node.start_position();
                    let text = node.utf8_text(source_code.as_bytes()).unwrap_or("");

                    if self.should_report(&rule.name, &node, source_code) {
                        let score_impact =
                            rule.severity.base_score_impact() * rule.weight_multiplier;

                        results.push(AnalysisResult {
                            rule_name: rule.name.clone(),
                            severity: rule.severity.clone(),
                            message: rule.message_template.clone(),
                            line: start.row + 1,
                            column: start.column + 1,
                            text: text.to_string(),
                            suggestion: rule.suggestion.clone(),
                            score_impact,
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    pub fn analyze_with_score(
        &self,
        source_code: &str,
        language: &Language,
    ) -> Result<(Vec<AnalysisResult>, CodeScore), Box<dyn std::error::Error>> {
        let results = self.analyze(source_code, language)?;
        let score = self.calculate_score(&results, source_code);
        Ok((results, score))
    }

    fn calculate_score(&self, results: &[AnalysisResult], source_code: &str) -> CodeScore {
        let base_score = 10.0;
        let line_count = source_code.lines().count();

        let mut breakdown = ScoreBreakdown {
            errors: 0,
            warnings: 0,
            info_issues: 0,
            style_issues: 0,
            error_deduction: 0.0,
            warning_deduction: 0.0,
            info_deduction: 0.0,
            style_deduction: 0.0,
            size_bonus: 0.0,
        };

        // Count issues and calculate deductions
        for result in results {
            match result.severity {
                Severity::Error => {
                    breakdown.errors += 1;
                    breakdown.error_deduction += result.score_impact.abs();
                }
                Severity::Warning => {
                    breakdown.warnings += 1;
                    breakdown.warning_deduction += result.score_impact.abs();
                }
                Severity::Info => {
                    breakdown.info_issues += 1;
                    breakdown.info_deduction += result.score_impact.abs();
                }
                Severity::Style => {
                    breakdown.style_issues += 1;
                    breakdown.style_deduction += result.score_impact.abs();
                }
            }
        }

        let total_deduction = breakdown.error_deduction
            + breakdown.warning_deduction
            + breakdown.info_deduction
            + breakdown.style_deduction;

        // Apply size-based adjustments
        let size_factor = if line_count > 200 {
            // Larger files get some leniency for minor issues
            let leniency = ((line_count as f64 - 200.0) / 1000.0).min(0.3); // Max 30% leniency
            breakdown.size_bonus =
                leniency * (breakdown.info_deduction + breakdown.style_deduction);
            1.0 + leniency
        } else if line_count < 50 {
            // Smaller files are held to higher standards
            0.9
        } else {
            1.0
        };

        // Calculate final score
        let adjusted_deduction = total_deduction / size_factor;
        let overall_score = (base_score - adjusted_deduction).max(0.0);
        let rounded_score = (overall_score * 10.0).round() / 10.0;

        let (rating, summary) = self.get_rating_and_summary(rounded_score, &breakdown);

        CodeScore {
            overall_score: rounded_score,
            max_score: base_score,
            total_issues: results.len(),
            breakdown,
            rating,
            summary,
        }
    }

    fn get_rating_and_summary(&self, score: f64, breakdown: &ScoreBreakdown) -> (String, String) {
        let rating = match score {
            9.0..=10.0 => "Excellent",
            7.5..=8.9 => "Good",
            6.0..=7.4 => "Fair",
            4.0..=5.9 => "Poor",
            _ => "Critical",
        }
        .to_string();

        let summary = if breakdown.errors > 0 {
            format!(
                "Code has {} critical errors that need immediate attention",
                breakdown.errors
            )
        } else if breakdown.warnings > 5 {
            "Multiple warnings detected - consider addressing them".to_string()
        } else if breakdown.info_issues > 10 {
            "Many minor issues found - good opportunity for cleanup".to_string()
        } else if score >= 9.0 {
            "Excellent code quality with minimal issues".to_string()
        } else if score >= 7.5 {
            "Good code quality with room for minor improvements".to_string()
        } else {
            "Code needs improvement in several areas".to_string()
        };

        (rating, summary)
    }

    fn should_report(&self, rule_name: &str, node: &tree_sitter::Node, source_code: &str) -> bool {
        match rule_name {
            "large_function" => {
                let line_count = node.end_position().row - node.start_position().row;
                line_count > 50
            }
            "missing_docs" => source_code[..node.start_byte()].contains("pub fn"),
            "go_missing_error_check" => self.is_unchecked_go_error(node, source_code),
            "go_large_function" => {
                let line_count = node.end_position().row - node.start_position().row;
                line_count > 40
            }
            _ => true,
        }
    }

    fn is_unchecked_go_error(&self, node: &tree_sitter::Node, source_code: &str) -> bool {
        if let Some(parent) = node.parent() {
            if parent.kind() == "assignment_statement" {
                let text_around = &source_code
                    [node.start_byte()..std::cmp::min(node.end_byte() + 200, source_code.len())];
                return !text_around.contains("if err != nil")
                    && !text_around.contains("if error != nil");
            }
        }
        true
    }

    // Factory methods for different language analyzers
    pub fn new_rust_analyzer() -> Self {
        let mut analyzer = CodeAnalyzer::new();

        analyzer.add_rule(
            AnalysisRule::new(
                "syntax_error".to_string(),
                "(ERROR) @error".to_string(),
                Severity::Error,
                "Syntax error".to_string(),
                None,
            )
            .with_weight(2.0),
        ); // Critical - double impact

        analyzer.add_rule(AnalysisRule::new(
            "unwrap_usage".to_string(),
            r#"(call_expression function: (field_expression field: (field_identifier) @method) (#eq? @method "unwrap")) @call"#.to_string(),
            Severity::Warning,
            "Use of .unwrap() can cause panics".to_string(),
            Some("Consider using .expect() with a message or proper error handling".to_string()),
        ).with_weight(1.5)); // Higher impact - can cause runtime panics

        analyzer.add_rule(
            AnalysisRule::new(
                "large_function".to_string(),
                "(function_item name: (identifier) @name) @function".to_string(),
                Severity::Style,
                "Function may be too large".to_string(),
                Some("Consider breaking into smaller functions".to_string()),
            )
            .with_weight(1.2),
        ); // Slightly higher impact for maintainability

        analyzer
    }

    pub fn new_javascript_analyzer() -> Self {
        let mut analyzer = CodeAnalyzer::new();

        analyzer.add_rule(
            AnalysisRule::new(
                "syntax_error".to_string(),
                "(ERROR) @error".to_string(),
                Severity::Error,
                "Syntax error".to_string(),
                None,
            )
            .with_weight(2.0),
        );

        analyzer.add_rule(AnalysisRule::new(
            "console_log".to_string(),
            r#"(call_expression function: (member_expression object: (identifier) @obj property: (property_identifier) @prop) (#eq? @obj "console") (#eq? @prop "log")) @call"#.to_string(),
            Severity::Info,
            "Console.log statement found".to_string(),
            Some("Remove before production".to_string()),
        ).with_weight(0.5)); // Lower impact - common in development

        analyzer.add_rule(
            AnalysisRule::new(
                "var_usage".to_string(),
                "(variable_declaration kind: \"var\") @var".to_string(),
                Severity::Warning,
                "Use of 'var' keyword".to_string(),
                Some("Use 'let' or 'const' instead".to_string()),
            )
            .with_weight(1.3),
        ); // Higher impact - can lead to scoping issues

        analyzer
    }

    pub fn new_go_analyzer() -> Self {
        let mut analyzer = CodeAnalyzer::new();

        analyzer.add_rule(
            AnalysisRule::new(
                "syntax_error".to_string(),
                "(ERROR) @error".to_string(),
                Severity::Error,
                "Syntax error".to_string(),
                None,
            )
            .with_weight(2.0),
        );

        analyzer.add_rule(AnalysisRule::new(
            "go_missing_error_check".to_string(),
            r#"(assignment_statement left: (expression_list (identifier) @var (identifier) @err) (#eq? @err "err")) @assignment"#.to_string(),
            Severity::Warning,
            "Potential unchecked error".to_string(),
            Some("Check for 'if err != nil' after this assignment".to_string()),
        ).with_weight(1.8)); // High impact - can hide important errors

        analyzer.add_rule(AnalysisRule::new(
            "go_unused_variable".to_string(),
            r#"(short_var_declaration left: (expression_list (identifier) @var) (#not-match? @var "^_"))"#.to_string(),
            Severity::Info,
            "Potentially unused variable".to_string(),
            Some("Use _ if variable is intentionally unused".to_string()),
        ).with_weight(0.7)); // Lower impact - compiler catches this

        analyzer.add_rule(
            AnalysisRule::new(
                "go_panic_usage".to_string(),
                r#"(call_expression function: (identifier) @func (#eq? @func "panic")) @call"#
                    .to_string(),
                Severity::Warning,
                "Use of panic()".to_string(),
                Some("Consider returning an error instead of panicking".to_string()),
            )
            .with_weight(1.6),
        ); // High impact - can crash programs

        analyzer.add_rule(
            AnalysisRule::new(
                "go_large_function".to_string(),
                "(function_declaration name: (identifier) @name) @function".to_string(),
                Severity::Style,
                "Function may be too large".to_string(),
                Some("Consider breaking into smaller functions".to_string()),
            )
            .with_weight(1.1),
        );

        analyzer.add_rule(AnalysisRule::new(
            "go_too_many_parameters".to_string(),
            r#"(function_declaration parameters: (parameter_list (parameter_declaration) @param1 (parameter_declaration) @param2 (parameter_declaration) @param3 (parameter_declaration) @param4 (parameter_declaration) @param5 (parameter_declaration) @param6)) @function"#.to_string(),
            Severity::Style,
            "Function has too many parameters".to_string(),
            Some("Consider using a struct or reducing parameters".to_string()),
        ).with_weight(1.3)); // Higher impact - affects API usability

        analyzer.add_rule(
            AnalysisRule::new(
                "go_global_variable".to_string(),
                r#"(source_file (var_declaration) @global_var)"#.to_string(),
                Severity::Info,
                "Global variable declaration".to_string(),
                Some("Consider if this global variable is necessary".to_string()),
            )
            .with_weight(0.8),
        ); // Moderate impact - can be necessary

        analyzer.add_rule(AnalysisRule::new(
            "go_missing_package_doc".to_string(),
            r#"(source_file (package_clause) @package (#not-has-prev-sibling? @package comment))"#.to_string(),
            Severity::Info,
            "Package missing documentation".to_string(),
            Some("Add package documentation comment".to_string()),
        ).with_weight(0.6)); // Lower impact for internal packages

        analyzer.add_rule(
            AnalysisRule::new(
                "go_todo_comment".to_string(),
                r#"(comment) @comment (#match? @comment "TODO|FIXME|XXX|HACK")"#.to_string(),
                Severity::Info,
                "TODO comment found".to_string(),
                Some("Consider addressing this TODO item".to_string()),
            )
            .with_weight(0.3),
        ); // Very low impact - often intentional

        analyzer.add_rule(
            AnalysisRule::new(
                "go_empty_if_block".to_string(),
                r#"(if_statement consequence: (block) @block (#eq? @block "{}"))"#.to_string(),
                Severity::Style,
                "Empty if block".to_string(),
                Some("Remove empty if block or add implementation".to_string()),
            )
            .with_weight(1.0),
        );

        analyzer.add_rule(AnalysisRule::new(
            "go_magic_number".to_string(),
            r#"(int_literal) @number (#not-eq? @number "0") (#not-eq? @number "1") (#not-eq? @number "2")"#.to_string(),
            Severity::Style,
            "Magic number found".to_string(),
            Some("Consider using a named constant".to_string()),
        ).with_weight(0.4)); // Lower impact - context dependent

        analyzer.add_rule(AnalysisRule::new(
            "go_deep_nesting".to_string(),
            r#"(if_statement consequence: (block (if_statement consequence: (block (if_statement consequence: (block (if_statement) @deep_if))))))"#.to_string(),
            Severity::Style,
            "Deep nesting detected (4+ levels)".to_string(),
            Some("Consider extracting nested logic into separate functions".to_string()),
        ).with_weight(1.4)); // Higher impact - affects readability significantly

        analyzer
    }

    pub fn format_score_as_json(&self, results: &[AnalysisResult], score: &CodeScore) -> Value {
        json!({
            "score": score.overall_score,
            "max_score": score.max_score,
            "rating": score.rating,
            "summary": score.summary,
            "total_issues": score.total_issues,
            "breakdown": {
                "errors": score.breakdown.errors,
                "warnings": score.breakdown.warnings,
                "info_issues": score.breakdown.info_issues,
                "style_issues": score.breakdown.style_issues,
                "deductions": {
                    "from_errors": score.breakdown.error_deduction,
                    "from_warnings": score.breakdown.warning_deduction,
                    "from_info": score.breakdown.info_deduction,
                    "from_style": score.breakdown.style_deduction
                },
                "size_bonus": score.breakdown.size_bonus
            },
            "issues": results.iter().map(|r| json!({
                "rule": r.rule_name,
                "severity": format!("{:?}", r.severity),
                "message": r.message,
                "line": r.line,
                "column": r.column,
                "text": r.text,
                "suggestion": r.suggestion,
                "score_impact": r.score_impact
            })).collect::<Vec<_>>()
        })
    }
}


pub fn analyze_code_with_analyzer(
    file_path: *const c_char,
    language: Language,
    analyzer: CodeAnalyzer,
) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(file_path) };
    let file_path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    match run_analysis(file_path_str, language, analyzer) {
        Ok(result) => match CString::new(result) {
            Ok(c_string) => c_string.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

fn run_analysis(
    file_path: &str,
    language: Language,
    analyzer: CodeAnalyzer,
) -> Result<String, Box<dyn std::error::Error>> {
    let source_code = fs::read_to_string(file_path)?;
    let (results, score) = analyzer.analyze_with_score(&source_code, &language)?;

    // Use the new JSON formatting method
    let output = analyzer.format_score_as_json(&results, &score);
    Ok(serde_json::to_string_pretty(&output)?)
}
