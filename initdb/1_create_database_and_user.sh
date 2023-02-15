#!/bin/bash
set -e 

pwd=$(<$RECORDER_PASSWORD_FILE)

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE USER recorder WITH LOGIN PASSWORD '$pwd';
    CREATE DATABASE salty;
    GRANT pg_write_all_data TO recorder;
    GRANT pg_read_all_data TO recorder;
EOSQL
