use spatialos_sdk::worker::connection::{WorkerConnection, Connection};
use spatialos_sdk::worker::view::{View, ViewQuery};
use spatialos_sdk::worker::EntityId;
use rust_ldn_demo::shared::generated::demo::{Headquarters, HeadquartersCommandResponse, Score, HeadquartersUpdate};
use spatialos_sdk::worker::component::UpdateParameters;

pub struct HqBehaviour { }

impl HqBehaviour {
    pub fn new() -> Self {
        HqBehaviour { }
    }

    pub fn tick(&mut self, view: &View, connection: &mut WorkerConnection) {
        let mut params = UpdateParameters::new();
        params.allow_loopback();

        for entity in view.query::<HqScoreRequestQuery>() {
            let requests = view.get_command_requests::<Headquarters>(entity.entity_id).unwrap();

            for (req_id, _) in &requests {
                connection.send_command_response::<Headquarters>(*req_id, HeadquartersCommandResponse::Deposit(Score {}));
            }

            connection.send_component_update::<Headquarters>(entity.entity_id, HeadquartersUpdate {
                score: Some(entity.hq.score + requests.len() as u32)
            }, params);
        }
    }
}

struct HqScoreRequestQuery<'a> {
    pub entity_id: EntityId,
    pub hq: &'a Headquarters,
}

impl <'a, 'b: 'a> ViewQuery<'b> for HqScoreRequestQuery<'a> {
    fn filter(view: &View, entity_id: EntityId) -> bool {
        view.get_component::<Headquarters>(entity_id).is_some()
        && view.has_command_requests::<Headquarters>(entity_id)
    }

    fn select(view: &'b View, entity_id: EntityId) -> Self {
        HqScoreRequestQuery {
            entity_id,
            hq: view.get_component::<Headquarters>(entity_id).expect("Error")
        }
    }
}