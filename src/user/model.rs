use serde::{Deserialize, Serialize};

use crate::schema::users;

#[derive(Debug, Clone, Serialize, Queryable, Insertable)]
pub struct User {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub name: String,
    pub password: String,
    pub email: String,
}
