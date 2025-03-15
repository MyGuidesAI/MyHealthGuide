# Auth Logging System for MyHealthGuide API

## Overview

The Auth Logging System is a comprehensive solution for tracking and monitoring all authentication-related events in the MyHealthGuide API. This system provides an audit trail for security, debugging, and compliance purposes, capturing detailed information about login attempts, token operations, and access control decisions.

## Key Features

- **Complete Event Coverage**: Logs all authentication-related events including logins, logouts, token validation, refresh, and revocation
- **Structured Logging**: Standardized log format with consistent fields for easy parsing and analysis
- **Detailed Context**: Captures contextual information such as IP addresses, user agents, resource paths, and duration
- **Success/Failure Tracking**: Explicitly records whether each authentication action succeeded or failed
- **Method Identification**: Tracks which authentication method was used (OIDC, JWT, password)
- **Performance Metrics**: Records duration of authentication operations for performance monitoring
- **Role-Based Access Control Auditing**: Logs access denied events with required roles information

## Log Format

Auth logs follow this standardized format:

```
AUTH-LOG [EVENT_TYPE] [USER_ID] [SUCCESS/FAILURE] [TIMESTAMP] [DETAILS]
```

Example logs:
```
AUTH-LOG [LOGIN] [user123] [SUCCESS] [2023-10-15T14:30:22Z] User successfully authenticated via OIDC
AUTH-LOG [TOKEN_VALIDATION] [user456] [FAILURE] [2023-10-15T15:45:10Z] Token expired
AUTH-LOG [ACCESS_DENIED] [user789] [FAILURE] [2023-10-15T16:22:05Z] Required roles: admin
```

## Event Types

The system tracks the following event types:

| Event Type | Description |
|------------|-------------|
| LOGIN | User login attempt (password or OIDC) |
| LOGOUT | User logout action |
| TOKEN_REFRESH | Refresh token operation |
| TOKEN_REVOCATION | Token revocation |
| OIDC_CALLBACK | Auth0 callback processing |
| FAILED_LOGIN | Failed login attempt |
| ACCESS_DENIED | Access denied due to insufficient permissions |
| TOKEN_VALIDATION | JWT token validation |
| SESSION_EXPIRED | Authentication session expiration |

## Implementation Details

The auth logging system is implemented in the `myhealth-domain/src/auth/logging.rs` module and integrated with the following components:

1. **OIDC Authentication Flow**:
   - Login handler - logs the start of auth flow and redirect to Auth0
   - Callback handler - logs authentication result from Auth0

2. **JWT Token Operations**:
   - Token validation - logs token verification results
   - Token refresh - logs refresh operations
   - Token revocation (logout) - logs token removal

3. **Authorization Middleware**:
   - Auth middleware - logs token validation and extraction
   - Role-based access control - logs access control decisions

4. **Login/Logout Operations**:
   - Password login - logs successful and failed login attempts
   - Logout - logs user logout events

## Usage in Development

To view auth logs during development:

```bash
docker-compose logs -f dev | grep "AUTH-LOG"
```

For more context around each log entry:

```bash
docker-compose logs -f dev | grep -A 3 "AUTH-LOG"
```

## Testing

A comprehensive test page has been provided in `auth-logging-test.html` that allows testing of various authentication scenarios:

- OIDC login flow
- Password-based authentication
- Token validation
- Token refresh
- Token revocation (logout)
- Access control testing

## Future Enhancements

1. **Database Integration**: Store auth logs in a database for long-term storage and analysis
2. **Alert Integration**: Set up alerts for suspicious activity patterns
3. **Dashboard Visualization**: Build a dashboard to visualize authentication trends
4. **Rate Limiting**: Track and limit failed authentication attempts
5. **IP-based Analysis**: Analyze authentication patterns by IP and geography 