use bevy::prelude::*;
use wasm_bindgen::prelude::*;
//Retrograde appears to have broken itself so I guess we won't be using it
// use bevy_retrograde::prelude::*;

//Actual resolution of the window will be the default 1280 * 720 but I think it's best to limit ourselves to 480 * 270
//This means we need scaling factors... thankfully, these resolutions are both 16:9, so they can be constantly defined:
const SCALING_FACTOR: f64 = 1280. / 480.;

//Helper function to convert from game coords to rendering coords, could be a dumb way to implement it idk
fn to_screenspace(x: i32, y: i32) -> (f64, f64) {
    (x as f64 * SCALING_FACTOR, y as f64 * SCALING_FACTOR)
}


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
    
    app.add_startup_system(setup.system());
    app.add_startup_stage("game setup", SystemStage::single(spawn_player.system()));
    app.run();
}


struct Materials {
    player_material: Handle<ColorMaterial>,
}


struct Player;
struct Position {
    x: i32,
    y: i32    
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(Materials {
        player_material: materials.add(Color::rgb(0., 0., 1.).into())
    });

}

fn spawn_player(mut commands: Commands, materials: Res<Materials>) {
    commands.spawn_bundle(SpriteBundle {
        material: materials.player_material.clone(),
        sprite: Sprite::new(Vec2::new(10., 10.)),
        ..Default::default()
    }).insert(Player).insert(Position {x: 1, y: 1});
}

// fn player_movement(keyboard_input: Res<Input<KeyCode>>, mut Query<)