use spatialos_sdk::worker::view::{View, ViewQuery};
use spatialos_sdk::worker::connection::WorkerConnection;
use rust_ldn_demo::shared::generated::demo::Tree;

use spatialos_sdk::worker::EntityId;

use kdtree::KdTree;
use rust_ldn_demo::shared::generated::improbable::{Position, Coordinates};
use std::collections::{HashSet, HashMap};
use std::any::Any;
use rust_ldn_demo::shared::utils::squared_distance;

pub struct TrackTreesBehaviour {
    trees: HashMap<EntityId, Coordinates>
}

impl TrackTreesBehaviour {
    pub fn new() -> Self {
        TrackTreesBehaviour {
            trees: HashMap::new()
        }
    }

    pub fn tick(&mut self, view: &View, _connection: &mut WorkerConnection) {

        for removed in view.iter_entities_removed() {
            self.trees.remove(removed);
        }

        for ref added in view.query::<TreeAddedQuery>() {
            self.trees.insert(added.id, added.position.coords.clone());
        }
    }

    pub fn within(&self, coords: Coordinates, radius: f64) -> impl Iterator<Item = (&EntityId, &Coordinates)> {
        self.trees.iter()
            .filter(move |(id, c)|  {
                squared_distance(&coords, c) < radius.powi(2)
            })
    }
}

struct TreeAddedQuery<'a> {
    pub id: EntityId,
    pub position: &'a Position
}

impl<'a, 'b: 'a> ViewQuery<'b> for TreeAddedQuery<'a> {
    fn filter(view: &View, entity_id: EntityId) -> bool {
        view.was_entity_added(entity_id) && view.get_component::<Tree>(entity_id).is_some() && view.get_component::<Position>(entity_id).is_some()
    }

    fn select(view: &'b View, entity_id: EntityId) -> Self {
        TreeAddedQuery {
            id: entity_id,
            position: view.get_component::<Position>(entity_id).unwrap(),
        }
    }
}