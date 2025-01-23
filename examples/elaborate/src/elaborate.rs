use crate::user::Claims;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Link, Stylesheet, Title};
use leptos_oidc::{Algorithm, Auth, AuthErrorContext, AuthLoaded, AuthLoading, AuthParameters, AuthSignal, Authenticated, LoginLink, LogoutLink, ReloadButton, TokenData};
use leptos_router::components::{ProtectedRoute, Route, Router, Routes};
use leptos_router::path;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Clone, thiserror::Error)]
pub enum AppConfigError {
    /// An error related to handling parameters.
    #[error("params error: {0}")]
    Params(#[from] leptos_router::params::ParamsError),

    /// An error related to the serialization or deserialization of JSON data.
    #[error("failed to serialize/deserialilze json: {0}")]
    Serde(#[from] Arc<serde_json::Error>),

    #[error("failed to request configuration: {0}")]
    Request(#[from] Arc<gloo_net::Error>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    oidc: AuthParameters,
}

#[derive(Clone)]
pub struct AppGlobals {}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/main.css"/>

        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>

        <Router>
            <AppWithRouter/>
        </Router>
    }
}

#[component]
pub fn AppWithRouter() -> impl IntoView {
    let auth: AuthSignal = Auth::signal();
    provide_context(auth);

    let app_globals: LocalResource<Result<AppGlobals, AppConfigError>> =
        LocalResource::new(move || async {
            let app_config = gloo_net::http::Request::get("/config.json")
                .send()
                .await
                .map_err(Arc::new)?
                .json::<AppConfig>()
                .await
                .map_err(Arc::new)?;
            Auth::init(app_config.oidc.clone());

            Ok(AppGlobals {})
        });
    provide_context(app_globals);

    let user = Signal::derive(move || {
        auth.with(|auth| {
            auth.authenticated()
                .map(|auth| auth.decoded_access_token::<Claims>(Algorithm::RS256, &["account"]))
                .flatten()
        })
    });
    provide_context(user);
    let manager = move || user.get().map(|user| user.claims.has_role("managerrole"));
    let tester = move || {
        user.get()
            .map(|user| user.claims.has_group("testgroup") || user.claims.has_role("managerrole"))
    };

    view! {
        <h1>Leptos OIDC</h1>
        <Navigation/>
        <DebugInfo/>
        <ReloadButton/>
        <Routes fallback=Home>
            <Route path=path!("/") view=Home/>
            <Route path=path!("/logout") view=Logout/>
            <ProtectedRoute
                path=path!("/manager")
                view=Manager
                condition=manager
                redirect_path=|| "/"
            />
            <ProtectedRoute
                path=path!("/tester")
                view=Tester
                condition=tester
                redirect_path=|| "/"
            />
            <Route
                path=path!("/profile")
                view=|| {
                    view! {
                        <AppConfigLoaded>
                            <AuthLoaded>
                                <Authenticated unauthenticated=Unauthenticated>
                                    <Profile/>
                                </Authenticated>
                            </AuthLoaded>
                        </AppConfigLoaded>
                    }
                }
            />
        </Routes>
    }
}

#[component]
pub fn AppConfigLoaded(
    children: ChildrenFn,
    #[prop(optional, into)] fallback: ViewFnOnce,
) -> impl IntoView {
    let app_globals = use_context::<LocalResource<Result<AppGlobals, AppConfigError>>>()
        .expect("AppConfigLoaded component: App globals resource should exist!");
    let children = StoredValue::new(children);

    view! {
        <Transition
            fallback
        >
            { move || {
                Suspend::new(async move {
                    if let Ok(_app_config) = app_globals.await {
                        Either::Right(view! {
                            { children.read_value()() }
                        })
                    } else {
                        Either::Left(view! {
                            <p>Failed to load app configuration!</p>
                        })
                    }
                })
            }}

        </Transition>
    }
}

#[component]
pub fn Tester() -> impl IntoView {
    view! {
        <Title text="Tester"/>
        <h1>Tester</h1>
        <p>Your Tester Page</p>

    }
}

#[component]
pub fn Manager() -> impl IntoView {
    view! {
        <Title text="Management"/>
        <h1>Management</h1>
        <p>Your Management Page</p>
        <a href="/profile">Profile page</a>

    }
}

#[component]
pub fn AuthErrorPage() -> impl IntoView {
    let auth = use_context::<AuthSignal>().expect("AuthStore not initialized in error page");
    let error_message = move || auth.get().error().map(|err| format!("{err:?}"));
    view! {
        <h1>Error occurred</h1>
        <p>There was an error in the authentication process!</p>
        { error_message }
    }
}

#[component]
pub fn Navigation() -> impl IntoView {
    let user = use_context::<Signal<Option<TokenData<Claims>>>>()
        .expect("Navigation: user store should exist!");
    let manager = move || {
        user.get()
            .map(|user| user.claims.has_role("managerrole"))
            .unwrap_or(false)
    };
    let tester = move || {
        user.get()
            .map(|user| user.claims.has_group("testgroup") || user.claims.has_role("managerrole"))
            .unwrap_or(false)
    };

    view! {
        <h2>Navigation</h2>
        <ul>
            <li><a href="/">Home</a></li>
            <li><a href="/profile">Profile</a></li>
            <Show when=manager>
                <li><a href="/manager">Management</a></li>
            </Show>
            <Show when=tester>
                <li><a href="/tester">Tester</a></li>
            </Show>
        </ul>
        <AppConfigLoaded fallback=|| view! { <p>Loading ..</p> }>
            <AuthLoaded>
                <Authenticated unauthenticated=move || view! { <LoginLink class="text-login">Sign in</LoginLink> }>
                    <LogoutLink class="text-logout">Sign out</LogoutLink>
                </Authenticated>
            </AuthLoaded>
        </AppConfigLoaded>

    }
}

#[component]
pub fn DebugInfo() -> impl IntoView {
    let user = use_context::<Signal<Option<TokenData<Claims>>>>()
        .expect("Navigation: user store should exist!");

    let name = move || user.get().map(|user| user.claims.preferred_username);
    let email = move || user.get().map(|user| user.claims.email);
    let groups = move || {
        user.get().map(|user| {
            user.claims
                .groups
                .into_iter()
                .collect::<Vec<_>>()
                .join(", ")
        })
    };
    let roles = move || {
        user.get()
            .map(|user| user.claims.roles.into_iter().collect::<Vec<_>>().join(", "))
    };

    let manager = move || {
        user.get()
            .map(|user| user.claims.has_role("managerrole"))
            .unwrap_or(false)
    };
    let tester = move || {
        user.get()
            .map(|user| user.claims.has_group("testgroup"))
            .unwrap_or(false)
    };

    view! {
        <h1>Debug</h1>
        <table>
            <tr>
                <td><b>User name</b></td>
                <td>{ name }</td>
            </tr>
            <tr>
                <td><b>EMail</b></td>
                <td>{ email }</td>
            </tr>
            <tr>
                <td><b>Testuser</b></td>
                <td>{ tester }</td>
            </tr>
            <tr>
                <td><b>Manager</b></td>
                <td>{ manager }</td>
            </tr>
            <tr>
                <td><b>Groups</b></td>
                <td>{ groups }</td>
            </tr>
            <tr>
                <td><b>Roles</b></td>
                <td>{ roles }</td>
            </tr>
        </table>
        <AuthErrorContext fallback=|| view! { <p>No error detected.</p>}>
            <AuthErrorPage></AuthErrorPage>
        </AuthErrorContext>
        <AuthLoading><b>Authentication is loading</b></AuthLoading>
    }
}

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <Title text="Home"/>
        <h1>Home</h1>
        <p>Your Landing Page without authentication</p>
        <a href="/profile">Profile page</a>

    }
}

/// This will be rendered, if the authentication library is still loading
#[component]
pub fn Loading() -> impl IntoView {
    view! {
        <Title text="Loading"/>
        <h1>Loading</h1>
        <p>Waiting for authentication backend to respond.</p>

        // Your Loading Page/Animation
    }
}

/// This will be rendered, if the user is unauthenticated
#[component]
pub fn Unauthenticated() -> impl IntoView {
    view! {
        <Title text="Unauthenticated"/>
        <h1>Unauthenticated</h1>
        <div><a href="/">Home</a></div>
        <LoginLink class="text-login">Sign in</LoginLink>
        // Your Unauthenticated Page
    }
}

#[component]
pub fn Logout() -> impl IntoView {
    view! {
        <Title text="Logout"/>
        <h1>Logout</h1>
        <p>You were successfully logged out!</p>
        <a href="/">Home</a>

    }
}

/// This will be rendered, if the user is authentication
#[component]
pub fn Profile() -> impl IntoView {
    let user = use_context::<Signal<Option<TokenData<Claims>>>>()
        .expect("Navigation: user store should exist!");

    view! {
        <Title text="Profile"/>
        <h1>Profile</h1>

        <LogoutLink class="text-logout">Sign out</LogoutLink>
        // Your Profile Page
        { move || {
            match user.get().map(|user| user.claims) {
                None => Either::Left(view!{ <p> Token is empty</p>}),
                Some(token) => {
                    let groups = token.groups.iter().map(|group| group.trim_start_matches('/')).collect::<Vec<_>>().join(", ");
                    let roles = token.roles.into_iter().collect::<Vec<_>>().join(", ");

                    Either::Right(view! {
                        <table class="table is-stripped">
                            <tbody>
                                <tr>
                                    <td>Username</td> <td>{ token.preferred_username }</td>
                                </tr>
                                <tr>
                                    <td>Fullname</td> <td>{ token.name }</td>
                                </tr>
                                <tr>
                                    <td>Email</td> <td>{ token.email }</td>
                                </tr>
                                <tr>
                                    <td>Roles</td> <td>{ roles }</td>
                                </tr>
                                <tr>
                                    <td>Groups</td> <td>{ groups }</td>
                                </tr>
                            </tbody>
                        </table>
                    })
                }
            }
        }}

    }
}
