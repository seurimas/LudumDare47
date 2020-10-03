use crate::player::spawn_player_world;
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
    window::ScreenDimensions,
};
use std::collections::HashMap;

const TILE_SIZE: u32 = 32;
const TILE_CENTER: (u32, u32) = (0, 8);
const FLOOR_TILE: usize = 0;

pub fn initialize_camera(world: &mut World, dimensions: &ScreenDimensions) -> Entity {
    // Setup camera in a way that our screen covers whole arena and (0, 0) is in the bottom left.
    let mut transform = Transform::default();
    transform.set_translation_xyz(0.0, 0.0, 1.);

    let entities = world.entities();
    let update = world.write_resource::<LazyUpdate>();
    let builder = update.create_entity(&entities);

    builder
        .with(Camera::standard_2d(
            dimensions.width() / 2.0,
            dimensions.height() / 2.0,
        ))
        .with(transform)
        .build()
}

#[derive(Debug, Clone)]
pub struct StageDescription {
    width: u32,
    height: u32,
    player_spawn: (u32, u32),
}

#[derive(Component, Debug, Copy, Clone)]
#[storage(VecStorage)]
pub struct Platform {
    pub x: u32,
    pub y: u32,
    pub has_player: bool,
}

#[derive(Debug, Clone)]
pub struct StageState {
    platforms: HashMap<(u32, u32), Entity>,
}

impl Default for StageState {
    fn default() -> Self {
        StageState {
            platforms: HashMap::new(),
        }
    }
}

impl StageState {
    pub fn target_platform(&self, current: Platform, tx: f32, ty: f32) -> Option<&Entity> {
        let mut x = if tx > 0.0 {
            current.x + 1
        } else if tx < 0.0 {
            if current.x == 0 {
                return None;
            }
            current.x - 1
        } else {
            current.x
        };
        let mut y = if ty > 0.0 {
            current.y + 1
        } else if ty < 0.0 {
            if current.y == 0 {
                return None;
            }
            current.y - 1
        } else {
            current.y
        };
        self.platforms.get(&(x, y))
    }
}

impl Default for StageDescription {
    fn default() -> Self {
        StageDescription {
            width: 4,
            height: 4,
            player_spawn: (0, 0),
        }
    }
}

pub fn initialize_stage(world: &mut World, stage_desc: StageDescription) {
    let tile_spritesheet = {
        let sprites = world.read_resource::<SpriteStorage>();
        sprites.tiles.clone()
    };
    let mut platforms = HashMap::new();
    if let Some((player_spawn, translation)) = {
        let mut player_spawn_and_loc = None;
        let entities = world.entities();
        let update = world.write_resource::<LazyUpdate>();
        let prefabs = world.read_resource::<PrefabStorage>();
        for x in 0..stage_desc.width {
            for y in 0..stage_desc.height {
                let mut transform = Transform::default();
                transform.set_translation_xyz(
                    (x * TILE_SIZE) as f32,
                    (y * TILE_SIZE / 2) as f32,
                    y as f32 / -10.0,
                );
                let builder = update.create_entity(&entities);
                let translation = transform.translation().clone();
                let map_entity = builder
                    .with(prefabs.platform.clone())
                    .with(transform)
                    .with(Platform {
                        x,
                        y,
                        has_player: false,
                    })
                    .build();
                if stage_desc.player_spawn.0 == x && stage_desc.player_spawn.1 == y {
                    player_spawn_and_loc = Some((map_entity, translation));
                }
                platforms.insert((x, y), map_entity);
            }
        }
        player_spawn_and_loc
    } {
        spawn_player_world(world, Some(player_spawn), translation);
    }
    world.insert::<StageDescription>(stage_desc);
    world.insert::<StageState>(StageState { platforms });
}

struct PlatformAnimationSystem;
impl<'s> System<'s> for PlatformAnimationSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, Platform>,
        ReadStorage<'s, AnimationSet<AnimationId, SpriteRender>>,
        WriteStorage<'s, AnimationControlSet<AnimationId, SpriteRender>>,
        Entities<'s>,
        SoundPlayer<'s>,
    );

    fn run(
        &mut self,
        (players, mut platforms, animation_sets, mut control_sets, entities, sound): Self::SystemData,
    ) {
        for (platform, animation_set, entity) in (&mut platforms, &animation_sets, &entities).join()
        {
            let mut need_to_react = false;
            for (player) in (&players).join() {
                if !platform.has_player
                    && player.platform == Some(entity)
                    && !player.state.is_airborne()
                {
                    need_to_react = true;
                    platform.has_player = true;
                } else if platform.has_player && player.platform != Some(entity) {
                    platform.has_player = false;
                }
            }
            if need_to_react {
                println!("REACTING");
                sound.play_normal(|store| rand_in(&store.foo_scale));
                if let Some(control_set) = get_animation_set(&mut control_sets, entity) {
                    set_active_animation(
                        control_set,
                        AnimationId::Move,
                        &animation_set,
                        EndControl::Stay,
                        1.0,
                    );
                }
            }
        }
    }
}

pub struct StageBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for StageBundle {
    fn build(
        self,
        _world: &mut World,
        dispatcher: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        dispatcher.add(PlatformAnimationSystem, "platform_animation", &[]);
        Ok(())
    }
}
