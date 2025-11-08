use bevy::prelude::*;

const TAG_TOP_HAT: i8 = 0;
const TAG_BEANIE: i8 = 1;

const LANE_LAYOUT_LEFT: f32 = -450.0;
const LANE_LAYOUT_BOTTOM: f32 = -200.0;
const LANE_LAYOUT_HEIGHT: f32 = 400.0;
const LANE_LAYOUT_LANE_WIDTH: f32 = 100.0;
const LANE_LAYOUT_LANE_COUNT: u8 = 9;
const LANE_LAYOUT_MARGIN: f32 = 10.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(build_lane_layout())
        .add_systems(Startup, (spawn_camera, spawn_ghosts))
        .add_systems(Update, (animate_ghosts, begin_scooting_ghosts, scoot_ghosts))
        .run();
}

#[derive(Resource)]
struct LaneLayout {
    lanes: Vec<Rect>,
    margined_lanes: Vec<Rect>,
}

fn build_lane_layout() -> LaneLayout {
    let mut lanes = Vec::<Rect>::new();
    let mut margined_lanes = Vec::<Rect>::new();
    for lane in 0..LANE_LAYOUT_LANE_COUNT {
        lanes.push(get_lane_boundary(lane, 0.0));
        margined_lanes.push(get_lane_boundary(lane, LANE_LAYOUT_MARGIN));
    }
    return LaneLayout {
        lanes,
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

fn spawn_ghosts(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    lanes: Res<LaneLayout>,
    mut commands: Commands,
) {
    for lane_index in 1..(LANE_LAYOUT_LANE_COUNT - 1) {
        let hat_tag = if lane_index % 2 == 0 { TAG_TOP_HAT } else { TAG_BEANIE };
        let pos = get_random_point_in_rect(&lanes.margined_lanes[lane_index as usize]);
        commands.spawn((
            Ghost,
            Transform::from_xyz(pos.x, pos.y, 0.0),
            Visibility::default(),
            GhostTags {
                tags: vec![
                    hat_tag,
                ]
            },
            GhostLanePosition {
                lane: lane_index,
            },
        ))
        .with_child((
            Mesh2d(meshes.add(Rectangle::new(50.0, 50.0)).into()),
            MeshMaterial2d(materials.add(Color::srgb(1.0, 0.5, 0.2))),
            Transform::from_xyz(0.0, 10.0, 0.0),
            GhostAnimationLoop {
                theta: rand::random::<f32>() * 2.0 * std::f32::consts::PI,
                omega: std::f32::consts::PI,
                radius: 10.0,
                offset: 15.0,
            }
        ));
    }
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
    mut commands: Commands,
) 
{
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    }

    for (ghost_entity, ghost_tags, mut ghost_lane) in ghosts {
        if ghost_lane.lane > 0 && ghost_tags.tags.contains(&TAG_BEANIE) {
            if let Ok(mut ghost_cmd) = commands.get_entity(ghost_entity) {
                let next_lane = lanes.margined_lanes[(ghost_lane.lane - 1) as usize];
                ghost_cmd.insert(
                    GhostScooting {
                        scoot_target: get_random_point_in_rect(&next_lane),
                        movement_speed: 300.0,
                    });
                ghost_lane.lane -= 1;
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

