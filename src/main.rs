use rocket::launch;

#[launch]
async fn rocket() -> _ {
    // Initialize logging
    env_logger::init();

    pnrs::create_rocket()
}
