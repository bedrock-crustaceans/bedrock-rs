use crate::auth_error::AuthError;
use jsonwebtoken::jwk::JwkSet;

#[derive(Clone, Debug)]
pub struct AuthOIDC {
    pub issuer: String,
    pub jwks: JwkSet,
    pub audience: String,
}

impl AuthOIDC {
    const DISCOVERY: &str = "https://client.discovery.minecraft-services.net/api/v1.0/discovery/MinecraftPE/builds/1.0.0.0";
    const AUDIENCE: &str = "api://auth-minecraft-services/multiplayer";

    #[cfg(not(feature = "async"))]
    pub fn fetch() -> Result<Self, AuthError> {
        let discovery: serde_json::Value = reqwest::blocking::get(Self::DISCOVERY)?.json()?;

        let base = discovery["result"]["serviceEnvironments"]["auth"]["prod"]["serviceUri"]
            .as_str()
            .ok_or(AuthError::Missing("serviceUri"))?
            .to_string();

        let config_url = format!("{}/.well-known/openid-configuration", base);
        let config: serde_json::Value = reqwest::blocking::get(&config_url)?.json()?;

        let issuer = config["issuer"]
            .as_str()
            .ok_or(AuthError::Missing("issuer"))?
            .to_string();

        let jwks_uri = config["jwks_uri"]
            .as_str()
            .ok_or(AuthError::Missing("jwks_uri"))?;

        let jwks: JwkSet = reqwest::blocking::get(jwks_uri)?.json()?;

        Ok(Self {
            issuer,
            jwks,
            audience: Self::AUDIENCE.to_string(),
        })
    }

    #[cfg(feature = "async")]
    pub async fn fetch() -> Result<Self, AuthError> {
        let discovery: serde_json::Value = reqwest::get(Self::DISCOVERY).await?.json().await?;

        let base = discovery["result"]["serviceEnvironments"]["auth"]["prod"]["serviceUri"]
            .as_str()
            .ok_or(AuthError::Missing("serviceUri"))?
            .to_string();

        let config_url = format!("{}/.well-known/openid-configuration", base);
        let config: serde_json::Value = reqwest::get(&config_url).await?.json().await?;

        let issuer = config["issuer"]
            .as_str()
            .ok_or(AuthError::Missing("issuer"))?
            .to_string();

        let jwks_uri = config["jwks_uri"]
            .as_str()
            .ok_or(AuthError::Missing("jwks_uri"))?;

        let jwks: JwkSet = reqwest::get(jwks_uri).await?.json().await?;

        Ok(Self {
            issuer,
            jwks,
            audience: Self::AUDIENCE.to_string(),
        })
    }
}
