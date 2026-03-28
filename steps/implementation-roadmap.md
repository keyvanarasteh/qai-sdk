# Implementation Roadmap

This roadmap outlines the steps to port the AI providers from TypeScript to Rust.

## Phase 1: Infrastructure Setup
- [x] Initialize Cargo workspace in `qai-sdk` [x]
- [x] Create common utility crate `qai-core` if needed [x]

## Phase 2: Provider Conversion (One by One)
### Anthropic
- [x] Create `qai-anthropic` crate [x]
- [x] Port types and API client logic [x]
- [x] Implement unit tests [x]
- [x] Verify compilation and tests [x]

### Deepseek
- [x] Create `qai-deepseek` crate [x]
- [x] Port types and API client logic [x]
- [x] Implement unit tests [x]
- [x] Verify compilation and tests [x]

### Google
- [x] Create `qai-google` crate [x]
- [x] Port types and API client logic [x]
- [x] Implement unit tests [x]
- [x] Verify compilation and tests [x]

### OpenAI
- [x] Create `qai-openai` crate [x]
- [x] Port types and API client logic [x]
- [x] Implement unit tests [x]
- [x] Verify compilation and tests [x]

### OpenAI Compatible
- [x] Create `qai-openai-compatible` crate [x]
- [x] Port types and API client logic [x]
- [x] Implement unit tests [x]
- [x] Verify compilation and tests [x]

### xAI
- [x] Create `qai-xai` crate [x]
- [x] Port types and API client logic [x]
- [x] Implement unit tests [x]
- [x] Verify compilation and tests [x]

## Phase 3: Final Verification
- [x] Run workspace-wide checks [x]
- [x] Ensure all crates are correctly linked and tested [x]
