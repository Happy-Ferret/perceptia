// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of
// the MPL was not distributed with this file, You can obtain one at http://mozilla.org/MPL/2.0/

/// `DeviceMonitor` implements `dharma::Dispatcher`'s `EventHandler`. It is used to process
/// notifications from `udev` about adding and removing devices..

// -------------------------------------------------------------------------------------------------

use std::os::unix::io;
use std::os::unix::io::AsRawFd;

use libudev;

use dharma::{EventHandler, EventKind, Signaler};
use qualia::{Perceptron, perceptron};

use udev;

// -------------------------------------------------------------------------------------------------

/// `udev` device event handler.
pub struct DeviceMonitor {
    monitor_socket: libudev::MonitorSocket,
    signaler: Signaler<Perceptron>,
}

// -------------------------------------------------------------------------------------------------

/// Make `MonitorSocket` sendable between threads.
///
/// FIXME: `libudev` context should be `Arc<Mutex>`'ed. Current sulution is good enough if new
///        device is not plugged in while previous plug in event is still handled.
unsafe impl Send for DeviceMonitor {}

// -------------------------------------------------------------------------------------------------

impl DeviceMonitor {
    /// Constructs new `DeviceMonitor`.
    pub fn new(monitor_socket: libudev::MonitorSocket, signaler: Signaler<Perceptron>) -> Self {
        DeviceMonitor {
            monitor_socket: monitor_socket,
            signaler: signaler,
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// This code executes in main dispatchers thread.
impl EventHandler for DeviceMonitor {
    fn get_fd(&self) -> io::RawFd {
        self.monitor_socket.as_raw_fd()
    }

    fn process_event(&mut self, _: EventKind) {
        match self.monitor_socket.receive_event() {
            Some(ref event) => {
                if udev::is_input_device(event) {
                    self.signaler.emit(perceptron::INPUTS_CHANGED, Perceptron::InputsChanged);
                } else if udev::is_output_device(event) {
                    self.signaler.emit(perceptron::OUTPUTS_CHANGED, Perceptron::OutputsChanged);
                }
            }
            None => {
                log_warn2!("Received empty device monitor event");
            }
        };
    }
}

// -------------------------------------------------------------------------------------------------
