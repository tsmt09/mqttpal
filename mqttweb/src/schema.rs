// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
        password -> Text,
        email -> Nullable<Text>,
    }
}
