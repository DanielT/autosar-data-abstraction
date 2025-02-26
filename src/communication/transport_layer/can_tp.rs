use crate::communication::{AbstractIpdu, CanCluster, CanCommunicationConnector, IPdu, NPdu};
use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, EcuInstance, IdentifiableAbstractionElement,
    abstraction_element,
};
use autosar_data::{Element, ElementName, EnumItem};

//#########################################################

/// Container for `CanTp` configuration
///
/// There should be one `CanTpConfig` for each CAN network in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanTpConfig(Element);
abstraction_element!(CanTpConfig, CanTpConfig);
impl IdentifiableAbstractionElement for CanTpConfig {}

impl CanTpConfig {
    pub(crate) fn new(name: &str, package: &ArPackage, cluster: &CanCluster) -> Result<Self, AutosarAbstractionError> {
        let pkg_elem = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let tp_config_elem = pkg_elem.create_named_sub_element(ElementName::CanTpConfig, name)?;
        let tp_config = Self(tp_config_elem);
        tp_config.set_cluster(cluster)?;

        Ok(tp_config)
    }

    /// set the `CanCluster` associated with this configuration
    pub fn set_cluster(&self, cluster: &CanCluster) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::CommunicationClusterRef)?
            .set_reference_target(cluster.element())?;
        Ok(())
    }

    /// get the `CanCluster` associated with this configuration
    #[must_use]
    pub fn cluster(&self) -> Option<CanCluster> {
        self.element()
            .get_sub_element(ElementName::CommunicationClusterRef)
            .and_then(|elem| elem.get_reference_target().ok())
            .and_then(|target| CanCluster::try_from(target).ok())
    }

    /// create a `CanTp` ECU in the configuration
    pub fn create_can_tp_ecu(
        &self,
        ecu_instance: &EcuInstance,
        cycle_time_main_function: Option<f64>,
    ) -> Result<CanTpEcu, AutosarAbstractionError> {
        let ecu_collection = self.element().get_or_create_sub_element(ElementName::TpEcus)?;

        CanTpEcu::new(&ecu_collection, ecu_instance, cycle_time_main_function)
    }

    /// get all of the ECUs in the configuration
    pub fn can_tp_ecus(&self) -> impl Iterator<Item = CanTpEcu> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::TpEcus)
            .into_iter()
            .flat_map(|ecu_collection| ecu_collection.sub_elements())
            .filter_map(|tp_ecu_elem| CanTpEcu::try_from(tp_ecu_elem).ok())
    }

    /// create a new `CanTpAddress` in the configuration
    pub fn create_can_tp_address(&self, name: &str, address: u32) -> Result<CanTpAddress, AutosarAbstractionError> {
        let addresses_container = self.element().get_or_create_sub_element(ElementName::TpAddresss)?;
        CanTpAddress::new(name, &addresses_container, address)
    }

    /// get all of the Can Tp addresses in the configuration
    pub fn can_tp_addresses(&self) -> impl Iterator<Item = CanTpAddress> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::TpAddresss)
            .into_iter()
            .flat_map(|addresses_container| {
                addresses_container
                    .sub_elements()
                    .filter_map(|address_elem| CanTpAddress::try_from(address_elem).ok())
            })
    }

    /// create a new `CanTpChannel` in the configuration
    pub fn create_can_tp_channel(
        &self,
        name: &str,
        channel_id: u32,
        mode: CanTpChannelMode,
    ) -> Result<CanTpChannel, AutosarAbstractionError> {
        let channels_container = self.element().get_or_create_sub_element(ElementName::TpChannels)?;
        CanTpChannel::new(name, &channels_container, channel_id, mode)
    }

    /// iterate over all `CanTpChannel`s in the configuration
    pub fn can_tp_channels(&self) -> impl Iterator<Item = CanTpChannel> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::TpChannels)
            .into_iter()
            .flat_map(|channels_container| {
                channels_container
                    .sub_elements()
                    .filter_map(|channel_elem| CanTpChannel::try_from(channel_elem).ok())
            })
    }

    /// create a new `CanTpConnection` in the configuration
    pub fn create_can_tp_connection<T: AbstractIpdu>(
        &self,
        name: Option<&str>,
        addressing_format: CanTpAddressingFormat,
        can_tp_channel: &CanTpChannel,
        data_pdu: &NPdu,
        tp_sdu: &T,
        padding_activation: bool,
    ) -> Result<CanTpConnection, AutosarAbstractionError> {
        let connections_container = self.element().get_or_create_sub_element(ElementName::TpConnections)?;
        CanTpConnection::new(
            name,
            &connections_container,
            addressing_format,
            can_tp_channel,
            data_pdu,
            &tp_sdu.clone().into(),
            padding_activation,
        )
    }

    /// iterate over all `CanTpConnections` in the configuration
    pub fn can_tp_connections(&self) -> impl Iterator<Item = CanTpConnection> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::TpConnections)
            .into_iter()
            .flat_map(|connections_container| {
                connections_container
                    .sub_elements()
                    .filter_map(|connection_elem| CanTpConnection::try_from(connection_elem).ok())
            })
    }

    /// create a new `CanTpNode` in the configuration
    pub fn create_can_tp_node(&self, name: &str) -> Result<CanTpNode, AutosarAbstractionError> {
        let nodes_container = self.element().get_or_create_sub_element(ElementName::TpNodes)?;
        CanTpNode::new(name, &nodes_container)
    }

    /// get all of the `CanTpNodes` in the configuration
    pub fn can_tp_nodes(&self) -> impl Iterator<Item = CanTpNode> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::TpNodes)
            .into_iter()
            .flat_map(|nodes_container| {
                nodes_container
                    .sub_elements()
                    .filter_map(|node_elem| CanTpNode::try_from(node_elem).ok())
            })
    }
}

//#########################################################

/// A `CanTpEcu` represents an ECU that is using the `CanTp` module
#[derive(Debug, Clone, PartialEq)]
pub struct CanTpEcu(Element);
abstraction_element!(CanTpEcu, CanTpEcu);

impl CanTpEcu {
    pub(crate) fn new(
        parent: &Element,
        ecu_instance: &EcuInstance,
        cycle_time_main_function: Option<f64>,
    ) -> Result<Self, AutosarAbstractionError> {
        let tp_ecu_elem = parent.create_sub_element(ElementName::CanTpEcu)?;
        let tp_ecu = Self(tp_ecu_elem);

        tp_ecu.set_ecu_instance(ecu_instance)?;
        tp_ecu.set_cycle_time_main_function(cycle_time_main_function)?;

        Ok(tp_ecu)
    }

    /// set the ECU instance of the `CanTpEcu`
    pub fn set_ecu_instance(&self, ecu_instance: &EcuInstance) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::EcuInstanceRef)?
            .set_reference_target(ecu_instance.element())?;
        Ok(())
    }

    /// get the ECU instance of the `CanTpEcu`
    #[must_use]
    pub fn ecu_instance(&self) -> Option<EcuInstance> {
        self.element()
            .get_sub_element(ElementName::EcuInstanceRef)
            .and_then(|elem| elem.get_reference_target().ok())
            .and_then(|target| EcuInstance::try_from(target).ok())
    }

    /// set the cycle time of the `CanTp` main function of the ECU
    pub fn set_cycle_time_main_function(&self, cycle_time: Option<f64>) -> Result<(), AutosarAbstractionError> {
        if let Some(cycle_time) = cycle_time {
            self.element()
                .get_or_create_sub_element(ElementName::CycleTimeMainFunction)?
                .set_character_data(cycle_time)?;
        } else {
            let _ = self
                .element()
                .remove_sub_element_kind(ElementName::CycleTimeMainFunction);
        }
        Ok(())
    }

    /// get the cycle time of the `CanTp` main function of the ECU
    #[must_use]
    pub fn cycle_time_main_function(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::CycleTimeMainFunction)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_float())
    }
}

//#########################################################

/// A `CanTpAddress` represents a logical address in the `CanTp` module
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanTpAddress(Element);
abstraction_element!(CanTpAddress, CanTpAddress);
impl IdentifiableAbstractionElement for CanTpAddress {}

impl CanTpAddress {
    pub(crate) fn new(name: &str, parent: &Element, tp_address: u32) -> Result<Self, AutosarAbstractionError> {
        let address_elem = parent.create_named_sub_element(ElementName::CanTpAddress, name)?;
        let address = Self(address_elem);
        address.set_tp_address(tp_address)?;

        Ok(address)
    }

    /// set the address value of the `CanTpAddress`
    pub fn set_tp_address(&self, tp_address: u32) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::TpAddress)?
            .set_character_data(u64::from(tp_address))?;
        Ok(())
    }

    /// get the address value of the `CanTpAddress`
    #[must_use]
    pub fn tp_address(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::TpAddress)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }
}

//#########################################################

/// A `CanTpChannel` represents a channel in the `CanTp` module
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanTpChannel(Element);
abstraction_element!(CanTpChannel, CanTpChannel);
impl IdentifiableAbstractionElement for CanTpChannel {}

impl CanTpChannel {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        channel_id: u32,
        mode: CanTpChannelMode,
    ) -> Result<Self, AutosarAbstractionError> {
        let channel_elem = parent.create_named_sub_element(ElementName::CanTpChannel, name)?;
        let channel = Self(channel_elem);

        channel.set_channel_id(channel_id)?;
        channel.set_channel_mode(mode)?;

        Ok(channel)
    }

    /// set the channel id of the channel
    pub fn set_channel_id(&self, channel_id: u32) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::ChannelId)?
            .set_character_data(u64::from(channel_id))?;
        Ok(())
    }

    /// get the channel id of the channel
    #[must_use]
    pub fn channel_id(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::ChannelId)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set the channel mode of the channel
    pub fn set_channel_mode(&self, mode: CanTpChannelMode) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::ChannelMode)?
            .set_character_data::<EnumItem>(mode.into())?;
        Ok(())
    }

    /// get the channel mode of the channel
    #[must_use]
    pub fn channel_mode(&self) -> Option<CanTpChannelMode> {
        self.element()
            .get_sub_element(ElementName::ChannelMode)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.enum_value())
            .and_then(|enumitem| enumitem.try_into().ok())
    }
}

//#########################################################

/// The mode of a `CanTpChannel`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanTpChannelMode {
    /// Full duplex mode
    FullDuplex,
    /// Half duplex mode
    HalfDuplex,
}

impl From<CanTpChannelMode> for EnumItem {
    fn from(mode: CanTpChannelMode) -> Self {
        match mode {
            CanTpChannelMode::FullDuplex => EnumItem::FullDuplexMode,
            CanTpChannelMode::HalfDuplex => EnumItem::HalfDuplexMode,
        }
    }
}

impl TryFrom<EnumItem> for CanTpChannelMode {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<Self, Self::Error> {
        match value {
            EnumItem::FullDuplexMode => Ok(CanTpChannelMode::FullDuplex),
            EnumItem::HalfDuplexMode => Ok(CanTpChannelMode::HalfDuplex),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "CanTpChannelMode".to_string(),
            }),
        }
    }
}

//#########################################################

/// A connection identifies the sender and the receiver of this particular communication.
/// The `CanTp` module routes a Pdu through this connection.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanTpConnection(Element);
abstraction_element!(CanTpConnection, CanTpConnection);

impl IdentifiableAbstractionElement for CanTpConnection {
    /// get the name of the connection
    ///
    /// In early versions of the Autosar standard, `CanTpConnection` was not identifiable.
    /// This was fixed later by adding the Ident sub-element. This method returns the name
    /// provied in the Ident element, if it exists.
    #[must_use]
    fn name(&self) -> Option<String> {
        self.element()
            .get_sub_element(ElementName::Ident)
            .and_then(|elem| elem.item_name())
    }

    /// set the name of the connection
    fn set_name(&self, name: &str) -> Result<(), AutosarAbstractionError> {
        // rename an existing Ident element or create a new one
        if let Some(ident_elem) = self.element().get_sub_element(ElementName::Ident) {
            ident_elem.set_item_name(name)?;
        } else {
            self.element().create_named_sub_element(ElementName::Ident, name)?;
        }
        Ok(())
    }
}

impl CanTpConnection {
    pub(crate) fn new(
        name: Option<&str>,
        parent: &Element,
        addressing_format: CanTpAddressingFormat,
        can_tp_channel: &CanTpChannel,
        data_pdu: &NPdu,
        tp_sdu: &IPdu,
        padding_activation: bool,
    ) -> Result<Self, AutosarAbstractionError> {
        let connection_elem = parent.create_sub_element(ElementName::CanTpConnection)?;
        let connection = Self(connection_elem);

        if let Some(name) = name {
            connection.set_name(name)?;
        }
        connection.set_addressing_format(addressing_format)?;
        connection.set_channel(can_tp_channel)?;
        connection.set_data_pdu(data_pdu)?;
        connection.set_tp_sdu(tp_sdu)?;
        connection.set_padding_activation(padding_activation)?;

        Ok(connection)
    }

    /// set the `CanTpChannel` associated with this connection
    pub fn set_channel(&self, channel: &CanTpChannel) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::CanTpChannelRef)?
            .set_reference_target(channel.element())?;
        Ok(())
    }

    /// get the `CanTpChannel` associated with this connection
    #[must_use]
    pub fn channel(&self) -> Option<CanTpChannel> {
        self.element()
            .get_sub_element(ElementName::CanTpChannelRef)
            .and_then(|elem| elem.get_reference_target().ok())
            .and_then(|target| CanTpChannel::try_from(target).ok())
    }

    /// set the `NPdu` associated with this connection
    pub fn set_data_pdu(&self, data_pdu: &NPdu) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::DataPduRef)?
            .set_reference_target(data_pdu.element())?;
        Ok(())
    }

    /// get the `NPdu` associated with this connection
    ///
    /// This is the Pdu that is sent over the CAN network
    #[must_use]
    pub fn data_pdu(&self) -> Option<NPdu> {
        self.element()
            .get_sub_element(ElementName::DataPduRef)
            .and_then(|elem| elem.get_reference_target().ok())
            .and_then(|target| NPdu::try_from(target).ok())
    }

    /// set the `IPdu` associated with this connection
    pub fn set_tp_sdu<T: AbstractIpdu>(&self, tp_sdu: &T) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::TpSduRef)?
            .set_reference_target(tp_sdu.element())?;
        Ok(())
    }

    /// get the `IPdu` associated with this connection
    ///
    /// This is the Pdu that is sent over the transport protocol
    #[must_use]
    pub fn tp_sdu(&self) -> Option<IPdu> {
        self.element()
            .get_sub_element(ElementName::TpSduRef)
            .and_then(|elem| elem.get_reference_target().ok())
            .and_then(|target| IPdu::try_from(target).ok())
    }

    /// set the addressing format of the connection
    pub fn set_addressing_format(
        &self,
        addressing_format: CanTpAddressingFormat,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::AddressingFormat)?
            .set_character_data::<EnumItem>(addressing_format.into())?;
        Ok(())
    }

    /// get the addressing format of the connection
    #[must_use]
    pub fn addressing_format(&self) -> Option<CanTpAddressingFormat> {
        self.element()
            .get_sub_element(ElementName::AddressingFormat)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.enum_value())
            .and_then(|enumitem| enumitem.try_into().ok())
    }

    /// set the padding activation of the connection
    pub fn set_padding_activation(&self, padding_activation: bool) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::PaddingActivation)?
            .set_character_data(padding_activation)?;
        Ok(())
    }

    /// get the padding activation of the connection
    #[must_use]
    pub fn padding_activation(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::PaddingActivation)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_bool())
    }

    /// set the transmitter of the connection
    ///
    /// This is a `CanTpNode` representing an ECU that will send the data
    pub fn set_transmitter(&self, transmitter: &CanTpNode) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::TransmitterRef)?
            .set_reference_target(transmitter.element())?;
        Ok(())
    }

    /// get the transmitter of the connection
    #[must_use]
    pub fn transmitter(&self) -> Option<CanTpNode> {
        self.element()
            .get_sub_element(ElementName::TransmitterRef)
            .and_then(|elem| elem.get_reference_target().ok())
            .and_then(|target| CanTpNode::try_from(target).ok())
    }

    /// add a receiver to the connection
    ///
    /// This is a `CanTpNode` representing an ECU that will receive the data
    pub fn add_receiver(&self, receiver: &CanTpNode) -> Result<(), AutosarAbstractionError> {
        let receivers = self.element().get_or_create_sub_element(ElementName::ReceiverRefs)?;
        let receiver_ref_elem = receivers.create_sub_element(ElementName::ReceiverRef)?;
        receiver_ref_elem.set_reference_target(receiver.element())?;
        Ok(())
    }

    /// get all of the receivers of the connection
    pub fn receivers(&self) -> impl Iterator<Item = CanTpNode> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::ReceiverRefs)
            .into_iter()
            .flat_map(|receivers| {
                receivers.sub_elements().filter_map(|receiver_ref_elem| {
                    receiver_ref_elem
                        .get_reference_target()
                        .ok()
                        .and_then(|target| CanTpNode::try_from(target).ok())
                })
            })
    }
}

//#########################################################

/// The addressing format of a `CanTpConnection`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanTpAddressingFormat {
    /// Extended addressing format
    Extended,
    /// Mixed 11-bit addressing format
    Mixed,
    /// Mixed 29-bit addressing format
    Mixed29Bit,
    /// Normal fixed addressing format
    NormalFixed,
    /// Standard addressing format
    Standard,
}

impl From<CanTpAddressingFormat> for EnumItem {
    fn from(format: CanTpAddressingFormat) -> Self {
        match format {
            CanTpAddressingFormat::Extended => EnumItem::Extended,
            CanTpAddressingFormat::Mixed => EnumItem::Mixed,
            CanTpAddressingFormat::Mixed29Bit => EnumItem::Mixed29Bit,
            CanTpAddressingFormat::NormalFixed => EnumItem::Normalfixed,
            CanTpAddressingFormat::Standard => EnumItem::Standard,
        }
    }
}

impl TryFrom<EnumItem> for CanTpAddressingFormat {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<Self, Self::Error> {
        match value {
            EnumItem::Extended => Ok(CanTpAddressingFormat::Extended),
            EnumItem::Mixed => Ok(CanTpAddressingFormat::Mixed),
            EnumItem::Mixed29Bit => Ok(CanTpAddressingFormat::Mixed29Bit),
            EnumItem::Normalfixed => Ok(CanTpAddressingFormat::NormalFixed),
            EnumItem::Standard => Ok(CanTpAddressingFormat::Standard),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "CanTpAddressingFormat".to_string(),
            }),
        }
    }
}

//#########################################################

/// A `CanTpNode` provides the TP address and the connection to the topology description in a `CanTpConfig`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanTpNode(Element);
abstraction_element!(CanTpNode, CanTpNode);

impl IdentifiableAbstractionElement for CanTpNode {}

impl CanTpNode {
    pub(crate) fn new(name: &str, parent: &Element) -> Result<Self, AutosarAbstractionError> {
        let node_elem = parent.create_named_sub_element(ElementName::CanTpNode, name)?;
        Ok(Self(node_elem))
    }

    /// set the `CanTpAddress` of this Node
    pub fn set_address(&self, address: &CanTpAddress) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::TpAddressRef)?
            .set_reference_target(address.element())?;
        Ok(())
    }

    /// get the `CanTpAddress` of this Node
    #[must_use]
    pub fn address(&self) -> Option<CanTpAddress> {
        self.element()
            .get_sub_element(ElementName::TpAddressRef)
            .and_then(|elem| elem.get_reference_target().ok())
            .and_then(|target| CanTpAddress::try_from(target).ok())
    }

    /// set the reference to a `CanCommunicationConnector` of an ECU and a `CanPhysicalChannel`
    ///
    /// The connector connects the ECU to the physical channel, so by setting this reference, the
    /// ECU is also connected to the `CanTpNode`
    pub fn set_connector(&self, connector: &CanCommunicationConnector) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::ConnectorRef)?
            .set_reference_target(connector.element())?;
        Ok(())
    }

    /// get the `CanCommunicationConnector` of this Node
    #[must_use]
    pub fn connector(&self) -> Option<CanCommunicationConnector> {
        self.element()
            .get_sub_element(ElementName::ConnectorRef)
            .and_then(|elem| elem.get_reference_target().ok())
            .and_then(|target| CanCommunicationConnector::try_from(target).ok())
    }
}

//#########################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::{AutosarModelAbstraction, SystemCategory, communication::CanClusterSettings};
    use autosar_data::AutosarVersion;

    #[test]
    fn can_transport_protocol() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/pkg1").unwrap();

        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();
        let can_cluster = system
            .create_can_cluster("can_cluster", &package, &CanClusterSettings::default())
            .unwrap();
        let can_channel = can_cluster.create_physical_channel("can_channel").unwrap();
        let ecu_instance = system.create_ecu_instance("ecu_instance", &package).unwrap();
        let communication_controller = ecu_instance.create_can_communication_controller("can_ctrl").unwrap();
        let connector = communication_controller
            .connect_physical_channel("name", &can_channel)
            .unwrap();

        let can_tp_config = system
            .create_can_tp_config("can_tp_config", &package, &can_cluster)
            .unwrap();
        assert_eq!(can_tp_config.cluster().unwrap(), can_cluster);

        let tp_ecu = can_tp_config.create_can_tp_ecu(&ecu_instance, Some(1.0)).unwrap();
        assert_eq!(can_tp_config.can_tp_ecus().count(), 1);
        assert_eq!(can_tp_config.can_tp_ecus().next().unwrap(), tp_ecu);
        assert_eq!(tp_ecu.ecu_instance().unwrap().name().unwrap(), "ecu_instance");
        assert_eq!(tp_ecu.cycle_time_main_function().unwrap(), 1.0);

        let address = can_tp_config.create_can_tp_address("address", 0x1234).unwrap();
        assert_eq!(address.tp_address().unwrap(), 0x1234);
        assert_eq!(can_tp_config.can_tp_addresses().count(), 1);
        assert_eq!(can_tp_config.can_tp_addresses().next().unwrap(), address);

        let channel = can_tp_config
            .create_can_tp_channel("channel", 1, CanTpChannelMode::FullDuplex)
            .unwrap();
        let channel2 = can_tp_config
            .create_can_tp_channel("channel2", 2, CanTpChannelMode::FullDuplex)
            .unwrap();
        assert_eq!(can_tp_config.can_tp_channels().count(), 2);
        assert_eq!(channel.channel_id().unwrap(), 1);
        assert_eq!(channel.channel_mode().unwrap(), CanTpChannelMode::FullDuplex);

        let data_pdu = system.create_n_pdu("data_pdu", &package, 8).unwrap();
        let data_pdu2 = system.create_n_pdu("data_pdu2", &package, 8).unwrap();
        let tp_sdu = system.create_dcm_ipdu("ipdu", &package, 4096).unwrap();
        let tp_sdu2 = system.create_dcm_ipdu("ipdu2", &package, 4096).unwrap();

        let connection = can_tp_config
            .create_can_tp_connection(
                Some("connection"),
                CanTpAddressingFormat::Standard,
                &channel,
                &data_pdu,
                &tp_sdu,
                false,
            )
            .unwrap();
        assert_eq!(can_tp_config.can_tp_connections().count(), 1);
        assert_eq!(can_tp_config.can_tp_connections().next().unwrap(), connection);

        assert_eq!(connection.name().unwrap(), "connection");
        // in a CanTpConnection, the name is provided by the optional IDENT sub-element
        connection
            .element()
            .remove_sub_element_kind(ElementName::Ident)
            .unwrap();
        assert_eq!(connection.name(), None);
        connection.set_name("new_name").unwrap();
        assert_eq!(connection.name().unwrap(), "new_name");

        assert_eq!(connection.channel().unwrap(), channel);
        connection.set_channel(&channel2).unwrap();
        assert_eq!(connection.channel().unwrap(), channel2);

        assert_eq!(connection.data_pdu().unwrap(), data_pdu);
        connection.set_data_pdu(&data_pdu2).unwrap();
        assert_eq!(connection.data_pdu().unwrap(), data_pdu2);

        assert_eq!(connection.tp_sdu().unwrap(), tp_sdu.into());
        connection.set_tp_sdu(&tp_sdu2).unwrap();
        assert_eq!(connection.tp_sdu().unwrap(), tp_sdu2.into());

        assert_eq!(connection.addressing_format().unwrap(), CanTpAddressingFormat::Standard);
        connection
            .set_addressing_format(CanTpAddressingFormat::Extended)
            .unwrap();
        assert_eq!(connection.addressing_format().unwrap(), CanTpAddressingFormat::Extended);

        assert_eq!(connection.padding_activation().unwrap(), false);
        connection.set_padding_activation(true).unwrap();
        assert_eq!(connection.padding_activation().unwrap(), true);

        let node = can_tp_config.create_can_tp_node("node").unwrap();
        assert_eq!(can_tp_config.can_tp_nodes().count(), 1);
        assert_eq!(can_tp_config.can_tp_nodes().next().unwrap(), node);

        node.set_address(&address).unwrap();
        assert_eq!(node.address().unwrap(), address);
        node.set_connector(&connector).unwrap();
        assert_eq!(node.connector().unwrap(), connector);
        connection.set_transmitter(&node).unwrap();
        assert_eq!(connection.transmitter().unwrap(), node);

        connection.add_receiver(&node).unwrap();
        assert_eq!(connection.receivers().count(), 1);
    }
}
