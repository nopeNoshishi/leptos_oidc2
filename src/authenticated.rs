use crate::storage::TokenStorage;
use crate::utils::ParamBuilder;
use crate::{Algorithm, AuthParameters, IssuerMetadata, TokenData, Validation};
use jsonwebtoken::{decode, DecodingKey};
use serde::de::DeserializeOwned;

#[derive(Clone, Debug)]
pub struct AuthenticatedData {
    pub(crate) parameters: AuthParameters,
    pub(crate) issuer: IssuerMetadata,
    pub(crate) token_store: TokenStorage,
}

impl AuthenticatedData {
    /// Generates and returns the URL for initiating the logout process. This
    /// URL is used to redirect the user to the authentication provider's logout
    /// page.
    #[must_use]
    pub fn logout_url(&self) -> String {
        self.issuer
            .configuration
            .end_session_endpoint
            .clone()
            .push_param_query(
                "post_logout_redirect_uri",
                self.parameters
                    .post_logout_redirect_uri
                    .clone()
                    .push_param_query("destroy_session", "true"),
            )
            .push_param_query("id_token_hint", &self.token_store.id_token)
    }

    /// Checks if the user is authenticated.
    #[must_use]
    pub fn is_authenticated(&self) -> bool {
        self.token_store.is_valid()
    }

    /// Returns the ID token, if available, from the authentication response.
    #[must_use]
    pub fn id_token(&self) -> String {
        self.token_store.id_token.clone()
    }

    /// Returns the access token, if available, from the authentication response.
    #[must_use]
    pub fn access_token(&self) -> String {
        self.token_store.access_token.clone()
    }

    /// Returns the decoded id token, if available, from the authentication response.
    #[must_use]
    pub fn decoded_id_token<T: DeserializeOwned>(
        &self,
        algorithm: Algorithm,
        audience: &[&str],
    ) -> Option<TokenData<T>> {
        self.decode_token(algorithm, audience, &self.token_store.id_token)
    }

    /// Returns the decoded access token, if available, from the authentication response.
    #[must_use]
    pub fn decoded_access_token<T: DeserializeOwned>(
        &self,
        algorithm: Algorithm,
        audience: &[&str],
    ) -> Option<TokenData<T>> {
        self.decode_token(algorithm, audience, &self.token_store.access_token)
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
}
