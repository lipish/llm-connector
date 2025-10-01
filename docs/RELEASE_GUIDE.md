# Release Guide for llm-connector v0.2.0

## Pre-Release Checklist

### ✅ Code Quality
- [x] All unit tests passing (35/35)
- [x] All integration tests passing (5/5)
- [x] All examples compile successfully
- [x] Release build successful
- [x] No compiler warnings in release mode

### ✅ Documentation
- [x] CHANGELOG.md updated with v0.2.0 changes
- [x] README.md is up to date
- [x] All new features documented
- [x] Migration guide included in CHANGELOG

### ✅ Version Updates
- [x] Cargo.toml version updated to 0.2.0
- [x] CHANGELOG.md includes v0.2.0 section

### ✅ Git Status
- [x] All changes committed
- [x] Working directory clean (or use --allow-dirty)

## Release Steps

### Step 1: Login to crates.io

If you haven't logged in yet:

```bash
cargo login
```

You'll need your API token from https://crates.io/me

### Step 2: Dry Run

Test the package without actually publishing:

```bash
cargo publish --dry-run --allow-dirty
```

This will:
- Build the package
- Check for errors
- Show what would be published
- NOT actually publish

### Step 3: Publish to crates.io

Once the dry run succeeds:

```bash
cargo publish --allow-dirty
```

Or if your working directory is clean:

```bash
cargo publish
```

### Step 4: Create Git Tag

After successful publication:

```bash
git tag -a v0.2.0 -m "Release v0.2.0 - Type Safety and Ergonomic API Improvements"
git push origin v0.2.0
```

### Step 5: Create GitHub Release

1. Go to https://github.com/lipish/llm-connector/releases/new
2. Choose tag: v0.2.0
3. Release title: "v0.2.0 - Type Safety and Ergonomic API Improvements"
4. Copy content from CHANGELOG.md for v0.2.0
5. Publish release

## What's New in v0.2.0

### Major Improvements

1. **Type Safety**
   - New `Role` enum for compile-time validation
   - Prevents invalid role strings
   - Better IDE support

2. **Ergonomic API**
   - Message constructors: `Message::user()`, `Message::system()`, etc.
   - ChatRequest builder pattern
   - ToolChoice constructors
   - 70-85% less boilerplate code

3. **Bug Fixes**
   - Fixed ToolChoice serialization bug
   - Correct JSON output for all variants

### Breaking Changes

- `Message.role`: `String` → `Role` enum
- `Delta.role`: `Option<String>` → `Option<Role>`

### Migration Path

Simple migration with constructors:

```rust
// Old
let msg = Message {
    role: "user".to_string(),
    content: "Hello".to_string(),
    ..Default::default()
};

// New
let msg = Message::user("Hello");
```

## Post-Release

### Verify Publication

1. Check crates.io: https://crates.io/crates/llm-connector
2. Verify version shows as 0.2.0
3. Check documentation: https://docs.rs/llm-connector/0.2.0

### Announce

Consider announcing on:
- GitHub Discussions
- Reddit r/rust
- Twitter/X
- Your blog

### Monitor

- Watch for issues on GitHub
- Monitor crates.io download stats
- Check docs.rs build status

## Troubleshooting

### "working directory is dirty"

Use `--allow-dirty` flag:
```bash
cargo publish --allow-dirty
```

### "failed to verify package"

This usually means tests failed. Run:
```bash
cargo test --all
```

### "crate name already exists"

If you need to unpublish (within 72 hours):
```bash
cargo yank --vers 0.2.0
```

Then fix issues and republish.

### "authentication required"

Login again:
```bash
cargo login
```

## Rollback Plan

If critical issues are found after release:

1. **Yank the version** (within 72 hours):
   ```bash
   cargo yank --vers 0.2.0
   ```

2. **Fix the issues**

3. **Release patch version** (0.2.1):
   - Update Cargo.toml to 0.2.1
   - Update CHANGELOG.md
   - Republish

## Success Criteria

- ✅ Package published to crates.io
- ✅ Version 0.2.0 visible on crates.io
- ✅ Documentation built on docs.rs
- ✅ Git tag created and pushed
- ✅ GitHub release created
- ✅ No critical issues reported within 24 hours

## Notes

- **Yanking**: Can only yank within 72 hours of publication
- **Unpublishing**: Cannot completely remove a version after 72 hours
- **Semver**: Follow semantic versioning strictly
- **Breaking changes**: Require major version bump (0.x.0 → 1.0.0)

## Contact

If you encounter issues during release:
- Open an issue on GitHub
- Check cargo documentation: https://doc.rust-lang.org/cargo/
- Ask on Rust Discord: https://discord.gg/rust-lang

