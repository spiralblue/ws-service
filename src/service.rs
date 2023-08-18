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
// The service.rs file is the core of each service
// It enables the communication via UDP or GraphQL (depending on --features flag during compilation)

use cubeos_service::*;
use std::time::{Duration, Instant};

// Macro to create UDP-handler function or GraphQL Queries and Mutations
// The layout follows the rules:
// query/mutation: Command-Name => Function as defined in subsystem.rs; out: GraphQLOutput;
//
// Out is only needed for queries if the Output should be formatted in humanly readable way
// (e.g. Payload telemetry returns a Vec<u8>, but resembles analog data like Voltage,Current,Temperature etc.)
// If Out is not needed then please set to Output of function
service_macro! {
    use Error;
    subsystem::Subsystem{ 
        query: Initialised => fn initialised(&self) -> Result<()>;
        mutation: Time => fn time(&self) -> Result<()>;
        mutation: StartupCommand => fn startup_command(&self, cmd: Vec<u8>) -> Result<()>;
        mutation: Shutdown => fn shutdown(&self, time_remaining_s: u16) -> Result<()>;
    }
}