use crate::Outcome::{Canceled, ExceededCpu, ExceededMemory, ScriptNotFound, Success, Unknown};
use worker_sys::{TailItem as EdgeTailItem, TailLog as EdgeTailLog, TailException};

pub struct TailItem {
    inner: EdgeTailItem,
}

pub struct TailLog {
    inner: EdgeTailLog
}

impl TailItem {
    /// Type-specific information associated with this event.
    // TODO: add each event type and use serde-wasm-bindgen
    pub fn event(&self) -> ! {
        todo!("")
    }

    /// The timestamp of the event as a Unix timestamp.
    pub fn event_timestamp(&self) -> Option<i64> {
        self.inner.event_timestamp()
    }

    /// Any calls to console.* functions from this event.
    // TODO: logs[*].message is 'any' in JS
    pub fn logs(&self) -> Vec<TailLog> {
        todo!("")
    }

    /// Any unhandled exceptions from this event.
    pub fn exceptions(&self) -> Vec<TailException> {
        self.inner.exceptions()
    }

    /// The name of the Worker script.
    pub fn script_name(&self) -> Option<String> {
        self.inner.script_name()
    }

    /// The Dispatch Namespace associated to the Worker script, if applicable.
    pub fn dispatch_namespace(&self) -> Option<String> {
        self.inner.dispatch_namespace()
    }

    /// The tags associated to the Worker script, if applicable.
    pub fn script_tags(&self) -> Option<Vec<String>> {
        self.inner
            .script_tags()
            .map(|tags| tags.into_iter().map(Into::into).collect())
    }

    /// The outcome of the event.
    pub fn outcome(&self) -> Outcome {
        // TODO: use strum?
        match self.inner.outcome().as_str() {
            "ok" => Success,
            "exceededCpu" => ExceededCpu,
            "exceededMemory" => ExceededMemory,
            "scriptNotFound" => ScriptNotFound,
            "canceled" => Canceled,
            "unknown" => Unknown,
            _ => Unknown,
        }
    }
}

// TODO: document the reason behind these outcomes
pub enum Outcome {
    Unknown,
    Success,
    ExceededCpu,
    ExceededMemory,
    ScriptNotFound,
    Canceled,
}

// TODO: document
pub type TailEvent = Vec<TailItem>;

// TODO: add TailContext (only waitUntil)