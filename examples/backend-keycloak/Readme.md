# Keycloak docker image

This image is based on the official Keycloak docker image. 
* [Keycloak docker getting started](https://www.keycloak.org/getting-started/getting-started-docker)
* [Keycloak docker documentation](https://www.keycloak.org/server/containers)


> **WARNING**:
This is by all means no production-ready deployment of keycloak! It's a mere example to start keycloak in development mode.
Its purpose is solely for testing.

## Usage

* Start: 
  ```shell
  docker compose up --detach
  ```
* Show provisioning log:
  ```shell
  docker logs keycloak-init --follow
  ```
* Shutdown: 
  ```shell
  docker compose down
  ```
* Shutdown and destroy:
  ```shell
  docker compose down --volumes
  ```

## Provisioned data

See also [provision script](provision.sh).

* Leptos client: `leptos-client`
* Users: Name and password are the same.

| User     | Group        | Role        |
|----------|--------------|-------------|
| leptos   | testgroup    | testrole    |
| testuser | testgroup    | testrole    |
| manager  | managergroup | managerrole |
| nobody   | -            | -           |
