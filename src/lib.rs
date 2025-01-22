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
use leptos_router::hooks::{use_navigate, use_query};
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
pub type TokenStorageResult = Result<Option<TokenStorage>, AuthError>;

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
    /// Initializes a new `Auth` instance with the provided authentication
    /// parameters. This function creates and returns an `Auth` enum
    /// configured for authentication.
    ///
    /// # Panics
    ///
    /// The initialization panics if the user of this library
    /// did construct the `RwSignal<Auth>` and provided it in the leptos context
    ///
    /// ```
    /// use leptos::prelude::*;
    /// use leptos_oidc::Auth;
    /// let auth_store: RwSignal<Auth> = RwSignal::new(Auth::default());
    /// provide_context(auth_store);
    /// ```
    ///
    #[must_use]
    pub fn init(parameters: AuthParameters) -> LocalResource<Result<Auth, AuthError>> {
        tracing::debug!("Auth resource initialized.");

        let auth_resource = LocalResource::new(move || {
            let auth_store_signal =
                use_context::<RwSignal<Auth>>().expect("RwSignal<Auth> not initialized.");
            let parameters = parameters.clone();
            async move {
                async fn init(
                    parameters: &AuthParameters,
                    auth_store_signal: RwSignal<Auth>,
                ) -> Result<Auth, AuthError> {
                    let issuer = init_issuer_resource(parameters).await?;
                    let auth_store = init_auth_store(parameters, issuer.clone()).await?;
                    create_handle_refresh_effect(parameters.clone(), issuer, auth_store_signal);
                    Ok(auth_store)
                }

                // update signal
                match init(&parameters, auth_store_signal).await {
                    Ok(auth_store) => {
                        auth_store_signal.set(auth_store.clone());
                        Ok(auth_store)
                    }
                    Err(error) => {
                        auth_store_signal.set(Auth::Error(error.clone()));
                        Err(error)
                    }
                }
            }
        });
        provide_context(auth_resource);

        // let load_resource = Action::new(|resource: &LocalResource<Result<Auth, AuthError>>| {
        //     let resource = *resource;
        //     async move {
        //         tracing::debug!("Trigger loading auth resource.");
        //         let result = resource.await;
        //         match result {
        //             Ok(_) => {
        //                 tracing::debug!("Successfully loaded auth resource.");
        //             }
        //             Err(error) => {
        //                 tracing::info!("Error occurred while loading auth resource. {error:?}");
        //             }
        //         }
        //     }
        // });
        // load_resource.dispatch(auth_resource);

        auth_resource
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

/// Initialize the auth resource, which will handle the entire state of the authentication.
async fn init_auth_store(
    parameters: &AuthParameters,
    issuer: IssuerMetadata,
) -> Result<Auth, AuthError> {
    let (local_storage, set_local_storage, _remove_local_storage) =
        use_local_storage::<Option<TokenStorage>, JsonSerdeCodec>(LOCAL_STORAGE_KEY);

    let auth_response = use_query::<CallbackResponse>();
    match auth_response.get_untracked() {
        Ok(CallbackResponse::SuccessLogin(response)) => {
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
        Ok(CallbackResponse::SuccessLogout(response)) => {
            tracing::debug!("Logout redirect");
            use_navigate()(
                &parameters.post_logout_redirect_uri,
                NavigateOptions {
                    resolve: false,
                    replace: true,
                    scroll: true,
                    state: leptos_router::location::State::new(None),
                },
            );
            tracing::debug!("Logout: before destroying session");
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
        Ok(CallbackResponse::Error(error)) => Ok(Auth::Error(AuthError::Provider(error))),
        Err(_no_query_parameters) => {
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

/// This will handle the refresh, if there is an refresh token.
fn create_handle_refresh_effect(
    parameters: AuthParameters,
    issuer: IssuerMetadata,
    token_storage_signal: RwSignal<Auth>,
) {
    Effect::new(move || {
        if let Some(authenticated) = token_storage_signal.get().authenticated() {
            let expires_in = authenticated.token_store.expires_in - Local::now().naive_utc();
            #[allow(clippy::cast_precision_loss)]
            let wait_milliseconds =
                (expires_in.num_seconds() as f64 - REFRESH_TOKEN_SECONDS_BEFORE as f64).max(0.0)
                    * 1000.0;

            let UseTimeoutFnReturn { start, .. } = use_timeout_fn(
                move |(parameters, issuer, token_signal, refresh_token): (
                    AuthParameters,
                    IssuerMetadata,
                    RwSignal<Auth>,
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
                token_storage_signal,
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
