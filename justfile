DEFAULT: ci
cargo := "cargo"

ci: check test fmt clippy audit

build:
    {{cargo}} build

check:
    {{cargo}} check

test:
    {{cargo}} test

fmt:
    {{cargo}} fmt --all -- --check

clippy:
    {{cargo}} clippy -- -D warnings

audit:
    {{cargo}} audit
