use crate::music::*;
use crate::pickups::*;
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
const STAGE_SIZE: (f32, f32) = (
    TILE_SIZE as f32 * 5.,
    100. + // Dropsize
    TILE_SIZE as f32 * 2.5,
);

pub fn initialize_camera(world: &mut World, dimensions: &ScreenDimensions) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(
        STAGE_SIZE.0 / 2. - TILE_SIZE as f32 / 2.0,
        STAGE_SIZE.1 / 2. - TILE_SIZE as f32 / 2.0,
        200.,
    );

    let entities = world.entities();
    let update = world.write_resource::<LazyUpdate>();
    let builder = update.create_entity(&entities);

    builder
        .with(Camera::standard_2d(STAGE_SIZE.0, STAGE_SIZE.1))
        .with(transform)
        .build()
}

#[derive(Component, Debug, Copy, Clone)]
#[storage(VecStorage)]
pub struct Platform {
    pub x: u32,
    pub y: u32,
    pub has_player: bool,
    pub note: Note,
}

#[derive(Component, Debug, Copy, Clone)]
#[storage(VecStorage)]
pub struct Ball {
    ttl: f32,
    ttd: f32,
    drop_speed: f32,
    hit: bool,
    platform: Entity,
}

impl Ball {
    fn new(platform: Entity) -> Self {
        Ball {
            ttl: 1.2,
            ttd: 1.0,
            drop_speed: 100.0,
            hit: false,
            platform,
        }
    }
}

#[derive(Component, Debug, Copy, Clone)]
#[storage(VecStorage)]
pub struct Shadow {
    ttl: f32,
}

impl Default for Shadow {
    fn default() -> Self {
        Shadow { ttl: 1.0 }
    }
}

#[derive(Debug, Clone)]
pub struct StageDescription {
    width: u32,
    height: u32,
    player_spawn: (u32, u32),
    song: Song,
}

#[derive(Debug, Clone)]
pub struct StageState {
    platforms: HashMap<(u32, u32), Entity>,
    time_in_song: f32,
}

impl Default for StageState {
    fn default() -> Self {
        StageState {
            platforms: HashMap::new(),
            time_in_song: -4.0,
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
            width: 5,
            height: 4,
            player_spawn: (0, 0),
            song: Song::default(),
        }
    }
}

fn note_at(x: u32, y: u32) -> Note {
    (x + y * 4) as Note
}

fn spawn_flags(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(
        (3 * TILE_SIZE) as f32,
        (3 * TILE_SIZE / 2) as f32 + 100.0,
        0.0,
    );
    world.exec(|spawner: PrefabSpawner| {
        spawner.spawn_decor(transform, |sprites| &sprites.master, 2, |builder| builder)
    });
}

fn spawn_backdrop(world: &mut World, x: u32) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(
        (x * TILE_SIZE) as f32,
        (2 * TILE_SIZE / 2) as f32 + 100.0,
        -200.0,
    );
    world.exec(|spawner: PrefabSpawner| {
        spawner.spawn_decor(transform, |sprites| &sprites.master, 1, |builder| builder)
    });
}

fn spawn_chute(world: &mut World, x: u32, y: u32) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(
        (x * TILE_SIZE) as f32,
        (y * TILE_SIZE / 2) as f32 + 100.0,
        y as f32 / 10.0 - 100.0,
    );
    world.exec(|spawner: PrefabSpawner| {
        spawner.spawn_decor(
            transform,
            |sprites| &sprites.master,
            0,
            |builder| builder.with(Tint(note_color(note_at(x, y)))),
        )
    });
}

pub fn initialize_stage(world: &mut World, stage_desc: StageDescription) {
    let mut platforms = HashMap::new();
    if let Some((player_spawn, translation)) = {
        world.exec(|spawner: PrefabSpawner| {
            spawner.spawn_prefab(|prefabs| &prefabs.backdrop, |builder| builder);
        });
        for x in 0..stage_desc.width {
            for y in 0..stage_desc.height {
                spawn_chute(world, x, y);
            }
        }
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
                let parent_entity = builder.with(transform).build();
                let builder = update.create_entity(&entities);
                let sprite_entity = builder
                    .with(prefabs.platform.clone())
                    .with(Parent {
                        entity: parent_entity,
                    })
                    .with(Platform {
                        x,
                        y,
                        has_player: false,
                        note: note_at(x, y) as Note,
                    })
                    .build();
                if stage_desc.player_spawn.0 == x && stage_desc.player_spawn.1 == y {
                    player_spawn_and_loc = Some((sprite_entity, translation));
                }
                platforms.insert((x, y), sprite_entity);
            }
        }
        player_spawn_and_loc
    } {
        spawn_player_world(world, Some(player_spawn), translation);
    }
    world.insert::<StageDescription>(stage_desc);
    world.insert::<StageState>(StageState {
        platforms,
        time_in_song: -4.0,
    });
}

struct PlatformAnimationSystem;
impl<'s> System<'s> for PlatformAnimationSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, Shadow>,
        WriteStorage<'s, Ball>,
        WriteStorage<'s, Platform>,
        ReadStorage<'s, AnimationSet<AnimationId, SpriteRender>>,
        WriteStorage<'s, AnimationControlSet<AnimationId, SpriteRender>>,
        ReadStorage<'s, AnimationSet<AnimationId, Transform>>,
        WriteStorage<'s, AnimationControlSet<AnimationId, Transform>>,
        Entities<'s>,
        SoundPlayer<'s>,
    );

    fn run(
        &mut self,
        (
            players,
            mut shadows,
            mut balls,
            mut platforms,
            animation_sets,
            mut control_sets,
            t_animation_sets,
            mut t_control_sets,
            entities,
            sound,
        ): Self::SystemData,
    ) {
        for (platform, animation_set, t_animation_set, entity) in (
            &mut platforms,
            &animation_sets,
            &t_animation_sets,
            &entities,
        )
            .join()
        {
            let mut need_to_wobble = false;
            for (player) in (&players).join() {
                if !platform.has_player
                    && player.platform == Some(entity)
                    && !player.state.is_airborne()
                {
                    need_to_wobble = true;
                    platform.has_player = true;
                } else if platform.has_player && player.platform != Some(entity) {
                    platform.has_player = false;
                }
            }
            let mut need_to_play = false;
            for (mut ball) in (&mut balls).join() {
                if ball.platform == entity && !ball.hit && ball.ttd <= 0.0 {
                    need_to_play = true;
                    ball.hit = true;
                }
            }
            if need_to_wobble || need_to_play {
                if need_to_wobble {
                    sound.play_normal(|store| &store.tap);
                } else {
                    sound.play_normal(|store| {
                        store
                            .foo_scale
                            .get(platform.note as usize)
                            .expect("Missing note")
                    });
                }
                if let (Some(control_set), Some(t_control_set)) = (
                    get_animation_set(&mut control_sets, entity),
                    get_animation_set(&mut t_control_sets, entity),
                ) {
                    set_active_animation(
                        control_set,
                        AnimationId::Move,
                        &animation_set,
                        EndControl::Stay,
                        1.0,
                    );
                    set_active_animation(
                        t_control_set,
                        AnimationId::Move,
                        &t_animation_set,
                        EndControl::Stay,
                        1.0,
                    );
                }
            }
        }
    }
}

struct BallDropperSystem;
impl<'s> System<'s> for BallDropperSystem {
    type SystemData = (
        WriteStorage<'s, Ball>,
        WriteStorage<'s, Shadow>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, AnimationSet<AnimationId, SpriteRender>>,
        WriteStorage<'s, AnimationControlSet<AnimationId, SpriteRender>>,
        ReadStorage<'s, AnimationSet<AnimationId, Transform>>,
        WriteStorage<'s, AnimationControlSet<AnimationId, Transform>>,
        Entities<'s>,
        Read<'s, Time>,
    );
    fn run(
        &mut self,
        (
            mut balls,
            mut shadows,
            mut transforms,
            animation_sets,
            mut control_sets,
            t_animation_sets,
            mut t_control_sets,
            entities,
            time,
        ): Self::SystemData,
    ) {
        for (mut ball, animation_set, mut transform, entity) in
            (&mut balls, &animation_sets, &mut transforms, &entities).join()
        {
            if let Some(control_set) = get_animation_set(&mut control_sets, entity) {
                if get_active_animation(control_set).is_none() {
                    set_active_animation(
                        control_set,
                        AnimationId::Idle,
                        &animation_set,
                        EndControl::Stay,
                        1.0,
                    );
                }
            }
            if ball.ttd > 0.0 {
                transform.translation_mut().y -= ball.drop_speed * time.delta_seconds();
            }
            ball.ttd -= time.delta_seconds();
            ball.ttl -= time.delta_seconds();
            if ball.ttl < 0.0 {
                entities.delete(entity);
            }
        }
        for (mut shadow, animation_set, entity) in (&mut shadows, &animation_sets, &entities).join()
        {
            if let Some(control_set) = get_animation_set(&mut control_sets, entity) {
                if get_active_animation(control_set).is_none() {
                    set_active_animation(
                        control_set,
                        AnimationId::Idle,
                        &animation_set,
                        EndControl::Stay,
                        1.0,
                    );
                }
            }
            shadow.ttl -= time.delta_seconds();
            if shadow.ttl < 0.0 {
                entities.delete(entity);
            }
        }
    }
}

struct PlatformBeatSystem;
impl<'s> System<'s> for PlatformBeatSystem {
    type SystemData = (
        WriteStorage<'s, Platform>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, Transform>,
        Read<'s, StageDescription>,
        Write<'s, StageState>,
        Read<'s, Time>,
        PrefabSpawner<'s>,
        SoundPlayer<'s>,
    );

    fn run(
        &mut self,
        (mut platforms, parents, transforms, stage_desc, mut stage_state, time, spawner, sound): Self::SystemData,
    ) {
        let song = &stage_desc.song;
        let last_time = stage_state.time_in_song;
        stage_state.time_in_song += time.delta_seconds();
        let last_beat = last_time * ((song.bpm as f32) / 60.0);
        let new_beat = last_beat + (time.delta_seconds() * (song.bpm as f32) / 60.0);
        let last_sub_beat = (last_beat * SUBNOTES as f32) as i32;
        let new_sub_beat = (new_beat * SUBNOTES as f32) as i32;
        if new_sub_beat > last_sub_beat && new_sub_beat >= 0 {
            for note in song.get_notes_at(new_sub_beat) {
                for (platform, entity) in (&platforms, &spawner.entities).join() {
                    if platform.note as usize == note {
                        let mut ball_transform = Transform::default();
                        let mut shadow_transform = Transform::default();
                        if let Some(transform) = parents
                            .get(entity)
                            .and_then(|parent| transforms.get(parent.entity))
                        {
                            ball_transform.set_translation_xyz(
                                transform.translation().x,
                                transform.translation().y + 100.0,
                                transform.translation().z + 0.01,
                            );
                            shadow_transform.set_translation_xyz(
                                transform.translation().x,
                                transform.translation().y,
                                transform.translation().z + 0.01,
                            );
                        }
                        spawner.spawn_prefab(
                            |prefabs| &prefabs.shadow,
                            move |builder| builder.with(shadow_transform).with(Shadow::default()),
                        );
                        spawner.spawn_prefab(
                            |prefabs| &prefabs.ball,
                            move |builder| builder.with(ball_transform).with(Ball::new(entity)),
                        );
                    }
                }
            }
            for note in song.get_rewards_at(new_sub_beat) {
                for (platform, entity) in (&platforms, &spawner.entities).join() {
                    if platform.note as usize == note {
                        let mut note_transform = Transform::default();
                        if let Some(transform) = parents
                            .get(entity)
                            .and_then(|parent| transforms.get(parent.entity))
                        {
                            note_transform.set_translation_xyz(
                                transform.translation().x,
                                transform.translation().y,
                                transform.translation().z + 0.01,
                            );
                        }
                        spawner.spawn_prefab(
                            |prefabs| &prefabs.notes,
                            move |builder| {
                                builder.with(note_transform).with(NotePickup::new(entity))
                            },
                        );
                    }
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
        dispatcher.add(PlatformBeatSystem, "platform_beat", &[]);
        dispatcher.add(BallDropperSystem, "ball_dropper", &[]);
        dispatcher.add(NoteAnimationSystem, "note_animation", &[]);
        dispatcher.add(NotePickupSystem, "note_pickup", &[]);
        Ok(())
    }
}
