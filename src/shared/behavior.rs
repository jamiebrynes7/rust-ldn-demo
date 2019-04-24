use spatialos_sdk::worker::view::{View};
use spatialos_sdk::worker::connection::{Connection, WorkerConnection};

pub trait Behavior {
    fn tick(&mut self, view: &View, connection: &mut WorkerConnection);
}


pub struct BehaviorManager {
    behaviours: Vec<Box<dyn Behavior>>,
    view: View
}

impl BehaviorManager {
    pub fn new() -> Self {
        BehaviorManager {
            behaviours: Vec::new(),
            view: View::new()
        }
    }

    pub fn register_behaviour<T: Behavior + 'static>(&mut self, behaviour: T) {
        self.behaviours.push(Box::new(behaviour));
    }

    pub fn tick(&mut self, connection: &mut WorkerConnection) {
        let ops = connection.get_op_list(0);
        self.view.process_ops(&ops);

        for behaviour in &mut self.behaviours {
            behaviour.tick(&self.view, connection);
        }
    }
}