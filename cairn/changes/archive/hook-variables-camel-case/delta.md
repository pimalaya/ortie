---
cairn: delta
change: hook-variables-camel-case
---

## ADDED Requirements

### Requirement: Hook variables are camelCase
A hook command SHALL receive, as environment variables, `accessToken`, `tokenType`, `expiresIn`, `refreshToken` and `scope` on success, and `error`, `errorDescription` and `errorUri` on error, each when the server answered it, and until 4.0 SHALL also receive each under its former name (`ACCESS_TOKEN`, `TOKEN_TYPE`, `EXPIRES_IN`, `REFRESH_TOKEN`, `SCOPE`, `ERROR`, `ERROR_DESCRIPTION`, `ERROR_URI`).

#### Scenario: A command reading the token
- **GIVEN** `hooks.on-issue.success.command = 'printf %s "$accessToken" > token'`
- **WHEN** a token is issued
- **THEN** the file holds the access token

#### Scenario: A 2.x command
- **GIVEN** `hooks.on-issue.success.command = 'printf %s "$ACCESS_TOKEN" > token'`
- **WHEN** a token is issued
- **THEN** the file holds the access token

## MODIFIED Requirements

## REMOVED Requirements
