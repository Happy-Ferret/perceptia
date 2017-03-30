// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of
// the MPL was not distributed with this file, You can obtain one at http://mozilla.org/MPL/2.0/

//! Implementation of `dharma::Module` for Device Manager.

// -------------------------------------------------------------------------------------------------

use dharma::{InitResult, Module};
use qualia::{Context, Perceptron, perceptron};
use device_manager::DeviceManager;

// -------------------------------------------------------------------------------------------------

pub struct DeviceManagerModule {
    manager: Option<DeviceManager>,
}

// -------------------------------------------------------------------------------------------------

impl DeviceManagerModule {
    /// `DeviceManagerModule` constructor.
    pub fn new() -> Box<Module<T = Perceptron, C = Context>> {
        Box::new(DeviceManagerModule { manager: None })
    }
}

// -------------------------------------------------------------------------------------------------

impl Module for DeviceManagerModule {
    type T = Perceptron;
    type C = Context;

    fn initialize(&mut self, context: &mut Self::C) -> InitResult {
        self.manager = Some(DeviceManager::new(context.clone()));
        vec![perceptron::INPUTS_CHANGED, perceptron::OUTPUTS_CHANGED]
    }

    // FIXME: Finnish handling signals in `DeviceManagerModule`.
    fn execute(&mut self, package: &Self::T) {
        if let Some(ref mut manager) = self.manager {
            match *package {
                Perceptron::InputsChanged => manager.on_inputs_changed(),
                Perceptron::OutputsChanged => manager.on_outputs_changed(),
                _ => {}
            }
        }
    }

    fn finalize(&mut self) {
        log_info1!("Finalized Device Manager module");
    }
}

// -------------------------------------------------------------------------------------------------
