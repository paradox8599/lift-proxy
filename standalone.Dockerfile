FROM nixos/nix:2.28.0 AS builder

WORKDIR /app

COPY . .

# hadolint ignore=SC2046
RUN nix build \
    --extra-experimental-features nix-command \
    --extra-experimental-features flakes \
  && mkdir /deps && cp -R $(nix-store -qR result) /deps

FROM alpine:3 AS runner

RUN apk add --no-cache ca-certificates=20241121-r1 curl=8.12.1-r1

WORKDIR /app
COPY --from=builder /deps /nix/store
COPY --from=builder /app/result /app

CMD ["/app/bin/lift-proxy"]
