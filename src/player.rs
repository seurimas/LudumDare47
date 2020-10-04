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
    Moving {
        jump_impulse: f32,
        tx: f32,
        ty: f32,
    },
    Jumping {
        progress: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        z: f32,
    },
    Landing {
        ttl: f32,
    },
}
impl PlayerState {
    pub fn is_airborne(&self) -> bool {
        match self {
            PlayerState::Jumping { .. } => true,
            _ => false,
        }
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Player {
    pub move_speed: f32,
    pub jump_speed: f32,
    pub state: PlayerState,
    pub platform: Option<Entity>,
    pub on_edge: bool,
}

fn spawn_player(
    prefabs: &PrefabStorage,
    sprites: &SpriteStorage,
    player_builder: LazyBuilder,
    platform: Option<Entity>,
    platform_transform: na19::Vector3<f32>,
) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(
        platform_transform.x,
        platform_transform.y,
        platform_transform.y + 0.1,
    );
    player_builder
        .with(prefabs.player.clone())
        .with(transform)
        .with(Player {
            move_speed: 64.0,
            jump_speed: 4.0,
            state: PlayerState::Idle,
            platform: platform,
            on_edge: false,
        })
        .named("player")
        .build()
}

pub fn spawn_player_world(
    world: &mut World,
    platform: Option<Entity>,
    platform_transform: na19::Vector3<f32>,
) -> Entity {
    let entities = world.entities();
    let update = world.write_resource::<LazyUpdate>();
    let builder = update.create_entity(&entities);
    let prefabs = world.read_resource::<PrefabStorage>();
    let sprites = world.read_resource::<SpriteStorage>();
    let player = spawn_player(&prefabs, &sprites, builder, platform, platform_transform);
    let builder = update.create_entity(&entities);
    player
}

struct PlayerAnimationSystem;
impl<'s> System<'s> for PlayerAnimationSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        SimpleAnimationSystem<'s, SpriteRender>,
        Entities<'s>,
    );

    fn run(&mut self, (players, mut animator, entities): Self::SystemData) {
        for (player, entity) in (&players, &entities).join() {
            animator.start_if_idle(entity, AnimationId::Idle, EndControl::Loop(None), 1.0);
            match player.state {
                PlayerState::Moving { .. } => {
                    animator.start(entity, AnimationId::Move, EndControl::Normal, 1.0);
                }
                PlayerState::Jumping { .. } => {
                    animator.start(entity, AnimationId::Jump, EndControl::Stay, 1.0);
                }
                PlayerState::Landing { .. } => {
                    animator.start(entity, AnimationId::Land, EndControl::Stay, 1.0);
                }
                _ => {}
            }
        }
    }
}

fn jump_height(progress: f32) -> f32 {
    8. - 32. * (progress - 0.5) * (progress - 0.5)
}

struct PlayerJumpingSystem;
impl<'s> System<'s> for PlayerJumpingSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        ReadStorage<'s, Platform>,
        ReadStorage<'s, Parent>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
        Read<'s, StageState>,
        Entities<'s>,
        SoundPlayer<'s>,
    );
    fn run(
        &mut self,
        (mut players, platforms, parents, mut transforms, time, stage, entities, sound): Self::SystemData,
    ) {
        for (mut player, entity) in (&mut players, &entities).join() {
            match player.state {
                PlayerState::Idle => {}
                PlayerState::Landing { .. } => {}
                PlayerState::Jumping {
                    progress,
                    x1,
                    y1,
                    x2,
                    y2,
                    z,
                } => {
                    if let Some(mut player_loc) = transforms.get_mut(entity) {
                        player_loc.set_translation_xyz(
                            lerp(progress, x1, x2),
                            lerp(progress, y1, y2) + jump_height(progress),
                            z,
                        );
                        let new_progress = progress + (time.delta_seconds() * player.jump_speed);
                        if new_progress > 1.0 {
                            player.state = PlayerState::Landing { ttl: 0.1 };
                        } else {
                            player.state = PlayerState::Jumping {
                                progress: new_progress,
                                x1,
                                x2,
                                y1,
                                y2,
                                z,
                            };
                        }
                    }
                }
                PlayerState::Moving {
                    jump_impulse,
                    tx,
                    ty,
                } => {
                    if player.on_edge && jump_impulse > 0.2 {
                        if let Some(start) = transforms
                            .get(entity)
                            .map(|transform| transform.translation().clone())
                        {
                            if let Some(platform) =
                                player.platform.and_then(|entity| platforms.get(entity))
                            {
                                if let Some((target_platform, target_parent)) = stage
                                    .target_platform(*platform, tx, ty)
                                    .and_then(|platform_entity| {
                                        parents
                                            .get(*platform_entity)
                                            .map(|parent| (platform_entity, parent))
                                    })
                                {
                                    if let Some(end) = transforms
                                        .get(target_parent.entity)
                                        .map(|transform| transform.translation().clone())
                                    {
                                        player.state = PlayerState::Jumping {
                                            progress: 0.0,
                                            x1: start.x,
                                            x2: end.x,
                                            y1: start.y,
                                            y2: end.y,
                                            z: f32::min(start.z, end.z) + 0.1,
                                        };
                                        player.platform = Some(target_platform.clone());
                                        sound.play_normal(|store| &store.jump);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

struct PlayerPlatformingSystem;
impl<'s> System<'s> for PlayerPlatformingSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        ReadStorage<'s, Parent>,
        WriteStorage<'s, Transform>,
        Entities<'s>,
    );
    fn run(&mut self, (mut players, parents, mut transforms, entities): Self::SystemData) {
        if let Some((player, platform)) = {
            let mut ret = None;
            for (player, entity) in (&players, &entities).join() {
                if let Some(platform) = player
                    .platform
                    .and_then(|platform| parents.get(platform).map(|parent| parent.entity))
                {
                    ret = Some((entity, platform));
                }
            }
            ret
        } {
            if let Some(platform_loc) = transforms
                .get(platform)
                .map(|transform| transform.translation().clone())
            {
                if let Some(player_transform) = transforms.get_mut(player) {
                    let translation = player_transform.translation_mut();
                    tether_at(translation, &platform_loc, 4.0);
                    if distance_2d_iso(translation, &platform_loc) > 3.0 {
                        if let Some(mut player) = players.get_mut(player) {
                            player.on_edge = true;
                        }
                    }
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
                    PlayerState::Jumping { .. } => {}
                    PlayerState::Landing { ttl } => {
                        if ttl < time.delta_seconds() {
                            player.state = PlayerState::Idle;
                        } else {
                            player.state = PlayerState::Landing {
                                ttl: ttl - time.delta_seconds(),
                            };
                        }
                    }
                    _ => {
                        let mut translation = transform.translation_mut();
                        translation.x += x_tilt * player.move_speed * time.delta_seconds();
                        translation.y += y_tilt * player.move_speed * time.delta_seconds();
                        if f32::abs(x_tilt) > 0.0 || f32::abs(y_tilt) > 0.0 {
                            let (mut jump_impulse, old_tx, old_ty) = match player.state {
                                PlayerState::Moving {
                                    jump_impulse,
                                    tx,
                                    ty,
                                } => (jump_impulse, tx, ty),
                                _ => (0.0, 0.0, 0.0),
                            };
                            if f32::abs(x_tilt) > 0.0 && f32::abs(y_tilt) > 0.0 {
                                jump_impulse = 0.0;
                            }
                            player.state = PlayerState::Moving {
                                jump_impulse: jump_impulse + time.delta_seconds(),
                                tx: x_tilt,
                                ty: y_tilt,
                            };
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
        dispatcher.add(
            PlayerPlatformingSystem,
            "player_platforming",
            &["player_movement"],
        );
        dispatcher.add(
            PlayerJumpingSystem,
            "player_jumping",
            &["player_platforming"],
        );
        Ok(())
    }
}
