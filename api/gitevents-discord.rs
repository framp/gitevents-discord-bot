mod _discord;
mod _error;

use _discord::{handle_commands, validate_headers};
use std::env;
use vercel_lambda::{error::VercelError, lambda, IntoResponse, Request};

fn handler(req: Request) -> Result<impl IntoResponse, _error::Error> {
    dotenv::dotenv().ok();
    let public_key = env::var("DISCORD_PUBLIC_KEY").expect("Missing DISCORD_PUBLIC_KEY");
    let public_key = env::var("DISCORD_PUBLIC_KEY").expect("Missing DISCORD_PUBLIC_KEY");
    println!("{}", std::str::from_utf8(req.body()).unwrap());
    Ok(match validate_headers(&req, &public_key) {
        Ok(_) => match handle_commands(&req) {
            Ok(res) => {
                println!("{:?}", res);
                res.into_response()
            }
            Err(err) => {
                println!("{}", err.to_string());
                err.into_response()
            }
        },
        Err(err) => {
            println!("{}", err.to_string());
            err.into_response()
        }
    })
}

// Start the runtime with the handler
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(lambda!(handler))
}
