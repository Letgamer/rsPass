FROM rust:alpine as builder

# Install required dependencies for building in Alpine
# musl-dev because of: https://bxbrenden.github.io/
# curl because of: https://crates.io/crates/utoipa-swagger-ui
RUN apk add --no-cache musl-dev curl

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

FROM scratch
COPY --from=builder /usr/src/app/target/release/backend /usr/local/bin/backend

CMD ["/usr/local/bin/backend"]
EXPOSE 8080