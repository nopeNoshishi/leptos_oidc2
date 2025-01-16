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