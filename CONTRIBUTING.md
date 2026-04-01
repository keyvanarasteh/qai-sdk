# Contributing to QAI SDK

Thank you for your interest in contributing! This guide will help you get started.

## Getting Started

1. Fork and clone the repository
2. Copy `.env.example` to `.env` and fill in your API keys
3. Run `cargo build --workspace` to verify everything compiles
4. Run `cargo test --workspace` to run the test suite

## Adding a New Provider

QAI SDK is designed to be modular. Each provider lives in its own crate.

1. Create a new crate: `cargo init qai-yourprovider --lib`
2. Add it to `[workspace.members]` in the root `Cargo.toml`
3. Implement the core traits from `qai-core`:
   - `LanguageModel` (required) — chat and streaming
   - `EmbeddingModel`, `ImageModel`, etc. (optional)
4. Create a provider factory function: `pub fn create_yourprovider(settings: ProviderSettings) -> YourProvider`
5. Re-export from the root `qai-sdk` crate in `src/lib.rs`
6. Add an example in `examples/`

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy --workspace` and fix all warnings
- Add doc comments (`///`) to all public items
- Follow existing patterns in other provider crates

## Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p qai-openai

# Run a specific example (requires API keys in .env)
cargo run --example chat_basic
```

## Pull Request Process

1. Create a feature branch from `main`
2. Make your changes with clear commit messages
3. Ensure `cargo fmt`, `cargo clippy`, and `cargo test` all pass
4. Open a PR with a description of your changes

## License

By contributing, you agree that your contributions will be licensed under the same dual MIT/Apache-2.0 license as the project.
