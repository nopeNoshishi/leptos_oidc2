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
use crate::Auth;
use leptos::attr::AttributeValue;
use leptos::prelude::*;
use leptos::tachys::html::class::IntoClass;

/// A transparent component representing authenticated user status.
/// It provides a way to conditionally render its children based on the user's authentication status.
/// If the user is authenticated, it renders the children; otherwise, it falls back to the provided loading or unauthenticated view.
#[must_use]
#[component(transparent)]
pub fn Authenticated(
    children: ChildrenFn,
    #[prop(optional, into)] loading: ViewFnOnce,
    #[prop(optional, into)] unauthenticated: ViewFn,
) -> impl IntoView {
    let auth = expect_context::<Auth>();
    let unauthenticated = move || unauthenticated.run();
    let authenticated = move || auth.authenticated();
    let children = StoredValue::new(children);

    view! {
        <Transition fallback=loading>
            <Show
                when=authenticated.clone()
                fallback=unauthenticated.clone()
            >
                { children.read_value()() }
            </Show>
        </Transition>
    }
}

/// A transparent component representing the loading state of authentication.
/// It allows rendering its children when the authentication process is loading, with an optional fallback view.
#[must_use]
#[component(transparent)]
pub fn AuthLoading(
    children: ChildrenFn,
    #[prop(optional, into)] fallback: ViewFn,
) -> impl IntoView {
    let auth = expect_context::<Auth>();
    let loading = move || auth.loading();

    view! {
        <Show when=loading fallback=fallback>
            {children()}
        </Show>
    }
}

/// A transparent component representing the loaded state of authentication.
/// It allows rendering its children when the authentication process has completed, with an optional fallback view.
#[must_use]
#[component(transparent)]
pub fn AuthLoaded(children: ChildrenFn, #[prop(optional, into)] fallback: ViewFn) -> impl IntoView {
    let auth = expect_context::<Auth>();
    let loaded = move || !auth.loading();

    view! {
        <Show when=loaded fallback=fallback>
            {children()}
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
    let auth = expect_context::<Auth>();
    let login_url = move || auth.login_url();

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
    let auth = expect_context::<Auth>();
    let logout_url = move || auth.logout_url();

    view! {
        <a href=logout_url class=class>
            {children()}
        </a>
    }
}
