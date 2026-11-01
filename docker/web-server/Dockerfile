# ---------------------------------------------------------- #
# ------------------------ Build---------------------------- #
# ---------------------------------------------------------- #

FROM rust:1.99-trixie AS build
# HiGHS is built from source: cmake for the build, libclang for bindgen.
RUN apt-get update \
    && apt-get install -y --no-install-recommends cmake clang libclang-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY . .
# Cache mounts keep the registry and target/ between builds, so HiGHS is
# compiled once and later builds only recompile the crates that changed.
# target/ is not part of the image, hence the copy out of the mount.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked --package web-server \
    && cp target/release/web-server /usr/local/bin/

# ---------------------------------------------------------- #
# ---------------------- Run time--------------------------- #
# ---------------------------------------------------------- #

FROM debian:trixie-slim
RUN useradd --system golion
COPY --from=build /usr/local/bin/web-server /usr/local/bin/web-server
USER golion
ENV HOST=0.0.0.0
EXPOSE 3000
CMD ["web-server"]
