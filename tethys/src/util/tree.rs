use std::ops::{Deref, DerefMut};

pub struct BinaryTree<T> {
    data: Vec<BinaryTreeNode<T>>,
}

impl<T> BinaryTree<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
        }
    }

    pub fn root<'a>(&'a self) -> BinaryTreeNodeHandle<'a, T> {
        BinaryTreeNodeHandle {
            tree: self,
            index: 0,
        }
    }

    pub fn root_mut<'a>(&'a mut self) -> BinaryTreeNodeHandleMut<'a, T> {
        BinaryTreeNodeHandleMut {
            tree: self,
            index: 0,
        }
    }
}

struct BinaryTreeNode<T> {
    item: T,
    parent: Option<usize>,
    left: Option<usize>,
    right: Option<usize>,
}

#[derive(Clone)]
pub struct BinaryTreeNodeHandle<'a, T> {
    tree: &'a BinaryTree<T>,
    index: usize,
}

pub struct BinaryTreeNodeHandleMut<'a, T> {
    tree: &'a mut BinaryTree<T>,
    index: usize,
}

impl<'a, T> BinaryTreeNodeHandle<'a, T> {
    pub fn parent(&self) -> Option<Self> {
        match self.tree.data[self.index].parent {
            Some(index) => Some(Self {tree: self.tree, index}),
            None => None,
        }
    }
    pub fn left(&self) -> Option<Self> {
        match self.tree.data[self.index].left {
            Some(index) => Some(Self {tree: self.tree, index}),
            None => None,
        }
    }
    pub fn right(&self) -> Option<Self> {
        match self.tree.data[self.index].right {
            Some(index) => Some(Self {tree: self.tree, index}),
            None => None,
        }
    }
}

impl<'a, T> BinaryTreeNodeHandleMut<'a, T> {
    pub fn parent(self) -> Option<Self> {
        match self.tree.data[self.index].parent {
            Some(index) => {
                Some(Self {tree: self.tree, index})
            },
            None => None,
        }
    }
    pub fn left(self) -> Option<Self> {
        match self.tree.data[self.index].left {
            Some(index) => Some(Self {tree: self.tree, index}),
            None => None,
        }
    }
    pub fn right(self) -> Option<Self> {
        match self.tree.data[self.index].right {
            Some(index) => Some(Self {tree: self.tree, index}),
            None => None,
        }
    }
    pub fn insert_left(self, item: T) {
        let data_pointer = &mut self.tree.data as *mut Vec<BinaryTreeNode<T>>;
        let index = self.index;
        if let Some(h) = self.left() {
            h.delete();
        }
        let data = unsafe {&mut *data_pointer};
        let new_index = data.len();
        data.push(BinaryTreeNode{
            item,
            parent: Some(index),
            left: None,
            right: None,
        });
        data[index].left = Some(new_index);
    }
    pub fn insert_right(self, item: T) {
        let data_pointer = &mut self.tree.data as *mut Vec<BinaryTreeNode<T>>;
        let index = self.index;
        if let Some(h) = self.right() {
            h.delete();
        }
        let data = unsafe {&mut *data_pointer};
        let new_index = data.len();
        data.push(BinaryTreeNode{
            item,
            parent: Some(index),
            left: None,
            right: None,
        });
        data[index].right = Some(new_index);
    }
    pub fn delete(self) {
        let mut decrement_count = vec![0; self.tree.data.len()];
        let mut delete_indices = vec![false; self.tree.data.len()];
        let tree = self.tree as *mut BinaryTree<T>;
        {
            let mut process_queue = vec![self];
            while let Some(item) = process_queue.pop() {
                delete_indices[item.index] = true;
                for i in item.index..decrement_count.len() {
                    decrement_count[i] += 1;
                }
                let old_item: Self = unsafe { std::mem::transmute_copy(&item) };
                if let Some(item) = item.left() {
                    process_queue.push(item);
                }
                if let Some(item) = old_item.right() {
                    process_queue.push(item);
                }
            }
        }

        // Update the indices to be correct
        let tree = unsafe { &mut *tree };
        for item in &mut tree.data {
            if let Some(i) = &mut item.parent {
                *i -= decrement_count[*i];
            }
            if let Some(i) = &mut item.left {
                *i -= decrement_count[*i];
            }
            if let Some(i) = &mut item.right {
                *i -= decrement_count[*i];
            }
        }
        
        // Remove everything
        let mut i = 0;
        tree.data.retain(|_| {
            i += 1;
            !delete_indices[i-1]
        });
    }
}

impl<'a, T> Deref for BinaryTreeNodeHandle<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.tree.data[self.index].item
    }
}

impl<'a, T> Deref for BinaryTreeNodeHandleMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.tree.data[self.index].item
    }
}

impl<'a, T> DerefMut for BinaryTreeNodeHandleMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tree.data[self.index].item
    }
}