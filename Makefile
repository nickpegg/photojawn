# Default, which is handy to run before committing code
all: fmt lint test

# What to have CI systems run
ci: init lint test

# Final pre-flight checks then deploy everywhere!
shipit: all build staging prod

version := $(shell yq -p toml .tool.poetry.version < pyproject.toml)
scie_platforms := linux-aarch64 linux-x86_64 macos-aarch64 macos-x86_64


init:
	poetry install

# Everything to get the dev env set up
dev: init

fmt:
	poetry run ruff check --select I --fix	# import sorting
	poetry run ruff format

lint:
	poetry run ruff check --fix

test:
	poetry run mypy .
	poetry run pytest

# Faster tests, only running what's changed
test-fast:
	poetry run mypy .
	poetry run pytest --testmon

test-watch:
	find . -name '*py' -or -name '*html' -or -name poetry.lock | entr -r -c make test-fast

clean:
	rm -rv dist || true

docker:
	podman build -t nickpegg/photojawn . --build-arg GIT_COMMIT=$(shell git rev-parse --short HEAD)

dist:
	poetry build
	poetry run pex --project . -o dist/photojawn -c photojawn --scie eager $(foreach plat,$(scie_platforms), --scie-platform $(plat))

release: dist
	git push --tags
	gh release create --verify-tag v$(version)
	gh release upload v$(version) dist/photojawn-$(version)-*whl $(foreach plat,$(scie_platforms),dist/photojawn-$(plat))
