# syntax=docker/dockerfile:1

FROM rust:slim-buster

# install deps
RUN apt-get update && apt-get install -y git sqlite3 libsqlite3-dev

# setup ssh key
RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts

# get sources
RUN mkdir -p /src
RUN --mount=type=ssh git clone git@github.com:krepa098/smart-meter.git /src/smart-meter --depth=1
RUN --mount=type=ssh cd /src/smart-meter && git submodule update --init --recursive

# diesel cli
RUN cargo install diesel_cli --no-default-features --features sqlite --locked 

# trunk
RUN cargo install --locked trunk

# add scripts
RUN mkdir -p /usr/local/scripts
ADD scripts /usr/local/scripts

# build backend
RUN mkdir -p /usr/local/bin && \
    cd /src/smart-meter/backend && \ 
    cargo b --release && \
    mv target/release/backend ../backend && \
    rm -rf target && \
    cd / && \
    mv /src/smart-meter/backend /usr/local/bin/backend

# build frontend
RUN rustup target add wasm32-unknown-unknown
RUN cd /src/smart-meter/frontend && \
    trunk build --release && \
    cd / && \
    mv /src/smart-meter/frontend /usr/local/bin/frontend && \
    rm -rf /src

# backend.broadcast
EXPOSE 8989/udp

# backend.api
EXPOSE 8081

# frontend
EXPOSE 80
EXPOSE 443

# entrypoint
ENTRYPOINT ["/usr/local/scripts/run.sh"]
