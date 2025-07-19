use rocket::get;

#[get("/")]
pub async fn health_check() -> &'static str {
    "CLEF - Private NPM Registry Server is running!"
}
