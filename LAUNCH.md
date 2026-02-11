# Launch Plan

## Before Launch

- [ ] Delete `docs/internal/` folder
- [ ] Commit and push **all** pending changes (Cargo.toml metadata, cleanup, etc.)
- [ ] Final verification:
  ```bash
  cargo test --all
  cargo clippy --all-targets --all-features -- -D warnings
  cargo build --release --examples
  ```
- [ ] Verify package contents have no secrets/internal files:
  ```bash
  cargo package --list -p telex-macro | less
  cargo package --list -p telex-tui | less
  ```
- [ ] Authenticate with crates.io:
  ```bash
  cargo login
  # Paste your API token from https://crates.io/settings/tokens
  ```
- [ ] Squash history to single commit (**irreversible — do last, after everything above is committed and pushed**):
  ```bash
  git checkout --orphan fresh
  git add -A
  git commit -m "Initial commit"
  git branch -D main
  git branch -m main
  git push origin main --force
  git log --oneline  # Should show exactly 1 commit
  ```

## Launch (10 minutes)

### 1. Make repo public (must be first — crates.io verifies the repo URL)
GitHub → telex-tui/telex-tui → Settings → Danger Zone → Change visibility → Public

### 2. Publish to crates.io
```bash
cargo publish -p telex-macro    # Must be first (it's a dependency)
# Wait for "Uploaded telex-macro v0.2.0"
# Wait ~30s for crates.io to index it, then:
cargo publish -p telex-tui
# Wait for "Uploaded telex-tui v0.2.0"
```

### 3. Enable GitHub Pages
GitHub → Settings → Pages → Source: "GitHub Actions" → Save

### 4. Activate book deployment
Uncomment push triggers in `.github/workflows/book.yml`, then:
```bash
git add .github/workflows/book.yml
git commit -m "Enable automatic book deployment"
git push origin main
```

### 5. Update documentation links
- In `README.md`: add book link (`https://telex-tui.github.io/telex-tui/`)
- In `crates/telex/Cargo.toml`: uncomment `documentation = "..."`
```bash
git add README.md crates/telex/Cargo.toml
git commit -m "Add documentation links"
git push origin main
```

## Verify

- [ ] Repo is public: https://github.com/telex-tui/telex-tui
- [ ] Crate is live: https://crates.io/crates/telex-tui
- [ ] Book loads: https://telex-tui.github.io/telex-tui/
- [ ] Test fresh install:
  ```bash
  cargo new test-telex && cd test-telex
  cargo add telex-tui
  # Copy README example into src/main.rs, cargo run
  ```
- [ ] docs.rs builds (takes 5-10 min): https://docs.rs/crates/telex-tui/builds

## If Something Goes Wrong

| Problem | Fix |
|---------|-----|
| Publish fails | Check repo is public, package < 10MB, deps resolve |
| "crate not found: telex-macro" | Wait 30s for indexing, retry |
| Book shows 404 | Wait 1-2 min, hard refresh, check Actions tab |
| docs.rs build fails | Check build log, fix, trigger rebuild |
| Critical bug published | `cargo yank --vers 0.1.0 -p telex-tui`, publish 0.1.1 |
| Repo public too early | Settings → Danger Zone → Make private |

## Post-Launch

- [ ] Add GitHub Actions CI (tests, clippy, fmt on PRs)
- [ ] Monitor issues for early feedback
- [ ] Post announcement (if planned)
