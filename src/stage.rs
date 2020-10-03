use crate::prelude::*;
use amethyst::{
    animation::*,
    assets::Handle,
    core::{bundle::SystemBundle, transform::*},
    ecs::world::LazyBuilder,
    ecs::*,
    error::Error,
    input::{InputHandler, StringBindings},
    prelude::*,
    renderer::{camera::*, SpriteRender},
    tiles::{MortonEncoder, Tile, TileMap},
    window::ScreenDimensions,
};

const TILE_SIZE: u32 = 32;
const FLOOR_TILE: usize = 0;

pub fn initialize_camera(world: &mut World, dimensions: &ScreenDimensions) -> Entity {
    // Setup camera in a way that our screen covers whole arena and (0, 0) is in the bottom left.
    let mut transform = Transform::default();
    transform.set_translation_xyz(dimensions.width() * 0.5, dimensions.height() * 0.5, 1.);

    let entities = world.entities();
    let update = world.write_resource::<LazyUpdate>();
    let builder = update.create_entity(&entities);

    builder
        .with(Camera::standard_2d(dimensions.width(), dimensions.height()))
        .with(transform)
        .build()
}

#[derive(Debug, Clone)]
pub struct StageDescription {
    width: u32,
    height: u32,
}

impl Default for StageDescription {
    fn default() -> Self {
        StageDescription {
            width: 16,
            height: 12,
        }
    }
}

#[derive(Default, Clone)]
pub struct StageFloor;
impl Tile for StageFloor {
    fn sprite(&self, point: amethyst::core::math::Point3<u32>, world: &World) -> Option<usize> {
        let stage_desc = world.fetch::<StageDescription>();
        Some(0)
    }
}

pub fn initialize_stage(world: &mut World, stage_desc: StageDescription) {
    let tile_spritesheet = {
        let sprites = world.read_resource::<SpriteStorage>();
        sprites.tiles.clone()
    };
    let map_entity = world
        .create_entity()
        .with(TileMap::<StageFloor, MortonEncoder>::new(
            na19::Vector3::new(stage_desc.width, stage_desc.height, 1),
            na19::Vector3::new(TILE_SIZE, TILE_SIZE, 1),
            Some(tile_spritesheet),
        ))
        .with(Transform::default())
        .build();
    world.insert::<StageDescription>(stage_desc);
}
