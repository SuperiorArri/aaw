use std::sync::Arc;
use tokio::sync::Mutex;

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
    pub async fn push(&self, value: T) -> Result<(), T> {
        let mut producer = self.inner.lock().await;
        producer.push(value).map_err(|err| match err {
            rtrb::PushError::Full(value) => value,
        })
    }

    pub fn blocking_push(&self, value: T) -> Result<(), T> {
        let mut producer = self.inner.blocking_lock();
        producer.push(value).map_err(|err| match err {
            rtrb::PushError::Full(value) => value,
        })
    }

    pub fn inner(&self) -> &Arc<Mutex<rtrb::Producer<T>>> {
        &self.inner
    }
}

pub struct RtSharedConsumer<T> {
    inner: Arc<Mutex<rtrb::Consumer<T>>>,
}

impl<T> Clone for RtSharedConsumer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> RtSharedConsumer<T> {
    pub async fn pop(&self) -> Option<T> {
        let mut consumer = self.inner.lock().await;
        consumer.pop().ok()
    }

    pub fn blocking_pop(&self) -> Option<T> {
        let mut consumer = self.inner.blocking_lock();
        consumer.pop().ok()
    }

    pub fn inner(&self) -> &Arc<Mutex<rtrb::Consumer<T>>> {
        &self.inner
    }
}

pub fn create_mpmc<T>(capacity: usize) -> (RtSharedProducer<T>, RtSharedConsumer<T>) {
    let (producer, consumer) = rtrb::RingBuffer::new(capacity);
    let shared_producer = RtSharedProducer {
        inner: Arc::new(Mutex::new(producer)),
    };
    let shared_consumer = RtSharedConsumer {
        inner: Arc::new(Mutex::new(consumer)),
    };
    (shared_producer, shared_consumer)
}

pub fn create_mpsc<T>(capacity: usize) -> (RtSharedProducer<T>, RtConsumer<T>) {
    let (producer, consumer) = rtrb::RingBuffer::new(capacity);
    let shared_producer = RtSharedProducer {
        inner: Arc::new(Mutex::new(producer)),
    };
    (shared_producer, consumer)
}

pub fn create_spmc<T>(capacity: usize) -> (RtProducer<T>, RtSharedConsumer<T>) {
    let (producer, consumer) = rtrb::RingBuffer::new(capacity);
    let shared_consumer = RtSharedConsumer {
        inner: Arc::new(Mutex::new(consumer)),
    };
    (producer, shared_consumer)
}

pub fn create_spsc<T>(capacity: usize) -> (RtProducer<T>, RtConsumer<T>) {
    rtrb::RingBuffer::new(capacity)
}
