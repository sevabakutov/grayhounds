# 🎨 CODESTYLE & DESIGN PRINCIPLES (Rust)

This document defines the code style, architecture, and design philosophy for this project. The agent **MUST** adhere to these principles at all times.

---

## 1. The Golden Rule: Tooling is Truth

Code style is not subjective; it is enforced by tools.

* **`rustfmt`:** All code **MUST** be formatted using `rustfmt`. Run `cargo fmt` before submitting any code. The `rustfmt.toml` file (if present) is the single source of truth for formatting.
* **`clippy`:** This is our primary linter. All code **MUST** pass `cargo clippy` (with the default rules in `Cargo.toml` or `clippy.toml`).
    * **Guideline:** Treat `clippy` warnings as errors.
    * **Reference:** You should be familiar with the available Clippy lints (especially `pedantic`), as we aim to follow them. [Official Clippy Lints List](https://rust-lang.github.io/rust-clippy/master/).
    * **Rule:** Do not use `#[allow(...)]` to silence a `clippy` warning unless you provide a comment explaining *why* the lint is a false positive in this specific case.

---

## 2. Design Paradigm: Trait-Oriented SOLID

While not a traditional OOP language, our codebase **MUST** follow the SOLID principles, as translated into Rust idioms. We prioritize composition over inheritance.

* **S (Single Responsibility Principle - SRP):**
    * A `struct` should encapsulate one primary piece of state.
    * A `mod` should represent a single feature or domain.
    * A `fn` should do one logical thing.

* **O (Open/Closed Principle - OCP):**
    * **Use Traits heavily.** This is the core of OCP in Rust.
    * Functionality should be extended by implementing existing traits for new structs, or by creating new traits.
    * **Do not** modify existing, working structs/functions to add new behavior; extend them with traits.

* **L (Liskov Substitution Principle - LSP):**
    * Functions should depend on `Box<dyn Trait>` or generic `T: Trait`, not on concrete structs.
    * Any implementation of a trait **MUST** be usable wherever the trait is required, without causing panics or unexpected behavior.

* **I (Interface Segregation Principle - ISP):**
    * **Prefer small, focused traits.**
    * Avoid "God Traits" that have dozens of methods.
    * Example: Prefer `trait Reader` and `trait Writer` over a single `trait IO { ... }`.

* **D (Dependency Inversion Principle - DIP):**
    * Depend on abstractions (traits), not concretions (structs).
    * **Example (Good):** `fn process_data(repository: &impl UserRepository)`
    * **Example (Bad):** `fn process_data(repository: &PostgresRepository)`
    * Use dependency injection (e.g., passing trait objects or generics into a struct's constructor) to manage dependencies.

---

## 3. Core Rust Idioms

* **Error Handling:**
    * **NEVER** use `.unwrap()` or `.expect()` in library code or any code that can recover.
    * `.expect()` is only allowed in tests or during initial app setup (e.g., in `main.rs`) where a panic is the desired outcome.
    * **MUST** use `anyhow::Result<T>` for all fallible operations.
    * Use the `anyhow` crate for error propagation in application/binary code.

* **Data Structures:**
    * Use `structs` for complex data.
    * Use `enums` to represent state, choices, or variations (e.g., `enum State { Loading, Success(Data), Failed(Error) }`).
    * Embrace the `Option<T>` type. Do not use "magic values" (like `-1` or `null` strings) to represent "nothing."

* **Naming Conventions:**
    * **`PascalCase`:** Structs, Enums, Traits (e.g., `MyStruct`, `RequestStatus`).
    * **`snake_case`:** Functions, variables, module names (e.g., `fn get_user`, `let user_count`).
    * **`SCREAMING_SNAKE_CASE`:** Constants, statics (e.g., `const MAX_CONNECTIONS`).

* **Documentation:**
    * All `pub` items (functions, structs, traits, enums, modules) **MUST** have doc comments (`///`).
    * Use `//` for internal implementation comments.
    * Don't write doc tests (and don't run them if some already are existed in project)