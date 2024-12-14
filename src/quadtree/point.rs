use std::thread;

use bevy::{prelude::*, window::PrimaryWindow};
use rand::Rng;

use super::QuadTree;

#[derive(Component)]
pub struct Point {
    accel: Vec2,
    velo: Vec2,
    last_pos: Vec2,
    radius: f32
}

impl Point {
    fn new(accel: Vec2) -> Self {
        Self { accel: accel, velo: Vec2::ZERO, radius: 10., last_pos: Vec2::default() }
    }
}

#[derive(Component)]
pub struct PointParent;


pub fn draw_points(
    points: Query<(Entity, &Transform, &Point)>,
    mut gizmos: Gizmos
) {
    for (_ent, transform, point) in points.iter() {
        gizmos.circle_2d(transform.translation.truncate(), point.radius, Color::RED);
    }
}

fn gen_dir() -> f32 {
    let mut rng = rand::thread_rng();
    let is_negative = rng.gen::<bool>();
    let num = rng.gen_range(0.1..1.);
    if is_negative {
        return -num;
    }
    return num;
}

//SOURCES
// https://bevy-cheatbook.github.io/cookbook/cursor2world.html
// https://bevy-cheatbook.github.io/input/mouse.html
pub fn place_point(
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<crate::MainCamera>>,   
    buttons: Res<Input<MouseButton>>,   
    mut commands: Commands,
    parent_point: Query<Entity, With<PointParent>>,
    mut quad_tree: ResMut<QuadTree>,
    //mut gizmos: Gizmos
) {
    if parent_point.is_empty() {
        return;
    }

    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();

    if let Some(world_pos) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate()) {
            if world_pos.x < 0. || world_pos.x > 800. || world_pos.y > 0. || world_pos.y < -700. {
                return;
            }
            if !buttons.pressed(MouseButton::Left) {
                return;
            }
            let parent = parent_point.single();
            
            for i in 0..1 {
                let mut rng = rand::thread_rng();
                let x = gen_dir();
                let y = gen_dir();//rng.gen_range(0.5..1.);
                let speed = rng.gen_range(1.0..25.);
                let offset = i as f32 * 3.;

                let pos = Vec3::new(world_pos.x + offset, world_pos.y,1.);
                let transform = Transform::from_translation(pos);
                let mut point = Point::new(Vec2::new(x as f32, y as f32).normalize() * speed);
                point.last_pos = pos.truncate();

                let ent = commands.spawn((
                    point,
                    transform,
                    //Name::new(format!("point"))
                )).id();
                commands.entity(parent).add_child(ent);
                
                quad_tree.insert_point(&ent, Vec2::new(world_pos.x, world_pos.y));
            }
            /* 
            let rect = crate::quadtree::Rect::new(Vec2::new(world_pos.x, world_pos.y), Vec2::new(30.,30.));
            let overlaps = rect.overlaps(&quad_tree.bounds);
            if overlaps {
                let pos = rect.pos + Vec2::new(rect.size.x/2., -rect.size.y/2.);
                gizmos.rect_2d(pos, 0., rect.size, Color::RED);
            } else {
                let pos = rect.pos + Vec2::new(rect.size.x/2., -rect.size.y/2.);
                gizmos.rect_2d(pos, 0., rect.size, Color::WHITE);
            }
            */
            //println!("World coords: {}/{}", world_position.x, world_position.y);
    }
}

pub fn move_points(
    mut q_point: Query<(Entity, &mut Point, &mut Transform)>,
    mut quad_tree: ResMut<QuadTree>,
    time: Res<Time>
) {
    quad_tree.clear();
    for (ent, mut point, mut transform) in q_point.iter_mut() {
        let position = transform.translation.truncate();
        let accel = point.accel + Vec2::new(0.,-370.);
        let displacement = position - point.last_pos;
        let delta = time.delta_seconds();

        point.last_pos = position;

        //50 is damping
        let new_pos = position + displacement + (accel - displacement * 40.) * (delta * delta);
        transform.translation = new_pos.extend(1.);

        point.accel *= 0.;

        quad_tree.insert_point(&ent, transform.translation.truncate());
    }
}

pub fn apply_constraint(
    mut q_point: Query<(Entity, &mut Point, &mut Transform)>,
    quad_tree: ResMut<QuadTree>,
) {
    for (_ent, mut point, mut transform) in q_point.iter_mut() {
        let quad_pos = quad_tree.bounds.pos;
        let quad_size = quad_tree.bounds.size;
        let p_pos = transform.translation.truncate();
        let padding:f32 = 1.0;
        let rad: f32 = 10.+padding;

        //left check
        if p_pos.x-rad <= quad_pos.x {
            let dist = quad_pos.x + rad;
            transform.translation.x = dist;
            point.last_pos.x = dist
            //point.velo.x *= -1.;
        }
    
        //right check
        if p_pos.x+rad >= quad_pos.x + quad_size.x {
            let dist = quad_pos.x + quad_size.x - rad;
            transform.translation.x = dist;
            point.last_pos.x = dist
            //point.velo.x *= -1.;
        }

        //top check
        if p_pos.y+rad >= quad_pos.y {
            let dist = quad_pos.y - rad;
            transform.translation.y = dist;
            point.last_pos.y = dist
            //point.velo.y *= -1.;
        }
    
        //bottom check
        if p_pos.y-rad <= quad_pos.y - quad_size.y {
            let dist = quad_pos.y - quad_size.y + rad;
            transform.translation.y = dist;
            point.last_pos.y = dist
            //point.velo.y *= 0.;
            //point.accel.y += 10.;
        }
    }
}

pub fn point_collision(
    mut q_point: Query<(Entity, &mut Point, &mut Transform)>,
    quad_tree: Res<QuadTree>,
) {
    for _ in 0..2 {
    let mut results: Vec<(Entity, Entity)> = Vec::new();
    let eps = 0.0001;
    let response_coef = 0.75;

    for (ent, mut _point, transform) in q_point.iter_mut() {
        let a = transform.translation.truncate();
        let rect = crate::quadtree::Rect::new(Vec2::new(a.x-10., a.y+10.),Vec2::new(20.,20.));
        let query = quad_tree.query_area(&rect);

        //println!("query={:?}", query);
        'inner: for (o_ent, b) in query.iter() {
            let b_ent = o_ent.unwrap();
            if ent == b_ent {
                continue 'inner
            }
                
            let o2_o1 = a - *b;
            let d2 = o2_o1.x * o2_o1.x + o2_o1.y * o2_o1.y;
            let rad = 20.0*20.0;
            if d2 <= rad+21. && d2 > eps {
                results.push((ent, b_ent));
            }
        }
        //println!("d={}",d);
    }

    for (ent_a, ent_b) in results.iter() {
        let mut q = q_point.get_many_mut([*ent_a, *ent_b]).unwrap();

        let o2_o1 = q[0].2.translation.truncate() - q[1].2.translation.truncate();
        let d2 = o2_o1.x * o2_o1.x + o2_o1.y * o2_o1.y;

        //println!("d2={}", d2);
        if d2 <= 21.0*21.0 && d2 > eps {
            let dist = d2.sqrt();
            let delta = 0.5 * response_coef * (21.0 - dist);
            let col_vec = (o2_o1 / dist) * delta;
            //println!("col_vec={}",col_vec);
            q[0].2.translation += col_vec.extend(0.);
            q[1].2.translation -= col_vec.extend(0.);
        }
    }
    }

}