use crate::user::Claims;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Link, Stylesheet, Title};
use leptos_oidc::{Algorithm, Auth, AuthError, AuthErrorContext, AuthLoaded, AuthParameters, Authenticated, LoginLink, LogoutLink, TokenData};
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
pub struct AppGlobals {
    auth_resource: LocalResource<Result<Auth, AuthError>>,
}


#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let auth_store: RwSignal<Auth> = RwSignal::new(Auth::default());
    provide_context(auth_store);

    let app_globals: LocalResource<Result<AppGlobals, AppConfigError>> =
        LocalResource::new(move || async {
            let app_config = gloo_net::http::Request::get("/config.json")
                .send()
                .await
                .map_err(Arc::new)?
                .json::<AppConfig>()
                .await
                .map_err(Arc::new)?;
            let auth = Auth::init(app_config.oidc.clone());

            Ok(AppGlobals {
                auth_resource: auth,
            })
        });
    provide_context(app_globals);

    // Signal<Option<TokenData<Claims>>>
    let user = Signal::derive(move || {
        auth_store.with(|auth_store| auth_store.authenticated().map(|auth|
            auth.decoded_access_token::<Claims>(Algorithm::RS256, &["account"])
        ).flatten())
    });
    provide_context(user);
    let manager = move || user.with(|user| user.clone().map(|user| user.claims.has_group("managerrole")));

    view! {
        <Stylesheet id="leptos" href="/pkg/main.css"/>

        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>

        <Router>
            <h1>Leptos OIDC</h1>
                <Navigation/>
                <DebugInfo/>
                <Routes fallback=Home>
                    <Route path=path!("/") view=Home/>
                    <ProtectedRoute
                        path=path!("/manager")
                        view=Manager
                        condition=manager
                        redirect_path=|| "/"
                    />
                    <Route
                        path=path!("/profile")
                        view=|| {
                            view! {
                                <AppConfigLoaded>
                                    <p>Profile</p>
                                    <AuthLoaded>
                                        <Authenticated unauthenticated=Unauthenticated>
                                            <p>Profile</p>
                                            <Profile/>
                                        </Authenticated>
                                    </AuthLoaded>
                                </AppConfigLoaded>
                            }
                        }
                    />
                </Routes>
        </Router>


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
                    if let Ok(app_config) = app_globals.await {
                        let auth_resource = app_config.auth_resource;
                        //provide_context(auth_resource);
                        // provides authentication data in leptos context required by login/logout button
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
    let auth_store = use_context::<RwSignal<Auth>>().expect("AuthStore not initialized in error page");
    let error_message = move || auth_store.get().error().map(|err| format!("{err:?}"));
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
    let manager = move || user.get().map(|user| user.claims.has_role("managerrole")).unwrap_or(false);

    view! {
        <h2>Navigation</h2>
        <ul>
            <li><a href="/">Home</a></li>
            <li><a href="/profile">Profile</a></li>
            <Show when=manager>
                <li><a href="/manager">Management</a></li>
            </Show>
        </ul>
        <AppConfigLoaded fallback=|| view! { <p>Loading ..</p> }>
            Login/Logout:
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
    let groups = move || user.get().map(|user| user.claims.groups.into_iter().collect::<Vec<_>>().join(", "));
    let roles = move || user.get().map(|user| user.claims.roles.into_iter().collect::<Vec<_>>().join(", "));

    let manager = move || user.get().map(|user| user.claims.has_role("managerrole")).unwrap_or(false);
    let tester = move || user.get().map(|user| user.claims.has_group("testgroup")).unwrap_or(false);

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
        <AuthErrorContext fallback=|| view! { <p>no error context present</p>}>
            <AuthErrorPage></AuthErrorPage>
        </AuthErrorContext>
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
