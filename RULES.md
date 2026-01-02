# Project Rules

This document outlines the core rules and conventions for the `llm-connector` project.

## 1. Documentation Rules

To maintain a clean and navigable documentation structure:

- **Core Files Only**: Keep documentation consolidated in the `docs/` directory.
  - `docs/README.md`: Index and Quick Start.
  - `docs/PROVIDERS.md`: Unified guide for all providers.
  - `docs/ARCHITECTURE.md`: Technical design and architecture.
  - `docs/CONTRIBUTING.md`: Development guidelines.
  - `docs/MIGRATION.md`: Migration guides.
- **No Scattered Files**: Do not create generic `*_REPORT.md`, `*_SUMMARY.md` or isolated guides in `docs/` or `repro/`. Consolidate them into the core files.
- **No "guides" or "archive" Folders**: Do not re-create `docs/guides/` or `docs/archive/`. Detailed provider usage goes into `docs/PROVIDERS.md`.

## 2. Release Rules

- **Changelog**: All release notes must go into `CHANGELOG.md`.
- **No Root Release Notes**: **DO NOT** create `RELEASE_NOTES_*.md` files in the root directory.
  - Exception: Temporary files for the `gh release create` command are allowed but must be deleted immediately after use.
- **Tagging**: Git tags must match the version in `Cargo.toml`.

## 3. Coding Standards

- **English Only**: All comments, documentation, and commit messages must be in English. No Chinese characters in the codebase.
- **Zero Warnings**: The code must compile with 0 warnings.
- **Cleanups**: Remove unused imports and dead code immediately. Do not leave "commented out" code.

## 4. Scripts

- **Minimal Scripts**: Keep the `scripts/` directory clean. Only essential maintenance scripts (e.g., release helpers) are allowed.
- **No One-Off Scripts**: Do not check in "fix_*" or "test_*" scripts meant for single-use debugging.

