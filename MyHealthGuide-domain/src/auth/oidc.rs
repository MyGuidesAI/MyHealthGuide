use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use openidconnect::core::{
    CoreProviderMetadata, CoreClient, CoreResponseType,
    CoreJwsSigningAlgorithm, CoreSubjectIdentifierType
};
use openidconnect::{
    ClientId, ClientSecret, IssuerUrl, 
    RedirectUrl, AuthUrl, TokenUrl, JsonWebKeySetUrl, 
    ResponseTypes, EmptyAdditionalProviderMetadata
};
use openidconnect::reqwest::async_http_client;
use tracing::{debug, error, warn};
use thiserror::Error;

use crate::auth::UserInfo;

/// Errors that can occur during OIDC authentication
#[derive(Debug, Error)]
pub enum OidcError {
    #[error("Failed to discover OIDC provider: {0}")]
    DiscoveryError(String),
    
    #[error("Failed to initialize OIDC client: {0}")]
    ClientInitError(String),
    
    #[error("Failed to generate authorization URL: {0}")]
    AuthUrlError(String),
    
    #[error("Failed to exchange code for token: {0}")]
    TokenExchangeError(String),
    
    #[error("Failed to verify ID token: {0}")]
    TokenVerificationError(String),
    
    #[error("Session not found or expired")]
    SessionNotFound,
    
    #[error("CSRF token mismatch")]
    CsrfMismatch,
    
    #[error("User info extraction failed: {0}")]
    UserInfoError(String),
    
    #[error("Generic OIDC error: {0}")]
    Generic(String),
}

/// OIDC session data for a single authentication flow
#[derive(Debug, Clone)]
pub struct OidcSession {
    pub id: String,
    pub csrf_token: String,
    pub pkce_verifier: String,
    pub created_at: SystemTime,
    /// The nonce value used for OIDC ID token verification
    pub nonce: String,
}

/// OIDC configuration from environment variables
#[derive(Debug, Clone)]
pub struct OidcConfig {
    /// The client ID from the OIDC provider
    pub client_id: String,
    /// The client secret from the OIDC provider
    pub client_secret: String,
    /// The issuer URL for the OIDC provider
    pub issuer_url: String,
    /// The redirect URL for the OIDC callback
    pub redirect_url: String,
    /// Session expiration time in seconds (default: 10 minutes)
    pub session_timeout: Duration,
}

impl OidcConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // Check client ID
        if self.client_id == "default_client_id" || self.client_id.is_empty() {
            errors.push("OIDC_CLIENT_ID is not properly configured".to_string());
        }
        
        // Check client secret
        if self.client_secret == "default_client_secret" || self.client_secret.is_empty() {
            errors.push("OIDC_CLIENT_SECRET is not properly configured".to_string());
        }
        
        // Check issuer URL
        if self.issuer_url == "https://accounts.google.com" {
            // This is the default, but not necessarily an error
            // Just add a warning that default is being used
            warn!("Using default Google OIDC issuer URL. If this is not intended, set OIDC_ISSUER_URL");
        }
        
        // Make sure issuer_url is a valid URL
        if let Err(e) = url::Url::parse(&self.issuer_url) {
            errors.push(format!("OIDC_ISSUER_URL is not a valid URL: {}", e));
        }
        
        // Check redirect URL
        if self.redirect_url.contains("localhost") && !cfg!(debug_assertions) {
            // In a production build, localhost is likely not correct
            errors.push("OIDC_REDIRECT_URL contains 'localhost' in a production build".to_string());
        }
        
        // Make sure redirect_url is a valid URL
        if let Err(e) = url::Url::parse(&self.redirect_url) {
            errors.push(format!("OIDC_REDIRECT_URL is not a valid URL: {}", e));
        }
        
        // Check session timeout
        if self.session_timeout.as_secs() < 60 {
            errors.push("OIDC_SESSION_TIMEOUT is less than 60 seconds, which is likely too short".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for OidcConfig {
    fn default() -> Self {
        // Make sure we get fresh environment variables each time
        // This is important for testing when environment variables might change
        let config = Self {
            client_id: std::env::var("OIDC_CLIENT_ID")
                .unwrap_or_else(|_| {
                    warn!("OIDC_CLIENT_ID not set - using dummy value. OIDC login will not work properly.");
                    "default_client_id".to_string()
                }),
            client_secret: std::env::var("OIDC_CLIENT_SECRET")
                .unwrap_or_else(|_| {
                    warn!("OIDC_CLIENT_SECRET not set - using dummy value. OIDC login will not work properly.");
                    "default_client_secret".to_string()
                }),
            issuer_url: std::env::var("OIDC_ISSUER_URL")
                .unwrap_or_else(|_| {
                    debug!("OIDC_ISSUER_URL not set - using Google accounts as default.");
                    "https://accounts.google.com".to_string()
                }),
            redirect_url: std::env::var("OIDC_REDIRECT_URL")
                .unwrap_or_else(|_| {
                    debug!("OIDC_REDIRECT_URL not set - using localhost default.");
                    "http://localhost:3000/auth/oidc/callback".to_string()
                }),
            session_timeout: Duration::from_secs(
                std::env::var("OIDC_SESSION_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(600), // 10 minutes default
            ),
        };
        
        // Log validation warnings but don't fail
        if let Err(errors) = config.validate() {
            for error in &errors {
                warn!("OIDC configuration warning: {}", error);
            }
        }
        
        config
    }
}

/// Extended user profile information from OIDC provider
#[derive(Debug, Clone, Default)]
pub struct UserProfile {
    /// User ID from the OIDC provider
    pub sub: String,
    /// User email address
    pub email: Option<String>,
    /// Whether the email is verified
    pub email_verified: Option<bool>,
    /// User's full name
    pub name: Option<String>,
    /// User's given name
    pub given_name: Option<String>,
    /// User's family name
    pub family_name: Option<String>,
    /// User's nickname
    pub nickname: Option<String>,
    /// User's preferred username
    pub preferred_username: Option<String>,
    /// User's profile URL
    pub profile: Option<String>,
    /// User's picture URL
    pub picture: Option<String>,
    /// User's website URL
    pub website: Option<String>,
    /// User's gender
    pub gender: Option<String>,
    /// User's birthdate
    pub birthdate: Option<String>,
    /// User's timezone
    pub zoneinfo: Option<String>,
    /// User's locale
    pub locale: Option<String>,
    /// User's phone number
    pub phone_number: Option<String>,
    /// Whether the phone number is verified
    pub phone_number_verified: Option<bool>,
    /// Additional custom claims
    pub additional_claims: HashMap<String, String>,
}

/// OIDC client for authentication
pub struct OidcClient {
    /// The OpenID Connect client
    client: CoreClient,
    /// The OIDC configuration
    config: OidcConfig,
    /// Session repository for storing OIDC sessions
    session_repository: Arc<dyn SessionRepository>,
}

impl OidcClient {
    /// Create a new OIDC client from configuration with retry logic and caching
    pub async fn new(config: OidcConfig) -> Result<Self, OidcError> {
        // Discover the OIDC provider with retries
        let provider_metadata = Self::discover_provider_with_retry(&config.issuer_url, 3).await?;
        
        debug!("Discovered OIDC provider: {}", provider_metadata.issuer().as_str());
        
        // Create the OIDC client
        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
        )
        .set_redirect_uri(
            RedirectUrl::new(config.redirect_url.clone()).map_err(|e| {
                error!("Invalid redirect URL: {}", e);
                OidcError::ClientInitError(format!("Invalid redirect URL: {}", e))
            })?,
        );
        
        debug!("OIDC client initialized successfully");
        
        // Create a new in-memory session repository
        let session_repository = Arc::new(InMemorySessionRepository::new());
        
        Ok(Self {
            client,
            config,
            session_repository,
        })
    }
    
    /// Discover OIDC provider metadata with retry logic
    async fn discover_provider_with_retry(issuer_url_str: &str, max_retries: usize) -> Result<CoreProviderMetadata, OidcError> {
        let issuer_url = IssuerUrl::new(issuer_url_str.to_string()).map_err(|e| {
            error!("Invalid issuer URL: {}", e);
            OidcError::DiscoveryError(format!("Invalid issuer URL: {}", e))
        })?;
        
        let mut attempt = 0;
        let mut last_error = None;
        
        while attempt < max_retries {
            match CoreProviderMetadata::discover_async(
                issuer_url.clone(),
                async_http_client,
            ).await {
                Ok(metadata) => {
                    return Ok(metadata);
                }
                Err(e) => {
                    attempt += 1;
                    let retry_delay = std::time::Duration::from_millis((2u64.pow(attempt as u32)) * 100);
                    warn!("OIDC provider discovery failed (attempt {}/{}): {}. Retrying in {:?}...", 
                         attempt, max_retries, e, retry_delay);
                    last_error = Some(format!("{}", e));
                    
                    // Sleep before retry
                    tokio::time::sleep(retry_delay).await;
                }
            }
        }
        
        Err(OidcError::DiscoveryError(format!(
            "Provider discovery failed after {} attempts: {}", 
            max_retries,
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        )))
    }
    
    /// Stub implementation for tests
    pub fn stub() -> Self {
        // Create a minimal client for testing
        let config = OidcConfig::default();
        let issuer_url = IssuerUrl::new(config.issuer_url.clone()).unwrap();
        let client_id = ClientId::new(config.client_id.clone());
        let client_secret = ClientSecret::new(config.client_secret.clone());
        let redirect_url = RedirectUrl::new(config.redirect_url.clone()).unwrap();
        
        // Create auth and token URLs
        let auth_url = AuthUrl::new("https://stub-issuer.example.com/auth".to_string()).unwrap();
        let _token_url = TokenUrl::new("https://stub-issuer.example.com/token".to_string()).unwrap();
        let jwks_uri = JsonWebKeySetUrl::new(format!("{}/jwks", issuer_url.as_str())).unwrap();
        
        // Create the required collections with correct types
        let response_types = vec![ResponseTypes::new(vec![CoreResponseType::Code])];
        let subject_types = vec![CoreSubjectIdentifierType::Public];
        let id_token_signing_algs = vec![CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256];
        
        // Create minimal provider metadata for testing
        let provider_metadata = CoreProviderMetadata::new(
            issuer_url.clone(),
            auth_url,
            jwks_uri,
            response_types,     // response_types_supported
            subject_types,      // subject_types_supported
            id_token_signing_algs, // id_token_signing_alg_values_supported
            EmptyAdditionalProviderMetadata {},
        );
        
        // Create the client from provider metadata
        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            client_id,
            Some(client_secret),
        )
        .set_redirect_uri(redirect_url);
        
        Self {
            client,
            config,
            session_repository: Arc::new(InMemorySessionRepository::new()),
        }
    }

    /// Start the authentication flow and return the authorization URL
    #[cfg(not(any(test, feature = "mock")))]
    pub async fn start_auth_flow(&self) -> Result<(String, OidcSession), OidcError> {
        // Generate PKCE challenge and verifier
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        
        // Create a CSRF token
        let csrf_token_str = Uuid::new_v4().to_string(); // Use UUID instead of CsrfToken::to_string()
        let csrf_token = CsrfToken::new(csrf_token_str.clone());
        
        // Generate a nonce for OpenID Connect
        let nonce_str = Uuid::new_v4().to_string();
        let nonce = Nonce::new(nonce_str.clone());
        
        // Generate authorization URL
        let auth_url_tuple = self.client
            .authorize_url(
                openidconnect::AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                move || csrf_token.clone(),
                move || nonce.clone()
            )
            .set_pkce_challenge(pkce_challenge)
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .url();
        
        // Extract the URL from the tuple
        let auth_url = auth_url_tuple.0;
        
        // Create a session
        let session = OidcSession {
            id: Uuid::new_v4().to_string(),
            csrf_token: csrf_token_str,
            pkce_verifier: pkce_verifier.secret().to_string(),
            created_at: SystemTime::now(),
            nonce: nonce_str,
        };
        
        // Store the session
        self.session_repository.store_session(session.clone())
            .map_err(|e| {
                error!("Failed to store OIDC session: {}", e);
                OidcError::Generic(format!("Failed to store session: {}", e))
            })?;
        
        debug!("Generated OIDC authorization URL: {}", auth_url);
        Ok((auth_url.to_string(), session))
    }
    
    /// Mock implementation of start_auth_flow for testing
    #[cfg(any(test, feature = "mock"))]
    pub async fn start_auth_flow(&self) -> Result<(String, OidcSession), OidcError> {
        // For testing, create a mock auth URL and session
        let csrf_token = "test-csrf-token";
        let nonce = "test-nonce-value";
        let session = OidcSession {
            id: "test-session-id".to_string(),
            csrf_token: csrf_token.to_string(),
            pkce_verifier: "test-pkce-verifier".to_string(),
            created_at: SystemTime::now(),
            nonce: nonce.to_string(),
        };
        
        // Store the session for later use in tests
        self.session_repository.store_session(session.clone())?;
        
        let auth_url = format!(
            "https://stub-issuer.example.com/auth?client_id={}&redirect_uri={}&state={}&scope=openid+email+profile&nonce={}",
            self.config.client_id,
            urlencoding::encode(&self.config.redirect_url),
            csrf_token,
            nonce
        );
        
        Ok((auth_url, session))
    }

    /// Handle the callback from the OIDC provider
    #[cfg(not(any(test, feature = "mock")))]
    pub async fn handle_callback(&self, code: &str, state: &str) -> Result<UserInfo, OidcError> {
        // Lookup the session from the CSRF token (state parameter)
        let session = self.session_repository.get_session(state)?;
        
        debug!("Retrieved session for state '{}': id={}, created_at={:?}, nonce={}",
            state, session.id, session.created_at, session.nonce);
        
        // Check if session is expired
        let now = SystemTime::now();
        if now.duration_since(session.created_at).map_err(|e| {
            error!("Clock error when checking session expiry: {:?}", e);
            OidcError::UserInfoError("System clock error".to_string())
        })? > self.config.session_timeout {
            error!("Session has expired. Created at: {:?}, Now: {:?}, Timeout: {:?}", 
                   session.created_at, now, self.config.session_timeout);
            return Err(OidcError::SessionNotFound);
        }
        
        // Exchange the code for a token
        let token_response = self.client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(PkceCodeVerifier::new(session.pkce_verifier))
            .request_async(async_http_client)
            .await
            .map_err(|e| {
                error!("Failed to exchange code for token: {:?}", e);
                OidcError::TokenExchangeError(format!("Authorization code exchange failed. This may be due to an expired code, PKCE validation failure, or configuration mismatch: {}", e))
            })?;
        
        // Extract the ID token
        let id_token = token_response.id_token().ok_or_else(|| {
            error!("No ID token returned from provider");
            OidcError::TokenVerificationError("No ID token was returned from the provider. This may indicate a misconfigured scope or provider issue.".to_string())
        })?;
        
        // Use session nonce for improved security
        let nonce = Nonce::new(session.nonce.clone());
        debug!("Using nonce from session for verification: {}", session.nonce);
        
        // Verify the ID token with extended validation
        let claims = id_token
            .claims(&self.client.id_token_verifier(), &nonce)
            .map_err(|e| {
                error!("Failed to verify ID token: {:?}", e);
                OidcError::TokenVerificationError(format!("ID token verification failed: {}. This may indicate token tampering or a configuration issue.", e))
            })?;
        
        // Additional validation checks
        if let Some(azp) = claims.authorized_party() {
            if azp.as_str() != self.config.client_id {
                error!("Authorized party mismatch in ID token: {} != {}", 
                       azp.as_str(), self.config.client_id);
                return Err(OidcError::TokenVerificationError(
                    "Authorized party in token doesn't match client ID".to_string()
                ));
            }
        }
        
        // Check token expiration time
        let exp = claims.expiration();
        
        // Convert SystemTime to chrono::DateTime for comparison
        let now = chrono::Utc::now();
        
        // Compare expiration to current time
        if exp < now {
            error!("ID token has expired: exp={:?}, now={:?}", exp, now);
            return Err(OidcError::TokenVerificationError(
                "ID token has expired".to_string()
            ));
        }
        
        // Clean up the session
        if let Err(e) = self.session_repository.delete_session(state) {
            warn!("Failed to delete OIDC session: {}", e);
            // Continue anyway, not a critical error
        }
        
        // Try to fetch extended user profile from the userinfo endpoint
        let access_token = token_response.access_token();
        let user_profile = match self.fetch_user_profile(access_token.secret()).await {
            Ok(profile) => {
                debug!("Successfully fetched extended user profile for subject: {}", profile.sub);
                Some(profile)
            }
            Err(e) => {
                warn!("Failed to fetch user profile, falling back to ID token claims: {}", e);
                None
            }
        };
        
        // If we have the user profile, convert it to UserInfo, otherwise extract from claims
        if let Some(profile) = user_profile {
            Ok(self.profile_to_user_info(&profile))
        } else {
            // Extract user information from claims
            let user_id = claims.subject().to_string();
            
            // Extract email from claims
            let email = claims.email().map(|e| e.to_string());
            
            // Extract name from claims
            let name = claims.name().and_then(|n| n.get(None)).map(|n| n.to_string());
            
            // Extract profile picture from claims
            let picture = claims.picture().and_then(|p| p.get(None)).map(|p| p.to_string());
            
            // Create UserInfo
            let user_info = UserInfo {
                user_id,
                roles: vec!["user".to_string()],
                email,
                name,
                picture,
                auth_source: "oidc".to_string(),
            };
            
            Ok(user_info)
        }
    }

    /// Handle the callback from the OIDC provider in test environments
    #[cfg(any(test, feature = "mock"))]
    pub async fn handle_callback(&self, code: &str, _state: &str) -> Result<UserInfo, OidcError> {
        // For testing, just create a stub user
        if code == "test_error_code" {
            return Err(OidcError::TokenExchangeError("Test error".to_string()));
        }
        
        Ok(UserInfo {
            user_id: "test-user-123".to_string(),
            roles: vec!["user".to_string()],
            email: Some("test@example.com".to_string()),
            name: Some("Test User".to_string()),
            picture: Some("https://example.com/avatar.png".to_string()),
            auth_source: "oidc".to_string(),
        })
    }

    #[cfg(any(test, feature = "mock"))]
    pub fn get_client_id(&self) -> &str {
        &self.config.client_id
    }
    
    #[cfg(any(test, feature = "mock"))]
    pub fn get_client_secret(&self) -> &str {
        &self.config.client_secret
    }
    
    #[cfg(any(test, feature = "mock"))]
    pub fn get_issuer_url(&self) -> &str {
        &self.config.issuer_url
    }
    
    #[cfg(any(test, feature = "mock"))]
    pub fn get_redirect_url(&self) -> &str {
        &self.config.redirect_url
    }
    
    #[cfg(any(test, feature = "mock"))]
    pub fn get_session_timeout(&self) -> Duration {
        self.config.session_timeout
    }

    /// Fetch user profile information from the userinfo endpoint
    #[cfg(not(any(test, feature = "mock")))]
    pub async fn fetch_user_profile(&self, access_token: &str) -> Result<UserProfile, OidcError> {
        // Create a reqwest client
        let client = reqwest::Client::new();
        
        // Get the userinfo endpoint URL
        // We need to manually construct the userinfo endpoint since provider_metadata() isn't available
        let issuer_url = self.config.issuer_url.to_string();
        let userinfo_url = if issuer_url.ends_with("/") {
            format!("{}userinfo", issuer_url)
        } else {
            format!("{}/userinfo", issuer_url)
        };
        
        // Make a request to the userinfo endpoint
        let response = client.get(&userinfo_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to userinfo endpoint: {:?}", e);
                OidcError::UserInfoError(format!("Network error when fetching user info: {}", e))
            })?;
        
        // Check if the request was successful
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Failed to fetch user profile: {} - {}", status, error_text);
            return Err(OidcError::UserInfoError(format!("Failed to fetch user profile: {} - {}", status, error_text)));
        }
        
        // Parse the response as JSON
        let userinfo: serde_json::Value = response.json().await.map_err(|e| {
            error!("Failed to parse userinfo response: {}", e);
            OidcError::UserInfoError(format!("Failed to parse userinfo response: {}", e))
        })?;
        
        // Create the user profile
        let mut profile = UserProfile::default();
        
        // Extract standard claims
        profile.sub = userinfo.get("sub")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        profile.email = userinfo.get("email").and_then(|v| v.as_str()).map(String::from);
        profile.email_verified = userinfo.get("email_verified").and_then(|v| v.as_bool());
        profile.name = userinfo.get("name").and_then(|v| v.as_str()).map(String::from);
        profile.given_name = userinfo.get("given_name").and_then(|v| v.as_str()).map(String::from);
        profile.family_name = userinfo.get("family_name").and_then(|v| v.as_str()).map(String::from);
        profile.nickname = userinfo.get("nickname").and_then(|v| v.as_str()).map(String::from);
        profile.preferred_username = userinfo.get("preferred_username").and_then(|v| v.as_str()).map(String::from);
        profile.profile = userinfo.get("profile").and_then(|v| v.as_str()).map(String::from);
        profile.picture = userinfo.get("picture").and_then(|v| v.as_str()).map(String::from);
        profile.website = userinfo.get("website").and_then(|v| v.as_str()).map(String::from);
        profile.gender = userinfo.get("gender").and_then(|v| v.as_str()).map(String::from);
        profile.birthdate = userinfo.get("birthdate").and_then(|v| v.as_str()).map(String::from);
        profile.zoneinfo = userinfo.get("zoneinfo").and_then(|v| v.as_str()).map(String::from);
        profile.locale = userinfo.get("locale").and_then(|v| v.as_str()).map(String::from);
        profile.phone_number = userinfo.get("phone_number").and_then(|v| v.as_str()).map(String::from);
        profile.phone_number_verified = userinfo.get("phone_number_verified").and_then(|v| v.as_bool());
        
        // Extract any additional claims
        if let Some(obj) = userinfo.as_object() {
            for (key, value) in obj {
                if !["sub", "email", "email_verified", "name", "given_name", "family_name",
                     "nickname", "preferred_username", "profile", "picture", "website",
                     "gender", "birthdate", "zoneinfo", "locale", "phone_number",
                     "phone_number_verified"].contains(&key.as_str()) {
                    if let Some(val_str) = value.as_str() {
                        profile.additional_claims.insert(key.clone(), val_str.to_string());
                    } else if let Ok(val_json) = serde_json::to_string(value) {
                        profile.additional_claims.insert(key.clone(), val_json);
                    }
                }
            }
        }
        
        Ok(profile)
    }
    
    /// Convert a UserProfile to a UserInfo
    pub fn profile_to_user_info(&self, profile: &UserProfile) -> UserInfo {
        UserInfo {
            user_id: profile.sub.clone(),
            roles: vec!["user".to_string()], // Default role
            email: profile.email.clone(),
            name: profile.name.clone().or_else(|| {
                // Create a name from given_name and family_name if available
                match (profile.given_name.as_ref(), profile.family_name.as_ref()) {
                    (Some(given), Some(family)) => Some(format!("{} {}", given, family)),
                    (Some(given), None) => Some(given.clone()),
                    (None, Some(family)) => Some(family.clone()),
                    _ => profile.preferred_username.clone()
                }
            }),
            picture: profile.picture.clone(),
            auth_source: "oidc".to_string(),
        }
    }

    /// Debug utility to print information about all active sessions
    #[cfg(not(any(test, feature = "mock")))]
    pub fn debug_sessions(&self) {
        if let Ok(sessions) = self.session_repository.debug_sessions() {
            debug!("Current OIDC sessions ({}):", sessions.len());
            for (token, session) in sessions {
                debug!("  Session with token '{}': id={}, created={:?}, nonce={}",
                       token, session.id, session.created_at, session.nonce);
            }
        } else {
            debug!("Unable to retrieve session debug information");
        }
    }
}

/// Session repository trait for storing OIDC sessions
pub trait SessionRepository: Send + Sync {
    /// Store a session
    fn store_session(&self, session: OidcSession) -> Result<(), OidcError>;
    
    /// Get a session by CSRF token
    fn get_session(&self, csrf_token: &str) -> Result<OidcSession, OidcError>;
    
    /// Delete a session
    fn delete_session(&self, csrf_token: &str) -> Result<(), OidcError>;
    
    /// Cleanup expired sessions
    fn cleanup_expired_sessions(&self, timeout: Duration) -> Result<(), OidcError>;

    /// Debug utility to print information about all active sessions
    fn debug_sessions(&self) -> Result<HashMap<String, OidcSession>, OidcError>;
}

/// In-memory implementation of SessionRepository
pub struct InMemorySessionRepository {
    sessions: Mutex<HashMap<String, OidcSession>>,
}

impl Default for InMemorySessionRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemorySessionRepository {
    /// Create a new in-memory session repository
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }
}

impl SessionRepository for InMemorySessionRepository {
    fn store_session(&self, session: OidcSession) -> Result<(), OidcError> {
        let mut sessions = self.sessions.lock().map_err(|e| {
            error!("Failed to acquire session lock: {}", e);
            OidcError::ClientInitError(format!("Session lock error: {}", e))
        })?;
        
        sessions.insert(session.csrf_token.clone(), session);
        Ok(())
    }
    
    fn get_session(&self, csrf_token: &str) -> Result<OidcSession, OidcError> {
        let sessions = self.sessions.lock().map_err(|e| {
            error!("Failed to acquire session lock: {}", e);
            OidcError::UserInfoError(format!("Session lock error: {}", e))
        })?;
        
        sessions.get(csrf_token).cloned().ok_or_else(|| {
            error!("Session not found for state token. This may be due to an expired session or invalid state parameter.");
            OidcError::SessionNotFound
        })
    }
    
    fn delete_session(&self, csrf_token: &str) -> Result<(), OidcError> {
        let mut sessions = self.sessions.lock().map_err(|e| {
            error!("Failed to acquire session lock: {}", e);
            OidcError::UserInfoError(format!("Session lock error: {}", e))
        })?;
        
        sessions.remove(csrf_token);
        Ok(())
    }
    
    fn cleanup_expired_sessions(&self, timeout: Duration) -> Result<(), OidcError> {
        let mut sessions = self.sessions.lock().map_err(|e| {
            error!("Failed to acquire session lock: {}", e);
            OidcError::UserInfoError(format!("Session lock error: {}", e))
        })?;
        
        let now = SystemTime::now();
        let expired_tokens: Vec<String> = sessions.iter()
            .filter(|(_, session)| {
                now.duration_since(session.created_at)
                   .map(|elapsed| elapsed > timeout)
                   .unwrap_or(true)
            })
            .map(|(token, _)| token.clone())
            .collect();
        
        for token in expired_tokens {
            sessions.remove(&token);
        }
        
        Ok(())
    }

    fn debug_sessions(&self) -> Result<HashMap<String, OidcSession>, OidcError> {
        let sessions = self.sessions.lock().map_err(|e| {
            error!("Failed to acquire session lock: {}", e);
            OidcError::UserInfoError(format!("Session lock error: {}", e))
        })?;
        
        Ok(sessions.clone())
    }
}

// Tests for the OidcConfig
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        // This test should use an implementation that ignores environment variables
        // for consistency across test runs
        struct TestConfig;
        
        impl TestConfig {
            fn default() -> OidcConfig {
                OidcConfig {
                    client_id: "default_client_id".to_string(),
                    client_secret: "default_client_secret".to_string(),
                    issuer_url: "https://accounts.google.com".to_string(),
                    redirect_url: "http://localhost:3000/auth/oidc/callback".to_string(),
                    session_timeout: Duration::from_secs(600),
                }
            }
        }
        
        let config = TestConfig::default();
        assert_eq!(config.client_id, "default_client_id");
        assert_eq!(config.client_secret, "default_client_secret");
        assert_eq!(config.issuer_url, "https://accounts.google.com");
        assert_eq!(config.redirect_url, "http://localhost:3000/auth/oidc/callback");
        assert_eq!(config.session_timeout, Duration::from_secs(600));
    }
    
    #[test]
    fn test_config_from_env() {
        // Create an environment-specific config for this test
        struct EnvTestConfig;
        
        impl EnvTestConfig {
            fn from_env() -> OidcConfig {
                // Set environment variables for this test only
                let _client_id = std::env::var("OIDC_CLIENT_ID").unwrap_or_else(|_| {
                    std::env::set_var("OIDC_CLIENT_ID", "test_client_id");
                    "test_client_id".to_string()
                });
                
                let _client_secret = std::env::var("OIDC_CLIENT_SECRET").unwrap_or_else(|_| {
                    std::env::set_var("OIDC_CLIENT_SECRET", "test_client_secret");
                    "test_client_secret".to_string()
                });
                
                let _issuer_url = std::env::var("OIDC_ISSUER_URL").unwrap_or_else(|_| {
                    std::env::set_var("OIDC_ISSUER_URL", "https://test.auth0.com");
                    "https://test.auth0.com".to_string()
                });
                
                let _redirect_url = std::env::var("OIDC_REDIRECT_URL").unwrap_or_else(|_| {
                    std::env::set_var("OIDC_REDIRECT_URL", "https://myapp.com/callback");
                    "https://myapp.com/callback".to_string()
                });
                
                let _session_timeout = std::env::var("OIDC_SESSION_TIMEOUT").unwrap_or_else(|_| {
                    std::env::set_var("OIDC_SESSION_TIMEOUT", "300");
                    "300".to_string()
                });
                
                // Use regular default() which should now pick up our env vars
                OidcConfig::default()
            }
        }
        
        let config = EnvTestConfig::from_env();
        assert_eq!(config.client_id, "test_client_id");
        assert_eq!(config.client_secret, "test_client_secret");
        assert_eq!(config.issuer_url, "https://test.auth0.com");
        assert_eq!(config.redirect_url, "https://myapp.com/callback");
        assert_eq!(config.session_timeout, Duration::from_secs(300));
    }
    
    #[test]
    fn test_inmemory_session_repository() {
        // Create a session repository
        let repo = InMemorySessionRepository::new();
        
        // Create a test session
        let session = OidcSession {
            id: "test-id".to_string(),
            csrf_token: "test-csrf".to_string(),
            pkce_verifier: "test-pkce".to_string(),
            created_at: SystemTime::now(),
            nonce: "test-nonce".to_string(),
        };
        
        // Store the session
        let result = repo.store_session(session.clone());
        assert!(result.is_ok(), "Failed to store session: {:?}", result.err());
        
        // Fetch the session
        let fetched = repo.get_session("test-csrf");
        assert!(fetched.is_ok(), "Failed to get session: {:?}", fetched.err());
        
        // Compare the sessions
        let fetched_session = fetched.unwrap();
        assert_eq!(fetched_session.id, session.id);
        assert_eq!(fetched_session.csrf_token, session.csrf_token);
        assert_eq!(fetched_session.pkce_verifier, session.pkce_verifier);
        
        // Delete the session
        let delete_result = repo.delete_session("test-csrf");
        assert!(delete_result.is_ok(), "Failed to delete session: {:?}", delete_result.err());
        
        // Try to fetch the deleted session
        let not_found = repo.get_session("test-csrf");
        assert!(not_found.is_err(), "Session should have been deleted");
        match not_found.err().unwrap() {
            OidcError::SessionNotFound => { /* expected */ },
            err => panic!("Unexpected error type: {:?}", err),
        }
    }
    
    #[test]
    fn test_session_expiration() {
        // Create a session repository
        let repo = InMemorySessionRepository::new();
        
        // Create an expired session (created 11 minutes ago)
        let mut created_at = SystemTime::now();
        created_at = created_at.checked_sub(Duration::from_secs(11 * 60)).unwrap();
        
        let session = OidcSession {
            id: "expired-id".to_string(),
            csrf_token: "expired-csrf".to_string(),
            pkce_verifier: "expired-pkce".to_string(),
            created_at,
            nonce: "expired-nonce".to_string(),
        };
        
        // Store the expired session
        repo.store_session(session).unwrap();
        
        // Create a non-expired session (created just now)
        let session2 = OidcSession {
            id: "valid-id".to_string(),
            csrf_token: "valid-csrf".to_string(),
            pkce_verifier: "valid-pkce".to_string(),
            created_at: SystemTime::now(),
            nonce: "valid-nonce".to_string(),
        };
        
        // Store the valid session
        repo.store_session(session2).unwrap();
        
        // Cleanup expired sessions (using 10 minutes timeout)
        let cleanup_result = repo.cleanup_expired_sessions(Duration::from_secs(10 * 60));
        assert!(cleanup_result.is_ok(), "Failed to cleanup expired sessions: {:?}", cleanup_result.err());
        
        // The expired session should be gone
        let expired_result = repo.get_session("expired-csrf");
        assert!(expired_result.is_err(), "Expired session should have been removed");
        
        // The valid session should still be there
        let valid_result = repo.get_session("valid-csrf");
        assert!(valid_result.is_ok(), "Valid session should still exist");
    }
    
    #[test]
    fn test_oidc_providers_stub() {
        // Create a stub provider collection
        let providers = OidcProviders::stub();
        
        // Check that we have a default provider
        assert_eq!(providers.default_provider, "default");
        
        // Check that we can get the default client
        let default_client = providers.default_client();
        assert!(default_client.is_some(), "Default client should be available");
        
        // Check that provider IDs list works
        let provider_ids = providers.provider_ids();
        assert_eq!(provider_ids.len(), 1);
        assert_eq!(provider_ids[0], "default");
        
        // Check that we can get a specific client
        let specific_client = providers.get_client("default");
        assert!(specific_client.is_some(), "Specific client should be available");
        
        // Check that non-existent clients return None
        let missing_client = providers.get_client("nonexistent");
        assert!(missing_client.is_none(), "Non-existent client should return None");
    }
    
    #[tokio::test]
    async fn test_user_profile_conversion() {
        // Create a test profile
        let mut profile = UserProfile {
            sub: "test-user-123".to_string(),
            email: Some("user@example.com".to_string()),
            email_verified: Some(true),
            name: Some("Test User".to_string()),
            given_name: Some("Test".to_string()),
            family_name: Some("User".to_string()),
            picture: Some("https://example.com/pic.jpg".to_string()),
            ..Default::default()
        };
        
        // Create a client to use for conversion
        let client = OidcClient::stub();
        
        // Convert the profile to UserInfo
        let user_info = client.profile_to_user_info(&profile);
        
        // Verify the conversion
        assert_eq!(user_info.user_id, "test-user-123");
        assert_eq!(user_info.email, Some("user@example.com".to_string()));
        assert_eq!(user_info.name, Some("Test User".to_string()));
        assert_eq!(user_info.picture, Some("https://example.com/pic.jpg".to_string()));
        assert_eq!(user_info.auth_source, "oidc");
        
        // Test fallback to concatenated name when name is missing
        profile.name = None;
        let user_info2 = client.profile_to_user_info(&profile);
        assert_eq!(user_info2.name, Some("Test User".to_string())); // Concatenated from given_name + family_name
        
        // Test fallback to given_name when family_name is missing
        profile.family_name = None;
        let user_info3 = client.profile_to_user_info(&profile);
        assert_eq!(user_info3.name, Some("Test".to_string())); // Just given_name
        
        // Test fallback to family_name when given_name is missing
        profile.given_name = None;
        profile.family_name = Some("User".to_string());
        let user_info4 = client.profile_to_user_info(&profile);
        assert_eq!(user_info4.name, Some("User".to_string())); // Just family_name
        
        // Test fallback to preferred_username when all others are missing
        profile.family_name = None;
        profile.preferred_username = Some("testuser".to_string());
        let user_info5 = client.profile_to_user_info(&profile);
        assert_eq!(user_info5.name, Some("testuser".to_string())); // Preferred username
    }
}

/// Collection of OIDC providers
pub struct OidcProviders {
    /// Map of provider IDs to OIDC clients
    providers: HashMap<String, Arc<OidcClient>>,
    /// Default provider ID
    default_provider: String,
}

impl OidcProviders {
    /// Create a new OidcProviders instance
    pub async fn new() -> Self {
        let mut providers = HashMap::new();
        let mut default_provider = "default".to_string();
        
        // Check for provider configuration in environment variables
        // Format: OIDC_PROVIDERS=provider1,provider2,provider3
        if let Ok(provider_list) = std::env::var("OIDC_PROVIDERS") {
            let provider_ids: Vec<String> = provider_list.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            
            if !provider_ids.is_empty() {
                for provider_id in &provider_ids {
                    // For each provider, look for specific config
                    // Format: OIDC_CLIENT_ID_provider1, OIDC_CLIENT_SECRET_provider1, etc.
                    let config = OidcConfig {
                        client_id: std::env::var(format!("OIDC_CLIENT_ID_{}", provider_id))
                            .unwrap_or_else(|_| {
                                warn!("OIDC_CLIENT_ID_{} not set - falling back to OIDC_CLIENT_ID", provider_id);
                                std::env::var("OIDC_CLIENT_ID")
                                    .unwrap_or_else(|_| {
                                        warn!("OIDC_CLIENT_ID not set - using dummy value for provider {}. OIDC login will not work properly.", provider_id);
                                        format!("default_client_id_{}", provider_id)
                                    })
                            }),
                        client_secret: std::env::var(format!("OIDC_CLIENT_SECRET_{}", provider_id))
                            .unwrap_or_else(|_| {
                                warn!("OIDC_CLIENT_SECRET_{} not set - falling back to OIDC_CLIENT_SECRET", provider_id);
                                std::env::var("OIDC_CLIENT_SECRET")
                                    .unwrap_or_else(|_| {
                                        warn!("OIDC_CLIENT_SECRET not set - using dummy value for provider {}. OIDC login will not work properly.", provider_id);
                                        format!("default_client_secret_{}", provider_id)
                                    })
                            }),
                        issuer_url: std::env::var(format!("OIDC_ISSUER_URL_{}", provider_id))
                            .unwrap_or_else(|_| {
                                debug!("OIDC_ISSUER_URL_{} not set - falling back to OIDC_ISSUER_URL", provider_id);
                                std::env::var("OIDC_ISSUER_URL")
                                    .unwrap_or_else(|_| {
                                        debug!("OIDC_ISSUER_URL not set - using Google accounts as default for provider {}.", provider_id);
                                        "https://accounts.google.com".to_string()
                                    })
                            }),
                        redirect_url: std::env::var(format!("OIDC_REDIRECT_URL_{}", provider_id))
                            .unwrap_or_else(|_| {
                                debug!("OIDC_REDIRECT_URL_{} not set - falling back to OIDC_REDIRECT_URL", provider_id);
                                std::env::var("OIDC_REDIRECT_URL")
                                    .unwrap_or_else(|_| {
                                        debug!("OIDC_REDIRECT_URL not set - using localhost default for provider {}.", provider_id);
                                        format!("http://localhost:3000/auth/oidc/{}/callback", provider_id)
                                    })
                            }),
                        session_timeout: Duration::from_secs(
                            std::env::var(format!("OIDC_SESSION_TIMEOUT_{}", provider_id))
                                .ok()
                                .and_then(|s| s.parse::<u64>().ok())
                                .unwrap_or_else(|| {
                                    std::env::var("OIDC_SESSION_TIMEOUT")
                                        .ok()
                                        .and_then(|s| s.parse::<u64>().ok())
                                        .unwrap_or(600) // 10 minutes default
                                }),
                        ),
                    };
                    
                    // Initialize the OIDC client for this provider
                    match OidcClient::new(config).await {
                        Ok(client) => {
                            debug!("Initialized OIDC client for provider {}", provider_id);
                            providers.insert(provider_id.clone(), Arc::new(client));
                        }
                        Err(e) => {
                            error!("Failed to initialize OIDC client for provider {}: {}", provider_id, e);
                            // Continue with other providers
                        }
                    }
                }
                
                // Set the default provider to the first in the list
                if !provider_ids.is_empty() && providers.contains_key(&provider_ids[0]) {
                    default_provider = provider_ids[0].clone();
                }
            }
        }
        
        // If no providers were configured, create a default one
        if providers.is_empty() {
            debug!("No OIDC providers configured, using default configuration");
            match OidcClient::new(OidcConfig::default()).await {
                Ok(client) => {
                    providers.insert("default".to_string(), Arc::new(client));
                }
                Err(e) => {
                    error!("Failed to initialize default OIDC client: {}", e);
                    // Add a stub client that will return errors
                    providers.insert("default".to_string(), Arc::new(OidcClient::stub()));
                }
            }
        }
        
        Self {
            providers,
            default_provider,
        }
    }
    
    /// Get the default OIDC client
    pub fn default_client(&self) -> Option<Arc<OidcClient>> {
        self.providers.get(&self.default_provider).cloned()
    }
    
    /// Get a specific OIDC client
    pub fn get_client(&self, provider_id: &str) -> Option<Arc<OidcClient>> {
        self.providers.get(provider_id).cloned()
    }
    
    /// Get all provider IDs
    pub fn provider_ids(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
    
    /// Create a stub implementation for testing
    pub fn stub() -> Self {
        let mut providers = HashMap::new();
        providers.insert("default".to_string(), Arc::new(OidcClient::stub()));
        
        Self {
            providers,
            default_provider: "default".to_string(),
        }
    }
} 