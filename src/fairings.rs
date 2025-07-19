use log::info;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Data, Request};

pub struct RequestLogger;

#[rocket::async_trait]
impl Fairing for RequestLogger {
    fn info(&self) -> Info {
        Info {
            name: "Request Logger",
            kind: Kind::Request,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _: &mut Data<'_>) {
        info!(
            "{} {} {}",
            req.method(),
            req.uri(),
            req.headers().get_one("User-Agent").unwrap_or("Unknown")
        );
    }
}
