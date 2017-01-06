use futures::Future;
use std::io;
use std::sync::{Arc, Mutex};

use world::{Chunk, ChunkContainer, World};

pub trait SaveBackendMaster: Sync {
    fn get(&self,
           cont: *const Mutex<ChunkContainer>,
           world: &World,
           x: isize,
           z: isize)
           -> Box<Future<Item = Arc<Chunk>, Error = io::Error>>;
}

pub trait SaveBackend {
    fn save(&self, chunk: &Chunk) -> Box<Future<Item = (), Error = io::Error>>;
}