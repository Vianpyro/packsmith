# Packsmith dev image. Rust plus the tools `cargo xtask ci` and the CI
# workflow call. Deliberately small: no Node, no Java - the block SDKs and
# the web editor arrive in Phase 3/4 and get their own layers then.
FROM mcr.microsoft.com/devcontainers/base:bookworm

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get install -y --no-install-recommends \
        build-essential pkg-config libssl-dev \
        curl git jq unzip zip \
    && rm -rf /var/lib/apt/lists/*

USER vscode

# MSRV is pinned in Cargo.toml (rust-version), not here: a rust-toolchain
# file would force every contributor onto the floor version for daily work.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --default-toolchain stable --component rustfmt --component clippy
ENV PATH=/home/vscode/.cargo/bin:$PATH

# binstall fetches prebuilt binaries instead of compiling for minutes.
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
      https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
    && cargo binstall -y cargo-deny cargo-machete typos-cli
