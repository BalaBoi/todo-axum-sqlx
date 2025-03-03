use askama::Template;

use super::db::Task;

#[derive(Template)]
#[template(path = "new_todo.html")]
pub struct NewTodoTemplate;

#[derive(Template)]
#[template(path = "todos.html")]
pub struct TodosTemplate<'a> {
    pub username: &'a str,
    pub todos: Vec<Task>,
}

#[derive(Template)]
#[template(path = "edit_todo.html")]
pub struct EditTodoTemplate {
    pub todo: Task,
}
