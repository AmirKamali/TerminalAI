# Adding New Commands to TerminalAI

This guide explains how to add new AI-powered commands to the TerminalAI project. The process has been streamlined to make it easy and developer-friendly.

## Quick Start

To add a new command (e.g., `ps_ai`), follow these steps:

1. **Create the command definition** → `cmd/ps.conf`
2. **Create the binary** → `src/bin/ps_ai.rs`
3. **Add validation function** → `src/command_validator.rs`
4. **Update command parser** → `src/command_parser.rs`
5. **Update CI/CD workflow** → `.github/workflows/ci-cd.yml`
6. **Build and test** → Ensure everything works

## Real-World Example: Adding `ps_ai` Command

This section documents the complete process of adding the `ps_ai` (process management) command as a real-world example.

### Step 1: Create Command Definition File

Create `cmd/ps.conf` with the system prompt and arguments:

```conf
# PS AI Command Configuration

[SYSTEM_PROMPT]
You are an AI assistant specialized in process management operations using the `ps` (process status) command and related process utilities on Linux/Unix systems. Your role is to:

1. Interpret user requests for process monitoring, analysis, and management
2. Generate ROBUST, EXECUTABLE commands that provide useful process information
3. Use appropriate ps flags and options for different use cases
4. Combine ps with other tools like grep, awk, sort for advanced filtering
5. Always provide safe, informative commands that don't interfere with running processes

CRITICAL GUIDELINES:
- Use `ps aux` for comprehensive process listing (all users, all processes)
- Use `ps -ef` for alternative format (BSD vs System V style)
- For specific processes: Use `ps -p PID` or `ps -C process_name`
- For user processes: Use `ps -u username`
- For tree view: Use `ps -ejH` or `ps axjf`
- Always include useful columns: PID, CPU%, MEM%, TIME, COMMAND
- Use grep for filtering: `ps aux | grep pattern`
- Use sort for ordering: `ps aux --sort=-%cpu` (CPU usage descending)
- Use head/tail for limiting output: `ps aux | head -20`

COMMON PATTERNS:
- "show all processes" → `ps aux`
- "find process by name" → `ps aux | grep process_name`
- "show top CPU processes" → `ps aux --sort=-%cpu | head -10`
- "show top memory processes" → `ps aux --sort=-%mem | head -10`
- "show user processes" → `ps -u username`
- "show process tree" → `ps -ejH` or `ps axjf`
- "kill process by name" → `pkill process_name` or `killall process_name`
- "show process details" → `ps -p PID -o pid,ppid,cmd,%cpu,%mem,etime`

SAFETY CONSIDERATIONS:
- Never suggest killing system processes without warning
- Always show process details before suggesting kill operations
- Use `kill -TERM` before `kill -KILL` for graceful termination
- Warn about potential data loss when killing processes

Example response format:
```
To find and analyze processes using high CPU:

ps aux --sort=-%cpu | head -10

This shows the top 10 processes by CPU usage with full details.
```

[ARGUMENTS]
**Usage:** `ps_ai [prompt]`

**Description:** Generate intelligent process management commands based on natural language descriptions.

**Examples:**
- `ps_ai "show all running processes"`
- `ps_ai "find processes using high CPU"`
- `ps_ai "show memory usage for all processes"`
- `ps_ai "find and kill zombie processes"`
- `ps_ai "show process tree for current user"`
- `ps_ai "monitor processes for user john"`
- `ps_ai "find processes containing 'nginx'"`
- `ps_ai "show top 5 memory consuming processes"`

**Features:**
- Natural language processing for process operations
- Automatic flag suggestions (sorting, filtering, formatting)
- Safety warnings for destructive operations
- Cross-platform process management
- Integration with grep, awk, and other filtering tools
```

### Step 2: Create Binary File

Create `src/bin/ps_ai.rs`:

```rust
use anyhow::{Context, Result};
use clap::{Arg, Command};
use terminalai::{
    command_parser, command_validator, extract_and_execute_command, load_config,
    query_provider::QueryProvider,
};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("ps_ai")
        .version("0.1.0")
        .author("Terminal AI Contributors")
        .about("AI-powered process management operations")
        .arg(
            Arg::new("prompt")
                .help("Natural language description of the process operation")
                .required(true)
                .index(1),
        )
        .get_matches();

    let prompt = matches.get_one::<String>("prompt").unwrap();

    // Validate that this is a process-related query
    if let Err(e) = command_validator::validate_ps_query(prompt) {
        eprintln!("❌ {e}");
        std::process::exit(1);
    }

    // Load configuration
    let config = load_config()?;

    // Load command definition
    let (system_prompt, _args_section) = command_parser::load_command_definition("ps")?;

    // Create query provider
    let provider = QueryProvider::new(config).context("Failed to create query provider")?;

    println!("🤖 Processing your process management request...\n");

    // Send query to AI
    match provider.send_query(&system_prompt, prompt).await {
        Ok(response) => {
            println!("📋 AI Response:\n");
            println!("{response}");
            println!("\n{}", "=".repeat(50));

            // Extract and execute commands
            if let Err(e) = extract_and_execute_command(&response) {
                eprintln!("❌ Error executing commands: {e}");
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {e}");
            eprintln!("\n💡 Make sure Ollama is running and configured correctly.");
            eprintln!("Run 'tai init' to set up your configuration.");
            std::process::exit(1);
        }
    }

    Ok(())
}
```

### Step 3: Add Validation Function

Add to `src/command_validator.rs`:

```rust
pub fn validate_ps_query(prompt: &str) -> Result<()> {
    // Keywords that indicate process operations
    let process_keywords = [
        "process", "ps", "processes", "running", "status", "monitor", "top", "cpu", "memory",
        "kill", "terminate", "stop", "start", "restart", "zombie", "orphan", "thread",
        "pid", "process id", "usage", "consumption", "load", "performance", "consumers",
        "show", "list", "display", "view",
    ];

    // Keywords that indicate other operations
    let non_process_keywords = [
        "copy", "cp", "duplicate", "backup", "move", "transfer",
        "search", "grep", "locate", "install", "download", "update", "upgrade", "configure",
    ];

    validate_command_query(
        prompt,
        "ps_ai",
        "process management operations",
        &process_keywords,
        &non_process_keywords,
    )
}
```

### Step 4: Update Command Parser

Add to `src/command_parser.rs`:

```rust
// Add the constant at the top with other definitions
const PS_DEFINITION: &str = include_str!("../cmd/ps.conf");

// Add to the load_command_definition function
pub fn load_command_definition(command_name: &str) -> Result<(String, String)> {
    let content = match command_name {
        "cp" => CP_DEFINITION,
        "grep" => GREP_DEFINITION,
        "find" => FIND_DEFINITION,
        "template" => TEMPLATE_DEFINITION,
        "resolve" => RESOLVE_DEFINITION,
        "ps" => PS_DEFINITION,  // Add this line
        _ => return Err(anyhow::anyhow!("Unknown command: {}", command_name)),
    };
    // ... rest of function
}
```

### Step 5: Update Command Extraction (CRITICAL)

**IMPORTANT**: You must also update the command extraction function to recognize your new command's output format.

Add to `src/lib.rs` in the `extract_commands_from_response` function:

```rust
// Look for actual commands (starting with common command prefixes)
if trimmed.starts_with("cp ")
    || trimmed.starts_with("grep ")
    || trimmed.starts_with("find ")
    || trimmed.starts_with("ps ")  // Add your command prefix here
    || trimmed.starts_with("mkdir ")
    || trimmed.starts_with("npm ")
    || trimmed.starts_with("pip ")
    || trimmed.starts_with("python -m pip ")
    || trimmed.starts_with("rm -rf ")
    || trimmed.starts_with("yarn ")
    || trimmed.starts_with("poetry ")
    || trimmed.starts_with("pipenv ")
{
    commands_to_execute.push(trimmed.to_string());
}
```

**Why this is critical**: Without this step, your command will generate AI responses but won't extract and execute the commands, resulting in "No executable commands found" errors.

### Step 6: Update CI/CD Workflow

Update `.github/workflows/ci-cd.yml` to include the new command:

```yaml
# In the "Test build all binaries" section
- name: Test build all binaries
  run: |
    cargo build --bin tai
    cargo build --bin cp_ai  
    cargo build --bin find_ai
    cargo build --bin grep_ai
    cargo build --bin ps_ai  # Add this line
    cargo build --bin resolve_ai

# In the "Package binaries (Unix)" section
- name: Package binaries (Unix)
  if: matrix.os != 'windows-latest'
  run: |
    mkdir -p release
    cp target/${{ matrix.target }}/release/tai release/
    cp target/${{ matrix.target }}/release/cp_ai release/
    cp target/${{ matrix.target }}/release/find_ai release/
    cp target/${{ matrix.target }}/release/grep_ai release/
    cp target/${{ matrix.target }}/release/ps_ai release/  # Add this line
    cp target/${{ matrix.target }}/release/resolve_ai release/
    tar -czf ${{ matrix.artifact_name }}.tar.gz -C release .

# In the "Package binaries (Windows)" section
- name: Package binaries (Windows)
  if: matrix.os == 'windows-latest'
  run: |
    mkdir release
    cp target/${{ matrix.target }}/release/tai.exe release/
    cp target/${{ matrix.target }}/release/cp_ai.exe release/
    cp target/${{ matrix.target }}/release/find_ai.exe release/
    cp target/${{ matrix.target }}/release/grep_ai.exe release/
    cp target/${{ matrix.target }}/release/ps_ai.exe release/  # Add this line
    cp target/${{ matrix.target }}/release/resolve_ai.exe release/
    Compress-Archive -Path release/* -DestinationPath ${{ matrix.artifact_name }}.zip
```

### Step 7: Add Tests 

Add tests to `src/command_parser.rs`:

```rust
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
```

### Step 8: Build and Test

```bash
# Build the new command
cargo build --bin ps_ai

# Test that it compiles
cargo check --bin ps_ai

# Run all tests to ensure nothing is broken
cargo test

# Test your command (requires Ollama running)
./target/debug/ps_ai "show all processes"
```

### Step 9: Final Code Quality Check

Run the project formatting and linting script to ensure code quality:

```bash
# Run the format project script (ensures GitHub CI compatibility)
./format_project.sh
```

This script will:
- Format all code with `rustfmt`
- Verify build with `cargo check`
- Run `clippy` linting in GitHub CI mode
- Treat warnings as errors (`-D warnings`)

**Important**: This step ensures your code meets the same quality standards as the GitHub CI pipeline.

### Step 10: Verify Integration

The new command should now be included in releases and work with all AI providers. Test with various prompts:

```bash
ps_ai "find all processes over 10gb memory usage"
ps_ai "show top CPU consumers"
ps_ai "kill zombie processes"
ps_ai "show process tree for current user"
```

## Quick Start (Updated)

To add a new command (e.g., `format_ai`), follow these steps:

1. **Create the command definition** → `cmd/format.conf`
2. **Create the binary** → `src/bin/format_ai.rs`
3. **Add validation function** → `src/command_validator.rs`
4. **Update command parser** → `src/command_parser.rs`
5. **Update command extraction** → `src/lib.rs` (CRITICAL - add command prefix)
6. **Update CI/CD workflow** → `.github/workflows/ci-cd.yml`
7. **Build and test** → Ensure everything works
8. **Run format script** → `./format_project.sh` (final quality check)

## Step-by-Step Instructions

### Step 1: Create Command Definition File

Create `cmd/[command].md` with two main sections:

```markdown
# [COMMAND NAME] AI Command

## System Prompt

[Detailed instructions for the AI about how to handle this command type]

## Arguments

**Usage:** `[command]_ai [prompt]`

**Description:** [Brief description of what the command does]

**Examples:**
- `[command]_ai "example usage 1"`
- `[command]_ai "example usage 2"`

**Features:**
- Feature 1
- Feature 2
```

#### Example: `cmd/format.md`

```markdown
# FORMAT AI Command

## System Prompt

You are an AI assistant specialized in code formatting and beautification. Your role is to:

1. Interpret user requests for formatting code files
2. Generate commands using tools like prettier, rustfmt, clang-format, etc.
3. Handle different programming languages appropriately
4. Provide safe, executable formatting commands

IMPORTANT: Your response should contain actual executable commands that accomplish the user's formatting request.

Example response format:
```
To format all JavaScript files in the current directory:

find . -name "*.js" -exec prettier --write {} +

This will format all JS files using Prettier with default settings.
```

## Arguments

**Usage:** `format_ai [prompt]`

**Description:** Generate intelligent code formatting commands based on natural language descriptions.

**Examples:**
- `format_ai "format all Python files in src/ directory"`
- `format_ai "beautify this JavaScript file with 2-space indentation"`
- `format_ai "format Rust code using rustfmt"`

**Features:**
- Multi-language support
- Automatic tool detection
- Safe formatting operations
- Custom formatting options
```

### Step 2: Create Binary Using Template

1. **Copy the template:**
   ```bash
   cp src/bin/template_ai.rs src/bin/[command]_ai.rs
   ```

2. **Replace placeholders in the new file:**
   - `template_ai` → `[command]_ai`
   - `template` → `[command]`
   - `[COMMAND_DESCRIPTION]` → Brief description (e.g., "code formatting")
   - `[COMMAND_TYPE]` → Operation type (e.g., "formatting")

3. **Update validation keywords:**
   ```rust
   let valid_keywords = [
       "format", "beautify", "indent", "style", "pretty", "lint"
   ];
   
   let invalid_keywords = [
       "search", "find", "copy", "delete", "remove", "install"
   ];
   ```

4. **Update the validation call:**
   ```rust
   if let Err(e) = command_validator::validate_command_query(
       prompt,
       "format_ai",                    // Your command name
       "code formatting operations",   // Operation description
       &valid_keywords,
       &invalid_keywords
   ) {
       eprintln!("❌ {}", e);
       std::process::exit(1);
   }
   ```

5. **Update other placeholders:**
   ```rust
   let (system_prompt, _args_section) = command_parser::load_command_definition("format")?;
   println!("🎨 Processing your formatting request...\n");
   ```

### Step 3: Update Cargo.toml

Add your new binary to the `[[bin]]` sections in `Cargo.toml`:

```toml
[[bin]]
name = "format_ai"
path = "src/bin/format_ai.rs"
```

### Step 4: Build and Test

```bash
# Build the new command
cargo build --bin format_ai

# Test that it compiles
cargo check --bin format_ai

# Run all tests to ensure nothing is broken
cargo test

# Test your command (requires Ollama running)
./target/debug/format_ai "format my Python code"
```

### Step 5: Final Code Quality Check

Run the project formatting and linting script to ensure code quality:

```bash
# Run the format project script (ensures GitHub CI compatibility)
./format_project.sh
```

This script will:
- Format all code with `rustfmt`
- Verify build with `cargo check`
- Run `clippy` linting in GitHub CI mode
- Treat warnings as errors (`-D warnings`)

**Important**: This step ensures your code meets the same quality standards as the GitHub CI pipeline.

## Advanced Configuration

### Cross-Platform Compatibility

When creating commands that generate system commands, consider platform differences:

#### Platform-Specific Considerations

**Linux vs macOS/BSD Differences:**
- **Linux**: Supports `--sort` flags, `-eo` format, extended options
- **macOS/BSD**: Limited `ps` options, no `--sort` flag, different column names
- **Windows**: Different command syntax entirely

#### Best Practices for Cross-Platform Commands

1. **Use Universal Commands**: Prefer commands that work everywhere
   ```bash
   # Good (works everywhere)
   ps aux | sort -k4 -nr | head -10
   
   # Bad (Linux only)
   ps aux --sort=-%mem | head -10
   ```

2. **Test on Multiple Platforms**: If possible, test your commands on different OS
3. **Document Platform Limitations**: Include platform-specific notes in your system prompt
4. **Provide Fallbacks**: Give alternative commands for different platforms

#### Example: Cross-Platform System Prompt

```conf
[SYSTEM_PROMPT]
You are an AI assistant for [command] operations. Generate commands that work across platforms.

PLATFORM CONSIDERATIONS:
- Linux: Supports extended options and flags
- macOS/BSD: Limited options, use basic commands
- Windows: Different command syntax

CROSS-PLATFORM PATTERNS:
- Use basic commands that work everywhere
- Avoid platform-specific flags when possible
- Provide alternatives for different platforms
```

### Custom Validation Logic

If you need more complex validation than keyword matching, you can create a custom validation function in `src/command_validator.rs`:

```rust
pub fn validate_[command]_query(prompt: &str) -> Result<()> {
    // Custom logic here
    validate_command_query(
        prompt,
        "[command]_ai",
        "[operation] operations",
        &valid_keywords,
        &invalid_keywords
    )
}
```

Then use it in your binary:
```rust
if let Err(e) = command_validator::validate_[command]_query(prompt) {
    eprintln!("❌ {}", e);
    std::process::exit(1);
}
```

### Testing Your Command

Create tests for your command in the appropriate test files:

1. **Unit tests** in your binary file (optional)
2. **Integration tests** in `tests/integration_tests.rs`
3. **Validation tests** in `src/command_validator.rs` if you added custom validation

Example integration test:
```rust
#[test]
fn test_format_command_definition_exists() {
    let result = command_parser::load_command_definition("format");
    assert!(result.is_ok());
    let (system_prompt, args_section) = result.unwrap();
    assert!(system_prompt.contains("formatting"));
    assert!(args_section.contains("format_ai"));
}
```

## Command Categories and Examples

### File Operations
- `cp_ai` - Copy operations (existing)
- `mv_ai` - Move/rename operations
- `find_ai` - File finding operations
- `sync_ai` - File synchronization

### Text Processing
- `grep_ai` - Text search (existing)
- `sed_ai` - Text replacement/editing
- `awk_ai` - Text processing
- `sort_ai` - Sorting operations

### Development Tools
- `format_ai` - Code formatting
- `lint_ai` - Code linting
- `build_ai` - Build operations
- `test_ai` - Testing operations

### System Operations
- `ps_ai` - Process management
- `disk_ai` - Disk operations
- `net_ai` - Network operations
- `log_ai` - Log analysis

## Validation Keywords Reference

### Common Valid Keywords by Category

**File Operations:**
- Copy: `copy`, `cp`, `duplicate`, `backup`, `clone`
- Move: `move`, `mv`, `rename`, `relocate`
- Search: `search`, `find`, `locate`, `grep`, `scan`

**Development:**
- Format: `format`, `beautify`, `indent`, `style`, `pretty`
- Build: `build`, `compile`, `make`, `cargo`, `npm`
- Test: `test`, `check`, `verify`, `validate`

**System:**
- Process: `process`, `ps`, `kill`, `start`, `stop`
- Network: `network`, `ping`, `curl`, `wget`, `ssh`
- Disk: `disk`, `df`, `du`, `mount`, `unmount`

### Common Invalid Keywords

Most commands should exclude these unless specifically relevant:
- `delete`, `remove`, `rm` (unless it's a deletion command)
- `install`, `download`, `update` (unless it's a package manager)
- `search`, `find`, `grep` (unless it's a search command)
- `copy`, `cp`, `backup` (unless it's a copy command)

## Error Handling

The validation system provides helpful error messages:
- Explains what tools are needed
- Suggests using `tai` for broader operations
- Maintains scope boundaries between commands

Example error output:
```
💡 This command requires using file deletion tools (rm) which is out of scope of format_ai.
format_ai is designed specifically for code formatting operations only.

Use 'tai -p "delete old files"' instead for full system capabilities.
```

## Best Practices

1. **Clear Scope:** Each command should have a well-defined, narrow scope
2. **Good Keywords:** Choose keywords that clearly differentiate your command's purpose
3. **Comprehensive Testing:** Test both valid and invalid inputs
4. **Clear Documentation:** Write clear system prompts and examples
5. **Safety First:** Ensure generated commands are safe and non-destructive
6. **Consistent Naming:** Follow the `[verb]_ai` naming pattern

## Troubleshooting

### Common Issues

1. **Compilation Errors:**
   - Check that all placeholders are replaced
   - Ensure Cargo.toml includes your binary
   - Verify the command definition file exists

2. **Validation Failures:**
   - Review keyword lists for conflicts
   - Test with various prompts
   - Check error messages are informative

3. **Runtime Errors:**
   - Ensure Ollama is running
   - Verify command definition loads correctly
   - Check system prompt format

4. **"Unknown command" Error:**
   - **Problem**: Command returns "Unknown command: [name]"
   - **Solution**: Make sure you've added the command to `src/command_parser.rs`:
     ```rust
     // Add constant at top
     const YOUR_COMMAND_DEFINITION: &str = include_str!("../cmd/your_command.conf");
     
     // Add to match statement
     "your_command" => YOUR_COMMAND_DEFINITION,
     ```

5. **Validation Test Failures:**
   - **Problem**: Tests fail due to complex validation logic
   - **Solution**: Temporarily ignore failing tests with `#[ignore]` and refine validation later
   - **Example**: 
     ```rust
     #[test]
     #[ignore]
     fn test_validate_your_command_query() {
         // Test implementation
     }
     ```

6. **CI/CD Integration Issues:**
   - **Problem**: Command not included in releases
   - **Solution**: Update all three sections in `.github/workflows/ci-cd.yml`:
     - Test build section
     - Unix packaging section  
     - Windows packaging section

7. **Format/Linting Issues:**
   - **Problem**: `./format_project.sh` fails with formatting or linting errors
   - **Solution**: 
     ```bash
     # Auto-format code
     cargo fmt --all
     
     # Fix clippy warnings
     cargo clippy --all-targets --all-features --fix
     
     # Re-run format script
     ./format_project.sh
     ```
   - **Common Issues**:
     - Unused imports (remove them)
     - Format string issues (use `{variable}` syntax)
     - Missing documentation (add `///` comments)

8. **"No executable commands found" Error:**
   - **Problem**: Command generates AI response but shows "No executable commands found"
   - **Solution**: Add your command prefix to `extract_commands_from_response()` in `src/lib.rs`
   - **Example**: 
     ```rust
     if trimmed.starts_with("your_command ") {
         commands_to_execute.push(trimmed.to_string());
     }
     ```
   - **Why**: The command extraction function needs to recognize your command's output format

9. **Platform-Specific Command Failures:**
   - **Problem**: Commands work on one platform but fail on others
   - **Solution**: 
     - Use cross-platform commands in your system prompt
     - Avoid platform-specific flags (like `--sort` on macOS)
     - Test on multiple platforms if possible
     - Provide fallback commands for different platforms
   - **Example**: Use `ps aux | sort -k4 -nr` instead of `ps aux --sort=-%mem`

### Getting Help

- Review existing commands (`cp_ai`, `grep_ai`) as examples
- Check the test files for testing patterns
- Examine the validation system for keyword inspiration
- Look at the GitHub Actions workflow for CI integration

## Complete Checklist for New Commands

When adding a new command, ensure you've completed all these steps:

### ✅ Core Files
- [ ] `cmd/[command].conf` - Command definition with system prompt and arguments
- [ ] `src/bin/[command]_ai.rs` - Binary implementation
- [ ] `src/command_validator.rs` - Validation function (if needed)
- [ ] `src/command_parser.rs` - Command definition loading
- [ ] `src/lib.rs` - Command extraction function (CRITICAL - add command prefix)

### ✅ CI/CD Integration
- [ ] `.github/workflows/ci-cd.yml` - Test build section
- [ ] `.github/workflows/ci-cd.yml` - Unix packaging section
- [ ] `.github/workflows/ci-cd.yml` - Windows packaging section

### ✅ Testing
- [ ] `cargo build --bin [command]_ai` - Compiles successfully
- [ ] `cargo check --bin [command]_ai` - No compilation errors
- [ ] `cargo test` - All existing tests pass
- [ ] Add tests for command definition loading
- [ ] Add validation tests (if custom validation)

### ✅ Code Quality
- [ ] `./format_project.sh` - No formatting or linting issues
- [ ] Code follows project style guidelines
- [ ] No clippy warnings or errors
- [ ] All code is properly formatted with rustfmt

### ✅ Validation
- [ ] Command validates appropriate inputs
- [ ] Command rejects inappropriate inputs
- [ ] Error messages are helpful and clear
- [ ] Keywords are comprehensive and accurate

### ✅ Documentation
- [ ] System prompt is clear and comprehensive
- [ ] Examples are relevant and useful
- [ ] Features list is accurate
- [ ] Usage instructions are clear

### ✅ Integration
- [ ] Command works with all AI providers
- [ ] Command follows naming conventions
- [ ] Command integrates with existing infrastructure
- [ ] Command is included in release packages
- [ ] Command extraction works correctly (no "No executable commands found" errors)
- [ ] Commands work across platforms (Linux, macOS, Windows)

## Lessons Learned from ps_ai Implementation

The `ps_ai` command implementation revealed several important lessons for future command development:

### Critical Issues Discovered

1. **Command Extraction Missing**: The most critical issue was forgetting to add `ps ` to the command extraction function, causing "No executable commands found" errors.

2. **Platform Compatibility**: Generated commands used Linux-specific flags (`--sort`) that don't work on macOS, requiring cross-platform command patterns.

3. **System Prompt Design**: The initial system prompt was too Linux-focused, needing updates for cross-platform compatibility.

### Best Practices Established

1. **Always Update Command Extraction**: This is now a mandatory step in the process
2. **Design for Cross-Platform**: Use universal commands that work on Linux, macOS, and Windows
3. **Test Command Execution**: Verify that generated commands actually execute successfully
4. **Iterative System Prompt Refinement**: Start with basic patterns and refine based on testing

### Common Pitfalls to Avoid

- **Forgetting command extraction**: Always add your command prefix to `extract_commands_from_response()`
- **Platform-specific commands**: Avoid flags that only work on one platform
- **Insufficient testing**: Test with various prompts and on different platforms
- **Poor error messages**: Ensure AI responses provide clear, actionable commands

## Contributing

When submitting a new command:

1. Follow this guide completely
2. Include comprehensive tests
3. Update documentation if needed
4. Ensure all existing tests still pass
5. Test the command with various inputs
6. Complete the checklist above
7. **Test command extraction**: Verify commands are properly extracted and executed
8. **Test cross-platform compatibility**: Ensure commands work on different operating systems

Your new command will be automatically included in releases when merged!

## 🌐 Multi-Provider Compatibility

All new commands automatically work with any configured AI provider:
- 🦙 **Ollama** (Local)
- 🤖 **OpenAI** (GPT-3.5/GPT-4)
- 🧠 **Claude** (Anthropic)
- 💎 **Gemini** (Google)

The provider system is abstracted away, so your commands will work regardless of which AI backend the user has configured. See [MULTI_PROVIDER_GUIDE.md](MULTI_PROVIDER_GUIDE.md) for more details.