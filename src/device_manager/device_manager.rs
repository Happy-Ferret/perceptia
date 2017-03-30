// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of
// the MPL was not distributed with this file, You can obtain one at http://mozilla.org/MPL/2.0/

//! Device manager.

// -------------------------------------------------------------------------------------------------

use std::cell::RefCell;
use std::rc::Rc;

use dharma;
use qualia::Context;

use udev;
use device_access::RestrictedOpener;
use input_collector::InputCollector;
use output_collector::OutputCollector;

// -------------------------------------------------------------------------------------------------

/// Device Manager manages searching input and output devices and monitoring them.
pub struct DeviceManager {
    udev: udev::Udev,
    input_collector: InputCollector,
    output_collector: OutputCollector,
}

// -------------------------------------------------------------------------------------------------

/// XXX Initializing
impl DeviceManager {
    /// Constructs new `DeviceManager`.
    pub fn new(mut context: Context) -> Self {
        let udev = udev::Udev::new();
        let restricted_opener = Self::prepare_restricted_opener();

        let mut mine = DeviceManager {
            udev: udev.clone(),
            input_collector: InputCollector::new(context.clone(),
                                                 udev.clone(),
                                                 restricted_opener.clone()),
            output_collector: OutputCollector::new(context.get_dispatcher().clone(),
                                                   context.get_signaler().clone()),
        };

        // Initialize output devices
        mine.scan_output_devices();

        // Initialize input devices
        mine.scan_input_devices();

        // Initialize device monitor
        mine.initialize_device_monitor(&mut context);

        mine
    }

    /// XXX
    fn prepare_restricted_opener() -> Rc<RefCell<RestrictedOpener>> {
        let restricted_opener = Rc::new(RefCell::new(RestrictedOpener::new()));
        if let Err(err) = restricted_opener.borrow_mut().initialize_ipc() {
            log_warn1!("Failed to initialize IPC ({:?}). \
                        This may cause problems with access to devices.",
                        err);
        }
        restricted_opener
    }

    /// Initialize device monitoring.
    fn initialize_device_monitor(&mut self, context: &mut Context) {
        match self.udev.start_device_monitor(context.get_signaler().clone()) {
            Ok(device_monitor) => {
                log_debug!("ADD MONITOR");
                context.add_event_handler(Box::new(device_monitor), dharma::event_kind::READ);
            }
            Err(err) => {
                log_warn1!("Device Manager: {}", err);
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// XXX scanning
impl DeviceManager {
    /// Iterates over input devices to find usable ones and initialize event handlers for them.
    fn scan_input_devices(&mut self) {
        if let Err(err) = self.input_collector.scan_devices() {
            log_warn1!("Failed to scan input devices: {:?}", err);
        }
    }

    /// Find and initialize outputs.
    fn scan_output_devices(&mut self) {
        let oc = &mut self.output_collector;
        self.udev.iterate_drm_devices(|devnode, _| {
            // FIXME: Can not do:
            // self.output_collector.scan_device(devnode);
            // Is it compiler bug? XXX
            if let Err(err) = oc.scan_device(devnode) {
                log_error!("{}", err);
            }
        });
    }
}

// -------------------------------------------------------------------------------------------------

/// XXX handling
impl DeviceManager {
    /// XXX
    pub fn on_inputs_changed(&mut self) {
        self.scan_input_devices();
    }

    /// XXX
    pub fn on_outputs_changed(&mut self) {
        // TODO: Implement handling of found and lost outputs.
    }
}

// -------------------------------------------------------------------------------------------------
