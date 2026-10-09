use std::error::Error;
use std::future::Future;
use std::time::Duration;

pub type HttpError = Box<dyn Error + Send + Sync>;

pub const HTTP_TIMEOUT: Duration = Duration::from_secs(15);

pub trait HttpClient {
    fn get(&self, url: &str) -> Result<Vec<u8>, HttpError>;
}

pub trait AsyncHttpClient {
    fn get(&self, url: &str) -> impl Future<Output = Result<Vec<u8>, HttpError>>;
}

#[cfg(feature = "ureq")]
pub fn ureq_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(HTTP_TIMEOUT))
        .build()
        .into()
}

#[cfg(feature = "ureq")]
impl HttpClient for ureq::Agent {
    fn get(&self, url: &str) -> Result<Vec<u8>, HttpError> {
        Ok(ureq::Agent::get(self, url)
            .call()?
            .body_mut()
            .read_to_vec()?)
    }
}

#[cfg(feature = "reqwest")]
pub fn reqwest_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder().timeout(HTTP_TIMEOUT).build()
}

#[cfg(feature = "reqwest")]
impl AsyncHttpClient for reqwest::Client {
    async fn get(&self, url: &str) -> Result<Vec<u8>, HttpError> {
        Ok(reqwest::Client::get(self, url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?
            .to_vec())
    }
}

#[cfg(all(test, feature = "reqwest"))]
mod tests {
    use super::*;
    use crate::auth_data::AuthData;
    use crate::auth_data::AuthPayload;
    use crate::auth_data::AuthType;
    use crate::chain::ChainRoot;
    use crate::connection_request::ConnectionRequest;

    fn assert_send<T: Send>(_: &T) {}

    #[test]
    fn verifying_with_reqwest_is_send() {
        let client = reqwest_client().unwrap();
        let root = ChainRoot::default();
        let request = ConnectionRequest {
            auth: AuthData {
                auth_type: AuthType::Online,
                auth_payload: AuthPayload::Token(String::new()),
            },
            client_data: String::new(),
        };

        assert_send(&request.verify_refreshing_async(None, &root, &client));
        assert_send(&crate::oidc::AuthOIDC::fetch_async(&client));
    }
}
