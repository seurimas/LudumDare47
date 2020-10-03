use crate::hazards::Hazard;
use amethyst::{
    animation::*,
    assets::*,
    audio::{SourceHandle, WavFormat},
    derive::PrefabData,
    ecs::*,
    error::Error,
    prelude::*,
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
    sprite_scene: SpriteScenePrefab,
    animation_set: AnimationSetPrefab<AnimationId, SpriteRender>,
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
    pub spikes: Handle<Prefab<SpriteEntityPrefabData>>,
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
    pub ball: SpriteSheetHandle,
}

#[derive(Clone)]
pub struct SoundStorage {}

#[derive(Eq, PartialOrd, PartialEq, Hash, Debug, Copy, Clone, Deserialize, Serialize)]
pub enum AnimationId {
    //    Spawn,
    Idle,
    Move,
    //    Beat,
    //    Die,
    //    Hit,
    //    Kill,
}

pub type GameAssets = (SpriteStorage, PrefabStorage, SoundStorage);
