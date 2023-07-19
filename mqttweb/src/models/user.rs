use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct UserForm {
    email: String,
    password: String,
    remember: Option<String>,
}
