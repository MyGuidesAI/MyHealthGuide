# Auth0 Integration for MyHealthGuide API

## Overview

This document describes the Auth0 integration with the MyHealthGuide API. The API uses OpenID Connect (OIDC) to authenticate users with Auth0 as the identity provider.

## Configuration

The integration uses the following environment variables:

```
ENABLE_OIDC=true
OIDC_CLIENT_ID=your_client_id_here
OIDC_CLIENT_SECRET=your_client_secret_here
OIDC_REDIRECT_URL=http://localhost:3000/auth/oidc/callback
OIDC_ISSUER_URL=https://your-auth0-domain.auth0.com/
OIDC_SESSION_TIMEOUT=3600
JWT_AUDIENCE=myhealth-client
JWKS_CACHE_HOURS=24
```

## Key Components

1. **OIDC Client**: Implemented in `myhealth-domain/src/auth/oidc.rs`
2. **Auth Routes**: Implemented in `myhealth-domain/src/auth/routes.rs` 
3. **Session Management**: In-memory session store with CSRF and nonce verification
4. **JWT Validation**: Middleware for validating Auth0 JWTs in `myhealth-domain/src/auth/auth0.rs`
5. **Authorization**: Role-based access control in `myhealth-domain/src/auth/authorize.rs`
6. **Auth Logging**: Comprehensive auth event logging in `myhealth-domain/src/auth/logging.rs`

## Authentication Flow

1. User initiates login at `/auth/oidc/login`
2. API creates OIDC session with PKCE and nonce values
3. User is redirected to Auth0 login page
4. After successful login, Auth0 redirects to `/auth/oidc/callback` with authorization code
5. API exchanges code for tokens and verifies the ID token
6. User is authenticated and a session cookie is set

## API Authentication Flow

1. Client obtains an access token from Auth0 (direct implementation)
2. Client includes the token in the Authorization header: `Authorization: Bearer {token}`
3. API validates the token using our new JWT validation middleware
4. If valid, the request is processed with the user's context

## Recent Fixes and Enhancements

1. **Nonce Mismatch**: Fixed by explicitly storing and retrieving the nonce in the session
2. **Session Management**: Improved with proper storage and retrieval of session data
3. **RFC3339 Timestamps**: Added support for Auth0's RFC3339 timestamp format
4. **Debugging**: Enhanced logging throughout the authentication flow
5. **JWT Validation**: Added middleware for validating Auth0 JWTs
6. **Role-Based Access Control**: Implemented middleware for protecting routes based on roles
7. **Token Blacklisting**: Implemented token revocation and blacklisting
8. **Security Headers**: Enhanced security headers for auth-related endpoints
9. **Auth Event Logging**: Added comprehensive logging for all authentication events

## Testing Tools

1. **auth0-login-demo.html**: A simple HTML page for testing Auth0 login flow
2. **api-auth-test.html**: A page for testing API authentication with Auth0 tokens

## Recommendations

1. Use environment variables for all Auth0 configuration
2. Consider using a database for session storage in production
3. Regularly rotate client secrets
4. Implement a token refresh mechanism
5. Set up rate limiting for auth endpoints

## Auth Logging System

The application includes a comprehensive auth logging system that tracks all authentication-related events. This helps with debugging, security monitoring, and compliance requirements.

### Logged Events

- Login attempts (successful and failed)
- Token validation
- OIDC callbacks
- Access denied due to insufficient permissions
- Token revocation
- And more

### Log Format

Auth logs follow this format:

```
AUTH-LOG [EVENT_TYPE] [USER_ID] [SUCCESS/FAILURE] [TIMESTAMP] [DETAILS]
```

For example:

```
AUTH-LOG [LOGIN] [user123] [SUCCESS] [2023-10-15T14:30:22Z] User successfully authenticated via OIDC
AUTH-LOG [TOKEN_VALIDATION] [user456] [FAILURE] [2023-10-15T15:45:10Z] Token expired
AUTH-LOG [ACCESS_DENIED] [user789] [FAILURE] [2023-10-15T16:22:05Z] Required roles: admin
```

### Additional Data

Each log event also captures:

- IP address (when available)
- User agent (when available) 
- Auth method used (OIDC, JWT, etc.)
- Resource being accessed (for access control events)
- Operation duration in milliseconds
- Detailed error information (for failures)

This comprehensive logging system provides a complete audit trail of all authentication activities.

## JWT Validation Middleware

We've implemented a new JWT validation middleware that:

1. First tries to validate tokens using our internal JWT validation
2. If that fails, attempts to validate the token as an Auth0 JWT
3. Uses the Auth0 JWKS endpoint to validate token signatures
4. Caches JWKS keys for better performance
5. Extracts user information from validated tokens

This enables seamless authentication of API requests using tokens obtained directly from Auth0, without needing to go through our OIDC callback flow.

## Testing Tools

Several testing tools were created to help debug the Auth0 integration:

1. `debug-auth0-complete.html`: Complete debugging tool for the Auth0 flow
2. `test-api-auth0.html`: API testing tool for the OIDC endpoints
3. `auth0-login-demo.html`: Clean demo UI for testing the login flow
4. `api-auth-test.html`: Tool for testing direct API authentication using Auth0 tokens

## Recommended Future Improvements

1. **Persistent Session Storage**: Replace in-memory session store with Redis or database storage
2. **Enhanced Error Handling**: Add more detailed error messages for authentication failures
3. **Refresh Token Support**: Implement token refresh to extend session lifetime
4. **Authorization**: Implement role-based access control using Auth0 roles and permissions
5. **Multi-tenant Support**: Configure for supporting multiple organizations
6. **User Profile Sync**: Sync user profile data between Auth0 and local database
7. **Token Revocation**: Implement a token blacklist for invalidating tokens before they expire

## Common Issues

- **Session Not Found**: Check cookie settings and session timeout
- **Nonce Mismatch**: Ensure nonce is properly stored and retrieved
- **Expired Authorization Code**: Auth0 codes are single-use and short-lived
- **CORS Issues**: Configure correct CORS settings for cross-domain requests
- **Invalid JWT**: Check audience values and token signature algorithm

## Auth0 Dashboard Configuration

In the Auth0 dashboard, the application should be configured as follows:

1. **Application Type**: Regular Web Application
2. **Token Endpoint Authentication Method**: Post
3. **Allowed Callback URLs**: `http://localhost:3000/auth/oidc/callback`
4. **OIDC Conformant**: Enabled
5. **JsonWebToken Signature Algorithm**: RS256
6. **Grant Types**: Authorization Code (enabled), Implicit (disabled)
7. **APIs**: Define an API with the correct audience value (`myhealth-client`) 