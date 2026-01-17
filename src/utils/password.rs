use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString},
};
// use rand::{Rng, rngs::OsRng};
use rand::rngs::OsRng;

use crate::utils::response::ApiError;

// const CONSONANTS_CANDIDATES: [&str; 41] = [
//     "B", "C", "D", "F", "G", "H", "J", "K", "L", "M", "N", "P", "Q", "R", "S", "T", "V", "W", "X",
//     "Y", "Z", "b", "c", "d", "f", "g", "h", "j", "k", "m", "n", "p", "q", "r", "s", "t", "v", "w",
//     "x", "y", "z",
// ];
// const VOWELS_CANDIDATES: [&str; 8] = ["A", "E", "U", "a", "e", "i", "o", "u"];

pub fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

pub fn verify_password(password: String, hash: String) -> Result<bool, ApiError> {
    let parsed_hash = PasswordHash::new(&hash)?;
    let argon2 = Argon2::default();

    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

// pub fn get_random_password() -> String {
//     let mut rng = rand::thread_rng();
//     let l1 = rng.gen_range(0..41_usize);
//     let l2 = rng.gen_range(0..8_usize);
//     let l3 = rng.gen_range(0..41_usize);
//     let l4 = rng.gen_range(0..8_usize);
//     format!(
//         "{}{}{}{}-{}",
//         CONSONANTS_CANDIDATES[l1],
//         VOWELS_CANDIDATES[l2],
//         CONSONANTS_CANDIDATES[l3],
//         VOWELS_CANDIDATES[l4],
//         rng.gen_range(100..=999)
//     )
// }
