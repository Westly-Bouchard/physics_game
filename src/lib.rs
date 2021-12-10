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

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
enum Labels {
    Movement,
}


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

    app.init_resource::<Inventory>();
    
    app.insert_resource(ClearColor(Color::rgb(1., 1., 1.)));
    
    app.add_startup_system(setup.system().label("setup"));

    app.add_startup_stage(SETUP, SystemStage::single_threaded()
        .with_system_set(SystemSet::new()
            .with_system(spawn_map.system())
            .with_system(spawn_player.system()))
            .with_system(spawn_collectibles.system()));

    app.add_system(player_movement.system().label(Labels::Movement));
    app.add_system(player_collision_wall.system().after(Labels::Movement));
    app.add_system(collection_system.system().after(Labels::Movement));

    // app.add_system_set(SystemSet::new()
    //     .with_system(ui_dispatch.system())
    //     .with_system(ui_handle.system())
    //     .with_system(ui_buttons.system())
    // );

    // app.add_event::<UIStateUpdateEvent>();
    // app.add_event::<UIUserTransformEvent>();

    app.run();
}

#[derive(Default)]
struct Materials {
    player_material: Handle<ColorMaterial>,
    map: Handle<ColorMaterial>,
    resistor_material: Handle<ColorMaterial>,
    capacitor_material: Handle<ColorMaterial>
}

#[derive(Default)]
struct Inventory {
    has_resistor: bool,
    has_capacitor: bool,
}

struct Collider {
    width: i32,
    height: i32
}


struct Player;
struct Wall;
enum Collectable {
    RESISTOR,
    CAPACITOR
}
struct CollectedEvent {
    collectable_type: bool
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
    commands.insert_resource(Materials {
        player_material: materials.add(Color::rgb(0., 0., 1.).into()),
        map: materials.add(Color::rgb(0., 0., 0.,).into()),
        resistor_material: materials.add(Color::rgb(1., 0., 0.).into()),
        capacitor_material: materials.add(Color::rgb(1., 0.5, 0.).into()),
    });
}

fn spawn_map(mut commands: Commands, materials: Res<Materials>) {
    //Far Left
    commands.spawn_bundle(good_box_to_bad_box(&materials, -630, -350, -610, 350)).insert(Wall);

    //Far right
    commands.spawn_bundle(good_box_to_bad_box(&materials, 610, -350, 630, 350)).insert(Wall);

    // Top
    commands.spawn_bundle(good_box_to_bad_box(&materials, -494, 330, 610, 350)).insert(Wall);

    //Bottom
    commands.spawn_bundle(good_box_to_bad_box(&materials, -610, -350, 494, -330)).insert(Wall);

    commands.spawn_bundle(good_box_to_bad_box(&materials, -610, -68, -474, -48)).insert(Wall);
    
    commands.spawn_bundle(good_box_to_bad_box(&materials, -86, -330, -66, -194)).insert(Wall);
    commands.spawn_bundle(good_box_to_bad_box(&materials, 186, -330, 206, -194)).insert(Wall);

    commands.spawn_bundle(good_box_to_bad_box(&materials, -242, 78, -222, 330)).insert(Wall);
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
        transform: Transform::from_xyz(0., 0., 0.,),
        sprite: Sprite::new(Vec2::new(20., 20.)),
        ..Default::default()
    }).insert(Player);
}

fn spawn_collectibles(mut commands: Commands, materials: Res<Materials>) {
    commands.spawn_bundle(SpriteBundle {
        material: materials.resistor_material.clone(),
        transform: Transform::from_xyz(-400., 300., 0.),
        sprite: Sprite::new(Vec2::new(10., 10.)),
        ..Default::default()
    }).insert(Collectable::RESISTOR);

    commands.spawn_bundle(SpriteBundle {
        material: materials.capacitor_material.clone(),
        transform: Transform::from_xyz(50., 0., 0.,),
        sprite: Sprite::new(Vec2::new(10., 10.)),
        ..Default::default()
    }).insert(Collectable::CAPACITOR);
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

fn player_collision_wall(
    mut q: QuerySet<(Query<(&Transform, &Sprite), With<Player>>, Query<&mut Transform, With<Player>>, Query<(&Transform, &Sprite), With<Wall>>)>
) {
    let (player_transform, player_sprite) = q.q0().single().unwrap();
    let mut delta_X: f32 = 0.;
    let mut delta_Y: f32 = 0.;

    for (wall_transform, wall_sprite) in q.q2().iter() {
        let collision = collide (
            wall_transform.translation,
            wall_sprite.size,
            player_transform.translation,
            player_sprite.size,
        );

        if let Some(collision) = collision {
            match collision {
                Collision::Left => {
                    delta_X = (wall_transform.translation.x - player_transform.translation.x) + wall_sprite.size.x / 2.
                },
                Collision::Right => {
                    delta_X = (wall_transform.translation.x - player_transform.translation.x) - wall_sprite.size.x / 2.
                },
                Collision::Top => {
                    delta_Y = (wall_transform.translation.y - player_transform.translation.y) - wall_sprite.size.y / 2.
                },
                Collision::Bottom => {
                    delta_Y = (wall_transform.translation.y - player_transform.translation.y) + wall_sprite.size.y / 2.
                },
            }
        }
    }

    drop(player_transform);
    drop(player_sprite);

    let mut player_transform_mut = q.q1_mut().single_mut().unwrap();

    if delta_X != 0. {player_transform_mut.translation.x -= delta_X;}

    if delta_Y != 0. {player_transform_mut.translation.y -= delta_Y;}    
}

fn collection_system(
    player_query: Query<(&Transform, &Sprite), With<Player>>,
    collectable_query: Query<(Entity, &Transform, &Sprite, &Collectable)>,
    mut player_inventory: ResMut<Inventory>,
    mut commands: Commands
    /*mut collection_writer: EventWriter<CollectedEvent>,*/
) {
    let (player_transform, player_sprite) = player_query.single().unwrap();

    for (entity, collectable_transform, collectable_sprite, type_of) in collectable_query.iter() {
        let collision = collide(
            collectable_transform.translation,
            collectable_sprite.size,
            player_transform.translation,
            player_sprite.size,
        );

        if let Some(collision) = collision {
            match type_of {
                Collectable::RESISTOR => {
                    player_inventory.has_resistor = true;
                    println!("{}", player_inventory.has_resistor);
                    commands.entity(entity).despawn()
                }
                Collectable::CAPACITOR => {
                    player_inventory.has_resistor = true;
                    commands.entity(entity).despawn()
                }
            }

        }
    }
}


// ui cool, very based

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

struct UserTransformable {
    active: bool
}

impl Default for UserTransformable {
    fn default() -> Self {
        Self {
            active: true
        }
    }
}

enum UIUserTransformEvent {
    BeginTransform,
    FinalizeTransform
}


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

fn ui_dispatch(
    mut state_writer: EventWriter<UIStateUpdateEvent>,
    mut mouse_pos: EventReader<CursorMoved>,
    keyboard_input: Res<Input<KeyCode>>,
     mut state: ResMut<UIState>
) {
    if keyboard_input.pressed(KeyCode::Escape) && state.open {
        state.open = false;
        state_writer.send(UIStateUpdateEvent::Close);
    } else if keyboard_input.pressed(KeyCode::S) && !state.open {
        state.open = true;
        state_writer.send(UIStateUpdateEvent::Open);
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
                // .insert(UserTransformable)
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

// fn ui_user_transform_handle(mut query: Query<UserTransformable, With<&mut Transform>>) {
//     for (state, transform) in query.iter_mut() {

//     }
// }



fn ui_buttons(
    materials: Res<UIMaterials>,
    mut event_writer: EventWriter<UIUserTransformEvent>,
    mut query: Query<(&Interaction, &mut Handle<ColorMaterial>, &Children), (Changed<Interaction>, With<Button>)>,
    mut text_query: Query<&mut Text>) {
        for (interaction, mut material, children) in query.iter_mut() {
            let mut text = text_query.get_mut(children[0]).unwrap();

            match *interaction {
                Interaction::Clicked => {
                    text.sections[0].value = "Press".to_string();
                    event_writer.send(UIUserTransformEvent::BeginTransform);
                    *material = materials.button_pressed.clone();
                }
                Interaction::Hovered => {
                    text.sections[0].value = "Hover".to_string();
                    *material = materials.button_hovered.clone();
                }
                Interaction::None => {
                    text.sections[0].value = "Button".to_string();
                    event_writer.send(UIUserTransformEvent::FinalizeTransform);
                    *material = materials.button_default.clone();
                }
            }
        }

}
