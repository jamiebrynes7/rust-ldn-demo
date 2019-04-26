use std::any::Any;
use spatialos_sdk::worker::view::{View, ViewQuery};
use spatialos_sdk::worker::connection::{WorkerConnection, Connection};
use spatialos_sdk::worker::EntityId;
use rust_ldn_demo::shared::generated::demo::{Lumberjack, ActionType, LumberjackUpdate, Action, Headquarters, Tree, TreeCommandRequest, Chop, TreeCommandResponse};
use crate::behaviors::count_trees::TrackTreesBehaviour;
use rust_ldn_demo::shared::generated::improbable::{Position, Coordinates, PositionUpdate};

use kdtree::distance::squared_euclidean;
use rand::seq::SliceRandom;
use rand::prelude::ThreadRng;
use spatialos_sdk::worker::component::UpdateParameters;
use rust_ldn_demo::shared::utils::{squared_distance, normalized_direction, multiply, add_coords};
use spatialos_sdk::worker::commands::CommandParameters;
use std::collections::HashMap;


const SEARCH_DISTANCE: f64 = 125.0;
const MOVE_SPEED: f64 = 0.05; // At 60FPS -> 3 units/second.

pub struct LumberjackBehavior {
    rng: ThreadRng,
    update_params: UpdateParameters,
    commands_in_flight: HashMap<EntityId, u32>
}

impl LumberjackBehavior {
    pub fn new() -> Self {
        let mut params = UpdateParameters::new();
        params.allow_loopback();

        LumberjackBehavior {
            rng: rand::thread_rng(),
            update_params: params,
            commands_in_flight: HashMap::new()
        }
    }

    pub fn tick(&mut self, view: &View, connection: &mut WorkerConnection, trees: &TrackTreesBehaviour) {
        let hqs = view.query::<HqQuery>().collect::<Vec<HqQuery>>();

        for lumberjack in view.query::<LumberjackQuery>() {
            let ljack = lumberjack.lumberjack;
            let pos = lumberjack.position.coords.clone();

            match ljack.action.typ {
                ActionType::IDLE => {
                    let target = trees.within(pos, SEARCH_DISTANCE).collect::<Vec<(&EntityId, &Coordinates)>>();
                    let rand_tree = target.choose(&mut self.rng);

                    if rand_tree.is_none() {
                        continue;
                    }

                    connection.send_component_update::<Lumberjack>(lumberjack.entity_id, LumberjackUpdate {
                        resources: Some(0),
                        action: Some(Action {
                            typ: ActionType::FETCHING,
                            target: Some(*rand_tree.expect("Error").0)
                        })
                    }, self.update_params.clone());
                },
                ActionType::FETCHING => {
                    let target = ljack.action.target.expect("Error");

                    let target_position = match view.get_component::<Position>(target) {
                        Some(c) => c,
                        None => continue
                    };

                    let dist_squared = squared_distance(&pos, &target_position.coords);

                    if dist_squared < 3.0 {
                        let id = connection.send_command_request::<Tree>(target, TreeCommandRequest::TryChop(Chop {}), None, CommandParameters::new());
                        self.commands_in_flight.insert(lumberjack.entity_id, id.id);

                        connection.send_component_update::<Lumberjack>(lumberjack.entity_id, LumberjackUpdate {
                            resources: Some(1),
                            action: Some(Action {
                                typ: ActionType::WAITING,
                                target: ljack.action.target
                            })
                        }, self.update_params.clone())
                    }
                    else {
                        let new_position = move_lumberjack(&pos, &target_position.coords);

                        connection.send_component_update::<Position>(lumberjack.entity_id, PositionUpdate {
                            coords: Some(new_position)
                        }, self.update_params.clone());
                    }
                },
                ActionType::WAITING => {
                    let request_id = self.commands_in_flight.get(&lumberjack.entity_id).unwrap();

                    let response = match view.get_command_responses::<Tree>(ljack.action.target.unwrap()) {
                        Some(vec) => {
                            let matched_responses = vec.iter().filter_map(|(req_id, res)| {
                                if *request_id == req_id.id {
                                    Some(res.clone())
                                }
                                else {
                                    None
                                }
                            }).collect::<Vec<Result<&TreeCommandResponse, String>>>();

                            match matched_responses.first() {
                                Some(res) => res.clone(),
                                None => continue
                            }
                        }
                        None => continue
                    };

                    match response {
                        Ok(r) => {
                            // We got them resources. Update our state.
                            let mut hq_id = hqs.iter()
                                .map(|hq| {
                                    (hq.entity_id, squared_distance(&pos,&hq.position.coords))
                                }).collect::<Vec<(EntityId, f64)>>();

                            hq_id.sort_by(|a, b| {
                                a.1.partial_cmp(&b.1).expect("Error")
                            });

                            if hq_id.is_empty() {
                                eprintln!("No hqs found?");
                            }

                            connection.send_component_update::<Lumberjack>(lumberjack.entity_id, LumberjackUpdate {
                                resources: Some(1),
                                action: Some(Action {
                                    typ: ActionType::RETURNING,
                                    target: Some(hq_id[0].0)
                                })
                            }, self.update_params.clone());
                            self.commands_in_flight.remove(&lumberjack.entity_id);
                        },
                        Err(e) => {
                            eprintln!("Command failed {}. Retrying", e);
                            let id = connection.send_command_request::<Tree>(ljack.action.target.expect("Error"), TreeCommandRequest::TryChop(Chop {}), None, CommandParameters::new());
                            self.commands_in_flight.insert(lumberjack.entity_id, id.id);

                        }
                    }
                },
                ActionType::RETURNING => {
                    let target = ljack.action.target.expect("Error");

                    let target_position = match view.get_component::<Position>(target) {
                        Some(c) => c,
                        None => continue
                    };

                    if squared_distance(&pos, &target_position.coords) < 3.0 {
                        connection.send_component_update::<Lumberjack>(lumberjack.entity_id, LumberjackUpdate {
                            resources: Some(0),
                            action: Some(Action {
                                typ: ActionType::IDLE,
                                target: None
                            })
                        }, self.update_params.clone());
                    }
                    else {
                        let new_position = move_lumberjack(&pos, &target_position.coords);

                        connection.send_component_update::<Position>(lumberjack.entity_id, PositionUpdate {
                            coords: Some(new_position)
                        }, self.update_params.clone());
                    }
                }
                _ => {}
            }
        }
    }
}


fn move_lumberjack(from: &Coordinates, to: &Coordinates) -> Coordinates {
    let mut position_change = normalized_direction(to, from);
    multiply(&mut position_change, MOVE_SPEED);

    add_coords(from, &position_change)
}

struct LumberjackQuery<'a> {
    entity_id: EntityId,
    lumberjack: &'a Lumberjack,
    position: &'a Position
}

impl<'a, 'b: 'a> ViewQuery<'b> for LumberjackQuery<'a> {
    fn filter(view: &View, entity_id: EntityId) -> bool {
            view.is_authoritative::<Lumberjack>(entity_id)
    }

    fn select(view: &'b View, entity_id: EntityId) -> Self {
        LumberjackQuery {
            entity_id,
            lumberjack: view.get_component::<Lumberjack>(entity_id).expect("Error"),
            position: view.get_component::<Position>(entity_id).expect("Error")
        }
    }
}

struct HqQuery<'a> {
    entity_id: EntityId,
    position: &'a Position
}

impl<'a, 'b: 'a> ViewQuery<'b> for HqQuery<'a> {
    fn filter(view: &View, entity_id: EntityId) -> bool {
        view.get_component::<Headquarters>(entity_id).is_some() && view.get_component::<Position>(entity_id).is_some()
    }

    fn select(view: &'b View, entity_id: EntityId) -> Self {
        HqQuery {
            entity_id,
            position: view.get_component::<Position>(entity_id).expect("Error")
        }
    }
}