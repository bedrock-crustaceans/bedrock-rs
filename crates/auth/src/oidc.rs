use crate::error::AuthError;
use crate::http::{AsyncHttpClient, HttpClient};
use crate::jwt::{Jwk, JwkSet};
use std::sync::{Arc, PoisonError, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug)]
struct KeyCache {
    jwks: JwkSet,
    last_refresh: Instant,
}

#[derive(Clone, Debug)]
pub struct AuthOIDC {
    pub issuer: String,
    pub audience: String,
    pub jwks_uri: String,
    pub refresh_interval: Duration,
    keys: Arc<RwLock<KeyCache>>,
}

impl AuthOIDC {
    pub const DISCOVERY_URL: &str = "https://client.discovery.minecraft-services.net/api/v1.0/discovery/MinecraftPE/builds/1.0.0.0";
    pub const AUDIENCE: &str = "api://auth-minecraft-services/multiplayer";
    pub const DEFAULT_REFRESH_INTERVAL: Duration = Duration::from_secs(5 * 60);

    pub fn new(
        issuer: impl Into<String>,
        audience: impl Into<String>,
        jwks_uri: impl Into<String>,
        jwks: JwkSet,
    ) -> Self {
        Self {
            issuer: issuer.into(),
            audience: audience.into(),
            jwks_uri: jwks_uri.into(),
            refresh_interval: Self::DEFAULT_REFRESH_INTERVAL,
            keys: Arc::new(RwLock::new(KeyCache {
                jwks,
                last_refresh: Instant::now(),
            })),
        }
    }

    pub fn jwk(&self, kid: &str) -> Option<Jwk> {
        self.keys
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .jwks
            .find(kid)
            .cloned()
    }

    pub fn replace_jwks(&self, jwks: JwkSet) {
        self.keys
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .jwks = jwks;
    }

    pub fn claim_refresh(&self, kid: &str, now: Instant) -> bool {
        let mut keys = self.keys.write().unwrap_or_else(PoisonError::into_inner);
        let due = now.saturating_duration_since(keys.last_refresh) >= self.refresh_interval;
        if !due || keys.jwks.find(kid).is_some() {
            return false;
        }
        keys.last_refresh = now;
        true
    }

    pub fn fetch(client: &impl HttpClient) -> Result<Self, AuthError> {
        let discovery = client.get(Self::DISCOVERY_URL)?;
        let config = client.get(&openid_configuration_url(&discovery)?)?;
        let (issuer, jwks_uri) = issuer_and_jwks_uri(&config)?;
        let jwks = serde_json::from_slice(&client.get(&jwks_uri)?)?;
        Ok(Self::new(issuer, Self::AUDIENCE, jwks_uri, jwks))
    }

    pub async fn fetch_async(client: &impl AsyncHttpClient) -> Result<Self, AuthError> {
        let discovery = client.get(Self::DISCOVERY_URL).await?;
        let config = client.get(&openid_configuration_url(&discovery)?).await?;
        let (issuer, jwks_uri) = issuer_and_jwks_uri(&config)?;
        let jwks = serde_json::from_slice(&client.get(&jwks_uri).await?)?;
        Ok(Self::new(issuer, Self::AUDIENCE, jwks_uri, jwks))
    }

    pub fn refresh(&self, client: &impl HttpClient) -> Result<(), AuthError> {
        self.replace_jwks(serde_json::from_slice(&client.get(&self.jwks_uri)?)?);
        Ok(())
    }

    pub async fn refresh_async(&self, client: &impl AsyncHttpClient) -> Result<(), AuthError> {
        self.replace_jwks(serde_json::from_slice(&client.get(&self.jwks_uri).await?)?);
        Ok(())
    }
}

fn openid_configuration_url(discovery: &[u8]) -> Result<String, AuthError> {
    let discovery: serde_json::Value = serde_json::from_slice(discovery)?;
    let base = discovery["result"]["serviceEnvironments"]["auth"]["prod"]["serviceUri"]
        .as_str()
        .ok_or(AuthError::Missing("serviceUri"))?;
    Ok(format!("{base}/.well-known/openid-configuration"))
}

fn issuer_and_jwks_uri(config: &[u8]) -> Result<(String, String), AuthError> {
    let config: serde_json::Value = serde_json::from_slice(config)?;
    let issuer = config["issuer"]
        .as_str()
        .ok_or(AuthError::Missing("issuer"))?;
    let jwks_uri = config["jwks_uri"]
        .as_str()
        .ok_or(AuthError::Missing("jwks_uri"))?;
    Ok((issuer.to_owned(), jwks_uri.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{FakeHttp, block_on};
    use serde_json::json;

    fn jwks_with(kid: &str) -> JwkSet {
        serde_json::from_value(json!({ "keys": [{
            "kty": "RSA", "kid": kid, "use": "sig", "alg": "RS256", "n": "AQAB", "e": "AQAB",
        }] }))
        .unwrap()
    }

    fn oidc_with(kid: &str) -> AuthOIDC {
        AuthOIDC::new(
            "issuer",
            AuthOIDC::AUDIENCE,
            "https://jwks.invalid",
            jwks_with(kid),
        )
    }

    const CONFIG_URL: &str = "https://auth.invalid/.well-known/openid-configuration";
    const JWKS_URL: &str = "https://auth.invalid/jwks";

    fn minecraft_services(jwks: &JwkSet) -> FakeHttp {
        FakeHttp::serving(&[
            (
                AuthOIDC::DISCOVERY_URL,
                json!({ "result": { "serviceEnvironments": { "auth": { "prod": {
                    "serviceUri": "https://auth.invalid",
                } } } } }),
            ),
            (
                CONFIG_URL,
                json!({ "issuer": "https://issuer.invalid", "jwks_uri": JWKS_URL }),
            ),
            (JWKS_URL, serde_json::to_value(jwks).unwrap()),
        ])
    }

    #[test]
    fn fetch_follows_discovery_to_the_jwks() {
        let http = minecraft_services(&jwks_with("current"));

        let oidc = AuthOIDC::fetch(&http).unwrap();

        assert_eq!(
            http.requests(),
            [AuthOIDC::DISCOVERY_URL, CONFIG_URL, JWKS_URL]
        );
        assert_eq!(oidc.issuer, "https://issuer.invalid");
        assert_eq!(oidc.jwks_uri, JWKS_URL);
        assert!(oidc.jwk("current").is_some());
    }

    #[test]
    fn fetch_async_follows_discovery_to_the_jwks() {
        let http = minecraft_services(&jwks_with("current"));

        let oidc = block_on(AuthOIDC::fetch_async(&http)).unwrap();

        assert_eq!(
            http.requests(),
            [AuthOIDC::DISCOVERY_URL, CONFIG_URL, JWKS_URL]
        );
        assert!(oidc.jwk("current").is_some());
    }

    #[test]
    fn refresh_replaces_the_jwks_from_its_uri() {
        let oidc = oidc_with("known");
        let http = FakeHttp::serving(&[(
            "https://jwks.invalid",
            serde_json::to_value(jwks_with("rotated")).unwrap(),
        )]);

        oidc.refresh(&http).unwrap();
        assert!(oidc.jwk("rotated").is_some());

        oidc.replace_jwks(jwks_with("known"));
        block_on(oidc.refresh_async(&http)).unwrap();
        assert!(oidc.jwk("rotated").is_some());
    }

    #[test]
    fn known_kid_never_claims_refresh() {
        let oidc = oidc_with("known");

        assert!(!oidc.claim_refresh("known", Instant::now() + Duration::from_secs(3600)));
    }

    #[test]
    fn unknown_kid_waits_for_refresh_interval() {
        let oidc = oidc_with("known");

        assert!(!oidc.claim_refresh("rotated", Instant::now()));
    }

    #[test]
    fn unknown_kid_claims_one_refresh_per_interval() {
        let oidc = oidc_with("known");
        let later = Instant::now() + oidc.refresh_interval;

        assert!(oidc.claim_refresh("rotated", later));
        assert!(!oidc.claim_refresh("rotated", later));
        assert!(!oidc.claim_refresh("another", later + Duration::from_secs(1)));
        assert!(oidc.claim_refresh("another", later + oidc.refresh_interval));
    }

    #[test]
    fn replaced_jwks_serves_new_kid() {
        let oidc = oidc_with("known");

        oidc.replace_jwks(jwks_with("rotated"));

        assert!(oidc.jwk("rotated").is_some());
        assert!(oidc.jwk("known").is_none());
    }
}
