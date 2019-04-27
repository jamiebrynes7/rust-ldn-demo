use rust_ldn_demo::shared::generated::demo::{Chop, Tree, TreeCommandResponse, TreeUpdate, Fire, FireCommandRequest, FireUpdate, FireCommandResponse, TriggerFire};
use spatialos_sdk::worker::connection::{Connection, WorkerConnection};
use spatialos_sdk::worker::view::{View, ViewQuery};
use spatialos_sdk::worker::EntityId;
use rust_ldn_demo::shared::generated::improbable::{Coordinates, Position, Metadata, MetadataUpdate};
use rust_ldn_demo::shared::utils::squared_distance;
use spatialos_sdk::worker::component::UpdateParameters;
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use rand::prelude::ThreadRng;
use rand::Rng;
use spatialos_sdk::worker::commands::CommandParameters;
use std::time::{Duration, SystemTime};

const FIRE_SPREAD_RADIUS: f64 = 10.0;
const FIRE_SPREAD_CHANCE: f64 = 0.10;
const FIRE_SPREAD_TIMEOUT_MS: u128 = 5000;

pub struct TrackTreesBehaviour {
    trees: HashMap<EntityId, Coordinates>,
    inactive_trees: HashMap<EntityId, Coordinates>,
    params: UpdateParameters,
    rng: ThreadRng,
    last_spread: SystemTime
}

impl TrackTreesBehaviour {
    pub fn new() -> Self {
        let mut params = UpdateParameters::new();
        params.allow_loopback();

        TrackTreesBehaviour {
            trees: HashMap::new(),
            inactive_trees: HashMap::new(),
            params,
            rng: rand::thread_rng(),
            last_spread: SystemTime::now()
        }
    }

    pub fn tick(&mut self, view: &View, connection: &mut WorkerConnection) {
        for removed in view.iter_entities_removed() {
            self.trees.remove(removed);
            self.inactive_trees.remove(removed);
        }

        for ref added in view.query::<TreeAddedQuery>() {
            if added.tree.resources_left > 0 && !added.is_on_fire {
                self.trees.insert(added.id, added.position.coords.clone());
                continue;
            }

            self.inactive_trees.insert(added.id, added.position.coords.clone());
        }

        for entity in view.query::<TreeFireRequest>() {
            let requests = view.get_command_requests::<Fire>(entity.entity_id).unwrap();

            for (id, req)  in requests {
                match req {
                    FireCommandRequest::SetOnFire(_) => {
                        connection.send_component_update::<Fire>(entity.entity_id, FireUpdate {
                            is_on_fire: Some(true)
                        }, self.params.clone());

                        connection.send_component_update::<Metadata>(entity.entity_id, MetadataUpdate {
                            entity_type: Some("Tree (Fire)".into())
                        }, self.params.clone());

                        connection.send_command_response::<Fire>(id, FireCommandResponse::SetOnFire(TriggerFire {}));

                        match self.trees.remove(&entity.entity_id) {
                            Some(coords) => self.inactive_trees.insert(entity.entity_id, coords),
                            None => continue
                        };
                    },
                    FireCommandRequest::ClearFire(_) => {
                        connection.send_component_update::<Fire>(entity.entity_id, FireUpdate {
                            is_on_fire: Some(false)
                        }, self.params.clone());

                        let has_resources = view.get_component::<Tree>(entity.entity_id).unwrap().resources_left > 0;

                        let entity_name = format!("Tree{}", if has_resources { "" } else { " (Empty)" });

                        connection.send_component_update::<Metadata>(entity.entity_id, MetadataUpdate {
                            entity_type: Some(entity_name)
                        }, self.params.clone());

                        connection.send_command_response::<Fire>(id, FireCommandResponse::ClearFire(TriggerFire {}));

                        if has_resources {
                            match self.inactive_trees.remove(&entity.entity_id) {
                                Some(coords) => self.trees.insert(entity.entity_id, coords),
                                None => continue
                            };
                        }
                    }
                }
            }
        }

        for entity in view.query::<TreeRequestQuery>() {
            let requests = view.get_command_requests::<Tree>(entity.entity_id).unwrap();

            let max_responses = if self.inactive_trees.contains_key(&entity.entity_id) {
                0
            } else {
                min(requests.len(), entity.tree.resources_left as usize)
            };

            for i in 0..max_responses {
                connection.send_command_response::<Tree>(
                    requests[i].0,
                    TreeCommandResponse::TryChop(Chop {}),
                );
            }

            for i in max_responses..requests.len() {
                connection.send_command_failure(requests[i].0, "Tree not available.");
            }

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
                self.params.clone(),
            );

            if leftover_resources == 0 && entity.tree.resources_left > 0 {
                self.inactive_trees.insert(entity.entity_id, self.trees.remove(&entity.entity_id).unwrap());

                connection.send_component_update::<Metadata>(entity.entity_id, MetadataUpdate {
                    entity_type: Some("Tree (Empty)".into())
                }, self.params.clone());
            }
        }

        let now = SystemTime::now();

        if now.duration_since(self.last_spread).unwrap().as_millis() > FIRE_SPREAD_TIMEOUT_MS {
            self.last_spread = now;

            let targets = view.query::<TreesOnFire>().flat_map(|coords| {
                self.within(coords.coords.clone(), FIRE_SPREAD_RADIUS)
            }).collect::<HashSet<EntityId>>();

            for target in targets {
                if self.rng.gen_bool(FIRE_SPREAD_CHANCE) {
                    connection.send_command_request::<Fire>(target, FireCommandRequest::SetOnFire(TriggerFire {} ), None, CommandParameters::new());
                }
            }
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
    pub tree: &'a Tree,
    pub is_on_fire: bool
}

impl<'a, 'b: 'a> ViewQuery<'b> for TreeAddedQuery<'a> {
    fn filter(view: &View, entity_id: EntityId) -> bool {
        view.was_entity_added(entity_id)
            && view.get_component::<Tree>(entity_id).is_some()
            && view.get_component::<Position>(entity_id).is_some()
            && view.get_component::<Fire>(entity_id).is_some()
    }

    fn select(view: &'b View, entity_id: EntityId) -> Self {
        TreeAddedQuery {
            id: entity_id,
            position: view.get_component::<Position>(entity_id).unwrap(),
            tree: view.get_component::<Tree>(entity_id).unwrap(),
            is_on_fire: view.get_component::<Fire>(entity_id).unwrap().is_on_fire
        }
    }
}

struct TreeRequestQuery<'a> {
    pub entity_id: EntityId,
    pub tree: &'a Tree,
}

impl<'a, 'b: 'a> ViewQuery<'b> for TreeRequestQuery<'a> {
    fn filter(view: &View, entity_id: EntityId) -> bool {
        view.has_command_requests::<Tree>(entity_id) && view.get_component::<Tree>(entity_id).is_some()
    }

    fn select(view: &'b View, entity_id: EntityId) -> Self {
        TreeRequestQuery {
            entity_id,
            tree: view.get_component::<Tree>(entity_id).unwrap(),
        }
    }
}

struct TreeFireRequest {
    pub entity_id: EntityId
}

impl<'b> ViewQuery<'b> for TreeFireRequest {
    fn filter(view: &View, entity_id: EntityId) -> bool {
        view.has_command_requests::<Fire>(entity_id)
    }

    fn select(view: &'b View, entity_id: EntityId) -> Self {
        TreeFireRequest {
            entity_id
        }
    }
}

struct TreesOnFire {
    pub coords: Coordinates
}

impl<'b> ViewQuery<'b> for TreesOnFire {
    fn filter(view: &View, entity_id: EntityId) -> bool {
        let is_on_fire = match view.get_component::<Fire>(entity_id) {
            Some(fire) => fire.is_on_fire,
            None => false
        };

        is_on_fire && view.get_component::<Position>(entity_id).is_some()
    }

    fn select(view: &'b View, entity_id: EntityId) -> Self {
        TreesOnFire {
            coords: view.get_component::<Position>(entity_id).unwrap().coords.clone()
        }
    }
}
