use crate::user::Claims;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Link, Stylesheet, Title};
use leptos_oidc::{
    Algorithm, Auth, AuthError, AuthLoaded, AuthParameters, Authenticated, LoginLink, LogoutLink,
};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use serde::Deserialize;
use std::ops::Deref;
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

impl Deref for AppGlobals {
    type Target = LocalResource<Result<Auth, AuthError>>;
    fn deref(&self) -> &Self::Target {
        &self.auth_resource
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
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
    let (claims, set_claims) = signal::<Option<Claims>>(None);

    Effect::new(move || match app_globals.get() {
        None => {}
        Some(globals) => {
            if let Ok(globals) = &*globals {
                let auth_resource = globals.get();
                match auth_resource {
                    None => {}
                    Some(auth) => {
                        if let Ok(auth) = &*auth {
                            let token =
                                auth.decoded_access_token::<Claims>(Algorithm::RS256, &["account"]);
                            let claims = token.map(|token| token.claims);
                            set_claims.set(claims);
                        }
                    }
                }
            }
        }
    });

    let claims_string = move || claims.get().map(|claim| format!("{claim:?}"));
    let testgroup =
        move || claims.with(|claim| claim.clone().map(|claim| claim.has_group("testgroup")));
    let manager =
        move || claims.with(|claim| claim.clone().map(|claim| claim.has_role("managerrole")));

    view! {
        <Stylesheet id="leptos" href="/pkg/main.css"/>

        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>


        <Router>
            <h1>Leptos OIDC</h1>
                <p>Claims: { claims_string }</p>
                <p>Test group: { testgroup }</p>
                <p>manager: { manager }</p>
                <Routes fallback=Home>
                    <Route path=path!("/") view=Home/>
                    <Route
                        path=path!("/profile")
                        view=|| {
                            view! {
                                <AppConfigLoaded>
                                    <Authenticated unauthenticated=Unauthenticated>
                                        <Profile/>
                                    </Authenticated>
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
    let app_globals = expect_context::<LocalResource<Result<AppGlobals, AppConfigError>>>();
    let children = StoredValue::new(children);

    view! {
        <Transition
            fallback
        >
            { move || {
                Suspend::new(async move {
                    if let Ok(app_config) = app_globals.await {
                        let auth_resource = app_config.auth_resource;
                        provide_context(auth_resource);
                        // provides authentication data in leptos context required by login/logout button
                        Either::Right(view! {
                            <AuthLoaded fallback=Loading>
                                { children.read_value()() }
                            </AuthLoaded>
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
pub fn AuthErrorPage() -> impl IntoView {
    let auth_error = expect_context::<AuthError>();
    let error_message = format!("{auth_error:?}");
    view! {
        <h1>Error occurred</h1>
        <p>There was an error in the authentication process!</p>
        { error_message }
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
    let auth = expect_context::<Auth>();
    let token = auth.decoded_access_token::<Claims>(Algorithm::RS256, &["account"]);
    view! {
        <Title text="Profile"/>
        <h1>Profile</h1>
        <div><a href="/">Home</a></div>
        <LogoutLink class="text-logout">Sign out</LogoutLink>
        // Your Profile Page
        {
            match token {
                None => Either::Left(view!{ <p> Token is empty</p>}),
                Some(token) => {
                    let groups = token.claims.groups.iter().map(|group| group.trim_start_matches('/')).collect::<Vec<_>>().join(", ");
                    let roles = token.claims.roles.into_iter().collect::<Vec<_>>().join(", ");

                    Either::Right(view! {
                        <table class="table is-stripped">
                            <tbody>
                                <tr>
                                    <td>Username</td> <td>{ token.claims.preferred_username }</td>
                                </tr>
                                <tr>
                                    <td>Fullname</td> <td>{ token.claims.name }</td>
                                </tr>
                                <tr>
                                    <td>Email</td> <td>{ token.claims.email }</td>
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
        }

    }
}
