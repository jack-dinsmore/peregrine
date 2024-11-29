use std::{alloc::Layout, borrow::Borrow, rc::Rc, sync::{Arc, Mutex}};

pub enum MaybeInstanced<T> {
    Singleton(Arc<T>),
    Instance(Instance<T>),
}

pub struct Container<const CAPACITY: usize, T> {
    data: [*const T; CAPACITY],
    counters: [u32; CAPACITY],
    mutexes: [Mutex<u8>; CAPACITY],
}

pub struct Instance<T> {
    data: *const T,
    counter: Option<*mut u32>,
    mutex: Option<*const Mutex<u8>>,
}

pub struct Loader<'a, const CAPACITY: usize, T> {
    container: &'a Container<CAPACITY, T>,
    load_function: Rc<dyn Fn(usize)->MaybeInstanced<T> + 'a>,
}

impl<T> MaybeInstanced<T> {
    pub fn inner(&self) -> &T {
        match self {
            MaybeInstanced::Singleton(t) => t.as_ref(),
            MaybeInstanced::Instance(t) => t.as_ref(),
        }
    }
}

impl<T> Clone for MaybeInstanced<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Singleton(arg0) => Self::Singleton(arg0.clone()),
            Self::Instance(arg0) => Self::Instance(arg0.clone()),
        }
    }
}

impl<const CAPACITY: usize, T> Clone for Loader<'_, CAPACITY, T> {
    fn clone(&self) -> Self {
        Self {
            container: self.container,
            load_function: self.load_function.clone()
        }
    }
}

impl<T> Instance<T> {
    fn as_ref<'a>(&'a self) -> &'a T {
        unsafe { &*self.data }
    }
    
    pub(crate) fn identifier(&self) -> usize {
        self.data as usize
    }
}
impl<T> Drop for Instance<T> {
    fn drop(&mut self) {
        if let Some(counter) = self.counter {
            unsafe {
                *counter -= 1;
    
                // Drop the data
                if *counter == 0 {
                    std::ptr::drop_in_place(self.data as *mut u8);
                    std::alloc::dealloc(self.data as *mut u8, Layout::new::<T>());
                }
            }
        }
    }
}
impl<T> Clone for Instance<T> {
    fn clone(&self) -> Self {
        if let Some(counter) = self.counter {
            unsafe {
                let _lock = (*self.mutex.unwrap()).lock();
                *counter += 1;
            }
        }
        Self {
            data: self.data.clone(),
            counter: self.counter.clone(),
            mutex: self.mutex.clone()
        }
    }
}

impl<const CAPACITY: usize, T> Container<CAPACITY, T> {
    pub fn new() -> Self {
        Self {
            data: [const {std::ptr::null()}; CAPACITY],
            counters: [0; CAPACITY],
            mutexes: [const {Mutex::new(0)}; CAPACITY],
        }
    }

    pub fn loader<'a>(&'a self, load_function: impl Fn(usize)-> MaybeInstanced<T> + 'a) -> Loader<'a, CAPACITY, T> {
        Loader {
            container: self,
            load_function: Rc::new(load_function)
        }
    }

    pub fn borrow<'a>(&'a self, index: usize, load_function: &(dyn Fn(usize)->MaybeInstanced<T> + 'a)) -> MaybeInstanced<T> {
        {
            let _lock = self.mutexes[index].lock();
            let self_ptr = self as *const Self as *mut Self;
            if self.counters[index] == 0 {
                let model = match load_function(index) {
                    MaybeInstanced::Singleton(data) => Box::new(Arc::try_unwrap(data).unwrap_or_else(|_| panic!("Your load function must return objects which are not borrowed by anyone else"))),
                    MaybeInstanced::Instance(_) => panic!("Your ModelContainer load_function must return singletons only")
                };
                unsafe {
                    (*self_ptr).data[index] = Box::into_raw(model);// Leak the model pointer
                }
            }
            unsafe {
                (*self_ptr).counters[index] += 1;
            }
        }
        MaybeInstanced::Instance(Instance {
            data: self.data[index],
            mutex: Some(&self.mutexes[index] as *const Mutex<u8>),
            counter: Some(&self.counters[index] as *const u32 as *mut u32),
        })
    }
}

impl<'a, const CAPACITY: usize, T> Loader<'a, CAPACITY, T> {
    pub fn borrow(&self, index: usize) -> MaybeInstanced<T> {
        self.container.borrow(index, self.load_function.borrow())
    }
}