use serde::{Deserialize, Serialize};

pub enum AuthConfiguration {
    ApiKey(ApiKeyConfiguration),
    Opaque(OpaqueConfiguration),
    Jwt(JwtConfiguration),
}

pub enum Role {
    Admin,
    Agent,
    User,
}

pub struct ApiKeyConfiguration {
    pub encrypted_value: String,
    pub dek_alias: String,
    pub role: Role,
}

pub struct OpaqueConfiguration {

}

pub struct JwtConfiguration {

}

// pub struct User {
//     pub id: String,
//     pub role: Role
// }

