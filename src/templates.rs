use minijinja::{Environment, Template};
use std::sync::OnceLock;

static ENV: OnceLock<Environment<'static>> = OnceLock::new();

fn create_env() -> Environment<'static> {
	let mut env = Environment::new();
	env.add_template("compare.html", include_str!("../templates/compare.html"))
		.unwrap();
	env.add_template("inspect.html", include_str!("../templates/inspect.html"))
		.unwrap();
	env.add_template("layout.html", include_str!("../templates/layout.html"))
		.unwrap();
	env.add_template("index.html", include_str!("../templates/index.html"))
		.unwrap();
	env.add_template("error.html", include_str!("../templates/error.html"))
		.unwrap();
	env
}

pub fn get_template(name: &str) -> Template<'static> {
	ENV.get_or_init(create_env).get_template(name).unwrap()
}
