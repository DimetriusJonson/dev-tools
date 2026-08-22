# Cross-platform justfile for dev-tools
# Install just: cargo install just

set dotenv-load := true

# Default recipe - show available commands
default:
    @just --list

build-macos:
    LEPTOS_TAILWIND_VERSION="v4.3.3" cargo leptos build --release -vv
    mkdir -p src-tauri/binaries
    cp target/release/server src-tauri/binaries/webdev_useful_tools_server-x86_64-apple-darwin
    TAURI_SIGNING_PRIVATE_KEY=$(cat ~/.tauri/dev-tools.key) cargo tauri build

build-linux:
    LEPTOS_TAILWIND_VERSION="v4.3.3" cargo leptos build --release -vv
    mkdir -p src-tauri/binaries
    cp target/release/server src-tauri/binaries/webdev_useful_tools_server-x86_64-unknown-linux-gnu
    TAURI_SIGNING_PRIVATE_KEY=$(cat ~/.tauri/dev-tools.key) cargo tauri build

build-windows:
    LEPTOS_TAILWIND_VERSION="v4.3.3" cargo leptos build --release -vv
    mkdir -p src-tauri/binaries
    cp target/release/server.exe src-tauri/binaries/webdev_useful_tools_server-x86_64-pc-windows-msvc.exe
    TAURI_SIGNING_PRIVATE_KEY=$(cat ~/.tauri/dev-tools.key) cargo tauri build

tauri-dev:
    LEPTOS_TAILWIND_VERSION="v4.3.3" cargo leptos build
    mkdir -p src-tauri/binaries
    cp target/debug/server.exe src-tauri/binaries/webdev_useful_tools_server-x86_64-pc-windows-msvc.exe
    TAURI_SIGNING_PRIVATE_KEY=$(cat ~/.tauri/dev-tools.key) cargo tauri dev

github-release version:
    git tag -f {{version}} -m release {{version}}
    git push origin -f {{version}}   



