use std::{cell::RefCell, ops::Deref, rc::Rc};

use super::unreachr;

pub struct BinaryTree<T> {
    data: Vec<BinaryTreeNode<T>>,
}

impl<T> BinaryTree<T> {
    pub fn new(root_item: T) -> Self {
        Self {
            data: vec![BinaryTreeNode {
                item: root_item,
                parent: None,
                left: None,
                right: None
            }],
        }
    }

    pub fn root<'a>(&'a self) -> BinaryTreeNodeHandle<'a, T> {
        BinaryTreeNodeHandle {
            tree: self,
            index: 0,
        }
    }

    pub fn root_mut<'a>(self, action: impl Fn(BinaryTreeNodeHandleMut<T>)) -> Self {
        let tree = Rc::new(RefCell::new(self));
        action(BinaryTreeNodeHandleMut {
            tree: tree.clone(),
            index: 0,
        });
        RefCell::into_inner(unreachr(Rc::try_unwrap(tree)))
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

pub struct BinaryTreeNodeHandleMut<T> {
    tree: Rc<RefCell<BinaryTree<T>>>,
    index: usize,
}

impl<T> Clone for BinaryTreeNodeHandleMut<T> {
    fn clone(&self) -> Self {
        Self { tree: self.tree.clone(), index: self.index }
    }
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

impl<T> BinaryTreeNodeHandleMut<T> {
    pub fn parent(&self) -> Option<Self> {
        match self.tree.borrow().data[self.index].parent {
            Some(index) => {
                Some(Self {tree: self.tree.clone(), index})
            },
            None => None,
        }
    }
    pub fn left(&self) -> Option<Self> {
        match self.tree.borrow().data[self.index].left {
            Some(index) => Some(Self {tree: self.tree.clone(), index}),
            None => None,
        }
    }
    pub fn right(&self) -> Option<Self> {
        match self.tree.borrow().data[self.index].right {
            Some(index) => Some(Self {tree: self.tree.clone(), index}),
            None => None,
        }
    }
    pub fn insert_left(&self, item: T) {
        if let Some(h) = self.left() {
            h.delete();
        }
        let new_index = self.tree.borrow().data.len();
        self.tree.borrow_mut().data.push(BinaryTreeNode{
            item,
            parent: Some(self.index),
            left: None,
            right: None,
        });
        self.tree.borrow_mut().data[self.index].left = Some(new_index);
    }
    pub fn insert_right(&self, item: T) {
        if let Some(h) = self.right() {
            h.delete();
        }
        let new_index = self.tree.borrow().data.len();
        self.tree.borrow_mut().data.push(BinaryTreeNode{
            item,
            parent: Some(self.index),
            left: None,
            right: None,
        });
        self.tree.borrow_mut().data[self.index].right = Some(new_index);
    }
    pub fn delete(self) {
        let mut decrement_count = vec![0; self.tree.borrow().data.len()];
        let mut delete_indices = vec![false; decrement_count.len()];
        {
            let mut process_queue = vec![self.clone()];
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
        for item in self.tree.borrow_mut().data.iter_mut() {
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
        self.tree.borrow_mut().data.retain(|_| {
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