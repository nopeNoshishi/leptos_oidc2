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
