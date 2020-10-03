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
