use crate::music::*;
use crate::pickups::*;
use crate::player::*;
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
    TILE_SIZE as f32 * 5., // 160
    100. + // Dropsize
    24. + // Low board
    TILE_SIZE as f32 * 2.5, // 204
);

pub fn initialize_camera(world: &mut World, dimensions: &ScreenDimensions) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(
        STAGE_SIZE.0 / 2. - TILE_SIZE as f32 / 2.0,
        STAGE_SIZE.1 / 2. - TILE_SIZE as f32 / 2.0 - 24.,
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
    pub dead: bool,
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
}

#[derive(Debug, Clone)]
pub struct StageState {
    platforms: HashMap<(u32, u32), Entity>,
    time_in_song: f32,
    missed: i32,
    pub notes_found: Vec<Note>,
    pub winning: bool,
    pub losing: bool,
    pub playing: bool,
    song: Song,
    songs: [Song; SONG_COUNT],
    song_index: i32,
}

impl Default for StageState {
    fn default() -> Self {
        StageState::new(HashMap::new(), Song::alouette())
    }
}

impl StageState {
    pub fn new(platforms: HashMap<(u32, u32), Entity>, song: Song) -> Self {
        StageState {
            platforms,
            time_in_song: -4.0,
            missed: 0,
            notes_found: Vec::new(),
            winning: false,
            losing: false,
            playing: false,
            song,
            songs: Song::songs(),
            song_index: 0,
        }
    }
    pub fn get_spawn(&self) -> Option<&Entity> {
        self.platforms.get(&(2, 2))
    }
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

    pub fn win(&mut self) {
        self.song = Song::payout_song(&self.notes_found);
        self.time_in_song = -0.5;
        self.missed = 0;
        self.notes_found = Vec::new();
        self.winning = true;
        self.song_index += 1;
    }

    pub fn reset(&mut self) {
        self.song_index = 0;
        self.missed = 0;
        self.notes_found = Vec::new();
        self.song_index = 0;
        self.playing = false;
        self.losing = false;
        self.winning = false;
    }

    pub fn lose(&mut self) {
        self.song = Song::lose_song();
        self.time_in_song = -0.5;
        self.missed = 3;
        self.losing = true;
    }

    pub fn start_new_song(&mut self) {
        self.song = self.songs[(self.song_index as usize) % SONG_COUNT].clone();
        self.time_in_song = -4.0;
        self.playing = true;
        self.winning = false;
        self.losing = false;
        self.missed = 0;
        self.notes_found = Vec::new();
    }

    fn beat(&self) -> i32 {
        (SUBNOTES as f32 * self.time_in_song * ((self.song.bpm as f32) / 60.0)) as i32
    }
}

impl Default for StageDescription {
    fn default() -> Self {
        StageDescription {
            width: 5,
            height: 4,
            player_spawn: (0, 0),
        }
    }
}

fn note_at(x: u32, y: u32) -> Note {
    (x + y * 5) as Note
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
                        dead: true,
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
        spawn_player_world(world);
    }
    world.insert::<StageDescription>(stage_desc);
    world.insert::<StageState>(StageState::new(platforms, Song::alouette()));
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
        Write<'s, StageState>,
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
            mut stage_state,
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
            if !stage_state.losing && platform.dead {
                platform.dead = false;
                if let (Some(control_set), Some(t_control_set)) = (
                    get_animation_set(&mut control_sets, entity),
                    get_animation_set(&mut t_control_sets, entity),
                ) {
                    set_active_animation(
                        control_set,
                        AnimationId::Spawn,
                        &animation_set,
                        EndControl::Stay,
                        1.0,
                    );
                    set_active_animation(
                        t_control_set,
                        AnimationId::Spawn,
                        &t_animation_set,
                        EndControl::Stay,
                        1.0,
                    );
                }
            }
            if need_to_wobble || need_to_play {
                if need_to_wobble {
                    sound.play_normal(|store| &store.tap);
                } else if platform.has_player && !stage_state.winning && !stage_state.losing {
                    sound.play_normal(|store| &store.miss);
                    stage_state.missed += 1;
                } else {
                    if stage_state.losing {
                        platform.dead = true;
                        if let (Some(control_set), Some(t_control_set)) = (
                            get_animation_set(&mut control_sets, entity),
                            get_animation_set(&mut t_control_sets, entity),
                        ) {
                            set_active_animation(
                                control_set,
                                AnimationId::Die,
                                &animation_set,
                                EndControl::Loop(None),
                                1.0,
                            );
                            set_active_animation(
                                t_control_set,
                                AnimationId::Die,
                                &t_animation_set,
                                EndControl::Stay,
                                1.0,
                            );
                        }
                    }
                    if !stage_state.winning {
                        sound.play_normal(|store| {
                            store
                                .foo_scale
                                .get(platform.note as usize)
                                .expect("Missing note")
                        });
                    } else {
                        sound.play_normal(|store| {
                            store
                                .note_scale
                                .get(platform.note as usize)
                                .expect("Missing note")
                        });
                    }
                }
                if !stage_state.losing {
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
        if !stage_state.playing {
            return;
        }
        let last_time = stage_state.time_in_song;
        stage_state.time_in_song += time.delta_seconds();
        let last_beat = last_time * ((stage_state.song.bpm as f32) / 60.0);
        let new_beat = last_beat + (time.delta_seconds() * (stage_state.song.bpm as f32) / 60.0);
        let last_sub_beat = (last_beat * SUBNOTES as f32) as i32;
        let new_sub_beat = (new_beat * SUBNOTES as f32) as i32;
        if new_sub_beat > last_sub_beat && new_sub_beat >= 0 {
            for note in stage_state.song.get_notes_at(new_sub_beat) {
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
                if stage_state.winning {
                    stage_state
                        .notes_found
                        .retain(|found_note| *found_note != note);
                }
            }
            if !stage_state.winning {
                for note in stage_state
                    .song
                    .get_rewards_at(new_sub_beat, &stage_state.notes_found)
                {
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
                                    builder
                                        .with(note_transform)
                                        .with(NotePickup::new(entity, note, 5.))
                                },
                            );
                        }
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
        dispatcher.add(PlayerMissSystem, "player_miss", &[]);
        dispatcher.add(PlayerNoteIndicatorSystem, "player_notes", &[]);
        dispatcher.add(PlayerWinSystem, "player_win", &[]);
        Ok(())
    }
}

struct PlayerWinSystem;
impl<'s> System<'s> for PlayerWinSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        WriteStorage<'s, Platform>,
        WriteStorage<'s, Transform>,
        Write<'s, StageState>,
        SoundPlayer<'s>,
        PrefabSpawner<'s>,
    );

    fn run(
        &mut self,
        (mut players, mut platforms, mut transforms, mut stage_state, sound, spawner): Self::SystemData,
    ) {
        if stage_state.notes_found.len() == 8 && !stage_state.winning {
            stage_state.win();
        } else if stage_state.missed >= 3 && !stage_state.losing {
            stage_state.lose();
            for (player) in (&mut players).join() {
                player.state = PlayerState::Dying { ttd: 0.3 };
            }
        } else if stage_state.winning && stage_state.song.done(stage_state.beat()) {
            stage_state.start_new_song();
        } else if stage_state.losing && stage_state.song.done(stage_state.beat()) {
            for (player) in (&mut players).join() {
                match player.state {
                    PlayerState::Respawning { .. } => {}
                    PlayerState::Waiting { .. } => {
                        let mut all_dead = true;
                        for (platform) in (&platforms).join() {
                            if !platform.dead {
                                all_dead = false;
                            }
                        }
                        if all_dead {
                            stage_state.reset();
                        }
                    }
                    _ => {}
                }
            }
        } else if !stage_state.playing {
            for (player, transform) in (&mut players, &mut transforms).join() {
                match player.state {
                    PlayerState::Waiting { .. } => {}
                    _ => {
                        println!("{:?}", player.state);
                        stage_state.start_new_song();
                    }
                }
            }
        } else if stage_state.losing || !stage_state.playing {
            for (player, transform) in (&mut players, &mut transforms).join() {
                match player.state {
                    PlayerState::Respawning { .. } => {
                        transform.set_translation_xyz(0., -24., 100.);
                        player.platform = None;
                    }
                    _ => {}
                }
            }
        }
    }
}

#[derive(Component, Debug, PrefabData, Clone, Deserialize, Serialize)]
#[prefab(Component)]
#[storage(VecStorage)]
pub struct MissIndicator(i32);

struct PlayerMissSystem;
impl<'s> System<'s> for PlayerMissSystem {
    type SystemData = (
        ReadStorage<'s, MissIndicator>,
        WriteStorage<'s, SpriteRender>,
        Write<'s, StageState>,
        SoundPlayer<'s>,
    );

    fn run(&mut self, (misses, mut sprites, stage_state, sound): Self::SystemData) {
        for (miss_num, mut sprite) in (&misses, &mut sprites).join() {
            if miss_num.0 <= stage_state.missed {
                sprite.sprite_number = 5;
            } else {
                sprite.sprite_number = 4;
            }
        }
    }
}

#[derive(Component, Debug, PrefabData, Clone, Deserialize, Serialize)]
#[prefab(Component)]
#[storage(VecStorage)]
pub struct NoteIndicator(i32);

struct PlayerNoteIndicatorSystem;
impl<'s> System<'s> for PlayerNoteIndicatorSystem {
    type SystemData = (
        ReadStorage<'s, NoteIndicator>,
        WriteStorage<'s, Tint>,
        Write<'s, StageState>,
        Entities<'s>,
        SoundPlayer<'s>,
    );

    fn run(
        &mut self,
        (note_indicators, mut tints, stage_state, entities, sound): Self::SystemData,
    ) {
        for (note_indicator, entity) in (&note_indicators, &entities).join() {
            if note_indicator.0 < stage_state.notes_found.len() as i32 {
                let note = stage_state
                    .notes_found
                    .get(note_indicator.0 as usize)
                    .expect("Missing note indicator");
                tints.insert(entity, Tint(note_color(*note)));
            } else {
                tints.insert(entity, Tint(black()));
            }
        }
    }
}
