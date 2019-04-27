use rust_ldn_demo::shared::generated::demo::{Chop, Tree, TreeCommandResponse, TreeUpdate, Fire, FireCommandRequest, FireUpdate, FireCommandResponse, TriggerFire};
use spatialos_sdk::worker::connection::{Connection, WorkerConnection};
use spatialos_sdk::worker::view::{View, ViewQuery};
use spatialos_sdk::worker::EntityId;
use rust_ldn_demo::shared::generated::improbable::{Coordinates, Position, Metadata, MetadataUpdate};
use rust_ldn_demo::shared::utils::squared_distance;
use spatialos_sdk::worker::component::{UpdateParameters, Component};
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};

pub struct TrackTreesBehaviour {
    trees: HashMap<EntityId, Coordinates>,
    inactive_trees: HashMap<EntityId, Coordinates>
}

impl TrackTreesBehaviour {
    pub fn new() -> Self {
        TrackTreesBehaviour {
            trees: HashMap::new(),
            inactive_trees: HashMap::new()
        }
    }

    pub fn tick(&mut self, view: &View, connection: &mut WorkerConnection) {
        for removed in view.iter_entities_removed() {
            self.trees.remove(removed);
            self.inactive_trees.remove(removed);
        }

        for added in view.query::<TreeAddedQuery>() {
            if added.is_active {
                self.trees.insert(added.id, added.position.coords.clone());
                continue;
            }

            self.inactive_trees.insert(added.id, added.position.coords.clone());
        }

        for updated in view.query::<TreeUpdatedQuery<Tree>>() {
            if updated.current_value.resources_left == 0 {
                match self.trees.remove(&updated.entity_id) {
                    Some(coords) => self.inactive_trees.insert(updated.entity_id, coords),
                    None => continue
                };
            }

            // TODO: May need to add if supporting adding extra resources.
        }

        for updated in view.query::<TreeUpdatedQuery<Fire>>() {
            if updated.current_value.is_on_fire {
                match self.trees.remove(&updated.entity_id) {
                    Some(coords) => self.inactive_trees.insert(updated.entity_id, coords),
                    None => continue
                };
            }
            else {
                match self.inactive_trees.remove(&updated.entity_id) {
                    Some(coords) => self.trees.insert(updated.entity_id, coords),
                    None => continue
                };
            }
        }
    }

    pub fn within_active(
        &self,
        coords: Coordinates,
        radius: f64,
    ) -> impl Iterator<Item = EntityId> + '_ {
        self.trees
            .iter()
            .filter(move |(id, c)| squared_distance(&coords, c) < radius.powi(2))
            .map(|(id, _)| *id)
    }

    pub fn within_inactive(
        &self,
        coords: Coordinates,
        radius: f64,
    ) -> impl Iterator<Item = EntityId> + '_ {
        self.inactive_trees
            .iter()
            .filter(move |(id, c)| squared_distance(&coords, c) < radius.powi(2))
            .map(|(id, _)| *id)
    }
}

struct TreeAddedQuery<'a> {
    pub id: EntityId,
    pub position: &'a Position,
    pub is_active: bool,
}

impl<'a, 'b: 'a> ViewQuery<'b> for TreeAddedQuery<'a> {
    fn filter(view: &View, entity_id: EntityId) -> bool {
        view.was_entity_added(entity_id)
            && view.get_component::<Tree>(entity_id).is_some()
            && view.get_component::<Position>(entity_id).is_some()
            && view.get_component::<Fire>(entity_id).is_some()
    }

    fn select(view: &'b View, entity_id: EntityId) -> Self {
        let tree = view.get_component::<Tree>(entity_id).unwrap();
        let fire = view.get_component::<Fire>(entity_id).unwrap();
        TreeAddedQuery {
            id: entity_id,
            position: view.get_component::<Position>(entity_id).unwrap(),
            is_active: tree.resources_left > 0 && !fire.is_on_fire
        }
    }
}

struct TreeUpdatedQuery<'a, T: Component> {
    pub entity_id: EntityId,
    pub current_value: &'a T
}

impl <'a, 'b: 'a, T: Component> ViewQuery<'b> for TreeUpdatedQuery<'a, T> {
    fn filter(view: &View, entity_id: EntityId) -> bool {
        view.was_component_updated::<T>(entity_id)
    }

    fn select(view: &'b View, entity_id: EntityId) -> Self {
        TreeUpdatedQuery {
            entity_id,
            current_value: view.get_component::<T>(entity_id).unwrap()
        }
    }
}
