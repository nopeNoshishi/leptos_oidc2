use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
enum Audience {
    SingleAudience(String),
    MultipleAudiences(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Claims {
    /// Audience, either string or list of strings
    #[serde(rename = "aud")]
    audience: Audience,
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
    pub name: Option<String>,
    // Email address of the user
    pub email: Option<String>,
    // Username of the user
    pub preferred_username: String,
}

impl Claims {
    pub(crate) fn empty_vector() -> Vec<String> {
        Vec::new()
    }
    pub(crate) fn has_group(&self, group: &str) -> bool {
        self.groups.contains(&format!("/{group}"))
    }
    pub(crate) fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_claims() {
        let user_token = include_str!("../resources/leptos-token.json");
        let claims: Claims = serde_json::from_str(user_token).unwrap();
        match claims.audience {
            Audience::SingleAudience(audience) => {
                assert_eq!(audience, "account");
            }
            Audience::MultipleAudiences(_) => {}
        }
    }

    #[test]
    fn test_multiple_audiences() {
        let admin_token = include_str!("../resources/admin-token.json");
        let claims: Claims = serde_json::from_str(admin_token).unwrap();
        match claims.audience {
            Audience::SingleAudience(audience) => {}
            Audience::MultipleAudiences(audiences) => {
                assert_eq!(audiences.len(), 2);
            }
        }
    }
}
