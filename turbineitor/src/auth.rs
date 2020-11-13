use maratona_animeitor_rust::auth::UserKey;

//use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use crate::errors::Error;

pub fn sign_user_key(user_key : UserKey, secret : &str) -> Result<String, Error> {
    Ok(encode(
        &Header::default(), 
        &user_key, 
        &EncodingKey::from_secret(secret.as_ref()))?)
}

pub fn verify_user_key(token : &String, secret : &str) -> Result<UserKey, Error> {
    let user_key = decode::<UserKey>(token, 
        &DecodingKey::from_secret(secret.as_ref()), 
        &Validation::default())?;
    Ok(user_key.claims)
}
