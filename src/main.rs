//! Self-hosted website to publish personal notes, in markdown format

mod config;
mod index;
mod markdown;
mod settings;

use std::env;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::SystemTime;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use regex::{Captures, Regex};
use rocket::fs::NamedFile;
use rocket::http::{Header, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::{self, Responder, Response, content::RawHtml};
use rocket_async_compression::Compression;

use config::AppConfig;

use crate::markdown::MarkdownFile;

/// Pattern in template file, where title shall be inserted.
const TEMPLATE_PATTERN_TITLE: &str = "%TITLE%";
/// Pattern in template file, where content shall be inserted.
const TEMPLATE_PATTERN_CONTENT: &str = "%CONTENT%";

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
            && let Ok(decoded_bytes) = BASE64_STANDARD.decode(encoded)
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
enum GetResponse {
    /// Generated HTML content
    Html(RawHtml<String>),
    /// Static file
    File(NamedFile),
}
impl GetResponse {
    /// Build HTML from template
    fn build_html(config: &AppConfig, title: &str, body: &str) -> Self {
        static RE_STATIC_RESOURCE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r#"(src|href)="([^"]*)""#).unwrap());

        let html_output = config
            .template_content
            .replace(TEMPLATE_PATTERN_TITLE, title)
            .replace(TEMPLATE_PATTERN_CONTENT, body);

        // Cache busting: add "?mtime=<mtime>" to local static resources to allow efficient cache
        // => each resource is considered immutable and can be cached forever
        // => on a resource change, mtime is different, so URL is different and resource is retrieved
        let html_output = RE_STATIC_RESOURCE.replace_all(&html_output, |caps: &Captures<'_>| {
            let (unchanged, [attr, path]) = caps.extract();
            if !path.contains("://") && !path.contains("/.") {
                // check if file exists
                // TODO use trim_prefix once available
                let full_path = config.content_path.join(path.trim_start_matches('/'));
                if let Ok(meta) = std::fs::metadata(full_path) {
                    let mtime = meta
                        .modified()
                        .unwrap()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    return format!(r#"{attr}="{path}?mtime={mtime}""#);
                }
            }
            unchanged.to_string()
        });

        GetResponse::Html(RawHtml(html_output.to_string()))
    }
}

// add cache control header
impl<'r> Responder<'r, 'static> for GetResponse {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        match self {
            GetResponse::Html(html) => Response::build_from(html.respond_to(req)?)
                .header(Header::new("Cache-Control", "private, no-cache"))
                .ok(),
            // static files are immutable thanks to cache busting done by `build_html`
            GetResponse::File(file) => Response::build_from(file.respond_to(req)?)
                .header(Header::new(
                    "Cache-Control",
                    "private, max-age=31536000, immutable",
                ))
                .ok(),
        }
    }
}

/// Serves content to authenticated users.
///
/// - static files are served as is
/// - markdown files are converted to HTML
#[rocket::get("/<file..>")]
async fn get(
    file: PathBuf,
    _user: AuthenticatedUser,
    config: &rocket::State<AppConfig>,
) -> Option<GetResponse> {
    // Specific handling for the main page (empty path).
    if file.as_os_str().is_empty() {
        let mut root_node = index::Dir::default();
        index::walk(
            config.content_path.clone(),
            &config.content_path,
            &mut root_node,
            config,
        );
        let mut body_content = String::new();
        root_node.render(&mut body_content, config);
        return Some(GetResponse::build_html(
            config,
            "MyNotes - Index",
            &body_content,
        ));
    }

    let mut path = config.content_path.clone();
    path.push(&file);

    // If the file exists and is not markdown (e.g. an image), serve it directly.
    if let Ok(meta) = std::fs::metadata(&path) {
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

    let md_file = MarkdownFile::read(&file.to_string_lossy(), true, config)?;
    Some(GetResponse::build_html(
        config,
        &md_file.title,
        &md_file.html.unwrap(),
    ))
}

#[rocket::catch(401)]
/// Catch-all for unauthorized requests, returning the Basic Auth challenge.
fn unauthorized() -> BasicAuthPrompt {
    BasicAuthPrompt
}

#[rocket::launch]
/// Main entry point for the Rocket application.
fn rocket() -> _ {
    // Check for --version or -V argument
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        println!("mynotes version {}", env!("BUILD_GIT_VERSION"));
        std::process::exit(0);
    }

    let rocket = rocket::build();

    // Extract the custom "app" section from rocket.toml
    let mut app_config: AppConfig = rocket
        .figment()
        .extract_inner("app")
        .expect("Configuration 'app' section is missing in Rocket.toml");
    app_config
        .load_template()
        .expect("Failed to load template file");

    rocket
        // Inject the loaded configuration into Rocket's state.
        .manage(app_config)
        .attach(Compression::fairing())
        .mount("/", rocket::routes![get])
        .register("/", rocket::catchers![unauthorized])
}
