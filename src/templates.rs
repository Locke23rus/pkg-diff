use lazy_static::lazy_static;
use minijinja::{Environment, Template};

lazy_static! {
	static ref ENV: Environment<'static> = create_env();
}

fn create_env() -> Environment<'static> {
	let mut env = Environment::new();
	env.add_template("compare.html", include_str!("../templates/compare.html"))
		.unwrap();
	env.add_template("inspect.html", include_str!("../templates/inspect.html"))
		.unwrap();
	env.add_template("layout.html", include_str!("../templates/layout.html"))
		.unwrap();
	env.add_template("root.html", include_str!("../templates/root.html"))
		.unwrap();
	env
}

pub fn get_template(name: &str) -> Template<'static> {
	ENV.get_template(name).unwrap()
}
