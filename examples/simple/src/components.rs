use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Link, Stylesheet, Title};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use leptos_oidc::{Auth, AuthParameters, Authenticated, Challenge, LoginLink, LogoutLink};

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
    // Specify OIDC authentication parameters here.
    // Note: This is an example for keycloak, please change it to your needs
    let auth_parameters = AuthParameters {
        issuer: "http://localhost:8082/realms/master".to_string(),
        client_id: "leptos".to_string(),
        redirect_uri: "http://localhost:3000/profile".to_string(),
        post_logout_redirect_uri: "http://localhost:3000/".to_string(),
        challenge: Challenge::S256,
        scope: Some("openid%20profile%20email".to_string()),
        audience: None,
    };
    let auth = Auth::init(auth_parameters);

    view! {
        // This is an example for a navbar where you have a login and logout
        // button, based on the state.
        <Suspense
            fallback=move || view! { <p>"Loading..."</p> }
        >
            { move || {
                Suspend::new(async move {
                    provide_context(auth.await);  // provides authentication data in leptos context

                    view! {
                        <h1>Leptos OIDC</h1>

                        <Routes fallback=Home>
                            <Route path=path!("/") view=Home/>

                            // This is an example route for your profile, it will render
                            // loading if it's still loading, render unauthenticated if it's
                            // unauthenticated and it will render the children, if it's
                            // authenticated
                            <Route
                                path=path!("/profile")
                                view=move || {
                                    view! {
                                        <h2>Profile page</h2>
                                        <Authenticated
                                            loading=move || view! { <Loading/> }
                                            unauthenticated=move || view! { <Unauthenticated/> }
                                        >
                                            <Profile/>
                                        </Authenticated>
                                    }
                                }
                            />
                        </Routes>
                    }
                })
            }}

        </Suspense>

    }
}

#[component]
pub fn Home() -> impl IntoView {
    let auth = expect_context::<Auth>();

    view! {
        <Title text="Home"/>
        <h1>Home</h1>

        // Your Some Page without authentication
    }
}

/// This will be rendered, if the authentication library is still loading
#[component]
pub fn Loading() -> impl IntoView {
    view! {
        <Title text="Loading"/>
        <h1>Loading</h1>

        // Your Loading Page/Animation
    }
}

/// This will be rendered, if the user is unauthenticated
#[component]
pub fn Unauthenticated() -> impl IntoView {
    view! {
        <Title text="Unauthenticated"/>
        <h1>Unauthenticated</h1>
        <LoginLink class="text-login">Sign in</LoginLink>
        // Your Unauthenticated Page
    }
}

/// This will be rendered, if the user is authentication
#[component]
pub fn Profile() -> impl IntoView {
    view! {
        <Title text="Profile"/>
        <h1>Profile</h1>
        <LogoutLink class="text-logout">Sign out</LogoutLink>
        // Your Profile Page
    }
}
