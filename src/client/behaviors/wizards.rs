use rust_ldn_demo::shared::generated::demo::{WizardActionType, Wizard, WizardFaction, WizardUpdate, WizardAction, Fire, FireCommandRequest, TriggerFire};
use spatialos_sdk::worker::connection::{WorkerConnection, Connection};
use crate::behaviors::trees::TrackTreesBehaviour;
use spatialos_sdk::worker::view::{View, ViewQuery};
use rust_ldn_demo::shared::generated::improbable::{Position, Coordinates, PositionUpdate};
use spatialos_sdk::worker::EntityId;
use rand::seq::SliceRandom;
use spatialos_sdk::worker::component::UpdateParameters;
use rand::prelude::ThreadRng;
use rust_ldn_demo::shared::utils::{squared_distance, normalized_direction, multiply, add_coords, move_to};
use rust_ldn_demo::shared::generated::demo::WizardActionType::MOVING;
use spatialos_sdk::worker::commands::CommandParameters;

const SEARCH_RADIUS: f64 = 150.0;
const MOVE_SPEED: f64 = 0.05; // At 60FPS -> 3 units/second.
const DISTANCE_THRESHOLD: f64 = 3.0;

pub struct WizardBehavior {
    rng: ThreadRng,
    update_params: UpdateParameters,
}

impl WizardBehavior {
    pub fn new() -> Self {
        let mut params = UpdateParameters::new();
        params.allow_loopback();

        WizardBehavior {
            rng: rand::thread_rng(),
            update_params: params,
        }
    }

    pub fn tick(&mut self, view: &View, connection: &mut WorkerConnection, trees: &TrackTreesBehaviour) {

        for wizard in view.query::<WizardQuery>() {
            match wizard.wiz.action.typ {
                WizardActionType::IDLE => self.do_idle(&wizard, view, connection, trees),
                WizardActionType::MOVING => self.do_move(&wizard, view, connection),
                WizardActionType::SPELL => self.do_spell(&wizard, connection)
            }
        }
    }

    fn do_idle(&mut self, wiz: &WizardQuery, view: &View, connection: &mut WorkerConnection, trees: &TrackTreesBehaviour) {
        for attempts in 1..5 {
            let possible_targets = self.find_targets(wiz.wiz.faction, &wiz.position.coords, SEARCH_RADIUS * attempts as f64, trees);
            let rand_tree = possible_targets.choose(&mut self.rng);

            match rand_tree {
                Some(id) =>  {
                    let target_position = view.get_component::<Position>(*id).unwrap();

                    connection.send_component_update::<Wizard>(
                        wiz.entity_id,
                        WizardUpdate {
                            faction: None,
                            action: Some(WizardAction {
                                typ: WizardActionType::MOVING,
                                target: Some(*id),
                                target_pos: Some(target_position.coords.clone())
                            })
                        },
                        self.update_params.clone(),
                    );
                    break;
                },
                None => {}
            }
        }
    }

    fn do_move(&mut self, wiz: &WizardQuery, view: &View, connection: &mut WorkerConnection) {
        let target = wiz.wiz.action.target.unwrap();
        let target_position = wiz.wiz.action.target_pos.as_ref().unwrap();
        let pos = &wiz.position.coords;

        if squared_distance(pos, target_position) > DISTANCE_THRESHOLD {
            connection.send_component_update::<Position>(
                wiz.entity_id,
                PositionUpdate {
                    coords: Some(move_to(pos, target_position, MOVE_SPEED)),
                },
                self.update_params.clone(),
            );
        } else {
            connection.send_component_update::<Wizard>(
                wiz.entity_id,
                WizardUpdate {
                    faction: None,
                    action: Some(WizardAction {
                        typ: WizardActionType::SPELL,
                        target: Some(target),
                        target_pos: Some(target_position.clone())
                    })
                },
                self.update_params.clone(),
            );
        }
    }

    fn do_spell(&mut self, wiz: &WizardQuery, connection: &mut WorkerConnection) {
        let target = wiz.wiz.action.target.unwrap();

        match wiz.wiz.faction {
            WizardFaction::GOOD => {
                connection.send_command_request::<Fire>(target, FireCommandRequest::ClearFire(TriggerFire {}), None, CommandParameters::new())
            },
            WizardFaction::EVIL => {
                connection.send_command_request::<Fire>(target, FireCommandRequest::SetOnFire(TriggerFire {}), None, CommandParameters::new())
            }
        };

        connection.send_component_update::<Wizard>(
            wiz.entity_id,
            WizardUpdate {
                faction: None,
                action: Some(WizardAction {
                    typ: WizardActionType::IDLE,
                    target: None,
                    target_pos: None
                })
            },
            self.update_params.clone(),
        );
    }

    fn find_targets(&self,
        faction: WizardFaction,
        coords: &Coordinates,
        radius: f64,
        trees: &TrackTreesBehaviour
    ) -> Vec<EntityId> {
        match faction {
            WizardFaction::GOOD => trees.within_inactive(coords.clone(), radius).collect::<Vec<EntityId>>(),
            WizardFaction::EVIL => trees.within_active(coords.clone(), radius).collect::<Vec<EntityId>>()
        }
    }
}

struct WizardQuery<'a> {
    pub entity_id: EntityId,
    pub position: &'a Position,
    pub wiz: &'a Wizard
}

impl<'a, 'b: 'a> ViewQuery<'b> for WizardQuery<'a> {
    fn filter(view: &View, entity_id: EntityId) -> bool {
        view.get_component::<Position>(entity_id).is_some() &&
            view.get_component::<Wizard>(entity_id).is_some() &&
            view.is_authoritative::<Wizard>(entity_id)
    }

    fn select(view: &'b View, entity_id: EntityId) -> Self {
        WizardQuery {
            entity_id,
            position: view.get_component::<Position>(entity_id).unwrap(),
            wiz: view.get_component::<Wizard>(entity_id).unwrap()
        }
    }
}