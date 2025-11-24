use bevy::{
    prelude::*,
    render::render_resource::Extent3d,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
    window::{EnabledButtons, PresentMode, WindowTheme},
};
use rand::{SeedableRng, random_range};
use rand_chacha::ChaCha8Rng;

const RES_WIDTH: u32 = 1920;
const RES_HEIGHT: u32 = 1024;
const TILE_WIDTH: u32 = 64;
const TILE_HEIGHT: u32 = 64;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()).set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::AutoVsync,
                resolution: (RES_WIDTH, RES_HEIGHT).into(),
                title: "kobolds_dungeon".to_string(),
                name: Some("kobolds_dungeon".to_string()),
                enabled_buttons: EnabledButtons { maximize: false, ..Default::default() },
                prevent_default_event_handling: false,
                window_theme: Some(WindowTheme::Dark),
                visible: true,
                ..default()
            }),
            ..default()
        })) // fallback to nearest sampling
        .init_state::<AppState>()
        .add_systems(OnEnter(AppState::Setup), load)
        .add_systems(Update, check_assets.run_if(in_state(AppState::Setup)))
        .add_systems(OnEnter(AppState::Finished), setup)
        .run();
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum AppState {
    #[default]
    Setup,
    Finished,
}

#[derive(Resource, Default)]
struct TerrainSprite(Handle<Image>);

fn _setup(mut commands: Commands, asset_server: Res<AssetServer>, mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>) {
    let texture: Handle<Image> = asset_server.load("spritesheet.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 9, 9, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    commands.spawn(Camera2d);

    commands.spawn((Sprite::from_atlas_image(texture, TextureAtlas { layout: texture_atlas_layout, index: 0 }), Transform::from_scale(Vec3::splat(1.0))));
}

fn load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(TerrainSprite(asset_server.load("spritesheet.png")));
}

fn check_assets(mut next_state: ResMut<NextState<AppState>>, terrain_sprite: Res<TerrainSprite>, mut events: MessageReader<AssetEvent<Image>>) {
    // Advance the `AppState` once all sprite handles have been loaded by the `AssetServer`
    for event in events.read() {
        if event.is_loaded_with_dependencies(&terrain_sprite.0) {
            next_state.set(AppState::Finished);
        }
    }
}

fn setup(mut commands: Commands, terrain_sprite: Res<TerrainSprite>, mut image: ResMut<Assets<Image>>) {
    let image = image.get_mut(terrain_sprite.0.id()).unwrap();
    apply_transformation(image, TILE_WIDTH, TILE_HEIGHT);
    //image.reinterpret_stacked_2d_as_array(81);
    let chunk_size = UVec2::new(RES_WIDTH / TILE_WIDTH, RES_HEIGHT / TILE_HEIGHT);
    let tile_display_size = UVec2::new(TILE_WIDTH, TILE_HEIGHT);
    let tile_data: Vec<Option<TileData>> = make_maze(chunk_size);
    //let tile_data: Vec<Option<TileData>> = vec![None; chunk_size.element_product() as usize];

    let tilemap_chunk = TilemapChunk { chunk_size, tile_display_size, tileset: terrain_sprite.0.clone(), alpha_mode: Default::default() };

    //commands.spawn(Sprite::from_image(terrain_sprite.0.clone()));
    //commands.spawn((
    //    Camera3d::default(),
    //    Projection::from(OrthographicProjection {
    //        scaling_mode: bevy::camera::ScalingMode::FixedVertical { viewport_height: 6.0 },
    //        ..OrthographicProjection::default_3d()
    //    }),
    //    Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    //));
    commands.spawn((
        Camera2d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical { viewport_height: 1200.0 },
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(5.0, 5.0, -5.0).looking_at(Vec3::ZERO, Vec3::Z),
    ));
    commands.spawn((tilemap_chunk, TilemapChunkTileData(tile_data)));
}

fn apply_transformation(image: &mut Image, tile_width: u32, tile_height: u32) {
    let width = image.width();
    let height = image.height();
    let rows = height / tile_height;
    let cols = width / tile_width;
    let pixels: &Vec<u8> = image.data.as_ref().unwrap();
    let mut buf: Vec<u8> = Vec::with_capacity(pixels.len());
    for r in 0..rows {
        for c in 0..cols {
            for w in 0..tile_width {
                for h in 0..tile_height {
                    for byte in 0..4 {
                        let pixel = pixels[(((r * tile_width + w) + (c * tile_height + h) * width) * 4 + byte) as usize];
                        buf.push(pixel);
                    }
                }
            }
        }
    }

    image.data = Some(buf);
    image.texture_descriptor.size = Extent3d { width: 64, height: 64, depth_or_array_layers: 81 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MazeTile(u8); //this is technically bad, since we only need u4 but that is not Rust-native

fn make_maze(chunk_size: UVec2) -> Vec<Option<TileData>> {
    let rows = chunk_size.x;
    let cols = chunk_size.y;
    let mut data: Vec<Option<TileData>> = Vec::with_capacity((rows * cols) as usize);
    let mut maze: Vec<(MazeTile, bool)> = Vec::with_capacity((rows * cols) as usize);
    let mut unvisited: Vec<IVec2> = Vec::with_capacity((rows * cols) as usize);
    let mut stack: Vec<IVec2> = Vec::with_capacity((rows * cols) as usize);
    //node
    //let mut stack: Vec<MazeTile> = Vec::new();
    for x in 0..rows as i32 {
        for y in 0..cols as i32 {
            maze.push((MazeTile(15), false));
            unvisited.push(ivec2(x, y));
        }
    }

    let mut current = unvisited.pop().unwrap();
    maze[to_index(&current, rows)].1 = true;
    while !unvisited.is_empty() {
        //println!("unvisited left: {}", unvisited.len());
        let neighbours = {
            let mut data = Vec::new();
            let directions = [ivec2(1, 0), ivec2(0, 1), ivec2(-1, 0), ivec2(0, -1)];
            for dir in directions {
                let position = current + dir;
                //skip if out of bounds
                if position.x < 0 || rows as i32 <= position.x || position.y < 0 || cols as i32 <= position.y {
                    continue;
                }

                //only add unvisited nodes
                if maze[to_index(&position, rows) as usize].1 == false {
                    data.push(position);
                }
            }
            data
        };
        //println!("neighbours: {}", neighbours.len());
        if !neighbours.is_empty() {
            let next = neighbours[random_range(0..neighbours.len())];
            stack.push(current);
            let dir = next - current;
            maze[to_index(&current, rows)].0.remove_wall(&dir);
            maze[to_index(&next, rows)].0.remove_wall(&-dir);
            current = next;
            maze[to_index(&current, rows)].1 = true;
            unvisited.retain(|&x| x != current);
        } else if !stack.is_empty() {
            current = stack.pop().unwrap();
        }
    }

    for tile in maze {
        data.push(Some(TileData { tileset_index: to_tileset_index(tile.0), ..default() }))
    }
    data
}

fn to_index(position: &IVec2, rows: u32) -> usize {
    assert!(position.x >= 0 && position.y >= 0, "position is negative! {}", position);
    (position.x + position.y * rows as i32) as usize
}

fn to_tileset_index(tile: MazeTile) -> u16 {
    match tile.0 {
        0b0000 => 47,
        0b0001 => 09,
        0b0010 => 38,
        0b0011 => 20,
        0b0100 => 64,
        0b0101 => 39,
        0b0110 => 63,
        0b0111 => 37,
        0b1000 => 46,
        0b1001 => 00,
        0b1010 => 54,
        0b1011 => 11,
        0b1100 => 55,
        0b1101 => 45,
        0b1110 => 03,
        0b1111 => 57,
        _ => unreachable!(),
    }
}

impl MazeTile {
    fn remove_wall(&mut self, dir: &IVec2) {
        assert!([ivec2(1, 0), ivec2(0, 1), ivec2(-1, 0), ivec2(0, -1)].contains(dir));
        self.0 -= match dir {
            IVec2 { x: -1, y: 0 } => 0b0001,
            IVec2 { x: 0, y: 1 } => 0b0010,
            IVec2 { x: 1, y: 0 } => 0b0100,
            IVec2 { x: 0, y: -1 } => 0b1000,
            _ => unreachable!(),
        }
    }
}
