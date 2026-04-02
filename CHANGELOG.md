# Changelog

This is containing every change, there are and there will be some bugs. But
tackling them down and documenting them will hopefully help you out. :)

## v0.10.0

Breaking changes:

- Upgrade `jsonwebtoken` v9.3.1 → v10.3.0. The re-exported types `Algorithm`, `TokenData<T>`, and `Validation` may have API differences. Users who depend on these types directly should review the [jsonwebtoken v10 changelog](https://github.com/nickel-org/jsonwebtoken/blob/master/CHANGELOG.md).

- Upgrade `reqwest` v0.12.28 → v0.13.2. Users who also depend on `reqwest` directly will need to upgrade to `0.13`.

Other changes:

- Upgrade `leptos-use` v0.16.3 → v0.18.3.
- Fix: `[lib] name` was incorrectly set to `leptos_oidc` instead of `leptos_oidc2`, causing `leptos_oidc2::` to be unavailable as a module path.
- Fix: doc-comment examples now correctly reference `leptos_oidc2::`.
- Add: GitHub Actions CI workflow.
- Add: Dependabot configuration.

## v0.9.1

- Fork as `leptos_oidc2`, a community-maintained continuation of [leptos_oidc](https://gitlab.com/kerkmann/leptos_oidc).
- Implement automatic access token refresh via refresh token flow, avoiding unnecessary re-authentication.
- Handle OIDC providers that omit `refresh_expires_in`; treat refresh tokens as valid when expiration is not provided.

## v0.9.0

- Update dependencies for leptos [v0.8.0](https://github.com/leptos-rs/leptos/releases/tag/v0.8.0). 
- Update keycloak version in [examples](examples/backend-keycloak/Dockerfile) to version 26.2.5

## v0.8.1

- Update dependencies to ensure compatibility with newer versions of Leptos 0.7.

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
