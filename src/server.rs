use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::io;
use std::convert::From;
use std::collections::hash_map::HashMap;
use toml::{Parser, Table, Value};
use futures::future::{Future, join_all, ok};
use tokio_core::reactor::Core;

pub struct ServerList {
    core: Core,
    config_path: String,
    servers: HashMap<String, Server>,
}

#[derive(Debug)]
pub struct ConfigError {
    desc: String,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.desc)
    }
}

impl Error for ConfigError {
    fn description(&self) -> &str {
        &self.desc
    }
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        ConfigError { desc: err.description().to_string() }
    }
}

trait TomlValueConvert {
    fn as_table_move(self) -> Option<Table>;
}

impl TomlValueConvert for Value {
    fn as_table_move(self) -> Option<Table> {
        match self {
            Value::Table(t) => Some(t),
            _ => None,
        }
    }
}

impl ServerList {
    pub fn new(config_path: String) -> Result<ServerList, ConfigError> {
        let mut config_text = String::default();
        File::open(&config_path)?.read_to_string(&mut config_text)?;
        let mut parser = Parser::new(&config_text);
        let mut config = parser.parse()
            .ok_or(ConfigError { desc: format!("Config parse error: {}", parser.errors[0]) })?;
        let list = ServerList {
            core: Core::new()?,
            config_path: config_path,
            servers: config
            .remove("server")
            .ok_or(ConfigError { desc: "No server section".to_string() })?
            .as_table_move()
            .ok_or(ConfigError { desc: "Server section must be table".to_string() })?
            // TODO: drain instead of clone
            .iter()
            .map(|(name, serv)| Ok((name.clone(), Server::new(serv.clone())?)))
            .collect::<Result<HashMap<String, Server>, ConfigError>>()?,
        };
        Ok(list)
    }

    pub fn get(&self) -> &HashMap<String, Server> {
        &self.servers
    }
    pub fn get_mut(&mut self) -> &mut HashMap<String, Server> {
        &mut self.servers
    }

    pub fn run(&mut self) -> Result<(), ()> {
        // STUB
        let startup = join_all(vec![ok::<(), ()>(())]).map(|_| ());
        let server_stop = ok::<(), ()>(());
        let shutdown = join_all(vec![ok::<(), ()>(())]).map(|_| ());
        self.core
            .run(startup.select(server_stop).map(|(item, _)| item).map_err(|(error, _)| error))?;
        self.core.run(shutdown)?;
        Ok(())
    }
}

pub struct Server {
}

impl Server {
    fn new(mut config: Value) -> Result<Server, ConfigError> {
        config.as_table_move()
            .ok_or(ConfigError { desc: "Server config must be table".to_string() })?;
        Ok(Server {})
    }
}
