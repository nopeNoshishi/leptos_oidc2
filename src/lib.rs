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

use std::sync::Arc;

use chrono::Local;
use codee::string::JsonSerdeCodec;
use jsonwebtoken::{decode, jwk::Jwk, DecodingKey};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_navigate, use_query};
use leptos_router::NavigateOptions;

use leptos_use::{
    storage::{use_local_storage, use_session_storage},
    use_timeout_fn, UseTimeoutFnReturn,
};
use oauth2::{PkceCodeChallenge, PkceCodeVerifier};
use response::{CallbackResponse, SuccessCallbackResponse, TokenResponse};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use storage::{TokenStorage, CODE_VERIFIER_KEY, LOCAL_STORAGE_KEY};
use utils::ParamBuilder;

pub mod components;
pub mod error;
pub mod response;
pub mod storage;
pub mod utils;

pub use components::*;
pub use error::AuthError;

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

/// Authentication handler responsible for handling user authentication and
/// token management.
#[derive(Clone, Debug)]
pub struct Auth {
    parameters: AuthParameters,
    issuer: IssuerMetadata,
    token_store: RwSignal<TokenStorageResult>,
}

trait DecodeTokenStorage {
    fn decode_token_storage(&self) -> Option<TokenStorage>;
}

impl DecodeTokenStorage for RwSignal<TokenStorageResult> {
    fn decode_token_storage(&self) -> Option<TokenStorage> {
        self.get().ok().flatten()
    }
}

impl Auth {
    /// Initializes a new `Auth` instance with the provided authentication
    /// parameters. This function creates and returns an `Auth` struct
    /// configured for authentication.
    #[must_use]
    pub fn init(parameters: AuthParameters) -> LocalResource<Result<Self, AuthError>> {
        LocalResource::new(move || {
            let parameters = parameters.clone();
            async move {
                let issuer = init_issuer_resource(&parameters).await?;
                let token_store =
                    RwSignal::new(init_auth_resource(&parameters, &issuer.configuration).await);

                create_handle_refresh_effect(
                    parameters.clone(),
                    issuer.configuration.clone(),
                    token_store,
                );
                Ok(Self {
                    parameters,
                    issuer,
                    token_store,
                })
            }
        })
    }

    /// Generates and returns the URL for initiating the authentication process.
    /// This URL is used to redirect the user to the authentication provider's
    /// login page.
    #[must_use]
    pub fn login_url(&self) -> Option<String> {
        let mut params = self
            .issuer
            .configuration
            .authorization_endpoint
            .clone()
            .push_param_query("response_type", "code")
            .push_param_query("client_id", &self.parameters.client_id)
            .push_param_query("redirect_uri", &self.parameters.redirect_uri)
            .push_param_query(
                "scope",
                self.parameters
                    .scope
                    .clone()
                    .unwrap_or("openid".to_string()),
            );

        if let Some(audience) = &self.parameters.audience {
            params = params.push_param_query("audience", audience);
        }

        let (code_verifier, set_code_verifier, remove_code_verifier) =
            use_session_storage::<Option<String>, JsonSerdeCodec>(CODE_VERIFIER_KEY);

        match &self.parameters.challenge {
            Challenge::S256 | Challenge::Plain => {
                let code_challenge =
                    if let Some(code_verifier_secret) = code_verifier.get_untracked() {
                        let verifier = PkceCodeVerifier::new(code_verifier_secret);
                        if self.parameters.challenge == Challenge::S256 {
                            PkceCodeChallenge::from_code_verifier_sha256(&verifier)
                        } else {
                            PkceCodeChallenge::from_code_verifier_plain(&verifier)
                        }
                    } else {
                        let (code, verifier) = if self.parameters.challenge == Challenge::S256 {
                            PkceCodeChallenge::new_random_sha256()
                        } else {
                            PkceCodeChallenge::new_random_plain()
                        };
                        set_code_verifier.set(Some(verifier.secret().to_owned()));
                        code
                    };
                params = params.push_param_query("code_challenge", code_challenge.as_str());
                params = params
                    .push_param_query("code_challenge_method", code_challenge.method().as_str());
            }
            Challenge::None => {
                remove_code_verifier();
            }
        }

        Some(params)
    }

    /// Generates and returns the URL for initiating the logout process. This
    /// URL is used to redirect the user to the authentication provider's logout
    /// page.
    #[must_use]
    pub fn logout_url(&self) -> Option<String> {
        let url = self
            .issuer
            .configuration
            .end_session_endpoint
            .clone()
            .push_param_query(
                "post_logout_redirect_uri",
                self.parameters
                    .post_logout_redirect_uri
                    .clone()
                    .push_param_query("destroy_session", "true"),
            );

        if let Ok(Some(token)) = &self.token_store.get() {
            return Some(url.push_param_query("id_token_hint", &token.id_token));
        }

        Some(url)
    }

    /// Checks if the user is authenticated.
    #[must_use]
    pub fn authenticated(&self) -> bool {
        self.token_store
            .decode_token_storage()
            .is_some_and(|storage| storage.is_valid())
    }

    /// Returns the ID token, if available, from the authentication response.
    #[must_use]
    pub fn id_token(&self) -> Option<String> {
        self.token_store
            .decode_token_storage()
            .map(|response| response.id_token)
    }

    /// Returns the access token, if available, from the authentication response.
    #[must_use]
    pub fn access_token(&self) -> Option<String> {
        self.token_store
            .decode_token_storage()
            .map(|response| response.access_token)
    }

    /// Returns the decoded id token, if available, from the authentication response.
    #[must_use]
    pub fn decoded_id_token<T: DeserializeOwned>(
        &self,
        algorithm: Algorithm,
        audience: &[&str],
    ) -> Option<TokenData<T>> {
        let token = self.token_store.decode_token_storage()?.id_token;
        self.decode_token(algorithm, audience, &token)
    }

    /// Returns the decoded access token, if available, from the authentication response.
    #[must_use]
    pub fn decoded_access_token<T: DeserializeOwned>(
        &self,
        algorithm: Algorithm,
        audience: &[&str],
    ) -> Option<TokenData<T>> {
        let token = self.token_store.decode_token_storage()?.access_token;
        self.decode_token(algorithm, audience, &token)
    }

    fn decode_token<T: DeserializeOwned>(
        &self,
        algorithm: Algorithm,
        audience: &[&str],
        token: &str,
    ) -> Option<TokenData<T>> {
        let mut validation = Validation::new(algorithm);
        validation.set_audience(audience);

        for key in &self.issuer.keys.keys {
            let Ok(decoding_key) = DecodingKey::from_jwk(key) else {
                continue;
            };

            match decode::<T>(token, &decoding_key, &validation) {
                Ok(data) => return Some(data),
                Err(_) => continue,
            }
        }
        None
    }

    /// This can be used to set the `redirect_uri` dynamically. It's helpful if
    /// you would like to be redirected to the current page.
    pub fn set_redirect_uri(&mut self, uri: String) {
        self.parameters.redirect_uri = uri;
    }
}

/// Initialize the issuer resource, which will fetch the JWKS and endpoints.
///
/// # Panics
///
/// The init function can panic when the issuer and jwks could not be fetched successfully.
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
async fn init_auth_resource(
    parameters: &AuthParameters,
    configuration: &Configuration,
) -> TokenStorageResult {
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
                    return Ok(Some(token_storage));
                }
            }

            let token_storage = fetch_token(parameters, configuration, response).await?;

            set_local_storage.set(Some(token_storage.clone()));

            Ok(Some(token_storage))
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
                //remove_local_storage(); // does not seem to delete local storage
            }

            Ok(None)
        }
        Ok(CallbackResponse::Error(error)) => Err(AuthError::Provider(error)),
        Err(_no_query_parameters) => {
            if let Some(token_storage) = local_storage.get_untracked() {
                if token_storage.is_valid() {
                    Ok(Some(token_storage))
                } else {
                    set_local_storage.set(None);
                    // remove_local_storage(); TODO: remove
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        }
    }
}

/// This will handle the refresh, if there is an refresh token.
fn create_handle_refresh_effect(
    parameters: AuthParameters,
    configuration: Configuration,
    token_storage_signal: RwSignal<TokenStorageResult>,
) {
    Effect::new(move || {
        if let Ok(Some(token_storage)) = token_storage_signal.get() {
            let expires_in = token_storage.expires_in - Local::now().naive_utc();
            #[allow(clippy::cast_precision_loss)]
            let wait_milliseconds =
                (expires_in.num_seconds() as f64 - REFRESH_TOKEN_SECONDS_BEFORE as f64).max(0.0)
                    * 1000.0;

            let UseTimeoutFnReturn { start, .. } = use_timeout_fn(
                move |(parameters, configuration, token_signal, refresh_token): (
                    AuthParameters,
                    Configuration,
                    RwSignal<TokenStorageResult>,
                    String,
                )| {
                    spawn_local(async move {
                        let (_, set_storage, remove_storage) =
                            use_local_storage::<Option<TokenStorage>, JsonSerdeCodec>(
                                LOCAL_STORAGE_KEY,
                            );
                        match refresh_token_request(&parameters, &configuration, refresh_token)
                            .await
                            .map(Some)
                        {
                            Ok(token_storage) => {
                                token_signal.set(Ok(token_storage.clone())); // change signal to re-run effect
                                set_storage.set(token_storage);
                            }
                            Err(error) => {
                                token_signal.set(Err(error)); // change signal to re-run effect
                                remove_storage();
                            }
                        }
                    });
                },
                wait_milliseconds,
            );

            start((
                parameters.clone(),
                configuration.clone(),
                token_storage_signal,
                token_storage.refresh_token.clone(),
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
