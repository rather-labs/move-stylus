# Install Move-Stylus CLI

## Prerequisites

To install the Move-Stylus CLI, you need to have [Rust](https://doc.rust-lang.org/cargo/getting-started/installation.html) and [git](https://git-scm.com/) installed on your machine.

## Installation

For the moment, the only way to install the Stylus CLI is by building it from source. You can do this by cloning the Stylus repository and using Cargo to build and install the CLI.

#### Install `cargo-stylus`

```bash
RUSTFLAGS="-C link-args=-rdynamic" cargo install --force --version 0.6.3 cargo-stylus
```

#### Cloning the Repository

```bash
git clone https://github.com/rather-labs/move-stylus/
```

#### Building and Installing the compiler and CLI

```bash
cd move-stylus
cargo install --locked --path crates/move-cli
```

## Verify Installation

After the installation is complete, you can verify that the Move-Stylus CLI is installed correctly by checking its version:

```bash
move-stylus --version
```

You should see output similar to:

```
move-stylus 0.1.0
```

> [!NOTE]
> You may need to restart your terminal or add Cargo's bin directory to your `PATH` if the command is not found. The default location for Cargo's bin directory is `$HOME/.cargo/bin`.



