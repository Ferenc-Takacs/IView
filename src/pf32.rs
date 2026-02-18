
use std::ops::{Add,Sub,Mul,Div,AddAssign,SubAssign,MulAssign};
use std::fmt;

#[derive(Clone, Copy, PartialEq)]
pub struct Pf32{ pub x: f32, pub y: f32, }
impl Default for Pf32 { fn default() -> Self { Self { x: 0.0, y: 0.0 } } }
impl Into<(f32,f32)> for Pf32 { fn into(self) -> (f32,f32) { ( self.x, self.y ) } }
impl Into<Pf32> for (f32,f32) { fn into(self) -> Pf32 { Pf32{ x:self.0, y:self.1 } } }
impl Into<(i32,i32)> for Pf32 { fn into(self) -> (i32,i32) { ( (self.x+0.5) as i32, (self.y+0.5) as i32 ) } }
impl Into<Pf32> for (i32,i32) { fn into(self) -> Pf32 { Pf32{ x:self.0 as f32, y:self.1 as f32 } } }
impl Into<(u32,u32)> for Pf32 { fn into(self) -> (u32,u32) { ( (self.x+0.5) as u32, (self.y+0.5) as u32 ) } }
impl Into<Pf32> for (u32,u32) { fn into(self) -> Pf32 { Pf32{ x:self.0 as f32, y:self.1 as f32 } } }
impl Into<egui::Vec2> for Pf32 { fn into(self) -> egui::Vec2 { egui::Vec2{ x:self.x, y:self.y } } }
impl Into<Pf32> for egui::Vec2 { fn into(self) -> Pf32 { Pf32{ x:self.x, y:self.y } } }
impl Into<(f64,f64)> for Pf32 { fn into(self) -> (f64,f64) { ( self.x as f64, self.y as f64 ) } }
impl Into<Pf32> for (f64,f64) { fn into(self) -> Pf32 { Pf32{ x:self.0 as f32, y:self.1 as f32 } } }
impl Into<egui::Pos2> for Pf32 { fn into(self) -> egui::Pos2 { egui::Pos2{ x:self.x, y:self.y } } }
impl Into<Pf32> for egui::Pos2 { fn into(self) -> Pf32 { Pf32{ x:self.x, y:self.y } } }
impl fmt::Display for Pf32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
impl fmt::Debug for Pf32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} {}]", self.x, self.y)
}
}impl Add for Pf32 { type Output = Self;
    fn add(self, a: Self) -> Self { Pf32{ x: self.x + a.x, y: self.y + a.y } }
}
impl AddAssign for Pf32 {
    fn add_assign(&mut self, b: Self) { *self = Self { x: self.x + b.x, y: self.y + b.y, }; }
}
impl Sub for Pf32 { type Output = Self;
    fn sub(self, a: Self) -> Self { Pf32{ x: self.x - a.x, y: self.y - a.y } }
}
impl SubAssign for Pf32 {
    fn sub_assign(&mut self, b: Self) { *self = Self { x: self.x - b.x, y: self.y - b.y, }; }
}
impl Mul<f32> for Pf32 { type Output = Self;
    fn mul(self, a: f32) -> Self { Pf32{ x: self.x * a, y: self.y * a } }
}
impl MulAssign<f32> for Pf32 {
    fn mul_assign(&mut self, b: f32) { *self = Self { x: self.x * b, y: self.y * b, }; }
}
impl Mul for Pf32 { type Output = f32;
    fn mul(self, a: Self) -> f32 { self.x * a.x + self.y * a.y }
}
impl Div<f32> for Pf32 { type Output = Self;
    fn div(self, a: f32) -> Self { Pf32{ x: self.x / a, y: self.y / a } }
}
impl Div for Pf32 { type Output = Pf32;
    fn div(self, a: Self) -> Pf32 { Pf32{ x: self.x / a.x, y: self.y / a.y } }
}
impl Pf32 {
    pub fn pf32(x:f32,y:f32)->Pf32 { Pf32{x,y}}
    pub fn hypot(self, b: Pf32) -> f32 { (self - b).length() }
    pub fn length( self ) -> f32 {  (self.x*self.x+self.y*self.y).sqrt() }
    pub fn min( self, b: Pf32 ) -> Pf32 { Pf32{ x: self.x.min(b.x), y: self.y.min(b.y) } }
    pub fn max( self, b: Pf32 ) -> Pf32 { Pf32{ x: self.x.max(b.x), y: self.y.max(b.y) } }
    pub fn floor( self ) -> Pf32 { Pf32{ x: self.x.floor(), y: self.y.floor() } }
    pub fn even( self ) -> Pf32 { Pf32{ x: (self.x*0.5).floor()*2.0, y: (self.y*0.5).floor()*2.0 } }
    pub fn clamp( self, a: Pf32, b: Pf32 ) -> Pf32 { Pf32{x: self.x.clamp(a.x,b.x) , y: self.y.clamp(a.y,b.y) } }
}

