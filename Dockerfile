FROM --platform="${BUILDPLATFORM}" rust:1.72.1-slim as build
WORKDIR /src

COPY . .
COPY scripts/build-doker.sh ./script.sh

ARG TARGETARCH
RUN dpkg --add-architecture "${TARGETARCH}"
RUN apt-get update && \
    apt-get install -y \
    make \
    pkg-config \
    libssl-dev:"${TARGETARCH}"

RUN bash script.sh


FROM debian:bookworm-slim as final
RUN apt update && apt upgrade -y && \
    apt install openssl -y && \
    apt clean

COPY --from=build /output/silly-cors /app/silly-cors

ENTRYPOINT [ "/app/silly-cors" ]