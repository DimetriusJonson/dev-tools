# Cross-platform justfile for dev-tools
# Install just: cargo install just

set dotenv-load := true

# Default recipe - show available commands
default:
    @just --list

build-linux:
    cargo leptos build --release -vv
    mkdir -p src-tauri/binaries
    cp target/release/server src-tauri/binaries/webdev_useful_tools_server-x86_64-unknown-linux-gnu
    cargo tauri build

build-windows:
    cargo leptos build --release -vv
    mkdir -p src-tauri/binaries
    cp target/release/server.exe src-tauri/binaries/webdev_useful_tools_server-x86_64-pc-windows-msvc.exe
    cargo tauri build


