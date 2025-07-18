use rocket::get;

#[get("/")]
pub async fn health_check() -> &'static str {
    "PNRS - Private NPM Registry Server is running!"
}
