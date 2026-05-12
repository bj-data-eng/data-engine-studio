//! Minimal project-owned replacement for the historical `dispatch` crate.
//!
//! This crate intentionally implements only the API surface currently required
//! by our macOS dependency graph: `Queue::main().exec_sync(...)`.
//!
//! The original crate exposes a broad wrapper over Grand Central Dispatch. We
//! keep this shim narrow so any future upstream request for more Dispatch API
//! fails at compile time and can be audited before it is added.

#![warn(missing_docs)]

use std::any::Any;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::panic::{self, AssertUnwindSafe};
use std::ptr::NonNull;

/// A handle to Apple's main dispatch queue.
///
/// This replacement intentionally supports only the main queue because that is
/// the only `dispatch` API used by the current `objc2-foundation` dependency
/// path. Additions should be made only when a concrete upstream caller requires
/// them and after auditing the new unsafe boundary.
#[derive(Clone, Copy, Debug)]
pub struct Queue {
    ptr: NonNull<DispatchObject>,
    _not_send_sync: PhantomData<*mut ()>,
}

impl Queue {
    /// Returns the serial dispatch queue associated with the application's main
    /// thread.
    pub fn main() -> Self {
        let ptr = dispatch_get_main_queue();
        let ptr = NonNull::new(ptr).expect("libdispatch returned a null main queue");
        Self {
            ptr,
            _not_send_sync: PhantomData,
        }
    }

    /// Executes `work` synchronously on this queue and returns its result.
    ///
    /// This is a narrow compatibility implementation for
    /// `objc2_foundation::run_on_main`. Callers must not call this from the
    /// target queue unless the higher-level caller has already detected that
    /// case and runs the closure directly.
    pub fn exec_sync<T, F>(&self, work: F) -> T
    where
        F: Send + FnOnce() -> T,
        T: Send,
    {
        let mut state = SyncCallState {
            work: Some(work),
            result: None,
            panic: None,
        };

        // SAFETY: `state` lives on this stack until `dispatch_sync_f` returns.
        // The callback is synchronous, receives the same pointer, and stores
        // either a result or captured panic in `state` without unwinding across
        // the C boundary.
        unsafe {
            dispatch_sync_f(
                self.ptr.as_ptr(),
                (&mut state as *mut SyncCallState<F, T>).cast::<c_void>(),
                run_sync_call::<F, T>,
            );
        }

        if let Some(payload) = state.panic {
            panic::resume_unwind(payload);
        }

        state
            .result
            .expect("dispatch_sync_f returned before running its callback")
    }
}

struct SyncCallState<F, T> {
    work: Option<F>,
    result: Option<T>,
    panic: Option<Box<dyn Any + Send>>,
}

extern "C" fn run_sync_call<F, T>(context: *mut c_void)
where
    F: FnOnce() -> T,
{
    // SAFETY: `exec_sync` passes a valid mutable pointer to `SyncCallState`.
    // `dispatch_sync_f` calls this function synchronously before `exec_sync`
    // returns, so the pointer cannot outlive its stack allocation.
    let state = unsafe { &mut *context.cast::<SyncCallState<F, T>>() };
    let call = || {
        let work = state
            .work
            .take()
            .expect("dispatch_sync_f invoked callback more than once");
        work()
    };

    match panic::catch_unwind(AssertUnwindSafe(call)) {
        Ok(result) => state.result = Some(result),
        Err(payload) => state.panic = Some(payload),
    }
}

#[repr(C)]
struct DispatchObject {
    _private: [u8; 0],
}

type DispatchQueue = *mut DispatchObject;
type DispatchFunction = extern "C" fn(*mut c_void);

extern "C" {
    static _dispatch_main_q: DispatchObject;

    fn dispatch_sync_f(queue: DispatchQueue, context: *mut c_void, work: DispatchFunction);
}

fn dispatch_get_main_queue() -> DispatchQueue {
    // SAFETY: `_dispatch_main_q` is provided by libdispatch on Apple platforms.
    unsafe { &_dispatch_main_q as *const DispatchObject as DispatchQueue }
}
