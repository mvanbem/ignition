use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + Send + 'static) -> Self {
        Self {
            future: Box::pin(future),
        }
    }

    pub fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
