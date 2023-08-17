// @generated automatically by Diesel CLI.

diesel::table! {
    roles (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    user_role (user_id, role_id) {
        user_id -> Integer,
        role_id -> Integer,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
        password -> Text,
        email -> Nullable<Text>,
    }
}

diesel::joinable!(user_role -> roles (role_id));
diesel::joinable!(user_role -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(roles, user_role, users,);
