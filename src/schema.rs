// @generated automatically by Diesel CLI.

diesel::table! {
    mqtt_clients (id) {
        id -> Integer,
        name -> Text,
        url -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
        password -> Text,
        email -> Nullable<Text>,
        role_id -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(mqtt_clients, users,);
