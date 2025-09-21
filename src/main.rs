mod web;
mod worker;

use anyhow::Result;
use std::env;

fn main() -> Result<()> {
	let mut args = env::args().skip(1);
	let application_id = args.next();
	if let Some(application_id) = application_id {
		let interaction_token = args.next().expect("interaction token is required");
		worker::main(&application_id, &interaction_token)
	} else {
		web::main()
	}
}
