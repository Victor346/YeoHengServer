use argon2::{self, Config};

pub fn salt_password(password: String) -> String {
    let pass = password.as_bytes();
    let salt = std::env::var("SALT_SECRET")
        .expect("Error retrieving salt");
    let salt = salt.as_bytes();

    let config  = Config::default();
    argon2::hash_encoded(pass, salt, &config).unwrap()
}