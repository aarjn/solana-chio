
<div align="center"> <img src="assets/logo.png" alt="Chio CLI Logo" width="20%"> <h1>Chio</h1> <p> Setup Solana Pinocchio projects blazingly fast</p>
Author: 

<a class="header-badge" target="_blank" href="https://twitter.com/4rjunc"> <img alt="Twitter" src="https://img.shields.io/badge/@4rjunc-000000?style=for-the-badge&logo=x&logoColor=white"> </a> </div>

## About

Chio is a command-line tool designed to make it easy to set up and manage [Pinocchio](https://github.com/anza-xyz/pinocchio) projects on Solana. It automates common development tasks including project initialization, building, testing, and deployment with simple commands.

## Features

- 🚀 Fast project scaffolding with best practices
- 📁 Proper directory structure for Solana/Pinocchio development
- 🔨 Simple build, test, and deployment commands
- 💻 Comprehensive testing environment setup

## Installation

### From GitHub

```bash
cargo install --git https://github.com/4rjunc/solana-chio --force
```

### From Source

1. Clone the repository
   ```bash
   git clone https://github.com/4rjunc/solana-chio.git
   cd solana-chio
   ```

2. Build the tool
   ```bash
   cargo build --release
   ```

3. Install globally
   ```bash
   cargo install --path .
   ```

## Usage

### Available Commands

```bash
# Initialize a new project (default tests: mollusk)
chio init <project-name>

# Use LiteSVM tests instead of Mollusk
chio init <project-name> --test-framework litesvm

# Build your project
chio build

# Run tests
chio test

# Deploy your program
chio deploy

# Sync keypairs between target/deploy/keypair.json and declared id in lib.rs
chio keys sync

# Generate a new keypair and sync it with the declared id
chio keys generate

# Get help
chio --help
```

### Example

Create a new Pinocchio project and get started:

```bash
# Create a new project
chio init my-pinocchio-app

# Navigate to your project
cd my-pinocchio-app

# Build your project
chio build

# Run tests
chio test

# LiteSVM example
chio init my-pinocchio-app --test-framework litesvm
cd my-pinocchio-app
chio build
chio test
```



## Project Structure

When you initialize a project with `chio init`, it creates the following structure:

```
my-project/
├── Cargo.toml
├── src/
│   ├── lib.rs               # Library crate using no_std
│   ├── entrypoint.rs        # Program entrypoint
│   ├── errors.rs            # Error definitions
│   ├── instructions/        # Program instructions
│   │   ├── mod.rs
│   │   ├── deposit.rs
│   │   └── withdraw.rs
│   └── states/              # Account state definitions
│       ├── mod.rs
│       └── utils.rs
└── tests/                   # Test files
    └── tests.rs
```


## Contributing

Contributions are welcome! Here's how you can contribute:

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests to ensure everything works
5. Commit your changes (`git commit -m 'Add some amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Development Setup

1. Ensure you have Rust and Cargo installed
2. Install Solana CLI tools
3. Clone the repository
4. Build with `cargo build --release`
5. To install too `cargo install --path .`

