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

use crate::storage::CODE_VERIFIER_KEY;
use crate::utils::ParamBuilder;
use crate::{AuthParameters, Challenge, IssuerMetadata};
use codee::string::JsonSerdeCodec;
use leptos::prelude::{GetUntracked, Set};
use leptos_use::storage::use_session_storage;
use oauth2::{PkceCodeChallenge, PkceCodeVerifier};

#[derive(Clone, Debug)]
pub struct UnauthenticatedData {
    pub(crate) parameters: AuthParameters,
    pub(crate) issuer: IssuerMetadata,
}

impl UnauthenticatedData {
    /// Generates and returns the URL for initiating the authentication process.
    /// This URL is used to redirect the user to the authentication provider's
    /// login page.
    #[must_use]
    pub fn login_url(&self) -> String {
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

        params
    }
}
