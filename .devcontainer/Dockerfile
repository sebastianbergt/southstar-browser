# .devcontainer/Dockerfile — Locked-down build container with the pinned Rust toolchain, cargo-deny and the system libraries the browser builds against; build from the repository root.
FROM debian:trixie-slim@sha256:a72b4433de1cf7d6bae0c4cee74d0547f58601a643b3375f070c51600d63b6b2 AS toolchain

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get -y install --no-install-recommends \
        curl ca-certificates \
        git build-essential pkg-config \
        python3 \
        libgtk-4-dev libepoxy-dev libssl-dev zlib1g-dev libbrotli-dev libzstd-dev \
        libuchardet-dev libsqlite3-dev libseccomp-dev libwebp-dev libsdl2-dev \
        libharfbuzz-dev libfontconfig-dev libfreetype-dev \
        libavformat-dev libavcodec-dev libavutil-dev libswscale-dev libswresample-dev && \
    rm -rf /var/lib/apt/lists/*

ARG USER_UID=1000
ARG USER_GID=1000
ARG USERNAME=vscode
RUN groupadd --gid $USER_GID $USERNAME \
    && useradd --uid $USER_UID --gid $USER_GID -m -s /bin/bash $USERNAME \
    && passwd -l root

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

COPY rust-toolchain.toml /tmp/toolchain/
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --no-modify-path --profile minimal --default-toolchain none && \
    cd /tmp/toolchain && \
    rustup toolchain install && \
    rustup default "$(sed -n 's/^channel *= *"\(.*\)"/\1/p' rust-toolchain.toml)" && \
    rustc --version && cargo clippy --version && cargo fmt --version && \
    rm -rf /tmp/toolchain && \
    chown -R $USER_UID:$USER_GID $RUSTUP_HOME $CARGO_HOME

RUN find / -xdev -type f \( -perm -4000 -o -perm -2000 \) -exec chmod a-s {} +

ENV SHELL=/bin/bash
USER $USERNAME
WORKDIR /home/$USERNAME

FROM toolchain AS fuzz
RUN cargo install --locked cargo-fuzz@0.13.2 && \
    rm -rf $CARGO_HOME/registry $CARGO_HOME/git
ENV RUSTC_BOOTSTRAP=1

FROM toolchain
RUN cargo install --locked cargo-deny@0.19.9 && \
    rm -rf $CARGO_HOME/registry $CARGO_HOME/git
