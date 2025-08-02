use anyhow::{Context, Result};
use clap::{Arg, Command};
use colored::*;
use std::path::Path;
use std::process::Command as StdCommand;
use terminalai::{command_parser, command_validator, load_config, query_provider::QueryProvider};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("resolve_ai")
        .version("0.1.0")
        .author("Terminal AI Contributors")
        .about("AI-powered package dependency resolution")
        .arg(
            Arg::new("type")
                .short('t')
                .long("type")
                .help("Package manager type (npm or python)")
                .value_parser(["npm", "python"])
                .value_name("TYPE"),
        )
        .arg(
            Arg::new("package")
                .short('p')
                .long("package")
                .help("Package name with specific version (e.g., 'react@18.2.0' or 'requests==2.31.0')")
                .value_name("PACKAGE"),
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .help("Dependency file path (e.g., 'package.json', 'requirements.txt')")
                .value_name("FILE"),
        )
        .arg(
            Arg::new("env")
                .short('e')
                .long("env")
                .help("Python environment type (venv or conda). Default: venv (uses pip)")
                .value_parser(["venv", "conda"])
                .value_name("ENV"),
        )
        .group(
            clap::ArgGroup::new("input_mode")
                .args(["type", "package"])
                .multiple(true)
                .conflicts_with("file"),
        )
        .group(
            clap::ArgGroup::new("file_mode")
                .args(["file"])
                .multiple(false)
                .conflicts_with("input_mode"),
        )
        .get_matches();

    // Get environment preference (default to venv/pip)
    let env_type = matches
        .get_one::<String>("env")
        .map(|s| s.as_str())
        .unwrap_or("venv");

    // Handle different input modes
    let (package_type, package, is_file_mode) = if let Some(file_path) =
        matches.get_one::<String>("file")
    {
        // File mode - detect package manager type from file
        let detected_type = detect_package_manager_from_file(file_path)?;
        (detected_type, file_path.clone(), true)
    } else {
        // Single package mode
        let package_type = matches
            .get_one::<String>("type")
            .ok_or_else(|| anyhow::anyhow!("Package type is required when not using file mode"))?;
        let package = matches
            .get_one::<String>("package")
            .ok_or_else(|| anyhow::anyhow!("Package is required when not using file mode"))?;

        // Validate that this is a package resolution query
        if let Err(e) = command_validator::validate_resolve_query(package_type, package) {
            eprintln!("❌ {e}");
            std::process::exit(1);
        }

        // Check for common invalid packages and provide immediate feedback
        let final_package =
            if let Some(suggestion) = check_for_common_invalid_packages(package_type, package) {
                eprintln!("⚠️  {suggestion}");

                // For typos, check if we can auto-correct and continue
                if let Some(corrected_package) = detect_common_typos(package) {
                    println!("\n🤖 Proceeding with the corrected package: {corrected_package}");
                    corrected_package
                } else {
                    std::process::exit(1);
                }
            } else {
                package.clone()
            };

        (package_type.clone(), final_package, false)
    };

    // Load configuration
    let config = load_config()?;

    // Load command definition
    let (system_prompt, _args_section) = command_parser::load_command_definition("resolve")?;

    // Create query provider
    let provider = QueryProvider::new(config).context("Failed to create query provider")?;

    println!("🤖 Processing your package resolution request...\n");

    if is_file_mode {
        println!("📁 Dependency file: {package}");
        println!("🔧 Detected type: {package_type}");
    } else {
        println!("📦 Package: {package}");
        println!("🔧 Type: {package_type}");
    }

    // Create a concise prompt for the AI - start with BASIC installation only
    let prompt = if is_file_mode {
        let package_manager = if package_type == "python" {
            match env_type {
                "conda" => "conda",
                _ => "pip",
            }
        } else {
            "npm"
        };
        format!(
            "Generate the BASIC installation command for {package_type} file '{package}' using {package_manager}. Start with the standard installation command only. Do NOT include cache clearing, purging, or force reinstall commands - these will be used only if the basic installation fails. Provide ONLY the basic executable command."
        )
    } else {
        // Detect common invalid packages upfront
        let upfront_detection = if package_type == "python" {
            if package.starts_with("python==") || package.starts_with("python3==") {
                format!("\n\nWARNING: '{package}' is NOT a pip package. Python interpreter versions must be installed using system package managers:\n- pyenv: pyenv install 3.13.3 && pyenv global 3.13.3 (RECOMMENDED)\n- macOS: brew install python@3.13\n- conda: conda install python=3.13\n\nGenerate system installation commands instead of pip commands.")
            } else if package.starts_with("node==") {
                format!("\n\nWARNING: '{package}' is NOT a pip package. Node.js must be installed using:\n- nvm: nvm install 18.17.0\n- brew: brew install node@18\n\nGenerate Node.js installation commands instead of pip commands.")
            } else if is_scientific_package(&package) {
                // Scientific package recommendation removed - let users install with pip if they prefer
                String::new()
            } else {
                let pkg_name = extract_package_name(&package);
                match env_type {
                    "conda" => format!("\n\nNOTE: Using conda environment as specified:\n- conda install {pkg_name}"),
                    _ => format!("\n\nNOTE: Using pip (default) for Python packages:\n- pip install {pkg_name}")
                }
            }
        } else if package_type == "npm"
            && (package.starts_with("python==") || package.starts_with("python3=="))
        {
            format!("\n\nWARNING: '{package}' is NOT an npm package. Python must be installed using:\n- pyenv: pyenv install 3.13.3 (RECOMMENDED)\n- macOS: brew install python@3.13\n- conda: conda install python=3.13\n\nGenerate Python installation commands instead of npm commands.")
        } else {
            String::new()
        };

        let package_manager = if package_type == "python" {
            match env_type {
                "conda" => "conda",
                _ => "pip",
            }
        } else {
            "npm"
        };
        format!(
            "Generate the BASIC installation command for {package_type} package '{package}' using {package_manager}. Start with the standard installation command only (e.g., '{package_manager} install {package}'). Do NOT include cache clearing, purging, upgrade pip, or force reinstall commands - these will be used only if the basic installation fails. Provide ONLY the basic executable command.{upfront_detection}"
        )
    };

    // Send query to AI
    match provider.send_query(&system_prompt, &prompt).await {
        Ok(response) => {
            // Extract and execute commands with iterative approach
            if let Err(e) = execute_resolution_commands(
                &response,
                &package_type,
                &package,
                is_file_mode,
                env_type,
                &provider,
                &system_prompt,
            )
            .await
            {
                eprintln!("❌ Error executing resolution commands: {e}");
                std::process::exit(1);
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

/// Check for common invalid packages and provide immediate feedback
fn check_for_common_invalid_packages(package_type: &str, package: &str) -> Option<String> {
    if package_type == "python" {
        if package.starts_with("python==") || package.starts_with("python3==") {
            return Some(format!(
                "Package '{package}' is invalid. Python interpreter versions cannot be installed via pip.\n\n💡 Use these alternatives instead (PYENV PREFERRED):\n• pyenv: pyenv install 3.13.3 && pyenv global 3.13.3 (RECOMMENDED)\n• macOS: brew install python@3.13\n• conda: conda install python=3.13\n• Check your current Python: python --version"
            ));
        } else if package.starts_with("node==") {
            return Some(format!(
                "Package '{package}' is invalid. Node.js cannot be installed via pip.\n\n💡 Use these alternatives instead:\n• nvm: nvm install 18.17.0\n• brew: brew install node@18\n• Download from nodejs.org"
            ));
        } else if let Some(corrected_package) = detect_common_typos(package) {
            return Some(format!(
                "Package '{package}' may be a typo. Did you mean '{corrected_package}'?\n\n💡 If you meant '{corrected_package}', use:\n• pip install {corrected_package} (RECOMMENDED)\n• conda install {corrected_package}"
            ));
        } else if is_scientific_package(package) {
            // Scientific package warning removed - let users install with pip if they prefer
            return None;
        }
    } else if package_type == "npm"
        && (package.starts_with("python==") || package.starts_with("python3=="))
    {
        return Some(format!(
            "Package '{package}' is invalid. Python cannot be installed via npm.\n\n💡 Use these alternatives instead (PYENV PREFERRED):\n• pyenv: pyenv install 3.13.3 (RECOMMENDED)\n• macOS: brew install python@3.13\n• conda: conda install python=3.13"
        ));
    }
    None
}

/// Detect common package name typos and suggest corrections
fn detect_common_typos(package: &str) -> Option<String> {
    let package_name = extract_package_name(package).to_lowercase();
    let version_part = if package.contains("==") {
        package.split("==").nth(1).unwrap_or("")
    } else {
        ""
    };

    let corrected_name = match package_name.as_str() {
        "numby" => Some("numpy"),
        "numpie" => Some("numpy"),
        "numbpy" => Some("numpy"),
        "pandsa" => Some("pandas"),
        "panda" => Some("pandas"),
        "scikitlearn" => Some("scikit-learn"),
        "sklearn" => Some("scikit-learn"),
        "matplot" => Some("matplotlib"),
        "plotlib" => Some("matplotlib"),
        "tensorflow" if package_name == "tensorflow" => None, // Not a typo
        "tensorlow" => Some("tensorflow"),
        "tensrflow" => Some("tensorflow"),
        "requests" if package_name == "requests" => None, // Not a typo
        "reqests" => Some("requests"),
        "reqeusts" => Some("requests"),
        "beautifulsoup" => Some("beautifulsoup4"),
        "bs4" => Some("beautifulsoup4"),
        "pillow" if package_name == "pillow" => None, // Not a typo
        "pil" => Some("pillow"),
        _ => None,
    };

    corrected_name.map(|name| {
        if version_part.is_empty() {
            name.to_string()
        } else {
            format!("{name}=={version_part}")
        }
    })
}

/// Check if a package is a common scientific/ML package that works better with conda
fn is_scientific_package(package: &str) -> bool {
    let package_name = extract_package_name(package).to_lowercase();
    matches!(
        package_name.as_str(),
        "pytorch"
            | "torch"
            | "torchvision"
            | "tensorflow"
            | "tf-nightly"
            | "numpy"
            | "scipy"
            | "pandas"
            | "scikit-learn"
            | "sklearn"
            | "matplotlib"
            | "seaborn"
            | "plotly"
            | "bokeh"
            | "jupyter"
            | "ipython"
            | "notebook"
            | "jupyterlab"
            | "conda"
            | "anaconda"
            | "miniconda"
    )
}

/// Remove all duplicate commands from a list (not just consecutive)
/// Also removes commands that are essentially the same but with different package names (typos)
fn deduplicate_commands(commands: Vec<String>) -> Vec<String> {
    let mut deduplicated = Vec::new();
    let mut seen_commands = std::collections::HashSet::new();
    let mut seen_command_patterns = std::collections::HashSet::new();

    for command in commands {
        let normalized_command = command.trim().to_lowercase();

        // First check for exact duplicates
        if seen_commands.contains(&normalized_command) {
            continue;
        }

        // Then check for similar command patterns (same command structure, different package names)
        let command_pattern = normalize_command_pattern(&normalized_command);
        if seen_command_patterns.contains(&command_pattern) {
            continue;
        }

        seen_commands.insert(normalized_command);
        seen_command_patterns.insert(command_pattern);
        deduplicated.push(command);
    }

    deduplicated
}

/// Normalize command pattern by replacing package names with a placeholder
/// This helps detect commands that are the same except for the package name
fn normalize_command_pattern(command: &str) -> String {
    let mut normalized = command.to_string();

    // Common package manager patterns
    if command.contains("pip install") || command.contains("python -m pip install") {
        // Replace package names with placeholder for pip commands
        if let Some(install_pos) = command.find("install ") {
            let after_install = &command[install_pos + 8..];
            if let Some(package_end) = after_install.find(' ') {
                let package_part = &after_install[..package_end];
                normalized = command.replace(package_part, "PACKAGE");
            } else {
                // Package is at the end of the command
                normalized = command.replace(after_install.trim(), "PACKAGE");
            }
        }
    } else if command.contains("conda install") {
        // Replace package names with placeholder for conda commands
        if let Some(install_pos) = command.find("install ") {
            let after_install = &command[install_pos + 8..];
            if let Some(package_end) = after_install.find(' ') {
                let package_part = &after_install[..package_end];
                normalized = command.replace(package_part, "PACKAGE");
            } else {
                // Package is at the end of the command
                normalized = command.replace(after_install.trim(), "PACKAGE");
            }
        }
    } else if command.contains("npm install") {
        // Replace package names with placeholder for npm commands
        if let Some(install_pos) = command.find("install ") {
            let after_install = &command[install_pos + 8..];
            if let Some(package_end) = after_install.find(' ') {
                let package_part = &after_install[..package_end];
                normalized = command.replace(package_part, "PACKAGE");
            } else {
                // Package is at the end of the command
                normalized = command.replace(after_install.trim(), "PACKAGE");
            }
        }
    }

    normalized
}

/// Detect package manager type from dependency file
fn detect_package_manager_from_file(file_path: &str) -> Result<String> {
    let path = Path::new(file_path);

    // Check if file exists
    if !path.exists() {
        return Err(anyhow::anyhow!(
            "Dependency file '{}' does not exist",
            file_path
        ));
    }

    // Get file name and extension
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?;

    let file_name_lower = file_name.to_lowercase();

    // Detect package manager based on file name
    match file_name_lower.as_str() {
        "package.json" | "package-lock.json" | "yarn.lock" => Ok("npm".to_string()),
        "requirements.txt" | "poetry.lock" | "pipfile" | "pipfile.lock" => Ok("python".to_string()),
        _ => {
            // Try to read file content for better detection
            let content = std::fs::read_to_string(path)
                .map_err(|_| anyhow::anyhow!("Could not read file '{}'", file_path))?;

            if content.contains("\"dependencies\"") || content.contains("\"devDependencies\"") {
                Ok("npm".to_string())
            } else if content.contains("==") || content.contains(">=") || content.contains("<=") {
                Ok("python".to_string())
            } else {
                Err(anyhow::anyhow!(
                    "Could not detect package manager type from file '{}'. Supported files: package.json, requirements.txt, yarn.lock, poetry.lock, Pipfile",
                    file_path
                ))
            }
        }
    }
}

/// Execute resolution commands with iterative approach
async fn execute_resolution_commands(
    ai_response: &str,
    package_type: &str,
    package: &str,
    is_file_mode: bool,
    env_type: &str,
    provider: &QueryProvider,
    system_prompt: &str,
) -> Result<()> {
    let mut commands_to_execute =
        deduplicate_commands(terminalai::extract_commands_from_response(ai_response));
    let mut attempt_count = 0;
    const MAX_ATTEMPTS: u32 = 15; // Increased for more iterative attempts
    let mut error_history = Vec::new();

    if commands_to_execute.is_empty() {
        println!("⚠️  No executable commands found in AI response.");
        println!("💡 AI Response:");
        println!("{ai_response}");
        return Ok(());
    }

    // Show initial commands to user and ask for confirmation
    println!("Terminal AI suggest following commands:");
    for (i, cmd) in commands_to_execute.iter().enumerate() {
        println!("  {}. {}", i + 1, cmd);
    }

    print!("\n❓ Execute these resolution commands? [Y/n]: ");
    std::io::Write::flush(&mut std::io::stdout()).unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    if input.trim().to_lowercase() == "n" || input.trim().to_lowercase() == "no" {
        println!("❌ Resolution commands not executed.");
        return Ok(());
    }

    // Execute commands with iterative error handling
    while !commands_to_execute.is_empty() && attempt_count < MAX_ATTEMPTS {
        attempt_count += 1;
        println!(
            "\n🔄 Attempt {}: Executing {} commands",
            attempt_count,
            commands_to_execute.len()
        );

        let mut new_commands = Vec::new();
        let mut has_failures = false;

        for (cmd_index, cmd) in commands_to_execute.iter().enumerate() {
            println!("\n📋 Command {}: {}", cmd_index + 1, cmd);

            // Execute the command
            let output = execute_single_command(cmd)?;

            // Check if the command was successful
            if output.status.success() {
                println!("✅ Command completed successfully");
                if !output.stdout.is_empty() {
                    println!("Output: {}", String::from_utf8_lossy(&output.stdout));
                }

                // If this was an installation command and it succeeded, verify the installation
                if is_installation_command(cmd, package_type, package, is_file_mode) {
                    if verify_package_installation(package_type, package, is_file_mode, env_type)? {
                        if is_file_mode {
                            println!(
                                "🎉 Dependencies from '{package}' successfully installed and verified!"
                            );
                        } else {
                            println!("🎉 Package '{package}' successfully installed and verified!");
                        }
                        return Ok(());
                    } else {
                        println!("⚠️  Installation command succeeded but verification failed");
                        has_failures = true;
                    }
                }
            } else {
                has_failures = true;
                let exit_code = output.status.code().unwrap_or(-1);
                let stderr_output = String::from_utf8_lossy(&output.stderr);

                println!("❌ Command failed with exit code: {exit_code}");
                if !stderr_output.is_empty() {
                    println!("Error: {stderr_output}");
                }

                // Store error information for AI analysis
                error_history.push(format!(
                    "Command '{cmd}' failed with exit code {exit_code}: {stderr_output}"
                ));

                // If this is an installation command that failed, try to get new resolution commands from AI
                if is_installation_command(cmd, package_type, package, is_file_mode) {
                    println!("🤖 Analyzing error and requesting new resolution steps...");

                    match request_error_resolution(
                        package_type,
                        package,
                        is_file_mode,
                        env_type,
                        &error_history,
                        provider,
                        system_prompt,
                    )
                    .await
                    {
                        Ok(additional_commands) => {
                            let deduplicated_additional = deduplicate_commands(additional_commands);
                            if !deduplicated_additional.is_empty() {
                                println!(
                                    "🆕 AI generated {} new resolution commands:",
                                    deduplicated_additional.len()
                                );
                                for (i, new_cmd) in deduplicated_additional.iter().enumerate() {
                                    println!("  {}. {}", i + 1, new_cmd);
                                }

                                // Ask user for confirmation of new commands
                                print!("\n❓ Execute these new resolution commands? [Y/n]: ");
                                std::io::Write::flush(&mut std::io::stdout()).unwrap();

                                let mut confirm_input = String::new();
                                std::io::stdin().read_line(&mut confirm_input).unwrap();

                                if confirm_input.trim().to_lowercase() == "n"
                                    || confirm_input.trim().to_lowercase() == "no"
                                {
                                    println!("❌ New resolution commands not executed.");
                                } else {
                                    new_commands.extend(deduplicated_additional);
                                }
                            }
                        }
                        Err(e) => {
                            println!("⚠️  Failed to get new resolution commands from AI: {e}");
                        }
                    }
                }
            }
        }

        // Update commands for next iteration
        commands_to_execute = new_commands;

        // If no failures occurred and no new commands were generated, we're done
        if !has_failures && commands_to_execute.is_empty() {
            break;
        }

        // If we've reached max attempts and still have failures
        if attempt_count >= MAX_ATTEMPTS && has_failures {
            println!(
                "🛑 Maximum resolution attempts ({MAX_ATTEMPTS}) reached. Installation failed."
            );
            println!("📋 Error history:");
            for (i, error) in error_history.iter().enumerate() {
                println!("  {}. {}", i + 1, error);
            }
            return Err(anyhow::anyhow!(
                "Failed to install {} after {} attempts",
                if is_file_mode {
                    format!("dependencies from '{package}'")
                } else {
                    format!("package '{package}'")
                },
                MAX_ATTEMPTS
            ));
        }
    }

    println!("✅ All resolution commands completed");
    Ok(())
}

/// Request new resolution commands from AI based on error history
async fn request_error_resolution(
    package_type: &str,
    package: &str,
    is_file_mode: bool,
    env_type: &str,
    error_history: &[String],
    provider: &QueryProvider,
    system_prompt: &str,
) -> Result<Vec<String>> {
    let error_summary = error_history.join("\n");

    // Check if this is an invalid package name error (for error analysis context)
    let _has_invalid_package_error = error_history.iter().any(|error| {
        error.contains("No matching distribution found")
            || error.contains("Could not find a version that satisfies")
    });

    // Detect common invalid package patterns
    let invalid_package_suggestions = if package_type == "python" {
        if package.starts_with("python==") || package.starts_with("python3==") {
            "\n\nDETECTED INVALID PACKAGE: Python interpreter versions cannot be installed via pip. Use system package managers instead:\n- conda: conda install python=3.13 (RECOMMENDED)\n- pyenv: pyenv install 3.13.3 && pyenv global 3.13.3\n- macOS: brew install python@3.13".to_string()
        } else if package.starts_with("node==") {
            "\n\nDETECTED INVALID PACKAGE: Node.js cannot be installed via pip. Use:\n- nvm: nvm install 18.17.0\n- brew: brew install node@18\n- Download from nodejs.org".to_string()
        } else if is_scientific_package(package) {
            // Respect user's environment choice
            let pkg_name = extract_package_name(package);
            match env_type {
                "conda" => format!("\n\nSUGGESTION: Try conda alternatives:\n- conda install {pkg_name}\n- conda install -c conda-forge {pkg_name}"),
                _ => format!("\n\nSUGGESTION: Try pip alternatives:\n- pip install {pkg_name}\n- pip install --no-cache-dir {pkg_name}")
            }
        } else {
            let pkg_name = extract_package_name(package);
            match env_type {
                "conda" => format!("\n\nSUGGESTION: Try conda alternatives:\n- conda install {pkg_name}"),
                _ => format!("\n\nSUGGESTION: Try pip alternatives:\n- pip install {pkg_name}\n- pip install --no-cache-dir {pkg_name}")
            }
        }
    } else if package_type == "npm" {
        if package.starts_with("python==") || package.starts_with("python3==") {
            "\n\nDETECTED INVALID PACKAGE: Python cannot be installed via npm. Use:\n- conda: conda install python=3.13 (RECOMMENDED)\n- pyenv: pyenv install 3.13.3\n- macOS: brew install python@3.13".to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let prompt = if is_file_mode {
        let package_manager = if package_type == "python" {
            match env_type {
                "conda" => "conda",
                _ => "pip",
            }
        } else {
            "npm"
        };
        format!(
            "The following errors occurred while trying to install dependencies from '{package}' ({package_type}) using {package_manager}:\n\n{error_summary}\n\nAnalyze these errors and provide ONLY executable {package_manager} commands to fix the issues. Focus on:\n1. Version conflicts - suggest removing conflicting packages before installing\n2. Invalid package names - if 'No matching distribution found', suggest correct alternatives\n3. Missing system dependencies (headers, libraries, compilers)\n4. Package manager configuration issues\n5. Build environment problems\n\nFor version conflicts, ALWAYS suggest uninstalling conflicting packages first.\nProvide ONLY {package_manager} executable commands, one per line, NO explanations. Do NOT suggest alternative package managers.{invalid_package_suggestions}"
        )
    } else {
        let env_note = if package_type == "python" {
            match env_type {
                "conda" => "\nUsing conda environment as specified by user.",
                _ => "\nUsing pip environment as specified by user (default).",
            }
        } else {
            ""
        };

        let package_manager = if package_type == "python" {
            match env_type {
                "conda" => "conda",
                _ => "pip",
            }
        } else {
            "npm"
        };
        format!(
            "The following errors occurred while trying to install package '{package}' ({package_type}) using {package_manager}:\n\n{error_summary}\n\nAnalyze these errors and provide ONLY executable {package_manager} commands to fix the issues. Focus on:\n1. Version conflicts - suggest removing conflicting packages before installing\n2. Invalid package names - if 'No matching distribution found', suggest correct alternatives  \n3. Missing system dependencies (headers, libraries, compilers)\n4. Package manager configuration issues\n5. Build environment problems\n\nFor version conflicts, ALWAYS suggest uninstalling conflicting packages first.\nFor invalid packages like 'python==X.X.X', suggest system installation methods instead.{env_note}\nProvide ONLY {package_manager} executable commands, one per line, NO explanations. Do NOT suggest alternative package managers.{invalid_package_suggestions}"
        )
    };

    match provider.send_query(system_prompt, &prompt).await {
        Ok(response) => {
            let new_commands = terminalai::extract_commands_from_response(&response);
            Ok(new_commands)
        }
        Err(e) => Err(anyhow::anyhow!(
            "Failed to get error resolution from AI: {e}"
        )),
    }
}

/// Execute a single command with live output and return the output
fn execute_single_command(cmd: &str) -> Result<std::process::Output> {
    let is_install_cmd = terminalai::is_install_update_remove_command(cmd);

    if is_install_cmd {
        println!(
            "{}",
            "[Terminal AI] - Executing package management command"
                .green()
                .bold()
        );
        println!("{}", format!("[Terminal AI] - Command: {cmd}").green());
        println!("{}", "[Terminal AI] - Live output:".green());
    }

    // For error analysis, we need to capture stderr while still showing live output
    // We'll use a hybrid approach: capture stderr for analysis, but show stdout live
    let mut command = StdCommand::new("sh");
    command.arg("-c");
    command.arg(cmd);
    command.stdin(std::process::Stdio::piped());
    command.stdout(std::process::Stdio::inherit());
    command.stderr(std::process::Stdio::piped()); // Capture stderr for error analysis

    let output = command
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute command '{cmd}': {e}"))?;

    // Print stderr output for user visibility (since we captured it)
    if !output.stderr.is_empty() {
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        eprint!("{stderr_str}");
    }

    if is_install_cmd {
        if output.status.success() {
            println!(
                "{}",
                "[Terminal AI] - Command completed successfully"
                    .green()
                    .bold()
            );
        } else {
            let exit_code = output.status.code().unwrap_or(-1);
            eprintln!(
                "{}",
                format!("[Terminal AI] - Command failed with exit code: {exit_code}")
                    .red()
                    .bold()
            );
        }
    }

    Ok(output)
}

/// Check if a command is an installation command for the target package
fn is_installation_command(
    cmd: &str,
    package_type: &str,
    package: &str,
    is_file_mode: bool,
) -> bool {
    let cmd_lower = cmd.to_lowercase();

    if is_file_mode {
        // For file mode, check for dependency file installation commands
        match package_type {
            "npm" => {
                cmd_lower.contains("npm install")
                    && (cmd_lower.contains("package.json")
                        || cmd_lower.contains("yarn.lock")
                        || !cmd_lower.contains(" "))
            }
            "python" => {
                (cmd_lower.contains("pip install") || cmd_lower.contains("python -m pip install"))
                    && (cmd_lower.contains("requirements.txt")
                        || cmd_lower.contains("poetry.lock")
                        || cmd_lower.contains("pipfile"))
            }
            _ => false,
        }
    } else {
        // For single package mode, check for specific package installation
        let package_name = extract_package_name(package);

        match package_type {
            "npm" => {
                cmd_lower.contains("npm install")
                    && (cmd_lower.contains(&package_name) || cmd_lower.contains("package.json"))
            }
            "python" => {
                (cmd_lower.contains("pip install") || cmd_lower.contains("python -m pip install"))
                    && (cmd_lower.contains(&package_name) || cmd_lower.contains("requirements.txt"))
            }
            _ => false,
        }
    }
}

/// Extract package name from package specification (e.g., "react@18.2.0" -> "react")
fn extract_package_name(package: &str) -> String {
    package
        .split(['@', '='])
        .next()
        .unwrap_or(package)
        .to_string()
}

/// Verify that the package was successfully installed
fn verify_package_installation(
    package_type: &str,
    package: &str,
    is_file_mode: bool,
    env_type: &str,
) -> Result<bool> {
    if is_file_mode {
        // For file mode, verify that dependencies are installed
        let verification_cmd = match package_type {
            "npm" => "npm list".to_string(),
            "python" => match env_type {
                "conda" => "conda list".to_string(),
                _ => "pip list".to_string(),
            },
            _ => return Ok(false),
        };

        println!("🔍 Verifying dependencies installation: {verification_cmd}");

        let output = execute_single_command(&verification_cmd)?;

        if output.status.success() {
            println!("✅ Dependencies verification successful");
            if !output.stdout.is_empty() {
                println!(
                    "Installed packages: {}",
                    String::from_utf8_lossy(&output.stdout)
                );
            }
            Ok(true)
        } else {
            println!("❌ Dependencies verification failed");
            if !output.stderr.is_empty() {
                println!("Error: {}", String::from_utf8_lossy(&output.stderr));
            }
            Ok(false)
        }
    } else {
        // For single package mode, verify specific package
        let package_name = extract_package_name(package);

        let verification_cmd = match package_type {
            "npm" => format!("npm list {package_name}"),
            "python" => match env_type {
                "conda" => format!("conda list {package_name}"),
                _ => format!("pip show {package_name}"),
            },
            _ => return Ok(false),
        };

        println!("🔍 Verifying installation: {verification_cmd}");

        let output = execute_single_command(&verification_cmd)?;

        if output.status.success() {
            println!("✅ Package verification successful");
            if !output.stdout.is_empty() {
                println!("Package info: {}", String::from_utf8_lossy(&output.stdout));
            }
            Ok(true)
        } else {
            println!("❌ Package verification failed");
            if !output.stderr.is_empty() {
                println!("Error: {}", String::from_utf8_lossy(&output.stderr));
            }
            Ok(false)
        }
    }
}
