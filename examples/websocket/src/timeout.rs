use crate::{helpers::set_timeout, user::Users};

macro_rules! timeout {
    ($self:ident, $timeout:ident, $ops:tt) => {
        let millis = $self.timeout.$timeout;
        set_timeout(
            worker::wasm_bindgen::closure::Closure::once_into_js(|| {
                $self.$ops();
            }),
            millis,
        );
    };
}

#[derive(Debug)]
struct Timeout {
    ping_timeout: i32,
    pong_timeout: i32,
    max_heart_beat: usize,
}

#[derive(Debug)]
pub struct TimeoutHandle<T> {
    timeout: Timeout,
    item: T,
}

impl<T> TimeoutHandle<T> {
    pub const fn new(item: T, ping_timeout: i32, pong_timeout: i32) -> Self {
        Self {
            item,
            timeout: Timeout {
                max_heart_beat: 5,
                ping_timeout,
                pong_timeout,
            },
        }
    }

    pub const fn max_heart_beat(mut self, max_heart_beat: usize) -> Self {
        self.timeout.max_heart_beat = max_heart_beat;
        self
    }
}

pub trait HeartBeat {
    fn ping(self);
    fn pong(self);
    fn run(self);
    fn set_ping_timeout(self);
    fn set_pong_timeout(self);
}

impl HeartBeat for TimeoutHandle<(Users, Box<str>)> {
    fn ping(self) {
        let (users, id) = &self.item;
        // send "ping" to client
        // if client is still connected then give the client a timeout for "pong" response
        if users.ping(id).is_some() {
            self.set_pong_timeout();
        }
    }

    fn pong(self) {
        let (users, id) = &self.item;
        // this will be called 5 secs after sending "ping" to the client
        // check if client has responded
        if let Some(pong) = users.hb(id) {
            if pong >= self.timeout.max_heart_beat {
                users.close(id);
                return;
            }
        } else {
            return;
        }
        self.set_ping_timeout();
    }

    fn run(self) {
        // this trait will achieve something like below in javascript
        //
        // function run() {
        //   // start firing after 5 secs
        //   setTimeout(ping, 5000);
        //   function ping() {
        //     // if the client is still connected then cooldown 5 secs before checking for "pong" response
        //     if (this.users.ping(this.id)) {
        //       setTimeout(pong, 5000);
        //     }
        //     function pong() {
        //       // see if the client has responded with a "pong" yet
        //       const hb = this.users.hb(this.id);
        //       // apparently 0 is false in javascript
        //       if (hb === undefined) {
        //         return;
        //       } else if (hb === this.timeout.max_heart_beat) {
        //         this.users.close(this.id);
        //         return;
        //       }
        //       // runs again
        //       setTimeout(ping, 5000);
        //     }
        //   }
        // }
        self.set_ping_timeout();
    }

    fn set_ping_timeout(self) {
        timeout!(self, ping_timeout, ping);
    }

    fn set_pong_timeout(self) {
        timeout!(self, pong_timeout, pong);
    }
}
