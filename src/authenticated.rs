use crate::storage::TokenStorage;
use crate::utils::ParamBuilder;
use crate::{AuthParameters, IssuerMetadata};

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
        self
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
            )
            .push_param_query("id_token_hint", &self.token_store.id_token)
    }
}
