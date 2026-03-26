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

/// Parse a single command from JSON string
/// Format: {"type": "crosswalk", "params": {"key": "value", ...}}
pub fn parse_command(json: &str) -> Option<MarkingCommand> {
    todo!("Implement: parse JSON to MarkingCommand")
}

/// Parse a list of commands from JSON string
/// Format: {"commands": [{...}, {...}]}
pub fn parse_command_list(json: &str) -> Vec<MarkingCommand> {
    todo!("Implement: parse JSON array of commands")
}

/// Execute a marking command, producing DXF entities
pub fn execute_command(
    command: &MarkingCommand,
    centerlines: &[DxfLine],
) -> CommandResult {
    todo!("Implement: dispatch command to crosswalk/stopline/etc generator")
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
}
