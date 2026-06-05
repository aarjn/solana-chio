//! Tests for template generation
//!
//! This module tests all template functions to ensure they generate
//! correct and consistent code for scaffolded Pinocchio programs.

#[cfg(test)]
mod templates {
    use chio::content::templates::*;

    #[test]
    fn lib_rs_is_no_std() {
        let output = lib_rs();
        assert!(output.contains("#![no_std]"));
    }

    #[test]
    fn lib_rs_declares_required_modules() {
        let output = lib_rs();
        assert!(output.contains("pub mod errors"));
        assert!(output.contains("pub mod instructions"));
        assert!(output.contains("pub mod states"));
    }

    #[test]
    fn lib_rs_reexports_id() {
        let output = lib_rs();
        assert!(output.contains("pub use entrypoint::{id, ID}"));
    }

    #[test]
    fn entrypoint_rs_has_declare_id_macro() {
        let address = "11111111111111111111111111111112";
        let output = entrypoint_rs(address);
        assert!(output.contains("declare_id!"));
        assert!(output.contains(address));
    }

    #[test]
    fn entrypoint_rs_has_program_entrypoint_macro() {
        let output = entrypoint_rs("11111111111111111111111111111112");
        assert!(output.contains("program_entrypoint!"));
    }

    #[test]
    fn entrypoint_rs_has_no_allocator_macro() {
        let output = entrypoint_rs("11111111111111111111111111111112");
        assert!(output.contains("no_allocator!()"));
    }

    #[test]
    fn entrypoint_rs_has_default_panic_handler() {
        let output = entrypoint_rs("11111111111111111111111111111112");
        assert!(output.contains("default_panic_handler!()"));
    }

    #[test]
    fn entrypoint_rs_has_process_instruction_function() {
        let output = entrypoint_rs("11111111111111111111111111111112");
        assert!(output.contains("fn process_instruction"));
    }

    #[test]
    fn entrypoint_rs_handles_initialize_instruction() {
        let output = entrypoint_rs("11111111111111111111111111111112");
        assert!(output.contains("ProgramInstruction::InitializeState"));
    }

    #[test]
    fn entrypoint_rs_imports_required_modules() {
        let output = entrypoint_rs("11111111111111111111111111111112");
        assert!(output.contains("use crate::instructions"));
        assert!(output.contains("use pinocchio"));
    }

    #[test]
    fn entrypoint_rs_handles_invalid_instruction_data() {
        let output = entrypoint_rs("11111111111111111111111111111112");
        assert!(output.contains("ProgramError::InvalidInstructionData"));
    }

    #[test]
    fn errors_rs_defines_my_program_error_enum() {
        let output = errors_rs();
        assert!(output.contains("pub enum MyProgramError"));
    }

    #[test]
    fn errors_rs_has_all_required_error_variants() {
        let output = errors_rs();
        assert!(output.contains("InvalidInstructionData"));
        assert!(output.contains("PdaMismatch"));
        assert!(output.contains("InvalidOwner"));
    }

    #[test]
    fn errors_rs_has_shank_derive_macro() {
        let output = errors_rs();
        assert!(output.contains("shank::ShankType"));
    }

    #[test]
    fn errors_rs_implements_from_program_error() {
        let output = errors_rs();
        assert!(output.contains("impl From<MyProgramError> for ProgramError"));
    }

    #[test]
    fn readme_md_has_title() {
        let output = readme_md();
        assert!(output.contains("# Chio Pinocchio Project"));
    }

    #[test]
    fn readme_md_has_project_structure_section() {
        let output = readme_md();
        assert!(output.contains("## Project Structure"));
    }

    #[test]
    fn readme_md_documents_all_directories() {
        let output = readme_md();
        assert!(output.contains("src/"));
        assert!(output.contains("entrypoint.rs"));
        assert!(output.contains("lib.rs"));
        assert!(output.contains("instructions/"));
        assert!(output.contains("states/"));
        assert!(output.contains("errors.rs"));
    }

    #[test]
    fn readme_md_has_command_examples() {
        let output = readme_md();
        assert!(output.contains("chio build"));
        assert!(output.contains("chio test"));
        assert!(output.contains("chio deploy"));
    }

    #[test]
    fn readme_md_mentions_mollusk_framework() {
        let output = readme_md();
        assert!(output.contains("mollusk-svm"));
    }

    #[test]
    fn gitignore_ignores_target_directory() {
        let output = gitignore();
        assert!(output.contains("/target"));
    }

    #[test]
    fn gitignore_ignores_env_files() {
        let output = gitignore();
        assert!(output.contains(".env"));
    }

    #[test]
    fn initialize_instruction_template_has_initialize_struct() {
        let output = instructions::initialize();
        assert!(output.contains("pub struct Initialize"));
    }

    #[test]
    fn initialize_instruction_has_owner_and_bump_fields() {
        let output = instructions::initialize();
        assert!(output.contains("pub owner: [u8; 32]"));
        assert!(output.contains("pub bump: u8"));
    }

    #[test]
    fn initialize_instruction_validates_signer() {
        let output = instructions::initialize();
        assert!(output.contains("is_signer"));
        assert!(output.contains("MissingRequiredSignature"));
    }

    #[test]
    fn initialize_instruction_validates_pda() {
        let output = instructions::initialize();
        assert!(output.contains("validate_pda"));
        assert!(output.contains("MyState::validate_pda"));
    }

    #[test]
    fn initialize_instruction_creates_account() {
        let output = instructions::initialize();
        assert!(output.contains("CreateAccount"));
        assert!(output.contains("invoke_signed"));
    }

    #[test]
    fn initialize_instruction_checks_empty_account() {
        let output = instructions::initialize();
        assert!(output.contains("is_data_empty"));
        assert!(output.contains("AccountAlreadyInitialized"));
    }

    #[test]
    fn instructions_mod_rs_defines_program_instruction_enum() {
        let output = instructions::instructions_mod_rs();
        assert!(output.contains("pub enum ProgramInstruction"));
    }

    #[test]
    fn instructions_mod_rs_has_initialize_state_variant() {
        let output = instructions::instructions_mod_rs();
        assert!(output.contains("InitializeState"));
    }

    #[test]
    fn instructions_mod_rs_implements_try_from() {
        let output = instructions::instructions_mod_rs();
        assert!(output.contains("impl TryFrom<&u8> for ProgramInstruction"));
    }

    #[test]
    fn instructions_mod_rs_discriminator_mapping() {
        let output = instructions::instructions_mod_rs();
        assert!(output.contains("0 => Ok(ProgramInstruction::InitializeState)"));
    }

    #[test]
    fn states_mod_rs_declares_submodules() {
        let output = states::states_mod_rs();
        assert!(output.contains("pub mod state"));
        assert!(output.contains("pub mod utils"));
    }

    #[test]
    fn states_mod_rs_has_re_exports() {
        let output = states::states_mod_rs();
        assert!(output.contains("pub use state::*"));
        assert!(output.contains("pub use utils::*"));
    }

    #[test]
    fn state_rs_defines_my_state_struct() {
        let output = states::state_rs();
        assert!(output.contains("pub struct MyState"));
    }

    #[test]
    fn state_rs_has_owner_field() {
        let output = states::state_rs();
        assert!(output.contains("pub owner: [u8; 32]"));
    }

    #[test]
    fn state_rs_has_seed_constant() {
        let output = states::state_rs();
        assert!(output.contains("pub const SEED"));
        assert!(output.contains("\"init\""));
    }

    #[test]
    fn state_rs_has_validate_pda_function() {
        let output = states::state_rs();
        assert!(output.contains("fn validate_pda"));
        assert!(output.contains("create_program_address"));
    }

    #[test]
    fn state_rs_has_initialize_function() {
        let output = states::state_rs();
        assert!(output.contains("fn initialize"));
        assert!(output.contains("ix_data.owner"));
    }

    #[test]
    fn utils_rs_defines_data_len_trait() {
        let output = states::utils_rs();
        assert!(output.contains("pub trait DataLen"));
        assert!(output.contains("const LEN: usize"));
    }

    #[test]
    fn utils_rs_has_load_functions() {
        let output = states::utils_rs();
        assert!(output.contains("load_acc_unchecked"));
        assert!(output.contains("load_acc_mut_unchecked"));
        assert!(output.contains("load_ix_data"));
    }

    #[test]
    fn utils_rs_has_byte_conversion_functions() {
        let output = states::utils_rs();
        assert!(output.contains("fn to_bytes"));
        assert!(output.contains("fn to_mut_bytes"));
    }

    #[test]
    fn utils_rs_validates_data_length() {
        let output = states::utils_rs();
        assert!(output.contains("bytes.len() != T::LEN"));
        assert!(output.contains("InvalidAccountData"));
    }

    #[test]
    fn unit_test_rs_contains_mollusk_imports() {
        let test_output = unit_tests::unit_test_rs("user_addr", "prog_addr", "test_proj");
        assert!(test_output.contains("use mollusk_svm"));
    }

    #[test]
    fn unit_test_rs_contains_provided_addresses() {
        let user_addr = "user_address_here";
        let prog_addr = "program_address_here";
        let test_output = unit_tests::unit_test_rs(user_addr, prog_addr, "test_proj");
        assert!(test_output.contains(user_addr));
        assert!(test_output.contains(prog_addr));
    }

    #[test]
    fn unit_test_rs_contains_test_function() {
        let test_output = unit_tests::unit_test_rs("user_addr", "prog_addr", "test_proj");
        assert!(test_output.contains("#[test]"));
        assert!(test_output.contains("fn test_initialize_mystate"));
    }

    #[test]
    fn unit_test_rs_creates_pda() {
        let test_output = unit_tests::unit_test_rs("user_addr", "prog_addr", "test_proj");
        assert!(test_output.contains("find_program_address"));
        assert!(test_output.contains("MyState::SEED"));
    }

    #[test]
    fn unit_test_rs_creates_accounts() {
        let test_output = unit_tests::unit_test_rs("user_addr", "prog_addr", "test_proj");
        assert!(test_output.contains("payer_account"));
        assert!(test_output.contains("mystate_account"));
    }

    #[test]
    fn unit_test_rs_contains_project_name() {
        let proj_name = "my_test_project";
        let test_output = unit_tests::unit_test_rs("user", "prog", proj_name);
        assert!(test_output.contains(&format!("use {}", proj_name)));
    }

    #[test]
    fn litesvm_initialize_rs_contains_litesvm_imports() {
        let test_output = unit_tests::litesvm_initialize_rs("test_proj");
        assert!(test_output.contains("use litesvm"));
    }

    #[test]
    fn litesvm_initialize_rs_has_setup_function() {
        let test_output = unit_tests::litesvm_initialize_rs("test_proj");
        assert!(test_output.contains("fn setup()"));
        assert!(test_output.contains("LiteSVM::new()"));
    }

    #[test]
    fn litesvm_initialize_rs_loads_program() {
        let test_output = unit_tests::litesvm_initialize_rs("test_proj");
        assert!(test_output.contains("add_program"));
        assert!(test_output.contains(".so"));
    }

    #[test]
    fn litesvm_initialize_rs_creates_test_function() {
        let test_output = unit_tests::litesvm_initialize_rs("test_proj");
        assert!(test_output.contains("#[test]"));
        assert!(test_output.contains("fn test_initialize"));
    }

    #[test]
    fn litesvm_initialize_rs_airdrop_funds() {
        let test_output = unit_tests::litesvm_initialize_rs("test_proj");
        assert!(test_output.contains("airdrop"));
        assert!(test_output.contains("LAMPORTS_PER_SOL"));
    }

    #[test]
    fn litesvm_initialize_rs_finds_program_address() {
        let test_output = unit_tests::litesvm_initialize_rs("test_proj");
        assert!(test_output.contains("find_program_address"));
    }

    #[test]
    fn litesvm_initialize_rs_verifies_account_state() {
        let test_output = unit_tests::litesvm_initialize_rs("test_proj");
        assert!(test_output.contains("get_account"));
        assert!(test_output.contains("MyState::LEN"));
    }

    #[test]
    fn template_consistency_both_tests_use_mystate() {
        let mollusk_test = unit_tests::unit_test_rs("user", "prog", "proj");
        let litesvm_test = unit_tests::litesvm_initialize_rs("proj");
        assert!(mollusk_test.contains("MyState"));
        assert!(litesvm_test.contains("MyState"));
    }

    #[test]
    fn template_consistency_both_tests_use_initialize() {
        let mollusk_test = unit_tests::unit_test_rs("user", "prog", "proj");
        let litesvm_test = unit_tests::litesvm_initialize_rs("proj");
        assert!(mollusk_test.contains("Initialize"));
        assert!(litesvm_test.contains("Initialize"));
    }

    #[test]
    fn litesvm_initialize_rs_contains_project_name() {
        let proj_name = "my_test_project";
        let test_output = unit_tests::litesvm_initialize_rs(proj_name);
        assert!(test_output.contains(&format!("use {}", proj_name)));
    }
}
