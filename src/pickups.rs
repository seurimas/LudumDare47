use crate::prelude::*;
use amethyst::renderer::{palette::Srgba, resources::Tint};

#[derive(Component, Debug, Copy, Clone)]
#[storage(VecStorage)]
pub struct NotePickup {
    color: Option<Srgba>,
    platform: Entity,
}

impl NotePickup {
    pub fn new(platform: Entity) -> Self {
        NotePickup {
            color: None,
            platform,
        }
    }
}

pub struct NotePickupSystem;
impl<'s> System<'s> for NotePickupSystem {
    type SystemData = (
        WriteStorage<'s, NotePickup>,
        WriteStorage<'s, Player>,
        Entities<'s>,
    );
    fn run(&mut self, (notes, players, entities): Self::SystemData) {
        for (note, note_entity) in (&notes, &entities).join() {
            for (player) in (&players).join() {
                if Some(note.platform) == player.platform && !player.state.is_airborne() {
                    entities.delete(note_entity);
                }
            }
        }
    }
}

pub struct NoteAnimationSystem;
impl<'s> System<'s> for NoteAnimationSystem {
    type SystemData = (
        WriteStorage<'s, NotePickup>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, Tint>,
        ReadStorage<'s, AnimationSet<AnimationId, Transform>>,
        WriteStorage<'s, AnimationControlSet<AnimationId, Transform>>,
        Entities<'s>,
    );
    fn run(
        &mut self,
        (
            mut notes,
            mut sprites,
            mut tints,
            t_animation_sets,
            mut t_control_sets,
            entities,
        ): Self::SystemData,
    ) {
        for (mut note, t_animation_set, mut sprite, entity) in
            (&mut notes, &t_animation_sets, &mut sprites, &entities).join()
        {
            if let Some(t_control_set) = get_animation_set(&mut t_control_sets, entity) {
                if get_active_animation(t_control_set).is_none() {
                    set_active_animation(
                        t_control_set,
                        AnimationId::Beat,
                        &t_animation_set,
                        EndControl::Loop(None),
                        1.0,
                    );
                }
            }
            if note.color.is_none() {
                let color = rand_color();
                note.color = Some(color);
                tints.insert(entity, Tint(color));
                sprite.sprite_number = rand_upto(3);
            }
        }
    }
}
