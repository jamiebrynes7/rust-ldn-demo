use rust_ldn_demo::shared::behavior::Behavior;
use spatialos_sdk::worker::view::{View, ViewQuery};
use spatialos_sdk::worker::connection::WorkerConnection;
use rust_ldn_demo::shared::generated::demo::Tree;
use spatialos_sdk::worker::EntityId;

pub struct CountTreesBehaviour {

}

impl Behavior for CountTreesBehaviour {
    fn tick(&mut self, view: &View, connection: &mut WorkerConnection) {
        let tree_count = view.query::<TreeQuery>().count();
        println!("{}", tree_count);
    }
}

struct TreeQuery {}

impl<'b> ViewQuery<'b> for TreeQuery {
    fn filter(view: &View, entity_id: &EntityId) -> bool {
        view.is_authoritative::<Tree>(entity_id)
    }

    fn select(view: &'b View, entity_id: EntityId) -> Self {
        TreeQuery {}
    }
}