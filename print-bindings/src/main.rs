use clap::{crate_name, crate_version, App, Arg};
use env_logger::{Builder, Env};

use zoinks_lib::schemafile2print;
// use zoinks_codegen::schema2struct;

fn main() {
    Builder::from_env(Env::default().default_filter_or("info"))
        .init();

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let in_file = matches.value_of("input").unwrap();

    println!("{}", schemafile2print(in_file));
}
