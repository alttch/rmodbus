VERSION=0.5.3

all: test

test: test-std test-nostd test-nostd-smallcontext

test-std:
	cargo test -- --test-threads=1 --nocapture

test-nostd:
	cargo test --features nostd -- --test-threads=1 --nocapture

test-nostd-smallcontext:
	cargo test --features "nostd smallcontext" -- --test-threads=1 --nocapture

clean:
	find . -type d -name target -exec rm -rf {} \; || exit 0
	find . -type f -name Cargo.lock -exec rm -f {} \; || exit 0

tag:
	git tag -a v${VERSION} -m v${VERSION}
	git push origin --tags

ver:
	sed -i 's/^version = ".*/version = "${VERSION}"/g' Cargo.toml

doc:
	grep -v "^//!" src/lib.rs > src/lib.rs.tmp
	sed 's|^|//! |g' README.md > src/lib.rs
	cat src/lib.rs.tmp >> src/lib.rs
	rm -f src/lib.rs.tmp
	cargo doc

pub: doc test publish-cargo-crate

publish-cargo-crate:
	cargo publish
