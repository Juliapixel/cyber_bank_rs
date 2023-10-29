pub mod login;
pub mod registration;

#[allow(dead_code)]
struct DbUser {
    id: i64,
    user_id: uuid::Uuid,
    email: String,
    username: String,
    password: Vec<u8>,
    salt: Vec<u8>
}

// argon2 256-bit hash with 2 iterations, 1 parallelism and 32MB of memory used
pub fn salt_and_hash(passwd: String, salt: &[u8]) -> Vec<u8> {
    let hashed_salted_passwd = argon2::hash_raw(
        passwd.as_bytes(),
        salt,
        &argon2::Config{
            ad: &[],
            hash_length: 32,
            lanes: 1,
            mem_cost: 32 * 1024,
            secret: &[],
            time_cost: 2,
            variant: argon2::Variant::Argon2id,
            version: argon2::Version::Version13,
        }
    ).unwrap();
    return hashed_salted_passwd;
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.route("/register", actix_web::web::post().to(registration::register))
        .route("/login", actix_web::web::get().to(login::login));
}
