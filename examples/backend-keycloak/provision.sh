#!/bin/bash

source /keycloak_functions.sh

main() {
  #set -ex

  echo "provisioning keycloak"
  wait_for_keycloak || { echo "Keycloak not ready, exiting"; exit 1;}
  kcauth

  REALM_NAME="master"
  CLIENT_ACCESS_TOKEN_LIFESPAN=300
  LEPTOS_CLIENT="leptos-client"
  create_public_client "$LEPTOS_CLIENT" '"http://localhost:3000/profile", "http://localhost:3000/*"' "http://localhost:3000?destroy_session=true##http://localhost:3000/logout?destroy_session=true##http://localhost:3000/profile?destroy_session=true" 'http://localhost:3000' $CLIENT_ACCESS_TOKEN_LIFESPAN "$REALM_NAME"

  # Create keycloak realm user groups
  create_realm_group testgroup "$REALM_NAME"
  create_realm_group managergroup "$REALM_NAME"

  # Create keycloak realm user roles
  create_realm_role testrole "$REALM_NAME"
  create_realm_role managerrole "$REALM_NAME"

  # Create keycloak realm test users:
  # username, password, group, role, realm
  create_user testuser testuser testgroup testrole "$REALM_NAME"
  create_user leptos leptos testgroup testrole "$REALM_NAME"
  create_user manager manager managergroup managerrole "$REALM_NAME"
  # Create user nobody (no group, no role)
  create_user nobody nobody "" "" "$REALM_NAME"

  # map roles and groups to access token
    GROUP_SCOPE_NAME="groups"
    create_client_scope "$GROUP_SCOPE_NAME" "default" "$REALM_NAME"
    create_client_scope_groups "$GROUP_SCOPE_NAME" "$REALM_NAME"
    add_client_scope_to_client "$LEPTOS_CLIENT" "$GROUP_SCOPE_NAME" "$REALM_NAME"
    update_existing_client_scope_realm_roles "roles" "$REALM_NAME"

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
