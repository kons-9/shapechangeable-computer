use anyhow::Result;
use network_node::header::Header;

/// when rx receive a message, it will be sent to task manager(using interrupt)
pub struct TaskManager {
    tasks: Vec<Task>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }
    pub fn interrupt(&mut self, header: Header, message: Vec<u8>) {
        self.tasks
            .iter_mut()
            .find(|task| task.app.get_event().iter().any(|h| h == &header))
            .map(|task| task.app.process_messages(message));
    }
}

pub struct Task {
    app: Box<dyn TaskApp>,
}

trait TaskApp {
    fn process_messages(&mut self, messages: Vec<u8>) -> Result<()>;
    fn get_event(&self) -> Vec<Header>;
}

pub struct SystemTask();

impl TaskApp for SystemTask {
    fn process_messages(&mut self, messages: Vec<u8>) -> Result<()> {
        assert_eq!(messages.len(), 0);
        unimplemented!()
    }
    fn get_event(&self) -> Vec<Header> {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_task_test() {
        let mut task_manager = TaskManager::new();
        let task = Task {
            app: Box::new(SystemTask()),
        };
        task_manager.add_task(task);

        let header = Header::Data;
        task_manager.interrupt(header, vec![]);
    }
}
