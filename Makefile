VERSION=0.3.5

all: test

test: test-std test-std-single test-nostd test-nostd-single test-nostd-single-smallcontext

test-std:
	cargo test --features std -- --test-threads=1 --nocapture

test-std-single:
	cargo test --features "std single" -- --test-threads=1 --nocapture

test-nostd:
	cargo test --features nostd -- --test-threads=1 --nocapture

test-nostd-single:
	cargo test --features "nostd single" -- --test-threads=1 --nocapture

test-nostd-single-smallcontext:
	cargo test --features "nostd single smallcontext" -- --test-threads=1 --nocapture

clean:
	find . -type d -name target -exec rm -rf {} \; || exit 0
	find . -type f -name Cargo.lock -exec rm -f {} \; || exit 0

tag:
	git tag -a v${VERSION}
	git push origin --tags

ver:
	sed -i 's/^version = ".*/version = "${VERSION}"/g' Cargo.toml

doc:
	grep -v "^//! " src/lib.rs > src/lib.rs.tmp
	sed 's|^|//! |g' README.md > src/lib.rs
	cat src/lib.rs.tmp >> src/lib.rs
	rm -f src/lib.rs.tmp
	cargo doc --features std

pub: doc test publish-cargo-crate

publish-cargo-crate:
	cargo publish --features std
