#!/usr/bin/env bash

set -x
set -eo pipefail


if ! [ -x "$(command -v psql)" ]; then 
    >&2 echo "Error: psql is not installed"
    exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then 
    >&2 echo "Error: sqlx is not installed"
    >&2 echo "Use:"
    >&2 echo "     cargo install --version='~0.6' sqlx-cli \
    --no-default-features --features rustls,postgres"
    >&2 echo "to install it"
    exit 1
fi


DB_USER=${POSTGRES_USER:=postgres}

DB_PASSWORD="${POSTGRES_PASSWORD:=password}"

DB_NAME="${POSTGRES_DB:=newsletter}"

DB_PORT="${POSTGRES_PORT:=5432}"

docker run \
    -e POSTGRES_USER=${DB_USER} \
    -e POSTGRES_PASSWORD=${DB_PASSWORD} \
    -e POSTGRES_DB=${DB_NAME} \
    -p "${DB_PORT}":5432 \
    -d postgres \
    postgres -N 1000 # maximum number of connections for testing

export PGPASSWORD="${DB_PASSWORD}"
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    >&2 echo "Postgres is still unavailable - sleeping zzz"
    sleep 1
done

>&2 echo "Postgres livesssss running on port ${DB_PORT}!"

DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
export DATABASE_URL

sqlx database create
