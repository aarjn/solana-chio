pub mod templates {

    //lib.rs
    pub fn lib_rs(address: &str) -> String {
        format!(
            r#"#![no_std]

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

#[cfg(feature = "std")]
extern crate std;

pub mod errors;
pub mod instructions;
pub mod states;

pinocchio::address::declare_id!("{}");"#,
            address
        )
    }

    // entrypoint.rs template
    pub fn entrypoint_rs() -> &'static str {
        r#"#![allow(unexpected_cfgs)]

use crate::instructions::{self, ProgramInstruction};
use pinocchio::{
    error::ProgramError, default_panic_handler, no_allocator, program_entrypoint,
    AccountView, Address, ProgramResult,
};

// This is the entrypoint for the program.
program_entrypoint!(process_instruction);
//Do not allocate memory.
no_allocator!();
// Use the no_std panic handler.
default_panic_handler!();

#[inline(always)]
fn process_instruction(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let (ix_disc, instruction_data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match ProgramInstruction::try_from(ix_disc)? {
        ProgramInstruction::InitializeState => {
            pinocchio_log::log!("initialize");
            instructions::initialize(accounts, instruction_data)
        }
    }
}"#
    }

    // Configuration files
    pub fn readme_md() -> &'static str {
        r#"# Chio Pinocchio Project

A Solana program built with the Chio CLI tool.

## Project Structure

```
src/
├── entrypoint.rs          # Program entry point with nostd_panic_handler
├── lib.rs                 # Library crate (no_std optimization)
├── instructions/          # Program instruction handlers
├── states/                # Account state definitions
│   └── utils.rs           # State management helpers (load_acc, load_mut_acc)
└── errors.rs              # Program error definitions

tests/
└── tests.rs               # Unit tests using mollusk-svm framework
```

## Commands

```bash
# Build the program
 chio build

# Run tests
 chio test

# Deploy the program
 chio deploy

# Get help
 chio help
```

---

**Author of Chio CLI**: [4rjunc](https://github.com/4rjunc) | [Twitter](https://x.com/4rjunc)"#
    }

    pub fn gitignore() -> &'static str {
        r#"/target
.env"#
    }

    pub fn cargo_toml_mollusk(project_name: &str) -> String {
        format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
pinocchio = "0.10.2"

[dev-dependencies]
solana-sdk = "3.0.0"
mollusk-svm = "0.9.0"
mollusk-svm-bencher = "0.9.0"

[features]
no-entrypoint = []
std = []
test-default = ["no-entrypoint", "std"]
"#,
            project_name
        )
    }

    pub fn cargo_toml_litesvm(project_name: &str) -> String {
        format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
pinocchio = "0.10.2"

[dev-dependencies]
solana-sdk = "3.0.0"
litesvm = "0.9.1"
litesvm-token = "0.9.1"

[features]
no-entrypoint = []
std = []
test-default = ["no-entrypoint", "std"]
"#,
            project_name
        )
    }

    pub fn errors_rs() -> &'static str {
        r#"use pinocchio::error::ProgramError;

#[derive(Clone, PartialEq, shank::ShankType)]
pub enum MyProgramError {
    InvalidInstructionData,
    PdaMismatch,
    InvalidOwner,
}

impl From<MyProgramError> for ProgramError {
    fn from(e: MyProgramError) -> Self {
        Self::Custom(e as u32)
    }
}
"#
    }

    pub mod instructions {
        pub fn initialize() -> &'static str {
            r#"use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, rent::Rent},
    AccountView, ProgramResult,
};

use pinocchio_system::instructions::CreateAccount;

use crate::{
    errors::MyProgramError,
    states::{
        utils::{load_ix_data, DataLen},
        MyState,
    },
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Initialize {
    pub owner: [u8; 32],
    pub bump: u8,
}

impl DataLen for Initialize {
    const LEN: usize = core::mem::size_of::<Initialize>();
}

pub fn initialize(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [payer_acc, state_acc, _system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !payer_acc.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !state_acc.is_data_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let ix_data = unsafe { load_ix_data::<Initialize>(data)? };

    if ix_data.owner != *payer_acc.address().as_ref() {
        return Err(MyProgramError::InvalidOwner.into());
    }

    let pda_bump_bytes = [ix_data.bump];

    MyState::validate_pda(ix_data.bump, state_acc.address(), &ix_data.owner)?;

    // signer seeds
    let signer_seeds = [
        Seed::from(MyState::SEED.as_bytes()),
        Seed::from(&ix_data.owner),
        Seed::from(&pda_bump_bytes[..]),
    ];
    let signers = [Signer::from(&signer_seeds[..])];

    CreateAccount {
        from: payer_acc,
        to: state_acc,
        space: MyState::LEN as u64,
        owner: &crate::ID,
        lamports: Rent::get()?.minimum_balance_unchecked(MyState::LEN),
    }
    .invoke_signed(&signers)?;

    MyState::initialize(state_acc, ix_data)?;

    Ok(())
}"#
        }

        pub fn instructions_mod_rs() -> &'static str {
            r#"use pinocchio::error::ProgramError;

pub mod initialize;

pub use initialize::*;

#[repr(u8)]
pub enum ProgramInstruction {
    InitializeState,
}

impl TryFrom<&u8> for ProgramInstruction {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(ProgramInstruction::InitializeState),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}"#
        }
    }

    pub mod states {
        pub fn states_mod_rs() -> &'static str {
            r#"pub mod state;
pub mod utils;

pub use state::*;
pub use utils::*;"#
        }

        pub fn state_rs() -> &'static str {
            r#"use super::utils::{load_acc_mut_unchecked, DataLen};
use pinocchio::{
    error::ProgramError,
    AccountView, Address, ProgramResult,
};

use crate::{errors::MyProgramError, instructions::Initialize};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MyState {
    pub owner: [u8; 32],
}

impl DataLen for MyState {
    const LEN: usize = core::mem::size_of::<MyState>();
}

impl MyState {
    pub const SEED: &'static str = "init";

    pub fn validate_pda(bump: u8, pda: &Address, owner: &[u8; 32]) -> Result<(), ProgramError> {
        let seed_with_bump = &[Self::SEED.as_bytes(), owner.as_ref(), &[bump]];
        let derived = Address::create_program_address(seed_with_bump, &crate::ID)?;
        if derived != *pda {
            return Err(MyProgramError::PdaMismatch.into());
        }
        Ok(())
    }

    pub fn initialize(my_stata_acc: &AccountView, ix_data: &Initialize) -> ProgramResult {
        let my_state =
            unsafe { load_acc_mut_unchecked::<MyState>(my_stata_acc.borrow_unchecked_mut()) }?;

        my_state.owner = ix_data.owner;
        Ok(())
    }
}"#
        }

        pub fn utils_rs() -> &'static str {
            r#"use pinocchio::error::ProgramError;

use crate::errors::MyProgramError;

pub trait DataLen {
    const LEN: usize;
}

#[inline(always)]
pub unsafe fn load_acc_unchecked<T: DataLen>(bytes: &[u8]) -> Result<&T, ProgramError> {
    if bytes.len() != core::mem::size_of::<T>() {
        return Err(ProgramError::InvalidAccountData);
    }
    if bytes.as_ptr() as usize % core::mem::align_of::<T>() != 0 {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(&*(bytes.as_ptr() as *const T))
}

#[inline(always)]
pub unsafe fn load_acc_mut_unchecked<T: DataLen>(bytes: &mut [u8]) -> Result<&mut T, ProgramError> {
    if bytes.len() != core::mem::size_of::<T>() {
        return Err(ProgramError::InvalidAccountData);
    }
    if bytes.as_ptr() as usize % core::mem::align_of::<T>() != 0 {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(&mut *(bytes.as_mut_ptr() as *mut T))
}

#[inline(always)]
pub unsafe fn load_ix_data<T: DataLen>(bytes: &[u8]) -> Result<&T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(MyProgramError::InvalidInstructionData.into());
    }
    Ok(&*(bytes.as_ptr() as *const T))
}

pub unsafe fn to_bytes<T: DataLen>(data: &T) -> &[u8] {
    core::slice::from_raw_parts(data as *const T as *const u8, T::LEN)
}

pub unsafe fn to_mut_bytes<T: DataLen>(data: &mut T) -> &mut [u8] {
    core::slice::from_raw_parts_mut(data as *mut T as *mut u8, T::LEN)
}"#
        }
    }

    pub mod unit_tests {
        pub fn unit_test_rs(address: &str, program_address: &str, project_name: &str) -> String {
            let template = r#"use mollusk_svm::result::{Check, ProgramResult};
use mollusk_svm::{program, Mollusk};
use solana_sdk::account::Account;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
extern crate alloc;
use alloc::vec;

use {project_name}::instructions::Initialize;
use {project_name}::states::{to_bytes, MyState};

pub const PROGRAM: Pubkey = pubkey!("{program_address}");

pub const PAYER: Pubkey = pubkey!("{address}");

pub fn mollusk() -> Mollusk {
    let mollusk = Mollusk::new(&PROGRAM, "target/deploy/{project_name}");
    mollusk
}

#[test]
fn test_initialize_mystate() {
    let mollusk = mollusk();

    //system program and system account
    let (system_program, system_account) = program::keyed_account_for_system_program();

    // Create the PDA
    let (mystate_pda, bump) =
        Pubkey::find_program_address(&[MyState::SEED.as_bytes(), &PAYER.to_bytes()], &PROGRAM);

    //Initialize the accounts
    let payer_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);
    let mystate_account = Account::new(0, 0, &system_program);

    //Push the accounts in to the instruction_accounts vec!
    let ix_accounts = vec![
        AccountMeta::new(PAYER, true),
        AccountMeta::new(mystate_pda, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    // Create the instruction data
    let ix_data = Initialize {
        owner: PAYER.to_bytes(),
        bump,
    };

    // Ix discriminator = 0
    let mut ser_ix_data = vec![0];

    // Serialize the instruction data
    ser_ix_data.extend_from_slice(unsafe { to_bytes(&ix_data) });

    // Create instruction
    let instruction = Instruction::new_with_bytes(PROGRAM, &ser_ix_data, ix_accounts);

    // Create tx_accounts vec
    let tx_accounts = &vec![
        (PAYER, payer_account.clone()),
        (mystate_pda, mystate_account.clone()),
        (system_program, system_account.clone()),
    ];

    let init_res =
        mollusk.process_and_validate_instruction(&instruction, tx_accounts, &[Check::success()]);

    assert!(init_res.program_result == ProgramResult::Success);
}
        "#;

            template
                .replace("{address}", address)
                .replace("{program_address}", program_address)
                .replace("{project_name}", project_name)
        }

        pub fn litesvm_initialize_rs(project_name: &str) -> String {
            let template = r#"// use this to run the tests -
// cargo test --features std  -- --no-capture

use std::path::PathBuf;

use litesvm::{types::TransactionResult, LiteSVM};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};

use {project_name}::instructions::Initialize;
use {project_name}::states::{self, MyState};
use {project_name}::states::utils::DataLen;

pub fn program_id() -> Pubkey {
    // Convert Pinocchio program ID to solana-sdk Pubkey
    Pubkey::new_from_array({project_name}::ID.0)
}

pub fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();

    let so_path = PathBuf::from("target/deploy").join("{project_name}.so");

    let program_data = std::fs::read(so_path).expect("Failed to read program .so file");
    svm.add_program(program_id(), &program_data).expect("add_program failed");

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL).expect("airdrop failed");

    (svm, payer)
}

pub struct InitializeData {
    pub payer: Pubkey,
    pub state_pda: (Pubkey, u8),
}

impl InitializeData {
    pub fn new(payer: &Keypair) -> Self {
        let (state_pda, bump) =
            Pubkey::find_program_address(&[MyState::SEED.as_bytes(), &payer.pubkey().to_bytes()], &program_id());
        Self {
            payer: payer.pubkey(),
            state_pda: (state_pda, bump),
        }
    }
}

pub fn initialize(
    svm: &mut LiteSVM,
    payer: &Keypair,
    data: &InitializeData,
) -> TransactionResult {
    // Discriminator 0 for Initialize, followed by Initialize{ owner, bump }
    let ix = Initialize { owner: data.payer.to_bytes(), bump: data.state_pda.1 };

    let mut ix_data = vec![0u8];
    // Serialize instruction payload using the program's utility
    let ix_bytes = unsafe { states::utils::to_bytes(&ix) };
    ix_data.extend_from_slice(ix_bytes);

    let system_program = Pubkey::new_from_array(pinocchio_system::ID.0);
    let accounts = vec![
        AccountMeta::new(data.payer, true),
        AccountMeta::new(data.state_pda.0, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let ix = Instruction { program_id: program_id(), accounts, data: ix_data };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[payer],
        svm.latest_blockhash(),
    );

    svm.send_transaction(tx)
}

#[test]
pub fn test_initialize() {
    let (mut svm, payer) = setup();
    let init_data = InitializeData::new(&payer);

    let _res = initialize(&mut svm, &payer, &init_data).expect("transaction failed");

    // Fetch PDA account and verify it was created with expected size
    let state_account = svm.get_account(&init_data.state_pda.0).expect("missing state account");
    assert_eq!(state_account.data.len(), MyState::LEN, "state size mismatch");
}
"#;
            template.replace("{project_name}", project_name)
        }
    }
}
