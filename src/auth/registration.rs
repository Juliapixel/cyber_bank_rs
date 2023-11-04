use std::sync::OnceLock;

use actix_web::{http::StatusCode, web::Json, HttpRequest, Responder};
use log::error;
use rand::RngCore;
use regex::Regex;
use serde::{Serialize, Deserialize};
use sqlx::PgPool;


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum InvalidPasswordError {
    InvalidLength(usize),
    NotEnoughUppercaseChars(u32),
    NotEnoughLowercaseChars(u32),
    NotEnoughDigits(u32),
    NotEnoughSpecialChars(u32),
    InvalidChar(char),
}

/// checks that the password fits a number of security constraints:
/// - length greater than 8
/// - contains at least one lowercase ASCII character
/// - contains at least one uppercase ASCII character
/// - contains at least one numeric ASCII character
/// - contains at least one "special" ASCII character, as defined by `char::is_ascii_punctuation()`
/// - does not contain any non-ASCII characters (sorry)
fn validate_password(passwd: &str) -> Result<(), InvalidPasswordError> {
    use InvalidPasswordError as E;

    if passwd.len() < 8 || passwd.len() > 64 {
        return Err(E::InvalidLength(passwd.len()));
    }

    let mut uppercase_count: u32 = 0;
    let mut lowercase_count: u32 = 0;
    let mut number_count: u32 = 0;
    let mut special_count: u32 = 0;
    let mut invalid_char: Option<char> = None;
    for i in passwd.chars() {
        if !i.is_ascii() {
            invalid_char = Some(i);
            break;
        }
        if i.is_ascii_uppercase() {
            uppercase_count += 1;
        } else if i.is_ascii_lowercase() {
            lowercase_count += 1;
        } else if i.is_ascii_digit() {
            number_count += 1;
        } else if i.is_ascii_punctuation() || i == ' ' {
            special_count += 1;
        } else {
            invalid_char = Some(i);
            break;
        }
    }

    if let Some(c) = invalid_char {
        return Err(E::InvalidChar(c));
    } else if uppercase_count == 0 {
        return Err(E::NotEnoughUppercaseChars(uppercase_count));
    } else if lowercase_count == 0 {
        return Err(E::NotEnoughLowercaseChars(lowercase_count));
    } else if number_count == 0 {
        return Err(E::NotEnoughDigits(number_count));
    } else if special_count == 0 {
        return Err(E::NotEnoughSpecialChars(special_count));
    } else {
        return Ok(());
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum InvalidUsernameError {
    InvalidLength(usize),
    InvalidChar(char),
    AlreadyInUse
}

/// checks that usernames fit a number of constraints:
/// - length is greater than 4 and smaller than 32 (might be changed later)
/// - only contains ASCII alphanumeric characters and/or the characters '.', '_' and '-'
fn validate_username(username: &str) -> Result<(), InvalidUsernameError> {
    use InvalidUsernameError as E;

    if username.len() < 4 || username.len() > 32 {
        return Err(E::InvalidLength(username.len()));
    }

    let mut invalid_char = None;
    for i in username.chars() {
        if i.is_ascii_alphanumeric() {
            continue;
        } else if ['.', '_', '-'].contains(&i) {
            continue;
        }
        invalid_char = Some(i);
    }
    if let Some(c) = invalid_char {
        return Err(E::InvalidChar(c));
    }
    return Ok(());
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum InvalidEmailError {
    AlreadyInUse,
    InvalidFormat,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RegisterError {
    InvalidPassword(InvalidPasswordError),
    InvalidUsername(InvalidUsernameError),
    InvalidEmail(InvalidEmailError),
    RegistrationError,
}

#[derive(Serialize, Deserialize)]
pub struct Registerer {
    email: String,
    username: String,
    password: String
}

static EMAIL_VALIDATION: OnceLock<Regex> = OnceLock::new();

fn get_email_validator() -> &'static Regex {
    EMAIL_VALIDATION.get_or_init(|| {
        Regex::new(r"^[\w\-]+(?:\.[\w\-])*@(?:[\w\-]\.)*[\w\-]+\.[a-z]{2,6}(?:\.[a-z]{2})?$").unwrap()
    })
}

/// creates a user account, verifying validity of given username, email and
/// password.
// TODO: make sure usernames are case-insensitive and can only be lowercase
pub async fn register(req: HttpRequest, userinfo: Json<Registerer>) -> impl Responder {
    let mut errors: Vec<RegisterError> = Vec::new();

    match validate_password(&userinfo.password) {
        Ok(_) => (),
        Err(e) => {
            errors.push(RegisterError::InvalidPassword(e));
        }
    };

    match validate_username(&userinfo.username) {
        Ok(_) => (),
        Err(e) => {
            errors.push(RegisterError::InvalidUsername(e));
        },
    };

    if !get_email_validator().is_match(&userinfo.email) {
        errors.push(RegisterError::InvalidEmail(InvalidEmailError::InvalidFormat));
    }

    let pool = req.app_data::<PgPool>().unwrap();

    match sqlx::query!(r"SELECT COUNT(username) FROM users
        WHERE username = $1
        ",
        userinfo.username
    ).fetch_one(pool).await {
        Ok(o) => {
            if let Some(count) = o.count {
                if count != 0 {
                    errors.push(RegisterError::InvalidUsername(InvalidUsernameError::AlreadyInUse));
                }
            }
        },
        Err(e) => panic!("{e}"),
    };

    match sqlx::query!(r"SELECT COUNT(email) FROM users
        WHERE email = $1",
        userinfo.email
    ).fetch_one(pool).await {
        Ok(o) => {
            if let Some(count) = o.count {
                if count != 0 {
                    errors.push(RegisterError::InvalidEmail(InvalidEmailError::AlreadyInUse));
                }
            }
        },
        Err(e) => panic!("{e}"),
    };

    if !errors.is_empty() {
        return Json(errors)
            .customize()
            .with_status(StatusCode::BAD_REQUEST)
            .respond_to(&req);
    }

    let mut salt = [0u8; 64];
    rand::thread_rng().fill_bytes(&mut salt);
    let hashed_salted_passwd: Vec<u8> = super::salt_and_hash(userinfo.password.clone(), &salt);

    let insert = sqlx::query!(r"INSERT INTO users
        (user_id, email, username, password, salt, creation_date)
        VALUES (
        gen_random_uuid(),
        $1,
        $2,
        $3,
        $4,
        $5
        );",
        userinfo.email,
        userinfo.username,
        hashed_salted_passwd,
        &salt,
        chrono::Utc::now()
    ).execute(pool).await;
    match insert {
        Ok(o) => {
            if o.rows_affected() == 0 {
                return Json(RegisterError::RegistrationError)
                    .customize()
                    .with_status(StatusCode::BAD_REQUEST)
                    .respond_to(&req);
            } else {
                return Json(()).customize().with_status(StatusCode::CREATED).respond_to(&req);
            }
        },
        Err(e) => {
            error!("failed inserting user into table: {e}");
            return Json(())
                .customize()
                .with_status(StatusCode::BAD_REQUEST)
                .respond_to(&req);
        }
    };
}
