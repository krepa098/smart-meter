# syntax=docker/dockerfile:1

# ===============================================================================
# BUILDER
# ===============================================================================
FROM rust:slim-buster AS builder

# install deps
RUN apt-get update && apt-get install -y git sqlite3 libsqlite3-dev

# setup ssh key
RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts

# get sources
RUN mkdir -p /src
RUN --mount=type=ssh git clone git@github.com:krepa098/smart-meter.git /src/smart-meter --depth=1
RUN --mount=type=ssh cd /src/smart-meter && git submodule update --init --recursive

# trunk
RUN cargo install --root=/usr/local --locked trunk

# diesel cli
RUN cargo install diesel_cli --root=/usr/local --no-default-features --features sqlite --locked

# add scripts
RUN mkdir -p /usr/local/scripts
ADD scripts /usr/local/scripts

# build backend
WORKDIR /src/smart-meter/backend
RUN cargo b --release
RUN mv target/release/backend backend
RUN rm -rf target

# build frontend
RUN rustup target add wasm32-unknown-unknown
WORKDIR /src/smart-meter/frontend
RUN trunk build --release
    
# ===============================================================================
# RUNNER
# ===============================================================================
FROM rust:slim-buster AS runner

# install deps
RUN apt-get update && apt-get install -y sqlite3 libsqlite3-dev

# diesel cli
COPY --from=builder /usr/local/bin/diesel /usr/local/bin/diesel

# trunk
COPY --from=builder /usr/local/bin/trunk /usr/local/bin/trunk

# add scripts
RUN mkdir -p /usr/local/scripts
ADD scripts /usr/local/scripts

# copy builds
RUN mkdir -p /usr/local/bin
COPY --from=builder /src/smart-meter/backend /usr/local/bin/backend
COPY --from=builder /src/smart-meter/frontend /usr/local/bin/frontend

# backend.broadcast
EXPOSE 8989/udp

# backend.api
EXPOSE 8081

# frontend
EXPOSE 80
EXPOSE 443

# entrypoint
ENTRYPOINT ["/usr/local/scripts/run.sh"]
