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
const TAG_GHOST_1: i8 = 8;
const TAG_GHOST_2: i8 = 9;
const TAG_GHOST_3: i8 = 10;
const TAG_GHOST_4: i8 = 11;
const TAG_GHOST_5: i8 = 12;
const TAG_GHOST_6: i8 = 13;
const TAG_GHOST_7: i8 = 14;
const TAG_GHOST_8: i8 = 15;

const LANE_LAYOUT_LEFT: f32 = -510.0;
const LANE_LAYOUT_BOTTOM: f32 = -270.0;
const LANE_LAYOUT_HEIGHT: f32 = 540.0;
const LANE_LAYOUT_LANE_WIDTH: f32 = 89.0;
const LANE_LAYOUT_LANE_COUNT: u8 = 9;
const LANE_LAYOUT_MARGIN: f32 = 30.0;
const LANE_LAYOUT_BUFFER_LANES: u8 = 2;
const LANE_LAYOUT_SPAWN_LANES: u8 = LANE_LAYOUT_LANE_COUNT - LANE_LAYOUT_BUFFER_LANES - LANE_LAYOUT_BUFFER_LANES;

const Z_POS_BACKGROUND: f32 = -10.0;
const Z_POS_GHOSTS: f32 = -8.0;
const Z_POS_FRAME: f32 = -7.0;
const Z_POS_DEVICE_BACK: f32 = -6.0;
const Z_POS_DEVICE_BUTTONS: f32 = -5.0;
const Z_POS_CAMERA: f32 = 10.0;

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

const GHOST_HAT_NAMES: [&str; 8] = [
    "arrow",
    "cone",
    "party",
    "arrow",
    "cone",
    "party",
    "arrow",
    "cone",
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
    .insert_resource(BoxesMade { val: false })
    .add_message::<CaptureGhostsInitialized>()
    .add_systems(PreStartup, load_sprites)
    .add_systems(Startup, (
        spawn_ui,
        spawn_camera,
        spawn_ghosts,
        //spawn_debug_lane_boxes,
    ))
    .add_systems(Update, (
        animate_ghosts,
        begin_scooting_ghosts,
        scoot_ghosts,
        update_remote_elements,
        handle_remote_clicks,
        spawn_debug_clickable_boxes,
        handle_capture_ghosts,
    ))
    .run();
}

#[derive(Resource, Default)]
struct Sprites {
    //by body, then by hat
    ghosts: Option<[[Handle<Image>; 8]; 8]>,
    background: Option<Handle<Image>>,
    frame: Option<Handle<Image>>,
    frame_counter: Option<[Handle<Image>; 10]>,
    remote_base: Option<Handle<Image>>,
    remote_dial: Option<[Handle<Image>; 3]>,
    // by wave, then by state
    remote_wave_toggles: Option<[[Handle<Image>; 2]; 5]>,
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

fn load_sprites(
    assets: Res<AssetServer>,
    mut sprites: ResMut<Sprites>
) {
    let mut handles = Vec::<[Handle<Image>; 8]>::new();
    for body in 0..8 {
        let mut handles_by_body = Vec::<Handle<Image>>::new();
        let body_name = GHOST_BODY_NAMES[body];
        for hat in 0..8 {
            let hat_name = GHOST_HAT_NAMES[hat];
            let file_name = format!("ghosts/{body_name}_{hat_name}.png");
            let handle: Handle<Image> = assets.load(file_name);
            handles_by_body.push(handle);
        }
        handles.push(handles_by_body
            .try_into()
            .expect("Vec should have 8 elements"));
    }
    sprites.ghosts = Some(handles.try_into().expect("Vec should have 8 elements"));
    sprites.background = Some(assets.load("ui/Background.png"));
    sprites.frame = Some(assets.load("ui/Frame.png"));
    sprites.remote_base = Some(assets.load("ui/RemoteBase.png"));
    let mut dial_handles = Vec::<Handle<Image>>::new();
    for dial_idx in 1..=3 {
        let file_name = format!("ui/Dial{}.png", dial_idx);
        dial_handles.push(assets.load(file_name));
    }
    sprites.remote_dial = Some(dial_handles.try_into().expect("Vec should have 3 elements"));
    
    let mut wave_toggles = Vec::<[Handle<Image>; 2]>::new();
    for wave_idx in 1..=5 {
        let wave_off = format!("ui/Wave{}_off.png", wave_idx);
        let wave_on = format!("ui/Wave{}_on.png", wave_idx);
        wave_toggles.push([
            assets.load(wave_off),
            assets.load(wave_on),
        ]);
    }
    sprites.remote_wave_toggles = Some(wave_toggles.try_into().expect("Vec should have 5 elements"));

    let mut counters = Vec::<Handle<Image>>::new();
    for counter_idx in 1..=10 {
        let file_name = format!("ui/Counter{}.png", counter_idx);
        counters.push(assets.load(file_name));
    }
    sprites.frame_counter = Some(counters.try_into().expect("Vec should have 10 elements"));
}

fn build_button_config(selected_tags: &[i8; 4]) -> ButtonConfig {
    let mut rng = rand::rng();
    let inverted = rng.random::<bool>();
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
        inverted,
        enabled: true,
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
    ];
    all_hats.shuffle(&mut rng);
    let mut all_ghosts = vec![
        TAG_GHOST_1,
        TAG_GHOST_2,
        TAG_GHOST_3,
        TAG_GHOST_4,
        TAG_GHOST_5,
        TAG_GHOST_6,
        TAG_GHOST_7,
        TAG_GHOST_8,
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
        dial_strength: 2,
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
    let ghost = (TAG_GHOST_1..=TAG_GHOST_8).choose(&mut rng).unwrap();
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
    theta: f32,
    omega: f32,
    radius: f32,
    offset: f32,
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
    tags: Vec<i8>
}

#[derive(PartialEq, Eq)]
enum ClickableType {
    Dial,
    WaveEnable(i8),
    WaveInvert(i8),
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
    mut commands: Commands,
) {
    let background = sprites.background.clone().expect("Sprites should be loaded");
    let frame = sprites.frame.clone().expect("Sprites should be loaded");
    let remote_base = sprites.remote_base.clone().expect("Sprites should be loaded");
    let dial = sprites.remote_dial.as_ref().expect("Sprites should be loaded")[0].clone();
    let waves = sprites.remote_wave_toggles.as_ref().expect("Sprites should be loaded");
    commands.spawn((
        Sprite::from_image(background),
        Transform::from_xyz(0.0, 0.0, Z_POS_BACKGROUND)
            .with_scale(Vec3::new(0.2, 0.2, 1.0))
    ));
    commands.spawn((
        Sprite::from_image(frame),
        Transform::from_xyz(0.0, 0.0, Z_POS_FRAME)
            .with_scale(Vec3::new(0.2, 0.2, 1.0))
    ))
    .with_child((
        Transform::from_xyz(0.0, 0.0, 0.0),
        Clickable {
            clickable_type: ClickableType::CaptureGhosts,
            bounds: Rect::new(-900.0, 1600.0, -150.0, 1800.0),
        },
    ));
    // TODO: fix this
    commands.spawn((
        Sprite::from_image(remote_base),
        Transform::from_xyz(450.0, -60.0, Z_POS_DEVICE_BACK)
            .with_scale(Vec3::new(0.2, 0.2, 1.0))
    )).with_children(|cmd| {
        cmd.spawn((
            Sprite::from_image(dial),
            Transform::from_xyz(270.0, 620.0, 1.0),
        )).with_child((
            Transform::from_xyz(0.0, 0.0, 1.0),
            Clickable {
                clickable_type: ClickableType::Dial,
                bounds: Rect::new(-140.0, -15.0, 125.0, -280.0),
            }
        ));
        spawn_wave_button(cmd, Vec2::new(30.0, 50.0), waves[0][0].clone(), 0);
        spawn_wave_button(cmd, Vec2::new(22.0, -300.0), waves[1][0].clone(), 1);
        spawn_wave_button(cmd, Vec2::new(14.0, -650.0), waves[2][0].clone(), 2);
        spawn_wave_button(cmd, Vec2::new(6.0, -1000.0), waves[3][0].clone(), 3);
        spawn_wave_button(cmd, Vec2::new(-2.0, -1350.0), waves[4][0].clone(), 4);
    });
}

fn spawn_wave_button(
    commands: &mut RelatedSpawnerCommands<'_, ChildOf>,
    position: Vec2,
    sprite_handle: Handle<Image>,
    button_idx: i8,
) {
    commands.spawn((
        Sprite::from_image(sprite_handle),
        Transform::from_xyz(position.x, position.y, 1.0),
    )).with_children(|mut cmd| {
        cmd.spawn((
            Transform::from_xyz(0.0, 0.0, 1.0),
            Clickable {
                clickable_type: ClickableType::WaveEnable(button_idx),
                bounds: Rect::new(-400.0, -100.0, 175.0, 120.0),
            }
        ));
        // TODO: fix this
        cmd.spawn((
            Transform::from_xyz(0.0, 0.0, 1.0),
            Clickable {
                clickable_type: ClickableType::WaveInvert(button_idx),
                bounds: Rect::new(250.0, -75.0, 375.0, 85.0),
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
            let sprite = sprites[(ghost_tag - TAG_GHOST_1) as usize][(hat_tag - TAG_HAT_1) as usize].clone();
            commands.spawn((
                Ghost,
                Transform::from_xyz(pos.x, pos.y, 0.0),
                Visibility::default(),
                GhostTags {
                    tags: vec![
                        hat_tag,
                        ghost_tag,
                    ]
                },
                GhostLanePosition {
                    lane: lane_index,
                },
            ))
            .with_child((
                Sprite::from_image(sprite),
                Transform::from_xyz(0.0, 10.0, Z_POS_GHOSTS).with_scale(Vec3::new(0.05, 0.05, 1.0)),
                GhostAnimationLoop {
                    theta: rand::random::<f32>() * 2.0 * std::f32::consts::PI,
                    omega: std::f32::consts::PI,
                    radius: 10.0,
                    offset: 15.0,
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
        TAG_GHOST_1 => Some(TagType::Body),
        TAG_GHOST_2 => Some(TagType::Body),
        TAG_GHOST_3 => Some(TagType::Body),
        TAG_GHOST_4 => Some(TagType::Body),
        TAG_GHOST_5 => Some(TagType::Body),
        TAG_GHOST_6 => Some(TagType::Body),
        TAG_GHOST_7 => Some(TagType::Body),
        TAG_GHOST_8 => Some(TagType::Body),
        _ => None,
    };
}

fn spawn_camera(
    mut commands: Commands,
) {
    commands.spawn((
        Camera2d,
        Camera::default()
    ));
}

fn animate_ghosts(
    ghosts: Query<(&mut Transform, &mut GhostAnimationLoop)>,
    time: Res<Time>,
) {
    for (mut transform, mut ghost_anim) in ghosts {
        ghost_anim.theta += ghost_anim.omega * time.delta_secs();
        transform.translation.y = ghost_anim.theta.sin() * ghost_anim.radius + ghost_anim.offset;
    }
}

fn add_to_tag_moves(tag_moves: &mut HashMap::<i8, i8>, button: &ButtonConfig) {
    if button.enabled {
        for interaction in button.interactions.iter() {
            if let Some(interaction) = interaction {
                if let Some(val) = tag_moves.get_mut(&interaction.tag) {
                    *val += interaction.strength;
                } else {
                    tag_moves.insert(interaction.tag, interaction.strength);
                }
            }
        }
    }
}

fn begin_scooting_ghosts(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    ghosts: Query<(Entity, &GhostTags, &mut GhostLanePosition), (With<Ghost>, Without<GhostScooting>)>,
    lanes: Res<LaneLayout>,
    ghost_wave: Res<GhostWaveConfig>,
    mut commands: Commands,
) {
    let mut tag_moves = HashMap::<i8, i8>::new();
    if keyboard_input.just_pressed(KeyCode::Space) {
        add_to_tag_moves(&mut tag_moves, &ghost_wave.buttons[0]);
        add_to_tag_moves(&mut tag_moves, &ghost_wave.buttons[1]);
        add_to_tag_moves(&mut tag_moves, &ghost_wave.buttons[2]);
        add_to_tag_moves(&mut tag_moves, &ghost_wave.buttons[3]);
        add_to_tag_moves(&mut tag_moves, &ghost_wave.buttons[4]);
    }
    //foreach ghost with a matching tag, apply that movement, clamping
    //it for now to the edges

    //in the future, instead of clamping probably apply a new component
    //to make them move off the map
    let wave_strength = ghost_wave.dial_strength as i8;
    for (ghost_entity, ghost_tags, mut ghost_lane_pos) in ghosts {
        if let Ok(mut ghost_cmd) = commands.get_entity(ghost_entity) {
            let mut move_acc = 0i8;
            for tag in &ghost_tags.tags {
                if let Some(lane_change) = tag_moves.get(&tag) {
                    move_acc += lane_change * wave_strength;
                }
            }
            if move_acc != 0 {
            // apply the move component 
                let ghost_lane = ghost_lane_pos.lane as i8;
                let new_lane_idx = (ghost_lane + move_acc).clamp(0, (LANE_LAYOUT_LANE_COUNT - 1) as i8);
                if new_lane_idx == ghost_lane {
                    continue;
                }
                let next_lane = lanes.margined_lanes[new_lane_idx as usize];
                ghost_cmd.insert(
                    GhostScooting {
                        scoot_target: get_random_point_in_rect(&next_lane),
                        movement_speed: 300.0,
                    });
                ghost_lane_pos.lane = new_lane_idx as u8;
            }
        }
    }
}

fn scoot_ghosts(
    ghosts: Query<(Entity, &GhostScooting, &mut Transform)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (ghost_entity, scooting, mut transform) in ghosts {
        let direction = scooting.scoot_target - transform.translation.xy();
        let remaining_distance = direction.length();
        let direction = direction.normalize();
        let move_distance = time.delta_secs() * scooting.movement_speed;
        if move_distance > remaining_distance {
            transform.translation.x = scooting.scoot_target.x;
            transform.translation.y = scooting.scoot_target.y;
            if let Ok(mut ghost_cmd) = commands.get_entity(ghost_entity) {
                ghost_cmd.remove::<GhostScooting>();
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
    mut on_capture_fire: MessageWriter<CaptureGhostsInitialized>
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        if let Ok((camera, camera_transform)) = camera.single() {
            if let Ok(window) = window.single() {
                if let Some(cursor_pos) = window.cursor_position() {
                    if let Ok(cursor_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                        for (clickable_transform, clickable) in query {
                            let clickable_pos = clickable_transform.translation();
                            let clickable_scale = clickable_transform.scale();
                            let width = clickable.bounds.width() * clickable_scale.x;
                            let height = clickable.bounds.height() * clickable_scale.y;
                            let half_width = width / 2.0;
                            let half_height = height / 2.0;
                            let left = clickable_pos.x - half_width;
                            let bottom = clickable_pos.y - half_height;

                            // this only sorta works, I think I'm not applying the scale right
                            // but once scaling stops I can remove the scale stuff, then it should be ok
                            if cursor_pos.x >= left && cursor_pos.x <= left + width 
                                && cursor_pos.y >= bottom && cursor_pos.y < bottom + height 
                            {
                                match clickable.clickable_type {
                                    ClickableType::Dial => { buttons.dial_strength = (buttons.dial_strength % 3) + 1; },
                                    ClickableType::WaveEnable(idx) => { 
                                        buttons.buttons[idx as usize].enabled = !buttons.buttons[idx as usize].enabled;
                                    },
                                    ClickableType::WaveInvert(idx) => {
                                        buttons.buttons[idx as usize].inverted = !buttons.buttons[idx as usize].inverted;
                                    },
                                    ClickableType::CaptureGhosts => {
                                        on_capture_fire.write(CaptureGhostsInitialized);
                                    },
                                };
                            }
                        }
                    }
                }
            }
        }
    }
}

fn handle_capture_ghosts(
    mut on_capture_fired: MessageReader<CaptureGhostsInitialized>,
    ghosts: Query<(Entity, &GhostLanePosition)>,
    mut commands: Commands,
) {
    if on_capture_fired.is_empty() {
        return;
    }
    on_capture_fired.clear();
    for (entity, ghost_lane) in ghosts {
        if ghost_lane.lane == 4 { //5th, center lane
            // TODO: Update points, despawn the ghosts, flash the screen(?), create lil ghost souls
            commands.entity(entity).despawn();
        }
    }
}

fn update_remote_elements(
    sprites: Res<Sprites>,
    ghost_wave: Res<GhostWaveConfig>,
    query: Query<&Clickable>,
    parents: Query<(&mut Sprite, &Children)>,
) {
    let waves = sprites.remote_wave_toggles.as_ref().expect("Images should be loaded");
    let dial = sprites.remote_dial.as_ref().expect("Images should be loaded");
    for (mut sprite, children) in parents {
        for child in children.iter() {
            if let Ok(clickable) = query.get(child) {
                if let ClickableType::WaveEnable(idx) = clickable.clickable_type {
                    let idx = idx as usize;
                    let is_enabled = ghost_wave.buttons[idx].enabled;
                    sprite.image = if is_enabled {
                        waves[idx][1].clone()
                    } else {
                        waves[idx][0].clone()
                    };
                }
                if clickable.clickable_type == ClickableType::Dial {
                    let dial_idx = (ghost_wave.dial_strength - 1) as usize;
                    sprite.image = dial[dial_idx].clone();
                }
            }
        }
    }
}

fn spawn_debug_lane_boxes(
    mut commands: Commands,
    mut gizmo_assets: ResMut<Assets<GizmoAsset>>,
) {
    let center_y = LANE_LAYOUT_BOTTOM
        + (LANE_LAYOUT_HEIGHT / 2.0);
    for i in 0..LANE_LAYOUT_LANE_COUNT {
        let center_x = LANE_LAYOUT_LEFT + (i as f32 + 0.5) * LANE_LAYOUT_LANE_WIDTH;
        let mut gizmo = GizmoAsset::new();
        gizmo.rect_2d(
            Isometry2d::from_xy(center_x, center_y),
            Vec2::new(LANE_LAYOUT_LANE_WIDTH, LANE_LAYOUT_HEIGHT),
            Color::WHITE);
        commands.spawn(
            Gizmo {
                handle: gizmo_assets.add(gizmo),
                line_config: GizmoLineConfig {
                    width: 2.,
                    ..default()
                },
                ..default()
            }
        );
    }
}

#[derive(Resource)]
struct BoxesMade {
    val: bool
}

fn spawn_debug_clickable_boxes(
    mut commands: Commands,
    clickables: Query<(Entity, &Clickable)>,
    mut boxes_made: ResMut<BoxesMade>,
    mut gizmo_assets: ResMut<Assets<GizmoAsset>>,
) {
    if boxes_made.val {
        return;
    }
    for (entity, clickable) in clickables {
        let rect = clickable.bounds;
        let mut gizmo = GizmoAsset::new();
        let half_width = rect.width() / 2.0;
        let half_height = rect.height() / 2.0;
        gizmo.rect_2d(
            Isometry2d::from_xy(rect.min.x + half_width, rect.min.y + half_height),
            Vec2::new(rect.width(), rect.height()),
            Color::BLACK);
        commands.entity(entity)
            .insert(
                Gizmo {
                    handle: gizmo_assets.add(gizmo),
                    line_config: GizmoLineConfig {
                        width: 2.0,
                        ..default()
                    },
                    ..default()
                });
    }
    boxes_made.val = true;
}
