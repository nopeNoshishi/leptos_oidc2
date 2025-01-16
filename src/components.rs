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

use crate::{Auth, AuthParameters};
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
    let auth = expect_context::<Auth>();
    let unauthenticated = move || unauthenticated.run();
    let authenticated = move || auth.authenticated();
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
pub fn AuthInitialized(
    children: ChildrenFn,
    parameters: AuthParameters,
    #[prop(optional, into)] fallback: ViewFnOnce,
) -> impl IntoView {
    let auth = Auth::init(parameters);
    let children = StoredValue::new(children);

    view! {
        <Suspense fallback>
            { move || {
                Suspend::new(async move {
                    match auth.await {
                        Ok(auth) => {
                            // provides authentication data in leptos context
                            provide_context(auth);
                            Either::Right(view! {
                                { children.read_value()() }
                            })
                        }
                        Err(auth_error) => {
                            let details = format!("{auth_error:?}");
                            let message = "Failed to load identity provider metadata".to_string();
                            Either::Left(view! {
                                <ErrorPage message=message details/>
                            })
                        }
                    }
                })
            }}

        </Suspense>
    }
}

#[component]
pub fn ErrorPage(message: String, details: String) -> impl IntoView {
    view! {
        <h1>Error occurred</h1>
        <p>{ message }</p>
        <p>{ details }</p>
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
