PRE_COMMIT_BIN := .tox/pre-commit/bin/pre-commit
JSON_Schema_Test_Suite := JSON-Schema-Test-Suite/.git

.PHONY: development
development: ${PRE_COMMIT_BIN} ${JSON_Schema_Test_Suite}
	@true

${JSON_Schema_Test_Suite}:
	git submodule update --init

${PRE_COMMIT_BIN}:
	tox -e pre-commit -- install --overwrite --install-hooks

.PHONY: clean
clean:
	rm -rf .tox/ target/

.PHONY: lint
lint: ${PRE_COMMIT_BIN}
	${PRE_COMMIT_BIN} run --all-files
	touch src/lib.rs   # touch a file of the rust project to "force" cargo to recompile it so clippy will actually run
	cargo +nightly clippy --all-targets --all-features -- -D clippy::pedantic -D clippy::nursery

.PHONY: test
test:
	cargo test

.PHONY: build-release
build-release:
	cargo build --release

.PHONY: build
build:
	cargo build

.PHONY: expand-macros
expand-macros:
	cargo +nightly rustc --all-features -- -Z external-macro-backtrace -Z unstable-options --pretty=expanded
