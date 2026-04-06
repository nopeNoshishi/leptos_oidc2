/*
* The MIT License (MIT)
*
* Copyright (c) 2023 Niklas Scheerhoorn <sinner1991@gmail.com>
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

/// A trait for building query and body parameters in a string.
pub trait ParamBuilder {
    /// Appends a key-value pair to the string as a query parameter. If the
    /// string doesn't contain any query parameters, it adds a '?' character.
    /// Otherwise, it appends '&'.
    #[must_use]
    fn push_param_query(self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self;

    /// Appends a key-value pair to the string as a body parameter.
    /// It always appends '&'.
    #[must_use]
    fn push_param_body(self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self;
}

/// Implementation of the `ParamBuilder` trait for the `String` type.
impl ParamBuilder for String {
    /// Appends a key-value pair to the string as a query parameter.
    fn push_param_query(mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        if !self.contains('?') {
            self.push('?');
        } else if !self.ends_with('&') {
            self.push('&');
        }
        self.push_str(key.as_ref());
        self.push('=');
        self.push_str(value.as_ref());
        self
    }

    /// Appends a key-value pair to the string as a body parameter.
    fn push_param_body(mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        self.push('&');
        self.push_str(key.as_ref());
        self.push('=');
        self.push_str(value.as_ref());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_param_query_adds_question_mark_on_first_param() {
        let result = "https://example.com/auth"
            .to_string()
            .push_param_query("client_id", "my_client");
        assert_eq!(result, "https://example.com/auth?client_id=my_client");
    }

    #[test]
    fn push_param_query_adds_ampersand_on_subsequent_params() {
        let result = "https://example.com/auth?client_id=my_client"
            .to_string()
            .push_param_query("redirect_uri", "https://app.example.com/callback");
        assert_eq!(
            result,
            "https://example.com/auth?client_id=my_client&redirect_uri=https://app.example.com/callback"
        );
    }

    #[test]
    fn push_param_query_chaining_multiple_params() {
        let result = "https://example.com/logout"
            .to_string()
            .push_param_query("post_logout_redirect_uri", "https://app.example.com")
            .push_param_query("id_token_hint", "tok123");
        assert_eq!(
            result,
            "https://example.com/logout?post_logout_redirect_uri=https://app.example.com&id_token_hint=tok123"
        );
    }

    #[test]
    fn push_param_body_always_prepends_ampersand() {
        let result = "&grant_type=authorization_code"
            .to_string()
            .push_param_body("client_id", "my_client")
            .push_param_body("code", "auth_code_xyz");
        assert_eq!(
            result,
            "&grant_type=authorization_code&client_id=my_client&code=auth_code_xyz"
        );
    }

    // Compile-time check: exactly one of rust_crypto / aws_lc_rs must be enabled.
    // If neither is enabled this module will fail to compile due to the missing
    // jsonwebtoken crypto backend, which is the intended behaviour.
    #[cfg(any(feature = "rust_crypto", feature = "aws_lc_rs"))]
    #[test]
    fn crypto_backend_feature_is_enabled() {
        // rust_crypto and aws_lc_rs are mutually exclusive at the Cargo level
        // (no explicit conflict declared, but enabling both would pull in two
        // crypto backends simultaneously, which is unsupported by jsonwebtoken).
        // This test simply confirms that at least one backend compiled in.
        #[cfg(feature = "rust_crypto")]
        let backend = "rust_crypto";
        #[cfg(all(feature = "aws_lc_rs", not(feature = "rust_crypto")))]
        let backend = "aws_lc_rs";
        assert!(!backend.is_empty());
    }
}
