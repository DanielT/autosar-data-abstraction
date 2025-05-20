use crate::communication::{AbstractIpdu, FlexrayCluster, FlexrayCommunicationConnector, IPdu, NPdu, TpAddress};
use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, EcuInstance, IdentifiableAbstractionElement,
    abstraction_element,
};
use autosar_data::{Element, ElementName};

/// `FlexrayTpConfig` defines exactly one `FlexRay` ISO TP Configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlexrayTpConfig(Element);
abstraction_element!(FlexrayTpConfig, FlexrayTpConfig);
impl IdentifiableAbstractionElement for FlexrayTpConfig {}

impl FlexrayTpConfig {
    pub(crate) fn new(
        name: &str,
        package: &ArPackage,
        cluster: &FlexrayCluster,
    ) -> Result<Self, AutosarAbstractionError> {
        let pkg_elem = package.element().get_or_create_sub_element(ElementName::Elements)?;

        let tp_config_elem = pkg_elem.create_named_sub_element(ElementName::FlexrayTpConfig, name)?;
        let tp_config = Self(tp_config_elem);
        tp_config.set_cluster(cluster)?;

        Ok(tp_config)
    }

    /// set the `FlexrayCluster` of the `FlexrayTpConfig`
    pub fn set_cluster(&self, cluster: &FlexrayCluster) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::CommunicationClusterRef)?
            .set_reference_target(cluster.element())?;
        Ok(())
    }

    /// get the `FlexrayCluster` of the `FlexrayTpConfig`
    #[must_use]
    pub fn cluster(&self) -> Option<FlexrayCluster> {
        self.element()
            .get_sub_element(ElementName::CommunicationClusterRef)
            .and_then(|elem| elem.get_reference_target().ok())
            .and_then(|elem| elem.try_into().ok())
    }

    /// create a new `FlexrayTpPduPool`
    pub fn create_flexray_tp_pdu_pool(&self, name: &str) -> Result<FlexrayTpPduPool, AutosarAbstractionError> {
        let pdu_pools_elem = self.element().get_or_create_sub_element(ElementName::PduPools)?;
        FlexrayTpPduPool::new(name, &pdu_pools_elem)
    }

    /// iterate over all `FlexrayTpPduPools`
    pub fn flexray_tp_pdu_pools(&self) -> impl Iterator<Item = FlexrayTpPduPool> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::PduPools)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .map(FlexrayTpPduPool)
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

    /// create a new `FlexrayTpConnection`
    pub fn create_flexray_tp_connection<T: AbstractIpdu>(
        &self,
        name: Option<&str>,
        transmitter: &FlexrayTpNode,
        direct_tp_sdu: &T,
        connection_control: &FlexrayTpConnectionControl,
    ) -> Result<FlexrayTpConnection, AutosarAbstractionError> {
        let tp_connections_elem = self.element().get_or_create_sub_element(ElementName::TpConnections)?;
        FlexrayTpConnection::new(
            name,
            &tp_connections_elem,
            transmitter,
            &direct_tp_sdu.clone().into(),
            connection_control,
        )
    }

    /// iterate over all `FlexrayTpConnections`
    pub fn flexray_tp_connections(&self) -> impl Iterator<Item = FlexrayTpConnection> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::TpConnections)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .map(FlexrayTpConnection)
    }

    /// create a new `FlexrayTpConnectionControl`
    pub fn create_flexray_tp_connection_control(
        &self,
        name: &str,
    ) -> Result<FlexrayTpConnectionControl, AutosarAbstractionError> {
        let connection_controls_elem = self
            .element()
            .get_or_create_sub_element(ElementName::TpConnectionControls)?;
        FlexrayTpConnectionControl::new(name, &connection_controls_elem)
    }

    /// iterate over all `FlexrayTpConnectionControls`
    pub fn flexray_tp_connection_controls(&self) -> impl Iterator<Item = FlexrayTpConnectionControl> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::TpConnectionControls)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .map(FlexrayTpConnectionControl)
    }

    /// create a `FlexrayTpEcu` in the `FlexrayTpConfig`
    pub fn create_flexray_tp_ecu(
        &self,
        ecu_instance: &EcuInstance,
        full_duplex_enabled: bool,
    ) -> Result<FlexrayTpEcu, AutosarAbstractionError> {
        let ecu_collection = self.element().get_or_create_sub_element(ElementName::TpEcus)?;
        FlexrayTpEcu::new(&ecu_collection, ecu_instance, full_duplex_enabled)
    }

    /// iterate over all `FlexrayTpEcus`
    pub fn flexray_tp_ecus(&self) -> impl Iterator<Item = FlexrayTpEcu> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::TpEcus)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|elem| FlexrayTpEcu::try_from(elem).ok())
    }

    /// create a new `FlexrayTpNode`
    pub fn create_flexray_tp_node(&self, name: &str) -> Result<FlexrayTpNode, AutosarAbstractionError> {
        let nodes_elem = self.element().get_or_create_sub_element(ElementName::TpNodes)?;
        FlexrayTpNode::new(name, &nodes_elem)
    }

    /// iterate over all `FlexrayTpNodes`
    pub fn flexray_tp_nodes(&self) -> impl Iterator<Item = FlexrayTpNode> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::TpNodes)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .map(FlexrayTpNode)
    }
}

//##################################################################

/// A `FlexrayTpPduPool` contains a set of `NPdus` that can be used for sending and receiving
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlexrayTpPduPool(Element);
abstraction_element!(FlexrayTpPduPool, FlexrayTpPduPool);
impl IdentifiableAbstractionElement for FlexrayTpPduPool {}

impl FlexrayTpPduPool {
    pub(crate) fn new(name: &str, parent: &Element) -> Result<Self, AutosarAbstractionError> {
        let pdu_pool_elem = parent.create_named_sub_element(ElementName::FlexrayTpPduPool, name)?;
        Ok(Self(pdu_pool_elem))
    }

    /// add an `NPdu` to the `PduPool`
    pub fn add_n_pdu(&self, n_pdu: &NPdu) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NPduRefs)?
            .create_sub_element(ElementName::NPduRef)?
            .set_reference_target(n_pdu.element())?;
        Ok(())
    }

    /// iterate over all referenced `NPdus`
    pub fn n_pdus(&self) -> impl Iterator<Item = NPdu> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::NPduRefs)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|ref_elem| {
                ref_elem
                    .get_reference_target()
                    .ok()
                    .and_then(|elem| elem.try_into().ok())
            })
    }
}

//##################################################################

/// A `FlexrayTpConnection` defines a connection between `FlexrayTpNodes`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlexrayTpConnection(Element);
abstraction_element!(FlexrayTpConnection, FlexrayTpConnection);

impl IdentifiableAbstractionElement for FlexrayTpConnection {
    /// get the name of the connection
    ///
    /// In early versions of the Autosar standard, TpConnections were not identifiable.
    /// This was fixed later by adding the Ident sub-element. This method returns the name
    /// provied in the Ident element, if it exists.
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

impl FlexrayTpConnection {
    pub(crate) fn new(
        name: Option<&str>,
        parent: &Element,
        transmitter: &FlexrayTpNode,
        direct_tp_sdu: &IPdu,
        connection_control: &FlexrayTpConnectionControl,
    ) -> Result<Self, AutosarAbstractionError> {
        let tp_connection_elem = parent.create_sub_element(ElementName::FlexrayTpConnection)?;
        if let Some(name) = name {
            tp_connection_elem.create_named_sub_element(ElementName::Ident, name)?;
        }
        let tp_connection = Self(tp_connection_elem);
        tp_connection.set_transmitter(transmitter)?;
        tp_connection.set_direct_tp_sdu(direct_tp_sdu)?;
        tp_connection.set_connection_control(connection_control)?;

        Ok(tp_connection)
    }

    /// set the transmitter of the connection
    pub fn set_transmitter(&self, transmitter: &FlexrayTpNode) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::TransmitterRef)?
            .set_reference_target(transmitter.element())?;
        Ok(())
    }

    /// get the transmitter of the connection
    #[must_use]
    pub fn transmitter(&self) -> Option<FlexrayTpNode> {
        self.element()
            .get_sub_element(ElementName::TransmitterRef)
            .and_then(|elem| elem.get_reference_target().ok())
            .and_then(|elem| elem.try_into().ok())
    }

    /// set the direct TP SDU of the connection
    pub fn set_direct_tp_sdu<T: AbstractIpdu>(&self, direct_tp_sdu: &T) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::DirectTpSduRef)?
            .set_reference_target(direct_tp_sdu.element())?;
        Ok(())
    }

    /// get the direct TP SDU of the connection
    #[must_use]
    pub fn direct_tp_sdu(&self) -> Option<IPdu> {
        self.element()
            .get_sub_element(ElementName::DirectTpSduRef)
            .and_then(|elem| elem.get_reference_target().ok())
            .and_then(|elem| elem.try_into().ok())
    }

    /// set the connection control of the connection
    pub fn set_connection_control(
        &self,
        connection_control: &FlexrayTpConnectionControl,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::TpConnectionControlRef)?
            .set_reference_target(connection_control.element())?;
        Ok(())
    }

    /// get the connection control of the connection
    #[must_use]
    pub fn connection_control(&self) -> Option<FlexrayTpConnectionControl> {
        self.element()
            .get_sub_element(ElementName::TpConnectionControlRef)?
            .get_reference_target()
            .ok()?
            .try_into()
            .ok()
    }

    /// add a receiver to the connection
    pub fn add_receiver(&self, receiver: &FlexrayTpNode) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::ReceiverRefs)?
            .create_sub_element(ElementName::ReceiverRef)?
            .set_reference_target(receiver.element())?;
        Ok(())
    }

    /// iterate over all receivers of the connection
    pub fn receivers(&self) -> impl Iterator<Item = FlexrayTpNode> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::ReceiverRefs)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|ref_elem| {
                ref_elem
                    .get_reference_target()
                    .ok()
                    .and_then(|elem| elem.try_into().ok())
            })
    }

    /// set the reversed TP SDU of the connection
    /// This is used if the connection supports both sending and receiving
    pub fn set_reversed_tp_sdu<T: AbstractIpdu>(&self, reversed_tp_sdu: &T) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::ReversedTpSduRef)?
            .set_reference_target(reversed_tp_sdu.element())?;
        Ok(())
    }

    /// get the reversed TP SDU of the connection
    #[must_use]
    pub fn reversed_tp_sdu(&self) -> Option<IPdu> {
        self.element()
            .get_sub_element(ElementName::ReversedTpSduRef)
            .and_then(|elem| elem.get_reference_target().ok())
            .and_then(|elem| elem.try_into().ok())
    }

    /// set the TX `FlexrayTpPduPool` of the connection
    pub fn set_tx_pdu_pool(&self, tx_pdu_pool: &FlexrayTpPduPool) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::TxPduPoolRef)?
            .set_reference_target(tx_pdu_pool.element())?;
        Ok(())
    }

    /// get the TX `FlexrayTpPduPool` of the connection
    #[must_use]
    pub fn tx_pdu_pool(&self) -> Option<FlexrayTpPduPool> {
        self.element()
            .get_sub_element(ElementName::TxPduPoolRef)
            .and_then(|elem| elem.get_reference_target().ok())
            .and_then(|elem| elem.try_into().ok())
    }

    /// set the RX `FlexrayTpPduPool` of the connection
    pub fn set_rx_pdu_pool(&self, rx_pdu_pool: &FlexrayTpPduPool) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::RxPduPoolRef)?
            .set_reference_target(rx_pdu_pool.element())?;
        Ok(())
    }

    /// get the RX `FlexrayTpPduPool` of the connection
    #[must_use]
    pub fn rx_pdu_pool(&self) -> Option<FlexrayTpPduPool> {
        self.element()
            .get_sub_element(ElementName::RxPduPoolRef)
            .and_then(|elem| elem.get_reference_target().ok())
            .and_then(|elem| elem.try_into().ok())
    }

    /// set the multicast `TpAddress` of the connection
    /// This element is optional; setting None will remove the element
    pub fn set_multicast_address(&self, multicast_address: Option<&TpAddress>) -> Result<(), AutosarAbstractionError> {
        if let Some(multicast_address) = multicast_address {
            // add or update the multicast address
            self.element()
                .get_or_create_sub_element(ElementName::MulticastRef)?
                .set_reference_target(multicast_address.element())?;
        } else {
            // remove the multicast address
            let _ = self.element().remove_sub_element_kind(ElementName::MulticastRef);
        }
        Ok(())
    }

    /// get the multicast `TpAddress` of the connection
    #[must_use]
    pub fn multicast_address(&self) -> Option<TpAddress> {
        self.element()
            .get_sub_element(ElementName::MulticastRef)
            .and_then(|elem| elem.get_reference_target().ok())
            .and_then(|elem| elem.try_into().ok())
    }
}

//##################################################################

/// A `FlexrayTpConnectionControl` defines the connection control parameters for a `FlexrayTpConnection`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlexrayTpConnectionControl(Element);
abstraction_element!(FlexrayTpConnectionControl, FlexrayTpConnectionControl);
impl IdentifiableAbstractionElement for FlexrayTpConnectionControl {}

impl FlexrayTpConnectionControl {
    pub(crate) fn new(name: &str, parent: &Element) -> Result<Self, AutosarAbstractionError> {
        let connection_control_elem = parent.create_named_sub_element(ElementName::FlexrayTpConnectionControl, name)?;
        Ok(Self(connection_control_elem))
    }

    /// set the maxFcWait value
    pub fn set_max_fc_wait(&self, max_fc_wait: u32) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::MaxFcWait)?
            .set_character_data(u64::from(max_fc_wait))?;
        Ok(())
    }

    /// get the maxFcWait value
    #[must_use]
    pub fn max_fc_wait(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::MaxFcWait)?
            .character_data()?
            .parse_integer()
    }

    /// set the maxNumberOfNpduPerCycle value
    pub fn set_max_number_of_npdu_per_cycle(
        &self,
        max_number_of_npdu_per_cycle: u32,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::MaxNumberOfNpduPerCycle)?
            .set_character_data(u64::from(max_number_of_npdu_per_cycle))?;
        Ok(())
    }

    /// get the maxNumberOfNpduPerCycle value
    #[must_use]
    pub fn max_number_of_npdu_per_cycle(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::MaxNumberOfNpduPerCycle)?
            .character_data()?
            .parse_integer()
    }

    /// set the maxRetries value
    pub fn set_max_retries(&self, max_retries: u32) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::MaxRetries)?
            .set_character_data(u64::from(max_retries))?;
        Ok(())
    }

    /// get the maxRetries value
    #[must_use]
    pub fn max_retries(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::MaxRetries)?
            .character_data()?
            .parse_integer()
    }

    /// set the separationCycleExponent value
    pub fn set_separation_cycle_exponent(&self, separation_cycle_exponent: u32) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::SeparationCycleExponent)?
            .set_character_data(u64::from(separation_cycle_exponent))?;
        Ok(())
    }

    /// get the separationCycleExponent value
    #[must_use]
    pub fn separation_cycle_exponent(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::SeparationCycleExponent)?
            .character_data()?
            .parse_integer()
    }
}

//##################################################################

/// A `FlexrayTpEcu` represents an ECU within the `FlexrayTpConfig`
#[derive(Debug, Clone, PartialEq)]
pub struct FlexrayTpEcu(Element);
abstraction_element!(FlexrayTpEcu, FlexrayTpEcu);

impl FlexrayTpEcu {
    pub(crate) fn new(
        parent: &Element,
        ecu_instance: &EcuInstance,
        full_duplex_enabled: bool,
    ) -> Result<Self, AutosarAbstractionError> {
        let tp_ecu_elem = parent.create_sub_element(ElementName::FlexrayTpEcu)?;
        let tp_ecu = Self(tp_ecu_elem);

        tp_ecu.set_ecu_instance(ecu_instance)?;
        tp_ecu.set_full_duplex_enabled(full_duplex_enabled)?;

        Ok(tp_ecu)
    }

    /// set the ECU instance of the `FlexrayTpEcu`
    pub fn set_ecu_instance(&self, ecu_instance: &EcuInstance) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::EcuInstanceRef)?
            .set_reference_target(ecu_instance.element())?;
        Ok(())
    }

    /// get the ECU instance of the `FlexrayTpEcu`
    #[must_use]
    pub fn ecu_instance(&self) -> Option<EcuInstance> {
        self.element()
            .get_sub_element(ElementName::EcuInstanceRef)
            .and_then(|elem| elem.get_reference_target().ok())
            .and_then(|elem| EcuInstance::try_from(elem).ok())
    }

    /// set the full duplex enabled status of the `FlexrayTpEcu`
    pub fn set_full_duplex_enabled(&self, full_duplex_enabled: bool) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::FullDuplexEnabled)?
            .set_character_data(full_duplex_enabled)?;
        Ok(())
    }

    /// get the full duplex enabled status of the `FlexrayTpEcu`
    #[must_use]
    pub fn full_duplex_enabled(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::FullDuplexEnabled)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_bool())
    }

    /// set the cycle time of the TP main function in seconds
    pub fn set_cycle_time_main_function(
        &self,
        cycle_time_main_function: Option<f64>,
    ) -> Result<(), AutosarAbstractionError> {
        if let Some(cycle_time_main_function) = cycle_time_main_function {
            self.element()
                .get_or_create_sub_element(ElementName::CycleTimeMainFunction)?
                .set_character_data(cycle_time_main_function)?;
        } else {
            let _ = self
                .element()
                .remove_sub_element_kind(ElementName::CycleTimeMainFunction);
        }
        Ok(())
    }

    /// get the cycle time of the TP main function in seconds
    #[must_use]
    pub fn cycle_time_main_function(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::CycleTimeMainFunction)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_float())
    }

    /// set the cancellation status of the `FlexrayTpEcu`
    pub fn set_cancellation(&self, cancellation: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(cancellation) = cancellation {
            self.element()
                .get_or_create_sub_element(ElementName::Cancellation)?
                .set_character_data(cancellation)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::Cancellation);
        }
        Ok(())
    }

    /// get the cancellation status of the `FlexrayTpEcu`
    #[must_use]
    pub fn cancellation(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::Cancellation)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_bool())
    }
}

//##################################################################

/// A `FlexrayTpNode` provides the TP address and the connection to the topology description in a `FlexrayTpConfig`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlexrayTpNode(Element);
abstraction_element!(FlexrayTpNode, FlexrayTpNode);
impl IdentifiableAbstractionElement for FlexrayTpNode {}

impl FlexrayTpNode {
    pub(crate) fn new(name: &str, parent: &Element) -> Result<Self, AutosarAbstractionError> {
        let node_elem = parent.create_named_sub_element(ElementName::FlexrayTpNode, name)?;
        Ok(Self(node_elem))
    }

    /// set or remove `FlexrayTpAddress` of the node
    /// A TP address is mandatory for unicast nodes, but optional for multicast nodes
    /// Setting None will remove the element
    pub fn set_tp_address(&self, tp_address: Option<&TpAddress>) -> Result<(), AutosarAbstractionError> {
        if let Some(tp_address) = tp_address {
            // add or update the TP address
            self.element()
                .get_or_create_sub_element(ElementName::TpAddressRef)?
                .set_reference_target(tp_address.element())?;
        } else {
            // remove the TP address
            if let Some(tp_address_elem) = self.element().get_sub_element(ElementName::TpAddressRef) {
                self.element().remove_sub_element(tp_address_elem)?;
            }
        }
        Ok(())
    }

    /// get the `FlexrayTpAddress` of the node
    #[must_use]
    pub fn tp_address(&self) -> Option<TpAddress> {
        self.element()
            .get_sub_element(ElementName::TpAddressRef)
            .and_then(|elem| elem.get_reference_target().ok())
            .and_then(|elem| elem.try_into().ok())
    }

    /// add a `FlexrayCommunicationConnector` to the node
    /// The node can be associated with up to 2 connectors.
    /// In a system description this reference is mandatory.
    pub fn add_communication_connector(
        &self,
        connector: &FlexrayCommunicationConnector,
    ) -> Result<(), AutosarAbstractionError> {
        let connector_refs = self.element().get_or_create_sub_element(ElementName::ConnectorRefs)?;

        if connector_refs.sub_elements().count() >= 2 {
            return Err(AutosarAbstractionError::InvalidParameter(
                "A FlexrayTpNode can only have up to 2 connectors".to_string(),
            ));
        }
        connector_refs
            .create_sub_element(ElementName::ConnectorRef)?
            .set_reference_target(connector.element())?;
        Ok(())
    }

    /// iterate over all `FlexrayCommunicationConnectors` of the node
    pub fn communication_connectors(&self) -> impl Iterator<Item = FlexrayCommunicationConnector> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::ConnectorRefs)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|ref_elem| {
                ref_elem
                    .get_reference_target()
                    .ok()
                    .and_then(|elem| elem.try_into().ok())
            })
    }
}

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        AutosarModelAbstraction, SystemCategory,
        communication::{FlexrayChannelName, FlexrayClusterSettings},
    };
    use autosar_data::AutosarVersion;

    #[test]
    fn test_flexray_iso_transport_protocol() {
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
        // create a reversed TP SDU (DCM-I-PDU)
        let reversed_tp_sdu = system.create_dcm_ipdu("diag_rev", &package, 1024).unwrap();

        // create some NPdus
        let npdu1 = system.create_n_pdu("npdu1", &package, 64).unwrap();
        let npdu2 = system.create_n_pdu("npdu2", &package, 64).unwrap();

        // create a FlexrayTpConfig
        let fr_tp_config = system
            .create_flexray_tp_config("FrTpConfig", &package, &flexray_cluster)
            .unwrap();
        assert_eq!(fr_tp_config.cluster().unwrap(), flexray_cluster);

        // create a FlexrayTpPduPool
        let fr_tp_pdu_pool_tx = fr_tp_config.create_flexray_tp_pdu_pool("FrTpPduPool_Tx").unwrap();
        fr_tp_pdu_pool_tx.add_n_pdu(&npdu1).unwrap();
        assert_eq!(fr_tp_pdu_pool_tx.n_pdus().next(), Some(npdu1));
        let fr_tp_pdu_pool_rx = fr_tp_config.create_flexray_tp_pdu_pool("FrTpPduPool_Rx").unwrap();
        fr_tp_pdu_pool_rx.add_n_pdu(&npdu2).unwrap();
        assert_eq!(fr_tp_pdu_pool_rx.n_pdus().next(), Some(npdu2));

        assert!(fr_tp_config.flexray_tp_pdu_pools().count() == 2);

        // create a FlexrayTpAddress
        let tp_address_1 = fr_tp_config.create_tp_address("TpAddress1", 0x1234).unwrap();
        assert_eq!(fr_tp_config.tp_addresses().next(), Some(tp_address_1.clone()));
        assert_eq!(tp_address_1.address().unwrap(), 0x1234);
        let tp_address_2 = fr_tp_config.create_tp_address("TpAddress2", 0x5678).unwrap();
        assert_eq!(fr_tp_config.tp_addresses().count(), 2);

        // create a FlexrayTpNode
        let tp_node_1 = fr_tp_config.create_flexray_tp_node("TpNode1").unwrap();
        tp_node_1.set_tp_address(Some(&tp_address_1)).unwrap();
        assert_eq!(tp_node_1.tp_address().unwrap(), tp_address_1);
        tp_node_1.add_communication_connector(&connector).unwrap();
        assert_eq!(tp_node_1.communication_connectors().next(), Some(connector));
        let tp_node_2 = fr_tp_config.create_flexray_tp_node("TpNode2").unwrap();
        tp_node_2.set_tp_address(Some(&tp_address_2)).unwrap();

        assert_eq!(fr_tp_config.flexray_tp_nodes().count(), 2);
        assert_eq!(fr_tp_config.flexray_tp_nodes().next(), Some(tp_node_1.clone()));

        // create a FlexrayTpConnectionControl
        let connection_control = fr_tp_config
            .create_flexray_tp_connection_control("ConnectionControl")
            .unwrap();
        assert_eq!(fr_tp_config.flexray_tp_connection_controls().count(), 1);
        assert_eq!(
            fr_tp_config.flexray_tp_connection_controls().next().unwrap(),
            connection_control
        );
        connection_control.set_max_fc_wait(10).unwrap();
        assert_eq!(connection_control.max_fc_wait().unwrap(), 10);
        connection_control.set_max_number_of_npdu_per_cycle(5).unwrap();
        assert_eq!(connection_control.max_number_of_npdu_per_cycle().unwrap(), 5);
        connection_control.set_max_retries(3).unwrap();
        assert_eq!(connection_control.max_retries().unwrap(), 3);
        connection_control.set_separation_cycle_exponent(2).unwrap();
        assert_eq!(connection_control.separation_cycle_exponent().unwrap(), 2);

        // create a FlexrayTpConnection
        let connection = fr_tp_config
            .create_flexray_tp_connection(None, &tp_node_1, &tp_sdu, &connection_control)
            .unwrap();
        assert_eq!(fr_tp_config.flexray_tp_connections().count(), 1);
        assert_eq!(fr_tp_config.flexray_tp_connections().next().unwrap(), connection);

        connection.add_receiver(&tp_node_2).unwrap();
        connection.set_tx_pdu_pool(&fr_tp_pdu_pool_tx).unwrap();
        connection.set_rx_pdu_pool(&fr_tp_pdu_pool_rx).unwrap();
        connection.set_multicast_address(Some(&tp_address_2)).unwrap();
        connection.set_reversed_tp_sdu(&reversed_tp_sdu).unwrap();
        assert_eq!(connection.receivers().count(), 1);
        assert_eq!(connection.receivers().next(), Some(tp_node_2));
        assert_eq!(connection.tx_pdu_pool().unwrap(), fr_tp_pdu_pool_tx);
        assert_eq!(connection.rx_pdu_pool().unwrap(), fr_tp_pdu_pool_rx);
        assert_eq!(connection.multicast_address().unwrap(), tp_address_2);
        assert_eq!(connection.connection_control().unwrap(), connection_control);
        assert_eq!(connection.transmitter().unwrap(), tp_node_1);
        assert_eq!(connection.direct_tp_sdu().unwrap(), tp_sdu.clone().into());
        assert_eq!(connection.reversed_tp_sdu().unwrap(), reversed_tp_sdu.clone().into());

        assert_eq!(connection.name(), None);
        connection.set_name("Connection1").unwrap();
        assert_eq!(connection.name(), Some("Connection1".to_string()));

        // add a FlexrayTpEcu to the FlexrayTpConfig
        let fr_tp_ecu = fr_tp_config.create_flexray_tp_ecu(&ecu_instance, true).unwrap();
        fr_tp_ecu.set_cycle_time_main_function(Some(0.01)).unwrap();
        fr_tp_ecu.set_cancellation(Some(true)).unwrap();
        assert_eq!(fr_tp_config.flexray_tp_ecus().count(), 1);
        assert_eq!(fr_tp_config.flexray_tp_ecus().next().unwrap(), fr_tp_ecu);
        assert_eq!(fr_tp_ecu.ecu_instance().unwrap(), ecu_instance);
        assert!(fr_tp_ecu.full_duplex_enabled().unwrap());
        assert_eq!(fr_tp_ecu.cycle_time_main_function().unwrap(), 0.01);
        assert!(fr_tp_ecu.cancellation().unwrap());
    }
}
