use core::f64;

use cgmath::{InnerSpace, Vector3};

use super::{CollisionReport, LineCollider};

pub struct GridCollider {
    data: Vec<isize>,
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub cx: i32,// Where is the origin in these grid coordinates
    pub cy: i32,
    pub cz: i32
}

impl GridCollider {

    pub fn new() -> Self {
        Self {
            x: 0, y: 0, z: 0,
            cx: 0, cy: 0, cz: 0,
            data: Vec::new()
        }
    }

    pub fn with_capacity(x: u32, y: u32, z: u32) -> Self {
        Self {
            x, y, z,
            cx: 0, cy: 0, cz: 0,
            data: Vec::with_capacity((x * y * z) as usize)
        }
    }

    /// Return the actual grid entry corresponding to this position. -1 for null
    pub fn get_entry(&self, x: i32, y: i32, z: i32) -> isize {
        if x < -self.cx || y < -self.cy || z < -self.cz { return -1; }
        if x >=self.x as i32-self.cx || y >= self.z as i32-self.cz || z >= self.z as i32-self.cz { return -1; }
        let index = self.get_index(x, y, z);
        self.data[index as usize]
    }

    /// Returns a mutable reference to the actual grid entry corresponding to this position. None for null
    pub fn get_entry_mut(&mut self, x: i32, y: i32, z: i32) -> Option<&mut isize> {
        if x < -self.cx || y < -self.cy || z < -self.cz { return None; }
        if x >= self.x as i32-self.cx || y >= self.y as i32-self.cy || z >= self.z as i32-self.cz { return None; }
        let index = self.get_index(x, y, z);
        Some(&mut self.data[index as usize])
    }

    pub fn reshape(&mut self, x: u32, y: u32, z: u32, cx: i32, cy: i32, cz: i32) {
        let mut new_data = vec![-1; (x*y*z) as usize];
        for ((xi, yi, zi), data) in self.indexed_iter() {
            let new_index = ((xi+cx) as u32 + (yi+cy) as u32 * x + (zi+cz) as u32 * x * y) as usize;
            new_data[new_index] = data;
        }
        self.x = x;
        self.y = y;
        self.z = z;
        self.cx = cx;
        self.cy = cy;
        self.cz = cz;
        self.data = new_data;
    }

    /// Return the index of the grid entry corresponding to this position
    fn get_index(&self, x: i32, y: i32, z: i32) -> isize {
        ((x+self.cx) + (y+self.cy) * self.x as i32 + (z+self.cz) * self.x as i32 * self.y as i32) as isize
    }

    pub fn indexed_iter<'a>(&'a self) -> IndexedIterator<'a> {
        IndexedIterator {
            index: 0,
            grid: self
        }
    }

    pub fn check_line(&self, line: LineCollider) -> CollisionReport {
        // Find the line segment intersecting the cube
        let mut start_alpha = line.start.unwrap_or(-f64::INFINITY);
        let mut stop_alpha = line.stop.unwrap_or(f64::INFINITY);
        for (vec, normal) in [ // Defined such that vec / normal is the alpha of this plane
            (self.cx as f64 + line.p.x, -line.v.x),
            ((self.x as i32 - self.cx) as f64 - line.p.x, line.v.x),
            (self.cy as f64 + line.p.y, -line.v.y),
            ((self.y as i32 - self.cy) as f64 - line.p.y, line.v.y),
            (self.cz as f64 + line.p.z, -line.v.z),
            ((self.z as i32 - self.cz) as f64 - line.p.z, line.v.z),
        ] {
            let alpha = vec / normal;
            if normal < 0. {
                // Normal is pointing toward p
                start_alpha = start_alpha.max(alpha);
            } else {
                // Normal is pointing away from p
                stop_alpha = stop_alpha.min(alpha);
            }
        }

        // Initialize the algorithm
        let delta = line.v * (stop_alpha - start_alpha);
        let dx = delta.x.abs().round() as i32;
        let dy = delta.y.abs().round() as i32;
        let dz = delta.z.abs().round() as i32;
        let xs = delta.x.signum() as i32;
        let ys = delta.y.signum() as i32;
        let zs = delta.z.signum() as i32;
        let driving_axis = if dx >= dy && dx >= dz {
            0
        } else if dy >= dx && dy >= dz {
            1
        } else {
            2
        };
        let (mut p1, mut p2) = match driving_axis {
            0 => (2 * dy - dx, 2 * dz - dx),
            1 => (2 * dx - dy, 2 * dz - dy),
            2 => (2 * dy - dz, 2 * dx - dz),
            _ => unreachable!(),
        };

        let start: Vector3<i32> = (line.p + line.v * start_alpha).cast().unwrap();
        let mut point = start;
        
        loop {
            let ix = point.x + self.cx;
            if ix < 0 || ix >= self.x as i32 {break;}
            let iy = point.y + self.cy;
            if iy < 0 || iy >= self.y as i32 {break;}
            let iz = point.z + self.cz;
            if iz < 0 || iz >= self.z as i32 {break;}
            let entry = self.get_entry(ix, iy, iz);
            if entry != -1 {
                let mut collide_alpha = line.start.unwrap_or(-f64::INFINITY);
                for (vec, normal) in [ // Defined such that vec / normal is the alpha of this plane
                    (-point.x as f64 + line.p.x, -line.v.x),
                    (point.x as f64 + 1. - line.p.x, line.v.x),
                    (-point.y as f64 + line.p.y, -line.v.y),
                    (point.y as f64 + 1. - line.p.y, line.v.y),
                    (-point.z as f64 + line.p.z, -line.v.z),
                    (point.z as f64 + 1. - line.p.z, line.v.z),
                ] {
                    let alpha = vec / normal;
                    if normal < 0. {
                        // Normal is pointing toward p
                        collide_alpha = collide_alpha.max(alpha);
                    }
                }
                let mut report = CollisionReport::new(Vector3::new(0., 0., 0.,), line.p + line.v * collide_alpha);
                report.index.push(entry);
                return report;
            }
            if (start - point).magnitude2() > delta.magnitude2() as i32 {break;}

            // Update the point
            match driving_axis {
                0 => {
                    point.x += xs;
                    if p1 >= 0 {
                        point.y += ys;
                        p1 -= 2 * dx;
                    }
                    if p2 >= 0 {
                        point.z += zs;
                        p2 -= 2 * dx;
                    }
                    p1 += 2 * dy;
                    p2 += 2 * dz;
                },
                1 => {
                    point.y += ys;
                    if p1 >= 0 {
                        point.x += xs;
                        p1 -= 2 * dy;
                    }
                    if p2 >= 0 {
                        point.z += zs;
                        p2 -= 2 * dy;
                    }
                    p1 += 2 * dx;
                    p2 += 2 * dz;
                },
                2 => {
                    point.z += zs;
                    if p1 >= 0 {
                        point.y += ys;
                        p1 -= 2 * dz;
                    }
                    if p2 >= 0 {
                        point.x += xs;
                        p2 -= 2 * dz;
                    }
                    p1 += 2 * dy;
                    p2 += 2 * dx;
                },
                _ => unreachable!()
            }
        }
        CollisionReport::none()
    }
    
    pub(crate) fn check_point(&self, p: Vector3<f64>) -> CollisionReport {
        let ix = (p.x as i32 + self.cx) as i32;
        if ix < 0 || ix >= self.x as i32 {return CollisionReport::none();}
        let iy = (p.y as i32 + self.cy) as i32;
        if iy < 0 || iy >= self.y as i32 {return CollisionReport::none();}
        let iz = (p.z as i32 + self.cz) as i32;
        if iz < 0 || iz >= self.z as i32 {return CollisionReport::none();}
        let entry = self.get_entry(ix, iy, iz);
        if entry != -1 {
            // I don't know the depth
            let mut report = CollisionReport::new(Vector3::new(0., 0., 0.,), p);
            report.index.push(entry);
            report
        } else {
            CollisionReport::none()
        }
    } 
}

pub struct IndexedIterator<'a> {
    index: u32,
    grid: &'a GridCollider
}

impl Iterator for IndexedIterator<'_> {
    type Item = ((i32, i32, i32), isize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.grid.x * self.grid.y * self.grid.z {return None;}
        let x = (self.index % self.grid.x) as i32 - self.grid.cx;
        let y = ((self.index / self.grid.x) % self.grid.y) as i32 - self.grid.cy;
        let z = ((self.index / (self.grid.x * self.grid.y)) % self.grid.z) as i32 - self.grid.cz;
        let data = self.grid.data[self.index as usize];
        self.index += 1;
        Some(((x, y, z), data))
    }
}