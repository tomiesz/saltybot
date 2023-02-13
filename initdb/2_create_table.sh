#!/bin/bash
set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" -d "salty" <<-EOSQL
    CREATE TABLE players (
        id SERIAL PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        wins INT DEFAULT 0,
        losses INT DEFAULT 0
    )
EOSQL
