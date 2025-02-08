use crate::communication::{AbstractCommunicationConnector, CommunicationConnector, ISignalTriggering, PduTriggering};
use crate::{AbstractionElement, AutosarAbstractionError, EcuInstance};
use autosar_data::{Element, ElementName};

mod can;
mod ethernet;
mod flexray;

pub use can::*;
pub use ethernet::*;
pub use flexray::*;

//##################################################################

/// trait for physical channels
pub trait AbstractPhysicalChannel: AbstractionElement {
    /// the type of communication connector used by this physical channel
    type CommunicationConnectorType: AbstractCommunicationConnector;

    /// iterate over all PduTriggerings of this physical channel
    fn pdu_triggerings(&self) -> impl Iterator<Item = PduTriggering> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::PduTriggerings)
            .into_iter()
            .flat_map(|triggerings| triggerings.sub_elements())
            .filter_map(|triggering| PduTriggering::try_from(triggering).ok())
    }

    /// iterate over all ISignalTriggerings of this physical channel
    fn signal_triggerings(&self) -> impl Iterator<Item = ISignalTriggering> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::ISignalTriggerings)
            .into_iter()
            .flat_map(|triggerings| triggerings.sub_elements())
            .filter_map(|triggering| ISignalTriggering::try_from(triggering).ok())
    }

    /// iterate over all connectors between this physical channel and any ECU
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::{*, communication::*};
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
    /// # let package = model.get_or_create_package("/pkg1")?;
    /// # let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// # let cluster = system.create_can_cluster("Cluster", &package, &CanClusterSettings::default())?;
    /// # let can_channel = cluster.create_physical_channel("Channel")?;
    /// # let ecu = system.create_ecu_instance("ECU", &package)?;
    /// # let can_controller = ecu.create_can_communication_controller("Controller")?;
    /// can_controller.connect_physical_channel("Connector", &can_channel)?;
    /// for connector in can_channel.connectors() {
    ///    println!("Connector: {:?}", connector);
    /// }
    /// # assert_eq!(can_channel.connectors().count(), 1);
    /// # Ok(())}
    fn connectors(&self) -> impl Iterator<Item = Self::CommunicationConnectorType> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::CommConnectors)
            .into_iter()
            .flat_map(|connectors| connectors.sub_elements())
            .filter_map(|ccrc| {
                ccrc.get_sub_element(ElementName::CommunicationConnectorRef)
                    .and_then(|connector| connector.get_reference_target().ok())
                    .and_then(|connector| Self::CommunicationConnectorType::try_from(connector).ok())
            })
    }

    /// get the connector element between this channel and an ecu
    #[must_use]
    fn ecu_connector(&self, ecu_instance: &EcuInstance) -> Option<Self::CommunicationConnectorType> {
        // get a connector referenced by this physical channel which is contained in the ecu_instance
        // iterate over the CommunicationConnectorRefConditionals
        for connector in self.connectors() {
            if let Ok(connector_ecu_instance) = connector.ecu_instance() {
                if connector_ecu_instance == *ecu_instance {
                    return Some(connector);
                }
            }
        }

        None
    }
}

//##################################################################

/// A physical channel is a communication channel between two ECUs.
///
/// This enum wraps the different physical channel types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalChannel {
    /// A CAN physical channel
    Can(CanPhysicalChannel),
    /// An Ethernet physical channel
    Ethernet(EthernetPhysicalChannel),
    /// A `FlexRay` physical channel
    FlexRay(FlexrayPhysicalChannel),
}

impl AbstractPhysicalChannel for PhysicalChannel {
    type CommunicationConnectorType = CommunicationConnector;
}

impl AbstractionElement for PhysicalChannel {
    fn element(&self) -> &autosar_data::Element {
        match self {
            PhysicalChannel::Can(cpc) => cpc.element(),
            PhysicalChannel::Ethernet(epc) => epc.element(),
            PhysicalChannel::FlexRay(fpc) => fpc.element(),
        }
    }
}

impl From<CanPhysicalChannel> for PhysicalChannel {
    fn from(value: CanPhysicalChannel) -> Self {
        PhysicalChannel::Can(value)
    }
}

impl From<EthernetPhysicalChannel> for PhysicalChannel {
    fn from(value: EthernetPhysicalChannel) -> Self {
        PhysicalChannel::Ethernet(value)
    }
}

impl From<FlexrayPhysicalChannel> for PhysicalChannel {
    fn from(value: FlexrayPhysicalChannel) -> Self {
        PhysicalChannel::FlexRay(value)
    }
}

impl TryFrom<Element> for PhysicalChannel {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::CanPhysicalChannel => Ok(CanPhysicalChannel::try_from(element)?.into()),
            ElementName::EthernetPhysicalChannel => Ok(EthernetPhysicalChannel::try_from(element)?.into()),
            ElementName::FlexrayPhysicalChannel => Ok(FlexrayPhysicalChannel::try_from(element)?.into()),
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "PhysicalChannel".to_string(),
            }),
        }
    }
}

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        communication::{AbstractFrame, CanAddressingMode, CanClusterSettings, CanFrameType, TransferProperty},
        AutosarModelAbstraction, ByteOrder, SystemCategory,
    };
    use autosar_data::AutosarVersion;

    #[test]
    fn abstract_physical_channel() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();
        let settings = CanClusterSettings::default();
        let cluster = system.create_can_cluster("CanCluster", &pkg, &settings).unwrap();
        let channel = cluster.create_physical_channel("channel_name").unwrap();

        let ecu = system.create_ecu_instance("ECU", &pkg).unwrap();
        let can_controller = ecu.create_can_communication_controller("Controller").unwrap();
        let connector = can_controller.connect_physical_channel("Connector", &channel).unwrap();

        let frame = system.create_can_frame("Frame", &pkg, 8).unwrap();
        let isignal_ipdu = system.create_isignal_ipdu("ISignalIPdu", &pkg, 8).unwrap();

        let system_signal = pkg.create_system_signal("SystemSignal").unwrap();
        let signal = system.create_isignal("Signal", &pkg, 8, &system_signal, None).unwrap();
        isignal_ipdu
            .map_signal(
                &signal,
                0,
                ByteOrder::MostSignificantByteLast,
                None,
                TransferProperty::Triggered,
            )
            .unwrap();
        frame
            .map_pdu(&isignal_ipdu, 0, ByteOrder::MostSignificantByteLast, None)
            .unwrap();

        let frame_triggering = channel
            .trigger_frame(&frame, 0x100, CanAddressingMode::Standard, CanFrameType::Can20)
            .unwrap();

        assert_eq!(channel.frame_triggerings().count(), 1);
        assert_eq!(channel.frame_triggerings().next(), Some(frame_triggering));
        assert_eq!(channel.pdu_triggerings().count(), 1);
        assert_eq!(
            channel.pdu_triggerings().next().unwrap().pdu().unwrap(),
            isignal_ipdu.into()
        );
        assert_eq!(channel.signal_triggerings().count(), 1);

        assert_eq!(channel.connectors().count(), 1);
        assert_eq!(channel.ecu_connector(&ecu).unwrap(), connector);
    }
}
