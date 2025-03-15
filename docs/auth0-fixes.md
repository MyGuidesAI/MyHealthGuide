# Auth0 Integration Fixes

Based on the logs and error messages, here's a comprehensive solution to fix your Auth0 OIDC integration.

## Root Causes Identified

1. **JWT Timestamp Format Issue**: The timestamp format in Auth0's token is causing parsing errors with the `openidconnect` library. Auth0 uses RFC 3339 format, but the library expects a different format by default.

2. **Authorization Code Exchange Failures**: The authorization code is being rejected with "invalid_grant" errors, which can happen due to:
   - Expired authorization codes
   - Code already used once
   - PKCE validation failures
   - Redirect URI mismatches
   - Application configuration issues

## Step 1: Fix the JWT Timestamp Format Issue

You've already made the necessary change to `Cargo.toml` to enable RFC 3339 timestamp parsing:

```toml
openidconnect = { version = "3.4", features = ["reqwest", "accept-rfc3339-timestamps"] }
```

## Step 2: Verify and Update Auth0 Application Settings

Log into your Auth0 dashboard and check your application settings:

1. **Application Type**: Ensure it's set to "Regular Web Application"
2. **Token Endpoint Authentication Method**: Set to "POST"
3. **Allowed Callback URLs**: Must exactly match your API's callback URL (`http://localhost:3000/auth/oidc/callback`)
4. **Allowed Web Origins**: Add `http://localhost:3000`
5. **Allowed Logout URLs**: Add `http://localhost:3000`
6. **Grant Types**: Enable "Authorization Code" and disable "Implicit" if not needed
7. **Advanced Settings > OAuth > OIDC Conformant**: Ensure this is enabled
8. **Advanced Settings > OAuth > JsonWebToken Signature Algorithm**: Set to RS256

## Step 3: Fix PKCE Implementation

The PKCE implementation appears to be working, but ensure the code verifier is being properly stored and retrieved:

1. In `myhealth-domain/src/auth/oidc.rs`, verify that:
   - The `pkce_verifier` is being stored correctly in the `OidcSession`
   - The session is being retrieved correctly using the state parameter
   - The `set_pkce_verifier` call in the token exchange process uses the correct verifier

## Step 4: Check the Redirect URI

Make sure your environment variables match the configured redirect URI in Auth0:

```
OIDC_REDIRECT_URL=http://localhost:3000/auth/oidc/callback
```

Ensure this URL is exactly the same in:
- Your Docker environment
- The Auth0 dashboard "Allowed Callback URLs"
- The "Request Properties" section when you click Debug for a login attempt in Auth0

## Step 5: Test with the Direct HTML Test Page

Use the `troubleshoot-auth0-direct.html` file we created to test a direct login. This bypasses your API's login generation and attempts a login directly with Auth0.

## Step 6: Monitor the Auth0 Logs

In the Auth0 dashboard, go to "Monitoring > Logs" to see detailed information about auth attempts. Look for:
- Failed login attempts
- Authentication errors
- Token exchange errors

## Step 7: Additional Debugging

If issues persist:

1. Make sure to check the API logs during the process using:
   ```
   docker-compose logs -f dev
   ```

2. Check for any CORS issues if using the browser directly

3. Try temporarily adding more verbose logging in `handle_callback`

## Specific Codebase Fixes

If you're still having issues after updating the JWT timestamp feature, you might need to modify the following files:

**1. Authentication Flow Timeout:**
If your auth flow is timing out, increase the session timeout in your .env:
```
OIDC_SESSION_TIMEOUT=600
```

**2. Strict CSRF Token Validation:**
If you suspect CSRF token validation issues, check how the token is stored and validated.

## Final Note

If none of the above fixes work, we could consider lowering the security requirements temporarily by modifying `handle_callback` to be more permissive during development (NOT for production). 