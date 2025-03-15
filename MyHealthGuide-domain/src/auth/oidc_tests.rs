#[cfg(test)]
mod oidc_tests {
    use crate::auth::oidc::{OidcClient, OidcError};
    
    
    use std::time::Duration;

    #[tokio::test]
    async fn test_stub_client_creation() {
        let client = OidcClient::stub();
        assert_eq!(client.get_client_id(), "stub-client-id");
        assert_eq!(client.get_client_secret(), "stub-client-secret");
        assert_eq!(client.get_issuer_url(), "https://stub-issuer.example.com");
        assert_eq!(client.get_redirect_url(), "http://localhost:3000/callback");
        assert_eq!(client.get_session_timeout(), Duration::from_secs(600));
    }

    #[tokio::test]
    async fn test_stub_callback_success() {
        let client = OidcClient::stub();
        
        // Test successful callback
        let result = client.handle_callback("test_code", "test_state").await;
        assert!(result.is_ok());
        
        let user_info = result.unwrap();
        assert_eq!(user_info.user_id, "test-user-123");
        assert_eq!(user_info.roles, vec!["user".to_string()]);
        assert_eq!(user_info.email, Some("test@example.com".to_string()));
        assert_eq!(user_info.name, Some("Test User".to_string()));
        assert_eq!(user_info.picture, Some("https://example.com/avatar.png".to_string()));
        assert_eq!(user_info.auth_source, "oidc");
    }
    
    #[tokio::test]
    async fn test_stub_callback_error() {
        let client = OidcClient::stub();
        
        // Test error callback
        let result = client.handle_callback("test_error_code", "test_state").await;
        assert!(result.is_err());
        
        match result {
            Err(OidcError::TokenExchangeError(msg)) => {
                assert_eq!(msg, "Test error");
            },
            _ => panic!("Expected TokenExchangeError"),
        }
    }
    
    // Add more tests as needed
} 