use super::{Part, PartLayout};

#[derive(Debug)]
pub struct PartGrid {
    data: Vec<isize>,
    x: u32,
    y: u32,
    z: u32,
    cx: i32,// Where is the origin in these grid coordinates
    cy: i32,
    cz: i32
}
impl PartGrid {
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

    /// Add a part to the grid, where data represents the index of the part
    pub fn update(&mut self, part: &Part, layout: PartLayout, data: usize) {
        let (ul, lr) = part.get_bbox(layout);
        let underflow_x = (ul.x + self.cx).min(0);
        let underflow_y = (ul.y + self.cy).min(0);
        let underflow_z = (ul.z + self.cz).min(0);
        let overflow_x = (lr.x + self.cx).max(self.x as i32);
        let overflow_y = (lr.y + self.cy).max(self.y as i32);
        let overflow_z = (lr.z + self.cz).max(self.z as i32);
        if underflow_x < 0 || underflow_y < 0 || underflow_z < 0 || overflow_x as u32 >= self.x || overflow_y as u32 >= self.y || overflow_z as u32 >= self.z {
            self.reshape(
                (overflow_x - underflow_x) as u32+1,
                (overflow_y - underflow_y) as u32+1,
                (overflow_z - underflow_z) as u32+1,
                self.cx - underflow_x,
                self.cy - underflow_y,
                self.cz - underflow_z,
            );
        } 
        for block in part.get_blocks(layout) {
            let index = self.get_index(block.x, block.y, block.z) as usize;
            self.data[index] = data as isize;
        }
    }

    fn reshape(&mut self, x: u32, y: u32, z: u32, cx: i32, cy: i32, cz: i32) {
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
}

pub struct IndexedIterator<'a> {
    index: u32,
    grid: &'a PartGrid
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