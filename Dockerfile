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

RUN sh script.sh

FROM gcr.io/distroless/cc as final
COPY --from=build /output/silly-cors /app

ENTRYPOINT [ "/app/silly-cors" ]