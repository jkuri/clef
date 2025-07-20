use include_dir::{Dir, include_dir};
use rocket::http::ContentType;
use rocket::response::content::RawHtml;
use rocket::{Route, get, head, routes};
use std::path::PathBuf;

// Include the static files from web/clef/dist at compile time
static ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/web/clef/dist");

/// Serve the main index.html file for the root route
#[get("/")]
pub fn index() -> RawHtml<&'static str> {
    RawHtml(ASSETS.get_file("index.html").map_or("Not found", |f| {
        std::str::from_utf8(f.contents()).unwrap_or("Invalid UTF-8")
    }))
}

/// Serve static files (CSS, JS, images, etc.) or fallback to SPA
#[get("/<file..>", rank = 10)]
pub fn static_files(file: PathBuf) -> (ContentType, Vec<u8>) {
    let path = file.display().to_string();

    // Try to serve static file first
    if let Some(file_content) = ASSETS.get_file(&path) {
        // Determine content type based on file extension
        let content_type = match file.extension().and_then(|ext| ext.to_str()) {
            Some("html") => ContentType::HTML,
            Some("css") => ContentType::CSS,
            Some("js") => ContentType::JavaScript,
            Some("json") => ContentType::JSON,
            Some("png") => ContentType::PNG,
            Some("jpg") | Some("jpeg") => ContentType::JPEG,
            Some("gif") => ContentType::GIF,
            Some("svg") => ContentType::SVG,
            Some("ico") => ContentType::Icon,
            Some("woff") => ContentType::WOFF,
            Some("woff2") => ContentType::WOFF2,
            Some("ttf") => ContentType::TTF,
            Some("otf") => ContentType::OTF,
            _ => ContentType::Binary,
        };

        return (content_type, file_content.contents().to_vec());
    }

    // If no static file found, serve index.html for SPA routing
    let index_content = ASSETS
        .get_file("index.html")
        .map(|f| f.contents().to_vec())
        .unwrap_or_else(|| b"Not found".to_vec());

    (ContentType::HTML, index_content)
}

/// Handle HEAD requests for static files or SPA fallback
#[head("/<file..>", rank = 10)]
pub fn static_files_head(file: PathBuf) -> (ContentType, ()) {
    let path = file.display().to_string();

    // Try to serve static file first
    if let Some(_file_content) = ASSETS.get_file(&path) {
        // Determine content type based on file extension
        let content_type = match file.extension().and_then(|ext| ext.to_str()) {
            Some("html") => ContentType::HTML,
            Some("css") => ContentType::CSS,
            Some("js") => ContentType::JavaScript,
            Some("json") => ContentType::JSON,
            Some("png") => ContentType::PNG,
            Some("jpg") | Some("jpeg") => ContentType::JPEG,
            Some("gif") => ContentType::GIF,
            Some("svg") => ContentType::SVG,
            Some("ico") => ContentType::Icon,
            Some("woff") => ContentType::WOFF,
            Some("woff2") => ContentType::WOFF2,
            Some("ttf") => ContentType::TTF,
            Some("otf") => ContentType::OTF,
            _ => ContentType::Binary,
        };

        return (content_type, ());
    }

    // If no static file found, return HTML content type for SPA fallback
    (ContentType::HTML, ())
}

/// Get all static file routes
pub fn get_static_routes() -> Vec<Route> {
    routes![index, static_files, static_files_head]
}
