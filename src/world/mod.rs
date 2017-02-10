mod backend;

mod detail {
    use futures::{Async, Future, Poll};
    use futures::task;
    use futures::task::Task;
    use std::collections::{BTreeMap, VecDeque};
    use std::io;
    use std::mem::swap;
    use std::ops::DerefMut;
    use std::sync::{Arc, Mutex, Weak};
    use tokio_core::reactor::Handle;
    use world::backend::save::{SaveBackend, SaveBackendMaster};
    enum ChunkState {
        Loading(Box<Future<Item = Arc<Chunk>, Error = io::Error>>),
        Loaded(Weak<Chunk>),
        Unloading,
        Unloaded,
    }
    struct ChunkContainer {
        pub state: ChunkState,
        pub queue: VecDeque<Task>,
    }

    struct ChunkFuture {
        world: Arc<World>,
        x: isize,
        z: isize,
    }

    impl Future for ChunkFuture {
        type Item = Arc<Chunk>;
        type Error = io::Error;
        fn poll(&mut self) -> Poll<Arc<Chunk>, io::Error> {
            World::poll_get_chunk(self.world.clone(), self.x, self.z)
        }
    }

    // TODO: a better mutex
    pub struct World {
        chunks: Mutex<BTreeMap<(isize, isize), Mutex<ChunkContainer>>>,
        save: Box<SaveBackendMaster>,
        handle: Handle,
    }

    impl World {
        fn get_chunk(self_: Arc<World>, x: isize, z: isize) -> ChunkFuture {
            ChunkFuture {
                world: self_,
                x: x,
                z: z,
            }
        }
        // HACK: Arc as self
        fn poll_get_chunk(self_: Arc<World>, x: isize, z: isize) -> Poll<Arc<Chunk>, io::Error> {
            let mut chunks = self_.chunks.lock().unwrap();
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
                    container.state = ChunkState::Loading(self_.save
                        .load(Chunk {
                                  owner: Arc::downgrade(&self_),
                                  parent: mtx as *const Mutex<ChunkContainer>,
                                  backend: None,
                                  handle: self_.handle.clone(),
                              },
                              x,
                              z));
                    if let ChunkState::Loading(ref mut future) = container.state {
                        future.poll()
                    } else {
                        unreachable!()
                    }
                }
            }
        }
    }

    pub struct Chunk {
        owner: Weak<World>,
        parent: *const Mutex<ChunkContainer>,
        backend: Option<Box<SaveBackend>>,
        handle: Handle,
    }

    #[cfg_attr(feature = "cargo-clippy", allow(if_let_redundant_pattern_matching))]
    impl Drop for Chunk {
        fn drop(&mut self) {
            if let Some(_) = self.owner.upgrade() {
                unsafe {
                    let parent = self.parent;
                    let mut container = (*parent).lock().unwrap();
                    container.state = ChunkState::Unloading;
                    let mut backend = None;
                    swap(&mut backend, &mut self.backend);
                    if let Some(backend) = backend {
                        // FIXME: rust-lang/rfcs#811
                        let fut = backend.save(self)
                            .and_then(move |_| {
                                (*parent).lock().unwrap().state = ChunkState::Unloaded;
                                Ok(())
                            })
                            .or_else(|err| {
                                // TODO: error handling
                                Ok(())
                            });
                        self.handle.spawn(fut);
                    }
                }
            }
        }
    }

}

use self::detail::Chunk;
pub use self::detail::World;