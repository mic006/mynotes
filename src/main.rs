mod config;

use base64::{Engine as _, engine::general_purpose};
use rocket::http::{Header, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::{self, Responder, Response};

use config::AppConfig;

/// Request guard that ensures a user is authenticated via Basic Auth.
struct AuthenticatedUser(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();

    /// Extract and validate the Authorization header.
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // Retrieve the AppConfig from Rocket's managed state.
        let config = request
            .rocket()
            .state::<AppConfig>()
            .expect("Config not managed");

        // Check for the "Authorization" header.
        let auth_header = request.headers().get_one("Authorization");

        if let Some(auth) = auth_header
            // Expecting "Basic <base64_encoded_credentials>"
            && let Some(encoded) = auth.strip_prefix("Basic ")
            && let Ok(decoded_bytes) = general_purpose::STANDARD.decode(encoded)
            && let Ok(decoded) = String::from_utf8(decoded_bytes)
        {
            let parts: Vec<&str> = decoded.splitn(2, ':').collect();
            if parts.len() == 2 {
                let (user, pass) = (parts[0], parts[1]);
                // Validate credentials against the configuration.
                if config.users.get(user).is_some_and(|p| p == pass) {
                    return Outcome::Success(AuthenticatedUser(user.to_string()));
                }
            }
        }

        Outcome::Error((Status::Unauthorized, ()))
    }
}

/// Responder that triggers a Basic Auth prompt in the browser.
struct BasicAuthPrompt;

impl<'r> Responder<'r, 'static> for BasicAuthPrompt {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            // Returns a 401 Unauthorized with the proper header.
            .status(Status::Unauthorized)
            .header(Header::new("WWW-Authenticate", "Basic realm=\"My Notes\""))
            .ok()
    }
}

#[rocket::get("/")]
/// Serves the index.html file only to authenticated users.
async fn index(_user: AuthenticatedUser) -> Option<rocket::fs::NamedFile> {
    rocket::fs::NamedFile::open("index.html").await.ok()
}

#[rocket::catch(401)]
/// Catch-all for unauthorized requests, returning the Basic Auth challenge.
fn unauthorized() -> BasicAuthPrompt {
    BasicAuthPrompt
}

#[rocket::launch]
/// Main entry point for the Rocket application.
fn rocket() -> _ {
    let rocket = rocket::build();

    // Extract the custom "app" section from rocket.toml
    let app_config: AppConfig = rocket
        .figment()
        .extract_inner("app")
        .expect("Configuration 'app' section is missing in Rocket.toml");

    rocket
        // Inject the loaded configuration into Rocket's state.
        .manage(app_config)
        .mount("/", rocket::routes![index])
        .register("/", rocket::catchers![unauthorized])
}
