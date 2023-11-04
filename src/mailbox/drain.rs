use std::ops::{Deref, DerefMut};
use std::time::Duration;

use tokio::sync::mpsc;

pub struct DrainedReceiver<T>(pub(super) mpsc::UnboundedReceiver<T>);

impl<T> DrainedReceiver<T> {
    pub async fn drain(mut self, timeout: Duration) -> Box<[T]> {
        let sleeper = async { std::thread::sleep(timeout) };
        let mut buffer = Vec::with_capacity(8);

        tokio::pin!(sleeper);

        loop {
            tokio::select! {
            _ = &mut sleeper => break,
            m = self.recv() => match m {
                Some(mesg) => buffer.push(mesg),
                None       => { break; },
            }
            }
        }

        buffer.into_boxed_slice()
    }
}

impl<T> Deref for DrainedReceiver<T> {
    type Target = mpsc::UnboundedReceiver<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for DrainedReceiver<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
