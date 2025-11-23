.PHONY: help sync-version bump-version check-version

help:
	@echo "Targets:"
	@echo "  sync-version        Sync local Cargo.toml version to crates.io"
	@echo "  bump-version VERSION=x.y.z  Bump local version to x.y.z"
	@echo "  check-version       Compare local vs remote version"
	@echo "  release VERSION=x.y.z        Bump, commit, tag, push and publish"

sync-version:
	bash scripts/version_sync.sh sync

bump-version:
	@if [ -z "$(VERSION)" ]; then echo "ERROR: provide VERSION=x.y.z"; exit 2; fi
	bash scripts/version_sync.sh bump $(VERSION)

check-version:
	bash scripts/version_sync.sh check

release:
	@if [ -z "$(VERSION)" ]; then echo "ERROR: provide VERSION=x.y.z"; exit 2; fi
	bash scripts/release.sh release $(VERSION)