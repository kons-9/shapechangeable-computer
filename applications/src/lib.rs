use crate::manager::MessageQueue;
use anyhow::Result;
use network_node::header::Header;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

pub mod manager;
pub mod task;

// App is Sized
pub trait App {
    // interface of network
    // type Output;
    fn process_messages(&mut self, messages: Vec<u8>) -> Result<()>;

    /// default do nothing
    fn set_priority(&mut self, priority: u8) {
        eprintln!("priority: {}, but not implemented", priority);
        ()
    }

    /// default error handler is just ignore it
    fn error_handler(&mut self, error: anyhow::Error) -> Result<()> {
        eprintln!("error: {}", error);
        Ok(())
    }

    /// if this app is related to a specific header, return it
    fn my_headers(&self) -> Option<Vec<Header>> {
        None
    }

    /// default app wake up only when message is received and host notifys it
    fn create_task(mut self, message: &Arc<MessageQueue>, priority: u8) -> JoinHandle<()>
    where
        Self: Sized + Send + 'static,
    {
        let message = Arc::clone(message);
        self.set_priority(priority);
        thread::spawn(move || loop {
            let old_message = {
                // swap message and release lock
                let message_queue = message.queue.lock().unwrap();
                let mut message = message
                    .condvar
                    .wait_while(message_queue, |q| q.is_empty())
                    .unwrap();
                let mut old_message = Vec::new();
                std::mem::swap(&mut *message, &mut old_message);
                old_message
            };

            self.process_messages(old_message)
                .map_err(|e| self.error_handler(e))
                .unwrap();
        })
    }
}

pub struct Output();

pub mod image {
    use anyhow::Result;

    pub struct GetImageApp {}
    impl GetImageApp {
        pub fn new() -> Self {
            Self {}
        }
    }
    impl super::App for GetImageApp {
        // type Output = (Vec<u8>, u32, Option<Point>);

        fn process_messages(&mut self, messages: Vec<u8>) -> Result<()> {
            println!("get image: {:?}", messages);
            unimplemented!()
        }
    }
}
