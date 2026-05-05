use super::*;

impl lextok {
    #[inline(always)]
    pub const fn as_int(self) -> c_int {
        self as _
    }
}
impl ASG {
    #[inline(always)]
    pub const fn as_int(self) -> c_int {
        self as _
    }
}
impl REDIR {
    #[inline(always)]
    pub const fn as_int(self) -> c_int {
        self as _
    }
    const fn is_between(self, ge: Self, le: Self) -> bool {
        let s = self.as_int();
        s >= ge.as_int() && s <= le.as_int()
    }
    pub const fn IS_WRITE_FILE(self) -> bool {
        self.is_between(REDIR::REDIR_WRITE, REDIR::REDIR_READWRITE)
    }
    pub const fn IS_APPEND_REDIR(self) -> bool {
        self.IS_WRITE_FILE() && (self.as_int() & 2 != 0)
    }
    pub const fn IS_CLOBBER_REDIR(self) -> bool {
        self.IS_WRITE_FILE() && (self.as_int() & 1 != 0)
    }
    pub const fn IS_ERROR_REDIR(self) -> bool {
        self.is_between(Self::REDIR_ERRWRITE, Self::REDIR_ERRAPPNOW)
    }
    pub const fn IS_READFD(self) -> bool {
        self.is_between(Self::REDIR_READWRITE, Self::REDIR_MERGEIN)
            || self.as_int() == Self::REDIR_INPIPE.as_int()
    }
    pub const fn IS_REDIROP(self) -> bool {
        let s = self.as_int();
        s >= lextok::OUTANG.as_int() && s <= lextok::TRINANG.as_int()
    }
}

impl QT {
    #[inline(always)]
    pub const fn as_int(self) -> c_int {
        self as _
    }
    pub const fn QT_IS_SINGLE(self) -> bool {
        let s = self.as_int();
        s == Self::QT_SINGLE.as_int() || s == Self::QT_SINGLE_OPTIONAL.as_int()
    }
}

impl eprog {
    pub const fn is_real(&self) -> bool {
        self.flags & EF_REAL != 0
    }
    pub const fn is_heap(&self) -> bool {
        self.flags & EF_HEAP != 0
    }
    pub const fn is_map(&self) -> bool {
        self.flags & EF_MAP != 0
    }
    pub const fn is_run(&self) -> bool {
        self.flags & EF_RUN != 0
    }
    /// Shows whether the refcount is 0
    pub const fn should_delete(&self) -> bool {
        self.nref <= 0
    }
}
