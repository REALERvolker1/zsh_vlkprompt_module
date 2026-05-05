#[allow(unused_imports)]
use super::*;

impl linklist {
    /// Get the first node in the list, or `None` if is is null
    /// # Safety
    /// Caller asserts the pointer is not dangling or contended
    pub const unsafe fn firstnode(&self) -> Option<&linknode> {
        if self.first.is_null() {
            return None;
        }
        // SAFETY: We null-checked
        Some(unsafe { self.first.as_ref_unchecked() })
    }
}
impl linknode {
    /// Return the `next` link of the node.  Returns `null_mut()` if the
    /// node is the list terminator.
    pub const fn nextnode(&self) -> *mut linknode {
        self.next
    }

    /// Return the `prev` link of the node.
    pub const fn prevnode(&self) -> *mut linknode {
        self.prev
    }
}
