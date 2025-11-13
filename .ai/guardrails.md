# 🛑 AGENT GUARDRAILS (Rust Project)

The agent must **NEVER** perform the following actions under any circumstances:

## 1. Git & Cargo Operations

* **FORBIDDEN:** Never `git push --force`.
* **FORBIDDEN:** Never commit directly to `main`, `master`, or `develop` branches. All work must be done in a feature branch.
* **FORBIDDEN:** Do not run `cargo publish` to publish crates.
* **FORBIDDEN:** Do not manually edit `Cargo.lock`. Use `cargo update` only for specific, approved dependency updates. Do not run a blind `cargo update`.
* **FORBIDDEN:** Do not add new dependencies to `Cargo.toml` (under `[dependencies]`, `[dev-dependencies]`, etc.) without explicit user approval.

## 2. Unsafe Code & Security

* **FORBIDDEN:** **Crucial:** Never introduce new `unsafe` blocks into the code.
* **FORBIDDEN:** Do not modify existing `unsafe` blocks unless it is the *explicit* task and has been confirmed by the user.
* **FORBIDDEN:** Do not bypass or disable `clippy` lints (e.g., with `#[allow(...)]`) unless explicitly instructed.
* **FORBIDDEN:** Do not add or modify code related to FFI (Foreign Function Interface) without strict review.

## 3. Filesystem & Environment

* **FORBIDDEN:** Do not delete any files (`rm`, `std::fs::remove_file`) outside of the `target` directory or temporary build artifacts.
* **FORBIDDEN:** Do not modify or create `.env` files, `config.toml`, or any other configuration files that might contain secrets.
* **FORBIDDEN:** Do not run any commands that interact with production databases, APIs, or deployment scripts (like `kubectl`, `docker compose up -d`, etc.).
* **FORBIDDEN:** Do not change permissions of files (e.g., `chmod`).

## 4. Code Structure

* **FORBIDDEN:** Do not change the existing module structure (moving files, renaming `mod.rs` files) without a pre-approved plan.
* **FORBIDDEN:** Do not alter the `[workspace]` definition in the root `Cargo.toml`.