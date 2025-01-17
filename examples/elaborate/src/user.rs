use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Claims {
    /// Audience
    #[serde(rename = "aud")]
    audience: String,
    /// Issued at (as UTC timestamp)
    #[serde(rename = "iat")]
    issued_at: usize,
    /// Issuer
    #[serde(rename = "iss")]
    issuer: String,
    /// Expiration time (as UTC timestamp)
    #[serde(rename = "exp")]
    expiration_utc: usize,
    /// Subject (whom token refers to)
    #[serde(rename = "sub")]
    pub subject: String,
    // Roles the user belongs to (custom claim if present)
    #[serde(default = "Claims::empty_vector")]
    pub roles: Vec<String>,
    // Groups of the user (custom claim if present)
    #[serde(default = "Claims::empty_vector")]
    pub groups: Vec<String>,
    // Name of the user
    pub name: String,
    // Email address of the user
    pub email: String,
    // Username of the user
    pub preferred_username: String,
}

impl Claims {
    pub(crate) fn empty_vector() -> Vec<String> { Vec::new() }
}
