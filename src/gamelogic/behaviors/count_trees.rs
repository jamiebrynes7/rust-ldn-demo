use rust_ldn_demo::shared::behavior::Behavior;
use spatialos_sdk::worker::view::{View, ViewQuery};
use spatialos_sdk::worker::connection::WorkerConnection;
use rust_ldn_demo::shared::generated::demo::Tree;
use spatialos_sdk::worker::EntityId;

use kdtree::KdTree;
use rust_ldn_demo::shared::generated::improbable::Position;
use std::collections::HashSet;

pub struct TrackTreesBehaviour {
    tree: KdTree<f64, EntityId, [f64; 3]>,
    entities: HashSet<EntityId>
}

impl TrackTreesBehaviour {
    pub fn new() -> Self {
        TrackTreesBehaviour {
            tree: KdTree::new(3),
            entities: HashSet::new()
        }
    }
}

impl Behavior for TrackTreesBehaviour {
    fn tick(&mut self, view: &View, connection: &mut WorkerConnection) {
        let mut should_rebuild_tree = false;

        for ref change in view.query::<TreeChange>() {
            match change.added_entity {
                Some(id) => {
                    should_rebuild_tree = true;
                    self.entities.insert(id);
                    continue
                },
                None => {}
            }

            match change.removed_entity {
                Some(id) => {
                    should_rebuild_tree = true;
                    self.entities.remove(&id);
                }
                None => {}
            }
        }

        if should_rebuild_tree {
            self.tree = KdTree::new_with_capacity(3, self.entities.len());
            println!("Rebuild kd-tree with capacity: {}", self.entities.len());
            for ref tree_entity in view.query::<TreeQuery>() {
                let pos = &tree_entity.position.coords;
                self.tree.add([pos.x, pos.y, pos.z], tree_entity.entity_id).expect("Error inserting into KD tree.");
            }
        }
    }
}

struct TreeChange {
    pub added_entity: Option<EntityId>,
    pub removed_entity: Option<EntityId>
}

impl<'b> ViewQuery<'b> for TreeChange {
    fn filter(view: &View, entity_id: EntityId) -> bool {
        (view.was_entity_added(entity_id) && view.get_component::<Tree>(entity_id).is_some())
            || view.was_entity_removed(entity_id)
    }

    fn select(view: &'b View, entity_id: EntityId) -> Self {
        if view.was_entity_removed(entity_id) {
            return TreeChange {
                added_entity: None,
                removed_entity: Some(entity_id)
            }
        }

        TreeChange {
            added_entity: Some(entity_id),
            removed_entity: None
        }
    }
}

struct TreeQuery<'a> {
    entity_id: EntityId,
    position: &'a Position
}

impl<'a, 'b: 'a> ViewQuery<'b> for TreeQuery<'a> {
    fn filter(view: &View, entity_id: EntityId) -> bool {
        view.get_component::<Tree>(entity_id).is_some() && view.get_component::<Position>(entity_id).is_some()
    }

    fn select(view: &'b View, entity_id: EntityId) -> Self {
        TreeQuery {
            entity_id,
            position: view.get_component::<Position>(entity_id).unwrap(),
        }
    }
}