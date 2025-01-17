use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Link, Stylesheet, Title};
use leptos_oidc::{Algorithm, Auth, AuthError, AuthParameters, Authenticated, LoginLink, LogoutLink};
use serde::Deserialize;
use std::sync::Arc;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use crate::user::Claims;

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
    oidc: AuthParameters
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view!{
        <Stylesheet id="leptos" href="/pkg/main.css"/>

        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>


        <Router>
            <h1>Leptos OIDC</h1>
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

#[must_use]
#[component(transparent)]
pub fn AuthLoaded(
    children: ChildrenFn,
    #[prop(optional, into)] fallback: ViewFnOnce,
) -> impl IntoView {
    let auth_resource = expect_context::<LocalResource<Result<Auth, AuthError>>>();
    let children = StoredValue::new(children);

    view! {
        <Transition fallback>
            { move || {
                Suspend::new(async move {
                    if let Ok(auth) = auth_resource.await {
                        // provides authentication data in leptos context required by login/logout button
                        provide_context(auth);
                        Either::Right(view! {
                            { children.read_value()() }
                        })
                    } else {
                        Either::Left(())
                    }
                })
            }}

        </Transition>
    }
}

#[component]
pub fn AppConfigLoaded(
    children: ChildrenFn,
    #[prop(optional, into)] fallback: ViewFnOnce,
) -> impl IntoView {
    let config: LocalResource<Result<AppConfig, AppConfigError>> = LocalResource::new(move || async {
        let app_config = gloo_net::http::Request::get("/config.json")
            .send().await
            .map_err(Arc::new)?
            .json::<AppConfig>().await
            .map_err(Arc::new)?;

        Ok(app_config)
    });
    let children = StoredValue::new(children);

    view! {
        <Transition
            fallback
        >
            { move || {
                Suspend::new(async move {
                    if let Ok(app_config) = config.await {
                        let auth_resource = Auth::init(app_config.oidc);
                        provide_context(auth_resource);
                        // provides authentication data in leptos context required by login/logout button
                        Either::Right(view! {
                            <p>Config loaded</p>
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
                None => Either::Left(()),
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
