use bevy::{
    ecs::relationship::RelatedSpawnerCommands,
    prelude::*,
    window::PrimaryWindow,
};
use rand::prelude::*;
use std::collections::HashMap;

const TAG_HAT_1: i8 = 0;
const TAG_HAT_2: i8 = 1;
const TAG_HAT_3: i8 = 2;
const TAG_HAT_4: i8 = 3;
const TAG_HAT_5: i8 = 4;
const TAG_HAT_6: i8 = 5;
const TAG_HAT_7: i8 = 6;
const TAG_HAT_8: i8 = 7;
const TAG_HAT_9: i8 = 8;
const TAG_HAT_10: i8 = 9;
const TAG_HAT_11: i8 = 10;
const TAG_HAT_12: i8 = 11;
const TAG_HAT_13: i8 = 12;
const TAG_HAT_14: i8 = 13;
const TAG_BODY_1: i8 = 14;
const TAG_BODY_2: i8 = 15;
const TAG_BODY_3: i8 = 16;
const TAG_BODY_4: i8 = 17;
const TAG_BODY_5: i8 = 18;
const TAG_BODY_6: i8 = 19;
const TAG_BODY_7: i8 = 20;
const TAG_BODY_8: i8 = 21;

const LANE_LAYOUT_LEFT: f32 = -1635.5;
const LANE_LAYOUT_BOTTOM: f32 = -900.0;
const LANE_LAYOUT_HEIGHT: f32 = 1500.0;
const LANE_LAYOUT_LANE_WIDTH: f32 = 284.5;
const LANE_LAYOUT_LANE_COUNT: u8 = 9;
const LANE_LAYOUT_MARGIN: f32 = 100.0;
const LANE_LAYOUT_BUFFER_LANES: u8 = 2;
const LANE_LAYOUT_SPAWN_LANES: u8 = LANE_LAYOUT_LANE_COUNT - LANE_LAYOUT_BUFFER_LANES - LANE_LAYOUT_BUFFER_LANES;
const LANE_LAYOUT_DESPAWN_LEFT: f32 = -2000.0;
const LANE_LAYOUT_DESPAWN_RIGHT: f32 = 2000.0;

const Z_POS_BACKGROUND: f32 = -10.0;
const Z_POS_GHOSTS: f32 = -8.0;
const Z_POS_FRAME: f32 = -7.0;
const Z_POS_DEVICE_BACK: f32 = -6.0;

const GHOST_SPRITE_SCALE: f32 = 0.4;
const GHOSTS_PER_LANE: u8 = 3;
// don't spawn ghosts in the edges
const EXPECTED_TOTAL_GHOSTS: u8 = GHOSTS_PER_LANE * LANE_LAYOUT_SPAWN_LANES;

const GHOST_BODY_NAMES: [&str; 8] = [
    "Booloon",
    "Ghoost",
    "Ghostie",
    "Handshee",
    "Puppergeist",
    "SoapSprite",
    "Timboo",
    "Yolkai",
];

const GHOST_HAT_NAMES: [&str; 14] = [
    "arrow",
    "belt",
    "bow",
    "cone",
    "crown",
    "flower",
    "glasses",
    "lollipop",
    "mug",
    "mustache",
    "party",
    "propellor",
    "tophat",
    "wings",
];

const GHOST_WAVE_NAMES: [&str; 5] = [
    "Rectified",
    "Sawtooth",
    "Sine",
    "Square",
    "Triangle",
];

fn main() {
    let target_ghosts = choose_target_ghosts();
    let ghost_wave = build_ghost_wave_config(&target_ghosts);
    App::new()
    .add_plugins(DefaultPlugins)
    .insert_resource(build_lane_layout())
    .insert_resource(Sprites::default())
    .insert_resource(target_ghosts)
    .insert_resource(ghost_wave)
    .insert_resource(UIEnabled { enabled: true })
    .insert_resource(PlayerResources { charges: 3, points: 0 })
    .add_message::<CaptureGhostsInitialized>()
    .add_message::<RemoteFired>()
    .add_message::<GameWon>()
    .add_message::<GameLost>()
    .add_systems(PreStartup, load_sprites)
    .add_systems(Startup, (
        spawn_ui,
        spawn_camera,
        spawn_ghosts,
    ))
    .add_systems(Update, (
        animate_ghosts,
        begin_scooting_ghosts,
        scoot_ghosts,
        update_remote_lights,
        update_remote_invert_switches,
        update_remote_dial,
        update_wave_handle,
        handle_remote_clicks,
        capture_ghosts,
        handle_ui_enabled,
        handle_game_end,
    ))
    .run();
}

#[derive(Resource, Default)]
struct Sprites {
    //by body, then by hat
    ghosts: Option<[[Handle<Image>; 14]; 8]>,
    background: Option<Handle<Image>>,
    frame: Option<Handle<Image>>,
    frame_counter: Option<[Handle<Image>; 10]>,
    remote_base: Option<Handle<Image>>,
    remote_dial: Option<[Handle<Image>; 3]>,
    // by wave, then by state
    remote_wave_buttons: Option<[Handle<Image>; 5]>,
    remote_wave_light: Option<[Handle<Image>; 2]>,
    remote_wave_inverter: Option<[Handle<Image>; 2]>,
    remote_handle: Option<Handle<Image>>,
    particles: Option<[Handle<Image>; 5]>,
}

#[derive(Resource)]
struct UIEnabled {
    enabled: bool,
}

#[derive(Resource)]
struct PlayerResources {
    charges: u8,
    points: u8,
}

#[derive(Clone)]
struct GhostInteraction {
    tag: i8,
    strength: i8,
}

struct ButtonConfig {
    interactions: [Option<GhostInteraction>; 4],
    inverted: bool,
    enabled: bool,
}

#[derive(Resource)]
struct GhostWaveConfig {
    buttons: [ButtonConfig; 5],
    dial_strength: u8,
    unique_tags: Vec<i8>
}

#[derive(Resource)]
struct TargetGhostTags {
    target_body: i8,
    target_hat: i8,
}

#[derive(Component)]
struct TargetGhostDisplay;

fn load_sprites(
    assets: Res<AssetServer>,
    mut sprites: ResMut<Sprites>
) {
    let mut handles = Vec::<[Handle<Image>; 14]>::new();
    for body in 0..8 {
        let mut handles_by_body = Vec::<Handle<Image>>::new();
        let body_name = GHOST_BODY_NAMES[body];
        for hat in 0..14 {
            let hat_name = GHOST_HAT_NAMES[hat];
            let file_name = format!("ghosts/{body_name}_{hat_name}.png");
            let handle: Handle<Image> = assets.load(file_name);
            handles_by_body.push(handle);
        }
        handles.push(handles_by_body
            .try_into()
            .expect("Vec should have 14 elements"));
    }
    sprites.ghosts = Some(handles.try_into().expect("Vec should have 8 elements"));
    sprites.background = Some(assets.load("ui/Background.png"));
    sprites.frame = Some(assets.load("ui/Frame.png"));
    sprites.remote_base = Some(assets.load("ui/Machine.png"));
    let mut dial_handles = Vec::<Handle<Image>>::new();
    for dial_idx in 1..=3 {
        let file_name = format!("ui/Dial{}.png", dial_idx);
        dial_handles.push(assets.load(file_name));
    }
    sprites.remote_dial = Some(dial_handles.try_into().expect("Vec should have 3 elements"));
    
    let mut wave_buttons = Vec::<Handle<Image>>::new();
    for wave in GHOST_WAVE_NAMES {
        let file_name = format!("ui/Button{}.png", wave);
        wave_buttons.push(assets.load(file_name));
    }
    sprites.remote_wave_buttons = Some(wave_buttons.try_into().expect("Vec should have 5 elements"));

    sprites.remote_wave_light = Some([
        assets.load("ui/LightOff.png"),
        assets.load("ui/LightOn.png"),
    ]);

    sprites.remote_wave_inverter = Some([
        assets.load("ui/InvertSetOff.png"),
        assets.load("ui/InvertSetOn.png"),
    ]);

    sprites.remote_handle = Some(assets.load("ui/Handle.png"));

    let mut counters = Vec::<Handle<Image>>::new();
    for counter_idx in 1..=10 {
        let file_name = format!("ui/Counter{}.png", counter_idx);
        counters.push(assets.load(file_name));
    }
    sprites.frame_counter = Some(counters.try_into().expect("Vec should have 10 elements"));

    let mut wave_particles = Vec::<Handle<Image>>::new();
    for wave in GHOST_WAVE_NAMES {
        let file_name = format!("ui/Particle{}.png", wave);
        wave_particles.push(assets.load(file_name));
    }
    sprites.particles = Some(wave_particles.try_into().expect("Vec should have 5 elements"));
}

fn build_button_config(selected_tags: &[i8; 4]) -> ButtonConfig {
    let mut rng = rand::rng();
    let mut interactions = [
        Option::<GhostInteraction>::None,
        Option::<GhostInteraction>::None,
        Option::<GhostInteraction>::None,
        Option::<GhostInteraction>::None,
    ];

    for i in 0..selected_tags.len() {
        let tag = selected_tags[i];
        if tag == -1 {
            continue;
        }
        //we want any value between -3 and 3 except 0
        let mut strength = rng.random_range(-1..=0);
        if strength >= 0 {
            strength += 1;
        }
        interactions[i] = 
            Some(
            GhostInteraction {
                tag,
                strength,
            });
    }

    return ButtonConfig {
        interactions,
        inverted: false,
        enabled: false,
    };
}

fn build_ghost_wave_config(target_ghost: &TargetGhostTags) -> GhostWaveConfig {
    let mut rng = rand::rng();
    let mut all_hats = vec![
        TAG_HAT_1,
        TAG_HAT_2,
        TAG_HAT_3,
        TAG_HAT_4,
        TAG_HAT_5,
        TAG_HAT_6,
        TAG_HAT_7,
        TAG_HAT_8,
        TAG_HAT_9,
        TAG_HAT_10,
        TAG_HAT_11,
        TAG_HAT_12,
        TAG_HAT_13,
        TAG_HAT_14,
    ];
    all_hats.shuffle(&mut rng);
    let mut all_ghosts = vec![
        TAG_BODY_1,
        TAG_BODY_2,
        TAG_BODY_3,
        TAG_BODY_4,
        TAG_BODY_5,
        TAG_BODY_6,
        TAG_BODY_7,
        TAG_BODY_8,
    ];
    all_ghosts.shuffle(&mut rng);

    let mut selected_hats = Vec::<i8>::new();
    let mut selected_ghosts = Vec::<i8>::new();

    let mut i = 0usize;
    loop {
        let mut need_more_tags = false;
        if selected_hats.len() < 3 { //TODO: configure the amount here by the current level
            need_more_tags = true;

            let hat = all_hats[i];
            if hat != target_ghost.target_hat {
                selected_hats.push(hat);
            }
        }
        if selected_ghosts.len() < 3 { //TODO: configure the amount here by the current level
            need_more_tags = true;

            let ghost = all_ghosts[i];
            if ghost != target_ghost.target_body {
                selected_ghosts.push(ghost);
            }
        }
        i += 1;
        if !need_more_tags {
            break;
        }
    }
    let mut tag_pool = Vec::<i8>::new();
    tag_pool.extend(&selected_hats);
    tag_pool.extend(&selected_hats);
    tag_pool.extend(&selected_ghosts);
    tag_pool.extend(&selected_ghosts);
    tag_pool.shuffle(&mut rng);

    //probably treat the two primary buttons differently, then build the rest from the scraps
    //each button is going to do 1-3 things to start with
    //at least two of those buttons will move the main target by one of their tags, as well as one other
    //unrelated tag.
    //1 + 2 + 2 + 3 + 3 = 11
    // build them, then shuffle them to stuff into the final struct
    let mut button_1 = [-1i8;4];
    let mut button_2 = [-1i8;4];
    let mut button_3 = [-1i8;4];
    let mut button_4 = [-1i8;4];
    let mut button_5 = [-1i8;4];
    let mut spare_tags = Vec::<i8>::new();
    //TODO: configure these based on the current level
    select_button_interactions(3, &mut tag_pool, &mut spare_tags, &mut button_1);
    select_button_interactions(3, &mut tag_pool, &mut spare_tags, &mut button_2);
    select_button_interactions(2, &mut tag_pool, &mut spare_tags, &mut button_3);
    select_button_interactions(2, &mut tag_pool, &mut spare_tags, &mut button_4);
    select_button_interactions(1, &mut tag_pool, &mut spare_tags, &mut button_5);

    let mut unique_tags = Vec::<i8>::new();

    for tag in &button_1 {
        if *tag != -1 && !unique_tags.contains(tag) {
            unique_tags.push(*tag);
        }
    }
    for tag in &button_2 {
        if *tag != -1 && !unique_tags.contains(tag) {
            unique_tags.push(*tag);
        }
    }
    for tag in &button_3 {
        if *tag != -1 && !unique_tags.contains(tag) {
            unique_tags.push(*tag);
        }
    }
    for tag in &button_4 {
        if *tag != -1 && !unique_tags.contains(tag) {
            unique_tags.push(*tag);
        }
    }
    for tag in &button_5 {
        if *tag != -1 && !unique_tags.contains(tag) {
            unique_tags.push(*tag);
        }
    }

    GhostWaveConfig {
        buttons: [
            build_button_config(&button_1),
            build_button_config(&button_2),
            build_button_config(&button_3),
            build_button_config(&button_4),
            build_button_config(&button_5),
        ],
        dial_strength: 1,
        unique_tags,
    }
}

fn select_button_interactions(n: usize, tag_pool: &mut Vec<i8>, spare_tags: &mut Vec<i8>, button_array: &mut [i8; 4]) {
    for i in 0..n {
        loop {
            let tag = tag_pool.pop().unwrap();
            if button_array.contains(&tag) {
                spare_tags.push(tag);
            } else {
                button_array[i] = tag;
                break;
            }
        }
    }
    for tag in spare_tags.drain(..) {
        tag_pool.push(tag);
    }
}

fn choose_target_ghosts() -> TargetGhostTags {
    let mut rng = rand::rng();
    let hat = (TAG_HAT_1..=TAG_HAT_8).choose(&mut rng).unwrap();
    let ghost = (TAG_BODY_1..=TAG_BODY_8).choose(&mut rng).unwrap();
    println!("body: {}; hat: {}", ghost, hat);
    return TargetGhostTags {
        target_body: ghost,
        target_hat: hat,
    };
}

#[derive(Resource)]
struct LaneLayout {
    //lanes: Vec<Rect>,
    margined_lanes: Vec<Rect>,
}

fn build_lane_layout() -> LaneLayout {
    //let mut lanes = Vec::<Rect>::new();
    let mut margined_lanes = Vec::<Rect>::new();
    for lane in 0..LANE_LAYOUT_LANE_COUNT {
        //lanes.push(get_lane_boundary(lane, 0.0));
        margined_lanes.push(get_lane_boundary(lane, LANE_LAYOUT_MARGIN));
    }
    return LaneLayout {
        //lanes,
        margined_lanes,
    };
}

fn get_lane_boundary(lane: u8, margin: f32) -> Rect {
    let left = LANE_LAYOUT_LEFT + (lane as f32 * LANE_LAYOUT_LANE_WIDTH) + margin;
    let right = left + LANE_LAYOUT_LANE_WIDTH - margin - margin;
    let top = LANE_LAYOUT_BOTTOM + LANE_LAYOUT_HEIGHT - margin;
    let bottom = LANE_LAYOUT_BOTTOM + margin;
    return Rect {
        min: Vec2::new(left, bottom),
        max: Vec2::new(right, top),
    };
}

fn get_random_point_in_rect(rect: &Rect) -> Vec2 {
    let width = rect.max.x - rect.min.x;
    let height = rect.max.y - rect.min.y;
    let x = rect.min.x + rand::random::<f32>() * width;
    let y = rect.min.y + rand::random::<f32>() * height;
    return Vec2::new(x, y);
}

#[derive(Component)]
struct Ghost;

#[derive(Component)]
struct GhostAnimationLoop {
    theta_x: f32,
    omega_x: f32,
    radius_x: f32,

    theta_y: f32,
    omega_y: f32,
    radius_y: f32,
    offset_y: f32,
}

#[derive(Component)]
struct GhostLanePosition {
    lane: u8
}

#[derive(Component)]
struct GhostScooting {
    scoot_target: Vec2,
    movement_speed: f32,
}

#[derive(Component)]
struct GhostTags {
    body_tag: i8,
    hat_tag: i8,
}

#[derive(PartialEq, Eq)]
enum ClickableType {
    Dial,
    WaveEnable(i8),
    WaveInvert(i8),
    FireWave,
    CaptureGhosts,
}

#[derive(Message)]
struct CaptureGhostsInitialized;

#[derive(Component)]
struct Clickable {
    clickable_type: ClickableType,
    bounds: Rect,
}

fn spawn_ui(
    sprites: Res<Sprites>,
    target_ghost: Res<TargetGhostTags>,
    mut commands: Commands,
) {
    let ghost_sprites = sprites.ghosts.as_ref().expect("Sprites should be loaded");
    let background = sprites.background.clone().expect("Sprites should be loaded");
    let frame = sprites.frame.clone().expect("Sprites should be loaded");
    let remote_base = sprites.remote_base.clone().expect("Sprites should be loaded");
    let dial = sprites.remote_dial.as_ref().expect("Sprites should be loaded")[0].clone();
    let handle = sprites.remote_handle.clone().expect("Sprites should be loaded");
    let waves = sprites.remote_wave_buttons.as_ref().expect("Sprites should be loaded");
    let toggles = sprites.remote_wave_inverter.as_ref().expect("Sprites should be loaded");
    let lights = sprites.remote_wave_light.as_ref().expect("Sprites should be loaded");
    let body_idx = (target_ghost.target_body - TAG_BODY_1) as usize;
    let hat_idx = target_ghost.target_hat as usize;
    let target_ghost_sprite = ghost_sprites[body_idx][hat_idx].clone();
    commands.spawn((
        Sprite::from_image(background),
        Transform::from_xyz(0.0, 0.0, Z_POS_BACKGROUND)
    ));
    commands.spawn((
        Sprite::from_image(frame),
        Transform::from_xyz(0.0, 0.0, Z_POS_FRAME)
    ))
    .with_children(|cmd| {
        cmd.spawn((
            Transform::from_xyz(-345.0, 1075.0, 0.0),
            Clickable {
                clickable_type: ClickableType::CaptureGhosts,
                bounds: Rect::new(-220.0, -75.0, 220.0, 75.0),
            },
        ));
        cmd.spawn((
            TargetGhostDisplay,
            Sprite::from_image(target_ghost_sprite),
            Transform::from_xyz(-1550.0, 830.0, 1.0)
                .with_scale(Vec3::new(0.5, 0.5, 1.0))
                .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, std::f32::consts::PI / 12.0)),
        ));
    });
    commands.spawn((
        Sprite::from_image(remote_base),
        Transform::from_xyz(1425.0, -180.0, Z_POS_DEVICE_BACK)
    )).with_children(|cmd| {
        cmd.spawn((
            StrengthDial,
            Sprite::from_image(dial),
            Transform::from_xyz(180.0, 400.0, 1.0),
        )).with_child((
            Transform::from_xyz(0.0, -80.0, 1.0),
            Clickable {
                clickable_type: ClickableType::Dial,
                bounds: Rect::new(-80.0, -80.0, 80.0, 80.0),
            }
        ));
        cmd.spawn((
            FireWaveHandle,
            Transform::from_xyz(-195.0, 400.0, 1.0),
        )).with_child((
            Transform::from_xyz(5.0, 100.0, 0.0),
            Sprite::from_image(handle),
            Clickable {
                clickable_type: ClickableType::FireWave,
                bounds: Rect::new(-100.0, -100.0, 100.0, 100.0),
            }
        ));

        for i in 0..5 {
            const BUTTON_SPACING_X_START: f32 = -30.0;
            const BUTTON_SPACING_X: f32 = 5.0;
            const BUTTON_SPACING_Y_START: f32 = 50.0;
            const BUTTON_SPACING_Y: f32 = 235.0;
            let btn_x = BUTTON_SPACING_X_START - (BUTTON_SPACING_X * i as f32);
            let btn_y = BUTTON_SPACING_Y_START - (BUTTON_SPACING_Y * i as f32);
            spawn_wave_button(
                cmd,
                Vec2::new(btn_x, btn_y),
                waves[i].clone(),
                toggles[0].clone(),
                lights[0].clone(),
                i as i8);
        }
    });
}

#[derive(Component)]
struct WaveButtonLight {
    button_idx: i8,
}

#[derive(Component)]
struct InverterSwitch {
    button_idx: i8,
}

#[derive(Component)]
struct FireWaveHandle;

#[derive(Component)]
struct StrengthDial;

fn spawn_wave_button(
    commands: &mut RelatedSpawnerCommands<'_, ChildOf>,
    position: Vec2,
    button_sprite_handle: Handle<Image>,
    toggle_sprite_handle: Handle<Image>,
    light_sprite_handle: Handle<Image>,
    button_idx: i8,
) {
    commands.spawn((
        Transform::from_xyz(position.x, position.y, 1.0),
        Visibility::Visible,
    )).with_children(|cmd| {
        cmd.spawn((
            Sprite::from_image(button_sprite_handle),
            Transform::from_xyz(0.0, 0.0, 1.0),
            Clickable {
                clickable_type: ClickableType::WaveEnable(button_idx),
                bounds: Rect::new(-180.0, -55.0, 180.0, 55.0),
            }
        ));
        cmd.spawn((
            Sprite::from_image(light_sprite_handle),
            Transform::from_xyz(-260.0, 10.0, 1.0),
            WaveButtonLight {
                button_idx,
            },
        ));
        // TODO: fix this
        cmd.spawn((
            Sprite::from_image(toggle_sprite_handle),
            Transform::from_xyz(300.0, -20.0, 1.0),
            Clickable {
                clickable_type: ClickableType::WaveInvert(button_idx),
                bounds: Rect::new(-90.0, -90.0, 30.0, 90.0),
            },
            InverterSwitch {
                button_idx,
            }
        ));
    });
}

fn spawn_ghosts(
    sprites: Res<Sprites>,
    lanes: Res<LaneLayout>,
    ghost_wave: Res<GhostWaveConfig>,
    mut commands: Commands,
) {
    let sprites = sprites.ghosts.as_ref().expect("Sprites should be loaded");

    // let's make 3 ghosts in each lane
    // each of them will consist of one type and one hat

    // need to randomly but evenly distribute the tags we know about across all lanes by repeating
    // the set of unique tags

    let mut hats = Vec::<i8>::new();
    let mut bodies = Vec::<i8>::new();
    for tag in &ghost_wave.unique_tags {
        if let Some(tag_type) = get_tag_type(*tag) {
            if tag_type == TagType::Body {
                bodies.push(*tag);
            } else {
                hats.push(*tag);
            }
        }
    }
    let mut rng = rand::rng();
    //repeat hats and ghosts enough times to have enough for the expected ghosts
    let hats_to_repeat = hats.clone();
    let times_to_repeat = EXPECTED_TOTAL_GHOSTS.div_ceil(hats.len() as u8);
    for _ in 0..times_to_repeat {
        hats.extend(&hats_to_repeat);
    }
    let bodies_to_repeat = bodies.clone();
    let times_to_repeat = EXPECTED_TOTAL_GHOSTS.div_ceil(bodies.len() as u8);
    for _ in 0..times_to_repeat {
        bodies.extend(&bodies_to_repeat);
    }

    hats.shuffle(&mut rng);
    bodies.shuffle(&mut rng);

    for lane_index in 0..LANE_LAYOUT_SPAWN_LANES {
        for _ in 0..3 {
            let lane_index = lane_index + LANE_LAYOUT_BUFFER_LANES;
            let hat_tag = hats.pop().expect("Should have enough hat tags to share");
            let ghost_tag = bodies.pop().expect("Should have enough body tags to share");
            let pos = get_random_point_in_rect(&lanes.margined_lanes[lane_index as usize]);
            let sprite = sprites[(ghost_tag - TAG_BODY_1) as usize][(hat_tag - TAG_HAT_1) as usize].clone();
            let radius_x = 20.0 + rand::random::<f32>() * 10.0;
            let omega_x = std::f32::consts::PI / 8.0 + rand::random::<f32>() * std::f32::consts::PI / 4.0;
            let theta_x = rand::random::<f32>() * 2.0 * std::f32::consts::PI;

            let radius_y = 40.0 + rand::random::<f32>() * 20.0;
            let omega_y = std::f32::consts::PI / 4.0 + rand::random::<f32>() * std::f32::consts::PI / 2.0;
            let theta_y = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
            commands.spawn((
                Ghost,
                Transform::from_xyz(pos.x, pos.y, 0.0),
                Visibility::default(),
                GhostTags {
                    body_tag: ghost_tag,
                    hat_tag: hat_tag,
                },
                GhostLanePosition {
                    lane: lane_index,
                },
            ))
            .with_child((
                Sprite::from_image(sprite),
                Transform::from_xyz(0.0, 10.0, Z_POS_GHOSTS).with_scale(Vec3::new(GHOST_SPRITE_SCALE, GHOST_SPRITE_SCALE, 1.0)),
                GhostAnimationLoop {
                    theta_x,
                    omega_x,
                    radius_x,
                    theta_y,
                    omega_y,
                    radius_y,
                    offset_y: 45.0,
                }
            ));
        }
    }
}

#[derive(PartialEq, Eq)]
enum TagType {
    Body,
    Hat,
}

fn get_tag_type(tag: i8) -> Option<TagType> {
    return match tag {
        TAG_HAT_1 => Some(TagType::Hat),
        TAG_HAT_2 => Some(TagType::Hat),
        TAG_HAT_3 => Some(TagType::Hat),
        TAG_HAT_4 => Some(TagType::Hat),
        TAG_HAT_5 => Some(TagType::Hat),
        TAG_HAT_6 => Some(TagType::Hat),
        TAG_HAT_7 => Some(TagType::Hat),
        TAG_HAT_8 => Some(TagType::Hat),
        TAG_HAT_9 => Some(TagType::Hat),
        TAG_HAT_10 => Some(TagType::Hat),
        TAG_HAT_11 => Some(TagType::Hat),
        TAG_HAT_12 => Some(TagType::Hat),
        TAG_HAT_13 => Some(TagType::Hat),
        TAG_HAT_14 => Some(TagType::Hat),
        TAG_BODY_1 => Some(TagType::Body),
        TAG_BODY_2 => Some(TagType::Body),
        TAG_BODY_3 => Some(TagType::Body),
        TAG_BODY_4 => Some(TagType::Body),
        TAG_BODY_5 => Some(TagType::Body),
        TAG_BODY_6 => Some(TagType::Body),
        TAG_BODY_7 => Some(TagType::Body),
        TAG_BODY_8 => Some(TagType::Body),
        _ => None,
    };
}

fn spawn_camera(
    mut commands: Commands,
) {
    commands.spawn((
        Camera2d,
        Camera::default(),
        Transform::from_scale(Vec3::new(3.0, 3.0, 1.0)),
    ));
}

fn animate_ghosts(
    ghosts: Query<(&mut Transform, &mut GhostAnimationLoop)>,
    time: Res<Time>,
) {
    const GHOST_SQUIDGE_RADIUS: f32 = 0.01;

    for (mut transform, mut ghost_anim) in ghosts {
        ghost_anim.theta_y += ghost_anim.omega_y * time.delta_secs();
        ghost_anim.theta_x += ghost_anim.omega_x * time.delta_secs();
        transform.translation.y = ghost_anim.theta_y.sin() * ghost_anim.radius_y + ghost_anim.offset_y;
        transform.translation.x = ghost_anim.theta_x.sin() * ghost_anim.radius_x;
        transform.scale.x = GHOST_SPRITE_SCALE + (ghost_anim.theta_y * 2.0).sin() * GHOST_SQUIDGE_RADIUS;
        transform.scale.y = GHOST_SPRITE_SCALE + (ghost_anim.theta_y * 2.0 + std::f32::consts::PI / 2.0).sin() * GHOST_SQUIDGE_RADIUS;
    }
}

fn add_to_tag_moves(tag_moves: &mut HashMap::<i8, i8>, button: &ButtonConfig) {
    if button.enabled {
        for interaction in button.interactions.iter() {
            if let Some(interaction) = interaction {
                let invert_mod = if button.inverted { -1 } else { 1 };
                if let Some(val) = tag_moves.get_mut(&interaction.tag) {
                    *val += interaction.strength * invert_mod;
                } else {
                    tag_moves.insert(interaction.tag, interaction.strength * invert_mod);
                }
            }
        }
    }
}

#[derive(Message)]
struct RemoteFired;

#[derive(Component)]
struct WanderingOff;

fn begin_scooting_ghosts(
    mut on_fire: MessageReader<RemoteFired>,
    ghosts: Query<(Entity, &GhostTags, &mut GhostLanePosition), (With<Ghost>, Without<GhostScooting>)>,
    lanes: Res<LaneLayout>,
    ghost_wave: Res<GhostWaveConfig>,
    target_ghost: Res<TargetGhostTags>,
    mut commands: Commands,
    mut on_lose: MessageWriter<GameLost>,
) {
    if on_fire.is_empty() {
        return;
    }
    on_fire.clear();

    let mut tag_moves = HashMap::<i8, i8>::new();
    add_to_tag_moves(&mut tag_moves, &ghost_wave.buttons[0]);
    add_to_tag_moves(&mut tag_moves, &ghost_wave.buttons[1]);
    add_to_tag_moves(&mut tag_moves, &ghost_wave.buttons[2]);
    add_to_tag_moves(&mut tag_moves, &ghost_wave.buttons[3]);
    add_to_tag_moves(&mut tag_moves, &ghost_wave.buttons[4]);

    let wave_strength = ghost_wave.dial_strength as i8;
    for (ghost_entity, ghost_tags, mut ghost_lane_pos) in ghosts {
        if let Ok(mut ghost_cmd) = commands.get_entity(ghost_entity) {
            let mut move_acc = 0i8;
            if let Some(lane_change) = tag_moves.get(&ghost_tags.body_tag) {
                move_acc += lane_change * wave_strength;
            }
            if let Some(lane_change) = tag_moves.get(&ghost_tags.hat_tag) {
                move_acc += lane_change * wave_strength;
            }
            if move_acc != 0 {
                // apply the move component 
                let is_target = 
                    ghost_tags.body_tag == target_ghost.target_body
                    && ghost_tags.hat_tag == target_ghost.target_hat;
                let ghost_lane = ghost_lane_pos.lane as i8;
                let new_lane_idx = ghost_lane + move_acc;
                if new_lane_idx == ghost_lane {
                    continue;
                } else if new_lane_idx < 0 {
                    if is_target {
                        on_lose.write(GameLost);
                    }
                    let random_y = rand::random::<f32>() * LANE_LAYOUT_HEIGHT - LANE_LAYOUT_HEIGHT / 2.0;
                    ghost_cmd.insert((
                        WanderingOff,
                        GhostScooting {
                            scoot_target: Vec2::new(LANE_LAYOUT_DESPAWN_LEFT, random_y),
                            movement_speed: 400.0,
                        },
                    ));
                    // TODO: combine into single component?
                    ghost_cmd.remove::<Ghost>();
                    ghost_cmd.remove::<GhostTags>();
                    ghost_cmd.remove::<GhostLanePosition>();
                } else if new_lane_idx >= LANE_LAYOUT_LANE_COUNT as i8 {
                    if is_target {
                        on_lose.write(GameLost);
                    }
                    let random_y = rand::random::<f32>() * LANE_LAYOUT_HEIGHT - LANE_LAYOUT_HEIGHT / 2.0;
                    ghost_cmd.insert((
                        WanderingOff,
                        GhostScooting {
                            scoot_target: Vec2::new(LANE_LAYOUT_DESPAWN_RIGHT, random_y),
                            movement_speed: 400.0,
                        },
                    ));
                    // TODO: combine into single component?
                    ghost_cmd.remove::<Ghost>();
                    ghost_cmd.remove::<GhostTags>();
                    ghost_cmd.remove::<GhostLanePosition>();
                } else {
                    let next_lane = lanes.margined_lanes[new_lane_idx as usize];
                    ghost_cmd.insert(
                        GhostScooting {
                            scoot_target: get_random_point_in_rect(&next_lane),
                            movement_speed: 600.0,
                        });
                    ghost_lane_pos.lane = new_lane_idx as u8;
                }
            }
        }
    }
}

fn scoot_ghosts(
    ghosts: Query<(Entity, &GhostScooting, Option<&WanderingOff>, &mut Transform)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (ghost_entity, scooting, wandering, mut transform) in ghosts {
        let direction = scooting.scoot_target - transform.translation.xy();
        let remaining_distance = direction.length();
        let direction = direction.normalize();
        let move_distance = time.delta_secs() * scooting.movement_speed;
        if move_distance > remaining_distance {
            transform.translation.x = scooting.scoot_target.x;
            transform.translation.y = scooting.scoot_target.y;
            if let Ok(mut ghost_cmd) = commands.get_entity(ghost_entity) {
                if wandering.is_some() {
                    ghost_cmd.despawn();
                } else {
                    ghost_cmd.remove::<GhostScooting>();
                }
            }
            return;
        }
        let new_vel = direction * move_distance;

        transform.translation.x += new_vel.x;
        transform.translation.y += new_vel.y;
    }
}

fn handle_remote_clicks(
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut buttons: ResMut<GhostWaveConfig>,
    query: Query<(&GlobalTransform, &Clickable)>,
    mut on_capture_fire: MessageWriter<CaptureGhostsInitialized>,
    mut on_remote_fire: MessageWriter<RemoteFired>,
    ui_enabled: Res<UIEnabled>,
) {
    if !ui_enabled.enabled {
        return;
    }
    if mouse_button.just_pressed(MouseButton::Left) {
        if let Ok((camera, camera_transform)) = camera.single() {
            if let Ok(window) = window.single() {
                if let Some(cursor_pos) = window.cursor_position() {
                    if let Ok(cursor_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                        for (clickable_transform, clickable) in query {
                            let clickable_pos = clickable_transform.translation();
                            let left = clickable_pos.x + clickable.bounds.min.x;
                            let right = clickable_pos.x + clickable.bounds.max.x;
                            let bottom = clickable_pos.y + clickable.bounds.min.y;
                            let top = clickable_pos.y + clickable.bounds.max.y;

                            if cursor_pos.x >= left && cursor_pos.x <= right
                                && cursor_pos.y >= bottom && cursor_pos.y <= top
                            {
                                match clickable.clickable_type {
                                    ClickableType::Dial => { 
                                        buttons.dial_strength = (buttons.dial_strength % 3) + 1;
                                    },
                                    ClickableType::WaveEnable(idx) => { 
                                        buttons.buttons[idx as usize].enabled = !buttons.buttons[idx as usize].enabled;
                                    },
                                    ClickableType::WaveInvert(idx) => {
                                        buttons.buttons[idx as usize].inverted = !buttons.buttons[idx as usize].inverted;
                                    },
                                    ClickableType::CaptureGhosts => {
                                        on_capture_fire.write(CaptureGhostsInitialized);
                                    },
                                    ClickableType::FireWave => {
                                        on_remote_fire.write(RemoteFired);
                                    }
                                };
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Message)]
struct GameWon;

#[derive(Message)]
struct GameLost;

fn capture_ghosts(
    mut on_capture_fired: MessageReader<CaptureGhostsInitialized>,
    ghosts: Query<(Entity, &GhostLanePosition, &GhostTags)>,
    target: Res<TargetGhostTags>,
    mut player_resources: ResMut<PlayerResources>,
    mut commands: Commands,
    mut on_win: MessageWriter<GameWon>,
    mut on_lose: MessageWriter<GameLost>,
) {
    if on_capture_fired.is_empty() {
        return;
    }
    on_capture_fired.clear();
    let mut any_ghosts_captured = false;
    let mut points_delta = 0i8;
    let mut target_ghosts_exist_in_other_lanes = false;
    for (entity, ghost_lane, ghost_tags) in ghosts {
        let is_target = 
            ghost_tags.body_tag == target.target_body
            && ghost_tags.hat_tag == target.target_hat;
        if ghost_lane.lane == 4 { //5th, center lane
            any_ghosts_captured = true;
            if is_target {
                points_delta += 1;
            } else {
                points_delta -= 2;
            }
            println!("captured: body: {}; hat: {}", ghost_tags.body_tag, ghost_tags.hat_tag);
            // TODO: flash the screen(?), create lil ghost souls
            commands.entity(entity).despawn();
        } else {
            if is_target {
                target_ghosts_exist_in_other_lanes = true;
            }
        }
    }
    if any_ghosts_captured {
        //TODO: maybe do something with negative points?
            //Maybe you just lose instead?
        player_resources.points = points_delta.max(0) as u8;
        player_resources.charges -= 1;
        if player_resources.charges == 0 && target_ghosts_exist_in_other_lanes {
            on_lose.write(GameLost);
        }
        if !target_ghosts_exist_in_other_lanes {
            on_win.write(GameWon);
        }
    }
}
fn update_remote_lights(
    sprites: Res<Sprites>,
    ghost_wave: Res<GhostWaveConfig>,
    lights: Query<(&mut Sprite, &WaveButtonLight)>,
) {
    let light_sprites = sprites.remote_wave_light.as_ref().expect("Images should be loaded");
    for (mut light_sprite, light) in lights {
        let is_enabled = ghost_wave.buttons[light.button_idx as usize].enabled;
        if is_enabled {
            light_sprite.image = light_sprites[1].clone();
        } else {
            light_sprite.image = light_sprites[0].clone();
        }
    }
}

fn update_remote_invert_switches(
    sprites: Res<Sprites>,
    ghost_wave: Res<GhostWaveConfig>,
    inverters: Query<(&mut Sprite, &InverterSwitch)>,
) {
    let inverter_sprites = sprites.remote_wave_inverter.as_ref().expect("Images should be loaded");
    for (mut sprite, inverter) in inverters {
        let is_inverted = ghost_wave.buttons[inverter.button_idx as usize].inverted;
        if is_inverted {
            sprite.image = inverter_sprites[1].clone();
        } else {
            sprite.image = inverter_sprites[0].clone();
        }
    }
}

fn update_remote_dial(
    sprites: Res<Sprites>,
    ghost_wave: Res<GhostWaveConfig>,
    dials: Query<&mut Sprite, With<StrengthDial>>,
) {
    let dial_sprites = sprites.remote_dial.as_ref().expect("Images should be loaded");
    for mut sprite in dials {
        let dial_idx = (ghost_wave.dial_strength - 1) as usize;
        sprite.image = dial_sprites[dial_idx].clone();
    }
}

fn update_wave_handle(
    ui_enabled: Res<UIEnabled>,
    handles: Query<&mut Transform, With<FireWaveHandle>>
) {
    for mut transform in handles {
        let z_rot_rads = if ui_enabled.enabled {
            0.0
        } else {
            std::f32::consts::PI
        };
        transform.rotation = Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, z_rot_rads);
    }
}

fn handle_ui_enabled(
    mut ui_enabled: ResMut<UIEnabled>,
    ghosts: Query<&GhostScooting, With<Ghost>>,
) {
    ui_enabled.enabled = ghosts.is_empty();
}

//TODO: actually do something here
fn handle_game_end(
    mut on_win: MessageReader<GameWon>,
    mut on_lose: MessageReader<GameLost>,
) {
    for _ in on_win.read() {
        println!("you win!");
    }
    for _ in on_lose.read() {
        println!("you lose!");
    }
}
