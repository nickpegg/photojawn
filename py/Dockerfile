# Build stage
FROM python:3.12-slim AS build

WORKDIR /app

RUN pip install --upgrade pip &&\
    pip install --no-cache-dir poetry &&\
    poetry config virtualenvs.create false &&\
    apt update && apt install -y build-essential

COPY pyproject.toml poetry.lock ./
RUN poetry install --without=dev

# Clean up apt stuff
RUN apt remove -y build-essential &&\
    apt autoremove -y &&\
    rm -rf /var/lib/apt/lists/*

COPY . .


# Run stage
FROM python:3.12-slim

ENV PYTHONUNBUFFERED 1
ENV PYTHONDONTWRITEBYTECODE 1

# # Default port, overridable by setting an outside env var, which Heroku does
# ENV PORT 8000
# ENV NUM_WORKERS 8
# EXPOSE ${PORT}

WORKDIR /app

# Copy all the built libs from the build container
COPY --from=build / /

ARG GIT_COMMIT=unknown
LABEL git_commit=$GIT_COMMIT
ENV GIT_COMMIT $GIT_COMMIT

CMD TODO
