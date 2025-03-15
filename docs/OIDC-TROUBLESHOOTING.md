# Auth0 OIDC Integration Troubleshooting Guide

This guide documents the issues we encountered integrating Auth0 with our application and how we resolved them.

## The Problem

We faced several issues with the Auth0 OIDC integration:

1. Authentication failed with `ID token verification failed: Invalid nonce: nonce mismatch`
2. Authorization codes being rejected as "expired" or "invalid"
3. Session management issues during the OIDC flow

## Root Causes Identified

1. **Incorrect Nonce Handling**: The nonce generated when creating the Auth URL was not being properly stored in the session
2. **Session Storage Issues**: Sessions were not being properly tracked and retrieved
3. **JSON Timestamp Format**: Auth0's JWT tokens use RFC3339 timestamp format, which wasn't being accepted by the OpenID Connect library

## Step-by-Step Fix

### 1. Enhanced Session Structure

We updated the `OidcSession` struct to include a `nonce` field that explicitly stores the nonce value:

```rust
pub struct OidcSession {
    pub id: String,
    pub created: SystemTime,
    pub pkce_verifier: Option<PkceCodeVerifier>,
    pub nonce: String,  // Added this field
}
```

### 2. Improved Nonce Generation and Storage

In the `start_auth_flow` method, we modified the code to:

1. Generate a nonce value using UUID
2. Store the nonce in the session structure
3. Use the same nonce when generating the authorization URL

```rust
// Generate random nonce
let nonce_str = Uuid::new_v4().to_string();
let nonce = Nonce::new(nonce_str.clone());

// Create session with explicit nonce value
let session = OidcSession {
    id: session_id.to_string(),
    created: SystemTime::now(),
    pkce_verifier: Some(pkce_verifier),
    nonce: nonce_str,
};
```

### 3. Fixed Nonce Verification

In the `handle_callback` method, we now use the nonce from the session for verification:

```rust
// Use the nonce from the session for verification
let nonce = Nonce::new(session.nonce.clone());

// Verify the ID token using the session nonce
let id_token = id_token_verifier.verify(
    response.id_token().ok_or(OidcError::MissingIdToken)?,
    &nonce,
    &None,
)?;
```

### 4. Added Debug Logging

We added extensive logging to help diagnose session and token issues:

```rust
debug!("Current OIDC sessions ({}): {:?}", session_count, current_sessions);

debug!("Retrieved session: id={}, created={:?}, nonce={}", 
    session.id, session.created, session.nonce);
```

### 5. Updated OpenID Connect Library

We added the `accept-rfc3339-timestamps` feature to the OpenID Connect library in `Cargo.toml`:

```toml
openidconnect = { version = "3.4", features = ["reqwest", "accept-rfc3339-timestamps"] }
```

## Testing the Integration

1. Use the test tools (`test-api-auth0.html` or `debug-auth0-complete.html`) to initiate the login flow
2. Monitor logs for session creation and nonce generation
3. Complete Auth0 login process
4. Verify that the callback succeeds with proper nonce verification

## Common Issues and Solutions

1. **Session Not Found**: Check that cookies are being properly set and sent. Ensure same-site and secure settings are appropriate.
2. **Nonce Mismatch**: Ensure the nonce generated during authorization URL creation is saved in the session and retrieved during callback.
3. **Expired Authorization Code**: Auth0 codes are single-use and short-lived. Try again with a fresh code.
4. **CORS Issues**: If testing from a different domain than your API, you may encounter CORS issues.

## Auth0 Configuration Best Practices

1. Set Application Type to **Regular Web Application**
2. Configure Token Endpoint Authentication Method to **Post**
3. Set Allowed Callback URLs to include your application's callback URL
4. Enable OIDC Conformant
5. Set JsonWebToken Signature Algorithm to **RS256**
6. Enable Authorization Code grant type and disable Implicit grant type 