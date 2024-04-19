//
// Copyright (C) 2022 CUAVA
//
// Licensed under the Apache License, Version 2.0 (the "License")
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// Contributed by: Patrick Oppel (patrick.oppel94@gmail.com)
//
// In this file the subsystem that contains all the functions to interact with the API is defined
//
// Comments generated in parts with GPT-3 (see disclaimer in README)

use cubeos_service::*;
use uart_rs::{UartError,UartResult};
use ws_api::*;
use ws_api::Command as SBCommand;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Instant,Duration};
use log::{error, info};
use chrono::prelude::*;
use std::io::Read;

const POWER_UP_ALLOWANCE: Duration = Duration::from_secs(60 * 5);
const ACKNOWLEDGE_MESSAGE_TIMEOUT: Duration = Duration::from_secs(60 * 1);
const ACKNOWLEDGE_MESSAGE_ATTEMPT_TIMEOUT: Duration = Duration::from_secs(1);
const SHUTDOWN_ALLOWANCE: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct Subsystem {
    spiralblue: Arc<Mutex<UartConnection>>,
}
impl Subsystem {
    /// Initialisation of the Subsystem
    ///
    /// # Arguments
    /// * `uart_path` - A string that represents the path to the uart device.
    /// * `uart_setting` - A serial::PortSettings that represents the settings of the uart device.
    /// * `uart_timeout` - A Duration that represents the timeout of the uart device.
    ///
    /// # Output
    ///
    /// * `ExampleResult<Self>` - Returns `Self` or ExampleError.
    pub fn new(
        uart_path: String,
        uart_setting: serial::PortSettings,
        uart_timeout: Duration,
    ) -> UartResult<Self> {
        Ok(Self {
            spiralblue: Arc::new(Mutex::new(UartConnection::new(
                uart_path,
                uart_setting,
                uart_timeout,
            )?)),
        })
    }

    // pub fn initialised(&self) -> UartResult<()> {
    //     match self.wait_for_message(CommandType::Initialised, POWER_UP_ALLOWANCE) {
    //         Ok(()) => {
    //             info!("Initialised");
    //             Ok(self.spiralblue.lock().unwrap().send_message(
    //                 SBCommand::simple_command(CommandType::InitialisedAcknowledge)
    //             )?)
    //         }
    //         Err(e) => Err(e),
    //     }
    // }

    pub fn initialised(&self) -> UartResult<()> {
        match self.spiralblue.lock().unwrap().receive_init(POWER_UP_ALLOWANCE) {
            Ok(mut data) => {
                let last_three_bytes = data.split_off(data.len() - 3);
                println!("Received: {:?}", data);
                println!("Last three bytes: {:?}", last_three_bytes);
                if let Some(received_command) = SBCommand::from_bytes(last_three_bytes) {
                    if received_command.command_type == CommandType::Initialised {
                        info!("Initialised");
                        Ok(())
                        // Ok(self.spiralblue.lock().unwrap().send_message(
                        //     SBCommand::simple_command(CommandType::InitialisedAcknowledge)
                        // )?)
                    } else {
                        Err(UartError::from(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "Received {:?} instead of {:?}",
                                received_command, CommandType::Initialised
                            ),
                        )))
                    }
                } else {
                    Err(UartError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Received {:?} instead of {:?}",
                            data, CommandType::Initialised
                        ),
                    )))
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn time(&self) -> UartResult<()> {
        let sb_command = SBCommand::time(Utc::now());
        info!("Send Time: {:?}", sb_command.data);
        Ok(self.send_message_with_acknowledgment(
            sb_command,
            CommandType::TimeAcknowledge,
            ACKNOWLEDGE_MESSAGE_TIMEOUT,
        )?)
    }

    pub fn startup_command(&self, cmd: Vec<u8>) -> UartResult<()> {
        let sb_command = SBCommand::startup_command(cmd);
        info!("Send Startup SBCommand: {:?}", sb_command.data);
        Ok(self.send_message_with_acknowledgment(
            sb_command,
            CommandType::StartupCommandAcknowledge,
            ACKNOWLEDGE_MESSAGE_TIMEOUT,
        )?)
    }

    pub fn shutdown(&self, time_remaining_s: u16) -> UartResult<()> {
        let time_remaining = Duration::from_secs(time_remaining_s as u64);
        match self.wait_for_message(CommandType::PowerDown, time_remaining) {
            Ok(()) => {
                info!("Shutdown");
                Ok(self.spiralblue.lock().unwrap().send_message(
                    SBCommand::simple_command(CommandType::PowerDownAcknowledge)
                )?)
            }
            Err(e) => Ok(self.send_message_with_acknowledgment(
                SBCommand::simple_command(CommandType::PowerDown),
                CommandType::PowerDownAcknowledge,
                SHUTDOWN_ALLOWANCE,
            )?)
        }
    }

    pub fn ftp(&self) -> UartResult<()> {
        Ok(self.spiralblue.lock().unwrap().ftp()?)
    }

    fn wait_for_message(
        &self,
        message_type: CommandType,
        timeout: Duration,
    ) -> UartResult<()> {
        match self.spiralblue.lock().unwrap().receive_message(timeout) {
            Ok(Some(message)) => {
                if message.command_type == message_type {
                    Ok(())
                } else {
                    Err(UartError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Received {:?} instead of {:?}",
                            message.command_type, message_type
                        ),
                    )))
                }
            }
            _ => Err(UartError::from(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                format!(
                    "Did not receive message of type {:?} in time",
                    message_type
                ),
            ))),
        }        
    }

    fn send_message_with_acknowledgment(
        &self,
        command: SBCommand,
        expected_acknowledgment_type: CommandType,
        timeout: Duration,
    ) -> UartResult<()> {
        match self.spiralblue.lock().unwrap().send_message(command) {
            Ok(()) => {
                match self.wait_for_message(
                    expected_acknowledgment_type,
                    timeout,
                ) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        error!("{:?}",e);
                        Err(e.into())
                    }
                }
            }
            Err(e) => {
                error!("{:?}",e);
                Err(e.into())
            }
        }
    }
}