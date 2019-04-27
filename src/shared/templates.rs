use crate::shared::generated::demo::{Action, ActionType, Headquarters, Lumberjack, Tree};
use crate::shared::generated::improbable::Vector3d;
use crate::shared::{CLIENT_LAYER, GAMELOGIC_LAYER};
use spatialos_sdk::worker::entity::Entity;
use spatialos_sdk::worker::entity_builder::EntityBuilder;

const TREE_RESOURCE_COUNT: u32 = 5;

pub fn tree(position: &Vector3d) -> Result<Entity, String> {
    let mut builder = EntityBuilder::new(position.x, position.y, position.z, GAMELOGIC_LAYER);
    builder.set_metadata("Tree", GAMELOGIC_LAYER);
    builder.set_persistent(GAMELOGIC_LAYER);
    builder.set_entity_acl_write_access(GAMELOGIC_LAYER);
    builder.add_read_access(CLIENT_LAYER);

    builder.add_component(
        Tree {
            resources_left: TREE_RESOURCE_COUNT,
        },
        GAMELOGIC_LAYER,
    );

    builder.build()
}

pub fn lumberjack(position: &Vector3d) -> Result<Entity, String> {
    let mut builder = EntityBuilder::new(position.x, position.y, position.z, GAMELOGIC_LAYER);
    builder.set_metadata("Lumberjack", GAMELOGIC_LAYER);
    builder.set_persistent(GAMELOGIC_LAYER);
    builder.set_entity_acl_write_access(GAMELOGIC_LAYER);
    builder.add_read_access(CLIENT_LAYER);

    builder.add_component(
        Lumberjack {
            action: Action {
                typ: ActionType::IDLE,
                target: None,
            },
        },
        GAMELOGIC_LAYER,
    );

    builder.build()
}

pub fn headquarters(position: &Vector3d) -> Result<Entity, String> {
    let mut builder = EntityBuilder::new(position.x, position.y, position.z, GAMELOGIC_LAYER);
    builder.set_metadata("Headquarters", GAMELOGIC_LAYER);
    builder.set_persistent(GAMELOGIC_LAYER);
    builder.set_entity_acl_write_access(GAMELOGIC_LAYER);
    builder.add_read_access(CLIENT_LAYER);

    builder.add_component(Headquarters { score: 0 }, GAMELOGIC_LAYER);

    builder.build()
}
