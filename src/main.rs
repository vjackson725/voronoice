use std::time::Instant;

use bevy::{prelude::*, render::{camera::Camera, pipeline::PrimitiveTopology}};

mod pipeline;
use pipeline::*;

mod voronoi;
use crate::voronoi::Voronoi;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(VertexColorPlugin)
        .add_resource(ClearColor(Color::rgb(0., 0., 0.))) //background
        .add_startup_system(setup.system())
        .add_system(handle_input.system())
        .run();
}

fn color_white(_i: usize) -> Color {
    Color::WHITE
}

fn color_red(_i: usize) -> Color {
    Color::RED
}

fn generate_voronoi(size: usize) -> Voronoi {
    let start = Instant::now();
    let mut voronoi = Voronoi::default();
    voronoi.randomize_points(size);
    voronoi.build();
    println!("Generated new voronoi of size {} in {:?}", size, start.elapsed());

    // let len = 6;
    // let r = 1.0;
    // voronoi.sites.push(Point { x: 0.0, y: 0.0 });
    // for i in 0..len {
    //     let a = (i as f64 * 360.0 / len as f64).to_radians();
    //     voronoi.sites.push(Point {
    //         x: r * a.sin(),
    //         y: r * a.cos()
    //     });
    // }

    voronoi
}

struct VoronoiMeshOptions {
    topology: PrimitiveTopology
}

impl Default for VoronoiMeshOptions {
    fn default() -> Self {
        VoronoiMeshOptions {
            topology: PrimitiveTopology::LineList
        }
    }
}

fn spawn_voronoi(commands: &mut Commands, mut meshes: ResMut<Assets<Mesh>>, voronoi: &Voronoi, options: &VoronoiMeshOptions) {
    let start = Instant::now();
    let voronoi_generator = voronoi::VoronoiMeshGenerator { voronoi: &voronoi, coloring: color_red, topology: options.topology };
    let triangle_generator = voronoi::VoronoiMeshGenerator { voronoi: &voronoi, coloring: color_white, topology: PrimitiveTopology::LineList };

    commands
        .spawn(
        ColorBundle {
                mesh: meshes.add(voronoi_generator.build_voronoi_mesh()),
                transform: Transform::from_translation(Vec3::new(
                    0.0,
                    0.0,
                    0.0,
                )),
                ..Default::default()
            })
        .spawn(
            ColorBundle {
                    mesh: meshes.add(triangle_generator.build_delauney_mesh()),
                    transform: Transform::from_translation(Vec3::new(
                        0.0,
                        0.0,
                        0.0,
                    )),
                    ..Default::default()
        })
    ;

    println!("Generated new voronoi meshes in {:?}", start.elapsed());
}

const CAMERA_Y: f32 = 6.0;

// right hand
// triangulation anti-clockwise
fn setup(
    commands: &mut Commands
) {
    // let mut voronoi_loyd_1 = voronoi.loyd_relaxation();
    // voronoi_loyd_1.build();

    // let mut voronoi_loyd_2 = voronoi_loyd_1.loyd_relaxation();
    // voronoi_loyd_2.build();

    // let mut voronoi_loyd_3 = voronoi_loyd_2.loyd_relaxation();
    // voronoi_loyd_3.build();

    let camera_pos = Vec3::new(0.000001, CAMERA_Y, 0.0);
    let mut camera_t = Transform::from_translation(camera_pos)
        .looking_at(Vec3::default(), Vec3::unit_y());
    // roll camera so Z point up, and X right
    camera_t.rotate(Quat::from_rotation_ypr(0.0, 0.0, 180f32.to_radians()));

    commands
        // camera
        .spawn(Camera3dBundle {
            transform: camera_t,
            ..Default::default()
        });
}

#[derive(Default)]
struct State {
    voronoi_opts: VoronoiMeshOptions,
    voronoi: Option<Voronoi>,
    size: usize,
}
fn handle_input(
    mut state: Local<State>,
    input: Res<Input<KeyCode>>,
    meshes: ResMut<Assets<Mesh>>,
    commands: &mut Commands,
    query: Query<Entity, With<VertexColor>>,
    mut camera_query: Query<&mut Transform, With<Camera>>) {

    let mut respawn = false;

    // no voronoi, generate random one
    if !state.voronoi.is_some() {
        respawn = true;
        state.size = 20;
        state.voronoi = Some(generate_voronoi(state.size));
    }

    // span new voronoi with new rendering but same points
    if input.just_pressed(KeyCode::P) {
        let options = &mut state.voronoi_opts;
        options.topology = match options.topology {
            PrimitiveTopology::TriangleList => PrimitiveTopology::LineList,
            PrimitiveTopology::LineList => PrimitiveTopology::PointList,
            _ => PrimitiveTopology::TriangleList,
        };

        respawn = true;
    }

    // change number of points
    if input.just_pressed(KeyCode::Up) {
        respawn = true;
        state.size += 100;
        state.voronoi = Some(generate_voronoi(state.size));
    } else if input.just_pressed(KeyCode::Down) {
        respawn = true;
        state.size = state.size.max(120) - 100;
        state.voronoi = Some(generate_voronoi(state.size));
    } else if input.just_pressed(KeyCode::PageUp) {
        respawn = true;
        state.size += 1000;
        state.voronoi = Some(generate_voronoi(state.size));
    } else if input.just_pressed(KeyCode::PageDown) {
        respawn = true;
        state.size = state.size.max(1020) - 1000;
        state.voronoi = Some(generate_voronoi(state.size));
    }

    // span new voronoi with new points
    if input.just_pressed(KeyCode::G) {
        respawn = true;
        state.voronoi = Some(generate_voronoi(state.size));
    }

    if respawn {
        for e in query.iter() {
            commands.despawn(e);
        }

        spawn_voronoi(commands, meshes, state.voronoi.as_ref().expect("Where is my voronoi"), &state.voronoi_opts);
    }

    if input.pressed(KeyCode::W) {
        for mut t in camera_query.iter_mut() {
            t.translation.y -= 0.1;
        }
    } else if input.pressed(KeyCode::S) {
        for mut t in camera_query.iter_mut() {
            t.translation.y += 0.1;
        }
    } else if input.pressed(KeyCode::R) {
        for mut t in camera_query.iter_mut() {
            t.translation.y = CAMERA_Y;
        }
    }
}

// Created 10000000 random points in 2074149 micrseconds
// delaunator: 10000000 points processed in 10,111,821 micrseconds
// Created 10000000 random points in 2053817 micrseconds
// voronoi: 10000000 points processed in 6,6532,576 micrseconds

// Created 10000 random points in 2066 micrseconds
// delaunator: 10000 points processed in 3796 micrseconds
// Created 10000 random points in 2048 micrseconds
// voronoi: 10000 points processed in 32993 micrseconds
// [andre@scout voronoi]$ cargo run
//     Finished dev [unoptimized + debuginfo] target(s) in 0.01s
//      Running `target/debug/terrain`
// Created 10000 random points in 2075 micrseconds
// delaunator: 10000 points processed in 3710 micrseconds
// Created 10000 random points in 2050 micrseconds
// voronoi: 10000 points processed in 31867 micrseconds
// [andre@scout voronoi]$ cargo run
//    Compiling terrain v0.1.0 (/home/andre/projects/learn/rust/voronoi)
//     Finished dev [unoptimized + debuginfo] target(s) in 0.29s
//      Running `target/debug/terrain`
// Created 100000 random points in 20829 micrseconds
// delaunator: 100000 points processed in 52593 micrseconds
// Created 100000 random points in 20560 micrseconds
// voronoi: 100000 points processed in 415524 micrseconds
// [andre@scout voronoi]$ cargo run
//    Compiling terrain v0.1.0 (/home/andre/projects/learn/rust/voronoi)
//     Finished dev [unoptimized + debuginfo] target(s) in 0.28s
//      Running `target/debug/terrain`
// Created 1000000 random points in 209930 micrseconds
// delaunator: 1000000 points processed in 744958 micrseconds
// Created 1000000 random points in 206342 micrseconds
// voronoi: 1000000 points processed in 5113890 micrseconds
// [andre@scout voronoi]$ cargo run
//    Compiling terrain v0.1.0 (/home/andre/projects/learn/rust/voronoi)
//     Finished dev [unoptimized + debuginfo] target(s) in 0.28s
//      Running `target/debug/terrain`
// Created 10000000 random points in 2074149 micrseconds
// delaunator: 10000000 points processed in 10111821 micrseconds
// Created 10000000 random points in 2053817 micrseconds
// voronoi: 10000000 points processed in 66532576 micrseconds
// [andre@scout voronoi]$ cargo run --release
