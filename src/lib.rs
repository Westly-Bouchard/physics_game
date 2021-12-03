use bevy::prelude::*;
use wasm_bindgen::prelude::*;
use bevy::render::pass::ClearColor;
// Retrograde appears to have broken itself so I guess we won't be using it
// use bevy_retrograde::prelude::*;

// Actual resolution of the window will be the default 1280 * 720 but I think it's best to limit ourselves to 480 * 270
// This means we need scaling factors... thankfully, these resolutions are both 16:9, so they can be constantly defined:
#[allow(dead_code)]
const SCALING_FACTOR: f64 = 1280. / 480.;

//Helper function to convert from game coords to rendering coords, could be a dumb way to implement it idk
#[allow(dead_code)]
fn to_screenspace(x: i32, y: i32) -> (f64, f64) {
    (x as f64 * SCALING_FACTOR, y as f64 * SCALING_FACTOR)
}

//Label for the setup stage, as we can not load the assets in the startup stage because it spawns the sprites before the
//assets are loaded
static SETUP: &str = "setup";


#[wasm_bindgen]
pub fn run() {
    let mut app = App::build();
    app.insert_resource(WindowDescriptor {
        width: 1280.,
        height: 720.,
        ..Default::default()
    });

    app.add_plugins(DefaultPlugins);

    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);

    app.init_resource::<Materials>();
    
    app.insert_resource(ClearColor(Color::rgb(1., 1., 1.)));
    
    app.add_startup_system(setup.system().label("setup"));

    app.add_startup_stage(SETUP, SystemStage::single_threaded()
        .with_system_set(SystemSet::new()
            .with_system(spawn_map.system())
            .with_system(spawn_player.system())));

    app.add_system(player_movement.system());

    app.run();
}

#[derive(Default)]
struct Materials {
    player_material: Handle<ColorMaterial>,
    map: Handle<ColorMaterial>
}


struct Player;

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(Materials {
        player_material: materials.add(Color::rgb(0., 0., 1.).into()),
        map: materials.add(Color::rgb(0., 0., 0.,).into())
    });
}

fn spawn_map(mut commands: Commands, materials: Res<Materials>) {
    //Far Left
    commands.spawn_bundle(good_box_to_bad_box(&materials, -630, 350, -610, -350));

    //Far right
    commands.spawn_bundle(good_box_to_bad_box(&materials, 610, 350, 630, -350));

    //Top
    commands.spawn_bundle(good_box_to_bad_box(&materials, -494, 350, 610, 330));

    //Bottom
    // commands.spawn_bundle(SpriteBundle {
    //     material: materials.map.clone(),
    //     transform: Transform::from_xyz(-106., -340., 0.),
    //     sprite: Sprite::new(Vec2::new(1134., thickness)),
    //     ..Default::default()
    // });

    commands.spawn_bundle(good_box_to_bad_box(&materials, -610, -350, 494, -320));

    // commands.spawn_bundle(SpriteBundle {
    //     material: materials.map.clone(),
    //     transform: Transform::from_xyz(0., -280., 0.),
    //     sprite: Sprite::new(Vec2::new(100., thickness)),
    //     ..Default::default()
    // });

    
}

fn good_box_to_bad_box(materials: &Res<Materials>, x1: i32, y1: i32, x2: i32, y2: i32) -> SpriteBundle {
    SpriteBundle {
        material: materials.map.clone(),
        transform: Transform::from_xyz((x1 + x2) as f32 / 2., (y1 + y2) as f32 / 2., 0.),
        sprite: Sprite::new(Vec2::new((x2 - x1) as f32, (y2 - y1) as f32)),
        ..Default::default()
    }
}

fn spawn_player(mut commands: Commands, materials: Res<Materials>) {
    commands.spawn_bundle(SpriteBundle {
        material: materials.player_material.clone(),
        sprite: Sprite::new(Vec2::new(10., 10.)),
        ..Default::default()
    }).insert(Player);
}

fn player_movement(keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut Transform, With<Player>>) {
    let x_movement = 2. * if keyboard_input.pressed(KeyCode::Left) {
        -1.
    } else if keyboard_input.pressed(KeyCode::Right) {
        1.
    } else {
        0.
    };
    let y_movement = 2. * if keyboard_input.pressed(KeyCode::Down) {
        -1.
    } else if keyboard_input.pressed(KeyCode::Up) {
        1.
    } else {
        0.
    };

    for mut transform in query.iter_mut() {
        transform.translation.x += x_movement;
        transform.translation.y += y_movement;
    }
}