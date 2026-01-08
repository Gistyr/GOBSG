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
- **If you are a solo developer, sole proprietorship, nonprofit, or any organization with fewer than 100 workers and under $1M annual revenue** --> You may use, modify, and distribute this software **for free** under PolyForm Small Business terms.
- **If your company is over 100 workers or over $1M annual revenue** --> You must obtain a **commercial license**. 
  - Request a commercial license at: [gobsg.dev](https://gobsg.dev)
#### For full license text: 
See `LICENSES/LICENSE-POLYFORM-SMALL-BUSINESS.md` or visit <https://polyformproject.org/licenses/small-business/1.0.0>.
### Learn
> The best way to understand this code is to read it.          
> The codebase is not very large, and there are descriptive comments.            
> Or have an LLM summarize it for you.               
# Intended Architecture  
**Three components are needed:** `web client`, `GOBSG`, `OpenID Provider` 
### Login Flow:
`web client` -> `GOBSG` -> `OpenID Provider` -> `GOBSG` -> `web client`
- Web client sends the user’s browser to GOBSG’s `/login` route.
- GOBSG creates a new session, adds security parameters to the session, and sends the user to the OpenID Provider's login page.
- The OpenID Provider will show the user a login screen, the user will login.
- OpenID Provider redirects back to GOBSG's /callback route, simultaneously passing user data to GOBSG in the form of a JSON Web Token.
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
# Settings
## main-config.toml
**Must be named `main-config.toml`**             
**Must be located in the same directory as the executable**             
**Read `Mandatory Settings` and `Optional Settings` below**          
**Do not change the order of any values in this file, because of serialization**              
```toml
# --- MANDATORY --- #
# --- See Mandatory Settings in README --- #

this_server_url = "" 
cookie_name = ""
cookie_domain = ""
secret_cookie_hex_key = ""
requesting_client_url = ""
issuer_url = ""
logout_url = ""
client = ""
client_secret = ""

# --- OPTIONAL --- #
# --- See Optional Settings in README --- #
# The values below above are the default values
# Leave commented out to use the default, uncomment to set your own

#listen_address = "0.0.0.0"  
#listen_port = 3090
#workers = 1 #default is: num_cpus::get() aka number of logical cores on the system
#redis_address = "redis://127.0.0.1:6379"

#heartbeat_logging = false
#heartbeat_interval_hours = 12
#machine_name = "machine"  
#container_name = "container"
#provider = "provider"

#keep_alive_time_secs = 75
#client_request_timeout_secs = 30
#client_disconnect_timeout_secs = 5
#max_connections = 25000

#early_refresh_skew_secs = 120

#user_details_fail_when_not_authenticated = true
#default_username = "0"
#default_user_id = "0"
```
### Mandatory Settings
- `this_server_url`: The URL for this server
    - "https://secure.mysite.com"
- `cookie_name`: The name of the cookie stored on the user's device
    - "mysite_bff_sid"
- `cookie_domain`: The domain for this server
    - "secure.mysite.com"
- `secret_cookie_hex_key`: Cryptographic key used to encrypt and sign session cookies
    - generate a 128-character hexadecimal string
- `requesting_client_url`: The URL of your client side application
    - "https://mysite.com"
- `issuer_url`: Your OpenID Connect provider’s “issuer”
    - This is unique to each provider
- `logout_url`: Your OpenID Connect provider's logout url
    - Also unique to each provider
- `client`: Your client ID
    - Given by your provider
- `client_secret`: Your client secret
    - Also given by your provider
### Optional Settings
- `listen_address`: IP/interface the server binds to
- `listen_port`: TCP port the server listens on
- `workers`: Number of Actix workers
    - Each worker runs its own event loop on a dedicated OS thread, adds concurrency
- `redis_address`: User sessions are stored in Redis
    - You must have Redis installed and running
- `heartbeat_logging`: Periodically emits a heartbeat log message
    - `heartbeat_interval_hours`: Interval between heartbeat logs, in hours
    - `machine_name`: The name of your machine 
    - `container_name`: The name of your container
    - `provider`: The name of your provider
- `keep_alive_time_secs`: HTTP keep-alive timeout for idle connections
- `client_request_timeout_secs`: Max time to wait for a full request from a client before timing out
- `client_disconnect_timeout_secs`: Grace period to finish work after a client disconnects
- `max_connections`: Upper limit on concurrent TCP connections accepted by the server
- `early_refresh_skew_secs`: grace period to refresh the access token slightly before it actually expires
    - Your access token validity must be longer than this time, by a few minutes ideally
- `user_details_fail_when_not_authenticated`: By default `/details` will fail if there is no session for the user
    - Set to `true` to return default values
    - `default_username`: Return this username for unauthenticated users
    - `default_user_id`: Return this user id for unauthenticated users
## logging-config.toml
**Must be named `logging-config.toml`**             
**Must be located in the same directory as the executable**                  
**Do not change the order of any values in this file, because of serialization**        
#### GOBSG uses the `better-logger` crate: [GitHub Link](https://github.com/Gistyr/better-logger) - [crates.io Link](https://crates.io/crates/better-logger) 
**Read the `better-logger` README for more information**           
- All the logging settings are the same as the `better-logger` documentation, except in `TOML` format
- `wasm_logging` and `async_logging` are purposely not a configurable option in `GOBSG`
  - Because `wasm_logging` must be `false` and `async_logging` must be `true`
  - All the other settings are the same, as seen in the `better-logger` documentation
```toml
terminal_logs = true 
terminal_log_lvl = ""
file_logs = false
file_log_lvl = ""
log_file_path = ""
network_logs = false
network_log_lvl = ""
network_endpoint_url = ""
debug_extra = true
[network_format]
type = "JsonText"
field = "text"
```
# Notes
- Your client-side application, this server, and your provider generally need to have the same domain name
  - Typically these 3 services would each have their own subdomain
- Your provider must issue JWT's that are compatible with the [openidconnect](https://crates.io/crates/openidconnect) crate
    - This generally is not an issue, but if you are using custom or non-standard JWTs then validation will fail
- This application is fairly specific in its use case
    - There is still work to be done to make it more general purpose
- You can see the documentation on `concat.sh` here: [LINK](https://github.com/Lozlof/concat.sh)
## Notice about the committed executable
#### Does not apply to crates.io, only applies to GitHub 
`cargo build --release --target x86_64-unknown-linux-gnu`                   
`/target/x86_64-unknown-linux-gnu/release/gobsg` is committed:                                 
**I do this so I don't have to build on the production VM.**          
**You should clean and rebuild your own executable.**                          
## Contributing
Contributions are welcome if you want to.          
# [gobsg.dev](https://gobsg.dev)