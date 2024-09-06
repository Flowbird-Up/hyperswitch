FROM rust:latest

ARG SOURCE_DIRECTORY=../..

RUN cargo install diesel_cli --no-default-features --features postgres && cargo install just

# Copying codebase from current dir to /app dir
# and creating a fresh build
WORKDIR /app
COPY $SOURCE_DIRECTORY/migrations ./migrations
COPY $SOURCE_DIRECTORY/v2_migrations ./v2_migrations
COPY $SOURCE_DIRECTORY/crates/diesel_models/ ./crates/diesel_models
COPY $SOURCE_DIRECTORY/diesel.toml .
COPY $SOURCE_DIRECTORY/diesel_v2.toml .
COPY $SOURCE_DIRECTORY/justfile .
