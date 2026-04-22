#!/bin/sh
pg_basebackup -d postgres://postgres@docker-primary-1:5432/postgres -D /home/pgdata -U postgres -Fp -R -Xs -c fast -l 'initial clone' -P -v -S standby1 &&
chown -R postgres:postgres /home &&
chmod -R 750 /home
export PGDATA=/home/pgdata &&
docker-entrypoint.sh  postgres

