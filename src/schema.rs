// @generated automatically by Diesel CLI.

diesel::table! {
    todos (id) {
        id -> Uuid,
        title -> Varchar,
        completed -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        user_id -> Uuid,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        email -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        password -> Varchar,
        name -> Varchar,
    }
}

diesel::joinable!(todos -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(todos, users,);
