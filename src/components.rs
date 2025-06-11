/*
* The MIT License (MIT)
*
* Copyright (c) 2023 Daniél Kerkmann <daniel@kerkmann.dev>
*
* Permission is hereby granted, free of charge, to any person obtaining a copy
* of this software and associated documentation files (the "Software"), to deal
* in the Software without restriction, including without limitation the rights
* to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
* copies of the Software, and to permit persons to whom the Software is
* furnished to do so, subject to the following conditions:
*
* The above copyright notice and this permission notice shall be included in all
* copies or substantial portions of the Software.
*
* THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
* IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
* FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
* AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
* LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
* OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
* SOFTWARE.
*/
#![allow(clippy::must_use_candidate)]

use crate::{Auth, AuthSignal};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

/// A transparent component representing authenticated user status.
/// It provides a way to conditionally render its children based on the user's authentication status.
/// If the user is authenticated, it renders the children; otherwise, it falls back to the provided loading or unauthenticated view.
#[must_use]
#[component(transparent)]
pub fn Authenticated(
    children: ChildrenFn,
    #[prop(optional, into)] unauthenticated: ViewFn,
) -> impl IntoView {
    let auth =
        use_context::<AuthSignal>().expect("AuthSignal not initialized in Authenticated component");
    let unauthenticated = move || unauthenticated.run();
    let authenticated = move || auth.get().is_authenticated();
    let children = StoredValue::new(children);

    view! {
        <Show
            when=authenticated
            fallback=unauthenticated
        >
            { children.read_value()() }
        </Show>
    }
}

#[must_use]
#[component(transparent)]
pub fn AuthLoaded(children: ChildrenFn, #[prop(optional, into)] fallback: ViewFn) -> impl IntoView {
    let auth =
        use_context::<AuthSignal>().expect("AuthSignal not initialized in AuthLoaded component");
    let children = StoredValue::new(children);
    let loaded = move || auth.get().is_loaded();

    view! {
        <Show when=loaded fallback>
            { children.read_value()() }
        </Show>
    }
}

/// A transparent component representing a login link.
/// It generates a login URL and renders a link with the provided children and optional CSS class.
#[must_use]
#[component(transparent)]
pub fn LoginLink(
    children: Children,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let auth = use_context::<AuthSignal>().expect("AuthSignal not present in LoginLink");
    let login_url = move || {
        auth.with(|auth| {
            auth.unauthenticated()
                .map(|unauthenticated| unauthenticated.login_url())
        })
    };

    view! {
        <a href=login_url class=class>
            {children()}
        </a>
    }
}

/// A transparent component representing a logout link.
/// It generates a logout URL and renders a link with the provided children and optional CSS class.
#[must_use]
#[component(transparent)]
pub fn LogoutLink(
    children: Children,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let auth = use_context::<AuthSignal>().expect("AuthSignal not present in LogoutLink");
    let logout_url = move || {
        auth.get()
            .authenticated()
            .map(|authenticated| authenticated.logout_url())
    };

    view! {
        <a href=logout_url class=class>
            {children()}
        </a>
    }
}

#[must_use]
#[component(transparent)]
pub fn AuthLoading(children: ChildrenFn) -> impl IntoView {
    let auth =
        use_context::<AuthSignal>().expect("AuthSignal not initialized in AuthLoaded component");
    let children = StoredValue::new(children);
    let loading = move || auth.get().is_loading();

    view! {
        <Show when=loading fallback=|| ()>
            { children.read_value()() }
        </Show>
    }
}

#[must_use]
#[component(transparent)]
pub fn AuthErrorContext(
    children: ChildrenFn,
    #[prop(optional, into)] fallback: ViewFn,
) -> impl IntoView {
    let auth =
        use_context::<AuthSignal>().expect("AuthErrorContext: RwSignal<AuthSignal> not present");
    let is_error = move || auth.get().error().is_some();

    view! {
        <Show when=is_error fallback=fallback >
            { children() }
        </Show>
    }
}

#[must_use]
#[component]
pub fn ReloadButton(#[prop(optional, into)] path: Option<String>) -> impl IntoView {
    let auth = use_context::<AuthSignal>().expect("AuthSignal not initialized in ReloadButton");
    let navigate = use_navigate();
    // Navigate to following address to trigger an error state, then use reload button:
    //   http://localhost:3000/profile?error=foo&error_description=bla
    // Destroy session:
    //   http://localhost:3000/profile?destroy_session=true
    let path = path.unwrap_or("/".to_string());

    view! {
        <button
            on:click=move |_| {
                // trigger reload of authentication
                auth.set(Auth::Loading);
                // remove query parameters by navigating back to home
                navigate(&path, NavigateOptions::default());
            }
        >
            "Reload"
        </button>
    }
}
