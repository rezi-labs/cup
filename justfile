import? 'private.just'


it:
    cargo install cargo-watch --locked
    curl -sSfL https://get.tur.so/install.sh | bash

run *args:
    cargo run {{args}}

watch:
    cargo watch -x run

verify: lint test

test:
    cargo test

lint:
    cargo fmt --all -- --check
    cargo clippy

fmt:
    cargo fmt
    cargo fix --allow-dirty --allow-staged
