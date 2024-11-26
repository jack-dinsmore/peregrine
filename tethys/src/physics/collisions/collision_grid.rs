use cgmath::Vector3;

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
        if x >=self.x as i32-self.cx || y >= self.z as i32-self.cz || z >= self.z as i32-self.cz { return None; }
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

    pub(crate) fn check_line(&self, line: LineCollider) -> CollisionReport {
        unimplemented!()
    }
    
    pub(crate) fn check_point(&self, p: Vector3<f64>) -> CollisionReport {
        unimplemented!()
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