use lazy_static::lazy_static;
use minijinja::{Environment, Source, Template};

lazy_static! {
	static ref ENV: Environment<'static> = create_env();
}

fn create_env() -> Environment<'static> {
	let mut env = Environment::new();
	let mut source = Source::new();
	source.load_from_path("templates", &["html"]).unwrap();
	env.set_source(source);
	env
}

pub fn get_template(name: &str) -> Template<'static> {
	ENV.get_template(name).unwrap()
}
