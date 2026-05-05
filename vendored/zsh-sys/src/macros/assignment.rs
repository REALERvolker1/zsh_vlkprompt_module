use super::*;

impl asgment {
    /// Assignment is array?
    #[inline]
    pub const fn ASG_ARRAYP(&self) -> bool {
        self.flags & ASG::ASG_ARRAY.as_int() != 0
    }
    /// Assignment has value?
    /// If the assignment is an array, then it certainly has a value --- we
    /// can only tell if there's an explicit assignment.
    #[inline]
    pub fn ASG_VALUEP(&self) -> bool {
        self.ASG_ARRAYP() || !unsafe { (*self.value.scalar).is_null() }
    }
}
