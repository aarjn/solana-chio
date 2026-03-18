use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    },
    #[command(name = "--help")]
    Help,
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
            println!("Building program");
            let status = Command::new("cargo")
                .arg("build-sbf")
                .spawn()?
                .wait()
                .with_context(|| "Failed to build project")?;

            if !status.success() {
                anyhow::bail!("Build failed with exit code: {:?}", status.code());
            } else {
                println!("Build completed successfully!");
            }
        }
        Commands::Test => {
            println!("Testing program");
            let status = Command::new("cargo")
                .arg("test")
                .spawn()?
                .wait()
                .with_context(|| "Failed to test project")?;

            if !status.success() {
                anyhow::bail!("Test failed with exit code: {:?}", status.code());
            } else {
                println!("Tested successfully!");
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
        Commands::Keys { action } => {
            handle_keys_action(action)?;
        }
        Commands::Help => {
            display_help_banner()?;
        }
    }

    Ok(())
}

fn display_help_banner() -> Result<()> {
    // banner
    println!(
        r#"
      *     *
  ___| |__ (_) ___
 / __| '_ \| |/ _ \
| (__| | | | | (_) |
 \___|_| |_|_|\___/
 "#
    );

    println!("👾 Setup your pinocchio project blazingly fast💨");

    println!("\n🏗️ AVAILABLE COMMANDS:");
    println!("   chio init <project_name> - Initialize a new Pinocchio project");
    println!("   chio build               - Build the project");
    println!("   chio test                - Run project tests");
    println!("   chio deploy              - Deploy the project");

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

    println!(
        r#"
      *     *
  ___| |__ (_) ___
 / __| '_ \| |/ _ \
| (__| | | | | (_) |
 \___|_| |_|_|\___/

 "#
    );
    println!("🧑🏻‍🍳 Initializing your pinocchio project: {}", project_name);
    println!(""); // Create the project directory
    let project_dir = Path::new(project_name);
    fs::create_dir_all(project_dir)
        .with_context(|| format!("Failed to create project directory: {}", project_name))?;

    // init new cargo project inside
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

    // generate keypair
    let keypair_path = format!("./target/deploy/{}-keypair.json", project_name);
    let keygen_output = Command::new("solana-keygen")
        .arg("new")
        .arg("-o")
        .arg(&keypair_path)
        .arg("--no-bip39-passphrase") // skip the passphrase prompt
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

    let program_address: String;
    if address_output.status.success() {
        program_address = String::from_utf8_lossy(&address_output.stdout)
            .trim()
            .to_string();
        println!("Generated program address: {}", program_address);
    } else {
        let error = String::from_utf8_lossy(&address_output.stderr);
        anyhow::bail!("Failed to get program address from keypair: {}", error);
    }

    let user_address_output = Command::new("solana")
        .arg("address")
        .current_dir(project_dir)
        .output()
        .with_context(|| "Failed to get user address")?;

    let user_address: String;
    if user_address_output.status.success() {
        user_address = String::from_utf8_lossy(&user_address_output.stdout)
            .trim()
            .to_string();
    } else {
        let error = String::from_utf8_lossy(&user_address_output.stderr);
        println!("Failed to get user Solana address: {}", error);
        user_address = String::new();
    }

    create_project_structure(
        project_dir,
        user_address,
        program_address.clone(),
        test_framework,
    )?;
    update_cargo_toml(project_dir, project_name, test_framework)?;

    init_git_repo(project_dir, project_name)?;

    println!("");
    println!(
        "✅ Pinocchio Project '{}' initialized successfully!",
        project_name
    );
    println!("\n📋 Next steps:");
    println!("$ cd {}", project_name);
    println!("$ chio build");
    println!("$ chio test");
    println!("$ chio deploy");
    println!("");

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

    match test_framework {
        TestFramework::Mollusk => {
            fs::write(
                src_dir.join("lib.rs"),
                templates::lib_rs(program_address.as_str()),
            )?;

            let test_dir = project_dir.join("tests");
            fs::create_dir_all(&test_dir)?;

            let project_name = project_dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("project");

            fs::write(
                test_dir.join("tests.rs"),
                templates::unit_tests::unit_test_rs(&user_address, &program_address, project_name),
            )?;
        }
        TestFramework::Litesvm => {
            fs::write(
                src_dir.join("lib.rs"),
                templates::lib_rs(program_address.as_str()),
            )?;

            let test_dir = project_dir.join("tests");
            fs::create_dir_all(&test_dir)?;

            let project_name_str = project_dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("project");

            fs::write(
                test_dir.join("initialize.rs"),
                templates::unit_tests::litesvm_initialize_rs(project_name_str),
            )?;
        }
    }

    fs::write(src_dir.join("entrypoint.rs"), templates::entrypoint_rs())?;

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
mollusk-svm = "0.7.0"
mollusk-svm-bencher = "0.7.0"
"#
        }
        TestFramework::Litesvm => {
            r#"
[dev-dependencies]
solana-sdk = "3.0.0"
litesvm = "0.8.1"
litesvm-token = "0.8.1"
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
pinocchio = "0.9.2"
pinocchio-log = "0.5.1"
pinocchio-pubkey = "0.3.0"
pinocchio-system = "0.3.0"
shank = "0.4.5"

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

fn find_unique_keypair_file(dir: &Path) -> Result<PathBuf> {
    // 1. Define the regex for *-keypair.json
    // ^ matches start, .* matches anything, \.json$ matches the extension exactly
    let re = Regex::new(r".*-keypair\.json$").unwrap();

    // 2. Read the directory and filter for files matching the regex
    let matches: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| re.is_match(name))
        })
        .collect();

    // 3. Ensure there is exactly one file
    match matches.len() {
        1 => Ok(matches[0].clone()),
        0 => anyhow::bail!("No keypair file found matching *-keypair.json"),
        n => anyhow::bail!("Expected 1 keypair file, but found {}: {:?}", n, matches),
    }
}

fn generate_and_update_keys(
    lib_path: &Path,
    kp_path: &PathBuf,
    content: &str,
    re: &Regex,
) -> Result<String> {
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
    let new_content = re.replace(content, format!(r#"declare_id!("{}")"#, new_address));
    fs::write(lib_path, new_content.to_string())
        .with_context(|| "Failed to write updated ID to src/lib.rs")?;

    Ok(new_address)
}

fn handle_keys_action(action: &KeyAction) -> Result<()> {
    let target_deploy_dir = Path::new("target/deploy");
    if !target_deploy_dir.exists() {
        fs::create_dir_all(target_deploy_dir)?;
    }

    let lib_path = Path::new("src/lib.rs");
    if !lib_path.exists() {
        anyhow::bail!("src/lib.rs not found. Please run 'chio init' first.");
    }

    let content = fs::read_to_string(lib_path).with_context(|| "Failed to read src/lib.rs")?;

    // Use * instead of + to allow empty strings like declare_id!("")
    let re = Regex::new(r#"declare_id!\s*\(\s*"([^"]*)"\s*\)"#).unwrap();

    // Check if the macro exists at all. Error if missing, proceed if empty.
    let captures = re.captures(&content)
        .ok_or_else(|| anyhow::anyhow!("The 'declare_id!' macro was not found in src/lib.rs. It must be present even if empty."))?;

    let declared_program_address = captures
        .get(1)
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();

    // Define the expected keypair path (usually based on project name or 'program')
    // Here we check for an existing one first to compare
    let keypair_path = find_unique_keypair_file(target_deploy_dir).with_context(|| {
        anyhow::anyhow!("No keypair found in target/deploy. Chio did not init properly")
    })?;

    match action {
        KeyAction::Sync => {
            let mut current_keypair_address = String::new();

            let output = Command::new("solana")
                .arg("address")
                .arg("-k")
                .arg(&keypair_path)
                .output()?;
            if output.status.success() {
                current_keypair_address =
                    String::from_utf8_lossy(&output.stdout).trim().to_string();
            }

            if !current_keypair_address.is_empty()
                && declared_program_address == current_keypair_address
            {
                println!("✅ Keys are already synced: {}", current_keypair_address);
            } else {
                if current_keypair_address.is_empty() {
                    println!("No keypair found. Generating one to sync...");
                } else {
                    println!("⚠️ Keys mismatch. Generating new keypair to sync...");
                }

                // Reuse logic to generate and update
                let new_address = generate_and_update_keys(lib_path, &keypair_path, &content, &re)?;
                println!("✅ New keypair generated and synced: {}", new_address);
            }
        }
        KeyAction::Generate => {
            println!("Generating a fresh keypair...");
            let new_address = generate_and_update_keys(lib_path, &keypair_path, &content, &re)?;
            println!("✅ Generated and updated src/lib.rs with: {}", new_address);
        }
    }

    Ok(())
}
