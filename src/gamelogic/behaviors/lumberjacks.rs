use crate::behaviors::count_trees::TrackTreesBehaviour;
use rust_ldn_demo::shared::generated::demo::{Action, ActionType, Chop, Headquarters, Lumberjack, LumberjackUpdate, Tree, TreeCommandRequest, TreeCommandResponse, HeadquartersCommandRequest, Score};
use rust_ldn_demo::shared::generated::improbable::{Coordinates, Position, PositionUpdate};
use spatialos_sdk::worker::connection::{Connection, WorkerConnection};
use spatialos_sdk::worker::view::{View, ViewQuery};
use spatialos_sdk::worker::{EntityId, RequestId};

use rand::prelude::ThreadRng;
use rand::seq::SliceRandom;
use rust_ldn_demo::shared::utils::{add_coords, multiply, normalized_direction, squared_distance};
use spatialos_sdk::worker::commands::CommandParameters;
use spatialos_sdk::worker::component::UpdateParameters;
use std::collections::HashMap;
use spatialos_sdk::worker::op::StatusCode;

const SEARCH_DISTANCE: f64 = 125.0;
const MOVE_SPEED: f64 = 0.05; // At 60FPS -> 3 units/second.
const DISTANCE_THRESHOLD: f64 = 3.0;

pub struct LumberjackBehavior {
    rng: ThreadRng,
    update_params: UpdateParameters,
    commands_in_flight: HashMap<EntityId, u32>,
}

impl LumberjackBehavior {
    pub fn new() -> Self {
        let mut params = UpdateParameters::new();
        params.allow_loopback();

        LumberjackBehavior {
            rng: rand::thread_rng(),
            update_params: params,
            commands_in_flight: HashMap::new(),
        }
    }

    pub fn tick(
        &mut self,
        view: &View,
        connection: &mut WorkerConnection,
        trees: &TrackTreesBehaviour,
    ) {
        let hqs = view.query::<HqQuery>().collect::<Vec<HqQuery>>();

        for lumberjack in view.query::<LumberjackQuery>() {
            match lumberjack.lumberjack.action.typ {
                ActionType::IDLE => self.do_idle(&lumberjack, connection, trees),
                ActionType::FETCHING => self.do_fetch(&lumberjack, view, connection),
                ActionType::WAITING => self.do_wait(&lumberjack, view, connection, &hqs),
                ActionType::RETURNING => self.do_return(&lumberjack, view, connection),
            }
        }
    }

    fn do_idle(
        &mut self,
        lumberjack: &LumberjackQuery,
        connection: &mut WorkerConnection,
        trees: &TrackTreesBehaviour,
    ) {
        let possible_targets = trees
            .within(lumberjack.position.coords.clone(), SEARCH_DISTANCE)
            .collect::<Vec<EntityId>>();

        let rand_tree = possible_targets.choose(&mut self.rng);

        match rand_tree {
            Some(id) => connection.send_component_update::<Lumberjack>(
                lumberjack.entity_id,
                LumberjackUpdate {
                    action: Some(Action {
                        typ: ActionType::FETCHING,
                        target: Some(*id),
                    }),
                },
                self.update_params.clone(),
            ),
            // TODO: Retry search with larger radius after this?
            None => {}
        }
    }

    fn do_fetch(
        &mut self,
        lumberjack: &LumberjackQuery,
        view: &View,
        connection: &mut WorkerConnection,
    ) {
        let target = lumberjack.lumberjack.action.target.expect("Error");
        let pos = &lumberjack.position.coords;

        let target_position = match view.get_component::<Position>(target) {
            Some(pos) => pos,
            None => return,
        };

        if squared_distance(pos, &target_position.coords) > DISTANCE_THRESHOLD {
            connection.send_component_update::<Position>(
                lumberjack.entity_id,
                PositionUpdate {
                    coords: Some(move_lumberjack(pos, &target_position.coords)),
                },
                self.update_params.clone(),
            );
        } else {
            let id = connection.send_command_request::<Tree>(
                target,
                TreeCommandRequest::TryChop(Chop {}),
                None,
                CommandParameters::new(),
            );
            self.commands_in_flight.insert(lumberjack.entity_id, id.id);

            connection.send_component_update::<Lumberjack>(
                lumberjack.entity_id,
                LumberjackUpdate {
                    action: Some(Action {
                        typ: ActionType::WAITING,
                        target: Some(target),
                    }),
                },
                self.update_params.clone(),
            );
        }
    }

    fn do_wait(
        &mut self,
        lumberjack: &LumberjackQuery,
        view: &View,
        connection: &mut WorkerConnection,
        hqs: &Vec<HqQuery>,
    ) {
        let request_id = self.commands_in_flight.get(&lumberjack.entity_id).unwrap();
        let target = lumberjack.lumberjack.action.target.expect("Error");
        let pos = &lumberjack.position.coords;

        let response = match view.get_command_response::<Tree>(target, RequestId::new(*request_id)) {
            Some(r) => r,
            None => return
        };

        self.commands_in_flight.remove(&lumberjack.entity_id);

        match response {
            StatusCode::Success(_) => {
                // We got them resources. Update our state.
                let mut hq_ids = hqs
                    .iter()
                    .map(|hq| (hq.entity_id, squared_distance(pos, &hq.position.coords)))
                    .collect::<Vec<(EntityId, f64)>>();

                hq_ids.sort_by(|a, b| a.1.partial_cmp(&b.1).expect("Error"));

                match hq_ids.get(0) {
                    Some((id, _)) => connection.send_component_update::<Lumberjack>(
                        lumberjack.entity_id,
                        LumberjackUpdate {
                            action: Some(Action {
                                typ: ActionType::RETURNING,
                                target: Some(*id),
                            }),
                        },
                        self.update_params.clone(),
                    ),
                    None => eprintln!("No hqs found?"),
                }
            },
            StatusCode::ApplicationError(_) => {
                connection.send_component_update::<Lumberjack>(
                    lumberjack.entity_id,
                    LumberjackUpdate {
                        action: Some(Action {
                            typ: ActionType::IDLE,
                            target: None
                        })
                    },
                    self.update_params.clone(),
                )
            },
            _ => {
                let id = connection.send_command_request::<Tree>(
                    target,
                    TreeCommandRequest::TryChop(Chop {}),
                    None,
                    CommandParameters::new(),
                );
                self.commands_in_flight.insert(lumberjack.entity_id, id.id);
            }
        }
    }

    fn do_return(&mut self, lumberjack: &LumberjackQuery, view: &View, connection: &mut WorkerConnection) {
        let target = lumberjack.lumberjack.action.target.expect("Error");
        let pos = &lumberjack.position.coords;

        let target_position = match view.get_component::<Position>(target) {
            Some(c) => c,
            None => return,
        };

        if squared_distance(pos, &target_position.coords) > DISTANCE_THRESHOLD {
            connection.send_component_update::<Position>(
                lumberjack.entity_id,
                PositionUpdate {
                    coords: Some(move_lumberjack(pos, &target_position.coords)),
                },
                self.update_params.clone(),
            );
        } else {
            connection.send_component_update::<Lumberjack>(
                lumberjack.entity_id,
                LumberjackUpdate {
                    action: Some(Action {
                        typ: ActionType::IDLE,
                        target: None,
                    }),
                },
                self.update_params.clone(),
            );

            // Fire and forget.
            connection.send_command_request::<Headquarters>(
                target,
                HeadquartersCommandRequest::Deposit(Score{}),
                None,
                CommandParameters::new()
            );
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
    position: &'a Position,
}

impl<'a, 'b: 'a> ViewQuery<'b> for LumberjackQuery<'a> {
    fn filter(view: &View, entity_id: EntityId) -> bool {
        view.is_authoritative::<Lumberjack>(entity_id)
    }

    fn select(view: &'b View, entity_id: EntityId) -> Self {
        LumberjackQuery {
            entity_id,
            lumberjack: view.get_component::<Lumberjack>(entity_id).expect("Error"),
            position: view.get_component::<Position>(entity_id).expect("Error"),
        }
    }
}

struct HqQuery<'a> {
    entity_id: EntityId,
    position: &'a Position,
}

impl<'a, 'b: 'a> ViewQuery<'b> for HqQuery<'a> {
    fn filter(view: &View, entity_id: EntityId) -> bool {
        view.get_component::<Headquarters>(entity_id).is_some()
            && view.get_component::<Position>(entity_id).is_some()
    }

    fn select(view: &'b View, entity_id: EntityId) -> Self {
        HqQuery {
            entity_id,
            position: view.get_component::<Position>(entity_id).expect("Error"),
        }
    }
}
