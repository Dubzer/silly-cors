ARG RUST_VERSION=1.72.1

FROM --platform=${BUILDPLATFORM} ghcr.io/blackdex/rust-musl:aarch64-musl-stable-${RUST_VERSION}-openssl3 as build-arm64
FROM --platform=${BUILDPLATFORM} ghcr.io/blackdex/rust-musl:x86_64-musl-stable-${RUST_VERSION}-openssl3 as build-amd64

FROM build-${TARGETARCH} as build
WORKDIR /src
COPY . .
RUN cargo build --release

RUN mkdir /output && \
    cd ./target && \
    mv ./$(ls -d */|grep musl)/release/silly-cors /output


FROM gcr.io/distroless/static
WORKDIR /app
COPY --from=build /output/silly-cors .

CMD ["/app/silly-cors"]

EXPOSE 3001