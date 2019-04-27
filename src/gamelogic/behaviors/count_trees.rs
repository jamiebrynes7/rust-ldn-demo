use rust_ldn_demo::shared::generated::demo::{Chop, Tree, TreeCommandResponse, TreeUpdate};
use spatialos_sdk::worker::connection::{Connection, WorkerConnection};
use spatialos_sdk::worker::view::{View, ViewQuery};
use spatialos_sdk::worker::EntityId;
use rust_ldn_demo::shared::generated::improbable::{Coordinates, Position};
use rust_ldn_demo::shared::utils::squared_distance;
use spatialos_sdk::worker::component::UpdateParameters;
use std::any::Any;
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};

pub struct TrackTreesBehaviour {
    trees: HashMap<EntityId, Coordinates>,
}

impl TrackTreesBehaviour {
    pub fn new() -> Self {
        TrackTreesBehaviour {
            trees: HashMap::new(),
        }
    }

    pub fn tick(&mut self, view: &View, connection: &mut WorkerConnection) {
        for removed in view.iter_entities_removed() {
            self.trees.remove(removed);
        }

        for ref added in view.query::<TreeAddedQuery>() {
            self.trees.insert(added.id, added.position.coords.clone());
        }

        for entity in view.query::<TreeRequestQuery>() {
            let requests = view.get_command_requests::<Tree>(entity.entity_id).unwrap();

            let max_responses = min(requests.len(), entity.tree.resources_left as usize);

            for i in 0..max_responses {
                connection.send_command_response::<Tree>(
                    requests[i].0,
                    TreeCommandResponse::TryChop(Chop {}),
                );
            }

            for i in max_responses..requests.len() {
                connection.send_command_failure(requests[i].0, "No more resources.");
            }

            let mut params = UpdateParameters::new();
            params.allow_loopback();

            let leftover_resources = if max_responses as u32 >= entity.tree.resources_left {
                0
            } else {
                entity.tree.resources_left - max_responses as u32
            };

            connection.send_component_update::<Tree>(
                entity.entity_id,
                TreeUpdate {
                    resources_left: Some(leftover_resources),
                },
                params,
            );
        }
    }

    pub fn within(
        &self,
        coords: Coordinates,
        radius: f64,
    ) -> impl Iterator<Item = EntityId> + '_ {
        self.trees
            .iter()
            .filter(move |(id, c)| squared_distance(&coords, c) < radius.powi(2))
            .map(|(id, _)| *id)
    }
}

struct TreeAddedQuery<'a> {
    pub id: EntityId,
    pub position: &'a Position,
}

impl<'a, 'b: 'a> ViewQuery<'b> for TreeAddedQuery<'a> {
    fn filter(view: &View, entity_id: EntityId) -> bool {
        view.was_entity_added(entity_id)
            && view.get_component::<Tree>(entity_id).is_some()
            && view.get_component::<Position>(entity_id).is_some()
    }

    fn select(view: &'b View, entity_id: EntityId) -> Self {
        TreeAddedQuery {
            id: entity_id,
            position: view.get_component::<Position>(entity_id).unwrap(),
        }
    }
}

struct TreeRequestQuery<'a> {
    pub entity_id: EntityId,
    pub tree: &'a Tree,
}

impl<'a, 'b: 'a> ViewQuery<'b> for TreeRequestQuery<'a> {
    fn filter(view: &View, entity_id: EntityId) -> bool {
        view.get_component::<Tree>(entity_id).is_some()
            && view.has_command_requests::<Tree>(entity_id)
    }

    fn select(view: &'b View, entity_id: EntityId) -> Self {
        TreeRequestQuery {
            entity_id,
            tree: view.get_component::<Tree>(entity_id).unwrap(),
        }
    }
}
