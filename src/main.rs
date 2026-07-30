use std::{ops::{Add, Mul, Neg, Sub}, vec};



mod naive;
mod render;
mod bh;
use crate::bh::MASS;
const DELTA_T:f32 = 0.01;
const N: usize = 10000;

#[derive(Clone,Copy,Debug)]
struct Vector{
    x: f32,
    y: f32,
    z: f32,
}
impl Vector {
    pub fn new(x:f32,y:f32,z:f32)->Self{
        Vector { x, y, z }
    }
    pub fn zero_vec()->Self{
        Vector { x: 0., y:0., z: 0. }

    }
    fn flat(&self)->Vec<f32>{
        vec![self.x ,self.y,self.z]
    }
}
impl Sub<&Vector> for &Vector{
    type Output = Vector;

    fn sub(self, rhs: &Vector) -> Self::Output {
        Vector::new(self.x-rhs.x, self.y-rhs.y,self.z-rhs.z)
    }
}
impl Mul<&Vector> for &Vector{
    type Output = f32;

    fn mul(self, rhs: &Vector) -> Self::Output {
        self.x*rhs.x + self.y*rhs.y+self.z*rhs.z
    }
}
impl Add<&Vector> for &Vector{
    type Output = Vector;

    fn add(self, rhs: &Vector) -> Self::Output {
        Vector::new(self.x+rhs.x, self.y+rhs.y,self.z+rhs.z)
    }
}
impl Mul<&f32> for &Vector{
    type Output = Vector;

    fn mul(self, rhs: &f32) -> Self::Output {
        Vector::new(self.x*rhs,self.y*rhs, self.z*rhs)
    }
}
impl Neg for Vector{
    type Output = Vector;
    fn neg(self)->Self::Output{
        Vector::new(-self.x, -self.y, -self.z)
    }
}
// ! please dear god no this is so fucking slow
#[derive(Clone,Debug,Copy)]
struct Star{
    pub pos: Vector,
    force: Vector,
    speed: Vector,
    mass: f32,
}
impl Star{
    pub fn new(x:f32,y:f32)->Self{

        Self { pos: Vector::new(x, y, 0.), 
            force: Vector::zero_vec(), // Vector::new(rng.random_range(-2..2) as f32,rng.random_range(-2..2) as f32 , 0.)
            speed: Vector::zero_vec(),
            mass:MASS}
    }
    pub fn flat(&self)->Vec<f32>{
        vec![self.pos.x,self.pos.y,0.,0.]
    }
    pub fn add_force(&mut self, new_force: &Vector){
        self.force = &self.force + new_force;
    }
    // ! implement runge-kutta 4 or someshit like that ffs euler my ass 
    pub fn update_pos(&mut self){
        self.speed = &self.speed + &(&self.force*&DELTA_T);
        self.pos = &self.pos + &(&self.speed*&DELTA_T);
        if self.pos.x > 1000. || self.pos.y > 800.{
            self.pos = &self.pos - &(&self.speed*&DELTA_T);
            self.speed = -self.speed;
        }
    }
    pub fn samsies(&self, other: &Self)->bool{
        return self.pos.x == other.pos.x && self.pos.y == other.pos.y
    }
    pub fn fake(mass:f32,com: Vector)->Self{
         Self { pos: com, 
            force: Vector::zero_vec(), // Vector::new(rng.random_range(-2..2) as f32,rng.random_range(-2..2) as f32 , 0.)
            speed: Vector::zero_vec(),
            mass:mass}
    }
}


pub trait Export{
    fn export_stars(&self)->Vec<f32>;
    fn update(&mut self);
}


fn main() {
    //naive::run();
    bh::run();
}

