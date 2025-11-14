# GOBSG
**G** -> Gistyr     
**O** -> OpenID Connect     
**B** -> Backend for Frontend      
**S** -> Session     
**G** -> Gateway   
#### A secure OIDC Backend-for-Frontend gateway providing cookie-based session management for web clients
## Intended Architecture
**Three components are needed:** `web client`, `GOBSG`, `OpenID Provider` 
#### Login Flow:
`web client` -> `GOBSG` -> `OpenID Provider` -> `GOBSG` -> `web client`
- Web client sends the user’s browser to GOBSG’s `/login` route
- GOBSG creates a new session, adds security parameters to the session, and sends the user to the OpenID Provider's login page
- The OpenID Provider will show the user a login screen, the user will login
- OpenID Provider redirects back to GOBSG's /callback route, simultaneously passing user data to GOBSG in the form of a Json Web Token
- GOBSG will validate the security parameters, check for errors, parse, and validate the JWT. If all is well, user data will be added to the session.
- GOBSG sends the browser back to the web client. The browser now carries a session cookie that identifies the server-side session.







