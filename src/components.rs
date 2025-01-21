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

use crate::{Auth, AuthError, AuthStore};
use leptos::either::Either;
use leptos::prelude::*;

/// A transparent component representing authenticated user status.
/// It provides a way to conditionally render its children based on the user's authentication status.
/// If the user is authenticated, it renders the children; otherwise, it falls back to the provided loading or unauthenticated view.
#[must_use]
#[component(transparent)]
pub fn Authenticated(
    children: ChildrenFn,
    #[prop(optional, into)] unauthenticated: ViewFn,
) -> impl IntoView {
    let auth_store = use_context::<RwSignal<AuthStore>>().expect("AuthStore not initialized in Authenticated component");
    let unauthenticated = move || unauthenticated.run();
    let authenticated = move || auth_store.get().is_authenticated();
    let children = StoredValue::new(children);

    view! {
        <Show
            when=authenticated.clone()
            fallback=unauthenticated.clone()
        >
            { children.read_value()() }
        </Show>
    }
}

#[must_use]
#[component(transparent)]
pub fn AuthLoaded(
    children: ChildrenFn,
    #[prop(optional, into)] fallback: ViewFnOnce,
) -> impl IntoView {
    let auth_resource = use_context::<LocalResource<Result<Auth, AuthError>>>()
        .expect("AuthLoaded: Local resource of auth does not exist");
    let children = StoredValue::new(children);

    view! {
        <Suspense fallback>
            { move || {
                Suspend::new(async move {
                    let auth_store = use_context::<RwSignal<AuthStore>>().expect("AuthStore not present in AuthLoaded");
                    match auth_resource.await {
                        Ok(auth) => {
                            tracing::info!("AuthLoaded: Authenticated!");
                            // provide auth state object to context
                            auth_store.set(AuthStore::Authenticated(auth));
                            Either::Right(view! {
                                { children.read_value()() }
                            })
                        }
                        Err(error) => {
                            tracing::error!("AuthLoaded: Error loading authentication {}", error);
                            // provide error object to context
                            auth_store.set(AuthStore::Error(error));
                            Either::Left(())
                        }
                    }
                })
            }}

        </Suspense>
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
    let auth = use_context::<RwSignal<AuthStore>>().expect("AuthStore not present in LoginLink");
    let login_url = move || auth.get().get_token().expect("LoginLink should be wrapped in authenticated component!").login_url();

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
    let auth = use_context::<RwSignal<AuthStore>>().expect("AuthStore not present in LogoutLink");
    let logout_url = move || auth.get().get_token().expect("LoginLink should be wrapped in authenticated component!").logout_url();

    view! {
        <a href=logout_url class=class>
            {children()}
        </a>
    }
}

#[must_use]
#[component(transparent)]
pub fn AuthLoading(children: ChildrenFn) -> impl IntoView {
    let auth_resource = use_context::<LocalResource<Result<Auth, AuthError>>>()
        .expect("AuthLoading: Local resource of auth does not exist");

    view! {
        <Suspense fallback=move || children() >
            { move || {
                Suspend::new(async move {
                    let _ = auth_resource.await;
                })
            }}
        </Suspense>
    }
}

#[must_use]
#[component(transparent)]
pub fn AuthErrorContext(
    children: ChildrenFn,
    #[prop(optional, into)] fallback: ViewFn,
) -> impl IntoView {
    let auth_store = use_context::<RwSignal<AuthStore>>()
        .expect("AuthErrorContext: RwSignal<AuthStore> not present");
    let error = move || auth_store.get().get_error();
    let is_error = move || auth_store.get().get_error().is_some();

    view! {
        <Show when=is_error fallback=fallback >
            { children() }
        </Show>
    }
}
