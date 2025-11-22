use bevy::prelude::*;
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
    .add_systems(PreStartup, load_sprites)
    .add_systems(Startup, (spawn_ui, spawn_camera, spawn_ghosts, spawn_debug_boxes))
    .add_systems(Update, (animate_ghosts, begin_scooting_ghosts, scoot_ghosts))
    .run();
}

#[derive(Resource, Default)]
struct Sprites {
    //by body, then by hat
    ghosts: Option<[[Handle<Image>; 8]; 8]>,
    background: Option<Handle<Image>>,
    frame: Option<Handle<Image>>,
    remote_base: Option<Handle<Image>>,
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
    let handles: [[Handle<Image>; 8]; 8] = handles.try_into().expect("Vec should have 8 elements");
    sprites.ghosts = Some(handles);
    sprites.background = Some(assets.load("ui/Background.png"));
    sprites.frame = Some(assets.load("ui/Frame.png"));
    sprites.remote_base = Some(assets.load("ui/RemoteBase.png"));
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

#[derive(Component)]
struct Background;

#[derive()]
struct Clickable;

fn spawn_ui(
    sprites: Res<Sprites>,
    mut commands: Commands,
) {
    let background = sprites.background.clone().expect("Sprites should be loaded");
    let frame = sprites.frame.clone().expect("Sprites should be loaded");
    let remote_base = sprites.remote_base.clone().expect("Sprites should be loaded");
    commands.spawn((
        Sprite::from_image(background),
        Transform::from_xyz(0.0, 0.0, Z_POS_BACKGROUND)
            .with_scale(Vec3::new(0.2, 0.2, 1.0))
    ));
    commands.spawn((
        Sprite::from_image(frame),
        Transform::from_xyz(0.0, 0.0, Z_POS_FRAME)
            .with_scale(Vec3::new(0.2, 0.2, 1.0))
    ));
    commands.spawn((
        Sprite::from_image(remote_base),
        Transform::from_xyz(450.0, -60.0, Z_POS_DEVICE_BACK)
            .with_scale(Vec3::new(0.2, 0.2, 1.0))
    ));
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

fn begin_scooting_ghosts(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    ghosts: Query<(Entity, &GhostTags, &mut GhostLanePosition), (With<Ghost>, Without<GhostScooting>)>,
    lanes: Res<LaneLayout>,
    ghost_wave: Res<GhostWaveConfig>,
    mut commands: Commands,
) {
    //TODO: skip building up this vec and instead write directly
    //to the hashmap
    let mut interactions = Vec::<GhostInteraction>::new();
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        // if button is enabled &&...
        for interaction in ghost_wave.buttons[0].interactions.iter() {
            if let Some(interaction) = interaction {
                interactions.push(interaction.clone());
            }
        }
    }
    if keyboard_input.just_pressed(KeyCode::Digit2) {
        for interaction in ghost_wave.buttons[1].interactions.iter() {
            if let Some(interaction) = interaction {
                interactions.push(interaction.clone());
            }
        }
    }
    if keyboard_input.just_pressed(KeyCode::Digit3) {
        for interaction in ghost_wave.buttons[2].interactions.iter() {
            if let Some(interaction) = interaction {
                interactions.push(interaction.clone());
            }
        }
    }
    if keyboard_input.just_pressed(KeyCode::Digit4) {
        for interaction in ghost_wave.buttons[3].interactions.iter() {
            if let Some(interaction) = interaction {
                interactions.push(interaction.clone());
            }
        }
    }
    if keyboard_input.just_pressed(KeyCode::Digit5) {
        for interaction in ghost_wave.buttons[4].interactions.iter() {
            if let Some(interaction) = interaction {
                interactions.push(interaction.clone());
            }
        }
    }
    //build up a list of the interactions that are going to happen
    //and for now, just assume every button is enabled
    
    //foreach interaction, build up a Map with a key of tag to see
    //the total lane movement per tag
    let mut tag_moves = HashMap::<i8, i8>::new();
    for interaction in interactions {
        debug!("interaction: {} - {}", interaction.tag, interaction.strength);    
        if let Some(val) = tag_moves.get_mut(&interaction.tag) {
            *val += interaction.strength;
        } else {
            tag_moves.insert(interaction.tag, interaction.strength);
        }
    }

    //foreach ghost with a matching tag, apply that movement, clamping
    //it for now to the edges

    //in the future, instead of clamping probably apply a new component
    //to make them move off the map
    for (ghost_entity, ghost_tags, mut ghost_lane_pos) in ghosts {
        if let Ok(mut ghost_cmd) = commands.get_entity(ghost_entity) {
            let mut move_acc = 0i8;
            for tag in &ghost_tags.tags {
                if let Some(lane_change) = tag_moves.get(&tag) {
                    move_acc += lane_change;
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

fn spawn_debug_boxes(
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
