use ::core::mem::MaybeUninit;

/// This type is unsound. It is meant to be used exclusively from callbacks
/// called from zsh itself. Don't be a dumbass with it.
pub struct Static<T>(pub T);
impl<T> Static<T> {
    pub const fn ptr(&mut self) -> *mut T {
        &raw mut self.0
    }
    pub const fn ptr_const(&self) -> *const T {
        &raw const self.0
    }
}
unsafe impl<T> Send for Static<T> {}
unsafe impl<T> Sync for Static<T> {}

impl<T> Static<MaybeUninit<T>> {
    /// Must be called in the setup callback
    pub fn setup(&mut self, val: T) -> &mut T {
        self.0.write(val)
    }
    pub fn get(&mut self) -> &mut T {
        unsafe { self.0.assume_init_mut() }
    }
}
