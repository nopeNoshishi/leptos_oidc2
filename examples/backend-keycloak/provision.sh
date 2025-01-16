#!/bin/bash

source /keycloak_functions.sh

main() {
  #set -ex

  echo "provisioning keycloak"
  wait_for_keycloak || { echo "Keycloak not ready, exiting"; exit 1;}
  kcauth

  CLIENT_ACCESS_TOKEN_LIFESPAN=300
  create_public_client "leptos-client" '"http://localhost:3000/profile", "http://localhost:3000/*"' 'http://localhost:3000?destroy_session=true' 'http://localhost:3000' $CLIENT_ACCESS_TOKEN_LIFESPAN "master"

}

main

if [ -n "$1" ]; then
  # Leave the container running if an argument is provided
  sleep infinity &

  # Wait for any process to exit
  wait -n

  # Exit with status of process that exited first
  exit $?
fi
