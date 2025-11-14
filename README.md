# GOBSG
**G** -> Gistyr     
**O** -> OpenID Connect     
**B** -> Backend for Frontend      
**S** -> Session     
**G** -> Gateway   
#### A secure OIDC Backend-for-Frontend gateway providing cookie-based session management for web clients
## Licensing Summary
This project is licensed under the [PolyForm Small Business License 1.0.0](https://polyformproject.org/licenses/small-business/1.0.0/)
### What this means
- **If you are a solo developer, sole proprietorship, nonprofit, or any organization with fewer than 100 workers AND under $1M annual revenue** --> You may use, modify, and distribute this software **for free** under PolyForm Small Business terms.
- **If your company is over 100 workers or over $1M annual revenue** --> You must obtain a **commercial license**. 
  - Request a commercial license by contacting: `contact@gistyr.dev`
#### For full license text: 
See `LICENSES/LICENSE-POLYFORM-SMALL-BUSINESS.md` or visit <https://polyformproject.org/licenses/small-business/1.0.0>.
# Intended Architecture
**Three components are needed:** `web client`, `GOBSG`, `OpenID Provider` 
### Login Flow:
`web client` -> `GOBSG` -> `OpenID Provider` -> `GOBSG` -> `web client`
- Web client sends the user’s browser to GOBSG’s `/login` route.
- GOBSG creates a new session, adds security parameters to the session, and sends the user to the OpenID Provider's login page.
- The OpenID Provider will show the user a login screen, the user will login.
- OpenID Provider redirects back to GOBSG's /callback route, simultaneously passing user data to GOBSG in the form of a Json Web Token.
- GOBSG will validate the security parameters, check for errors, parse, and validate the JWT. If all is well, user data will be added to the session.
- GOBSG sends the browser back to the web client. The browser now carries a session cookie that identifies the server-side session.
### Session Status Flow:
`web client` -> `GOBSG` -> `web client`
- Web client calls on GOBSG’s `/sessionstatus` route.
- GOBSG reads the session cookie, verifies that a valid session exists, and checks if the access token is still valid.
  - If the access token is close to expiring, GOBSG refreshes it automatically in the background.
- GOBSG responds with either `"logged_in"` or `"not_logged_in"`, allowing the web client to update its UI accordingly.
### User Details Flow:
`web client` -> `GOBSG` -> `web client`
- Web client calls on GOBSG’s `/details` route.
- GOBSG reads the session cookie and retrieves the user's `username` and `user_id` stored in the session.
  - If the user is logged in, GOBSG returns that information.
  - If the user is not logged in, GOBSG either returns a default `username` and `user_id` or an error, depending on your configuration.
### Logout Flow:
`web client` -> `GOBSG` -> `OpenID Provider` -> `GOBSG` -> `web client`
- Web client sends the user’s browser to GOBSG’s `/logout` route.
- GOBSG clears the user’s session and redirects the browser to the OpenID Provider’s logout endpoint.
- After completing its logout process, the OpenID Provider redirects the browser back to GOBSG.
- GOBSG then redirects the browser back to the web client.


