use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tracing::info;

/// Types of authentication events
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AuthEventType {
    /// User login attempt
    Login,
    /// User logout
    Logout,
    /// Token refresh
    TokenRefresh,
    /// Token revocation
    TokenRevocation,
    /// Auth0 callback
    OidcCallback,
    /// Password reset
    PasswordReset,
    /// User registration
    Registration,
    /// Failed login attempt
    FailedLogin,
    /// Access denied to resource
    AccessDenied,
    /// Session expired
    SessionExpired,
    /// Token validation
    TokenValidation,
}

impl std::fmt::Display for AuthEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthEventType::Login => write!(f, "LOGIN"),
            AuthEventType::Logout => write!(f, "LOGOUT"),
            AuthEventType::TokenRefresh => write!(f, "TOKEN_REFRESH"),
            AuthEventType::TokenRevocation => write!(f, "TOKEN_REVOCATION"),
            AuthEventType::OidcCallback => write!(f, "OIDC_CALLBACK"),
            AuthEventType::PasswordReset => write!(f, "PASSWORD_RESET"),
            AuthEventType::Registration => write!(f, "REGISTRATION"),
            AuthEventType::FailedLogin => write!(f, "FAILED_LOGIN"),
            AuthEventType::AccessDenied => write!(f, "ACCESS_DENIED"),
            AuthEventType::SessionExpired => write!(f, "SESSION_EXPIRED"),
            AuthEventType::TokenValidation => write!(f, "TOKEN_VALIDATION"),
        }
    }
}

/// Authentication event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthEvent {
    /// Type of authentication event
    pub event_type: AuthEventType,
    /// User ID (if available)
    pub user_id: Option<String>,
    /// Timestamp when the event occurred
    pub timestamp: DateTime<Utc>,
    /// IP address of the client
    pub ip_address: Option<String>,
    /// User agent string from the client
    pub user_agent: Option<String>,
    /// Whether the event was successful
    pub success: bool,
    /// Additional details about the event
    pub details: Option<String>,
    /// The resource being accessed (if applicable)
    pub resource: Option<String>,
    /// Duration of the operation in milliseconds (if applicable)
    pub duration_ms: Option<u64>,
    /// Authentication method used (password, token, etc.)
    pub auth_method: Option<String>,
}

impl AuthEvent {
    /// Create a new authentication event
    pub fn new(event_type: AuthEventType, user_id: Option<&str>, success: bool) -> Self {
        Self {
            event_type,
            user_id: user_id.map(String::from),
            timestamp: Utc::now(),
            ip_address: None,
            user_agent: None,
            success,
            details: None,
            resource: None,
            duration_ms: None,
            auth_method: None,
        }
    }
    
    /// Set the IP address
    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }
    
    /// Set the user agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }
    
    /// Set the details
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
    
    /// Set the resource
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }
    
    /// Set the duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }
    
    /// Set the authentication method
    pub fn with_auth_method(mut self, auth_method: impl Into<String>) -> Self {
        self.auth_method = Some(auth_method.into());
        self
    }
}

/// Log an authentication event
pub fn log_auth_event(event: AuthEvent) {
    // Log to tracing
    let user_id_str = event.user_id.as_deref().unwrap_or("anonymous");
    let status = if event.success { "SUCCESS" } else { "FAILURE" };
    let details = event.details.as_deref().unwrap_or("");
    
    info!(
        "AUTH-LOG [{}] [{}] [{}] [{}] {}",
        event.event_type,
        user_id_str,
        status,
        event.timestamp.to_rfc3339(),
        details
    );
    
    // In a real application, we'd also log to a database or other persistent storage
    #[cfg(feature = "db-logging")]
    {
        if let Err(e) = store_auth_event_in_database(&event) {
            error!("Failed to store auth event in database: {}", e);
        }
    }
}

/// Log a successful login
pub fn log_successful_login(user_id: &str, ip_address: Option<&str>, user_agent: Option<&str>) {
    let mut event = AuthEvent::new(AuthEventType::Login, Some(user_id), true)
        .with_auth_method("password");
    
    if let Some(ip) = ip_address {
        event = event.with_ip(ip);
    }
    
    if let Some(ua) = user_agent {
        event = event.with_user_agent(ua);
    }
    
    log_auth_event(event);
}

/// Log a failed login attempt
pub fn log_failed_login(username: &str, ip_address: Option<&str>, reason: &str) {
    let mut event = AuthEvent::new(AuthEventType::FailedLogin, Some(username), false)
        .with_details(reason)
        .with_auth_method("password");
    
    if let Some(ip) = ip_address {
        event = event.with_ip(ip);
    }
    
    log_auth_event(event);
}

/// Log a successful token validation
pub fn log_token_validation(user_id: &str, token_type: &str, success: bool) {
    let event = AuthEvent::new(AuthEventType::TokenValidation, Some(user_id), success)
        .with_auth_method(token_type);
    
    log_auth_event(event);
}

/// Log a token refresh
pub fn log_token_refresh(user_id: &str, success: bool, details: Option<&str>) {
    let mut event = AuthEvent::new(AuthEventType::TokenRefresh, Some(user_id), success);
    
    if let Some(d) = details {
        event = event.with_details(d);
    }
    
    log_auth_event(event);
}

/// Log a logout event
pub fn log_logout(user_id: &str) {
    let event = AuthEvent::new(AuthEventType::Logout, Some(user_id), true);
    log_auth_event(event);
}

/// Log a token revocation
pub fn log_token_revocation(user_id: &str, reason: Option<&str>) {
    let mut event = AuthEvent::new(AuthEventType::TokenRevocation, Some(user_id), true);
    
    if let Some(r) = reason {
        event = event.with_details(r);
    }
    
    log_auth_event(event);
}

/// Log an access denied event
pub fn log_access_denied(user_id: &str, resource: &str, required_roles: &[String]) {
    let details = format!("Required roles: {}", required_roles.join(", "));
    
    let event = AuthEvent::new(AuthEventType::AccessDenied, Some(user_id), false)
        .with_resource(resource)
        .with_details(details);
    
    log_auth_event(event);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_auth_event() {
        let event = AuthEvent::new(AuthEventType::Login, Some("user123"), true)
            .with_ip("192.168.1.1")
            .with_user_agent("Mozilla/5.0")
            .with_details("Login from dashboard")
            .with_resource("/admin")
            .with_duration(150)
            .with_auth_method("password");
        
        assert_eq!(event.event_type as u8, AuthEventType::Login as u8);
        assert_eq!(event.user_id, Some("user123".to_string()));
        assert_eq!(event.success, true);
        assert_eq!(event.ip_address, Some("192.168.1.1".to_string()));
        assert_eq!(event.user_agent, Some("Mozilla/5.0".to_string()));
        assert_eq!(event.details, Some("Login from dashboard".to_string()));
        assert_eq!(event.resource, Some("/admin".to_string()));
        assert_eq!(event.duration_ms, Some(150));
        assert_eq!(event.auth_method, Some("password".to_string()));
    }
    
    #[test]
    fn test_event_type_display() {
        assert_eq!(AuthEventType::Login.to_string(), "LOGIN");
        assert_eq!(AuthEventType::Logout.to_string(), "LOGOUT");
        assert_eq!(AuthEventType::FailedLogin.to_string(), "FAILED_LOGIN");
    }
} 