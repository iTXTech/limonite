use futures::Future;
use std::sync::Mutex;
use tokio_core::reactor::Handle;

use world::{ChunkContainer, ChunkState};
use world::backend::SaveBackend;

pub struct Chunk {
    parent: *const Mutex<ChunkContainer>,
    backend: Box<SaveBackend>,
    handle: Handle,
}

impl Drop for Chunk {
    fn drop(&mut self) {
        unsafe {
            let parent = self.parent;
            let mut container = (*parent).lock().unwrap();
            container.state = ChunkState::Unloading;
            self.handle.spawn(self.backend
                .save(self)
                .and_then(move |_| {
                    (*parent).lock().unwrap().state = ChunkState::Unloaded;
                    Ok(())
                })
                .or_else(|err| {
                    // TODO: error handling
                    Ok(())
                }));
        }
    }
}
