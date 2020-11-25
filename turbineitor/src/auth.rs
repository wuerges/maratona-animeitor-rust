use data::auth::UserKey;
use crate::Params;
use crate::errors::Error;

//use serde::{Serialize, Deserialize};
// use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

pub fn sign_user_key(user_key : UserKey, secret : &str) -> Result<String, Error> {
    Ok(encode(
        &Header::default(), 
        &user_key, 
        &EncodingKey::from_secret(secret.as_ref()))?)
}

pub fn verify_user_key(token : &String, params: &Params) -> Result<UserKey, Error> {

    let validation = Validation { validate_exp: false, ..Default::default()};
    let user_key = decode::<UserKey>(token, 
        &DecodingKey::from_secret(params.secret.as_ref()),        
        &validation)?;

    let claims = user_key.claims;

    if params.contest_number != claims.contest_number {
        return Err(Error::WrongContestNumber(params.contest_number, claims.contest_number));
    }
    if params.site_number != claims.site_number {
        return Err(Error::WrongSiteNumber(params.site_number, claims.site_number));
    }
    
    Ok(claims)
}
