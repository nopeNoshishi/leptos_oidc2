use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Link, Stylesheet, Title};
use leptos_oidc::{Auth, AuthError, AuthErrorContext, AuthLoaded, AuthLoading, AuthParameters, Authenticated, Challenge, LoginLink, LogoutLink};
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
    let auth_store: RwSignal<Auth> = RwSignal::new(Auth::default());
    provide_context(auth_store);

    let _ = Auth::init(parameters);

    view! {
        <Stylesheet id="leptos" href="/pkg/main.css"/>

        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>

        <Router>
            <AuthLoading><p>Authentication is loading</p></AuthLoading>
            <AuthErrorContext><AuthErrorPage></AuthErrorPage></AuthErrorContext>

            <h1>Leptos OIDC</h1>
                <Routes fallback=Home>
                    <Route path=path!("/") view=Home/>

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
        </Router>

    }
}

#[component]
pub fn ReloadAuthButton() -> impl IntoView {
    let auth_resource = use_context::<LocalResource<Result<Auth, AuthError>>>()
        .expect("Local resource of Result<AuthStore, AuthError> not found!");

    view! {
        <button
            on:click=move |_| {
                auth_resource.refetch();
            }
        >
            Reload
        </button>
    }
}

#[component]
pub fn AuthErrorPage() -> impl IntoView {
    let auth_store = use_context::<RwSignal<Auth>>()
        .expect("AuthErrorContext: RwSignal<AuthStore> not present");
    let error_message = move || {
        auth_store
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
