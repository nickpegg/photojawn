# Default, which is handy to run before committing code
all: fmt lint test

# What to have CI systems run
ci: lint test

# Final pre-flight checks then deploy everywhere!
# TODO
# shipit: all build staging prod

# version := $(shell yq -p toml .tool.poetry.version < pyproject.toml)


init:
	poetry install

# Everything to get the dev env set up
dev: init

fmt:
	cargo fmt

lint:
	cargo clippy

test:
	RUST_BACKTRACE=1 cargo test --release


test-watch:
	find . -name '*rs' -or -name '*html' -or -name Cargo.lock | entr -r -c make test

clean:
	cargo clean

# TODO?
# docker:
# 	podman build -t nickpegg/photojawn . --build-arg GIT_COMMIT=$(shell git rev-parse --short HEAD)

dist:
	cargo build --release

# TODO
# release: dist
# 	git push --tags
# 	gh release create --verify-tag v$(version)
# 	gh release upload v$(version) dist/photojawn-$(version)-*whl $(foreach plat,$(scie_platforms),dist/photojawn-$(plat))
