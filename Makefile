ifdef TRAVIS_RUST_VERSION
RUST_TOOLCHAIN := ${TRAVIS_RUST_VERSION}
endif

ifndef RUST_TOOLCHAIN
RUST_TOOLCHAIN := $(shell cat ${CURDIR}/rust-toolchain 2> /dev/null | head -n 1)
ifeq ("${RUST_TOOLCHAIN}","")
RUST_TOOLCHAIN := stable
endif
endif
export RUST_TOOLCHAIN

ifndef CODECOV_DIR
CODECOV_DIR := ${CURDIR}
endif

.PHONY: development
development: install-hooks
	git submodule update --init --recursive

.PHONY: install-hooks
install-hooks: .git/hooks/pre-commit
	@true

.git/hooks/pre-commit: venv .pre-commit-config.yaml
	./venv/bin/pre-commit install -f --install-hooks

# Python Virtual Environment needed to allow usage of pre-commit.com
venv:
	virtualenv venv
	venv/bin/pip install --no-cache pre-commit

.PHONY: clean
clean: clean-coverage
	rm -rf target venv

.PHONY: bump-submodules
bump-submodules:
	bash scripts/bump-all-submodules.sh

.PHONY: clippy
clippy:
	touch src/lib.rs   # touch a file of the rust project to "force" cargo to recompile it so clippy will actually run
	cargo +${RUST_TOOLCHAIN} clippy --all-targets ${CARGO_ARGS}

.PHONY: pre-commit
pre-commit: venv
	./venv/bin/pre-commit run --all-files

.PHONY: lint
lint: pre-commit clippy
	@true

.PHONY: build
build:
	cargo +${RUST_TOOLCHAIN} build --all-targets ${CARGO_ARGS}

.PHONY: test
test:
	cargo +${RUST_TOOLCHAIN} test --all-targets ${CARGO_ARGS}

.PHONY: doc
doc:
	cargo +${RUST_TOOLCHAIN} doc --no-deps ${CARGO_ARGS}

${CODECOV_DIR}/codecov.bash:
	mkdir -p ${CODECOV_DIR}
	curl -s https://codecov.io/bash > ${CODECOV_DIR}/codecov.bash

.coverage:
	mkdir -p ${CURDIR}/.coverage

.PHONY: coverage
coverage: export CARGO_INCREMENTAL := 0
coverage: export RUSTFLAGS := ${RUSTFLAGS} -Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off
coverage: clean-coverage ${CODECOV_DIR}/codecov.bash
	@command -v grcov @> /dev/null || (echo "grcov is not yet installed" && cargo install grcov)
	mkdir ${CURDIR}/.coverage
	${MAKE} test
	find ${CURDIR}/target -name "*$$(bash scripts/cargo-project-name.sh)*.gc*" | xargs zip -0 ${CURDIR}/.coverage/ccov.zip
	grcov ${CURDIR}/.coverage/ccov.zip \
		--source-dir ${CURDIR} \
		--output-type lcov \
		--llvm \
		--ignore "/*" \
		--ignore-not-existing \
		--output-file ${CURDIR}/.coverage/lcov.info
	@[ "${TRAVIS}" = "true" ] && \
		bash ${CODECOV_DIR}/codecov.bash -f ${CURDIR}/.coverage/lcov.info || \
		echo "Skip codecov uploads"

coverage-html: coverage
	command -v genhtml @> /dev/null || (echo "genhtml is not yet installed. Please install lcov (https://github.com/linux-test-project/lcov) tools"; exit 1)
	genhtml -o ${CURDIR}/.coverage/report/ --show-details --highlight --ignore-errors source --legend ${CURDIR}/.coverage/lcov.info

.PHONY: clean-coverage
clean-coverage:
	rm -rf ${CURDIR}/.coverage

.PHONY: expand-macros
expand-macros:
	cargo +${RUST_TOOLCHAIN} rustc --tests --all-features -- -Z external-macro-backtrace -Z unstable-options --pretty=expanded
