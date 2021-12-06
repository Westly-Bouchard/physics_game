use bevy::prelude::*;
use wasm_bindgen::prelude::*;
use bevy::render::pass::ClearColor;
use bevy::sprite::collide_aabb::{collide, Collision};
// Retrograde appears to have broken itself so I guess we won't be using it
// use bevy_retrograde::prelude::*;

// Actual resolution of the window will be the default 1280 * 720 but I think it's best to limit ourselves to 480 * 270
// This means we need scaling factors... thankfully, these resolutions are both 16:9, so they can be constantly defined:
#[allow(dead_code)]
const SCALING_FACTOR: f64 = 1280. / 480.;

const WINDOW_WIDTH: f32 = 1280.;
const WINDOW_HEIGHT: f32 = 720.;

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
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
        ..Default::default()
    });

    app.add_plugins(DefaultPlugins);

    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);

    app.init_resource::<Materials>();
    app.init_resource::<UIMaterials>();
    app.init_resource::<UIState>();
    
    app.insert_resource(ClearColor(Color::rgb(1., 1., 1.)));
    
    app.add_startup_system(setup.system().label("setup"));

    app.add_startup_stage(SETUP, SystemStage::single_threaded()
        .with_system_set(SystemSet::new()
            .with_system(spawn_map.system())
            .with_system(spawn_player.system())));

    app.add_system(player_movement.system());
    app.add_system_set(SystemSet::new()
        .with_system(ui_setup.system())
        .with_system(ui_handle.system())
        .with_system(ui_buttons.system())
    );

    app.add_event::<UIStateUpdateEvent>();

    app.run();
}

#[derive(Default)]
struct Materials {
    player_material: Handle<ColorMaterial>,
    map: Handle<ColorMaterial>
}


struct Player;
struct Wall;

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
    commands.insert_resource(Materials {
        player_material: materials.add(Color::rgb(0., 0., 1.).into()),
        map: materials.add(Color::rgb(0., 0., 0.,).into())
    });
}

fn spawn_map(mut commands: Commands, materials: Res<Materials>) {
    //Far Left
    commands.spawn_bundle(good_box_to_bad_box(&materials, -630, 350, -610, -350)).insert(Wall);

    //Far right
    commands.spawn_bundle(good_box_to_bad_box(&materials, 610, 350, 630, -350)).insert(Wall);

    //Top
    commands.spawn_bundle(good_box_to_bad_box(&materials, -494, 350, 610, 330)).insert(Wall);

    //Bottom
    commands.spawn_bundle(good_box_to_bad_box(&materials, -610, -350, 494, -330)).insert(Wall);

    commands.spawn_bundle(good_box_to_bad_box(&materials, -610, -48, -474, -68)).insert(Wall);
    
    commands.spawn_bundle(good_box_to_bad_box(&materials, -86, -330, -66, -194)).insert(Wall);
    commands.spawn_bundle(good_box_to_bad_box(&materials, 186, -330, 206, -194)).insert(Wall);

    commands.spawn_bundle(good_box_to_bad_box(&materials, -222, 330, -242, 78)).insert(Wall);
    commands.spawn_bundle(good_box_to_bad_box(&materials, -494, 58, -66, 78)).insert(Wall);
    commands.spawn_bundle(good_box_to_bad_box(&materials, -494, 78, -474, 194)).insert(Wall);
    commands.spawn_bundle(good_box_to_bad_box(&materials, -494, 194, -358, 214)).insert(Wall);

    commands.spawn_bundle(good_box_to_bad_box(&materials, -378, -78, -358, 58)).insert(Wall);

    commands.spawn_bundle(good_box_to_bad_box(&materials, 338, -214, 610, -194)).insert(Wall);
    commands.spawn_bundle(good_box_to_bad_box(&materials, -86, -330, -66, -194)).insert(Wall);

    commands.spawn_bundle(good_box_to_bad_box(&materials, 494, 78, 610, 98)).insert(Wall);
    commands.spawn_bundle(good_box_to_bad_box(&materials, 474, -58, 494, 98)).insert(Wall);

    commands.spawn_bundle(good_box_to_bad_box(&materials, -494, -234, -202, -214)).insert(Wall);
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

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
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



    for mut transform in player_query.iter_mut() {
        transform.translation.x += x_movement;
        transform.translation.y += y_movement;
    }
}

// ui cool

struct UIMaterials {
    button_default: Handle<ColorMaterial>,
    button_hovered: Handle<ColorMaterial>,
    button_pressed: Handle<ColorMaterial>,
    modal_background: Handle<ColorMaterial>
}

#[derive(Debug)]
struct UIState {
    open: bool
}

impl Default for UIState {
    fn default() -> Self {
        Self { open: false }
    }
}

struct UIModalView;

enum UIStateUpdateEvent {
    Open,
    Close
}

struct UIComponent;


impl FromWorld for UIMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        UIMaterials {
            button_default: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            button_hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            button_pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
            modal_background: materials.add(Color::rgb(0., 0., 0.).into())
        }
    }
}

fn ui_setup(mut event_writer: EventWriter<UIStateUpdateEvent>, keyboard_input: Res<Input<KeyCode>>, mut state: ResMut<UIState>) {
    if keyboard_input.pressed(KeyCode::Escape) && state.open {
        println!("Detected escape press, state is {}", state.open);
        state.open = false;
        event_writer.send(UIStateUpdateEvent::Close);

    } else if keyboard_input.pressed(KeyCode::S) && !state.open {
        println!("Detected S press, state is {}", state.open);
        state.open = true;
        event_writer.send(UIStateUpdateEvent::Open);
    }
}

fn ui_handle(
    mut commands: Commands, 
    mut event_reader: EventReader<UIStateUpdateEvent>,
    materials: Res<UIMaterials>, 
    asset_server: Res<AssetServer>,
    mut query: Query<Entity, With<UIComponent>>
) {
    match event_reader.iter().next() {
        None => {},
        Some(UIStateUpdateEvent::Open) => {
            commands
                .spawn_bundle(SpriteBundle {
                    material: materials.modal_background.clone(),
                    sprite: Sprite::new(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
                    ..Default::default()
                })
                .insert(UIModalView)
                .insert(UIComponent);
            commands
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.), Val::Px(65.)),
                        margin: Rect::all(Val::Auto),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    material: materials.button_default.clone(),
                    ..Default::default()
                })
                .insert(UIComponent)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Button",
                            TextStyle {
                                font: asset_server.load("fonts/FiraCode-Bold.ttf"),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    }).insert(UIComponent);
                });
        },
        Some(UIStateUpdateEvent::Close) => {
            for entity in query.iter_mut() {
                commands.entity(entity).despawn();
            }
        }
    }
}


fn ui_buttons(
    materials: Res<UIMaterials>,
     mut query: Query<(&Interaction, &mut Handle<ColorMaterial>, &Children), (Changed<Interaction>, With<Button>)>,
    mut text_query: Query<&mut Text>) {
        for (interaction, mut material, children) in query.iter_mut() {
            let mut text = text_query.get_mut(children[0]).unwrap();

            match *interaction {
                Interaction::Clicked => {
                    text.sections[0].value = "Press".to_string();
                    *material = materials.button_pressed.clone();
                }
                Interaction::Hovered => {
                    text.sections[0].value = "Hover".to_string();
                    *material = materials.button_hovered.clone();
                }
                Interaction::None => {
                    text.sections[0].value = "Button".to_string();
                    *material = materials.button_default.clone();
                }
            }
        }

}