---
description: How to publish a new version to crates.io via GitHub Actions
---

# Publish Release

## Steps

1. Bump version in `Cargo.toml`:
```toml
version = "0.1.5"  # Increment appropriately
```

2. Sync `Cargo.lock`:
```bash
// turbo
cargo check
```

3. Commit the version bump:
```bash
git add Cargo.toml Cargo.lock
git commit -m "release: v0.1.5 — description of changes"
```

4. Tag and push:
```bash
git tag v0.1.5
git push && git push --tags
```

5. The GitHub Actions release pipeline (`.github/workflows/release.yml`) will automatically:
   - Run `cargo publish --token $CARGO_REGISTRY_TOKEN --allow-dirty`
   - Create a GitHub Release with auto-generated release notes

## Prerequisites
- `CARGO_REGISTRY_TOKEN` must be set in GitHub repository secrets
- The release workflow triggers on `v*` tag pushes

## Versioning Convention
- `patch` (0.1.x): Bug fixes, doc updates, non-breaking additions
- `minor` (0.x.0): New features, new providers, API additions
- `major` (x.0.0): Breaking API changes

## Notes
- Always run `cargo check` before committing to sync `Cargo.lock`
- The `--allow-dirty` flag in the workflow handles CI-generated lockfile differences
- Monitor the pipeline at https://github.com/keyvanarasteh/qai-sdk/actions
