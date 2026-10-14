use core::sync::atomic::{AtomicU32, Ordering};

/// No signal will ever use this value. This means that we can initialize the
/// signal to this value, and check later to see if it was zeroed or set to some
/// other valid signal. This gives evidence that the runtime supports signals,
/// and this module got the hook to receive them
const RESERVED_SIGNAL_VALUE: u32 = 0xFFFF;

#[allow(non_upper_case_globals)]
#[no_mangle]
static __instance_signal: AtomicU32 = AtomicU32::new(RESERVED_SIGNAL_VALUE);

/// Value used by the runtime to represent a CPU limit warning signal
const ALMOST_OUT_OF_TIME_VALUE: u32 = 24;

/// Returns `true` if this worker is capable of receiving signals
pub fn is_registered() -> bool {
    __instance_signal.load(Ordering::Relaxed) != RESERVED_SIGNAL_VALUE
}

/// Returns `true` if the runtime has indicated that this worker is nearing
/// its CPU execution limit, and will be forcefully terminated soon
pub fn is_near_cpu_limit() -> bool {
    __instance_signal.load(Ordering::Relaxed) == ALMOST_OUT_OF_TIME_VALUE
}
