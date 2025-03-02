use askama::Template;

#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate;

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    errors: String,
}

impl LoginTemplate {
    pub fn new(errors: String) -> Self {
        Self { errors }
    }
}
