// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of
// the MPL was not distributed with this file, You can obtain one at http://mozilla.org/MPL/2.0/

//! Wrapper for `libudev`. Allows to find interesting devices.

// -------------------------------------------------------------------------------------------------

use libudev;
use nix;
use std::path::{Path, PathBuf};

use dharma::Signaler;
use qualia::{DeviceKind, Illusion, Perceptron};

use device_monitor::DeviceMonitor;

// -------------------------------------------------------------------------------------------------

const INPUT_MOUSE: &'static str = "ID_INPUT_MOUSE";
const INPUT_TOUCHPAD: &'static str = "ID_INPUT_TOUCHPAD";
const INPUT_KEYBOARD: &'static str = "ID_INPUT_KEYBOARD";

// -------------------------------------------------------------------------------------------------

/// XXX
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DeviceInfo {
    pub devnode: PathBuf,
    pub device_kind: DeviceKind,
}

// -------------------------------------------------------------------------------------------------

impl DeviceInfo {
    /// Constructs new `DeviceInfo`.
    pub fn new(devnode: &Path, device_kind: DeviceKind) -> Self {
        DeviceInfo {
            devnode: devnode.to_owned(),
            device_kind: device_kind,
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Wrapper for `libudev`'s context.
#[derive(Clone)]
pub struct Udev {
    context: libudev::Context,
}

// -------------------------------------------------------------------------------------------------

impl Udev {
    /// `Udev` constructor.
    pub fn new() -> Self {
        Udev {
            context: libudev::Context::new().expect("Failed to create udev context"),
        }
    }

    /// Iterate over connected input event devices and pass results to given handler.
    /// Panic if something goes wrong - this is crucial for perceptia to have input.
    pub fn iterate_input_devices<F>(&self, mut f: F)
        where F: FnMut(&Path, DeviceKind, &libudev::Device)
    {
        let mut enumerator =
            libudev::Enumerator::new(self.context.clone())
                .expect("Failed to create device enumerator");

        enumerator.match_subsystem("input").expect("Failed to apply filter for device enumerator");
        for device in enumerator.scan_devices().expect("Failed to scan devices") {
            let device_kind = determine_device_kind(&device);
            if device_kind != DeviceKind::Unknown && is_input_device(&device) {
                if let Some(devnode) = device.devnode() {
                    if exists_in_filesystem(&devnode) {
                        f(devnode, device_kind, &device);
                    }
                }
            }
        }
    }

    /// Iterate over connected output DRM devices and pass results to given handler.
    /// Panic if something goes wrong - this is crucial for perceptia to have output.
    pub fn iterate_drm_devices<F: FnMut(&Path, &libudev::Device)>(&self, mut f: F) {
        let mut enumerator =
            libudev::Enumerator::new(self.context.clone())
                .expect("Failed to create device enumerator");

        enumerator.match_subsystem("drm").expect("Failed to apply filter for device enumerator");
        for device in enumerator.scan_devices().expect("Failed to scan devices") {
            if is_output_device(&device) {
                if let Some(devnode) = device.devnode() {
                    if exists_in_filesystem(&devnode) {
                        log_info1!("Found output device: {:?}", devnode);
                        f(devnode, &device);
                    }
                }
            }
        }
    }

    /// Start device monitoring and return instance of `Dispatcher` `EventHandler` for processing
    /// device events.
    ///
    /// Returned `DeviceMonitor` contains file descriptor from `udev` monitor. `DeviceMonitor` will
    /// handle situations when the file descriptor becomes invalid.
    pub fn start_device_monitor(&mut self,
                                signaler: Signaler<Perceptron>)
                                -> Result<DeviceMonitor, Illusion> {
        let mut monitor = libudev::Monitor::new(self.context.clone())?;
        ensure!(monitor.match_subsystem("input"));
        ensure!(monitor.match_subsystem("drm"));
        let monitor_socket = monitor.listen()?;
        Ok(DeviceMonitor::new(monitor_socket, signaler))
    }
}

// -------------------------------------------------------------------------------------------------

/// Checks if given device exists is event device.
pub fn exists_in_filesystem(devnode: &Path) -> bool {
    nix::sys::stat::stat(devnode).is_ok()
}

// -------------------------------------------------------------------------------------------------

/// Checks if given device is input device.
pub fn is_input_device(device: &libudev::Device) -> bool {
    match device.sysname().to_os_string().into_string() {
        Ok(sysname) => sysname.starts_with("event"),
        Err(_) => false,
    }
}

// -------------------------------------------------------------------------------------------------

/// Checks if given device is output device.
pub fn is_output_device(device: &libudev::Device) -> bool {
    match device.sysname().to_os_string().into_string() {
        Ok(sysname) => sysname.starts_with("card"),
        Err(_) => false,
    }
}

// -------------------------------------------------------------------------------------------------

/// Reads devices properties and determines device kind basing on them.
pub fn determine_device_kind(device: &libudev::Device) -> DeviceKind {
    for property in device.properties() {
        if property.name() == INPUT_MOUSE {
            return DeviceKind::Mouse;
        } else if property.name() == INPUT_TOUCHPAD {
            return DeviceKind::Touchpad;
        } else if property.name() == INPUT_KEYBOARD {
            return DeviceKind::Keyboard;
        }
    }
    DeviceKind::Unknown
}

// -------------------------------------------------------------------------------------------------
