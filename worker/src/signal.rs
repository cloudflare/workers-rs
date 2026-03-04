use core::sync::atomic::{AtomicU32, Ordering};

/// No signal will ever use this value. This means that we can initialize the
/// signal to this value, and check later to see if it was zeroed or set to some
/// other valid signal. This gives evidence that the runtime supports signals,
/// and this module got the hook to receive them
const RESERVED_SIGNAL_VALUE: u32 = 0xFFFF;

#[allow(non_upper_case_globals)]
#[no_mangle]
static __instance_signal: AtomicU32 = AtomicU32::new(RESERVED_SIGNAL_VALUE);

/// A global signal value used by the runtime to communicate with workers.
/// ```rust
/// // Time consuming loop
/// while condition && !Signal::poll().is_almost_out_of_time() {
///     // ...
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Signal(pub u32);

impl Signal {
    const ALMOST_OUT_OF_TIME_VALUE: u32 = 42;
    /// Retrieve the most recent signal value
    pub fn poll() -> Self {
        Self(std::hint::black_box(
            __instance_signal.load(Ordering::Relaxed),
        ))
    }
    /// Access the raw signal value
    pub fn value(&self) -> u32 {
        self.0
    }
    /// Returns `true` if the runtime has indicated that this worker is nearing
    /// its CPU execution limit, and will be forcefully terminated soon
    pub fn is_almost_out_of_time(&self) -> bool {
        self.value() == Self::ALMOST_OUT_OF_TIME_VALUE
    }
    /// Returns `true` if this worker is capable of receiving signals
    pub fn is_listening(&self) -> bool {
        self.value() != RESERVED_SIGNAL_VALUE
    }
    /// # Safety
    /// Signals are global state managed by the runtime. Overwriting a
    /// signal may interfere with other code that is listening to it, causing
    /// unexpected behavior like non-termination.
    pub unsafe fn write(value: u32) {
        __instance_signal.store(value, Ordering::Relaxed);
    }
}
