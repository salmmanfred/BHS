use std::{ panic, println, time::SystemTime, unimplemented, unreachable} ;

use winit::event_loop::EventLoop;

use crate::{Export, N, Star, Vector, render::App};
use rand::RngExt;


const MAXLVL: usize = 4;
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
    fn z(&self)->Complex<f64>{
        Complex { re: self.x as f64, im: self.y as f64 }
    }
    pub fn is_well_separated(&self, other: &Bounds) -> bool {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dist = (dx * dx + dy * dy).sqrt();

        // Standard FMM condition: centers are farther apart than 1.5 - 2.0x box width
        // Adjust `1.5` if you want higher accuracy (2.0) or faster speed (1.2)
        dist > (self.w + other.w) * 0.75 
    }
}

pub const P: usize = 8;

pub fn build_pascal_table() -> [[f64; P + 1]; 2 * P] {
    let mut table = [[0.0; P + 1]; 2 * P];
    for n in 0..(2 * P) {
        table[n][0] = 1.0;
        for k in 1..=n.min(P) {
            table[n][k] = table[n - 1][k - 1] + table[n - 1][k];
        }
    }
    table
}

use num::{Complex, Float};
#[derive(Debug,Clone)]
struct Multipole{
    pub mass: f64,
    multi: [Complex<f64>; P],
    local: [Complex<f64>; P],
}
impl Multipole{
    pub fn new(stars: &Vec<Star>, bounds: &Bounds)->Self{
        let mass: f64 = stars.iter().map(|s| s.mass as f64).sum();
        let z0 = bounds.z();

        let mut aks = [Complex{re: 0_f64, im: 0.};P];
        let local = [Complex{re: 0_f64, im: 0.};P];

        for k in 1..P{
            let mut ak = Complex { re: 0_f64, im: 0. };
            for x in stars{
                let massi = x.mass as f64;
                //mass += massi as f64;
                ak += massi*(x.pos.z()-z0).powi(k as i32);
            }
            ak *= -1./k as f64;
            //mass/=7.;
            aks[k] = ak
        }
        Multipole{
            mass,
            multi: aks,
            local
        }
    }
    pub fn empty()->Self{
        let local = [Complex{re: 0_f64, im: 0.};P];

        Multipole { mass: 0., multi: local, local }
    }

    pub fn m2m(&mut self,source: &Multipole, source_bound: &Bounds, bounds: &Bounds,pascal: &[[f64; 9]; 16]){
        let z0 = source_bound.z();
        let x0 = bounds.z();
        let Z = z0-x0+Complex::new(1_f64,1.);


        self.mass += source.mass;

        for j in 1..P{
            let aj0 = -source.mass/(j as f64)*Z.powi(j as i32);
            let mut ajk = Complex::new(0_f64, 0.);
            for k in 1..=j{
                let comb = pascal[j - 1][k - 1];
                ajk += source.multi[k] * Z.powi(j as i32-k as i32)*comb;
            }
            self.multi[j] += aj0+ajk;
        }


    }
    pub fn l2l(&mut self,source: &Multipole, source_bound: &Bounds, bounds: &Bounds,pascal: &[[f64; 9]; 16]){
        let z0 = source_bound.z();
        let x0 = bounds.z();
        let z = x0-z0+Complex::new(1_f64,1.);

        for i in 0..P{
            for l in i..P{
                self.local[i] += source.local[l]*pascal[l][i]*z.powi(l as i32- i as i32)
            }
        }
    }

    pub fn calc_local(&mut self, source: &Multipole, source_bound: &Bounds, bounds: &Bounds,pascal: &[[f64; 9]; 16]){
        let z0 = source_bound.z();
        let x0 = bounds.z();
        let Z = z0-x0+Complex::new(1_f64,1.);

        let mut b0 = source.mass*(-Z).ln();
        for k in 1..P{
            //let sign = (-1_f64).powf(k as f64);
            let sign = if k % 2 == 0 { 1.0 } else { -1.0 };
            b0 += sign * source.multi[k]/(Z.powi(k as i32))
        }
        self.local[0] += b0;

        for l in 1..P{
            
            let Z_l = Z.powi(l as i32);
            let bl0 = -source.mass/(l as f64 * Z_l);
            let mut bl1: Complex<f64> = Complex::new(0_f64, 0.);
            for k in 1..P{
                //let sign = (-1_f64).powf(k as f64);
                let sign = if k % 2 == 0 { 1.0 } else { -1.0 };
                let comb = pascal[l + k - 1][l];
                bl1 += sign * source.multi[k]* comb/(Z.powi(k as i32)) 
            }
            self.local[l] += bl0 + bl1*(1./Z_l)

        }
    }
    fn local_to_force(&self,star: &mut Star, bounds: &Bounds){
        let z0 = bounds.z();
        let z = star.pos.z();
        let mut wprime = Complex::new(0_f64, 0.);

        for l in 1..P{
            wprime += l as f64 *self.local[l]*(z-z0).powi(l as i32 - 1)
        }
        star.add_force(&Vector::new(-wprime.re as f32, wprime.im as f32, 0.));
    }
}
#[derive(Debug,Clone)]
enum Node{
    
    Internal{
        nodes: Box<[Node;4]>,
        bounds: Bounds,
        multi: Option<Multipole>,

    },
    Leaf{
        star: Vec<Star>,
        bounds: Bounds,
        multi: Option<Multipole>,
        
    },
}
impl Node{
    pub fn new(stars: Vec<Star>,lvl: usize, bounds: Bounds)->Self{
        let mut tree = Self::Internal { nodes: Box::new([
            Self::create_internal(bounds.subdivide(0), lvl),
            Self::create_internal(bounds.subdivide(1), lvl),
            Self::create_internal(bounds.subdivide(2), lvl),
            Self::create_internal(bounds.subdivide(3), lvl),
            ]), bounds, multi: Option::None };

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
            return Self::Leaf { star: Vec::new(), bounds, multi: Option::None };
        }
        let lvl = lvl +1;

        Self::Internal { nodes: Box::new([
            Self::create_internal(bounds.subdivide(0), lvl),
            Self::create_internal(bounds.subdivide(1), lvl),
            Self::create_internal(bounds.subdivide(2), lvl),
            Self::create_internal(bounds.subdivide(3), lvl),
            ]), bounds, multi: Option::None }

    }
    pub fn push_down(&mut self, star: Star){

        match self{
            Self::Internal { nodes, bounds, multi:_ }=>{
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
            Self::Leaf { star: starvec, bounds:_ , multi:_} =>{
                starvec.push(star);
            }
        }
        
    }
    
    

    pub fn p2m(&mut self, pascal: &[[f64; P + 1]; 2 * P]){
        match self {
            Self::Internal { nodes, bounds, multi} =>{
                for node in nodes.iter_mut(){
                    node.p2m(pascal);
                }
                // calculate m2m for this instance
                let mut new_multi: Multipole  = Multipole::empty();

                for node in nodes.iter(){
                    match node {
                        Self::Internal { nodes:_, bounds: source_bound, multi:mb }=>{
                            
                                let source = mb.as_ref().unwrap();

                                new_multi.m2m(&source, source_bound, bounds, pascal);
                                

                            
                            
                        }
                        Self::Leaf { star:_, bounds: source_bound, multi:mb }=>{
                            let source = mb.as_ref().unwrap();

                            new_multi.m2m(&source, source_bound, bounds, pascal);
                        }
                    }
                }
                *multi = Some(new_multi)


                

            },
            Self::Leaf { star, bounds , multi}=>{
                let mlt = Some(Multipole::new(star, bounds));
                *multi = mlt;
                // ! leaf has nothing below it so the p2m calculation goes right here
            }
            
        }
    }
    pub fn get_children(&mut self,)->&mut Box<[Node;4]>{
        match self{
            Self::Internal { nodes, bounds:_ , multi:_}=>{
                
                return nodes;
            }
            Self::Leaf { star:_, bounds:_ , multi:_}=>{
                unreachable!()
            }
        }
    }
    fn is_leaf(&self)->bool{
        match self{
            Self::Internal { nodes:_, bounds:_ , multi:_}=> return false,
            Self::Leaf { star:_, bounds:_ , multi:_} => return true,
        }
    }

    fn flip(&mut self, nodeb: Self, pascal: &[[f64; P + 1]; 2 * P]){

        match self{
            Self::Internal { nodes:_, bounds , multi}=>{
                
                match nodeb{
                    Self::Internal { nodes:_, bounds: bb ,multi:mb}=>{
                        let dist = bounds.dist(&bb);
                        if dist <= bounds.thrs{
                            return
                        }
                        let mutli = multi.as_mut().unwrap();
                        mutli.calc_local(&mb.unwrap(), &bb, bounds,pascal);
                        
                    }
                    _=>{unreachable!()}
                }
            }
            Self::Leaf { star:_, bounds , multi}=>{
                 match nodeb{
                    Self::Leaf { star:_, bounds: bb ,multi:mb}=>{
                        let dist = bounds.dist(&bb);
                        if dist <= bounds.thrs{
                            return
                        }
                        let mutli = multi.as_mut().unwrap();
                        mutli.calc_local(&mb.unwrap(), &bb, bounds,pascal);

                    }
                    _=>{unreachable!()}
                }

            }
        }


    }


    pub fn m2l(&mut self, pascal: &[[f64; P + 1]; 2 * P]){
    // Generate the local expansion by flipping all the multipoles to local for every well seperate multipole
    match self{
        Self::Internal { nodes, bounds:_ ,multi:_}=>{
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
                    // ! can i remove the cloning? 
                    let b = children[b].clone();
                    children[a].flip(b,pascal);
                    
                }
            }
            // now the children have a local expansion
            // lets check deeper
            for x in nodes.iter_mut(){
                x.m2l(pascal);
            }
        }
        Self::Leaf { star:_, bounds:_ ,multi:_}=>{
           return // no m2l on this layer since it has no children and the leaf m2l is done at higher levels ^
        }
    }

    }
    pub fn l2l(&mut self,pascal: &[[f64; P + 1]; 2 * P]){
        // now we distribute all them local expansions that we calculated before from m2l but now down
        match self{
            Self::Internal { nodes, bounds:source_bound , multi}=>{
                for node in nodes.iter_mut(){
                    match node {
                        Self::Internal { nodes:_, bounds: bounds, multi:mb }=>{
                            let source = multi.as_ref().unwrap();
                            let a = mb.as_mut().unwrap();

                            a.l2l(source, source_bound, bounds,pascal);

                        }
                        Self::Leaf { star:_, bounds: bounds, multi:mb }=>{
                            let source = multi.as_ref().unwrap();
                            let a = mb.as_mut().unwrap();

                            a.l2l(source, source_bound, bounds,pascal);
                        }
                    }   
                    node.l2l(pascal);
                }
            }
            Self::Leaf { star:_, bounds:_, multi:_ }=>{
                return // the leaf cant distribute down
            }
        }
    }
    fn is_grandfather(&self)->bool{
        // is self a grandfather then its nodes[0] has to ben an internal but its nodes[0] has to be a leaf
        match self {

            Self::Internal { nodes, bounds:_ ,multi:_}=>{
                // the child is internal
                match &nodes[0] {
                    Self::Internal { nodes, bounds:_,multi:_ }=>{
                        //the childs child is a leaf => its a grandfather otherwise its just not 
                        return nodes[0].is_leaf()
                    }

                    _=>{return false}
                }

            }
            _=>{return false}
        }
    }
    fn leaf_grav(&mut self, nodeb: &Self){
        match self{
            Self::Internal { nodes:_, bounds:_ ,multi:_}=>{
                
                unreachable!()
            }
            Self::Leaf { star, bounds ,multi}=>{
                //let multi = multi.as_mut().unwrap();
                 match nodeb{
                    Self::Leaf { star: sb, bounds: bb,multi:mb }=>{
                        let dist = bounds.dist(bb)*1.3;
                        /*if dist >= bounds.thrs{
                            // if they are too far they will have the multipole effect instead
                            return;
                        }*/
                        for x in star{
                            for y in sb{
                                // the L2P step is implemented in gravity(x,y)
                                gravity(x,y);
                            }
                            // here we do the long dist gravity
                           
                            //multi.local_to_force(x,bounds);
                        }

                    }
                    _=>{unreachable!()}
                }

            }
        }
    }
    pub fn apply_far_field(&mut self) {
        match self {
            Self::Internal { nodes, .. } => {
                for node in nodes.iter_mut() {
                    node.apply_far_field();
                }
            }
            Self::Leaf { star, bounds, multi } => {
                let m = multi.as_ref().unwrap();
                for s in star.iter_mut() {
                    m.local_to_force(s, bounds);
                }
            }
        }
    }
    pub fn gravity(&mut self){
        // This will only update the gravity in the tree and then we have to collapse the tree into a vec 
        // with all the updated values
        let is_grand = self.is_grandfather();

        match self{
            Self::Internal { nodes, bounds ,multi}=>{
                if !is_grand {
                    for node in nodes.iter_mut(){
                        node.gravity();
                    }
                    return
                }
                let mut children: Vec<&mut Node> = Vec::new();
            
                for x in nodes.iter_mut(){
                    let a  = x.get_children();

                    children.extend(a.iter_mut())
                }
                for a in 0..children.len(){
                    for b in 0..children.len(){
                        /*if a == b{
                            continue;
                        }*/
                        let b = children[b].clone();
                        children[a].leaf_grav(&b);
                        
                    }
                }

                

            }

            Self::Leaf { star:_, bounds:_ ,multi:_}=>{
                unreachable!()
            }
        }

    }
    pub fn collapse(&self)->Vec<Star>{
        let mut stars: Vec<Star> = Vec::new();
        match self{
            Self::Internal { nodes, bounds:_, multi:_ }=>{

                for x in nodes.iter(){
                    stars.extend(x.collapse())

                }
            }
            Self::Leaf { star, bounds:_, multi:_ }=>{
                stars.extend(star);
            }
        }
        return stars
    }

    pub fn bounds(&self) -> &Bounds {
        match self {
            Self::Internal { bounds, .. } => bounds,
            Self::Leaf { bounds, .. } => bounds,
        }
    }
    
    pub fn multipole(&self) -> &Multipole {
        match self {
            Self::Internal { multi, .. } => multi.as_ref().unwrap(),
            Self::Leaf { multi, .. } => multi.as_ref().unwrap(),
        }
    }
    pub fn multipole_mut(&mut self) -> &mut Multipole {
        match self {
            Self::Internal { multi, .. } => multi.as_mut().unwrap(),
            Self::Leaf { multi, .. } => multi.as_mut().unwrap(),
        }
    }
    // ! rework this function
    pub fn interact(&mut self, source: &Node, pascal: &[[f64; P + 1]; 2 * P]) {
        let self_bounds = self.bounds().clone();
        let src_bounds = source.bounds();

        // 1. Well-Separated Check (M2L)
        if self_bounds.is_well_separated(src_bounds) {
             if let src_multi = source.multipole() {
                let my_multi = self.multipole_mut();
                    my_multi.calc_local(src_multi, src_bounds, &self_bounds,pascal);
                
            }
            return; // Interaction handled at this level!
        }

        // 2. Not Well-Separated: Match on `self` directly to avoid tuple move conflicts
        match self {
            Self::Leaf { .. } => {
                match source {
                    // Target is Leaf, Source is Leaf -> Near-field P2P
                    Self::Leaf { .. } => {
                        self.leaf_grav(source);
                    }
                    // Target is Leaf, Source is Internal -> Recurse down source children
                    Self::Internal { nodes: src_nodes, .. } => {
                        for src_child in src_nodes.iter() {
                            self.interact(src_child, pascal);
                        }
                    }
                }
            }
            Self::Internal { nodes: my_nodes, .. } => {
                match source {
                    // Target is Internal, Source is Leaf -> Recurse down target children
                    Self::Leaf { .. } => {
                        for my_child in my_nodes.iter_mut() {
                            my_child.interact(source, pascal);
                        }
                    }
                    // Target is Internal, Source is Internal -> Cross-recurse all children
                    Self::Internal { nodes: src_nodes, .. } => {
                        for my_child in my_nodes.iter_mut() {
                            for src_child in src_nodes.iter() {
                                my_child.interact(src_child, pascal);
                            }
                        }
                    }
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
    
    let softening_sq = 1.; 
    let rs = &dif_vec*&dif_vec;

    let rs_soft = rs.sqrt() + softening_sq;


    let mass = star.mass*star2.mass;
    // ! For now lets ignore the constant
    let grav_mag = mass/(rs_soft);//*10_f64.powf(-1.);
    let n_vec =&dif_vec*&(1./(rs.sqrt()));
    let grav_vec = &n_vec*&grav_mag;
    star.add_force(&grav_vec);
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

        let stars: Vec<Star> = (0..N).map(|_| Star::new(rng.random_range(0.0..1000.) as f32,rng.random_range(0.0..800.) as f32)).collect();
        /*let stars: Vec<Star> = (0..N).map(|_| {
            let y_center = 400.;
            let x_center = 500.;
            let r_max = 300.;
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
            ).collect();*/

        Self { stars: stars, itr:0, time: time_now }
    }
    pub fn create_tree(&self)->Node{

        let mut tree = Node::new(self.stars.clone(), 0, Bounds { x: 500., y: 400., w: 1000., h: 800., thrs: 0. });

        tree
    }
    /*pub fn gravity(&mut self){
        let network = self.create_tree();
        for star in &mut self.stars{
            star.force = Vector::zero_vec();
            //network.gravity(star);
        }
    }*/
    pub fn new_pos(&mut self){
        for x in &mut self.stars{
            x.update_pos();
        }
    }

}
impl Export for Funi {
    fn export_stars(&self)->Vec<f32> {
        
        let mut strs = Vec::new();
        for x in self.stars.clone(){
            let mut x = x;
            //x.pos.x *= 1000.;
            //x.pos.y *= 800.;
            strs.extend(x.flat())
        }
        strs
    }
    fn update(&mut self) {
        let pascal = build_pascal_table();
        
        let mut tree = self.create_tree();
        tree.p2m(&pascal);

        //tree.m2l();
        let source_tree = tree.clone();

        tree.interact(&source_tree, &pascal);
        tree.l2l(&pascal);
        tree.apply_far_field();
        //tree.gravity();
        self.stars = tree.collapse();
        self.new_pos();

        self.itr += 1;
        if self.itr % 10 == 1{
            //println!("{:#?}",self.stars[0]);
            let time = self.time.elapsed().unwrap().as_secs();
            if time < 1{
                return
            }
            let fps = self.itr as f64/time as f64;
            println!("It took {} seconds or {} fps", time, fps);
            self.itr = 0;
            self.time = SystemTime::now();
        }

    }
}

pub fn run(){
    let uni = Funi::new();
    let event_loop = EventLoop::with_user_event().build().unwrap();
    let mut app = App::new(&event_loop, uni);
    event_loop.run_app(&mut app).unwrap();
}