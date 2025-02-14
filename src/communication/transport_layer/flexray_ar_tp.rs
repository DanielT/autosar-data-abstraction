use crate::communication::{AbstractIpdu, FlexrayCluster, FlexrayCommunicationConnector, NPdu, Pdu, TpAddress};
use crate::{
    abstraction_element, AbstractionElement, ArPackage, AutosarAbstractionError, IdentifiableAbstractionElement,
};
use autosar_data::{Element, ElementName, EnumItem};

//#########################################################

/// The `FlexrayArTpConfig` represents the configuration of the Flexray Autosar Transport Protocol
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlexrayArTpConfig(Element);
abstraction_element!(FlexrayArTpConfig, FlexrayArTpConfig);
impl IdentifiableAbstractionElement for FlexrayArTpConfig {}

impl FlexrayArTpConfig {
    pub(crate) fn new(
        name: &str,
        package: &ArPackage,
        cluster: &FlexrayCluster,
    ) -> Result<Self, AutosarAbstractionError> {
        let pkg_elem = package.element().get_or_create_sub_element(ElementName::Elements)?;

        let tp_config_elem = pkg_elem.create_named_sub_element(ElementName::FlexrayArTpConfig, name)?;
        tp_config_elem
            .create_sub_element(ElementName::CommunicationClusterRef)?
            .set_reference_target(cluster.element())?;

        Ok(Self(tp_config_elem))
    }

    /// create a new `TpAddress`
    pub fn create_tp_address(&self, name: &str, address: u32) -> Result<TpAddress, AutosarAbstractionError> {
        let tp_addresses_elem = self.element().get_or_create_sub_element(ElementName::TpAddresss)?;
        TpAddress::new(name, &tp_addresses_elem, address)
    }

    /// iterate over all `TpAddresses`
    pub fn tp_addresses(&self) -> impl Iterator<Item = TpAddress> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::TpAddresss)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|elem| elem.try_into().ok())
    }

    /// create a new `FlexrayArTpChannel`
    pub fn create_flexray_ar_tp_channel(
        &self,
        ack_type: FrArTpAckType,
        extended_addressing: bool,
        maximum_message_length: MaximumMessageLengthType,
        minimum_separation_time: f32,
        multicast_segmentation: bool,
    ) -> Result<FlexrayArTpChannel, AutosarAbstractionError> {
        let tp_channels_elem = self.element().get_or_create_sub_element(ElementName::TpChannels)?;
        FlexrayArTpChannel::new(
            &tp_channels_elem,
            ack_type,
            extended_addressing,
            maximum_message_length,
            minimum_separation_time,
            multicast_segmentation,
        )
    }

    /// get an iterator over the channels in the configuration
    pub fn channels(&self) -> impl Iterator<Item = FlexrayArTpChannel> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::TpChannels)
            .into_iter()
            .flat_map(|tp_channels_elem| tp_channels_elem.sub_elements())
            .filter_map(|tp_channel_elem| tp_channel_elem.try_into().ok())
    }

    /// create a new `FlexrayArTpNode`
    pub fn create_flexray_ar_tp_node(&self, name: &str) -> Result<FlexrayArTpNode, AutosarAbstractionError> {
        let tp_nodes_elem = self.element().get_or_create_sub_element(ElementName::TpNodes)?;
        FlexrayArTpNode::new(name, &tp_nodes_elem)
    }

    /// get an iterator over the nodes
    pub fn nodes(&self) -> impl Iterator<Item = FlexrayArTpNode> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::TpNodes)
            .into_iter()
            .flat_map(|tp_nodes_elem| tp_nodes_elem.sub_elements())
            .filter_map(|tp_node_elem| tp_node_elem.try_into().ok())
    }
}

//#########################################################

/// The `FlexrayArTpChannel` represents a channel in the Flexray Autosar Transport Protocol
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlexrayArTpChannel(Element);
abstraction_element!(FlexrayArTpChannel, FlexrayArTpChannel);

impl FlexrayArTpChannel {
    pub(crate) fn new(
        parent: &Element,
        ack_type: FrArTpAckType,
        extended_addressing: bool,
        maximum_message_length: MaximumMessageLengthType,
        minimum_separation_time: f32,
        multicast_segmentation: bool,
    ) -> Result<Self, AutosarAbstractionError> {
        let tp_channel_elem = parent.create_sub_element(ElementName::FlexrayArTpChannel)?;
        tp_channel_elem
            .create_sub_element(ElementName::AckType)?
            .set_character_data::<EnumItem>(ack_type.into())?;
        tp_channel_elem
            .create_sub_element(ElementName::ExtendedAddressing)?
            .set_character_data(extended_addressing)?;
        tp_channel_elem
            .create_sub_element(ElementName::MaximumMessageLength)?
            .set_character_data::<EnumItem>(maximum_message_length.into())?;
        tp_channel_elem
            .create_sub_element(ElementName::MinimumSeparationTime)?
            .set_character_data(f64::from(minimum_separation_time))?;
        tp_channel_elem
            .create_sub_element(ElementName::MulticastSegmentation)?
            .set_character_data(multicast_segmentation)?;

        Ok(Self(tp_channel_elem))
    }

    /// get the ack type
    #[must_use]
    pub fn ack_type(&self) -> Option<FrArTpAckType> {
        self.element()
            .get_sub_element(ElementName::AckType)?
            .character_data()?
            .enum_value()?
            .try_into()
            .ok()
    }

    /// get the extended addressing
    #[must_use]
    pub fn extended_addressing(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::ExtendedAddressing)?
            .character_data()?
            .parse_bool()
    }

    /// get the maximum message length type
    #[must_use]
    pub fn maximum_message_length(&self) -> Option<MaximumMessageLengthType> {
        self.element()
            .get_sub_element(ElementName::MaximumMessageLength)?
            .character_data()?
            .enum_value()?
            .try_into()
            .ok()
    }

    /// get the minimum separation time
    #[must_use]
    pub fn minimum_separation_time(&self) -> Option<f32> {
        self.element()
            .get_sub_element(ElementName::MinimumSeparationTime)?
            .character_data()?
            .float_value()
            .map(|v| v as f32)
    }

    /// get the multicast segmentation
    #[must_use]
    pub fn multicast_segmentation(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::MulticastSegmentation)?
            .character_data()?
            .parse_bool()
    }

    /// create a new `FlexrayArTpConnection` for this channel
    pub fn create_flexray_ar_tp_connection<T: AbstractIpdu>(
        &self,
        name: Option<&str>,
        direct_tp_sdu: &T,
        source: &FlexrayArTpNode,
        target: &FlexrayArTpNode,
    ) -> Result<FlexrayArTpConnection, AutosarAbstractionError> {
        let parent = self.element().get_or_create_sub_element(ElementName::TpConnections)?;
        FlexrayArTpConnection::new(name, &parent, &direct_tp_sdu.clone().into(), source, target)
    }

    /// add an N-PDU to the channel
    ///
    /// The `NPdus` are logically assembled into a pool of Rx `NPdus` and another pool of Tx `NPdus`.
    /// This function is supported on autosar 4.1 and later, while Autosar 4.0 uses a different approach.
    pub fn add_npdu(&self, n_pdu: &NPdu) -> Result<(), AutosarAbstractionError> {
        let npdu_refs_elem = self.element().get_or_create_sub_element(ElementName::NPduRefs)?;
        npdu_refs_elem
            .create_sub_element(ElementName::NPduRef)?
            .set_reference_target(n_pdu.element())?;
        Ok(())
    }

    /// get the `NPdus` of the channel
    pub fn npdus(&self) -> impl Iterator<Item = NPdu> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::NPduRefs)
            .into_iter()
            .flat_map(|npdu_refs_elem| npdu_refs_elem.sub_elements())
            .filter_map(|npdu_ref_elem| {
                npdu_ref_elem
                    .get_reference_target()
                    .ok()
                    .and_then(|npdu_elem| npdu_elem.try_into().ok())
            })
    }
}

//#########################################################

/// Types of Acknowledgement that can be used in an `FlexrayArTpChannel`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrArTpAckType {
    /// Acknowledgement without retry
    AckWithoutRt,
    /// Acknowledgement with retry
    AckWithRt,
    /// No acknowledgement
    NoAck,
}

impl TryFrom<EnumItem> for FrArTpAckType {
    type Error = AutosarAbstractionError;

    fn try_from(item: EnumItem) -> Result<Self, Self::Error> {
        match item {
            EnumItem::AckWithoutRt => Ok(Self::AckWithoutRt),
            EnumItem::AckWithRt => Ok(Self::AckWithRt),
            EnumItem::NoAck => Ok(Self::NoAck),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: item.to_string(),
                dest: "FrArTpAckType".to_string(),
            }),
        }
    }
}

impl From<FrArTpAckType> for EnumItem {
    fn from(val: FrArTpAckType) -> Self {
        match val {
            FrArTpAckType::AckWithoutRt => EnumItem::AckWithoutRt,
            FrArTpAckType::AckWithRt => EnumItem::AckWithRt,
            FrArTpAckType::NoAck => EnumItem::NoAck,
        }
    }
}

//#########################################################

/// Types of Maximum Message Length that can be used in an `FlexrayArTpChannel`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaximumMessageLengthType {
    /// I4G: up to (2**32)-1 byte message length
    I4g,
    /// ISO: up to 4095 byte message length
    Iso,
    /// ISO6: payload length is limited to 6 byte (SF-I, FF-I, CF). This is necessary to route TP on CAN
    Iso6,
}

impl TryFrom<EnumItem> for MaximumMessageLengthType {
    type Error = AutosarAbstractionError;

    fn try_from(item: EnumItem) -> Result<Self, Self::Error> {
        match item {
            EnumItem::I4G => Ok(Self::I4g),
            EnumItem::Iso => Ok(Self::Iso),
            EnumItem::Iso6 => Ok(Self::Iso6),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: item.to_string(),
                dest: "MaximumMessageLengthType".to_string(),
            }),
        }
    }
}

impl From<MaximumMessageLengthType> for EnumItem {
    fn from(val: MaximumMessageLengthType) -> Self {
        match val {
            MaximumMessageLengthType::I4g => EnumItem::I4G,
            MaximumMessageLengthType::Iso => EnumItem::Iso,
            MaximumMessageLengthType::Iso6 => EnumItem::Iso6,
        }
    }
}

//#########################################################

/// `FlexrayArTpConnection` represents a connection within a `FlexrayArTpChannel`
///
/// The connection identifies the sender and the receiver of this particular communication.
/// The `FlexRay` Autosar Tp module routes a Pdu through this connection.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlexrayArTpConnection(Element);
abstraction_element!(FlexrayArTpConnection, FlexrayArTpConnection);

impl IdentifiableAbstractionElement for FlexrayArTpConnection {
    /// get the name of the connection
    ///
    /// In early versions of the Autosar standard, TpConnections were not identifiable.
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
        if let Some(ident_elem) = self.element().get_sub_element(ElementName::Ident) {
            ident_elem.set_item_name(name)?;
        } else {
            self.element().create_named_sub_element(ElementName::Ident, name)?;
        }
        Ok(())
    }
}

impl FlexrayArTpConnection {
    pub(crate) fn new(
        name: Option<&str>,
        parent: &Element,
        direct_tp_sdu: &Pdu,
        source: &FlexrayArTpNode,
        target: &FlexrayArTpNode,
    ) -> Result<Self, AutosarAbstractionError> {
        let tp_connection_elem = parent.create_sub_element(ElementName::FlexrayArTpConnection)?;
        if let Some(name) = name {
            tp_connection_elem.create_named_sub_element(ElementName::Ident, name)?;
        }
        tp_connection_elem
            .create_sub_element(ElementName::DirectTpSduRef)?
            .set_reference_target(direct_tp_sdu.element())?;
        tp_connection_elem
            .create_sub_element(ElementName::SourceRef)?
            .set_reference_target(source.element())?;
        tp_connection_elem
            .create_sub_element(ElementName::TargetRefs)?
            .create_sub_element(ElementName::TargetRef)?
            .set_reference_target(target.element())?;

        Ok(Self(tp_connection_elem))
    }

    /// get the direct tp sdu
    #[must_use]
    pub fn direct_tp_sdu(&self) -> Option<Pdu> {
        self.element()
            .get_sub_element(ElementName::DirectTpSduRef)
            .and_then(|refelem| refelem.get_reference_target().ok())
            .and_then(|elem| elem.try_into().ok())
    }

    /// get the source
    #[must_use]
    pub fn source(&self) -> Option<FlexrayArTpNode> {
        self.element()
            .get_sub_element(ElementName::SourceRef)
            .and_then(|refelem| refelem.get_reference_target().ok())
            .and_then(|elem| elem.try_into().ok())
    }

    /// add a target to the connection
    ///
    /// The connection can have multiple targets, but at least one target is required.
    pub fn add_target(&self, target: &FlexrayArTpNode) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::TargetRefs)?
            .create_sub_element(ElementName::TargetRef)?
            .set_reference_target(target.element())?;
        Ok(())
    }

    /// get the targets
    pub fn targets(&self) -> impl Iterator<Item = FlexrayArTpNode> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::TargetRefs)
            .into_iter()
            .flat_map(|target_refs_elem| target_refs_elem.sub_elements())
            .filter_map(|target_ref_elem| {
                target_ref_elem
                    .get_reference_target()
                    .ok()
                    .and_then(|target_elem| target_elem.try_into().ok())
            })
    }

    /// set or remove the reversed TP SDU
    ///
    /// If the connection supports both directions, then the reversed TP SDU is required.
    /// if Some(value) is passed, the reversed TP SDU is set to the given value, otherwise it is removed.
    pub fn set_reversed_tp_sdu<T: AbstractIpdu>(
        &self,
        reversed_tp_sdu: Option<&T>,
    ) -> Result<(), AutosarAbstractionError> {
        if let Some(reversed_tp_sdu) = reversed_tp_sdu {
            self.element()
                .get_or_create_sub_element(ElementName::ReversedTpSduRef)?
                .set_reference_target(reversed_tp_sdu.element())?;
        } else if let Some(reversed_tp_sdu_elem) = self.element().get_sub_element(ElementName::ReversedTpSduRef) {
            self.element().remove_sub_element(reversed_tp_sdu_elem)?;
        }
        Ok(())
    }

    /// get the reversed tp sdu
    #[must_use]
    pub fn reversed_tp_sdu(&self) -> Option<Pdu> {
        self.element()
            .get_sub_element(ElementName::ReversedTpSduRef)
            .and_then(|refelem| refelem.get_reference_target().ok())
            .and_then(|elem| elem.try_into().ok())
    }
}

//#########################################################

/// `FlexrayArTpNode` represents a node in the Flexray Autosar Transport Protocol
///
/// A TP node (sender or receiver) provides the TP address and the connection to the topology description
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlexrayArTpNode(Element);
abstraction_element!(FlexrayArTpNode, FlexrayArTpNode);
impl IdentifiableAbstractionElement for FlexrayArTpNode {}

impl FlexrayArTpNode {
    pub(crate) fn new(name: &str, parent: &Element) -> Result<Self, AutosarAbstractionError> {
        let tp_node_elem = parent.create_named_sub_element(ElementName::FlexrayArTpNode, name)?;
        Ok(Self(tp_node_elem))
    }

    /// set or remove the TP address
    ///
    /// if Some(value) is passed, the TP address is set to the given value, otherwise it is removed.
    pub fn set_tp_address(&self, tp_address: Option<&TpAddress>) -> Result<(), AutosarAbstractionError> {
        if let Some(tp_address) = tp_address {
            self.element()
                .get_or_create_sub_element(ElementName::TpAddressRef)?
                .set_reference_target(tp_address.element())?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::TpAddressRef);
        }
        Ok(())
    }

    /// get the TP address
    #[must_use]
    pub fn tp_address(&self) -> Option<TpAddress> {
        self.element()
            .get_sub_element(ElementName::TpAddressRef)
            .and_then(|refelem| refelem.get_reference_target().ok())
            .and_then(|elem| elem.try_into().ok())
    }

    /// add a reference to a `FlexrayCommunicationConnector`
    ///
    /// The connectors define the association with a `PhysicalChannel` and an ECU.
    /// In a `SystemDescription`, this reference is mandatory, but in an `ECUExtract` it is optional.
    /// Up to 2 connectors can be added to a node.
    pub fn add_flexray_communication_connector(
        &self,
        connector: &FlexrayCommunicationConnector,
    ) -> Result<(), AutosarAbstractionError> {
        // Todo: enforce the limit of 2 connectors
        self.element()
            .get_or_create_sub_element(ElementName::ConnectorRefs)?
            .create_sub_element(ElementName::ConnectorRef)?
            .set_reference_target(connector.element())?;
        Ok(())
    }

    /// get the connectors
    pub fn connectors(&self) -> impl Iterator<Item = FlexrayCommunicationConnector> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::ConnectorRefs)
            .into_iter()
            .flat_map(|connector_refs_elem| connector_refs_elem.sub_elements())
            .filter_map(|connector_ref_elem| {
                connector_ref_elem
                    .get_reference_target()
                    .ok()
                    .and_then(|connector_elem| connector_elem.try_into().ok())
            })
    }
}

//#########################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        communication::{FlexrayChannelName, FlexrayClusterSettings},
        AutosarModelAbstraction, SystemCategory,
    };
    use autosar_data::AutosarVersion;

    #[test]
    fn test_flexray_ar_transport_protocol() {
        let model = AutosarModelAbstraction::create("DoipTp.arxml", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/pkg1").unwrap();

        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();
        let flexray_cluster = system
            .create_flexray_cluster("flexray_cluster", &package, &FlexrayClusterSettings::default())
            .unwrap();
        let flexray_channel = flexray_cluster
            .create_physical_channel("flexray_channel_a", FlexrayChannelName::A)
            .unwrap();
        let ecu_instance = system.create_ecu_instance("ecu_instance", &package).unwrap();
        let communication_controller = ecu_instance
            .create_flexray_communication_controller("can_ctrl")
            .unwrap();
        let connector = communication_controller
            .connect_physical_channel("name", &flexray_channel)
            .unwrap();

        // create a direct TP SDU (DCM-I-PDU)
        let tp_sdu = system.create_dcm_ipdu("diag", &package, 1024).unwrap();

        // create some NPdus
        let npdu1 = system.create_n_pdu("npdu1", &package, 64).unwrap();
        let npdu2 = system.create_n_pdu("npdu2", &package, 64).unwrap();

        let fr_ar_tp_config = system
            .create_flexray_ar_tp_config("FrArTpConfig", &package, &flexray_cluster)
            .unwrap();
        let fr_ar_tp_channel = fr_ar_tp_config
            .create_flexray_ar_tp_channel(
                FrArTpAckType::AckWithRt,
                true,
                MaximumMessageLengthType::I4g,
                0.001,
                false,
            )
            .unwrap();
        assert_eq!(fr_ar_tp_config.channels().count(), 1);
        assert_eq!(fr_ar_tp_channel.ack_type(), Some(FrArTpAckType::AckWithRt));
        assert_eq!(fr_ar_tp_channel.extended_addressing(), Some(true));
        assert_eq!(
            fr_ar_tp_channel.maximum_message_length(),
            Some(MaximumMessageLengthType::I4g)
        );
        assert_eq!(fr_ar_tp_channel.minimum_separation_time(), Some(0.001));
        assert_eq!(fr_ar_tp_channel.multicast_segmentation(), Some(false));

        fr_ar_tp_channel.add_npdu(&npdu1).unwrap();
        fr_ar_tp_channel.add_npdu(&npdu2).unwrap();
        assert_eq!(fr_ar_tp_channel.npdus().count(), 2);

        let fr_ar_tp_node_source = fr_ar_tp_config.create_flexray_ar_tp_node("node_s").unwrap();
        let tp_address_source = fr_ar_tp_config.create_tp_address("tp_address_s", 1).unwrap();
        fr_ar_tp_node_source.set_tp_address(Some(&tp_address_source)).unwrap();
        assert_eq!(fr_ar_tp_node_source.tp_address(), Some(tp_address_source));
        fr_ar_tp_node_source
            .add_flexray_communication_connector(&connector)
            .unwrap();
        assert_eq!(fr_ar_tp_node_source.connectors().count(), 1);
        assert_eq!(fr_ar_tp_node_source.connectors().next(), Some(connector));

        let fr_ar_tp_node_target = fr_ar_tp_config.create_flexray_ar_tp_node("node_t").unwrap();
        let tp_address_target = fr_ar_tp_config.create_tp_address("tp_address_t", 2).unwrap();
        fr_ar_tp_node_target.set_tp_address(Some(&tp_address_target)).unwrap();
        assert_eq!(fr_ar_tp_node_target.tp_address(), Some(tp_address_target));

        assert_eq!(fr_ar_tp_config.tp_addresses().count(), 2);
        assert_eq!(fr_ar_tp_config.nodes().count(), 2);

        let flexray_ar_tp_connection = fr_ar_tp_channel
            .create_flexray_ar_tp_connection(Some("conn"), &tp_sdu, &fr_ar_tp_node_source, &fr_ar_tp_node_target)
            .unwrap();
        assert_eq!(flexray_ar_tp_connection.direct_tp_sdu(), Some(tp_sdu.into()));
        assert_eq!(flexray_ar_tp_connection.source(), Some(fr_ar_tp_node_source));
        assert_eq!(flexray_ar_tp_connection.targets().count(), 1);

        assert_eq!(flexray_ar_tp_connection.name(), Some("conn".to_string()));
        flexray_ar_tp_connection.set_name("new_name").unwrap();
        assert_eq!(flexray_ar_tp_connection.name(), Some("new_name".to_string()));

        let reversed_tp_sdu = system.create_dcm_ipdu("reversed_tp_sdu", &package, 1024).unwrap();
        flexray_ar_tp_connection
            .set_reversed_tp_sdu(Some(&reversed_tp_sdu))
            .unwrap();
        assert_eq!(
            flexray_ar_tp_connection.reversed_tp_sdu(),
            Some(reversed_tp_sdu.clone().into())
        );

        let fr_ar_tp_node_extra = fr_ar_tp_config.create_flexray_ar_tp_node("node_extra").unwrap();
        flexray_ar_tp_connection.add_target(&fr_ar_tp_node_extra).unwrap();
        assert_eq!(flexray_ar_tp_connection.targets().count(), 2);
    }
}
