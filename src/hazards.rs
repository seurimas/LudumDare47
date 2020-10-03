use crate::assets::{AnimationId, PrefabStorage, SpriteStorage};
use crate::prelude::*;
use amethyst::{assets::*, derive::PrefabData, prelude::*};

#[derive(Debug, PartialEq)]
pub enum HazardType {
    Spikes,
    Laser,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Hazard {
    hazard_type: HazardType,
    activated: bool,
}

impl Hazard {
    pub fn activation_speed(&self) -> f32 {
        match self.hazard_type {
            HazardType::Spikes => 4.0,
            HazardType::Laser => 4.0,
        }
    }
}

fn spawn_spikes(
    prefabs: &PrefabStorage,
    sprites: &SpriteStorage,
    player_builder: LazyBuilder,
    x: f32,
    y: f32,
) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_xyz(x, y, 0.0);
    player_builder
        //.with(prefabs.spikes.clone())
        .with(transform)
        .with(Hazard {
            hazard_type: HazardType::Spikes,
            activated: false,
        })
        .build()
}

pub fn spawn_spikes_world(world: &mut World, x: f32, y: f32) -> Entity {
    let entities = world.entities();
    let update = world.write_resource::<LazyUpdate>();
    let builder = update.create_entity(&entities);
    let prefabs = world.read_resource::<PrefabStorage>();
    let sprites = world.read_resource::<SpriteStorage>();
    let spikes = spawn_spikes(&prefabs, &sprites, builder, x, y);
    let builder = update.create_entity(&entities);
    spikes
}

struct HazardActivationSystem;
impl<'s> System<'s> for HazardActivationSystem {
    type SystemData = (
        WriteStorage<'s, Hazard>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
        Entities<'s>,
    );
    fn run(&mut self, (mut hazard, mut transforms, time, entities): Self::SystemData) {}
}

struct HazardAnimationSystem;
impl<'s> System<'s> for HazardAnimationSystem {
    type SystemData = (
        WriteStorage<'s, Hazard>,
        ReadStorage<'s, AnimationSet<AnimationId, SpriteRender>>,
        WriteStorage<'s, AnimationControlSet<AnimationId, SpriteRender>>,
        Entities<'s>,
    );

    fn run(&mut self, (mut hazards, animation_sets, mut control_sets, entities): Self::SystemData) {
        for (mut hazard, animation_set, entity) in (&mut hazards, &animation_sets, &entities).join()
        {
            if hazard.activated {
                continue;
            } else {
                hazard.activated = true;
            }
            if let Some(control_set) = get_animation_set(&mut control_sets, entity) {
                set_active_animation(
                    control_set,
                    AnimationId::Move,
                    &animation_set,
                    EndControl::Stay,
                    hazard.activation_speed(),
                );
            }
        }
    }
}

pub struct HazardsBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for HazardsBundle {
    fn build(
        self,
        _world: &mut World,
        dispatcher: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        dispatcher.add(HazardAnimationSystem, "hazard_animate", &[]);
        dispatcher.add(HazardActivationSystem, "hazard_activate", &[]);
        Ok(())
    }
}
