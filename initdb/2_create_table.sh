#!/bin/bash
set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" -d "salty" <<-EOSQL
    CREATE TABLE matches (
        id SERIAL PRIMARY KEY,
        player1 TEXT NOT NULL,
        player2 TEXT NOT NULL,
        winner SMALLINT NOT NULL
    )
EOSQL
