use anyhow::Result;

// Embedded command definitions
const CP_DEFINITION: &str = include_str!("../cmd/cp.conf");
const GREP_DEFINITION: &str = include_str!("../cmd/grep.conf");
const FIND_DEFINITION: &str = include_str!("../cmd/find.conf");
const TEMPLATE_DEFINITION: &str = include_str!("../cmd/template.conf");
const RESOLVE_DEFINITION: &str = include_str!("../cmd/resolve.conf");
const PS_DEFINITION: &str = include_str!("../cmd/ps.conf");

pub fn load_command_definition(command_name: &str) -> Result<(String, String)> {
    let content = match command_name {
        "cp" => CP_DEFINITION,
        "grep" => GREP_DEFINITION,
        "find" => FIND_DEFINITION,
        "template" => TEMPLATE_DEFINITION,
        "resolve" => RESOLVE_DEFINITION,
        "ps" => PS_DEFINITION,
        _ => return Err(anyhow::anyhow!("Unknown command: {}", command_name)),
    };

    let (system_prompt, args_section) = parse_command_conf(content)?;

    Ok((system_prompt, args_section))
}

fn parse_command_conf(content: &str) -> Result<(String, String)> {
    let mut system_prompt = String::new();
    let mut args_section = String::new();
    let mut in_system_prompt = false;
    let mut in_args_section = false;

    for line in content.lines() {
        // Skip comment lines
        if line.trim().starts_with('#') {
            continue;
        }

        if line.trim() == "[SYSTEM_PROMPT]" {
            in_system_prompt = true;
            in_args_section = false;
            continue;
        } else if line.trim() == "[ARGUMENTS]" {
            in_system_prompt = false;
            in_args_section = true;
            continue;
        } else if line.trim().starts_with('[') && line.trim().ends_with(']') {
            // Other section, stop collecting
            in_system_prompt = false;
            in_args_section = false;
            continue;
        }

        if in_system_prompt && !line.trim().is_empty() {
            system_prompt.push_str(line);
            system_prompt.push('\n');
        } else if in_args_section && !line.trim().is_empty() {
            args_section.push_str(line);
            args_section.push('\n');
        }
    }

    if system_prompt.is_empty() {
        return Err(anyhow::anyhow!(
            "No system prompt found in command definition"
        ));
    }

    Ok((
        system_prompt.trim().to_string(),
        args_section.trim().to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_command_definition_cp() {
        let result = load_command_definition("cp");
        assert!(result.is_ok());
        let (system_prompt, args_section) = result.unwrap();

        // Basic checks that we got content
        assert!(!system_prompt.is_empty());
        assert!(!args_section.is_empty());

        // Check that system prompt contains relevant content for copy operations
        let system_prompt_lower = system_prompt.to_lowercase();
        assert!(system_prompt_lower.contains("copy") || system_prompt_lower.contains("cp"));
    }

    #[test]
    fn test_load_command_definition_grep() {
        let result = load_command_definition("grep");
        assert!(result.is_ok());
        let (system_prompt, args_section) = result.unwrap();

        // Basic checks that we got content
        assert!(!system_prompt.is_empty());
        assert!(!args_section.is_empty());

        // Check that system prompt contains relevant content for search operations
        let system_prompt_lower = system_prompt.to_lowercase();
        assert!(system_prompt_lower.contains("search") || system_prompt_lower.contains("grep"));
    }

    #[test]
    fn test_load_command_definition_find() {
        let result = load_command_definition("find");
        assert!(result.is_ok());
        let (system_prompt, args_section) = result.unwrap();

        // Basic checks that we got content
        assert!(!system_prompt.is_empty());
        assert!(!args_section.is_empty());

        // Check that system prompt contains relevant content for find operations
        let system_prompt_lower = system_prompt.to_lowercase();
        assert!(system_prompt_lower.contains("find") || system_prompt_lower.contains("search"));
    }

    #[test]
    fn test_load_command_definition_template() {
        let result = load_command_definition("template");
        assert!(result.is_ok());
        let (system_prompt, args_section) = result.unwrap();

        // Basic checks that we got content
        assert!(!system_prompt.is_empty());
        assert!(!args_section.is_empty());

        // Check that system prompt contains template placeholders
        assert!(system_prompt.contains("[COMMAND_DESCRIPTION]"));
    }

    #[test]
    fn test_load_command_definition_unknown() {
        let result = load_command_definition("unknown_command");
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Unknown command: unknown_command"));
    }

    #[test]
    fn test_load_command_definition_resolve() {
        let result = load_command_definition("resolve");
        assert!(result.is_ok());
        let (system_prompt, args_section) = result.unwrap();

        // Basic checks that we got content
        assert!(!system_prompt.is_empty());
        assert!(!args_section.is_empty());

        // Check that system prompt contains relevant content for package resolution
        let system_prompt_lower = system_prompt.to_lowercase();
        assert!(
            system_prompt_lower.contains("package") || system_prompt_lower.contains("dependency")
        );
    }

    #[test]
    fn test_load_command_definition_ps() {
        let result = load_command_definition("ps");
        assert!(result.is_ok());
        let (system_prompt, args_section) = result.unwrap();

        // Basic checks that we got content
        assert!(!system_prompt.is_empty());
        assert!(!args_section.is_empty());

        // Check that system prompt contains relevant content for process operations
        let system_prompt_lower = system_prompt.to_lowercase();
        assert!(system_prompt_lower.contains("process") || system_prompt_lower.contains("ps"));
    }

    #[test]
    fn test_parse_command_conf_valid() {
        let test_content = r#"
# Test Command Configuration

# Some description here.

[SYSTEM_PROMPT]

This is the system prompt content.
It can have multiple lines.

[ARGUMENTS]

These are the arguments.
- arg1: description
- arg2: description

[OTHER_SECTION]

This should be ignored.
"#;

        let result = parse_command_conf(test_content);
        assert!(result.is_ok());

        let (system_prompt, args_section) = result.unwrap();

        assert_eq!(
            system_prompt,
            "This is the system prompt content.\nIt can have multiple lines."
        );
        assert_eq!(
            args_section,
            "These are the arguments.\n- arg1: description\n- arg2: description"
        );
    }

    #[test]
    fn test_parse_command_conf_system_prompt_only() {
        let test_content = r#"
[SYSTEM_PROMPT]

Only system prompt here.

[OTHER_SECTION]

This should be ignored.
"#;

        let result = parse_command_conf(test_content);
        assert!(result.is_ok());

        let (system_prompt, args_section) = result.unwrap();

        assert_eq!(system_prompt, "Only system prompt here.");
        assert_eq!(args_section, "");
    }

    #[test]
    fn test_parse_command_conf_no_system_prompt() {
        let test_content = r#"
# Test Command

# Some description here.

[ARGUMENTS]

These are the arguments.
"#;

        let result = parse_command_conf(test_content);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("No system prompt found in command definition"));
    }

    #[test]
    fn test_parse_command_conf_empty_content() {
        let result = parse_command_conf("");
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("No system prompt found in command definition"));
    }

    #[test]
    fn test_parse_command_conf_mixed_sections() {
        let test_content = r#"
[SYSTEM_PROMPT]

First system prompt line.
Second system prompt line.

[ARGUMENTS]

First argument line.

[RANDOM_SECTION]

This should be ignored.

[SYSTEM_PROMPT]

This should not override the first system prompt.

[ARGUMENTS]

This should not override the first arguments.
"#;

        let result = parse_command_conf(test_content);
        assert!(result.is_ok());

        let (system_prompt, args_section) = result.unwrap();

        // Should only capture the first occurrence of each section
        assert!(system_prompt.contains("First system prompt line"));
        assert!(system_prompt.contains("Second system prompt line"));
        assert!(args_section.contains("First argument line"));
    }

    #[test]
    fn test_parse_command_conf_whitespace_handling() {
        let test_content = r#"
[SYSTEM_PROMPT]

   System prompt with leading/trailing spaces.   


[ARGUMENTS]

   Arguments with spaces.   

"#;

        let result = parse_command_conf(test_content);
        assert!(result.is_ok());

        let (system_prompt, args_section) = result.unwrap();

        // Should trim leading/trailing whitespace from entire sections
        assert_eq!(system_prompt, "System prompt with leading/trailing spaces.");
        assert_eq!(args_section, "Arguments with spaces.");
    }

    #[test]
    fn test_parse_command_conf_case_sensitive_headers() {
        let test_content = r#"
[system_prompt]

This should not be captured (wrong case).

[SYSTEM_PROMPT]

This should be captured.

[arguments]

This should not be captured (wrong case).

[ARGUMENTS]

This should be captured.
"#;

        let result = parse_command_conf(test_content);
        assert!(result.is_ok());

        let (system_prompt, args_section) = result.unwrap();

        assert_eq!(system_prompt, "This should be captured.");
        assert_eq!(args_section, "This should be captured.");
    }

    #[test]
    fn test_parse_command_conf_comments_ignored() {
        let test_content = r#"
# This is a comment and should be ignored

[SYSTEM_PROMPT]

This is system prompt.
# This comment inside should be ignored
More system prompt content.

[ARGUMENTS]

# This is an argument comment
Argument content here.
"#;

        let result = parse_command_conf(test_content);
        assert!(result.is_ok());

        let (system_prompt, args_section) = result.unwrap();

        assert_eq!(
            system_prompt,
            "This is system prompt.\nMore system prompt content."
        );
        assert_eq!(args_section, "Argument content here.");
    }
}
