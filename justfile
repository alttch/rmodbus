VERSION := `grep ^version Cargo.toml|cut -d\" -f2`

all: test

test: test-std test-nostd

test-std:
	cargo test --all-features -- --test-threads=1 --nocapture

test-nostd:
	cargo test tests --no-default-features -- --test-threads=1 --nocapture

tag:
	git tag -a v{{VERSION}} -m v{{VERSION}}
	git push origin --tags

doc:
	cargo doc

pub: doc test publish-cargo-crate tag

publish-cargo-crate:
	cargo publish
