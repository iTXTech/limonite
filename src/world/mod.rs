use futures::{Async, Future, Poll};
use futures::task;
use futures::task::Task;
use std::collections::{BTreeMap, VecDeque};
use std::io;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex, Weak};

pub use world::chunk::Chunk;

mod chunk;
mod backend;
use self::backend::save::SaveBackendMaster;

// These are internal details that is only used in constructing Chunk.
pub enum ChunkState {
    Loading(Box<Future<Item = Arc<Chunk>, Error = io::Error>>),
    Loaded(Weak<Chunk>),
    Unloading,
    Unloaded,
}
pub struct ChunkContainer {
    pub state: ChunkState,
    pub queue: VecDeque<Task>,
}

pub struct ChunkFuture<'a> {
    world: &'a World,
    x: isize,
    z: isize,
}

impl<'a> Future for ChunkFuture<'a> {
    type Item = Arc<Chunk>;
    type Error = io::Error;
    fn poll(&mut self) -> Poll<Arc<Chunk>, io::Error> {
        self.world.poll_get(self.x, self.z)
    }
}

// TODO: a better mutex
pub struct World {
    chunks: Mutex<BTreeMap<(isize, isize), Mutex<ChunkContainer>>>,
    save: Box<SaveBackendMaster>,
}

impl World {
    pub fn get(&self, x: isize, z: isize) -> ChunkFuture {
        ChunkFuture {
            world: self,
            x: x,
            z: z,
        }
    }
    pub fn poll_get(&self, x: isize, z: isize) -> Poll<Arc<Chunk>, io::Error> {
        let mut chunks = self.chunks.lock().unwrap();
        let mtx = chunks.entry((x, z)).or_insert_with(|| {
            Mutex::new(ChunkContainer {
                state: ChunkState::Unloaded,
                queue: VecDeque::new(),
            })
        });
        let mut guard = mtx.lock().unwrap();
        let container = guard.deref_mut();
        match container.state {
            ChunkState::Loaded(ref weak) => {
                match weak.upgrade() {
                    Some(s) => {
                        container.queue.drain(..).map(|task| task.unpark()).count();
                        Ok(Async::Ready(s))
                    }
                    None => panic!("Chunk weak reference should not be expired"),
                }
            }
            ChunkState::Loading(_) |
            ChunkState::Unloading => {
                container.queue.push_back(task::park());
                Ok(Async::NotReady)
            }
            ChunkState::Unloaded => {
                // TODO: generator
                container.state = ChunkState::Loading(self.save
                    .get(mtx as *const Mutex<ChunkContainer>, self, x, z));
                if let ChunkState::Loading(ref mut future) = container.state {
                    future.poll()
                } else {
                    unreachable!()
                }
            }
        }
    }
}