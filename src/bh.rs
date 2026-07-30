use std::{println, time::SystemTime, unreachable} ;

use winit::event_loop::EventLoop;

use crate::{Export, N, Star, Vector, render::App};
use rand::RngExt;


pub const MASS: f32 = 10.;
const THETA: f32 = 0.9;
#[derive(Debug,Copy,Clone)]
struct Bounds{
    x: f32,
    y:f32,
    w: f32,
    h:f32,
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
                return Bounds{
                    x,y,w,h
                }
            }
            1=>{
                let x = self.x+self.w*0.25;
                let y = self.y-self.h*0.25;
                let w = self.w*0.5;
                let h = self.h*0.5;
                return Bounds{
                    x,y,w,h
                }
            }
            2=>{
                let x = self.x-self.w*0.25;
                let y = self.y+self.h*0.25;
                let w = self.w*0.5;
                let h = self.h*0.5;
                return Bounds{
                    x,y,w,h
                }
            }
            3=>{
                let x = self.x+self.w*0.25;
                let y = self.y+self.h*0.25;
                let w = self.w*0.5;
                let h = self.h*0.5;
                return Bounds{
                    x,y,w,h
                }
            }
            _=>{
                unreachable!()
            }
        }
    }
}
#[derive(Debug)]
enum Node{
    Empty,
    Internal{
        com: Vector,
        mass: f32,
        nodes: Box<[Node;4]>,
        bounds: Bounds
    },
    Leaf{
        star: Vec<Star>,
        bounds: Bounds,
        idx: usize,
    },
}
impl Node{
    pub fn new()->Self{
        Self::Empty
    }
    pub fn push(&mut self, star: Star,bounds1: Bounds, idx: usize,depth: usize){
        match self{
            Self::Empty=>{
                *self = Self::Leaf { star: vec![star], bounds: bounds1, idx }
            }
            Self::Internal { com, mass, nodes ,bounds}=>{
                if star.pos.x <= bounds.x{
                    if star.pos.y <= bounds.y{
                        nodes[0].push(star,bounds.subdivide(0),idx,depth+1);
                    }else{
                        nodes[2].push(star,bounds.subdivide(2),idx,depth+1);
                    }
                }
                else{
                    if star.pos.y <= bounds.y{
                        nodes[1].push(star,bounds.subdivide(1),idx,depth+1);

                    }else{
                        nodes[3].push(star,bounds.subdivide(3),idx,depth+1);

                    }
                }
                // Update the com
                let total_mass = *mass + MASS;
                com.x = (com.x * *mass + star.pos.x * MASS) / total_mass;
                com.y = (com.y * *mass + star.pos.y * MASS) / total_mass;
                *mass = total_mass
            }
            Self::Leaf { star: old_star, bounds , idx: old_idx}=>{

                if depth >= 10{
                    // Lets stop the madness
                    // ! leaves should have a com like for this instance atleast I feel like
                    old_star.push(star);
                }
                if old_star.len() >= 2{
                    old_star.push(star);

                    return;
                }
                
                if (old_star[0].pos.x - star.pos.x).abs() < 1e-4 && (old_star[0].pos.y - star.pos.y).abs() < 1e-4 {
                    old_star.push(star);
                    return;
                }
                

                let mut nn = Node::Internal { com: Vector::zero_vec(), mass: 0., 
                    nodes: Box::new([Node::Empty, Node::Empty, Node::Empty, Node::Empty]), 
                    bounds: *bounds };
                nn.push(old_star[0].clone(), *bounds,*old_idx,depth+1);
                nn.push(star, *bounds,idx,depth+1);
                
                
                *self =nn
                
            }
        }
    }
    fn gravity(&self,star: &mut Star){
        match self{
            Self::Empty =>{
                // Vector::zero_vec();
                return;
            },
            Self::Internal { com, mass, nodes, bounds }=>{

                let dif_vec = com-&star.pos;
                
                let distance = (&dif_vec*&dif_vec).sqrt();
                if bounds.w/distance > THETA{
                    for x in nodes.iter(){
                        x.gravity(star);
                    }
                }else{
                    gravity(star, &Star::fake(*mass,*com));
                }

                
            }
            Self::Leaf { star: star2, bounds:_, idx:_ }=>{
                for star3 in star2{
                    gravity(star, star3);
                }
            }

            
        }
        

    }
}
fn gravity(star: &mut Star, star2: &Star){
    if star.samsies(star2){
        return;
    }
    let dif_vec = &star2.pos-&star.pos;
    
    let softening_sq = 0.1; 
    let rs = &dif_vec*&dif_vec;

    let rs_soft = rs + softening_sq;


    let mass = star.mass*star2.mass;
    // ! For now lets ignore the constant
    let grav_mag = mass/(rs_soft)*10_f32.powf(-1.);
    let n_vec =&dif_vec*&(1./(rs_soft.sqrt()));
    let grav_vec = &n_vec*&grav_mag;
    star.add_force(&grav_vec);
}


struct Buni{
    stars: Vec<Star>,
    itr: usize,
    time: SystemTime,
}
impl Buni{
    pub fn new()->Self{
        let mut rng = rand::rng();
        let time_now = SystemTime::now();

        let stars: Vec<Star> = (0..N).map(|_| 
            Star::new(rng.random_range(0..1000) as f32,rng.random_range(0..800) as f32)).collect();

        Self { stars: stars, itr:0, time: time_now }
    }
    pub fn create_network(&self)->Node{
        // ! we could make it just store the id and not the star and save a lot of time from that 

        let mut first_node = Node::new();
        for (i,x) in self.stars.iter().enumerate(){
            first_node.push(*x,Bounds { x: 400., y: 400., w: 1000., h: 800. },i,0);
        }

        first_node
    }
    pub fn gravity(&mut self){
        let network = self.create_network();
        for star in &mut self.stars{
            star.force = Vector::zero_vec();
            network.gravity(star);
        }
    }
    pub fn new_pos(&mut self){
        for x in &mut self.stars{
            x.update_pos();
        }
    }

}

impl Export for Buni {
    fn export_stars(&self)->Vec<f32>{
        let mut strs = Vec::new();
        for x in self.stars.clone(){
            strs.extend(x.flat())
        }
        strs
    }
    fn update(&mut self,){
    
        self.gravity();
        self.new_pos();
         self.itr += 1;
        if self.itr % 10 == 1{
            let time = self.time.elapsed().unwrap().as_secs();
            if time < 1{
                return
            }
            let fps = self.itr as u64/time;
            println!("It took {} seconds or {} fps", time, fps)
        }
        
            
    }
}

pub fn run(){
    let uni = Buni::new();
    let event_loop = EventLoop::with_user_event().build().unwrap();
    let mut app = App::new(&event_loop, uni);
    event_loop.run_app(&mut app).unwrap();

}