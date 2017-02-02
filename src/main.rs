#[macro_use]
extern crate clap;
extern crate limonite;

use limonite::server::ServerList;

fn main() {
    let file_exists = |path| if std::fs::metadata(path).is_ok() {
        Ok(())
    } else {
        Err(String::from("File doesn't exist"))
    };

    // HACK: trigger rebuild on version change
    include_str!("../Cargo.toml");

    let matches = clap_app!(
        @app (app_from_crate!())
        (@arg config: -c --config [conf] default_value("/etc/limonite.conf")
        {file_exists} "Sets a custom config file")
    )
        .get_matches();

    let mut list = ServerList::new(matches.value_of("config").unwrap().to_string()).unwrap();

    list.run().unwrap();
}
