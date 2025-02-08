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
//! # fn main() -> Result<(), AutosarAbstractionError> {
//! let model = AutosarModelAbstraction::create("can.arxml", AutosarVersion::LATEST);
//! let system_package = model.get_or_create_package("/System")?;
//! let system = system_package.create_system("System", SystemCategory::SystemExtract)?;
//! let cluster_package = model.get_or_create_package("/Network/Clusters")?;
//!  
//! let settings = CanClusterSettings {
//!     can_fd_baudrate: Some(2000000),
//!     ..Default::default()
//! };
//! let can_cluster = system.create_can_cluster("CanCluster", &cluster_package, &settings)?;
//! assert_eq!(can_cluster.element().element_name(), ElementName::CanCluster);
//! let can_channel = can_cluster.create_physical_channel("CanChannel")?;
//!  
//! let ecu_package = model.get_or_create_package("/Ecus")?;
//!  
//! // create ECU A and connect it to the CAN channel
//! let ecu_instance_a = system.create_ecu_instance("Ecu_A", &ecu_package)?;
//! let canctrl_a = ecu_instance_a.create_can_communication_controller("CanController")?;
//! let channels_iter = canctrl_a.connected_channels();
//! assert_eq!(channels_iter.count(), 0);
//! canctrl_a.connect_physical_channel("Ecu_A_connector", &can_channel)?;
//! let channels_iter = canctrl_a.connected_channels();
//! assert_eq!(channels_iter.count(), 1);
//!  
//! // create ECU B and connect it to the CAN channel
//! let ecu_instance_b = system.create_ecu_instance("Ecu_B", &ecu_package)?;
//! let canctrl_b = ecu_instance_b.create_can_communication_controller("CanController")?;
//! canctrl_b.connect_physical_channel("Ecu_B_connector", &can_channel)?;
//!  
//! let frame_package = model.get_or_create_package("/Network/Frames")?;
//! let pdu_package = model.get_or_create_package("/Network/Pdus")?;
//! let isignal_package = model.get_or_create_package("/Network/Signals")?;
//! let syssignal_package = model.get_or_create_package("/System/Signals")?;
//!  
//! // create a base type for the CAN signals
//! let base_type_package = model.get_or_create_package("/BaseTypes")?;
//! let base_type_u8 = base_type_package.create_sw_base_type(
//!     "uint8",
//!     8,
//!     BaseTypeEncoding::None,
//!     None,
//!     None,
//!     Some("uint8"),
//! )?;
//!  
//! // create a frame which contains one Pdu: Id 0x101, length 8
//! let frame = system.create_can_frame("frame", &frame_package, 8)?;
//! let pdu = system.create_isignal_ipdu("pdu", &pdu_package, 8)?;
//! let ss_pdusignal1 = syssignal_package.create_system_signal("ss_pdusignal1")?;
//! let pdusignal1 = system
//!     .create_isignal("pdusignal1", &isignal_package, 4, &ss_pdusignal1, Some(&base_type_u8))?;
//! let ss_pdusignal2 = syssignal_package.create_system_signal("ss_pdusignal2")?;
//! let pdusignal2 = system
//!     .create_isignal("pdusignal2", &isignal_package, 4, &ss_pdusignal2, Some(&base_type_u8))?;
//! // map signal 1 to the first 4 bytes of the Pdu
//! pdu.map_signal(
//!     &pdusignal1,
//!     0,
//!     ByteOrder::MostSignificantByteFirst,
//!     None,
//!     TransferProperty::Triggered,
//! )?;
//! // map signal 2 to the second 4 bytes of the Pdu
//! pdu.map_signal(
//!     &pdusignal2,
//!     8, // since this signal uses ByteOrder::MostSignificantByteFirst, it starts at byte 8 and ends at byte 4
//!     ByteOrder::MostSignificantByteFirst,
//!     None,
//!     TransferProperty::Triggered,
//! )?;
//! // map the pdu to the frame
//! frame.map_pdu(
//!     &pdu,
//!     0,
//!     ByteOrder::MostSignificantByteLast,
//!     None,
//! )?;
//! // trigger the frame on the CAN channel (id 0x101)
//! let frame_triggering = can_channel
//!     .trigger_frame(&frame, 0x101, CanAddressingMode::Standard, CanFrameType::Can20)?;
//!  
//! // frame connection: Ecu_B -> Ecu_A
//! frame_triggering.connect_to_ecu(&ecu_instance_a, CommunicationDirection::In)?;
//! frame_triggering.connect_to_ecu(&ecu_instance_b, CommunicationDirection::Out)?;
//! # Ok(())}
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

//#########################################################

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_communication_direction() {
        // convert from CommunicationDirection to EnumItem
        let in_dir = CommunicationDirection::In;
        let out_dir = CommunicationDirection::Out;
        let in_enum: EnumItem = in_dir.into();
        let out_enum: EnumItem = out_dir.into();
        assert_eq!(in_enum, EnumItem::In);
        assert_eq!(out_enum, EnumItem::Out);

        // convert from EnumItem to CommunicationDirection
        let in_dir_converted: CommunicationDirection = in_enum.try_into().unwrap();
        let out_dir_converted: CommunicationDirection = out_enum.try_into().unwrap();
        assert_eq!(in_dir_converted, CommunicationDirection::In);
        assert_eq!(out_dir_converted, CommunicationDirection::Out);
        // conversion of an enumItem other than In or Out should fail
        let bad = CommunicationDirection::try_from(EnumItem::Abstract);
        assert!(bad.is_err());
    }
}
