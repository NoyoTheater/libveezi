use std::{env, error::Error};

use libveezi::*;

const VEEZI_API_BASE_URL: &str = "https://api.us.veezi.com/";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    let token = env::var("VEEZI_KEY").expect("VEEZI_KEY must be set");

    let client = Client::new(VEEZI_API_BASE_URL, token)?;

    let sessions_on_screen = client.get_screen(1).await?.sessions(&client).await?;
    println!("{:#?}", sessions_on_screen);

    Ok(())
}
