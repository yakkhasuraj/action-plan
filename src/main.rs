use dotenvy_macro::dotenv;
use server::run;
// use std::error::Error;

mod database;
mod handlers;
mod middlewares;
mod routes;
mod server;
mod utils;

#[tokio::main]
async fn main() /* -> Result<(), Box<dyn Error>> */
{
    dotenvy::dotenv().ok();

    // for (key, value) in env::vars() {
    //     println!("{key}: {value}");
    // }

    let database_uri = dotenv!("DATABASE_URL");

    run(database_uri).await;

    // Ok(())
}
