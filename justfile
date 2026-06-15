drun_pid1 := "docker run --rm --name pid pid1runner"

# List all recipies
default:
    just --list --unsorted

# Build pid binary
build-release-binary:
    cargo build --bin pid1 --release --target {{ arch() }}-unknown-linux-musl

# Build test container
test: build-release-binary
    cp target/{{ arch() }}-unknown-linux-musl/release/pid1 ./pid1-exe/etc/
    docker build --tag pid1runner ./pid1-exe/etc/

# Test docker image
test-init-image:
    {{ drun_pid1 }} ps aux
    {{ drun_pid1 }} ls
    {{ drun_pid1 }} ls /
    {{ drun_pid1 }} id
    {{ drun_pid1 }} --workdir=/home  pwd
    {{ drun_pid1 }} --env HELLO=WORLD --env=FOO=BYE printenv HELLO FOO

# Exec init image
exec-init-image:
    docker run --rm -it --name pid pid1runner sh

# Build binary for other architectures
binaries clean='false':
    cross build --bin pid1 --release --target x86_64-unknown-linux-gnu
    -{{ clean }} && docker image rm ghcr.io/cross-rs/x86_64-unknown-linux-gnu:0.2.5
    cross build --bin pid1 --release --target aarch64-unknown-linux-gnu
    -{{ clean }} && docker image rm ghcr.io/cross-rs/aarch64-unknown-linux-gnu:0.2.5
    cross build --bin pid1 --release --target aarch64-unknown-linux-musl
    -{{ clean }} && docker image rm ghcr.io/cross-rs/aarch64-unknown-linux-musl:0.2.5

# Copy binaries to artifacts directory
cp-binaries:
    mkdir -p artifacts
    cp target/x86_64-unknown-linux-musl/release/pid1  ./artifacts/pid1-x86_64-unknown-linux-musl
    cp target/x86_64-unknown-linux-gnu/release/pid1 ./artifacts/pid1-x86_64-unknown-linux-gnu
    cp target/aarch64-unknown-linux-gnu/release/pid1 ./artifacts/pid1-aarch64-unknown-linux-gnu
    cp target/aarch64-unknown-linux-musl/release/pid1 ./artifacts/pid1-aarch64-unknown-linux-musl
    file artifacts/*

# Lint
lint:
	cargo clippy -- --deny "warnings"
	cargo fmt -- --check
