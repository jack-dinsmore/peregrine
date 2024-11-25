use std::{alloc::Layout, borrow::Borrow, rc::Rc, sync::Mutex};

use super::{Model, ModelInner};

pub(super) struct ModelInstance {
    data: *const ModelInner,
    counter: *mut u32,
    mutex: *const Mutex<u8>,
}
impl ModelInstance {
    pub(super) fn as_ref<'a>(&'a self) -> &'a ModelInner {
        unsafe { &*self.data }
    }
    
    pub(crate) fn identifier(&self) -> usize {
        self.data as usize
    }
}
impl Drop for ModelInstance {
    fn drop(&mut self) {
        unsafe {
            (*self.counter) -= 1;

            // Drop the data
            if *self.counter == 0 {
                std::ptr::drop_in_place(self.data as *mut u8);
                std::alloc::dealloc(self.data as *mut u8, Layout::new::<ModelInner>());
            }
        }
    }
}
impl Clone for ModelInstance {
    fn clone(&self) -> Self {
        unsafe {
            let _lock = (*self.mutex).lock();
            *self.counter += 1;
        }
        Self {
            data: self.data.clone(),
            counter: self.counter.clone(),
            mutex: self.mutex.clone()
        }
    }
}

pub struct ModelContainer<const CAPACITY: usize> {
    data: [*const ModelInner; CAPACITY],
    counters: [u32; CAPACITY],
    mutexes: [Mutex<u8>; CAPACITY],
}
impl<const CAPACITY: usize> ModelContainer<CAPACITY> {
    pub fn new() -> Self {
        Self {
            data: [const {std::ptr::null()}; CAPACITY],
            counters: [0; CAPACITY],
            mutexes: [const {Mutex::new(0)}; CAPACITY],
        }
    }

    pub fn loader<'a>(&'a self, load_function: impl Fn(usize)-> Model + 'a) -> ModelLoader<'a, CAPACITY> {
        ModelLoader {
            container: self,
            load_function: Rc::new(load_function)
        }
    }

    pub fn borrow<'a>(&'a self, index: usize, load_function: &(dyn Fn(usize)->Model + 'a)) -> Model {
        {
            let _lock = self.mutexes[index].lock();
            let self_ptr = self as *const Self as *mut Self;
            if self.counters[index] == 0 {
                let model = match load_function(index) {
                    Model::Singleton(data) => Box::new(data),
                    Model::Instance(_) => panic!("Your ModelContainer load_function must return singletons only")
                };
                unsafe {
                    (*self_ptr).data[index] = Box::into_raw(model);// Leak the model pointer
                }
            }
            unsafe {
                (*self_ptr).counters[index] += 1;
            }
        }
        Model::Instance(ModelInstance {
            data: self.data[index],
            mutex: &self.mutexes[index] as *const Mutex<u8>,
            counter: &self.counters[index] as *const u32 as *mut u32,
        })
    }
}

#[derive(Clone)]
pub struct ModelLoader<'a, const CAPACITY: usize> {
    container: &'a ModelContainer<CAPACITY>,
    load_function: Rc<dyn Fn(usize)->Model + 'a>,
}
impl<'a, const CAPACITY: usize> ModelLoader<'a, CAPACITY> {
    pub fn borrow(&self, index: usize) -> Model {
        self.container.borrow(index, self.load_function.borrow())
    }
}