# Changelog

This is containing every change, there are and there will be some bugs. But
tackling them down and documenting them will hopefully help you out. :)

## v0.8.0

- Fork from this PR: [Update leptos-use, remove unused crate](https://gitlab.com/kerkmann/leptos_oidc/-/merge_requests/8/)

Breaking changes:
- Major upgrade for leptos version 0.7 ([new leptos feature](https://github.com/leptos-rs/leptos/releases/tag/v0.7.0) `.await on resources and async in <Suspense/>`)
- Add code examples: [simple](examples/simple/src/simple.rs)
- Add keycloak server to test the examples easily with an identity provider. 
  The keycloak server is provisioned with an OIDC client (`leptos-client`), users, roles and groups. See [keycloak](examples/backend-keycloak/Readme.md).
- Refactor to use an enum as signal that contains exactly the relevant information. This avoids unpacking of option values.
- The `Auth::init()` may be done in an async method now (see [elaborate example](examples/elaborate/src/elaborate.rs)). 
  Previously the init did use `provide_context` to provide the signal in the leptos context but this would require initializing somewhere higher up in the reactive tree. 
  Now the user is required to provide the AuthSignal:
    ```
    let auth: AuthSignal = Auth::signal();
    provide_context(auth);
    ```
- Parse response parameters in Authentication Code Flow from authentication provider (issuer) only from the paths indicated by `redirect_uri` and the `post_logout_redirect_uri` (e.g. `/profile` and `/logout`).

## v0.7.0

- Add `PKCE` capability (which should be used on production systems).

## v0.6.1

- Add `PartialEq`, `Eq`, `Hash` and `Serialize` whenever it's possible.
- Fix minor documentation for `keycloak` and `rauthy` backend.

## v0.6.0

- Refresh the access token automatically in the background.
- Fetch JWK and Issuer informations auomatically.

## v0.5.0

- Add optional `audience` field for AuthParameters.

## v0.4.1

- Fix: Use TimeDelta::try_seconds instead of Duratoin::seconds
- Fix: Add missing `audience` parameter for `decoded_access_token_unverified`

## v0.4.0

- Bump version for leptos 0.6
- Update README.md to show compatible leptos versions

## v0.3.1

- Fix decode_access_token, which was not decoding the `access_token`

## v0.3.0

- Add capability to decode the access token inside this crate 

## v0.2.2

- Fix `when reloading page and refresh_expires_in is null token is removed` #2

## v0.2.1

- Fix `crash when converting from SucessTokenResponse to TokenStorage` #1

## v0.2.0

- Add rauthy support
- Set fields like `refresh_expires_in` as optional
- Set clippy to pedantic in pipeline
- Add KeyCloak and rauthy backend example in the README.md
- Add CHANGELOG.md

## v0.1.1

- Add missing `use import` in the README.md example
- Fix endpoints in the example in the README.md

## v0.1.0

This is the initial release of a working POC, it's not perfect, but working. :)
