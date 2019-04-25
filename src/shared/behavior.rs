use spatialos_sdk::worker::view::{View};
use spatialos_sdk::worker::connection::{Connection, WorkerConnection};
use spatialos_sdk::worker::op::WorkerOp;

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
        let mut in_critical_section = false;
        self.view.clear_transient_data();

        loop {
            let ops = connection.get_op_list(0);
            self.view.process_ops(&ops);

            for op in ops.iter() {
                match op {
                    WorkerOp::CriticalSection(_) => in_critical_section = !in_critical_section,
                    _ => {}
                }
            }

            if !in_critical_section {
                break
            }
        }

        for behaviour in &mut self.behaviours {
            behaviour.tick(&self.view, connection);
        }
    }
}