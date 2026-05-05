#[allow(unused_imports)]
use super::*;

pub unsafe fn sigmsg(sig: c_int) -> &'static CStr {
    if sig >= 0 && sig <= SIGCOUNT {
        // SAFETY: This is the exact same indexing rule as the C macro,
        // with an extra negative guard so accidental bad inputs do not
        // index before the exported table.
        return unsafe { CStr::from_ptr(sig_msg[sig as usize]) };
    }
    c"unknown signal"
}

// pub const fn HOOKDEF(name: *mut c_char, func: )

// SIGNALS

pub fn TRAPCOUNT() -> c_int {
    VSIGCOUNT + SIGRTMAX() - SIGRTMIN() + 1
}

pub fn SIGNUM(x: c_int) -> c_int {
    if x >= VSIGCOUNT {
        x - VSIGCOUNT + SIGRTMIN()
    } else {
        x
    }
}
pub fn SIGIDX(x: c_int) -> c_int {
    let min = SIGRTMIN();

    if x >= min && x <= SIGRTMAX() {
        x - min + VSIGCOUNT
    } else {
        x
    }
}
pub unsafe fn child_block() -> sigset_t {
    unsafe { signal_block(sigchld_mask) }
}
pub unsafe fn child_unblock() -> sigset_t {
    unsafe { signal_unblock(sigchld_mask) }
}
pub unsafe fn winch_block() -> sigset_t {
    unsafe { signal_block(signal_mask(SIGWINCH)) }
}
pub unsafe fn winch_unblock() -> sigset_t {
    unsafe { signal_unblock(signal_mask(SIGWINCH)) }
}
/// ignore a signal
pub unsafe fn signal_ignore(S: c_int) -> usize {
    unsafe { signal(S, SIG_IGN) }
}
/// return a signal to it default action
pub unsafe fn signal_default(S: c_int) -> usize {
    unsafe { signal(S, SIG_DFL) }
}

/// Use a circular queue to save signals caught during
/// critical sections of code.  You call queue_signals to
/// start queueing, and unqueue_signals to process the
/// queue and stop queueing.  Since the kernel doesn't
/// queue signals, it is probably overkill for zsh to do
/// this, but it shouldn't hurt anything to do it anyway.
pub unsafe fn run_queued_signals() {
    unsafe {
        while queue_front != queue_rear {
            queue_front = (queue_front + 1) % MAX_QUEUE_SIZE;
            let oset = signal_setmask(signal_mask_queue[queue_front as usize]);
            zhandler(signal_queue[queue_front as usize]);
            signal_setmask(oset);
        }
    }
}

unsafe fn queueing_enabled_offset<const OFFSET: c_int>() -> c_int {
    let old = unsafe { queueing_enabled };
    unsafe { queueing_enabled += OFFSET };
    old
}

/// `(queueing_enabled++)`
pub unsafe fn queue_signals() -> c_int {
    unsafe {
        queue_in += 1;
        queueing_enabled_offset::<1>()
    }
}
/// `if (!--queueing_enabled) run_queued_signals()`
pub unsafe fn unqueue_signals() {
    unsafe {
        queue_in -= 1;
        queueing_enabled -= 1;
        if queueing_enabled == 0 {
            run_queued_signals();
        }
    }
}
pub unsafe fn dont_queue_signals() {
    unsafe {
        queue_in = queueing_enabled;
        queueing_enabled = 0;
        run_queued_signals();
    }
}
pub unsafe fn restore_queued_signals(q: c_int) {
    unsafe {
        queue_in = q;
        queueing_enabled = q;
    }
}
pub unsafe fn queue_signal_level() -> c_int {
    unsafe { queueing_enabled }
}
