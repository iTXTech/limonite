use futures::Future;
use std::io;
use std::sync::{Arc, Mutex, Weak};

use world::{Chunk, World};

pub trait SaveBackendMaster: Sync {
    fn load(&self,
            chunk: Chunk,
            x: isize,
            z: isize)
            -> Box<Future<Item = Arc<Chunk>, Error = io::Error>>;
}

pub trait SaveBackend {
    fn save(&self, chunk: &mut Chunk) -> Box<Future<Item = (), Error = io::Error>>;
}