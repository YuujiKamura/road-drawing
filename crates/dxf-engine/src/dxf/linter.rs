//! DXF linter
//!
//! Validates DXF content for common issues:
//! - Handle uniqueness
//! - Section structure (SECTION...ENDSEC pairs)
//! - Two-line chunk integrity (group code + value pairs)
//! - EOF marker

use std::collections::HashSet;

/// Lint error with location and description
#[derive(Debug, Clone, PartialEq)]
pub struct LintError {
    /// Line number (1-based)
    pub line: usize,
    /// Error code
    pub code: LintErrorCode,
    /// Human-readable message
    pub message: String,
}

/// Error codes for lint issues
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintErrorCode {
    /// Duplicate entity handle
    DuplicateHandle,
    /// Section not closed with ENDSEC
    UnclosedSection,
    /// ENDSEC without matching SECTION
    UnmatchedEndsec,
    /// Missing EOF marker
    MissingEof,
    /// Odd number of lines (group code/value pairs broken)
    BrokenChunk,
    /// Invalid group code (not a number)
    InvalidGroupCode,
    /// Empty content
    EmptyContent,
    /// Unexpected content after EOF
    ContentAfterEof,
}

/// Lint result
#[derive(Debug, Clone)]
pub struct LintResult {
    /// List of errors found
    pub errors: Vec<LintError>,
    /// List of warnings (non-fatal issues)
    pub warnings: Vec<LintError>,
    /// Statistics
    pub stats: LintStats,
}

/// Statistics from linting
#[derive(Debug, Clone, Default)]
pub struct LintStats {
    /// Total line count
    pub line_count: usize,
    /// Number of entities found
    pub entity_count: usize,
    /// Number of unique handles
    pub handle_count: usize,
    /// Sections found
    pub sections: Vec<String>,
}

impl LintResult {
    /// Returns true if no errors were found
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns true if errors were found
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// DXF Linter
pub struct DxfLinter;

impl DxfLinter {
    /// Lint DXF content and return results
    pub fn lint(content: &str) -> LintResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut stats = LintStats::default();

        let lines: Vec<&str> = content.lines().collect();
        stats.line_count = lines.len();

        if lines.is_empty() {
            errors.push(LintError {
                line: 0,
                code: LintErrorCode::EmptyContent,
                message: "DXF content is empty".to_string(),
            });
            return LintResult { errors, warnings, stats };
        }

        // Check for odd number of lines (broken chunks)
        if lines.len() % 2 != 0 {
            errors.push(LintError {
                line: lines.len(),
                code: LintErrorCode::BrokenChunk,
                message: format!(
                    "Odd number of lines ({}). DXF requires group code/value pairs.",
                    lines.len()
                ),
            });
        }

        // Track handles, sections, and entities
        let mut handles: HashSet<String> = HashSet::new();
        let mut section_stack: Vec<(usize, String)> = Vec::new();
        let mut found_eof = false;
        let mut eof_line = 0;

        // Parse line pairs
        let mut i = 0;
        while i + 1 < lines.len() {
            let group_code_str = lines[i].trim();
            let value = lines[i + 1].trim();
            let line_num = i + 1; // 1-based

            // Validate group code is numeric
            if group_code_str.parse::<i32>().is_err() {
                errors.push(LintError {
                    line: line_num,
                    code: LintErrorCode::InvalidGroupCode,
                    message: format!("Invalid group code: '{}'", group_code_str),
                });
                i += 2;
                continue;
            }

            let group_code: i32 = group_code_str.parse().unwrap();

            // Check for content after EOF
            if found_eof {
                errors.push(LintError {
                    line: line_num,
                    code: LintErrorCode::ContentAfterEof,
                    message: "Content found after EOF marker".to_string(),
                });
            }

            match group_code {
                0 => {
                    // Entity type or section marker
                    match value {
                        "SECTION" => {
                            // Next pair should be section name (group 2)
                            if i + 3 < lines.len() {
                                let next_code = lines[i + 2].trim();
                                let section_name = lines[i + 3].trim();
                                if next_code == "2" {
                                    section_stack.push((line_num, section_name.to_string()));
                                    stats.sections.push(section_name.to_string());
                                }
                            }
                        }
                        "ENDSEC" => {
                            if section_stack.pop().is_none() {
                                errors.push(LintError {
                                    line: line_num,
                                    code: LintErrorCode::UnmatchedEndsec,
                                    message: "ENDSEC without matching SECTION".to_string(),
                                });
                            }
                        }
                        "EOF" => {
                            found_eof = true;
                            eof_line = line_num;
                        }
                        _ => {
                            // Count entities (LINE, TEXT, CIRCLE, etc.)
                            if !value.starts_with('$') && value != "SECTION" {
                                stats.entity_count += 1;
                            }
                        }
                    }
                }
                5 => {
                    // Entity handle
                    let handle = value.to_uppercase();
                    if handles.contains(&handle) {
                        errors.push(LintError {
                            line: line_num + 1, // Value line
                            code: LintErrorCode::DuplicateHandle,
                            message: format!("Duplicate handle: {}", handle),
                        });
                    } else {
                        handles.insert(handle);
                    }
                }
                _ => {}
            }

            i += 2;
        }

        stats.handle_count = handles.len();

        // Check for unclosed sections
        for (line, name) in section_stack {
            errors.push(LintError {
                line,
                code: LintErrorCode::UnclosedSection,
                message: format!("Section '{}' not closed with ENDSEC", name),
            });
        }

        // Check for EOF
        if !found_eof {
            errors.push(LintError {
                line: lines.len(),
                code: LintErrorCode::MissingEof,
                message: "Missing EOF marker at end of file".to_string(),
            });
        }

        LintResult { errors, warnings, stats }
    }

    /// Quick check - returns true if DXF is valid
    pub fn is_valid(content: &str) -> bool {
        Self::lint(content).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_content() {
        let result = DxfLinter::lint("");
        assert!(result.has_errors());
        assert_eq!(result.errors[0].code, LintErrorCode::EmptyContent);
    }

    #[test]
    fn test_valid_minimal_dxf() {
        let dxf = "0\nSECTION\n2\nHEADER\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nENDSEC\n0\nEOF\n";
        let result = DxfLinter::lint(dxf);
        assert!(result.is_ok(), "Errors: {:?}", result.errors);
        assert_eq!(result.stats.sections, vec!["HEADER", "ENTITIES"]);
    }

    #[test]
    fn test_missing_eof() {
        let dxf = "0\nSECTION\n2\nHEADER\n0\nENDSEC\n";
        let result = DxfLinter::lint(dxf);
        assert!(result.has_errors());
        assert!(result.errors.iter().any(|e| e.code == LintErrorCode::MissingEof));
    }

    #[test]
    fn test_unclosed_section() {
        let dxf = "0\nSECTION\n2\nHEADER\n0\nEOF\n";
        let result = DxfLinter::lint(dxf);
        assert!(result.has_errors());
        assert!(result.errors.iter().any(|e| e.code == LintErrorCode::UnclosedSection));
    }

    #[test]
    fn test_unmatched_endsec() {
        let dxf = "0\nENDSEC\n0\nEOF\n";
        let result = DxfLinter::lint(dxf);
        assert!(result.has_errors());
        assert!(result.errors.iter().any(|e| e.code == LintErrorCode::UnmatchedEndsec));
    }

    #[test]
    fn test_duplicate_handle() {
        let dxf = "\
0\nSECTION\n\
2\nENTITIES\n\
0\nLINE\n\
5\n100\n\
0\nLINE\n\
5\n100\n\
0\nENDSEC\n\
0\nEOF\n";
        let result = DxfLinter::lint(dxf);
        assert!(result.has_errors());
        assert!(result.errors.iter().any(|e| e.code == LintErrorCode::DuplicateHandle));
    }

    #[test]
    fn test_unique_handles() {
        let dxf = "\
0\nSECTION\n\
2\nENTITIES\n\
0\nLINE\n\
5\n100\n\
0\nLINE\n\
5\n101\n\
0\nLINE\n\
5\n102\n\
0\nENDSEC\n\
0\nEOF\n";
        let result = DxfLinter::lint(dxf);
        assert!(result.is_ok(), "Errors: {:?}", result.errors);
        assert_eq!(result.stats.handle_count, 3);
    }

    #[test]
    fn test_broken_chunk_odd_lines() {
        let dxf = "0\nSECTION\n2\n"; // 3 lines - broken
        let result = DxfLinter::lint(dxf);
        assert!(result.has_errors());
        assert!(result.errors.iter().any(|e| e.code == LintErrorCode::BrokenChunk));
    }

    #[test]
    fn test_invalid_group_code() {
        let dxf = "ABC\nSECTION\n0\nEOF\n";
        let result = DxfLinter::lint(dxf);
        assert!(result.has_errors());
        assert!(result.errors.iter().any(|e| e.code == LintErrorCode::InvalidGroupCode));
    }

    #[test]
    fn test_content_after_eof() {
        let dxf = "0\nSECTION\n2\nHEADER\n0\nENDSEC\n0\nEOF\n0\nLINE\n";
        let result = DxfLinter::lint(dxf);
        assert!(result.has_errors());
        assert!(result.errors.iter().any(|e| e.code == LintErrorCode::ContentAfterEof));
    }

    #[test]
    fn test_entity_count() {
        let dxf = "\
0\nSECTION\n\
2\nENTITIES\n\
0\nLINE\n\
5\n100\n\
0\nTEXT\n\
5\n101\n\
0\nCIRCLE\n\
5\n102\n\
0\nENDSEC\n\
0\nEOF\n";
        let result = DxfLinter::lint(dxf);
        assert!(result.is_ok(), "Errors: {:?}", result.errors);
        assert_eq!(result.stats.entity_count, 3);
    }

    #[test]
    fn test_case_insensitive_handles() {
        let dxf = "\
0\nSECTION\n\
2\nENTITIES\n\
0\nLINE\n\
5\n1a\n\
0\nLINE\n\
5\n1A\n\
0\nENDSEC\n\
0\nEOF\n";
        let result = DxfLinter::lint(dxf);
        assert!(result.has_errors());
        assert!(result.errors.iter().any(|e| e.code == LintErrorCode::DuplicateHandle));
    }

    #[test]
    fn test_lint_writer_output() {
        use crate::dxf::entities::DxfLine;
        use crate::dxf::writer::DxfWriter;

        let writer = DxfWriter::new();
        let lines = vec![
            DxfLine::new(0.0, 0.0, 10.0, 10.0),
            DxfLine::new(10.0, 10.0, 20.0, 20.0),
        ];
        let output = writer.write(&lines, &[]);

        let result = DxfLinter::lint(&output);
        assert!(result.is_ok(), "Writer output failed lint: {:?}", result.errors);
    }
}
