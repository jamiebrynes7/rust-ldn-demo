use std::any::Any;
use spatialos_sdk::worker::view::{View, ViewQuery};
use spatialos_sdk::worker::connection::{WorkerConnection, Connection};
use spatialos_sdk::worker::EntityId;
use rust_ldn_demo::shared::generated::demo::{Lumberjack, ActionType, LumberjackUpdate, Action};
use crate::behaviors::count_trees::TrackTreesBehaviour;
use rust_ldn_demo::shared::generated::improbable::{Position, Coordinates, PositionUpdate};

use kdtree::distance::squared_euclidean;
use rand::seq::SliceRandom;
use rand::prelude::ThreadRng;
use spatialos_sdk::worker::component::UpdateParameters;
use rust_ldn_demo::shared::utils::{squared_distance, normalized_direction, multiply, add_coords};


const SEARCH_DISTANCE: f64 = 125.0;
const MOVE_SPEED: f64 = 0.05; // At 60FPS -> 3 units/second.

pub struct LumberjackBehavior {
    rng: ThreadRng,
    update_params: UpdateParameters
}

impl LumberjackBehavior {
    pub fn new() -> Self {
        let mut params = UpdateParameters::new();
        params.allow_loopback();

        LumberjackBehavior {
            rng: rand::thread_rng(),
            update_params: params
        }
    }

    pub fn tick(&mut self, view: &View, connection: &mut WorkerConnection, trees: &TrackTreesBehaviour) {
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
                            target: Some(*rand_tree.unwrap().0)
                        })
                    }, self.update_params.clone());
                },
                ActionType::FETCHING => {
                    let target = ljack.action.target.unwrap();
                    let target_position = view.get_component::<Position>(target).unwrap();

                    let dist_squared = squared_distance(&pos, &target_position.coords);

                    if dist_squared < 3.0 {
                        connection.send_component_update::<Lumberjack>(lumberjack.entity_id, LumberjackUpdate {
                            resources: Some(1),
                            action: Some(Action {
                                typ: ActionType::RETURNING,
                                target: None
                            })
                        }, self.update_params.clone())
                    }
                    else {
                        let mut position_change = normalized_direction(&target_position.coords, &pos);
                        multiply(&mut position_change, MOVE_SPEED);

                        let new_position = add_coords(&pos, &position_change);

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
            lumberjack: view.get_component::<Lumberjack>(entity_id).unwrap(),
            position: view.get_component::<Position>(entity_id).unwrap()
        }
    }
}