# Please: keep ALL_KNOWN_FEATURES constant updated
ALL_KNOWN_FEATURES := 

JSON_Schema_Test_Suite := JSON-Schema-Test-Suite/.git

# Usage: $(call call_on_all_features,makefile_target)
define call_all_features
set -eux && \
    ( \
        for cargo_args in "" --no-default-features --all-features; do CARGO_ARGS="${CARGO_ARGS} $${cargo_args}" ${MAKE} $(1); done; \
        for cargo_args in ${ALL_KNOWN_FEATURES}; do CARGO_ARGS="${CARGO_ARGS} --features $${cargo_args}" ${MAKE} $(1); done \
    )
endef

ifndef RUST_TOOLCHAIN
RUST_TOOLCHAIN := $(shell cat ${CURDIR}/rust-toolchain | head -n 1)
ifeq ("${RUST_TOOLCHAIN}","")
RUST_TOOLCHAIN := stable
endif
endif
export RUST_TOOLCHAIN

ifndef CONTAINER_COMMAND
CONTAINER_COMMAND := bash
endif

ifndef PRE_COMMIT_BIN
    PRE_COMMIT_BIN := .tox/pre-commit/bin/pre-commit
endif

ifndef CARGO_HOME
CARGO_HOME := ${HOME}/.cargo/
endif

.PHONY: development
development: ${PRE_COMMIT_BIN} ${JSON_Schema_Test_Suite}
	@true

${JSON_Schema_Test_Suite}:
	git submodule update --init

${PRE_COMMIT_BIN}:
	tox -e pre-commit -- install --overwrite --install-hooks

.PHONY: pre-commit
pre-commit: ${PRE_COMMIT_BIN}
	${PRE_COMMIT_BIN} run --all-files

.PHONY: clippy
clippy:
	touch src/lib.rs   # touch a file of the rust project to "force" cargo to recompile it so clippy will actually run
	cargo +${RUST_TOOLCHAIN} clippy --all-targets ${CARGO_ARGS} -- -D clippy::pedantic -D clippy::nursery

.PHONY: lint
lint: pre-commit
	$(call call_all_features,clippy)

.PHONY: build
build:
	cargo +${RUST_TOOLCHAIN} build --all-targets ${CARGO_ARGS}

.PHONY: tests
tests:
	cargo +${RUST_TOOLCHAIN} test --all-targets ${CARGO_ARGS} -- --nocapture

.PHONY: doc
doc:
	cargo +${RUST_TOOLCHAIN} doc ${CARGO_ARGS}

.PHONY: build-all-flavours
build-all-flavours:
	$(call call_all_features,build)

.PHONY: build-release
build-release:
	CARGO_ARGS="${CARGO_ARGS} --release" ${MAKE} build

.PHONY: build-release-all-flavours
build-release-all-flavours:
	$(call call_all_features,build-release)

.PHONY: tests-all-flavours
tests-all-flavours:
	$(call call_all_features,tests)

.PHONY: expand-macros
expand-macros:
	cargo +${RUST_TOOLCHAIN} rustc --tests --all-features -- -Z external-macro-backtrace -Z unstable-options --pretty=expanded

.PHONY: coverage
coverage:
	find ${CURDIR}/target/debug/ -name *jsonschema-validator* -type f -executable | xargs rm -rf
	CARGO_ARGS="--tests" ${MAKE} build-all-flavours
	find target/ -maxdepth 2 -type f -executable | while read executable; do \
	echo "Run $${executable}" > /dev/stderr && \
	mkdir -p ${CURDIR}/.coverage/$$(basename $${executable}) &&  \
	kcov  --exclude-pattern=${CARGO_HOME} --verify ${CURDIR}/.coverage/$$(basename $${executable}) $${executable}; \
	done

REPO_NAME := $(shell basename ${CURDIR} | tr A-Z a-z)
GIT_SHA := $(shell git rev-parse HEAD || echo "no-sha")
DOCKER_PREFIX := ${REPO_NAME}

.PHONY: build-docker-container
build-docker-container:
	docker build -t ${REPO_NAME}:${GIT_SHA} .

.PHONY: clean
clean: clean-coverage
	rm -rf ${CURDIR}/target/

.PHONY: clean-coverage
clean-coverage:
	rm -rf ${CURDIR}/.coverage || ( \
		make build-docker-container && \
		docker run \
			--rm \
			--volume ${CURDIR}:/code \
			${REPO_NAME}:${GIT_SHA} \
			bash -c "make clean-coverage" \
	)

.PHONY: docker-volumes
docker-volumes:
	docker volume create ${DOCKER_PREFIX}_registry
	docker volume create ${DOCKER_PREFIX}_target
	docker volume create ${DOCKER_PREFIX}_sccache

.PHONY: clean-docker-volumes
clean-docker-volumes:
	docker volume rm ${DOCKER_PREFIX}_registry ${DOCKER_PREFIX}_target ${DOCKER_PREFIX}_sccache

.coverage:
	mkdir -p ${CURDIR}/.coverage

.PHONY: update
update:
	cargo +${RUST_TOOLCHAIN} upgrade
	cargo +${RUST_TOOLCHAIN} update
	git diff --quiet Cargo.toml || (git add -p Cargo.toml && git commit -m 'Bump dependencies via `cargo update`')

Cargo.lock: update
	@true

.PHONY: coverage-in-container
coverage-in-container: clean-coverage .coverage Cargo.lock
	CONTAINER_COMMAND='bash -c "sccache --start-server && make coverage && sccache --show-stats"' ${MAKE} start-container

.PHONY: coverage-in-container
start-container: build-docker-container docker-volumes
	docker run \
		--env RUST_BACKTRACE=full \
		--env RUSTC_WRAPPER=sccache \
		--interactive \
		--privileged \
		--rm \
		--volume ${CURDIR}/.coverage:/code/.coverage \
		--volume ${CURDIR}/Cargo.toml:/code/Cargo.toml:ro \
		--volume ${CURDIR}/Cargo.lock:/code/Cargo.lock:ro \
		--volume ${CURDIR}/Makefile:/code/Makefile:ro \
		--volume ${CURDIR}/src/:/code/src/:ro \
		--volume ${CURDIR}/rust-toolchain:/code/rust-toolchain:ro \
		--volume ${CURDIR}/test-data:/code/test-data:ro \
		--volume ${DOCKER_PREFIX}_registry:/root/.cargo/registry \
		--volume ${DOCKER_PREFIX}_target:/code/target \
		--volume ${DOCKER_PREFIX}_sccache:/.sccache \
		--tty \
		${REPO_NAME}:${GIT_SHA} \
		${CONTAINER_COMMAND}

.PHONY: bump-submodules
bump-submodules:
	bash scripts/bump-all-submodules.sh
