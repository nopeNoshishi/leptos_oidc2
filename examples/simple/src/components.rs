use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Link, Stylesheet, Title};
use leptos_oidc::{AuthInitialized, AuthParameters, Authenticated, Challenge, LoginLink, LogoutLink};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let parameters = AuthParameters {
        issuer: "http://localhost:8082/realms/master".to_string(),
        client_id: "leptos-client".to_string(),
        redirect_uri: "http://localhost:3000/profile".to_string(),
        post_logout_redirect_uri: "http://localhost:3000".to_string(),
        challenge: Challenge::S256,
        scope: Some("openid%20profile%20email".to_string()),
        audience: None,
    };

    view! {
        <Stylesheet id="leptos" href="/pkg/main.css"/>

        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>

        <Router>
            <AuthInitialized
                parameters
                fallback=Loading
            >
                <h1>Leptos OIDC</h1>

                    <Routes fallback=Home>
                        <Route path=path!("/") view=Home/>

                        // This is an example route for your profile, it will render
                        // loading if it's still loading, render unauthenticated if it's
                        // unauthenticated, and it will render the children, if it's
                        // authenticated
                        <Route
                            path=path!("/profile")
                            view=move || {
                                view! {
                                    <Authenticated
                                        unauthenticated=Unauthenticated
                                    >
                                        <Profile/>
                                    </Authenticated>
                                }
                            }
                        />
                    </Routes>
            </AuthInitialized>
        </Router>

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

// #[component]
// pub fn ErrorPage(
//     message: String,
// ) -> impl IntoView {
//     view! {
//         <Title text="Error"/>
//         <h1>Error occurred</h1>
//         <p>{ message }</p>
//
//     }
// }

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
        <div><a href="/">Home</a></div>
        <LogoutLink class="text-logout">Sign out</LogoutLink>
        // Your Profile Page
    }
}
