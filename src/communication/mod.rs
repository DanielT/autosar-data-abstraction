//! # Communication between ECUs in a system
//!
//! This module contains the communication related elements of the AUTOSAR metamodel.
//!
//! Currently, the following bus types are supported:
//! - CAN
//! - Ethernet
//! - Flexray
//!
//! For each of the bus types, the following elements are available:
//! - Communication using Frames, PDUs and signals
//! - Network management
//! - Diagnostic transport protocol
//!
//! Ethernet also has support for:
//! - Old communication odel (using `SocketConnectionBundles`)
//! - New communication model (using `StaticSocketConnections`)
//! - `SomeIP`
//!
//! # Example
//!
//! ```
//! use autosar_data::*;
//! use autosar_data_abstraction::*;
//! use autosar_data_abstraction::communication::*;
//! use autosar_data_abstraction::datatype::*;
//!
//! let model = AutosarModel::new();
//! model.create_file("can.arxml", AutosarVersion::LATEST).unwrap();
//! let system_package = ArPackage::get_or_create(&model, "/System").unwrap();
//! let system = System::new("System", &system_package, SystemCategory::SystemExtract).unwrap();
//! let cluster_package = ArPackage::get_or_create(&model, "/Network/Clusters").unwrap();
//!  
//! let settings = CanClusterSettings {
//!     can_fd_baudrate: Some(2000000),
//!     ..Default::default()
//! };
//! let can_cluster = system
//!     .create_can_cluster("CanCluster", &cluster_package, &settings)
//!     .unwrap();
//! assert_eq!(can_cluster.element().element_name(), ElementName::CanCluster);
//! let can_channel = can_cluster.create_physical_channel("CanChannel").unwrap();
//!  
//! let ecu_package = ArPackage::get_or_create(&model, "/Ecus").unwrap();
//!  
//! // create ECU A and connect it to the CAN channel
//! let ecu_instance_a = system.create_ecu_instance("Ecu_A", &ecu_package).unwrap();
//! let canctrl_a = ecu_instance_a
//!     .create_can_communication_controller("CanController")
//!     .unwrap();
//! let channels_iter = canctrl_a.connected_channels();
//! assert_eq!(channels_iter.count(), 0);
//! canctrl_a
//!     .connect_physical_channel("Ecu_A_connector", &can_channel)
//!     .unwrap();
//! let channels_iter = canctrl_a.connected_channels();
//! assert_eq!(channels_iter.count(), 1);
//!  
//! // create ECU B and connect it to the CAN channel
//! let ecu_instance_b = system.create_ecu_instance("Ecu_B", &ecu_package).unwrap();
//! let canctrl_b = ecu_instance_b
//!     .create_can_communication_controller("CanController")
//!     .unwrap();
//! canctrl_b
//!     .connect_physical_channel("Ecu_B_connector", &can_channel)
//!     .unwrap();
//!  
//! let frame_package = ArPackage::get_or_create(&model, "/Network/Frames").unwrap();
//! let pdu_package = ArPackage::get_or_create(&model, "/Network/Pdus").unwrap();
//! let isignal_package = ArPackage::get_or_create(&model, "/Network/Signals").unwrap();
//! let syssignal_package = ArPackage::get_or_create(&model, "/System/Signals").unwrap();
//!  
//! // create a base type for the CAN signals
//! let base_type_package = ArPackage::get_or_create(&model, "/BaseTypes").unwrap();
//! let base_type_u8 = SwBaseType::new(
//!     "uint8",
//!     &base_type_package,
//!     8,
//!     BaseTypeEncoding::None,
//!     None,
//!     None,
//!     Some("uint8"),
//! )
//! .unwrap();
//!  
//! // create a frame which contains one Pdu: Id 0x101, length 8
//! let frame = system.create_can_frame("frame", 8, &frame_package).unwrap();
//! let pdu = system.create_isignal_ipdu("pdu", &pdu_package, 8).unwrap();
//! let ss_pdusignal1 = SystemSignal::new("ss_pdusignal1", &isignal_package).unwrap();
//! let pdusignal1 = system
//!     .create_isignal("pdusignal1", 4, &ss_pdusignal1, Some(&base_type_u8), &syssignal_package)
//!     .unwrap();
//! let ss_pdusignal2 = SystemSignal::new("ss_pdusignal2", &syssignal_package).unwrap();
//! let pdusignal2 = system
//!     .create_isignal("pdusignal2", 4, &ss_pdusignal2, Some(&base_type_u8), &isignal_package)
//!     .unwrap();
//! // map signal 1 to the first 4 bytes of the Pdu
//! pdu.map_signal(
//!     &pdusignal1,
//!     0,
//!     ByteOrder::MostSignificantByteFirst,
//!     None,
//!     TransferProperty::Triggered,
//! )
//! .unwrap();
//! // map signal 2 to the second 4 bytes of the Pdu
//! pdu.map_signal(
//!     &pdusignal2,
//!     8, // since this signal uses ByteOrder::MostSignificantByteFirst, it starts at byte 8 and ends at byte 4
//!     ByteOrder::MostSignificantByteFirst,
//!     None,
//!     TransferProperty::Triggered,
//! )
//! .unwrap();
//! // map the pdu to the frame
//! frame.map_pdu(
//!     &pdu,
//!     0,
//!     ByteOrder::MostSignificantByteLast,
//!     None,
//! )
//! .unwrap();
//! // trigger the frame on the CAN channel (id 0x101)
//! let frame_triggering = can_channel
//!     .trigger_frame(&frame, 0x101, CanAddressingMode::Standard, CanFrameType::Can20)
//!     .unwrap();
//!  
//! // frame connection: Ecu_B -> Ecu_A
//! frame_triggering.connect_to_ecu(&ecu_instance_a, CommunicationDirection::In)
//!     .unwrap();
//! frame_triggering.connect_to_ecu(&ecu_instance_b, CommunicationDirection::Out)
//!     .unwrap();
//! ```

use crate::AutosarAbstractionError;
use autosar_data::EnumItem;

mod cluster;
mod controller;
mod datatransformation;
mod frame;
mod network_management;
mod pdu;
mod physical_channel;
mod signal;
mod transport_layer;

pub use cluster::*;
pub use controller::*;
pub use datatransformation::*;
pub use frame::*;
pub use network_management::*;
pub use pdu::*;
pub use physical_channel::*;
pub use signal::*;
pub use transport_layer::*;

//#########################################################

/// The [`CommunicationDirection`] is used by the communication ports for frames, PDUs and signals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommunicationDirection {
    /// The communication is incoming
    In,
    /// The communication is outgoing
    Out,
}

impl TryFrom<EnumItem> for CommunicationDirection {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<Self, Self::Error> {
        match value {
            EnumItem::In => Ok(CommunicationDirection::In),
            EnumItem::Out => Ok(CommunicationDirection::Out),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "CommunicationDirection".to_string(),
            }),
        }
    }
}

impl From<CommunicationDirection> for EnumItem {
    fn from(value: CommunicationDirection) -> Self {
        match value {
            CommunicationDirection::In => EnumItem::In,
            CommunicationDirection::Out => EnumItem::Out,
        }
    }
}
