VERSION=0.3.0

all: switch-std test switch-nostd test

pub: switch-std publish-cargo-crate

test:
	cargo test -- --test-threads=1

test-nostd:
	cargo test -- --manifest-path=nostd/Cargo.toml -- --test-threads=1

clean:
	find . -type d -name target -exec rm -rf {} \; || exit 0
	find . -type f -name Cargo.lock -exec rm -f {} \; || exit 0

tag:
	git tag -a v${VERSION}
	git push origin --tags

ver:
	sed -i 's/^version = ".*/version = "${VERSION}"/g' Package.toml

switch-std:
	cat Package.toml std.toml > Cargo.toml

switch-nostd:
	cat Package.toml nostd.toml > Cargo.toml

publish-cargo-crate:
	cargo publish
