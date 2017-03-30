// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of
// the MPL was not distributed with this file, You can obtain one at http://mozilla.org/MPL/2.0/

//! XXX

// -------------------------------------------------------------------------------------------------

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use dharma::{EventHandlerId, event_kind};
use qualia::{Context, Illusion};

use evdev;
use udev::{DeviceInfo, Udev};
use input_gateway::InputGateway;
use drivers::InputDriver;
use device_access::RestrictedOpener;

// -------------------------------------------------------------------------------------------------

/// Output Collector manages input devices. When input device if found or lost Collector notifies
/// the rest of application about this event.
pub struct InputCollector {
    context: Context,
    udev: Udev,
    restricted_opener: Rc<RefCell<RestrictedOpener>>,
    current_devices: HashMap<DeviceInfo, EventHandlerId>,
}

// -------------------------------------------------------------------------------------------------

impl InputCollector {
    /// Contructs new `InputCollector`.
    pub fn new(context: Context,
               udev: Udev,
               restricted_opener: Rc<RefCell<RestrictedOpener>>) -> Self {
        InputCollector {
            context: context,
            udev: udev,
            restricted_opener: restricted_opener,
            current_devices: HashMap::new(),
        }
    }

    /// XXX
    pub fn scan_devices(&mut self) -> Result<(), Illusion> {
        let old_devices = self.collect_current_devices();
        let mut new_devices = HashSet::<DeviceInfo>::new();

        self.udev.iterate_input_devices(|devnode, devkind, _| {
            new_devices.insert(DeviceInfo::new(devnode, devkind));
        });

        for dev in new_devices.difference(&old_devices) {
            self.handle_new_device(dev.clone());
        }

        for dev in old_devices.difference(&new_devices) {
            self.handle_lost_device(dev);
        }

        Ok(())
    }
 }

// -------------------------------------------------------------------------------------------------

/// XXX
impl InputCollector {
    /// XXX
    fn handle_new_device(&mut self, device: DeviceInfo) {
        let config = self.context.get_config().get_input_config();
        let gateway = InputGateway::new(config,
                                        self.context.get_input_manager().clone(),
                                        self.context.get_signaler().clone());

        let r = evdev::Evdev::initialize_device(&device.devnode,
                                                device.device_kind,
                                                config,
                                                gateway,
                                                |path, oflag, mode| {
                                                    self.restricted_opener
                                                        .borrow()
                                                        .open_restricted(path, oflag, mode)
                                                });
        log_info1!("Found {:?}: {:?}", device.device_kind, device.devnode);
        match r {
            Ok(driver) => {
                let id = self.context.add_event_handler(driver, event_kind::READ);
                self.current_devices.insert(device, id);
            }
            Err(err) => {
                log_error!("Could not initialize input device: {}", err);
            }
        }
    }

    /// XXX
    fn handle_lost_device(&mut self, device: &DeviceInfo) {
        log_info1!("Lost {:?}: {:?}", device.device_kind, device.devnode);
        if let Some(id) = self.current_devices.remove(&device) {
            self.context.remove_event_handler(id);
        } else {
            log_warn2!("Lost input device which was never found: {:?}", device);
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// XXX
impl InputCollector {
    /// XXX
    ///
    /// Could not use `self.current_devices.keys().collect::<HashSet<DeviceInfo>>()` because key
    /// iterator is over `&DeviceInfo`.
    fn collect_current_devices(&self) -> HashSet<DeviceInfo> {
        let mut set = HashSet::new();
        for key in self.current_devices.keys() {
            set.insert(key.clone());
        }
        set
    }
}

// -------------------------------------------------------------------------------------------------
