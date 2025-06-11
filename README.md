# Leptos OIDC Authentication

**leptos_oidc** is a utility library for handling OpenID Connect (OIDC)
authentication within the Leptos framework. It simplifies the integration of
OIDC authentication flows with Leptos-based applications, making it easier to
manage user authentication and tokens.

## Table of Contents

- [Leptos compatibility](leptos-compatibility)
- [Features](#features)
- [Missing Features](#missing-features)
- [Tested Backends with Example](#tested-backends-with-example)
- [Usage](#usage)
  - [Initialization](#initialization)
  - [Generating Login and Logout URLs](#generating-login-and-logout-urls)
  - [Conditional Rendering Components](#conditional-rendering-components)
  - [Refreshing Access Tokens](#refreshing-access-tokens)
- [License](#license)

## Leptos compatibility

| Crate version | Compatible Leptos version |
|---------------|---------------------------|
| <= 0.3        | 0.5                       |
| 0.4-0.7       | 0.6                       |
| 0.8           | 0.7                       |

## Features

**leptos_oidc** offers the following features:

- Initialization of the OIDC authentication process.
- Generation of login and logout URLs for redirecting users to OIDC providers (e.g., Keycloak).
- Conditional rendering of components based on the authentication state.
- Refreshing access tokens and storing them in local storage.
- Working with client and server side rendering
- Automatically refresh the access token in the background.
- PKCE challenge

### Missing Features

- Make refresh token optional
- Some minor code refactoring/cleanup

### Tested Backends with Example

**leptos_oidc** was tested with various backends. This doesn't mean that other
backends are not supported. Every backend which supports `oidc` should work.
But feel free to ask for advice or give feedback!

Tested backends:
- [KeyCloak](https://github.com/keycloak/keycloak)
- [rauthy](https://github.com/sebadob/rauthy/)

You can find a setup guide for the backends under [docs/backends](docs/backends/README.md).

## Installation

To use **leptos_oidc** in your Leptos-based application, add it as a dependency
in your `Cargo.toml` file:

```toml
[dependencies]
leptos_oidc = "0.8"
```

Note: This needs at least `leptos v0.7`.

## Usage

### Initialization and Example

To get started with OIDC authentication, initialize the library with the
required authentication parameters. You can use the `AuthParameters` struct
to specify the OIDC endpoints, client ID, redirect URIs, and other relevant
information.

Please make sure that the `issuer` url is the base url without the `/.well-known/openid-configuration` and without a trailing slash.
A simple example may be found [here](examples/simple/src/simple.rs).

Note: Please keep in mind that the `Auth::init` needs to be `inside a Router`.
The internal state is using `use_query` and `use_navigate`, which is only available inside a
`Router`. 

### Generating Login and Logout URLs

**leptos_oidc** provides functions to generate login and logout URLs for your
application. These URLs are used to redirect users to the OIDC provider for
authentication and logout. They are available once the authentication is initialized.

```rust
use leptos::prelude::*;
use leptos_oidc::Auth;

#[component]
fn MyComponent() {
    let auth = expect_context::<AuthSignal>();
  
    // Generate the login URL to initiate the authentication process.
    let login_url = move || {
        auth.with(|auth| {
            auth
                .unauthenticated()
                .map(|unauthenticated| unauthenticated.login_url())
        })
    };
  
  // Generate the logout URL for logging out the user.
    let logout_url = move || {
        auth.get()
            .authenticated()
            .map(|authenticated| authenticated.logout_url())
    };
}
```

### Conditional Rendering Components

The library includes transparent components to conditionally render content
based on the authentication state. These components simplify the user interface
when dealing with authenticated and unauthenticated users.

```rust
use leptos::prelude::*;
use leptos_oidc::Auth;

#[component]
fn MyComponent() {

    view! {
        // Generate Sign In link
        <LoginLink class="optional-class-attributes">Sign in</LoginLink>

        // Generate Sign Out link
        <LogoutLink class="optional-class-attributes">Sign Out</LogoutLink>

        <AuthLoaded>"This will be rendered only when the auth library is not loading anymore"</AuthLoaded>

        <AuthLoading>"This will be rendered only when the auth library is still loading"</AuthLoading>

        <Authenticated>"This will only be rendered if the user is authenticated"</Authenticated>

        <AuthErrorContext>"This will only be rendered if there was an error during authentication"</AuthErrorContext>

        // A more complex example with optional fallbacks for the loading and unauthenticated state
        <Authenticated
            unauthenticated=move || view! { "this will only be rendered if the user is unauthenticated" }
            loading=move || view! { "this will only be rendered if the library is still loading" }
            >
                "This will only be rendered if the user is authenticated"
        </Authenticated>
    }
}
```

### Refreshing Access Tokens

This library is now capable of refreshing the `access_token` in the background. :)

## License

**leptos_oidc** is distributed under the [MIT License](https://opensource.org/licenses/MIT).
For more information, see the [LICENSE](https://gitlab.com/kerkmann/leptos_oidc/blob/main/LICENSE) file.
