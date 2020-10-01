use diesel::prelude::*;
use uuid::Uuid;

use crate::auth;
use crate::user::model;

/// Run query using Diesel to find user by uid and return it.
pub fn find_user_by_uid(
    uid: Uuid,
    conn: &PgConnection,
) -> Result<Option<model::User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let user = users
        .filter(id.eq(uid.to_string()))
        .first::<model::User>(conn)
        .optional()?;

    Ok(user)
}

/// Run query using Diesel to insert a new database row and return the result.
pub fn insert_new_user(
    // prevent collision with `name` column imported inside the function
    data: model::NewUser,
    conn: &PgConnection,
) -> Result<model::User, diesel::result::Error> {
    // It is common when using Diesel with Actix web to import schema-related
    // modules inside a function's scope (rather than the normal module's scope)
    // to prevent import collisions and namespace pollution.
    use crate::schema::users::dsl::*;

    let new_user = model::User {
        id: Uuid::new_v4().to_string(),
        name: data.name,
        email: data.email,
        password: auth::hash(data.password.as_bytes()),
    };

    diesel::insert_into(users).values(&new_user).execute(conn)?;

    Ok(new_user)
}
