FROM node:20-bullseye AS ui-builder
WORKDIR /ui
COPY ui/package.json ./
RUN npm install
COPY ui/ .
RUN npm run build

FROM rust:1.75-bullseye AS builder
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libsecret-1-dev \
    && rm -rf /var/lib/apt/lists/*
COPY . .
COPY --from=ui-builder /ui/dist /app/ui/dist
RUN cargo build --release --locked

FROM debian:bullseye-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends libsecret-1-0 ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/jwt-tester /usr/local/bin/jwt-tester
COPY --from=ui-builder /ui/dist /app/ui/dist
ENV JWT_TESTER_KEYCHAIN_BACKEND=file
ENV JWT_TESTER_DOCKER=1
ENV JWT_TESTER_UI_ASSETS_DIR=/app/ui/dist
EXPOSE 3000
VOLUME ["/data"]
ENTRYPOINT ["/usr/local/bin/jwt-tester"]
CMD ["--data-dir","/data","ui","--host","0.0.0.0","--port","3000","--allow-remote"]
