# Cross-platform justfile for dev-tools
# Install just: cargo install just

set dotenv-load := true

# Default recipe - show available commands
default:
    @just --list

build:
    LEPTOS_TAILWIND_VERSION="v4.3.3" TAURI_SIGNING_PRIVATE_KEY=$(cat ~/.tauri/dev-tools.key) cargo tauri build

tauri-dev:
    LEPTOS_TAILWIND_VERSION="v4.3.3" TAURI_SIGNING_PRIVATE_KEY=$(cat ~/.tauri/dev-tools.key) cargo tauri dev

github-release version:
    git tag -a {{version}} -m "release {{version}}"
    git push origin -f {{version}}   

github-release-fix version:
    git tag -f {{version}} -m "release {{version}}"
    git push origin -f {{version}}   
