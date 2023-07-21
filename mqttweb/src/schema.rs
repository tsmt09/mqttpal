// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
        email -> Text,
        password -> Text,
        remember -> Bool,
        role_id -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
