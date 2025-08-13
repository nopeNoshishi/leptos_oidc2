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

use chrono::{Local, NaiveDateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

use crate::response::SuccessTokenResponse;

/// The key used for storing authentication token data in local storage.
pub(crate) const LOCAL_STORAGE_KEY: &str = "auth";
pub(crate) const CODE_VERIFIER_KEY: &str = "code_verifier";

/// A structure representing the storage of authentication tokens.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct TokenStorage {
    pub id_token: String,
    pub access_token: String,
    pub expires_in: NaiveDateTime,
    pub refresh_token: String,
    pub refresh_expires_in: Option<NaiveDateTime>,
}

impl TokenStorage {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.expires_in >= Local::now().naive_utc()
    }

    pub fn is_refresh_token_valid(&self) -> bool {
        self.refresh_expires_in
            .is_some_and(|exp| exp >= Local::now().naive_utc())
    }
}

/// Converts a `SuccessTokenResponse` into a `TokenStorage` structure.
impl From<SuccessTokenResponse> for TokenStorage {
    fn from(value: SuccessTokenResponse) -> Self {
        Self {
            id_token: value.id_token,
            access_token: value.access_token,
            // Backend will validate that token is valid: iat field (issued at in seconds since epoch) < now < exp field (expiration time in seconds since epoch) and token signature
            // This shall memorize when to get a new access token
            expires_in: Utc::now().naive_utc()
                + TimeDelta::try_seconds(value.expires_in).unwrap_or_default(),
            refresh_token: value.refresh_token,
            refresh_expires_in: value.refresh_expires_in.map(|refresh_expires_in| {
                Utc::now().naive_utc()
                    + TimeDelta::try_seconds(refresh_expires_in).unwrap_or_default()
            }),
        }
    }
}
