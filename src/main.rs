use bevy::{
    prelude::*,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
};
use rand::{Rng, SeedableRng, random_range};
use rand_chacha::ChaCha8Rng;

fn main() {
    App::new().add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())).add_systems(Startup, setup).run();
}

#[derive(Component, Deref, DerefMut)]
struct UpdateTimer(Timer);

#[derive(Resource, Deref, DerefMut)]
struct SeededRng(ChaCha8Rng);
fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let chunk_size = UVec2::new(64, 64);
    let tile_display_size = UVec2::new(8, 8);
    let tile_data: Vec<Option<TileData>> =
        (0..chunk_size.element_product()).map(|_| random_range(0..5)).map(|i| if i == 0 { None } else { Some(TileData::from_tileset_index(i - 1)) }).collect();

    commands.spawn((
        TilemapChunk { chunk_size, tile_display_size, tileset: assets.load("spritesheet.png"), ..default() },
        TilemapChunkTileData(tile_data),
        UpdateTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));

    commands.spawn(Camera2d);

    commands.insert_resource(SeededRng(rng));
}
