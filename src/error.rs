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

use std::sync::Arc;

use thiserror::Error;

use crate::response::ErrorResponse;

/// An enumeration representing various authentication-related errors.
#[derive(Debug, Clone, Error)]
pub enum AuthError {
    /// An error caused by the authentication provider.
    #[error("provider error {0:?}")]
    Provider(ErrorResponse),

    /// An error related to a network request.
    #[error("request error: {0}")]
    Request(#[from] Arc<reqwest::Error>),

    /// An error related to handling parameters.
    #[error("params error: {0}")]
    Params(#[from] leptos_router::params::ParamsError),

    /// An error related to the serialization or deserialization of JSON data.
    #[error("failed to serialize/deserialilze json: {0}")]
    Serde(#[from] Arc<serde_json::Error>),

    /// An error indicating the inability to initialize local storage.
    #[error("unable to initialize local storage")]
    Storage,
}
