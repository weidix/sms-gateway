pub mod core;
#[cfg(not(feature = "mock-data"))]
pub mod manager;
#[cfg(feature = "mock-data")]
pub mod mock_manager;
pub mod pdu;
pub mod types;

#[cfg(not(feature = "mock-data"))]
pub use manager::ModemManager;
#[cfg(feature = "mock-data")]
pub use mock_manager::ModemManager;
pub use types::{ModemInfo, OperatorInfo, SignalQuality, SmsType};
