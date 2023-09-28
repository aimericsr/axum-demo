#!/usr/bin/env bash
set -x
set -eo pipefail

if ! [ -x "$(command -v psql)" ]; then
  echo >&2 "Error: psql is not installed."
  exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Use:"
  echo >&2 "    cargo install --version=0.7.2 sqlx-cli --no-default-features --features postgres"
  echo >&2 "to install it."
  exit 1
fi

if [ -z "$PGUSER" ]; then
  echo "La variable d'environnement PGUSER n'est pas définie."
  exit 1
fi

if [ -z "$PGPASSWORD" ]; then
  echo "La variable d'environnement PGPASSWORD n'est pas définie."
  exit 1
fi

if [ -z "$PGHOST" ]; then
  echo "La variable d'environnement PGHOST n'est pas définie."
  exit 1
fi

# Keep pinging Postgres until it's ready to accept commands
until [ $attempts -ge $max_attempts ] || psql -d "postgres" -c '\q'; do
  >&2 echo "Postgres is still unavailable - sleeping"
  sleep 1 
  attempts=$((attempts + 1))
done

if [ $attempts -ge $max_attempts ]; then
  >&2 echo "Maximum number of attempts reached. Exiting."
  exit 1
fi

>&2 echo "Postgres is up and running on port ${DB_PORT}!"

sqlx database create
sqlx migrate run
>&2 echo "Postgres has been migrated, ready to go!"