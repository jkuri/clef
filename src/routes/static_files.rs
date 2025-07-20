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

/// Serve static files (CSS, JS, images, etc.)
#[get("/<file..>", rank = 10)]
pub fn static_files(file: PathBuf) -> Option<(ContentType, Vec<u8>)> {
    let path = file.display().to_string();
    let file_content = ASSETS.get_file(&path)?;

    // Determine content type based on file extension
    let content_type = match file.extension()?.to_str()? {
        "html" => ContentType::HTML,
        "css" => ContentType::CSS,
        "js" => ContentType::JavaScript,
        "json" => ContentType::JSON,
        "png" => ContentType::PNG,
        "jpg" | "jpeg" => ContentType::JPEG,
        "gif" => ContentType::GIF,
        "svg" => ContentType::SVG,
        "ico" => ContentType::Icon,
        "woff" => ContentType::WOFF,
        "woff2" => ContentType::WOFF2,
        "ttf" => ContentType::TTF,
        "otf" => ContentType::OTF,
        _ => ContentType::Binary,
    };

    Some((content_type, file_content.contents().to_vec()))
}

/// Handle HEAD requests for static files
#[head("/<file..>", rank = 10)]
pub fn static_files_head(file: PathBuf) -> Option<(ContentType, ())> {
    let path = file.display().to_string();
    let _file_content = ASSETS.get_file(&path)?;

    // Determine content type based on file extension
    let content_type = match file.extension()?.to_str()? {
        "html" => ContentType::HTML,
        "css" => ContentType::CSS,
        "js" => ContentType::JavaScript,
        "json" => ContentType::JSON,
        "png" => ContentType::PNG,
        "jpg" | "jpeg" => ContentType::JPEG,
        "gif" => ContentType::GIF,
        "svg" => ContentType::SVG,
        "ico" => ContentType::Icon,
        "woff" => ContentType::WOFF,
        "woff2" => ContentType::WOFF2,
        "ttf" => ContentType::TTF,
        "otf" => ContentType::OTF,
        _ => ContentType::Binary,
    };

    Some((content_type, ()))
}

/// Get all static file routes
pub fn get_static_routes() -> Vec<Route> {
    routes![index, static_files, static_files_head]
}
