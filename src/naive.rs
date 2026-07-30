use std::{println, time::SystemTime};

use winit::event_loop::EventLoop;

use crate::{Export, Star, Vector, render::App};
use rand::{ RngExt};


struct Uni{
    stars: Vec<Star>,
    itr: usize,
    time: SystemTime,
}
impl Uni {
    pub fn new()->Self{
        let mut rng = rand::rng();

        let stars: Vec<Star> = (0..1000).map(|_| Star::new(rng.random_range(0..800) as f32,rng.random_range(0..600) as f32)).collect();


        let time_now = SystemTime::now();
        Self { stars: stars , itr: 0, time:time_now}
    }
    pub fn gravity(&mut self){
        for s1 in 0..self.stars.len(){
            self.stars[s1].force = Vector::zero_vec();
            for s2 in 0..self.stars.len(){
                if self.stars[s1].samsies(&self.stars[s2]){
                    continue;
                }
                let dif_vec = &self.stars[s2].pos-&self.stars[s1].pos;
                
                let softening_sq = 0.1; 
                let rs = &dif_vec*&dif_vec;

                let rs_soft = rs + softening_sq;


                let mass = 1000.;
                // ! For now lets ignore the constant
                let grav_mag = mass/(rs_soft)*10_f32.powf(-1.);
                let n_vec =&dif_vec*&(1./(rs_soft.sqrt()));
                let grav_vec = &n_vec*&grav_mag;
                self.stars[s1].add_force(&grav_vec);
            }
        }
    }
    pub fn new_pos(&mut self){
        for x in &mut self.stars{
            x.update_pos();
        }
    }
    
}
impl Export for Uni {
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

        if self.itr % 10 == 1{
            let time = self.time.elapsed().unwrap().as_secs();
            let fps = self.itr as u64/self.time.elapsed().unwrap().as_secs();
            println!("It took {} seconds or {} fps", time, fps)
        }
    }
}



pub fn run(){
    let univ = Uni::new();
    let event_loop = EventLoop::with_user_event().build().unwrap();
    let mut app = App::new(&event_loop, univ);
    event_loop.run_app(&mut app).unwrap();
}