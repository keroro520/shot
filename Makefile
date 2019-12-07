VERBOSE := $(if ${CI},--verbose,)
CLIPPY_OPTS := -D warnings -D clippy::clone_on_ref_ptr -D clippy::enum_glob_use -D clippy::fallible_impl_from

##@ Testing
test: ## Run all tests.
	cargo test ${VERBOSE} --all -- --nocapture

##@ Building
check: 
	cargo check ${VERBOSE} --all --all-targets

build: ## Build binary with release profile.
	cargo build ${VERBOSE} --release

##@ Code Quality
fmt:
	cargo fmt ${VERBOSE} --all -- --check

clippy:
	cargo clippy ${VERBOSE} --all --all-targets --all-features -- ${CLIPPY_OPTS}
