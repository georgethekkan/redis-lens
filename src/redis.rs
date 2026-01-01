
use color_eyre::eyre::{Context, Result};
use redis::{Client, Commands};


fn start(url: String, db: u8) -> Result<()> {
    println!("Connecting to Redis at {} using DB {}", url, db);

    let redis_url = format!("redis://{}/{}", url, db);

    let client = Client::open(redis_url).context("Failed to connect to Redis")?;

    let mut con = client.get_connection().context("Failed to get Redis connection")?;

    //let keys = con.scan::<String>("*").context("Failed to get keys from Redis")?;

    Ok(())
}



