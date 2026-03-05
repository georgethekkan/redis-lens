//#![allow(unused)]
use color_eyre::Result;

use color_eyre::eyre::Ok;
use redis_lens::redis::{LensClient, MockClient};
use redis_lens::{args, run, tracing};

fn main() -> Result<()> {
    let _guard = tracing::setup();

    color_eyre::install()?;

    let args = args::parse();

    if args.config.mock {
        let mock = MockClient::default();
        mock.setup_keys()?;
        run(&args, mock)?;
    } else {
        let client = LensClient::new(&args.config)?;
        run(&args, client)?;
    }

    Ok(())
}
