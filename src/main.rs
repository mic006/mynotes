mod config;
mod index;

use base64::{Engine as _, engine::general_purpose};
use pulldown_cmark::{Options, Parser, html};
use rocket::fs::NamedFile;
use rocket::http::{Header, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::{self, Responder, Response, content::RawHtml};
use std::path::PathBuf;

use config::AppConfig;

/// Markdown options used for all parsing operations.
fn get_markdown_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_GFM
}

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
            .header(Header::new("WWW-Authenticate", "Basic realm=\"MyNotes\""))
            .ok()
    }
}

/// A custom responder to handle either generated HTML content or static files.
#[derive(rocket::Responder)]
enum GetResponse {
    /// Generated HTML content
    Html(RawHtml<String>),
    /// Static file
    File(NamedFile),
}

#[rocket::get("/<file..>")]
/// Serves content to authenticated users.
///
/// - static files are served as is
/// - markdown files are converted to HTML
async fn get(
    file: PathBuf,
    _user: AuthenticatedUser,
    config: &rocket::State<AppConfig>,
) -> Option<GetResponse> {
    // Specific handling for the main page (empty path).
    if file.as_os_str().is_empty() {
        let mut root_node = index::Node::default();
        index::walk(
            config.content_path.clone(),
            &config.content_path,
            &mut root_node,
        )
        .await;
        let mut html_output = String::from(
            "<!DOCTYPE html><html><head><title>My Notes</title></head><body><h1>Notes Index</h1>",
        );
        root_node.render(&mut html_output);
        html_output.push_str("</body></html>");
        return Some(GetResponse::Html(RawHtml(html_output)));
    }

    let mut path = config.content_path.clone();
    path.push(file);

    // If the file exists and is not markdown (e.g. an image), serve it directly.
    if let Ok(meta) = rocket::tokio::fs::metadata(&path).await {
        if meta.is_dir() {
            return None;
        }
        if path.extension().is_none_or(|ext| ext != "md") {
            return NamedFile::open(path).await.ok().map(GetResponse::File);
        }
    } else if path.extension().is_none() {
        // If the file doesn't exist and has no extension, try appending .md.
        path.set_extension("md");
    }

    let content = rocket::tokio::fs::read_to_string(path).await.ok()?;

    let parser = Parser::new_ext(&content, get_markdown_options());
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    Some(GetResponse::Html(RawHtml(html_output)))
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
        .mount("/", rocket::routes![get])
        .register("/", rocket::catchers![unauthorized])
}
