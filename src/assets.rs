use crate::stage::{MissIndicator, NoteIndicator};
use amethyst::{
    animation::*,
    assets::*,
    audio::{SourceHandle, WavFormat},
    core::Transform,
    derive::PrefabData,
    ecs::*,
    error::Error,
    renderer::{
        sprite::{prefab::SpriteScenePrefab, SpriteSheetHandle},
        types::Texture,
        ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat,
    },
    utils::{application_root_dir, scene::BasicScenePrefab},
};
use na::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, PrefabData)]
pub struct SpriteEntityPrefabData {
    sprite_scene: Option<SpriteScenePrefab>,
    animation_set: Option<AnimationSetPrefab<AnimationId, SpriteRender>>,
    transform_animation_set: Option<AnimationSetPrefab<AnimationId, Transform>>,
    miss_indicator: Option<MissIndicator>,
    note_indicator: Option<NoteIndicator>,
}

pub fn load_prefab<'a>(
    world: &mut World,
    path: String,
    progress: &'a mut ProgressCounter,
) -> Handle<Prefab<SpriteEntityPrefabData>> {
    world.exec(|loader: PrefabLoader<'_, SpriteEntityPrefabData>| {
        loader.load(path, RonFormat, progress)
    })
}

#[derive(Clone)]
pub struct PrefabStorage {
    pub player: Handle<Prefab<SpriteEntityPrefabData>>,
    pub notes: Handle<Prefab<SpriteEntityPrefabData>>,
    pub ball: Handle<Prefab<SpriteEntityPrefabData>>,
    pub shadow: Handle<Prefab<SpriteEntityPrefabData>>,
    pub platform: Handle<Prefab<SpriteEntityPrefabData>>,
    pub backdrop: Handle<Prefab<SpriteEntityPrefabData>>,
}

pub fn load_sound_file<'a>(
    world: &mut World,
    path: String,
    progress: &'a mut ProgressCounter,
) -> SourceHandle {
    let loader = world.read_resource::<Loader>();
    loader.load(path, WavFormat, (), &world.read_resource())
}

pub fn load_texture<'a>(
    world: &mut World,
    path: String,
    progress: &'a mut ProgressCounter,
) -> Handle<Texture> {
    let loader = world.read_resource::<Loader>();
    let texture_storage = world.read_resource::<AssetStorage<Texture>>();
    loader.load(path, ImageFormat::default(), progress, &texture_storage)
}
pub fn load_spritesheet<'a>(
    world: &mut World,
    path: String,
    progress: &'a mut ProgressCounter,
) -> SpriteSheetHandle {
    let texture_handle = load_texture(world, format!("{}.png", path), progress);
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        format!("{}.ron", path), // Here we load the associated ron file
        SpriteSheetFormat(texture_handle),
        progress,
        &sprite_sheet_store,
    )
}

#[derive(Clone)]
pub struct SpriteStorage {
    pub master: SpriteSheetHandle,
}

pub const NOTE_COUNT: usize = 20;
pub const SCALE: [&'static str; NOTE_COUNT] = [
    "c4", "c4s", "d4", "d4s", "e4", "f4", "f4s", "g4", "g4s", "a4", "a4s", "b4", "c5", "c5s", "d5",
    "d5s", "e5", "f5", "f5s", "g5",
];

#[derive(Clone)]
pub struct SoundStorage {
    pub jump: SourceHandle,
    pub miss: SourceHandle,
    pub tap: SourceHandle,
    pub foo_scale: Vec<SourceHandle>,
    pub note_scale: Vec<SourceHandle>,
}

#[derive(Eq, PartialOrd, PartialEq, Hash, Debug, Copy, Clone, Deserialize, Serialize)]
pub enum AnimationId {
    Spawn,
    Idle,
    Move,
    Jump,
    Beat,
    Die,
    Hit,
    Land,
    //    Kill,
}

pub type GameAssets = (SpriteStorage, PrefabStorage, SoundStorage);
