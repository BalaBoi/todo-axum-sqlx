use askama::Template;


#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate;



#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    errors: Option<String>,
}

impl LoginTemplate {
    pub fn new(errors: Option<String>) -> Self {
        Self { errors }
    }
}
