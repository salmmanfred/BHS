use std::{ panic, println, time::SystemTime, unimplemented, unreachable} ;

use winit::event_loop::EventLoop;

use crate::{Export, N, Star, Vector, render::App};
use rand::RngExt;


const MAXLVL: usize = 3;
#[derive(Debug,Copy,Clone)]
struct Bounds{
    x: f32,
    y:f32,
    w: f32,
    h:f32,
    thrs: f32,
}
impl Bounds{
    pub fn subdivide(&self, i:u32)->Bounds{
        // ! there is probably a better way to do this
        match i{
            0=>{
                let x = self.x-self.w*0.25;
                let y = self.y-self.h*0.25;
                let w = self.w*0.5;
                let h = self.h*0.5;
                //thrs is the distance to the corner aka w*w+h*h 
                let thrs =(w*w+h*h).sqrt();
                return Bounds{
                    x,y,w,h,thrs
                }
            }
            1=>{
                let x = self.x+self.w*0.25;
                let y = self.y-self.h*0.25;
                let w = self.w*0.5;
                let h = self.h*0.5;
                let thrs =(w*w+h*h).sqrt();
                return Bounds{
                    x,y,w,h,thrs
                }
            }
            2=>{
                let x = self.x-self.w*0.25;
                let y = self.y+self.h*0.25;
                let w = self.w*0.5;
                let h = self.h*0.5;
                let thrs =(w*w+h*h).sqrt();
                return Bounds{
                    x,y,w,h,thrs
                }
            }
            3=>{
                let x = self.x+self.w*0.25;
                let y = self.y+self.h*0.25;
                let w = self.w*0.5;
                let h = self.h*0.5;
                let thrs =(w*w+h*h).sqrt();
                return Bounds{
                    x,y,w,h,thrs
                }
            }
            _=>{
                unreachable!()
            }
        }
    }
    fn dist(&self, b: &Bounds)->f32{
       let rx =  self.x-b.x;
       let ry = self.y-b.y;
       return (rx*rx+ry*ry).sqrt()
    }
}
use num::Complex;
pub const P: usize = 10;
#[derive(Debug,Clone)]
struct Multipole{
    pub mass: f32,
    pub cof: [Complex<f32>; P]
}
#[derive(Debug,Clone)]
enum Node{
    
    Internal{
        nodes: Box<[Node;4]>,
        bounds: Bounds,
    },
    Leaf{
        star: Vec<Star>,
        bounds: Bounds,

        
    },
}
impl Node{
    pub fn new(stars: Vec<Star>,lvl: usize, bounds: Bounds)->Self{
        let mut tree = Self::Internal { nodes: Box::new([
            Self::create_internal(bounds.subdivide(0), lvl),
            Self::create_internal(bounds.subdivide(1), lvl),
            Self::create_internal(bounds.subdivide(2), lvl),
            Self::create_internal(bounds.subdivide(3), lvl),
            ]), bounds };

        for star in stars{
            tree.push_down( star);
        }

        return tree

    }
    fn get_leaf_center(&self)->Vec<Vector>{
        unimplemented!()
    }
    fn create_internal(bounds: Bounds, lvl: usize)->Self{
        if lvl == MAXLVL{
            return Self::Leaf { star: Vec::new(), bounds };
        }
        let lvl = lvl +1;

        Self::Internal { nodes: Box::new([
            Self::create_internal(bounds.subdivide(0), lvl),
            Self::create_internal(bounds.subdivide(1), lvl),
            Self::create_internal(bounds.subdivide(2), lvl),
            Self::create_internal(bounds.subdivide(3), lvl),
            ]), bounds }

    }
    pub fn push_down(&mut self, star: Star){

        match self{
            Self::Internal { nodes, bounds }=>{
                if star.pos.x <= bounds.x{
                    if star.pos.y <= bounds.y{
                        nodes[0].push_down(star,);
                    }else{
                        nodes[2].push_down(star,);
                    }
                }
                else{
                    if star.pos.y <= bounds.y{
                        nodes[1].push_down(star,);
                    }else{
                        nodes[3].push_down(star,);
                    }
                }
            }
            Self::Leaf { star: starvec, bounds:_ } =>{
                starvec.push(star);
            }
        }
        
    }
    

    pub fn p2m(&mut self){
        match self {
            Self::Internal { nodes, bounds:_ } =>{
                for node in nodes.iter_mut(){
                    node.p2m();
                }
                // ! p2m calculate for the this instance
            },
            Self::Leaf { star, bounds }=>{
                // ! leaf has nothing below it so the p2m calculation goes right here
            }
            
        }
    }
    pub fn get_children(&mut self,)->&mut Box<[Node;4]>{
        match self{
            Self::Internal { nodes, bounds:_ }=>{
                // ! Please no cloning
                return nodes;
            }
            Self::Leaf { star:_, bounds:_ }=>{
                unreachable!()
            }
        }
    }
    fn is_leaf(&self)->bool{
        match self{
            Self::Internal { nodes:_, bounds:_ }=> return false,
            Self::Leaf { star:_, bounds:_ } => return true,
        }
    }

    fn flip(&mut self, nodeb: Self){

        match self{
            Self::Internal { nodes, bounds }=>{
                
                match nodeb{
                    Self::Internal { nodes:nb, bounds: bb }=>{
                        let dist = bounds.dist(&bb);
                        if dist <= bounds.thrs{
                            return
                        }
                        unimplemented!()
                    }
                    _=>{unreachable!()}
                }
            }
            Self::Leaf { star, bounds }=>{
                 match nodeb{
                    Self::Leaf { star: sb, bounds: bb }=>{
                        let dist = bounds.dist(&bb);
                        if dist <= bounds.thrs{
                            return
                        }
                        unimplemented!()

                    }
                    _=>{unreachable!()}
                }

            }
        }

        unimplemented!()

    }


    pub fn m2l(&mut self, ){
    // Generate the local expansion by flipping all the multipoles to local for every well seperate multipole
    match self{
        Self::Internal { nodes, bounds }=>{
            // all the nodes in this node will NOT be well seperated 
            // but some of the nodes children will be well seperated!

            let mut children: Vec<&mut Node> = Vec::new();
            if nodes[0].is_leaf(){
                return
            }
            for x in nodes.iter_mut(){
                let a  = x.get_children();
                

                children.extend(a.iter_mut())
            }
            //lets find all the childrens relation to one and another and flip them
            for a in 0..children.len(){
                for b in 0..children.len(){
                    if a == b{
                        continue;
                    }
                    let b = children[b].clone();
                    children[a].flip(b);
                    //let childb = &children[b];
                    //Node::flip(childa, childb);
                }
            }
            // now the children have a local expansion
            // lets check deeper
            for x in nodes.iter_mut(){
                x.m2l();
            }
        }
        Self::Leaf { star, bounds }=>{
           return // no m2l on this layer since it has no children and the leaf m2l is done at higher levels ^
        }
    }

    }
    pub fn l2l(&mut self){
        // now we distribute all them local expansions that we calculated before from m2l but now down
        match self{
            Self::Internal { nodes, bounds }=>{
                unimplemented!()
            }
            Self::Leaf { star, bounds }=>{
                return // the leaf cant distribute down
            }
        }
    }
    fn is_grandfather(&self)->bool{
        // is self a grandfather then its nodes[0] has to ben an internal but its nodes[0] has to be a leaf
        match self {

            Self::Internal { nodes, bounds:_ }=>{
                // the child is internal
                match &nodes[0] {
                    Self::Internal { nodes, bounds:_ }=>{
                        //the childs child is a leaf => its a grandfather otherwise its just not 
                        return nodes[0].is_leaf()
                    }

                    _=>{return false}
                }

            }
            _=>{return false}
        }
    }
    fn leaf_grav(nodea: &&mut Self, nodeb: &&mut Self){
        match nodea{
            Self::Internal { nodes, bounds }=>{
                
                unreachable!()
            }
            Self::Leaf { star, bounds }=>{
                 match nodeb{
                    Self::Leaf { star: sb, bounds: bb }=>{
                        let dist = bounds.dist(bb);
                        if dist >= bounds.thrs{
                            // if they are too far they will have the multipole effect instead
                            return;
                        }
                        for x in star{
                            for y in sb{
                                // the L2P step is implemented in gravity(x,y)
                                unimplemented!() //gravity(x,y);
                            }
                        }
                        unimplemented!()

                    }
                    _=>{unreachable!()}
                }

            }
        }
    }
    pub fn gravity(&mut self){
        // This will only update the gravity in the tree and then we have to collapse the tree into a vec 
        // with all the updated values
        let is_grand = self.is_grandfather();

        match self{
            Self::Internal { nodes, bounds }=>{
                if !is_grand {
                    for node in nodes.iter_mut(){
                        node.gravity();
                    }
                }
                for node in nodes.iter_mut(){
                    let children = node.get_children();
                }

            }

            Self::Leaf { star, bounds }=>{
                unreachable!()
            }
        }

    }


    
}
struct Funi{
    stars: Vec<Star>,
    itr: usize,
    time: SystemTime,
}
impl Funi{
    pub fn new()->Self{
        let mut rng = rand::rng();
        let time_now = SystemTime::now();

        let stars: Vec<Star> = (0..N).map(|_| {
            let y_center = 400.;
            let x_center = 500.;
            let r_max = 100.;
            let u: f32 = rng.random_range(0.0..1.0);
            let r = ( u * r_max).max(0.5); 
            let angle: f32 = rng.random_range(0.0..std::f32::consts::TAU);
            
            let x_c = r * angle.cos();
            let y_c = r * angle.sin();
            let x = x_center + x_c;
            let y = y_center + y_c;

            let speed_mag = (17000. / r).sqrt();
            let vx = (y_c / r) * speed_mag;
            let vy = (-x_c / r) * speed_mag;

            let speed = Vector::new(vx, vy, 0.0);

           
            
            
            let mut str = Star::new(x,y);
            if r_max <= 1.0{
                str.mass = 1000.;
            }
            str.speed = speed;
            str

        }
            ).collect();

        Self { stars: stars, itr:0, time: time_now }
    }
    pub fn create_tree(&self)->Node{

        let mut tree = Node::new(vec![Star::new(999.,799.) ], 0, Bounds { x: 500., y: 400., w: 1000., h: 800., thrs: 0. });
        tree.p2m();

        tree
    }
    pub fn gravity(&mut self){
        let network = self.create_tree();
        for star in &mut self.stars{
            star.force = Vector::zero_vec();
            //network.gravity(star);
        }
    }
    pub fn new_pos(&mut self){
        for x in &mut self.stars{
            x.update_pos();
        }
    }

}

pub fn run(){
    let f = Funi::new();
    f.create_tree();
}