FROM nixos/nix:2.28.0 AS builder

WORKDIR /app

COPY . .

# hadolint ignore=SC2046
RUN nix build \
  --extra-experimental-features nix-command \
  --extra-experimental-features flakes \
  && mkdir /deps && cp -R $(nix-store -qR result) /deps

FROM debian:bookworm-slim AS runner

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
  \
  ca-certificates=20230311 \
  curl=7.88.1-10+deb12u12 \
  \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /deps /nix/store
COPY --from=builder /app/result /app

CMD ["/app/bin/lift-proxy"]
