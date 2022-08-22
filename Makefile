VERSION=$(shell grep ^version Cargo.toml|cut -d\" -f2)

all: test

test: test-std test-nostd test-nostd-smallcontext

test-std:
	cargo test -- --test-threads=1 --nocapture

test-nostd:
	cargo test --features nostd -- --test-threads=1 --nocapture

test-nostd-smallcontext:
	cargo test --features "nostd smallcontext" -- --test-threads=1 --nocapture

tag:
	git tag -a v${VERSION} -m v${VERSION}
	git push origin --tags

doc:
	cargo doc

pub: doc test publish-cargo-crate

publish-cargo-crate:
	cargo publish
