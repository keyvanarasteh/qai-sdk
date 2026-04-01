# QAI SDK — Agent Rules

## Code Style

- **No `unwrap()` in library code** — always use `?` or proper error handling
- **No `println!` in library code** — use return values or logging
- **Snake case** for files: `my_module.rs`, `my_example.rs`
- **All public items must have doc comments** with `///`
- **Use `#[derive(Debug, Clone)]`** on all public types where applicable
- **Use `serde(skip_serializing_if = "Option::is_none")`** for optional API fields

## Error Handling

- Return `Result<T, ProviderError>` from all provider operations
- Map HTTP 429 → `ProviderError::RateLimit`
- Map HTTP 401/403 → `ProviderError::Unauthorized`
- Map `reqwest::Error` → `ProviderError::Network`
- Map `serde_json::Error` → `ProviderError::InvalidResponse`
- Use `ProviderError::NotSupported` for unimplemented features

## Architecture Rules

- Every provider implements `LanguageModel` trait at minimum
- Providers are feature-gated: `#[cfg(feature = "my-provider")]`
- Factory functions follow naming: `create_<provider>(ProviderSettings) -> Model`
- Provider-specific types are `pub(crate)` — not leaked to consumers
- The `prelude` module re-exports all common types

## Import Conventions

```rust
// Always use the prelude for consumer-facing code
use qai_sdk::prelude::*;

// Use specific imports for internal modules
use crate::core::types::*;
use crate::core::error::ProviderError;
```

## Testing Requirements

- Use `#[tokio::test]` for async tests
- Mock external APIs — never call real endpoints in unit tests
- Test both success and error paths
- Use `MockLanguageModel` from `test_utils` for trait-level tests

## API Design Principles

- **Provider-agnostic**: All core types work across all providers
- **Zero-cost abstractions**: Traits compiled away at monomorphization
- **Async-first**: All operations are async, using `tokio` runtime
- **Stream-native**: Streaming is first-class, not bolted on
- **Composable**: Registry, middleware, and agent layer on top of base traits

## Documentation Rules

- Every new module gets a doc page in `docs/<module>.md`
- Every doc page has a cover image in `assets/<module>_cover.png`
- Code examples must be complete and compilable
- Mermaid diagrams for architecture flows
- Update `README.md` feature table when adding features
- Update `AGENTS.md` import table when adding new public types
