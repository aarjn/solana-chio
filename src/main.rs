use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use console::Style;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use toml::Value;

use chio::content::templates;
use chio::is_valid_project_name;

#[derive(Copy, Clone, Eq, PartialEq, Debug, ValueEnum)]
enum TestFramework {
    Mollusk,
    Litesvm,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, ValueEnum)]
enum KeyAction {
    Sync,
    Generate,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        project_name: String,
        #[arg(long, value_enum, default_value_t = TestFramework::Mollusk)]
        test_framework: TestFramework,
    },
    Build,
    Test,
    Deploy,
    Keys {
        action: KeyAction,
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    Client {
        #[arg(long, default_value_t = false)]
        idl_only: bool,
    },
}

const MAX_LOG_LINES: usize = 6;

fn colorize_cargo_line(line: &str) -> String {
    let green_bold = Style::new().green().bold();
    let keywords = [
        "Compiling",
        "Linking",
        "Finished",
        "Running",
        "Downloading",
        "Downloaded",
    ];
    for keyword in &keywords {
        if let Some(rest) = line.strip_prefix(keyword) {
            return format!("{}{}", green_bold.apply_to(keyword), rest);
        }
    }
    line.to_string()
}

struct CommandResult {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr_lines: Vec<String>,
}

fn run_with_spinner(command: &str, args: &[&str], message: &str) -> Result<CommandResult> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["◰", "◳", "◲", "◱", "◰"])
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(100));

    let mut child = Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to run {} {}", command, args.join(" ")))?;

    let stderr = child.stderr.take().unwrap();
    let recent_lines: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let all_lines: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let recent_clone = Arc::clone(&recent_lines);
    let all_clone = Arc::clone(&all_lines);
    let spinner_clone = spinner.clone();
    let base_message = message.to_string();

    let reader_thread = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            let Ok(line) = line else { break };
            all_clone.lock().unwrap().push(line.clone());

            let trimmed = line.trim();
            if trimmed.starts_with("Compiling")
                || trimmed.starts_with("Linking")
                || trimmed.starts_with("Finished")
            {
                let mut recent = recent_clone.lock().unwrap();
                recent.push(line.clone());
                if recent.len() > MAX_LOG_LINES {
                    recent.remove(0);
                }
                let log_display = recent
                    .iter()
                    .map(|l| {
                        let t = l.trim();
                        let colored = colorize_cargo_line(t);
                        format!("\n  {}", colored)
                    })
                    .collect::<String>();
                spinner_clone.set_message(format!("{}{}", base_message, log_display));
            }
        }
    });

    let stdout = {
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut child.stdout.take().unwrap(), &mut buf)?;
        buf
    };

    let status = child.wait()?;
    reader_thread.join().unwrap();
    spinner.finish_and_clear();

    let stderr_lines = Arc::try_unwrap(all_lines).unwrap().into_inner().unwrap();

    Ok(CommandResult {
        status,
        stdout,
        stderr_lines,
    })
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init {
            project_name,
            test_framework,
        } => {
            init_project(project_name, *test_framework)?;
        }
        Commands::Build => {
            let result = run_with_spinner("cargo", &["build-sbf"], "Building program...")?;
            if result.status.success() {
                println!("Build completed successfully!");
            } else {
                for line in &result.stderr_lines {
                    eprintln!("{}", line);
                }
                anyhow::bail!("Build failed with exit code: {:?}", result.status.code());
            }
        }
        Commands::Test => {
            let build_result = run_with_spinner("cargo", &["build-sbf"], "Building program...")?;
            if !build_result.status.success() {
                for line in &build_result.stderr_lines {
                    eprintln!("{}", line);
                }
                anyhow::bail!(
                    "Build failed with exit code: {:?}",
                    build_result.status.code()
                );
            }
            println!("Build completed successfully!");

            let test_result = run_with_spinner("cargo", &["test"], "Running tests...")?;
            if test_result.status.success() {
                let stdout = String::from_utf8_lossy(&test_result.stdout);
                if !stdout.is_empty() {
                    println!("{}", stdout);
                }
                println!("All tests passed!");
            } else {
                let stdout = String::from_utf8_lossy(&test_result.stdout);
                if !stdout.is_empty() {
                    eprintln!("{}", stdout);
                }
                for line in &test_result.stderr_lines {
                    eprintln!("{}", line);
                }
                anyhow::bail!(
                    "Tests failed with exit code: {:?}",
                    test_result.status.code()
                );
            }
        }
        Commands::Deploy => {
            println!("Deploying program");

            let target_deploy_dir = Path::new("target/deploy");
            if !target_deploy_dir.exists() {
                anyhow::bail!("target/deploy directory not found. Please run 'chio build' first.");
            }

            let mut so_file = None;
            for entry in fs::read_dir(target_deploy_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("so") {
                    so_file = Some(path);
                    break;
                }
            }

            let so_path = so_file.ok_or_else(|| {
                anyhow::anyhow!(
                    "No .so file found in target/deploy. Please run 'chio build' first."
                )
            })?;

            let status = Command::new("solana")
                .arg("program")
                .arg("deploy")
                .arg(&so_path)
                .spawn()?
                .wait()
                .with_context(|| "Failed to deploy program")?;

            if !status.success() {
                anyhow::bail!("Deploy failed with exit code: {:?}", status.code());
            } else {
                println!("Program deployed successfully!");
            }
        }
        Commands::Keys { action, force } => {
            handle_keys_action(action, *force)?;
        }
        Commands::Client { idl_only } => {
            generate_client(*idl_only)?;
        }
    }

    Ok(())
}

fn command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

fn generate_client(idl_only: bool) -> Result<()> {
    let green = Style::new().green().bold();
    let dim = Style::new().dim();

    // Preflight: make sure we're inside a chio/pinocchio project.
    if !Path::new("src/entrypoint.rs").exists() {
        anyhow::bail!(
            "src/entrypoint.rs not found. Run this inside a chio project (see 'chio init')."
        );
    }

    // 1. Extract the IDL from the program source with shank (required).
    if !command_exists("shank") {
        anyhow::bail!("`shank` not found. Install it with:\n  cargo install shank-cli");
    }

    let idl_result = run_with_spinner(
        "shank",
        &["idl", "-o", "idl", "-r", "."],
        "Generating IDL...",
    )?;
    if !idl_result.status.success() {
        for line in &idl_result.stderr_lines {
            eprintln!("{}", line);
        }
        anyhow::bail!(
            "shank failed with exit code: {:?}",
            idl_result.status.code()
        );
    }

    let project_name = project_name_from_cargo_toml(Path::new("."))?;
    let idl_path = format!("idl/{}.json", project_name);
    if !Path::new(&idl_path).exists() {
        anyhow::bail!("Expected IDL at {} but shank did not produce it.", idl_path);
    }
    println!("  {} IDL written to {}", green.apply_to("✓"), idl_path);

    if idl_only {
        return Ok(());
    }

    // 2. Render the TypeScript client from the IDL with codama.
    let codama_cfg = Path::new("codama.json");
    if !codama_cfg.exists() {
        fs::write(codama_cfg, templates::codama_json(&project_name))?;
        println!("  {} codama.json", green.apply_to("✓"));
    }

    let package_json = Path::new("package.json");
    if !package_json.exists() {
        fs::write(package_json, templates::package_json(&project_name))?;
        println!("  {} package.json", green.apply_to("✓"));
    }

    if !command_exists("bun") {
        println!(
            "\n  {}",
            dim.apply_to("bun not found. Install bun (https://bun.sh), then run:")
        );
        println!("  $ bun install");
        println!("  $ bunx codama run js");
        return Ok(());
    }

    let install = run_with_spinner("bun", &["install"], "Installing client dependencies...")?;
    if !install.status.success() {
        for line in &install.stderr_lines {
            eprintln!("{}", line);
        }
        anyhow::bail!(
            "bun install failed with exit code: {:?}",
            install.status.code()
        );
    }

    let render = run_with_spinner(
        "bunx",
        &["codama", "run", "js"],
        "Generating TypeScript client...",
    )?;
    if !render.status.success() {
        for line in &render.stderr_lines {
            eprintln!("{}", line);
        }
        anyhow::bail!("codama failed with exit code: {:?}", render.status.code());
    }

    println!("\n  {} TypeScript client generated\n", green.apply_to("✓"));
    println!("  {} clients/js/src/generated", dim.apply_to("output"));

    Ok(())
}

fn init_project(project_name: &str, test_framework: TestFramework) -> Result<()> {
    // Validate project name - only allow alphanumeric characters and underscores
    if !is_valid_project_name(project_name) {
        anyhow::bail!(
            "Invalid project name '{}'. Project names can only contain letters, numbers, and underscores (_). \
            Hyphens (-) and other special characters are not allowed.",
            project_name
        );
    }

    let green = Style::new().green().bold();
    let dim = Style::new().dim();

    println!(
        r#"
  {}
  {}
  {}
  {}
  {}
  {}
"#,
        green.apply_to("██████╗ ██╗  ██╗██╗ ██████╗"),
        green.apply_to("██╔════╝██║  ██║██║██╔═══██╗"),
        green.apply_to("██║     ███████║██║██║   ██║"),
        green.apply_to("██║     ██╔══██║██║██║   ██║"),
        green.apply_to("╚██████╗██║  ██║██║╚██████╔╝"),
        green.apply_to("╚═════╝╚═╝  ╚═╝╚═╝ ╚═════╝"),
    );
    println!(
        "  {}",
        dim.apply_to(format!("initializing {}...", project_name))
    );

    let project_dir = Path::new(project_name);
    fs::create_dir_all(project_dir)
        .with_context(|| format!("Failed to create project directory: {}", project_name))?;

    let output = Command::new("cargo")
        .arg("init")
        .arg("--lib")
        .arg("--name")
        .arg(project_name)
        .current_dir(project_dir)
        .output()
        .with_context(|| "Failed to run 'cargo init'")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to initialize Cargo project: {}", error);
    }

    let deploy_dir = project_dir.join("target").join("deploy");
    fs::create_dir_all(&deploy_dir)?;

    let keypair_path = format!("./target/deploy/{}-keypair.json", project_name);
    let keygen_output = Command::new("solana-keygen")
        .arg("new")
        .arg("-o")
        .arg(&keypair_path)
        .arg("--no-bip39-passphrase")
        .current_dir(project_dir)
        .output()
        .with_context(|| "Failed to generate keypair")?;

    if !keygen_output.status.success() {
        let error = String::from_utf8_lossy(&keygen_output.stderr);
        anyhow::bail!("Failed to generate keypair: {}", error);
    }

    let address_output = Command::new("solana")
        .arg("address")
        .arg("-k")
        .arg(&keypair_path)
        .current_dir(project_dir)
        .output()
        .with_context(|| "Failed to read keypair address")?;

    let program_address: String = if address_output.status.success() {
        String::from_utf8_lossy(&address_output.stdout)
            .trim()
            .to_string()
    } else {
        let error = String::from_utf8_lossy(&address_output.stderr);
        anyhow::bail!("Failed to get program address from keypair: {}", error);
    };

    let user_address_output = Command::new("solana")
        .arg("address")
        .current_dir(project_dir)
        .output()
        .with_context(|| "Failed to get user address")?;

    let user_address = if user_address_output.status.success() {
        String::from_utf8_lossy(&user_address_output.stdout)
            .trim()
            .to_string()
    } else {
        String::new()
    };

    create_project_structure(
        project_dir,
        user_address,
        program_address.clone(),
        test_framework,
    )?;
    update_cargo_toml(project_dir, project_name, test_framework)?;

    init_git_repo(project_dir, project_name)?;

    println!(
        "\n  {} Project '{}' ready\n",
        green.apply_to("✓"),
        project_name
    );
    println!("  {} {}", dim.apply_to("program"), program_address);
    println!("\n  {}", dim.apply_to("next steps:"));
    println!("  $ cd {}", project_name);
    println!("  $ chio build");
    println!("  $ chio test");
    println!("  $ chio deploy");
    println!();

    Ok(())
}

fn init_git_repo(project_dir: &Path, project_name: &str) -> Result<()> {
    let git_init_output = Command::new("git")
        .arg("init")
        .current_dir(project_dir)
        .output()
        .with_context(|| "Failed to initialize git repository")?;

    if !git_init_output.status.success() {
        let error = String::from_utf8_lossy(&git_init_output.stderr);
        println!("Warning: Failed to initialize git repository: {}", error);
        return Ok(());
    }

    let git_add_output = Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(project_dir)
        .output()
        .with_context(|| "Failed to add files to git")?;

    if !git_add_output.status.success() {
        let error = String::from_utf8_lossy(&git_add_output.stderr);
        println!("Warning: Failed to add files to git: {}", error);
        return Ok(());
    }

    let commit_message = format!("Initial commit: Setup Pinocchio project '{}'", project_name);
    let git_commit_output = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(&commit_message)
        .current_dir(project_dir)
        .output()
        .with_context(|| "Failed to make initial commit")?;

    if !git_commit_output.status.success() {
        let error = String::from_utf8_lossy(&git_commit_output.stderr);
        println!("Warning: Failed to make initial commit: {}", error);
        // Check if it's because of missing git config
        if error.contains("user.email") || error.contains("user.name") {
            println!("Hint: Set your git config with:");
            println!("  git config --global user.email \"you@example.com\"");
            println!("  git config --global user.name \"Your Name\"");
        }
        return Ok(());
    }
    Ok(())
}

fn create_project_structure(
    project_dir: &Path,
    user_address: String,
    program_address: String,
    test_framework: TestFramework,
) -> Result<()> {
    fs::write(project_dir.join("README.md"), templates::readme_md())?;
    fs::write(project_dir.join(".gitignore"), templates::gitignore())?;

    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    fs::write(src_dir.join("lib.rs"), templates::lib_rs())?;

    let test_dir = project_dir.join("tests");
    fs::create_dir_all(&test_dir)?;

    let project_name = project_dir
        .file_name()
        .and_then(|name| name.to_str())
        .context("Failed to determine project name from directory path")?;

    let test_content = match test_framework {
        TestFramework::Mollusk => {
            templates::unit_tests::unit_test_rs(&user_address, &program_address, project_name)
        }
        TestFramework::Litesvm => templates::unit_tests::litesvm_initialize_rs(project_name),
    };

    fs::write(test_dir.join("tests.rs"), test_content)?;

    fs::write(
        src_dir.join("entrypoint.rs"),
        templates::entrypoint_rs(&program_address),
    )?;

    fs::write(src_dir.join("errors.rs"), templates::errors_rs())?;

    let instructions_dir = src_dir.join("instructions");
    fs::create_dir_all(&instructions_dir)?;

    fs::write(
        instructions_dir.join("mod.rs"),
        templates::instructions::instructions_mod_rs(),
    )?;
    fs::write(
        instructions_dir.join("initialize.rs"),
        templates::instructions::initialize(),
    )?;

    let states_dir = src_dir.join("states");
    fs::create_dir_all(&states_dir)?;

    fs::write(
        states_dir.join("mod.rs"),
        templates::states::states_mod_rs(),
    )?;
    fs::write(states_dir.join("utils.rs"), templates::states::utils_rs())?;

    fs::write(states_dir.join("state.rs"), templates::states::state_rs())?;

    // tests already handled per framework above

    Ok(())
}

fn update_cargo_toml(
    project_dir: &Path,
    project_name: &str,
    test_framework: TestFramework,
) -> Result<()> {
    let dev_deps = match test_framework {
        TestFramework::Mollusk => {
            r#"
[dev-dependencies]
solana-sdk = "3.0.0"
mollusk-svm = "0.9.0"
mollusk-svm-bencher = "0.9.0"
"#
        }
        TestFramework::Litesvm => {
            r#"
[dev-dependencies]
solana-sdk = "3.0.0"
litesvm = "0.9.1"
litesvm-token = "0.9.1"
"#
        }
    };

    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
pinocchio = "0.10.2"
pinocchio-log = "0.5.1"
pinocchio-system = "0.5.0"
shank = "0.4.8"

{dev_deps}

[features]
no-entrypoint = []
std = []
test-default = ["no-entrypoint", "std"]
"#,
        project_name,
        dev_deps = dev_deps
    );

    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    Ok(())
}

fn project_name_from_cargo_toml(project_dir: &Path) -> Result<String> {
    let cargo_toml_path = project_dir.join("Cargo.toml");
    let cargo_toml = fs::read_to_string(&cargo_toml_path)
        .with_context(|| format!("Failed to read {}", cargo_toml_path.display()))?;

    let parsed: Value = toml::from_str(&cargo_toml)
        .with_context(|| format!("Failed to parse {}", cargo_toml_path.display()))?;

    parsed
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Could not find [package].name in {}",
                cargo_toml_path.display()
            )
        })
}

fn expected_keypair_path(project_dir: &Path) -> Result<PathBuf> {
    let project_name = project_name_from_cargo_toml(project_dir)?;
    Ok(project_dir
        .join("target")
        .join("deploy")
        .join(format!("{}-keypair.json", project_name)))
}

fn generate_and_update_keys(
    lib_path: &Path,
    kp_path: &PathBuf,
    content: &str,
    re: &Regex,
    force: bool,
) -> Result<String> {
    // 1. Prevent accidental overwrites
    if kp_path.exists() && !force {
        anyhow::bail!(
            "Kepair already exists at {:?}. Use --force flag to overwrite",
            kp_path
        )
    }

    // 2. Generate new keypair
    let status = Command::new("solana-keygen")
        .arg("new")
        .arg("-o")
        .arg(kp_path)
        .arg("--force")
        .arg("--no-bip39-passphrase")
        .status()
        .with_context(|| "Failed to execute solana-keygen")?;

    if !status.success() {
        anyhow::bail!("solana-keygen failed to generate a new key.");
    }

    // 3. Get the new address
    let output = Command::new("solana")
        .arg("address")
        .arg("-k")
        .arg(kp_path)
        .output()?;

    let new_address = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // 4. Update the file content
    update_declared_id(lib_path, content, re, &new_address)?;

    Ok(new_address)
}

fn handle_keys_action(action: &KeyAction, force: bool) -> Result<()> {
    let project_dir = Path::new(".");
    let target_deploy_dir = Path::new("target/deploy");
    if !target_deploy_dir.exists() {
        fs::create_dir_all(target_deploy_dir)?;
    }

    let entrypoint_path = Path::new("src/entrypoint.rs");
    if !entrypoint_path.exists() {
        anyhow::bail!("src/entrypoint.rs not found. Please run 'chio init' first.");
    }

    let content =
        fs::read_to_string(entrypoint_path).with_context(|| "Failed to read src/entrypoint.rs")?;

    // Use * instead of + to allow empty strings like declare_id!("")
    let re = Regex::new(r#"declare_id!\s*\(\s*"([^"]*)"\s*\)"#).unwrap();

    // Check if the macro exists at all. Error if missing, proceed if empty.
    let captures = re.captures(&content)
        .ok_or_else(|| anyhow::anyhow!("The 'declare_id!' macro was not found in src/entrypoint.rs. It must be present even if empty."))?;

    let declared_program_address = captures
        .get(1)
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();

    let keypair_path = expected_keypair_path(project_dir)?;

    match action {
        KeyAction::Sync => {
            if !keypair_path.exists() {
                anyhow::bail!(
                    "Keypair not found at {}. Run 'chio keys generate' to create it.",
                    keypair_path.display()
                );
            }

            let output = Command::new("solana")
                .arg("address")
                .arg("-k")
                .arg(&keypair_path)
                .output()?;

            let current_keypair_address = if output.status.success() {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                anyhow::bail!("Failed to read address from existing keypair file");
            };

            if !current_keypair_address.is_empty()
                && declared_program_address == current_keypair_address
            {
                println!("Keys are already synced: {}", current_keypair_address);
            } else {
                println!("⚠️ Keys mismatch. Syncing keypair with declared...");

                // Reuse logic to generate and update
                update_declared_id(entrypoint_path, &content, &re, &current_keypair_address)?;
                println!("Keypair synced: {}", current_keypair_address);
            }
        }
        KeyAction::Generate => {
            println!("Generating a fresh keypair...");
            let new_address =
                generate_and_update_keys(entrypoint_path, &keypair_path, &content, &re, force)?;
            println!(
                "✅ Generated and updated src/entrypoint.rs with: {}",
                new_address
            );
        }
    }

    Ok(())
}

fn update_declared_id(lib_path: &Path, content: &str, re: &Regex, new_address: &str) -> Result<()> {
    let new_content = re.replace(content, format!(r#"declare_id!("{}")"#, new_address));
    fs::write(lib_path, new_content.to_string())
        .with_context(|| "Failed to write ID to src/entrypoint.rs")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{expected_keypair_path, project_name_from_cargo_toml};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn reads_project_name_from_cargo_toml() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"hello_world\"\nversion = \"0.1.0\"\n",
        )
        .expect("Failed to write Cargo.toml");

        let project_name =
            project_name_from_cargo_toml(temp_dir.path()).expect("Failed to read project name");

        assert_eq!(project_name, "hello_world");
    }

    #[test]
    fn builds_expected_keypair_path_from_project_name() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"hello_world\"\nversion = \"0.1.0\"\n",
        )
        .expect("Failed to write Cargo.toml");

        let keypair_path =
            expected_keypair_path(temp_dir.path()).expect("Failed to build keypair path");

        assert_eq!(
            keypair_path,
            temp_dir
                .path()
                .join("target")
                .join("deploy")
                .join("hello_world-keypair.json")
        );
    }
}
