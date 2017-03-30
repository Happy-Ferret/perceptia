// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of
// the MPL was not distributed with this file, You can obtain one at http://mozilla.org/MPL/2.0/

//! XXX 

// -------------------------------------------------------------------------------------------------

use std::path::Path;
use std::os::unix::io;

use nix::{self, Errno};
use nix::fcntl::{open, OFlag};
use nix::sys::stat::{Mode, stat};

use qualia::{Illusion, Ipc};

// -------------------------------------------------------------------------------------------------

/// XXX
pub struct RestrictedOpener {
    ipc: Ipc,
}

// -------------------------------------------------------------------------------------------------

impl RestrictedOpener {
    /// Constructs new `RestrictedOpener`.
    pub fn new() -> Self {
        RestrictedOpener {
            ipc: Ipc::new(),
        }
    }

    /// Tries to open device. If we have insufficient permissions ask `logind` to do it for us.
    pub fn open_restricted(&self,
                           path: &Path,
                           oflag: OFlag,
                           mode: Mode)
                           -> Result<io::RawFd, Illusion> {
        match open(path, oflag, mode) {
            Ok(fd) => Ok(fd),
            Err(nix::Error::Sys(errno)) => {
                if (errno == Errno::EPERM) || (errno == Errno::EACCES) {
                    match stat(path) {
                        Ok(st) => self.ipc.take_device(st.st_rdev as u64),
                        _ => Err(Illusion::General(format!("Could not stat file '{:?}'", path))),
                    }
                } else {
                    Err(Illusion::InvalidArgument(errno.desc().to_owned()))
                }
            }
            Err(nix::Error::InvalidPath) => {
                Err(Illusion::InvalidArgument(format!("Path '{:?}' does not exist!", path)))
            }
        }
    }

    /// Initialize connection to `logind`.
    pub fn initialize_ipc(&mut self) -> Result<(), Illusion> {
        self.ipc.initialize()
    }
}

// -------------------------------------------------------------------------------------------------
