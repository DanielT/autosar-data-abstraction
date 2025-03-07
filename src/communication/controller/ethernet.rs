use crate::communication::{
    AbstractCommunicationConnector, AbstractCommunicationController, EthernetPhysicalChannel, EthernetVlanInfo,
};
use crate::{
    AbstractionElement, AutosarAbstractionError, EcuInstance, IdentifiableAbstractionElement, abstraction_element,
};
use autosar_data::{AutosarDataError, AutosarModel, Element, ElementName, ElementsIterator, WeakElement};

/// An `EcuInstance` needs an `EthernetCommunicationController` in order to connect to an ethernet cluster.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EthernetCommunicationController(Element);
abstraction_element!(EthernetCommunicationController, EthernetCommunicationController);
impl IdentifiableAbstractionElement for EthernetCommunicationController {}

impl EthernetCommunicationController {
    // create an EthernetCommunicationController
    pub(crate) fn new(
        name: &str,
        ecu: &EcuInstance,
        mac_address: Option<String>,
    ) -> Result<Self, AutosarAbstractionError> {
        let commcontrollers = ecu.element().get_or_create_sub_element(ElementName::CommControllers)?;
        let ctrl = commcontrollers.create_named_sub_element(ElementName::EthernetCommunicationController, name)?;
        let ethccc = ctrl
            .create_sub_element(ElementName::EthernetCommunicationControllerVariants)?
            .create_sub_element(ElementName::EthernetCommunicationControllerConditional)?;
        if let Some(mac_address) = mac_address {
            // creating the mac address element fails if the supplied string has an invalid format
            let result = ethccc
                .create_sub_element(ElementName::MacUnicastAddress)
                .and_then(|mua| mua.set_character_data(mac_address));
            if let Err(mac_address_error) = result {
                let _ = commcontrollers.remove_sub_element(ctrl);
                return Err(mac_address_error.into());
            }
        }
        let coupling_port_name = format!("{name}_CouplingPort");
        let _ = ethccc
            .create_sub_element(ElementName::CouplingPorts)
            .and_then(|cps| cps.create_named_sub_element(ElementName::CouplingPort, &coupling_port_name));

        Ok(Self(ctrl))
    }

    /// return an iterator over the [`EthernetPhysicalChannel`]s connected to this controller
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// # let package = model.get_or_create_package("/pkg1")?;
    /// # let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// # let ecu_instance = system.create_ecu_instance("ecu_name", &package)?;
    /// let ethernet_controller = ecu_instance.create_ethernet_communication_controller("EthCtrl", None)?;
    /// # let cluster = system.create_ethernet_cluster("Cluster", &package)?;
    /// # let physical_channel = cluster.create_physical_channel("Channel", None)?;
    /// ethernet_controller.connect_physical_channel("connection", &physical_channel)?;
    /// for channel in ethernet_controller.connected_channels() {
    ///     // ...
    /// }
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the ECU-INSTANCE
    pub fn connected_channels(&self) -> impl Iterator<Item = EthernetPhysicalChannel> + Send + 'static {
        if let Ok(ecu) = self.ecu_instance().map(|ecuinstance| ecuinstance.element().clone()) {
            EthernetCtrlChannelsIterator::new(self, &ecu)
        } else {
            EthernetCtrlChannelsIterator {
                connector_iter: None,
                comm_controller: self.0.clone(),
                model: None,
            }
        }
    }

    /// Connect this [`EthernetCommunicationController`] inside an [`EcuInstance`] to an [`EthernetPhysicalChannel`] in the [`crate::System`]
    ///
    /// Creates an `EthernetCommunicationConnector` in the [`EcuInstance`] that contains this [`EthernetCommunicationController`].
    ///
    /// This function establishes the relationships:
    ///  - [`EthernetPhysicalChannel`] -> `EthernetCommunicationConnector`
    ///  - `EthernetCommunicationConnector` -> [`EthernetCommunicationController`]
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// # let package = model.get_or_create_package("/pkg1")?;
    /// # let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// # let ecu_instance = system.create_ecu_instance("ecu_name", &package)?;
    /// let ethernet_controller = ecu_instance.create_ethernet_communication_controller("EthCtrl", None)?;
    /// # let cluster = system.create_ethernet_cluster("Cluster", &package)?;
    /// # let physical_channel = cluster.create_physical_channel("Channel", None)?;
    /// ethernet_controller.connect_physical_channel("connection", &physical_channel)?;
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the ECU-INSTANCE
    pub fn connect_physical_channel(
        &self,
        connection_name: &str,
        eth_channel: &EthernetPhysicalChannel,
    ) -> Result<EthernetCommunicationConnector, AutosarAbstractionError> {
        let ecu: Element = self.0.named_parent()?.unwrap();
        let cluster_of_channel = eth_channel.cluster()?;

        // There can be multiple connectors referring to a single EthernetCommunicationController,
        // but all of these connectors must refer to different PhysicalChannels
        // (= VLANs) of the same EthernetCluster.
        for phys_channel in self.connected_channels() {
            if phys_channel == *eth_channel {
                return Err(AutosarAbstractionError::ItemAlreadyExists);
            }

            if phys_channel.cluster()? != cluster_of_channel {
                return Err(AutosarAbstractionError::InvalidParameter(
                    "The EthernetCommunicationController may only refer to different channels within the same cluster"
                        .to_string(),
                ));
            }
        }

        // create a new connector
        let connectors = ecu.get_or_create_sub_element(ElementName::Connectors)?;
        let connector = EthernetCommunicationConnector::new(connection_name, &connectors, self)?;

        // if the ethernet physical channel has a category (WIRED / WIRELESS / CANXL) then
        // set the category of the connector to the same value
        if let Some(category) = eth_channel
            .element()
            .get_sub_element(ElementName::Category)
            .and_then(|cat| cat.character_data())
            .and_then(|cdata| cdata.string_value())
        {
            let _ = connector
                .element()
                .create_sub_element(ElementName::Category)
                .and_then(|cat| cat.set_character_data(category));
        }

        // create a communication connector ref in the ethernet channel that refers to this connector
        let channel_connctor_refs = eth_channel
            .element()
            .get_or_create_sub_element(ElementName::CommConnectors)?;
        channel_connctor_refs
            .create_sub_element(ElementName::CommunicationConnectorRefConditional)
            .and_then(|ccrc| ccrc.create_sub_element(ElementName::CommunicationConnectorRef))
            .and_then(|ccr| ccr.set_reference_target(connector.element()))?;

        // if the PhysicalChannel has VLAN info AND if there is a coupling port in this CommunicationController
        // then the coupling port should link to the PhysicalChannel / VLAN
        if let Some(EthernetVlanInfo { .. }) = eth_channel.vlan_info() {
            if let Some(coupling_port) = self
                .0
                .get_sub_element(ElementName::EthernetCommunicationControllerVariants)
                .and_then(|eccv| eccv.get_sub_element(ElementName::EthernetCommunicationControllerConditional))
                .and_then(|eccc| eccc.get_sub_element(ElementName::CouplingPorts))
                .and_then(|cps| cps.get_sub_element(ElementName::CouplingPort))
            {
                coupling_port
                    .get_or_create_sub_element(ElementName::VlanMemberships)
                    .and_then(|vms| vms.create_sub_element(ElementName::VlanMembership))
                    .and_then(|vm| vm.create_sub_element(ElementName::VlanRef))
                    .and_then(|vr| vr.set_reference_target(eth_channel.element()))?;
            }
        }

        Ok(connector)
    }
}

impl AbstractCommunicationController for EthernetCommunicationController {}

//##################################################################

/// A connector between an [`EthernetCommunicationController`] in an ECU and an [`EthernetPhysicalChannel`]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EthernetCommunicationConnector(Element);
abstraction_element!(EthernetCommunicationConnector, EthernetCommunicationConnector);
impl IdentifiableAbstractionElement for EthernetCommunicationConnector {}

impl EthernetCommunicationConnector {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        controller: &EthernetCommunicationController,
    ) -> Result<Self, AutosarAbstractionError> {
        let connector = parent.create_named_sub_element(ElementName::EthernetCommunicationConnector, name)?;
        connector
            .create_sub_element(ElementName::CommControllerRef)
            .and_then(|refelem| refelem.set_reference_target(&controller.0))?;
        Ok(Self(connector))
    }
}

impl AbstractCommunicationConnector for EthernetCommunicationConnector {
    type CommunicationControllerType = EthernetCommunicationController;

    fn controller(&self) -> Result<Self::CommunicationControllerType, AutosarAbstractionError> {
        let controller = self
            .element()
            .get_sub_element(ElementName::CommControllerRef)
            .ok_or_else(|| {
                AutosarAbstractionError::ModelError(AutosarDataError::ElementNotFound {
                    target: ElementName::CommControllerRef,
                    parent: self.element().element_name(),
                })
            })?
            .get_reference_target()?;
        EthernetCommunicationController::try_from(controller)
    }
}

//##################################################################

#[doc(hidden)]
pub struct EthernetCtrlChannelsIterator {
    connector_iter: Option<ElementsIterator>,
    comm_controller: Element,
    model: Option<AutosarModel>,
}

impl EthernetCtrlChannelsIterator {
    fn new(controller: &EthernetCommunicationController, ecu: &Element) -> Self {
        let iter = ecu.get_sub_element(ElementName::Connectors).map(|c| c.sub_elements());
        let comm_controller = controller.element().clone();
        let model = comm_controller.model().ok();
        Self {
            connector_iter: iter,
            comm_controller,
            model,
        }
    }
}

impl Iterator for EthernetCtrlChannelsIterator {
    type Item = EthernetPhysicalChannel;

    fn next(&mut self) -> Option<Self::Item> {
        let model = self.model.as_ref()?;
        let connector_iter = self.connector_iter.as_mut()?;
        for connector in connector_iter.by_ref() {
            if connector.element_name() == ElementName::EthernetCommunicationConnector {
                if let Some(commcontroller_of_connector) = connector
                    .get_sub_element(ElementName::CommControllerRef)
                    .and_then(|ccr| ccr.get_reference_target().ok())
                {
                    if commcontroller_of_connector == self.comm_controller {
                        for ref_origin in model
                            .get_references_to(&connector.path().ok()?)
                            .iter()
                            .filter_map(WeakElement::upgrade)
                            .filter_map(|elem| elem.named_parent().ok().flatten())
                        {
                            // This assumes that each connector will only ever be referenced by at most one
                            // PhysicalChannel, which is true for well-formed files.
                            if ref_origin.element_name() == ElementName::EthernetPhysicalChannel {
                                return EthernetPhysicalChannel::try_from(ref_origin).ok();
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::{AutosarModelAbstraction, SystemCategory, communication::EthernetVlanInfo};
    use autosar_data::AutosarVersion;

    #[test]
    fn controller() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();
        let ecu = system.create_ecu_instance("ECU", &pkg).unwrap();

        // can't create a controller with an invalid MAC address
        let result = ecu.create_ethernet_communication_controller("Controller", Some("abcdef".to_string()));
        assert!(result.is_err());

        // create a controller
        let result = ecu.create_ethernet_communication_controller("Controller", Some("01:02:03:04:05:06".to_string()));
        let controller = result.unwrap();

        // create some physical channels
        let cluster = system.create_ethernet_cluster("EthCluster", &pkg).unwrap();
        let channel1 = cluster.create_physical_channel("C1", None).unwrap();
        let vlan_info = EthernetVlanInfo {
            vlan_name: "VLAN_1".to_string(),
            vlan_id: 1,
        };
        let channel2 = cluster.create_physical_channel("C2", Some(&vlan_info)).unwrap();

        // connect the controller to channel1
        let connector = controller
            .connect_physical_channel("connection_name1", &channel1)
            .unwrap();
        assert_eq!(connector.controller().unwrap(), controller);
        // can't connect to the same channel again
        let result = controller.connect_physical_channel("connection_name2", &channel1);
        assert!(result.is_err());
        // connect the controller to channel2
        let result = controller.connect_physical_channel("connection_name2", &channel2);
        assert!(result.is_ok());

        // create a different cluster and channel, then try to connect the controller to it
        let cluster2 = system.create_ethernet_cluster("EthCluster2", &pkg).unwrap();
        let channel3 = cluster2.create_physical_channel("C3", None).unwrap();
        let result = controller.connect_physical_channel("connection_name3", &channel3);
        // can't connect one ethernet controller to channels from different clusters
        assert!(result.is_err());

        let count = controller.connected_channels().count();
        assert_eq!(count, 2);

        // remove the controller and try to list its connected channels again
        let ctrl_parent = controller.element().parent().unwrap().unwrap();
        ctrl_parent.remove_sub_element(controller.element().clone()).unwrap();
        let count = controller.connected_channels().count();
        assert_eq!(count, 0);
    }

    #[test]
    fn connector() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();
        let ecu = system.create_ecu_instance("ECU", &pkg).unwrap();

        let controller = ecu
            .create_ethernet_communication_controller("Controller", None)
            .unwrap();
        assert_eq!(controller.ecu_instance().unwrap(), ecu);

        let cluster = system.create_ethernet_cluster("EthCluster", &pkg).unwrap();
        let channel = cluster.create_physical_channel("C1", None).unwrap();

        // create a connector
        let connector = controller
            .connect_physical_channel("connection_name", &channel)
            .unwrap();
        assert_eq!(connector.controller().unwrap(), controller);
        assert_eq!(connector.ecu_instance().unwrap(), ecu);

        // remove the connector and try to get the controller from it
        let conn_parent = connector.element().parent().unwrap().unwrap();
        conn_parent.remove_sub_element(connector.element().clone()).unwrap();
        let result = connector.controller();
        assert!(result.is_err());
    }
}
