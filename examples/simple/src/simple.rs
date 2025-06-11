use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Link, Stylesheet, Title};
use leptos_oidc::{Auth, AuthErrorContext, AuthLoaded, AuthLoading, AuthParameters, AuthSignal, Authenticated, Challenge, LoginLink, LogoutLink};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;


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
    provide_meta_context();
    let parameters = AuthParameters {
        issuer: "http://localhost:8082/realms/master".to_string(),
        client_id: "leptos-client".to_string(),
        redirect_uri: "http://localhost:3001/profile".to_string(),
        post_logout_redirect_uri: "http://localhost:3001/logout".to_string(),
        challenge: Challenge::S256,
        scope: Some("openid%20profile%20email".to_string()),
        audience: None,
    };
    let auth: AuthSignal = Auth::signal();
    provide_context(auth);

    let _ = Auth::init(parameters);

    view! {
        <AuthLoading><p>Authentication is loading</p></AuthLoading>
        <AuthErrorContext><AuthErrorPage></AuthErrorPage></AuthErrorContext>

        <h1>Leptos OIDC</h1>
        <Routes fallback=Home>
            <Route path=path!("/") view=Home/>
            <Route path=path!("/logout") view=Logout/>

            // This is an example route for your profile, it will render
            // loading if it's still loading, render unauthenticated if it's
            // unauthenticated, and it will render the children, if it's
            // authenticated
            <Route
                path=path!("/profile")
                view=|| {
                    view! {
                        <p>Profile page</p>
                        <AuthLoaded fallback=Loading>
                            <Authenticated unauthenticated=Unauthenticated>
                                <Profile/>
                            </Authenticated>
                        </AuthLoaded>
                    }
                }
            />
        </Routes>
    }
}


#[component]
pub fn AuthErrorPage() -> impl IntoView {
    let auth = use_context::<AuthSignal>()
        .expect("AuthErrorContext: RwSignal<AuthStore> not present");
    let error_message = move || {
        auth
            .get()
            .error()
            .map(|error| format!("{error:?}"))
    };

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

#[component]
pub fn Logout() -> impl IntoView {
    view! {
        <Title text="Logout"/>
        <h1>Logout</h1>
        <p>You were successfully logged out!</p>
        <a href="/">Home</a>

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
    view! {
        <Title text="Profile"/>
        <h1>Profile</h1>
        <div><a href="/">Home</a></div>
        <LogoutLink class="text-logout">Sign out</LogoutLink>
        // Your Profile Page
    }
}
