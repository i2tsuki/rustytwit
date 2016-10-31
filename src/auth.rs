extern crate egg_mode;

use std::io;

// AuthError
#[derive(Debug)]
pub enum AuthError {
    Io(io::Error),
    EggMode(egg_mode::error::Error),
}

impl From<io::Error> for AuthError {
    fn from(err: io::Error) -> AuthError {
        AuthError::Io(err)
    }
}

impl From<egg_mode::error::Error> for AuthError {
    fn from(err: egg_mode::error::Error) -> AuthError {
        AuthError::EggMode(err)
    }
}

pub fn authorize(consumer_token: egg_mode::Token) -> Result<egg_mode::Token<'static>, AuthError> {
    let request_token = try!(egg_mode::request_token(&consumer_token, "rustytwit"));
    let url = egg_mode::authorize_url(&request_token);

    println!("access the following url: {}", url);
    println!("PIN: ");

    let mut input: String = String::new();
    try!(io::stdin().read_line(&mut input));
    let pin = input.trim().to_string();

    // There are access_token, user_id, username receiving here
    let (access_token, _, _) = try!(egg_mode::access_token(&consumer_token, &request_token, pin));
    return Ok(access_token)
}
