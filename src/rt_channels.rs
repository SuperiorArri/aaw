use std::sync::{Arc, Mutex};

pub type RtConsumer<T> = rtrb::Consumer<T>;
pub type RtProducer<T> = rtrb::Producer<T>;

pub struct RtSharedProducer<T> {
    inner: Arc<Mutex<rtrb::Producer<T>>>,
}

impl<T> Clone for RtSharedProducer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> RtSharedProducer<T> {
    pub fn push(&self, value: T) -> Result<(), T> {
        let mut producer = self.inner.lock().expect("rt producer poisoned");
        producer.push(value).map_err(|err| match err {
            rtrb::PushError::Full(value) => value,
        })
    }
}

pub fn create_mpsc<T>(capacity: usize) -> (RtSharedProducer<T>, RtConsumer<T>) {
    let (producer, consumer) = rtrb::RingBuffer::new(capacity);
    let shared_producer = RtSharedProducer {
        inner: Arc::new(Mutex::new(producer)),
    };
    (shared_producer, consumer)
}

pub fn create_spsc<T>(capacity: usize) -> (RtProducer<T>, RtConsumer<T>) {
    rtrb::RingBuffer::new(capacity)
}
