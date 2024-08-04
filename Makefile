# Default, which is handy to run before committing code
all: fmt lint test

# What to have CI systems run
ci: init lint test

# Final pre-flight checks then deploy everywhere!
shipit: all build staging prod


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

run:
	podman-compose up

build:
	podman build -t nickpegg/photoalbum . --build-arg GIT_COMMIT=$(shell git rev-parse --short HEAD)

clean:
	podman-compose down --rmi all
