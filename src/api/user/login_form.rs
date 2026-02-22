use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
    #[serde(default, deserialize_with = "parse_checkbox")]
    pub stay_signed_in: bool,
}

fn parse_checkbox<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum FormValue {
        String(String),
        Vec(Vec<String>),
        Bool(bool),
    }

    let value = Option::<FormValue>::deserialize(deserializer)?;
    match value {
        Some(FormValue::String(s)) => Ok(s.is_truthy()),
        Some(FormValue::Vec(v)) => Ok(v.first().is_some_and(|s| s.is_truthy())),
        Some(FormValue::Bool(b)) => Ok(b),
        _ => Ok(false),
    }
}

trait StringBoolExt {
    fn is_truthy(&self) -> bool;
}

impl StringBoolExt for String {
    fn is_truthy(&self) -> bool {
        *self == "on" || *self == "true"
    }
}
