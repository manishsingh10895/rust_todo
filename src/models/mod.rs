pub mod todo_model;
pub mod user_model;

use diesel::{r2d2::ConnectionManager, PgConnection};
use serde::{Deserialize, Serialize};

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
