#!/bin/bash

export KCADM_PATH=/opt/keycloak/bin/kcadm.sh

wait_for_keycloak() {
  local timeout="${1:-600}"
  local sleep_time="${2:-5}"

  START_TIME="$(date +%s)"
  END_TIME=$((START_TIME + timeout))

  # wait until keycloak is ready and returns a status code < 400
  while ! curl --noproxy "*" --silent --fail --connect-timeout 2 --max-time 2 "$KEYCLOAK_URL" --output /dev/null; do
    local now
    now=$(date +%s)
    if [ "$now" -gt "$END_TIME" ]; then
      echo "Timeout while waiting for Keycloak to start up at: '$KEYCLOAK_URL'"
      return 1
    fi
    echo "Waiting for Keycloak to start up..."
    sleep "$sleep_time"
  done

  echo "Keycloak ready"
}
kcadm() { local cmd="$1" ; shift ; "$KCADM_PATH" "$cmd" --config /tmp/kcadm.config "$@" ; }
kcauth() { "$KCADM_PATH" config credentials --config /tmp/kcadm.config --server "$KEYCLOAK_URL" --realm master --user "$KEYCLOAK_ADMIN" --password "$KEYCLOAK_ADMIN_PASSWORD" ; }

get_admin_oauth_token() {
  # for debugging: may be used to get an admin token
  RESPONSE=$(curl --noproxy "*" --silent --data "client_id=admin-cli" --data "username=$KEYCLOAK_ADMIN" --data "password=$KEYCLOAK_ADMIN_PASSWORD" --data "grant_type=password" "$KEYCLOAK_URL"/realms/master/protocol/openid-connect/token)
  ADMIN_TOKEN=$(echo "$RESPONSE" | jq -r '.access_token')
  echo "$ADMIN_TOKEN"
}

curl_keycloak_get() {
  # for debugging: may be used to get any resource from keycloak
  ADMIN_TOKEN="$(get_admin_oauth_token)"
  ARGS="$1"

  curl --noproxy "*" --header "Authorization: Bearer $ADMIN_TOKEN" "$KEYCLOAK_URL/$ARGS"
}

list_realms() {
  kcadm get realms 2>/dev/null | jq -r ".[].realm"
}

create_realm() {
  realm_name="$1"
  EXISTING_REALMS=$(list_realms)
  if [[ "$EXISTING_REALMS" == *"${realm_name}"* ]]; then
    echo "Realm ${realm_name} already exists"
  else
    kcadm create realms -s realm="${realm_name}" -s enabled=true
  fi
}

get_client_id() {
  CLIENT_NAME="$1"
  CLIENT_REALM="${2:-$REALM}"

  kcadm get clients -r "${CLIENT_REALM}" | jq -r ".[] | select(.clientId==\"${CLIENT_NAME}\").id"
}

create_public_client() {
  CLIENT_NAME="$1"
  CLIENT_REDIRECT_URI="$2"
  CLIENT_POST_LOGOUT_REDIRECT_URI="$3"
  CLIENT_WEB_ORIGINS="$4"
  CLIENT_ACCESS_TOKEN_LIFESPAN="${5:-300}"
  CLIENT_REALM="${6:-$REALM}"

  if [ -z "$CLIENT_REALM" ]; then
    echo "ERROR: No realm provided for public client."
    return 1
  fi

  CLIENT_EXISTS=$(get_client_id "${CLIENT_NAME}" "${CLIENT_REALM}")
  if [ -z "$CLIENT_EXISTS" ]; then
    echo "Create public client ${CLIENT_NAME} in realm ${CLIENT_REALM}."
    kcadm create clients -r "${CLIENT_REALM}" -f - << EOF
      {
        "enabled": true,
        "clientId": "$CLIENT_NAME",
        "publicClient": true,
        "standardFlowEnabled": true,
        "fullScopeAllowed": true,
        "webOrigins": ["$CLIENT_WEB_ORIGINS"],
        "redirectUris": [$CLIENT_REDIRECT_URI],
        "attributes": {
          "access.token.lifespan": "$CLIENT_ACCESS_TOKEN_LIFESPAN",
          "post.logout.redirect.uris": "$CLIENT_POST_LOGOUT_REDIRECT_URI"
        }
      }
EOF
    echo "Public client ${CLIENT_NAME} created. Result code: $?"

  else
    echo "WARNING: Client ${CLIENT_NAME} already exists in realm ${CLIENT_REALM}."
    return
  fi

}

list_realm_groups() {
    REALM=${1:-$REALM}
    kcadm get groups -r "${REALM}" 2>/dev/null | jq -r ".[].name"
}

create_realm_group() {
  GROUP_NAME="$1"
  GROUP_REALM="${2:-$REALM}"

  if [ -z "$GROUP_REALM" ]; then
    echo "ERROR: No realm provided for realm group."
    return 1
  fi

  EXISTING_REALM_GROUPS=$(list_realm_groups "${GROUP_REALM}")
  if [[ "$EXISTING_REALM_GROUPS" == *"${GROUP_NAME}"* ]]; then
    echo "Realm group ${GROUP_NAME} already exists"
  else
      echo "Create realm group ${GROUP_NAME}"
      kcadm create groups -r "${GROUP_REALM}" -s name="${GROUP_NAME}"
  fi
}

list_realm_roles() {
  REALM=${1:-$REALM}
  kcadm get roles -r "${REALM}" 2>/dev/null | jq -r ".[].name"
}

create_realm_role() {
  ROLE_NAME="$1"
  ROLE_REALM="${2:-$REALM}"

  EXISTING_REALM_ROLES=$(list_realm_roles "${ROLE_REALM}")
  if [[ "$EXISTING_REALM_ROLES" == *"${ROLE_NAME}"* ]]; then
    echo "Realm ${ROLE_NAME} already exists"
  else
      echo "Create realm role ${ROLE_NAME}"
      kcadm create roles -r "${ROLE_REALM}" -f - << EOF
      {
        "name": "${ROLE_NAME}",
        "description": "${ROLE_NAME} role",
        "attributes": {}
      }
EOF
  fi
}


create_user() {
  USER_NAME="$1"
  USER_PASSWORD="$2"
  USER_GROUP="$3"
  USER_ROLE="$4"
  USER_REALM="${5:-$REALM}"
  if [ -z "$USER_NAME" ]; then
    echo "ERROR: No username provided."
    return 1
  fi
  if [ -z "$USER_PASSWORD" ]; then
    echo "ERROR: No password provided."
    return 1
  fi
  if [ -z "$USER_REALM" ]; then
    echo "ERROR: No realm provided."
    return 1
  fi

  echo "Create keycloak user ${USER_NAME} in realm ${USER_REALM}."
  kcadm create users -r "${USER_REALM}" -f - << EOF
  {
    "username": "${USER_NAME}",
    "email": "${USER_NAME}@example.com",
    "firstName": "Firstname",
    "lastName": "Lastname",
    "requiredActions": [],
    "emailVerified": false,
    "groups": [],
    "attributes": {},
    "enabled": true
  }
EOF
  USER_ID=$(kcadm get users -r "${USER_REALM}" | jq -r ".[] | select(.username==\"${USER_NAME}\").id")

  echo "Update user password for user '${USER_NAME}'."
  kcadm update users/"$USER_ID"/reset-password -r "${USER_REALM}" -f - << EOF
  {
    "temporary": false,
    "type": "password",
    "value": "$USER_PASSWORD"
  }
EOF

  if [ -n "$USER_GROUP" ]; then
    echo "add user '$USER_NAME' to group '$USER_GROUP'"
    USER_GROUP_ID=$(kcadm get groups -r "${USER_REALM}" | jq -r ".[] | select(.name==\"${USER_GROUP}\").id")
    kcadm update users/"$USER_ID"/groups/"$USER_GROUP_ID" -r "${USER_REALM}"
  fi

  if [ -n "$USER_ROLE" ]; then
    echo "add user '$USER_NAME' to role '$USER_ROLE'"
    USER_ROLE_ID=$(kcadm get roles -r "${USER_REALM}" | jq -r ".[] | select(.name==\"${USER_ROLE}\").id")
    USER_ROLE_CONTAINER_ID=$(kcadm get users/"$USER_ID"/role-mappings -r "${USER_REALM}" | jq -r '.realmMappings[] | select(.name=="default-roles-master").containerId')
    kcadm create users/"$USER_ID"/role-mappings/realm -r "${USER_REALM}" -f - << EOF
    [{
      "id": "$USER_ROLE_ID",
      "name": "$USER_ROLE",
      "description": "",
      "composite": false,
      "clientRole": false,
      "containerId": "$USER_ROLE_CONTAINER_ID"
    }]
EOF
  fi
}

get_client_scope_id() {
    CLIENT_SCOPE_NAME="$1"
    CLIENT_SCOPE_REALM="${2:-$REALM}"

  kcadm get client-scopes -r "${CLIENT_SCOPE_REALM}" | jq -r ".[] | select(.name==\"$CLIENT_SCOPE_NAME\").id"
}

create_client_scope() {
  CLIENT_SCOPE_NAME="$1"
  CLIENT_SCOPE_TYPE="${2:-default}"
  CLIENT_SCOPE_REALM="${3:-$REALM}"

  if [ -z "$CLIENT_SCOPE_REALM" ]; then
    echo "ERROR: No realm provided for client scope."
    return 1
  fi
  kcadm create client-scopes -r "${CLIENT_SCOPE_REALM}" -f - << EOF
    {
      "name": "$CLIENT_SCOPE_NAME",
      "description": "",
      "attributes": {
        "consent.screen.text": "",
        "display.on.consent.screen": "true",
        "include.in.token.scope": "true",
        "gui.order": ""
      },
      "type": "$CLIENT_SCOPE_TYPE",
      "protocol": "openid-connect"
    }
EOF
}

create_client_scope_groups() {
  SCOPE_NAME="${1:-groups}"
  SCOPE_REALM="$2"

  if [ -z "$SCOPE_REALM" ]; then
    echo "ERROR: No realm provided for client scope groups."
    return 1
  fi

  CLIENT_SCOPE_GROUP_ID=$(get_client_scope_id "$SCOPE_NAME" "$SCOPE_REALM")
  if [ -z "$CLIENT_SCOPE_GROUP_ID" ]; then
    echo "Client scope $SCOPE_NAME not found in realm $SCOPE_REALM"
    return 1
  fi

  kcadm create client-scopes/"$CLIENT_SCOPE_GROUP_ID"/protocol-mappers/models -r "${SCOPE_REALM}" -f - << EOF
    {
      "protocol": "openid-connect",
      "protocolMapper": "oidc-group-membership-mapper",
      "name": "$SCOPE_NAME",
      "config": {
        "claim.name": "$SCOPE_NAME",
        "full.path": "true",
        "id.token.claim": "true",
        "access.token.claim": "true",
        "userinfo.token.claim": "true"
      }
    }
EOF
}

update_existing_client_scope_realm_roles() {
  CLIENT_SCOPE_NAME="$1"
  CLIENT_SCOPE_REALM="$2"

  CLIENT_SCOPE_ROLE_ID=$(kcadm get client-scopes -r "$CLIENT_SCOPE_REALM" | jq -r ".[] | select(.name==\"$CLIENT_SCOPE_NAME\").id")
  CLIENT_SCOPE_ROLES_PROTOCOL_MAPPER_ROLES_ID=$(kcadm get client-scopes/"$CLIENT_SCOPE_ROLE_ID"/protocol-mappers/models/ -r "$CLIENT_SCOPE_REALM" | jq -r '.[] | select(.name=="realm roles").id')

  # renames the protocol mapper 'realm_access.roles' to 'roles'
  kcadm update client-scopes/"$CLIENT_SCOPE_ROLE_ID"/protocol-mappers/models/"$CLIENT_SCOPE_ROLES_PROTOCOL_MAPPER_ROLES_ID" -r "$CLIENT_SCOPE_REALM" -f - << EOF
    {
      "id": "$CLIENT_SCOPE_ROLES_PROTOCOL_MAPPER_ROLES_ID",
      "protocol": "openid-connect",
      "protocolMapper": "oidc-usermodel-realm-role-mapper",
      "name": "realm roles",
      "config": {
        "usermodel.realmRoleMapping.rolePrefix": "",
        "multivalued": "true",
        "claim.name": "roles",
        "jsonType.label": "String",
        "id.token.claim": "true",
        "access.token.claim": "true",
        "userinfo.token.claim": "true",
        "user.attribute": "foo"
      },
      "consentRequired": false
    }
EOF
}

add_client_scope_to_client() {
  CLIENT_NAME="$1"
  CLIENT_SCOPE_NAME="$2"
  CLIENT_REALM="${3:-$REALM}"

  CLIENT_ID=$(get_client_id "$CLIENT_NAME" "$CLIENT_REALM")
  CLIENT_SCOPE_ID=$(get_client_scope_id "$CLIENT_SCOPE_NAME" "$CLIENT_REALM")
  kcadm update clients/"$CLIENT_ID"/default-client-scopes/"$CLIENT_SCOPE_ID" -r "${CLIENT_REALM}"
  # shellcheck disable=SC2181
  if [ $? -eq 0 ]; then
    echo "Added client scope $CLIENT_SCOPE_NAME to client $CLIENT_NAME"
  else
    echo "WARNING: Failed to add client scope $CLIENT_SCOPE_NAME to client $CLIENT_NAME"
    return 1
  fi
}
