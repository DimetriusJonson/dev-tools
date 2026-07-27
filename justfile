# Cross-platform justfile for dev-tools
# Install just: cargo install just

set dotenv-load := true

# Default recipe - show available commands
default:
    @just --list

build-macos:
    trunk build --release 
    cargo build --release --bin server
    mkdir -p src-tauri/binaries
    cp target/release/server src-tauri/binaries/webdev_useful_tools_server-x86_64-apple-darwin
    cargo tauri build

build-linux:
    trunk build --release 
    cargo build --release --bin server
    mkdir -p src-tauri/binaries
    cp target/release/server src-tauri/binaries/webdev_useful_tools_server-x86_64-unknown-linux-gnu
    cargo tauri build

build-windows:
    trunk build --release 
    cargo build --release --bin server
    mkdir -p src-tauri/binaries
    cp target/release/server.exe src-tauri/binaries/webdev_useful_tools_server-x86_64-pc-windows-msvc.exe
    cargo tauri build


