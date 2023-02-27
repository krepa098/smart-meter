#!/bin/bash

#export TRUNK_ADDRESS="127.0.0.1"
export TRUNK_ADDRESS="192.168.178.199"
export TRUNK_PORT="80"
export API_URL="http://192.168.178.199:8081"
export DATABASE_URL="/data/database.db"

# create/migrate db if it does not exist
cd /usr/local/bin/backend && diesel migration run

# run backend
/usr/local/bin/backend/backend &

# run frontend
cd /usr/local/bin/src/frontend && trunk serve --release --address=$TRUNK_ADDRESS --port=$TRUNK_PORT
