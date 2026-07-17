# Cross-platform justfile for dev-tools
# Install just: cargo install just

set dotenv-load := true

# Default recipe - show available commands
default:
    @just --list

#build-linux:
#    cargo build --release --target x86_64-unknown-linux-gnu
#    cp target/x86_64-unknown-linux-gnu/release/my_app ./dist/my_app-linux

build-windows:
    cargo leptos build --release -vv
    mkdir -p src-tauri/binaries
    cp target/release/server.exe src-tauri/binaries/webdev_useful_tools_server-x86_64-pc-windows-msvc.exe
    cargo tauri build


