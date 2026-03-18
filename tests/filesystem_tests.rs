//! Tests for file system operations
//!
//! This module tests all file and directory creation functionality
//! to ensure projects are properly scaffolded with correct structure.

#[cfg(test)]
mod filesystem {
    use chio::content::templates;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn create_cargo_toml_mollusk_dependencies() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_dir = temp_dir.path();

        let cargo_toml = templates::cargo_toml_mollusk("test_project");

        fs::write(project_dir.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");

        let content =
            fs::read_to_string(project_dir.join("Cargo.toml")).expect("Failed to read Cargo.toml");

        assert!(content.contains("mollusk-svm"));
        assert!(content.contains("mollusk-svm-bencher"));
        assert!(content.contains("pinocchio"));
        assert!(content.contains("solana-sdk"));
    }

    #[test]
    fn create_cargo_toml_litesvm_dependencies() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_dir = temp_dir.path();

        let cargo_toml = templates::cargo_toml_litesvm("test_project");

        fs::write(project_dir.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");

        let content =
            fs::read_to_string(project_dir.join("Cargo.toml")).expect("Failed to read Cargo.toml");

        assert!(content.contains("litesvm"));
        assert!(content.contains("litesvm-token"));
        assert!(!content.contains("mollusk-svm"));
    }

    #[test]
    fn cargo_toml_project_name_in_package() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_dir = temp_dir.path();
        let project_name = "my_awesome_project";

        let cargo_toml = format!("[package]\nname = \"{}\"", project_name);
        fs::write(project_dir.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");

        let content =
            fs::read_to_string(project_dir.join("Cargo.toml")).expect("Failed to read Cargo.toml");

        assert!(content.contains(&format!("name = \"{}\"", project_name)));
    }

    #[test]
    fn cargo_toml_required_features() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_dir = temp_dir.path();

        let cargo_toml = r#"[features]
no-entrypoint = []
std = []
test-default = ["no-entrypoint", "std"]
"#;
        fs::write(project_dir.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");

        let content =
            fs::read_to_string(project_dir.join("Cargo.toml")).expect("Failed to read Cargo.toml");

        assert!(content.contains("[features]"));
        assert!(content.contains("no-entrypoint = []"));
        assert!(content.contains("std = []"));
        assert!(content.contains("test-default = [\"no-entrypoint\", \"std\"]"));
    }

    #[test]
    fn cargo_toml_library_crate_type() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_dir = temp_dir.path();

        let cargo_toml = r#"[lib]
crate-type = ["cdylib", "rlib"]
"#;
        fs::write(project_dir.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");

        let content =
            fs::read_to_string(project_dir.join("Cargo.toml")).expect("Failed to read Cargo.toml");

        assert!(content.contains("crate-type = [\"cdylib\", \"rlib\"]"));
    }

    #[test]
    fn create_required_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_dir = temp_dir.path();
        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir).expect("Failed to create src dir");

        // Create all required files
        fs::write(
            src_dir.join("lib.rs"),
            templates::lib_rs("11111111111111111111111111111112"),
        )
        .expect("Failed to write lib.rs");
        fs::write(src_dir.join("entrypoint.rs"), templates::entrypoint_rs())
            .expect("Failed to write entrypoint.rs");
        fs::write(src_dir.join("errors.rs"), templates::errors_rs())
            .expect("Failed to write errors.rs");
        fs::write(project_dir.join("README.md"), templates::readme_md())
            .expect("Failed to write README.md");
        fs::write(project_dir.join(".gitignore"), templates::gitignore())
            .expect("Failed to write .gitignore");

        // Verify all files exist
        assert!(src_dir.join("lib.rs").exists());
        assert!(src_dir.join("entrypoint.rs").exists());
        assert!(src_dir.join("errors.rs").exists());
        assert!(project_dir.join("README.md").exists());
        assert!(project_dir.join(".gitignore").exists());
    }

    #[test]
    fn create_required_directories() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_dir = temp_dir.path();

        fs::create_dir_all(project_dir.join("src/instructions"))
            .expect("Failed to create instructions dir");
        fs::create_dir_all(project_dir.join("src/states")).expect("Failed to create states dir");
        fs::create_dir_all(project_dir.join("tests")).expect("Failed to create tests dir");

        assert!(project_dir.join("src").exists());
        assert!(project_dir.join("src/instructions").exists());
        assert!(project_dir.join("src/states").exists());
        assert!(project_dir.join("tests").exists());
    }

    #[test]
    fn mollusk_test_file_contains_framework_reference() {
        let _temp_dir = TempDir::new().expect("Failed to create temp dir");

        let test_file = templates::unit_tests::unit_test_rs("user", "prog", "test_proj");

        assert!(test_file.contains("mollusk"));
        assert!(test_file.contains("use mollusk_svm"));
    }

    #[test]
    fn litesvm_test_file_contains_framework_reference() {
        let _temp_dir = TempDir::new().expect("Failed to create temp dir");

        let test_file = templates::unit_tests::litesvm_initialize_rs("test_proj");

        assert!(test_file.contains("litesvm"));
        assert!(test_file.contains("use litesvm"));
    }

    #[test]
    fn program_address_in_generated_lib() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_dir = temp_dir.path();
        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir).expect("Failed to create src dir");

        let test_address = "11111111111111111111111111111112";
        fs::write(src_dir.join("lib.rs"), templates::lib_rs(test_address))
            .expect("Failed to write lib.rs");

        let lib_content =
            fs::read_to_string(src_dir.join("lib.rs")).expect("Failed to read lib.rs");

        assert!(lib_content.contains(test_address));
    }

    #[test]
    fn readme_contains_project_structure() {
        let readme = templates::readme_md();

        assert!(readme.contains("src/"));
        assert!(readme.contains("tests/"));
        assert!(readme.contains("entrypoint.rs"));
        assert!(readme.contains("instructions/"));
        assert!(readme.contains("states/"));
    }

    #[test]
    fn gitignore_has_required_entries() {
        let gitignore = templates::gitignore();

        assert!(gitignore.contains("/target"));
        assert!(gitignore.contains(".env"));
    }
}
