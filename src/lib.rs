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

#![allow(clippy::module_name_repetitions)]

use std::ops::Not;
use std::sync::Arc;

use chrono::Local;
use codee::string::JsonSerdeCodec;
use jsonwebtoken::jwk::Jwk;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::web_sys::Url;
use leptos_router::hooks::{use_location, use_navigate, use_query};
use leptos_router::NavigateOptions;

use leptos_use::{
    storage::{use_local_storage, use_session_storage},
    use_timeout_fn, UseTimeoutFnReturn,
};
use response::{CallbackResponse, SuccessCallbackResponse, TokenResponse};
use serde::{Deserialize, Serialize};
use storage::{TokenStorage, CODE_VERIFIER_KEY, LOCAL_STORAGE_KEY};
use utils::ParamBuilder;

mod authenticated;
pub mod components;
pub mod error;
pub mod response;
pub mod storage;
mod unauthenticated;
pub mod utils;

use crate::authenticated::AuthenticatedData;
pub use components::*;
pub use error::AuthError;
use unauthenticated::UnauthenticatedData;

pub type Algorithm = jsonwebtoken::Algorithm;
pub type TokenData<T> = jsonwebtoken::TokenData<T>;
pub type Validation = jsonwebtoken::Validation;
#[derive(Clone, Debug)]
pub struct IssuerMetadata {
    configuration: Configuration,
    keys: Keys,
}
pub type AuthSignal = RwSignal<Auth>;

const REFRESH_TOKEN_SECONDS_BEFORE: usize = 30;

/// Represents authentication parameters required for initializing the `Auth`
/// structure. These parameters include authentication and token endpoints,
/// client ID, and other related data.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct AuthParameters {
    pub issuer: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub post_logout_redirect_uri: String,
    pub challenge: Challenge,
    pub scope: Option<String>,
    pub audience: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Challenge {
    #[default]
    S256,
    Plain,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Configuration {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub end_session_endpoint: String,
    pub jwks_uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Keys {
    keys: Vec<Jwk>,
}

// The different states of the main authentication process
#[derive(Clone, Debug, Default)]
pub enum Auth {
    #[default]
    Loading,
    Unauthenticated(UnauthenticatedData),
    Authenticated(AuthenticatedData),
    Error(AuthError),
}

impl Auth {
    #[must_use]
    pub fn get_unauthenticated(&self) -> Option<UnauthenticatedData> {
        match self {
            Auth::Unauthenticated(auth) => Some(auth.clone()),
            _ => None,
        }
    }
    #[must_use]
    pub fn authenticated(&self) -> Option<AuthenticatedData> {
        match self {
            Auth::Authenticated(auth) => Some(auth.clone()),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_loaded(&self) -> bool {
        self.is_loading().not()
    }

    #[must_use]
    pub fn is_loading(&self) -> bool {
        matches!(self, Auth::Loading)
    }
    #[must_use]
    pub fn error(&self) -> Option<AuthError> {
        match self {
            Auth::Error(auth) => Some(auth.clone()),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_authenticated(&self) -> bool {
        match self {
            Auth::Authenticated(auth) => auth.token_store.is_valid(),
            _ => false,
        }
    }
}

impl Auth {
    /// Construct the `AuthSignal` that must be provided in the context
    #[must_use]
    pub fn signal() -> AuthSignal {
        RwSignal::new(Auth::default())
    }

    /// Initializes a new `Auth` instance with the provided authentication
    /// parameters. This function creates and returns an `Auth` enum
    /// configured for authentication.
    ///
    /// # Panics
    ///
    /// The initialization panics if the user of this library
    /// did not construct the `AuthSignal` and provided it in the leptos context
    ///
    /// ```
    /// use leptos::prelude::*;
    /// use leptos_oidc::{Auth, AuthSignal};
    /// let auth: AuthSignal = Auth::signal();
    /// provide_context(auth);
    /// ```
    ///
    pub fn init(parameters: AuthParameters) {
        let auth = use_context::<AuthSignal>().expect("AuthSignal not initialized.");
        let fetch_resource = RwSignal::new(0);
        let pending_resource = RwSignal::new(true);

        // Create local resource to fetch issuer metadata and handle the state of authentication
        // This is a local resource which integrates this asynchronous task in the reactive system of leptos
        // This is required to have the ability to use navigation (use_navigate()) depending on the query parameters.
        LocalResource::new(move || {
            let _ = fetch_resource.get();
            let parameters = parameters.clone();
            async move {
                async fn init(
                    parameters: &AuthParameters,
                    auth: AuthSignal,
                ) -> Result<Auth, AuthError> {
                    let issuer = init_issuer_resource(parameters).await?;
                    let auth_result = init_auth(parameters, issuer.clone()).await?;
                    create_handle_refresh_effect(parameters.clone(), issuer, auth);
                    Ok(auth_result)
                }

                // update signal
                match init(&parameters, auth).await {
                    Ok(auth_result) => {
                        auth.set(auth_result.clone());
                        pending_resource.set(false);
                        Ok(auth)
                    }
                    Err(error) => {
                        auth.set(Auth::Error(error.clone()));
                        pending_resource.set(false);
                        Err(error)
                    }
                }
            }
        });

        // re-fetch the local resource in case the signal is set back to Auth::Loading
        Effect::new(move || {
            let signal = auth.get();
            if matches!(signal, Auth::Loading) && pending_resource.get().not() {
                pending_resource.set(true);
                let count = fetch_resource.get();
                fetch_resource.set(count + 1);
            }
        });
    }
}

/// Initialize the issuer resource, which will fetch the JWKS and endpoints.
///
async fn init_issuer_resource(parameters: &AuthParameters) -> Result<IssuerMetadata, AuthError> {
    let configuration = reqwest::Client::new()
        .get(format!(
            "{}/.well-known/openid-configuration",
            parameters.issuer
        ))
        .send()
        .await
        .map_err(Arc::new)?
        .json::<Configuration>()
        .await
        .map_err(Arc::new)?;

    let keys = reqwest::Client::new()
        .get(configuration.jwks_uri.clone())
        .send()
        .await
        .map_err(Arc::new)?
        .json::<Keys>()
        .await
        .map_err(Arc::new)?;

    Ok(IssuerMetadata {
        configuration,
        keys,
    })
}

fn check_authentication_response_url(parameters: &AuthParameters) -> bool {
    let location = use_location()
        .pathname
        .get()
        .trim_end_matches('/')
        .to_string();
    let redirect_uri = Url::new(&parameters.redirect_uri)
        .ok()
        .map_or(String::new(), |url| url.pathname());
    let logout_uri = Url::new(&parameters.post_logout_redirect_uri)
        .ok()
        .map_or(String::new(), |url| url.pathname());
    redirect_uri == location || logout_uri == location
}

/// Initialize the auth resource, which will handle the entire state of the authentication.
async fn init_auth(parameters: &AuthParameters, issuer: IssuerMetadata) -> Result<Auth, AuthError> {
    let (local_storage, set_local_storage, _remove_local_storage) =
        use_local_storage::<Option<TokenStorage>, JsonSerdeCodec>(LOCAL_STORAGE_KEY);

    let is_authentication_response_url = check_authentication_response_url(parameters);
    let auth_response = use_query::<CallbackResponse>();
    match (
        is_authentication_response_url,
        auth_response.get_untracked(),
    ) {
        (true, Ok(CallbackResponse::SuccessLogin(response))) => {
            use_navigate()(
                &parameters.redirect_uri,
                NavigateOptions {
                    resolve: false,
                    replace: true,
                    scroll: true,
                    state: leptos_router::location::State::new(None),
                },
            );

            if let Some(token_storage) = local_storage.get_untracked() {
                if token_storage.is_valid() {
                    return Ok(Auth::Authenticated(AuthenticatedData {
                        parameters: parameters.clone(),
                        issuer,
                        token_store: token_storage,
                    }));
                }
            }

            let token_storage = fetch_token(parameters, &issuer.configuration, response).await?;

            set_local_storage.set(Some(token_storage.clone()));

            Ok(Auth::Authenticated(AuthenticatedData {
                parameters: parameters.clone(),
                issuer,
                token_store: token_storage,
            }))
        }
        (true, Ok(CallbackResponse::SuccessLogout(response))) => {
            use_navigate()(
                &parameters.post_logout_redirect_uri,
                NavigateOptions {
                    resolve: false,
                    replace: true,
                    scroll: true,
                    state: leptos_router::location::State::new(None),
                },
            );
            if response.destroy_session {
                tracing::debug!("Logout: destroying session");
                set_local_storage.set(None);
                // remove_local_storage(); // does not seem to delete local storage
            }

            Ok(Auth::Unauthenticated(UnauthenticatedData {
                parameters: parameters.clone(),
                issuer,
            }))
        }
        (true, Ok(CallbackResponse::Error(error))) => Ok(Auth::Error(AuthError::Provider(error))),
        (_, _) => {
            if let Some(token_store) = local_storage.get_untracked() {
                if token_store.is_valid() {
                    Ok(Auth::Authenticated(AuthenticatedData {
                        parameters: parameters.clone(),
                        issuer,
                        token_store,
                    }))
                } else {
                    set_local_storage.set(None);
                    // remove_local_storage(); // does not seem to delete local storage
                    Ok(Auth::Unauthenticated(UnauthenticatedData {
                        parameters: parameters.clone(),
                        issuer,
                    }))
                }
            } else {
                Ok(Auth::Unauthenticated(UnauthenticatedData {
                    parameters: parameters.clone(),
                    issuer,
                }))
            }
        }
    }
}

/// This will handle the refresh, if there is a refresh token.
fn create_handle_refresh_effect(
    parameters: AuthParameters,
    issuer: IssuerMetadata,
    auth: AuthSignal,
) {
    Effect::new(move || {
        if let Some(authenticated) = auth.get().authenticated() {
            let expires_in = authenticated.token_store.expires_in - Local::now().naive_utc();
            #[allow(clippy::cast_precision_loss)]
            let wait_milliseconds =
                (expires_in.num_seconds() as f64 - REFRESH_TOKEN_SECONDS_BEFORE as f64).max(0.0)
                    * 1000.0;

            let UseTimeoutFnReturn { start, .. } = use_timeout_fn(
                move |(parameters, issuer, token_signal, refresh_token): (
                    AuthParameters,
                    IssuerMetadata,
                    AuthSignal,
                    String,
                )| {
                    spawn_local(async move {
                        let (_, set_storage, _remove_storage) =
                            use_local_storage::<Option<TokenStorage>, JsonSerdeCodec>(
                                LOCAL_STORAGE_KEY,
                            );
                        match refresh_token_request(
                            &parameters,
                            &issuer.configuration,
                            refresh_token,
                        )
                        .await
                        {
                            Ok(token_store) => {
                                // refreshing token was successful, change signal to re-run effect
                                token_signal.set(Auth::Authenticated(AuthenticatedData {
                                    parameters,
                                    issuer,
                                    token_store: token_store.clone(),
                                }));
                                set_storage.set(Some(token_store));
                            }
                            Err(error) => {
                                tracing::error!("Failed to refresh token storage: {}", error);
                                // change signal to re-run effect
                                token_signal.set(Auth::Unauthenticated(UnauthenticatedData {
                                    parameters,
                                    issuer,
                                }));
                                set_storage.set(None);
                            }
                        }
                    });
                },
                wait_milliseconds,
            );

            start((
                parameters.clone(),
                issuer.clone(),
                auth,
                authenticated.token_store.refresh_token.clone(),
            ));
        }
    });
}

/// Asynchronous function for fetching an authentication token.
/// This function is used to exchange an authorization code for an access token.
async fn fetch_token(
    parameters: &AuthParameters,
    configuration: &Configuration,
    auth_response: SuccessCallbackResponse,
) -> Result<TokenStorage, AuthError> {
    let mut body = "&grant_type=authorization_code"
        .to_string()
        .push_param_body("client_id", &parameters.client_id)
        .push_param_body("redirect_uri", &parameters.redirect_uri)
        .push_param_body("code", &auth_response.code);

    if let Some(state) = &auth_response.session_state {
        body = body.push_param_body("state", state);
    }

    let (code_verifier, _, remove_code_verifier) =
        use_session_storage::<Option<String>, JsonSerdeCodec>(CODE_VERIFIER_KEY);

    if let Some(code_verifier) = code_verifier.get_untracked() {
        body = body.push_param_body("code_verifier", code_verifier);

        remove_code_verifier();
    }

    let response = reqwest::Client::new()
        .post(configuration.token_endpoint.clone())
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(Arc::new)?
        .json::<TokenResponse>()
        .await
        .map_err(Arc::new)?;

    match response {
        TokenResponse::Success(success) => Ok(success.into()),
        TokenResponse::Error(error) => Err(AuthError::Provider(error)),
    }
}

/// Asynchronous function for re-fetching an authentication token.
/// This function is used to exchange a new access token and refresh token.
async fn refresh_token_request(
    parameters: &AuthParameters,
    configuration: &Configuration,
    refresh_token: String,
) -> Result<TokenStorage, AuthError> {
    let response = reqwest::Client::new()
        .post(configuration.token_endpoint.clone())
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(
            "&grant_type=refresh_token"
                .to_string()
                .push_param_body("client_id", &parameters.client_id)
                .push_param_body("refresh_token", refresh_token),
        )
        .send()
        .await
        .map_err(Arc::new)?
        .json::<TokenResponse>()
        .await
        .map_err(Arc::new)?;

    match response {
        TokenResponse::Success(success) => Ok(success.into()),
        TokenResponse::Error(error) => Err(AuthError::Provider(error)),
    }
}
