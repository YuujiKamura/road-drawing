//! Marking command: JSON parse + command execution
//!
//! Ported from Kotlin MarkingCommand.kt and MarkingCommandExecutor.kt
//!
//! Command format:
//! ```json
//! {"type": "crosswalk", "params": {"startOffset": "11000", "stripeCount": "7"}}
//! ```
//!
//! Command list format:
//! ```json
//! {"commands": [{"type": "crosswalk", ...}, {"type": "stopline", ...}]}
//! ```

use std::collections::HashMap;
use dxf_engine::{DxfLine, DxfText};

/// A marking command parsed from JSON
#[derive(Clone, Debug, PartialEq)]
pub struct MarkingCommand {
    pub command_type: String,
    pub params: HashMap<String, String>,
}

/// Result of executing a command
#[derive(Clone, Debug)]
pub struct CommandResult {
    pub lines: Vec<DxfLine>,
    pub texts: Vec<DxfText>,
    pub message: String,
}

/// Parse a single command from JSON string.
/// Format: {"type": "crosswalk", "params": {"key": "value", ...}}
/// Uses manual parsing (no external JSON library, matching Kotlin approach).
pub fn parse_command(json: &str) -> Option<MarkingCommand> {
    let command_type = extract_json_value(json, "type")?;
    if command_type.is_empty() {
        return None;
    }

    let params = if let Some(params_obj) = extract_json_object(json, "params") {
        parse_params(&params_obj)
    } else {
        HashMap::new()
    };

    Some(MarkingCommand {
        command_type,
        params,
    })
}

/// Parse a list of commands from JSON string.
/// Format: {"commands": [{...}, {...}]}
pub fn parse_command_list(json: &str) -> Vec<MarkingCommand> {
    let array_str = match extract_json_array(json, "commands") {
        Some(s) => s,
        None => return vec![],
    };

    // Split array into individual objects by tracking brace depth
    let mut commands = Vec::new();
    let mut depth = 0;
    let mut start = None;

    for (i, ch) in array_str.char_indices() {
        match ch {
            '{' => {
                if depth == 0 {
                    start = Some(i);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(s) = start {
                        let obj = &array_str[s..=i];
                        if let Some(cmd) = parse_command(obj) {
                            commands.push(cmd);
                        }
                    }
                    start = None;
                }
            }
            _ => {}
        }
    }

    commands
}

/// Execute a marking command, producing DXF entities.
pub fn execute_command(
    command: &MarkingCommand,
    centerlines: &[DxfLine],
) -> CommandResult {
    match command.command_type.as_str() {
        "crosswalk" => execute_crosswalk(command, centerlines),
        other => CommandResult {
            lines: vec![],
            texts: vec![],
            message: format!("Unknown command type: {other}"),
        },
    }
}

fn execute_crosswalk(command: &MarkingCommand, centerlines: &[DxfLine]) -> CommandResult {
    use crate::crosswalk::{generate_crosswalk, CrosswalkConfig};

    let config = CrosswalkConfig {
        start_offset: param_f64(&command.params, "startOffset", 11000.0),
        stripe_length: param_f64(&command.params, "stripeLength", 4000.0),
        stripe_width: param_f64(&command.params, "stripeWidth", 450.0),
        stripe_count: param_f64(&command.params, "stripeCount", 7.0) as usize,
        stripe_spacing: param_f64(&command.params, "stripeSpacing", 450.0),
        layer: command.params.get("layer").cloned().unwrap_or_else(|| "横断歩道".to_string()),
    };

    let lines = generate_crosswalk(centerlines, &config);
    let count = lines.len();

    CommandResult {
        lines,
        texts: vec![],
        message: format!("Generated {} crosswalk lines", count),
    }
}

fn param_f64(params: &HashMap<String, String>, key: &str, default: f64) -> f64 {
    params.get(key).and_then(|v| v.parse().ok()).unwrap_or(default)
}

// ── Manual JSON helpers (no external library) ──

fn extract_json_value(json: &str, key: &str) -> Option<String> {
    // Match "key" : "value" or "key": "value"
    let patterns = [
        format!(r#""{}""#, key),
        format!(r#"'{}'"#, key),
    ];

    for pattern in &patterns {
        if let Some(key_pos) = json.find(pattern.as_str()) {
            let after_key = &json[key_pos + pattern.len()..];
            // Skip whitespace and colon
            let after_colon = after_key.trim_start();
            if !after_colon.starts_with(':') {
                continue;
            }
            let after_colon = after_colon[1..].trim_start();
            // Extract quoted value
            if after_colon.starts_with('"') {
                let value_start = 1;
                if let Some(end) = after_colon[value_start..].find('"') {
                    return Some(after_colon[value_start..value_start + end].to_string());
                }
            }
        }
    }
    None
}

fn extract_json_object(json: &str, key: &str) -> Option<String> {
    let pattern = format!(r#""{}""#, key);
    let key_pos = json.find(&pattern)?;
    let after_key = &json[key_pos + pattern.len()..];
    let after_colon = after_key.trim_start();
    if !after_colon.starts_with(':') {
        return None;
    }
    let rest = after_colon[1..].trim_start();

    if !rest.starts_with('{') {
        return None;
    }

    let mut depth = 0;
    for (i, ch) in rest.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(rest[1..i].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

fn extract_json_array(json: &str, key: &str) -> Option<String> {
    let pattern = format!(r#""{}""#, key);
    let key_pos = json.find(&pattern)?;
    let after_key = &json[key_pos + pattern.len()..];
    let after_colon = after_key.trim_start();
    if !after_colon.starts_with(':') {
        return None;
    }
    let rest = after_colon[1..].trim_start();

    if !rest.starts_with('[') {
        return None;
    }

    let mut depth = 0;
    for (i, ch) in rest.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(rest[1..i].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_params(params_str: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    // Find all "key" : "value" pairs
    let mut remaining = params_str;
    while let Some(key_start) = remaining.find('"') {
        let after_key_start = &remaining[key_start + 1..];
        let key_end = match after_key_start.find('"') {
            Some(e) => e,
            None => break,
        };
        let key = after_key_start[..key_end].to_string();
        let after_key = &after_key_start[key_end + 1..];

        // Find colon
        let colon_pos = match after_key.find(':') {
            Some(p) => p,
            None => break,
        };
        let after_colon = after_key[colon_pos + 1..].trim_start();

        // Find value in quotes
        if after_colon.starts_with('"') {
            let val_content = &after_colon[1..];
            if let Some(val_end) = val_content.find('"') {
                let value = val_content[..val_end].to_string();
                map.insert(key, value);
                remaining = &val_content[val_end + 1..];
                continue;
            }
        }
        break;
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    // ================================================================
    // JSON parsing: single command
    // From Kotlin MarkingCommand.fromJson()
    // ================================================================

    #[test]
    fn test_parse_crosswalk_command() {
        let json = r#"{"type": "crosswalk", "params": {"startOffset": "11000", "stripeCount": "7"}}"#;
        let cmd = parse_command(json).unwrap();
        assert_eq!(cmd.command_type, "crosswalk");
        assert_eq!(cmd.params.get("startOffset"), Some(&"11000".to_string()));
        assert_eq!(cmd.params.get("stripeCount"), Some(&"7".to_string()));
    }

    #[test]
    fn test_parse_command_no_params() {
        let json = r#"{"type": "info"}"#;
        let cmd = parse_command(json).unwrap();
        assert_eq!(cmd.command_type, "info");
        assert!(cmd.params.is_empty());
    }

    #[test]
    fn test_parse_command_empty_params() {
        let json = r#"{"type": "crosswalk", "params": {}}"#;
        let cmd = parse_command(json).unwrap();
        assert_eq!(cmd.command_type, "crosswalk");
        assert!(cmd.params.is_empty());
    }

    #[test]
    fn test_parse_command_invalid_json() {
        assert!(parse_command("not json").is_none());
        assert!(parse_command("").is_none());
        assert!(parse_command("{}").is_none()); // no "type" field
    }

    #[test]
    fn test_parse_command_all_crosswalk_params() {
        let json = r#"{
            "type": "crosswalk",
            "params": {
                "startOffset": "11000",
                "stripeLength": "4000",
                "stripeWidth": "450",
                "stripeCount": "7",
                "stripeSpacing": "450",
                "layer": "横断歩道",
                "centerlineLayer": "中心"
            }
        }"#;
        let cmd = parse_command(json).unwrap();
        assert_eq!(cmd.params.len(), 7);
        assert_eq!(cmd.params.get("stripeLength"), Some(&"4000".to_string()));
        assert_eq!(cmd.params.get("layer"), Some(&"横断歩道".to_string()));
    }

    // ================================================================
    // JSON parsing: command list
    // From Kotlin MarkingCommand.listFromJson()
    // ================================================================

    #[test]
    fn test_parse_command_list() {
        let json = r#"{"commands": [
            {"type": "crosswalk", "params": {"startOffset": "5000"}},
            {"type": "crosswalk", "params": {"startOffset": "15000"}}
        ]}"#;
        let cmds = parse_command_list(json);
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0].params.get("startOffset"), Some(&"5000".to_string()));
        assert_eq!(cmds[1].params.get("startOffset"), Some(&"15000".to_string()));
    }

    #[test]
    fn test_parse_command_list_empty() {
        let json = r#"{"commands": []}"#;
        let cmds = parse_command_list(json);
        assert!(cmds.is_empty());
    }

    #[test]
    fn test_parse_command_list_invalid() {
        let cmds = parse_command_list("not json");
        assert!(cmds.is_empty());
    }

    // ================================================================
    // Command execution
    // ================================================================

    #[test]
    fn test_execute_crosswalk_command() {
        let centerlines = vec![
            DxfLine::new(0.0, 0.0, 20000.0, 0.0),
        ];
        let cmd = MarkingCommand {
            command_type: "crosswalk".to_string(),
            params: [
                ("startOffset".to_string(), "11000".to_string()),
                ("stripeCount".to_string(), "7".to_string()),
                ("stripeWidth".to_string(), "450".to_string()),
                ("stripeSpacing".to_string(), "450".to_string()),
                ("stripeLength".to_string(), "4000".to_string()),
            ].into_iter().collect(),
        };
        let result = execute_command(&cmd, &centerlines);
        assert_eq!(result.lines.len(), 28, "7 stripes × 4 lines = 28");
    }

    #[test]
    fn test_execute_unknown_command() {
        let cmd = MarkingCommand {
            command_type: "unknown_type".to_string(),
            params: HashMap::new(),
        };
        let result = execute_command(&cmd, &[]);
        assert!(result.lines.is_empty());
        assert!(result.message.contains("Unknown") || result.message.contains("unknown"));
    }

    #[test]
    fn test_execute_crosswalk_no_centerlines() {
        let cmd = MarkingCommand {
            command_type: "crosswalk".to_string(),
            params: HashMap::new(),
        };
        let result = execute_command(&cmd, &[]);
        assert!(result.lines.is_empty());
    }

    // ================================================================
    // Malformed JSON variants
    // ================================================================

    #[test]
    fn test_parse_command_truncated_json() {
        assert!(parse_command(r#"{"type": "cross"#).is_none());
    }

    #[test]
    fn test_parse_command_missing_closing_brace() {
        assert!(parse_command(r#"{"type": "crosswalk", "params": {"a": "b"}"#).is_some());
        // The outer brace isn't needed for extract_json_value to find "type"
    }

    #[test]
    fn test_parse_command_no_type_value() {
        // "type" key exists but value is empty
        assert!(parse_command(r#"{"type": ""}"#).is_none());
    }

    #[test]
    fn test_parse_command_type_not_string() {
        // "type" value is not in quotes — manual parser won't find it
        assert!(parse_command(r#"{"type": 42}"#).is_none());
    }

    #[test]
    fn test_parse_command_nested_braces_in_params() {
        // Params with nested structure — should still extract top-level params
        let json = r#"{"type": "crosswalk", "params": {"startOffset": "5000", "layer": "テスト"}}"#;
        let cmd = parse_command(json).unwrap();
        assert_eq!(cmd.params.get("startOffset"), Some(&"5000".to_string()));
        assert_eq!(cmd.params.get("layer"), Some(&"テスト".to_string()));
    }

    #[test]
    fn test_parse_command_extra_whitespace() {
        let json = r#"  {  "type"  :  "crosswalk"  ,  "params"  :  {  "startOffset"  :  "1000"  }  }  "#;
        let cmd = parse_command(json).unwrap();
        assert_eq!(cmd.command_type, "crosswalk");
        assert_eq!(cmd.params.get("startOffset"), Some(&"1000".to_string()));
    }

    // ================================================================
    // Missing required params — defaults should apply
    // ================================================================

    #[test]
    fn test_execute_crosswalk_missing_all_params() {
        let centerlines = vec![DxfLine::new(0.0, 0.0, 20000.0, 0.0)];
        let cmd = MarkingCommand {
            command_type: "crosswalk".to_string(),
            params: HashMap::new(), // all defaults
        };
        let result = execute_command(&cmd, &centerlines);
        // Default: 7 stripes × 4 lines = 28
        assert_eq!(result.lines.len(), 28);
    }

    #[test]
    fn test_execute_crosswalk_partial_params() {
        let centerlines = vec![DxfLine::new(0.0, 0.0, 20000.0, 0.0)];
        let cmd = MarkingCommand {
            command_type: "crosswalk".to_string(),
            params: [("stripeCount".to_string(), "3".to_string())].into_iter().collect(),
        };
        let result = execute_command(&cmd, &centerlines);
        assert_eq!(result.lines.len(), 12, "3 stripes × 4 = 12");
    }

    #[test]
    fn test_execute_crosswalk_invalid_param_value() {
        let centerlines = vec![DxfLine::new(0.0, 0.0, 20000.0, 0.0)];
        let cmd = MarkingCommand {
            command_type: "crosswalk".to_string(),
            params: [("stripeCount".to_string(), "abc".to_string())].into_iter().collect(),
        };
        let result = execute_command(&cmd, &centerlines);
        // "abc" can't parse as f64 → falls back to default 7
        assert_eq!(result.lines.len(), 28);
    }

    // ================================================================
    // Unknown command type
    // ================================================================

    #[test]
    fn test_execute_stopline_unknown() {
        let cmd = MarkingCommand {
            command_type: "stopline".to_string(),
            params: HashMap::new(),
        };
        let result = execute_command(&cmd, &[]);
        assert!(result.lines.is_empty());
        assert!(result.message.contains("stopline"));
    }

    #[test]
    fn test_execute_empty_type() {
        // This shouldn't happen if parse_command filters empty types,
        // but test execute_command robustness
        let cmd = MarkingCommand {
            command_type: "".to_string(),
            params: HashMap::new(),
        };
        let result = execute_command(&cmd, &[]);
        assert!(result.lines.is_empty());
    }

    // ================================================================
    // Command list edge cases
    // ================================================================

    #[test]
    fn test_parse_command_list_no_commands_key() {
        let cmds = parse_command_list(r#"{"data": [{"type": "crosswalk"}]}"#);
        assert!(cmds.is_empty(), "Missing 'commands' key should return empty");
    }

    #[test]
    fn test_parse_command_list_single_command() {
        let json = r#"{"commands": [{"type": "crosswalk", "params": {"stripeCount": "5"}}]}"#;
        let cmds = parse_command_list(json);
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].command_type, "crosswalk");
        assert_eq!(cmds[0].params.get("stripeCount"), Some(&"5".to_string()));
    }

    #[test]
    fn test_parse_command_list_mixed_valid_invalid() {
        let json = r#"{"commands": [
            {"type": "crosswalk", "params": {"startOffset": "1000"}},
            {"invalid": "no type here"},
            {"type": "stopline", "params": {}}
        ]}"#;
        let cmds = parse_command_list(json);
        // First and third are valid, second has no "type" → skipped
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0].command_type, "crosswalk");
        assert_eq!(cmds[1].command_type, "stopline");
    }

    #[test]
    fn test_parse_command_list_empty_string() {
        assert!(parse_command_list("").is_empty());
    }

    #[test]
    fn test_parse_command_list_commands_not_array() {
        let cmds = parse_command_list(r#"{"commands": "not an array"}"#);
        assert!(cmds.is_empty());
    }

    // ================================================================
    // param_f64 edge cases via execute
    // ================================================================

    #[test]
    fn test_execute_crosswalk_zero_stripe_count() {
        let centerlines = vec![DxfLine::new(0.0, 0.0, 20000.0, 0.0)];
        let cmd = MarkingCommand {
            command_type: "crosswalk".to_string(),
            params: [("stripeCount".to_string(), "0".to_string())].into_iter().collect(),
        };
        let result = execute_command(&cmd, &centerlines);
        assert!(result.lines.is_empty(), "stripeCount=0 should produce no lines");
    }

    #[test]
    fn test_execute_crosswalk_custom_layer() {
        let centerlines = vec![DxfLine::new(0.0, 0.0, 20000.0, 0.0)];
        let cmd = MarkingCommand {
            command_type: "crosswalk".to_string(),
            params: [
                ("stripeCount".to_string(), "1".to_string()),
                ("layer".to_string(), "カスタム層".to_string()),
            ].into_iter().collect(),
        };
        let result = execute_command(&cmd, &centerlines);
        assert_eq!(result.lines.len(), 4);
        assert!(result.lines.iter().all(|l| l.layer == "カスタム層"));
    }

    // ================================================================
    // CommandResult message content
    // ================================================================

    #[test]
    fn test_execute_crosswalk_result_message() {
        let centerlines = vec![DxfLine::new(0.0, 0.0, 20000.0, 0.0)];
        let cmd = MarkingCommand {
            command_type: "crosswalk".to_string(),
            params: [("stripeCount".to_string(), "3".to_string())].into_iter().collect(),
        };
        let result = execute_command(&cmd, &centerlines);
        assert!(result.message.contains("12"), "Message should contain line count: {}", result.message);
        assert!(result.texts.is_empty(), "Crosswalk should produce no texts");
    }

    // ================================================================
    // MarkingCommand equality
    // ================================================================

    #[test]
    fn test_marking_command_equality() {
        let cmd1 = MarkingCommand {
            command_type: "crosswalk".to_string(),
            params: [("a".to_string(), "1".to_string())].into_iter().collect(),
        };
        let cmd2 = MarkingCommand {
            command_type: "crosswalk".to_string(),
            params: [("a".to_string(), "1".to_string())].into_iter().collect(),
        };
        assert_eq!(cmd1, cmd2);
    }

    #[test]
    fn test_marking_command_inequality() {
        let cmd1 = MarkingCommand {
            command_type: "crosswalk".to_string(),
            params: HashMap::new(),
        };
        let cmd2 = MarkingCommand {
            command_type: "stopline".to_string(),
            params: HashMap::new(),
        };
        assert_ne!(cmd1, cmd2);
    }
}
