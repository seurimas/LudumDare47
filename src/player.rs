use crate::assets::{AnimationId, PrefabStorage, SpriteStorage};
use crate::prelude::*;
use amethyst::{
    animation::*,
    assets::Handle,
    core::{bundle::SystemBundle, timing::Time, transform::*},
    ecs::world::LazyBuilder,
    ecs::*,
    error::Error,
    input::{InputHandler, StringBindings},
    prelude::*,
    renderer::{camera::*, SpriteRender},
};

#[derive(Debug, PartialEq)]
pub enum PlayerState {
    Idle,
    Moving,
    Shielding,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Player {
    pub move_speed: f32,
    pub state: PlayerState,
}

fn spawn_player(
    prefabs: &PrefabStorage,
    sprites: &SpriteStorage,
    player_builder: LazyBuilder,
    x: f32,
    y: f32,
) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(x, y, 0.2);
    println!("{} {}", x, y);
    player_builder
        .with(prefabs.player.clone())
        .with(transform)
        .with(Player {
            move_speed: 256.0,
            state: PlayerState::Idle,
        })
        .named("player")
        .build()
}

pub fn spawn_player_world(world: &mut World, x: f32, y: f32) -> Entity {
    let entities = world.entities();
    let update = world.write_resource::<LazyUpdate>();
    let builder = update.create_entity(&entities);
    let prefabs = world.read_resource::<PrefabStorage>();
    let sprites = world.read_resource::<SpriteStorage>();
    let player = spawn_player(&prefabs, &sprites, builder, x, y);
    let builder = update.create_entity(&entities);
    player
}

struct PlayerAnimationSystem;
impl<'s> System<'s> for PlayerAnimationSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        ReadStorage<'s, AnimationSet<AnimationId, SpriteRender>>,
        WriteStorage<'s, AnimationControlSet<AnimationId, SpriteRender>>,
        Entities<'s>,
    );

    fn run(&mut self, (players, animation_sets, mut control_sets, entities): Self::SystemData) {
        for (player, animation_set, entity) in (&players, &animation_sets, &entities).join() {
            if let Some(control_set) = get_animation_set(&mut control_sets, entity) {
                if let Some(AnimationId::Idle) = get_active_animation(control_set) {
                    set_active_animation(
                        control_set,
                        AnimationId::Move,
                        &animation_set,
                        EndControl::Loop(None),
                        1.0,
                    );
                }
            }
        }
    }
}

struct PlayerMovementSystem;
impl<'s> System<'s> for PlayerMovementSystem {
    type SystemData = (
        Read<'s, InputHandler<StringBindings>>,
        WriteStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
        Entities<'s>,
    );
    fn run(&mut self, (input, mut player, mut transforms, time, entities): Self::SystemData) {
        let x_tilt = input.axis_value("leftright");
        let y_tilt = input.axis_value("updown");
        if let (Some(x_tilt), Some(y_tilt)) = (x_tilt, y_tilt) {
            for (mut player, mut transform) in (&mut player, &mut transforms).join() {
                match player.state {
                    PlayerState::Shielding => {}
                    _ => {
                        let mut translation = transform.translation_mut();
                        translation.x += x_tilt * player.move_speed * time.delta_seconds();
                        translation.y += y_tilt * player.move_speed * time.delta_seconds();
                        if f32::abs(x_tilt) > 0.0 || f32::abs(y_tilt) > 0.0 {
                            player.state = PlayerState::Moving;
                        } else {
                            player.state = PlayerState::Idle;
                        }
                    }
                }
            }
        }
    }
}

pub struct PlayerBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for PlayerBundle {
    fn build(
        self,
        _world: &mut World,
        dispatcher: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        dispatcher.add(PlayerAnimationSystem, "player_animation", &[]);
        dispatcher.add(PlayerMovementSystem, "player_movement", &[]);
        Ok(())
    }
}
