#[macro_use]
extern crate clap;
extern crate limonite;

use limonite::server::ServerList;

fn main() {
    let file_exists = |path| {
        if std::fs::metadata(path).is_ok() {
            Ok(())
        } else {
            Err(String::from("File doesn't exist"))
        }
    };

    let matches = clap_app!(limonite =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: "Minecraft and reverse proxy server")
        (@arg config: -c --config [conf] default_value("/etc/limonite.conf")
        {file_exists} "Sets a custom config file")
    )
        .get_matches();

    let mut list = ServerList::new(matches.value_of("config").unwrap().to_string()).unwrap();

    list.run().unwrap();
}
