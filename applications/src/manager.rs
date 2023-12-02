use super::App;
use network_node::header::Header;
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

pub struct AppManager {
    apps_handles: Vec<JoinHandle<()>>,
    message_queues: Vec<Arc<MessageQueue>>,
    event_map: HashMap<Header, Event>,
}

impl AppManager {
    pub fn add_app<APP>(&mut self, app: APP, priority: u8)
    where
        APP: App + Send + 'static,
    {
        let message_queue = Arc::new(MessageQueue::new());
        self.message_queues.push(message_queue.clone());
        let headers = app.my_headers();
        if headers.is_some() {
            self.add_event_map(headers.unwrap(), &message_queue);
        }
        self.apps_handles
            .push(app.create_task(&message_queue, priority));
    }

    fn add_event_map(&mut self, headers: Vec<Header>, message_queue: &Arc<MessageQueue>) {
        if !headers.is_empty() {
            for header in headers {
                let flag = self.event_map.contains_key(&header);
                if !flag {
                    self.event_map
                        .insert(header, Event::new(vec![message_queue.clone()]));
                } else {
                    self.event_map
                        .get_mut(&header)
                        .unwrap()
                        .push(message_queue.clone());
                }
            }
        }
    }

    pub fn do_task(self) -> ! {
        // wait for network event
        // get header

        let (handles, _message_queues, event_map) =
            (self.apps_handles, self.message_queues, self.event_map);

        for handle in handles {
            handle.join().unwrap();
        }

        // create network handler
        loop {
            // wait for network event...

            let header = Self::get_header();
            let event = event_map.get(&header).unwrap();
            let (message_queues, condvar, mutex) =
                (&event.message_queues, &event.condvar, &event.mutex);
            {
                let guard = mutex.lock().unwrap();
                let mut guard = condvar.wait_while(guard, |flag| !*flag).unwrap();
                *guard = false;
                for message_queue in message_queues {
                    message_queue.condvar.notify_all();
                }
                *guard = true;
            }
        }
    }

    pub fn get_header() -> Header {
        todo!()
    }
}

pub struct Event {
    pub condvar: Condvar,
    pub mutex: Mutex<bool>,
    pub message_queues: Vec<Arc<MessageQueue>>,
}

impl Event {
    fn new(message_queues: Vec<Arc<MessageQueue>>) -> Self {
        Self {
            condvar: Condvar::new(),
            message_queues,
            mutex: Mutex::new(true),
        }
    }
    fn push(&mut self, message_queue: Arc<MessageQueue>) {
        self.message_queues.push(message_queue);
    }
}

pub struct MessageQueue {
    pub queue: Mutex<Vec<u8>>,
    pub condvar: Condvar,
}

impl MessageQueue {
    fn new() -> Self {
        todo!()
    }
}
