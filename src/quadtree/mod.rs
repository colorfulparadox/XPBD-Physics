//use rand::Rng;
use bevy::prelude::*;

pub mod point;
pub use point::*;

#[derive(Debug)]
pub struct Rect {
    pos: Vec2,
    size: Vec2,
    half_size: Vec2,
    right: f32,
    left: f32,
    top: f32,
    bottom: f32
}

impl Default for Rect {
    fn default() -> Self {
        Rect {
            pos: Vec2 { x: 0.0, y: 0.0 },
            size: Vec2 { x: 1.0, y: 1.0 },
            half_size: Vec2 { x: 1.0, y: 1.0 } / 2.,
            right: 0.0 + 1.,
            left: 0.0,
            top: 0.0,
            bottom: 0.0 - 1.,
        }
    }
}

impl Rect {
    fn new(pos: Vec2, size: Vec2) -> Self {
        Rect { 
            pos: pos, 
            size: size,
            half_size: size / 2.,
            right: pos.x + size.x,
            left: pos.x,
            top: pos.y,
            bottom: pos.y - size.y, 
        }
    }

    fn contains(&self, p: Vec2) -> bool {
        (self.left <= p.x && p.x < self.right) && (self.top >= p.y && p.y > self.bottom)
    }

    
    fn overlaps(&self, r: &Rect) -> bool {
        r.left < self.right && 
        self.left < r.right &&
        r.top > self.bottom && 
        self.top > r.bottom
    }

    /*
    fn contains_rect(&self, r: &Rect) -> bool {
        (r.pos.x >= self.pos.x)
            && (r.pos.x + r.size.x < self.pos.x + self.size.x)
            && (r.pos.y >= self.pos.y)
            && (r.pos.y + r.size.y < self.pos.y + self.size.y)
    }
    
    fn overlaps(&self, r: &Rect) -> bool {
        (self.pos.x < r.pos.x + r.size.x)
            && (self.pos.x + self.size.x >= r.pos.x)
            && (self.pos.y < r.pos.y + r.size.y)
            && (self.pos.y + self.size.y >= r.pos.y)
    }
    */
}

#[derive(Debug, Resource)]
pub struct QuadTree {
    pub parent: Option<Box<QuadTree>>,
    pub quads: [Option<Box<QuadTree>>; 4],
    pub height: usize,
    pub bounds: Rect,
    pub capacity: usize,
    pub elements: Vec<(Option<Entity>, Vec2)>,
    pub divided: bool,
}

impl QuadTree {
    pub fn new(pos: Vec2, size: Vec2, capacity: usize) -> Self {
        Self {
            parent: None,
            quads: [None, None, None, None],
            height: 0,
            bounds: Rect::new(pos, size),
            capacity: capacity,
            elements: Vec::new(),
            divided: false,
        }
    }

    pub fn new_branch(parent: Option<Box<QuadTree>>, bounds: Rect, capacity: usize, height: usize) -> Self {
        Self {
            parent: parent,
            quads: [None, None, None, None],
            height: height,
            bounds: bounds,
            capacity: capacity,
            elements: Vec::new(),
            divided: false,
        }
    }

    pub fn insert_point(&mut self, ent: &Entity, point: Vec2) -> bool {
        if !self.bounds.contains(point) {
            //println!("does not contain");
            return false;
        }

        if self.divided == false {
            //println!("push element");
            self.elements.push((Some(*ent), point));
        } else {
            //println!("push element to other branches");
            return self.quads[0].as_mut().unwrap().insert_point(ent, point) ||
            self.quads[1].as_mut().unwrap().insert_point(ent, point) ||
            self.quads[2].as_mut().unwrap().insert_point(ent, point) ||
            self.quads[3].as_mut().unwrap().insert_point(ent, point);
        }


        if self.elements.len() >= self.capacity && self.height <= 4 {
            self.subdivide();
        }

        return true;
    }

    pub fn insert(&mut self, ent: &Entity, rect: &Rect) -> bool {

        if !self.bounds.overlaps(rect) {
            //println!("does not contain");
            return false;
        }

        if self.divided == false {
            //println!("push element");
            self.elements.push((Some(*ent), rect.pos));
        } else {
            //println!("push element to other branches");
           // return self.quads[0].as_mut().unwrap().insert(ent, point) ||
            //self.quads[1].as_mut().unwrap().insert(ent, point) ||
            //self.quads[2].as_mut().unwrap().insert(ent, point) ||
            //self.quads[3].as_mut().unwrap().insert(ent, point);
        }


        if self.elements.len() >= self.capacity && self.height <= 4 {
            self.subdivide();
        }

        return true;
    }

    pub fn query(&self, point: Vec2) -> Vec<(Option<Entity>, Vec2)> {
        let in_area = self.bounds.contains(point);

        if !in_area {
            return Vec::new();
        }
        
        if !self.divided {
            return self.elements.clone();
        }

        for quad_option in self.quads.iter() {
            let elements = quad_option.as_ref().unwrap().query(point);
            if !elements.is_empty() {
                return elements;
            }
        }
        return Vec::new();
    }

    pub fn query_area(&self, rect: &Rect) -> Vec<(Option<Entity>, Vec2)> {
        let in_area = self.bounds.overlaps(rect);

        let mut elements = self.elements.clone();

        if !in_area {
            return Vec::new();
        }
        
        //if !self.divided {
        //    return elements;
        //}

        for quad_option in self.quads.iter() {
            if quad_option.is_none() {
                break;
            }
            let quad = quad_option.as_ref().unwrap();
            let result = quad.query_area(rect);
            if result.is_empty() {
                continue
            }
            elements.extend(&result);
        }
        return elements;
    }

    pub fn subdivide(&mut self) {
        //println!("subdivide time!");

        self.divided = true;
        
        let parent = None;
        let new_height = self.height+1;
        let new_size = self.bounds.size/2.;
        //AAAAAAAAAAAAAH 
        let top_right_bound: Rect = Rect::new(self.bounds.pos, new_size);
        self.quads[0] = Some(Box::new(QuadTree::new_branch(parent, top_right_bound, self.capacity, new_height)));

        let top_left_bound: Rect = Rect::new(self.bounds.pos + Vec2::new(new_size.x, 0.), new_size);
        self.quads[1] = Some(Box::new(QuadTree::new_branch(None, top_left_bound, self.capacity, new_height)));

        let bottom_right_bound: Rect = Rect::new(self.bounds.pos + Vec2::new(0., -new_size.y), new_size);
        self.quads[2] = Some(Box::new(QuadTree::new_branch(None, bottom_right_bound, self.capacity, new_height)));

        let bottom_left_bound: Rect = Rect::new(self.bounds.pos + Vec2::new(new_size.x, -new_size.y), new_size);
        self.quads[3] = Some(Box::new(QuadTree::new_branch(None, bottom_left_bound, self.capacity, new_height)));

        //TAKE OLD POINTS AND PUT THEM IN NEW TREES 
        let elements = self.elements.clone();
        self.elements.clear();
        for (ent, point) in elements.iter() {
            //self.quads[0].as_mut().unwrap().insert(&ent.unwrap(), *point);
            //self.quads[1].as_mut().unwrap().insert(&ent.unwrap(), *point);
            //self.quads[2].as_mut().unwrap().insert(&ent.unwrap(), *point);
            //self.quads[3].as_mut().unwrap().insert(&ent.unwrap(), *point);
            self.insert_point(&ent.unwrap(),*point);
        }
    }

    pub fn clear(&mut self) {
        self.divided = false;
        self.elements.clear();
        for quad_option in self.quads.iter_mut() {
            if quad_option.is_none() {
                break;
            }
            let quad = quad_option.as_mut().unwrap();
            quad.clear();
            //drop(quad);
        }
        //self.quads = [None, None, None, None];
    }

}


pub fn draw_quad_rects(
    quad_tree: Res<QuadTree>, 
    mut gizmos: Gizmos
) {
    draw_quad_rects_helper(&quad_tree, &mut gizmos);
}

fn draw_quad_rects_helper(
    quad_tree: &QuadTree, 
    gizmos: &mut Gizmos
) {
    if quad_tree.divided == false {
        let pos = quad_tree.bounds.pos + Vec2::new(quad_tree.bounds.size.x/2., -quad_tree.bounds.size.y/2.);
        gizmos.rect_2d(pos, 0., quad_tree.bounds.size, Color::WHITE);
    }

    for option_quad in quad_tree.quads.iter() {
        if option_quad.is_none() {
            continue
        }
        let quad = option_quad.as_ref().unwrap();
        draw_quad_rects_helper(quad, gizmos);
    }
}

pub fn test_setup(
    mut commands: Commands
) {
    let quad_tree = QuadTree::new(Vec2::new(0.,0.), Vec2::new(1000., 800.), 10);

    //let mut rng = rand::thread_rng();
    commands.spawn((point::PointParent, Name::new(format!("point holder"))));
    /* 
    for i in 0..300 {
        let rand_x: f32 = rng.gen_range(1..=511) as f32;
        let rand_y: f32 = -rng.gen_range(1..=511) as f32;
        let ent = commands.spawn((
            Point,
            Transform::from_translation(Vec3::new(rand_x, rand_y, 1.)),
            Name::new(format!("point_{i}"))
        )).id();
        commands.entity(parent).add_child(ent);
        
        quad_tree.insert(&ent, Vec2::new(rand_x, rand_y));
    }
    */
    commands.insert_resource(quad_tree);
}

pub fn print_tree(
    quad_tree: Res<QuadTree>,
) {
    println!("------------------------------------------------------------");
    print_tree_helper(&quad_tree);
    println!("------------------------------------------------------------");
}

pub fn print_tree_helper(
    quad_tree: &QuadTree
) {
    println!("BRANCH height={} element={} pos={:?} size={:?}", quad_tree.height, quad_tree.elements.len(), quad_tree.bounds.pos, quad_tree.bounds.size);

    for option_quad in quad_tree.quads.iter() {
        if option_quad.is_none() {
            continue;
        }
        let quad = option_quad.as_ref().unwrap();
        print_tree_helper(quad);
    }
}