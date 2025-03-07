use crate::{
    AbstractionElement, AutosarAbstractionError, EcuInstance, IdentifiableAbstractionElement, abstraction_element,
    communication::{AbstractCommunicationConnector, AbstractCommunicationController, CanPhysicalChannel},
};
use autosar_data::{AutosarDataError, AutosarModel, Element, ElementName, ElementsIterator, WeakElement};

/// An `EcuInstance` needs a `CanCommunicationController` in order to connect to a CAN cluster.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanCommunicationController(Element);
abstraction_element!(CanCommunicationController, CanCommunicationController);
impl IdentifiableAbstractionElement for CanCommunicationController {}

impl CanCommunicationController {
    // create a new CanCommunicationController - called by EcuInstance::create_can_communication_controller
    pub(crate) fn new(name: &str, ecu: &EcuInstance) -> Result<Self, AutosarAbstractionError> {
        let commcontrollers = ecu.element().get_or_create_sub_element(ElementName::CommControllers)?;
        let ctrl = commcontrollers.create_named_sub_element(ElementName::CanCommunicationController, name)?;
        let _canccc = ctrl
            .create_sub_element(ElementName::CanCommunicationControllerVariants)?
            .create_sub_element(ElementName::CanCommunicationControllerConditional)?;

        Ok(Self(ctrl))
    }

    /// return an iterator over the [`CanPhysicalChannel`]s connected to this controller
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # use autosar_data_abstraction::communication::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// # let package = model.get_or_create_package("/pkg1")?;
    /// # let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// # let ecu_instance = system.create_ecu_instance("ecu_name", &package)?;
    /// let can_controller = ecu_instance.create_can_communication_controller("CanCtrl")?;
    /// # let cluster = system.create_can_cluster("Cluster", &package, None)?;
    /// # let physical_channel = cluster.create_physical_channel("Channel")?;
    /// can_controller.connect_physical_channel("connection", &physical_channel)?;
    /// for channel in can_controller.connected_channels() {
    ///     // ...
    /// }
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to get the ECU-INSTANCE
    pub fn connected_channels(&self) -> impl Iterator<Item = CanPhysicalChannel> + Send + 'static {
        if let Ok(ecu) = self.ecu_instance().map(|ecuinstance| ecuinstance.element().clone()) {
            CanCtrlChannelsIterator::new(self, &ecu)
        } else {
            CanCtrlChannelsIterator {
                connector_iter: None,
                comm_controller: self.0.clone(),
                model: None,
            }
        }
    }

    /// Connect this [`CanCommunicationController`] inside an [`EcuInstance`] to a [`CanPhysicalChannel`] in the [`crate::System`]
    ///
    /// Creates a [`CanCommunicationConnector`] in the [`EcuInstance`] that contains this [`CanCommunicationController`].
    ///
    /// This function establishes the relationships:
    ///  - [`CanPhysicalChannel`] -> [`CanCommunicationConnector`]
    ///  - [`CanCommunicationConnector`] -> [`CanCommunicationController`]
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # use autosar_data_abstraction::communication::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// # let package = model.get_or_create_package("/pkg1")?;
    /// # let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// # let ecu_instance = system.create_ecu_instance("ecu_name", &package)?;
    /// let can_controller = ecu_instance.create_can_communication_controller("CanCtrl")?;
    /// # let cluster = system.create_can_cluster("Cluster", &package, None)?;
    /// # let physical_channel = cluster.create_physical_channel("Channel")?;
    /// can_controller.connect_physical_channel("connection", &physical_channel)?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the ECU-INSTANCE
    pub fn connect_physical_channel(
        &self,
        connection_name: &str,
        can_channel: &CanPhysicalChannel,
    ) -> Result<CanCommunicationConnector, AutosarAbstractionError> {
        let ecu = self.0.named_parent()?.unwrap();
        // check that there is no existing connector for this CanCommunicationController
        if let Some(connectors) = ecu.get_sub_element(ElementName::Connectors) {
            for connector in connectors.sub_elements() {
                // Does the existing connector reference this CanCommunicationController?
                // A CanCommunicationController can only connect to a single CAN cluster, so a second
                // connector cannot be created.
                if let Some(ccref) = connector.get_sub_element(ElementName::CommControllerRef) {
                    if let Ok(commcontroller_of_connector) = ccref.get_reference_target() {
                        if commcontroller_of_connector == self.0 {
                            return Err(AutosarAbstractionError::ItemAlreadyExists);
                        }
                    }
                }
            }
        }
        // create a new connector
        let connectors = ecu.get_or_create_sub_element(ElementName::Connectors)?;
        let connector = CanCommunicationConnector::new(connection_name, &connectors, self)?;

        let channel_connctor_refs = can_channel
            .element()
            .get_or_create_sub_element(ElementName::CommConnectors)?;
        channel_connctor_refs
            .create_sub_element(ElementName::CommunicationConnectorRefConditional)
            .and_then(|ccrc| ccrc.create_sub_element(ElementName::CommunicationConnectorRef))
            .and_then(|ccr| ccr.set_reference_target(connector.element()))?;

        Ok(connector)
    }
}

impl AbstractCommunicationController for CanCommunicationController {}

//##################################################################

/// A connector between a [`CanCommunicationController`] in an ECU and a [`CanPhysicalChannel`]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanCommunicationConnector(Element);
abstraction_element!(CanCommunicationConnector, CanCommunicationConnector);
impl IdentifiableAbstractionElement for CanCommunicationConnector {}

impl CanCommunicationConnector {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        controller: &CanCommunicationController,
    ) -> Result<Self, AutosarAbstractionError> {
        let connector = parent.create_named_sub_element(ElementName::CanCommunicationConnector, name)?;
        connector
            .create_sub_element(ElementName::CommControllerRef)?
            .set_reference_target(controller.element())?;
        Ok(Self(connector))
    }
}

impl AbstractCommunicationConnector for CanCommunicationConnector {
    type CommunicationControllerType = CanCommunicationController;

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
        CanCommunicationController::try_from(controller)
    }
}

//##################################################################

#[doc(hidden)]
pub struct CanCtrlChannelsIterator {
    connector_iter: Option<ElementsIterator>,
    comm_controller: Element,
    model: Option<AutosarModel>,
}

impl CanCtrlChannelsIterator {
    fn new(controller: &CanCommunicationController, ecu: &Element) -> Self {
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

impl Iterator for CanCtrlChannelsIterator {
    type Item = CanPhysicalChannel;

    fn next(&mut self) -> Option<Self::Item> {
        let model = self.model.as_ref()?;
        let connector_iter = self.connector_iter.as_mut()?;
        for connector in connector_iter.by_ref() {
            if connector.element_name() == ElementName::CanCommunicationConnector {
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
                            if ref_origin.element_name() == ElementName::CanPhysicalChannel {
                                return CanPhysicalChannel::try_from(ref_origin).ok();
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
    use crate::{AutosarModelAbstraction, SystemCategory};
    use autosar_data::AutosarVersion;

    #[test]
    fn controller() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();
        let ecu = system.create_ecu_instance("ECU", &pkg).unwrap();

        // create a controller
        let result = ecu.create_can_communication_controller("Controller");
        let controller = result.unwrap();

        // create some physical channels
        let cluster = system.create_can_cluster("CanCluster", &pkg, None).unwrap();
        let channel1 = cluster.create_physical_channel("C1").unwrap();

        // connect the controller to channel1
        let connector = controller
            .connect_physical_channel("connection_name1", &channel1)
            .unwrap();
        assert_eq!(connector.controller().unwrap(), controller);
        // can't connect to the same channel again
        let result = controller.connect_physical_channel("connection_name2", &channel1);
        assert!(result.is_err());

        let count = controller.connected_channels().count();
        assert_eq!(count, 1);

        // remove the controller and try to list its connected channels again
        let ctrl_parent = controller.0.parent().unwrap().unwrap();
        ctrl_parent.remove_sub_element(controller.0.clone()).unwrap();
        let count = controller.connected_channels().count();
        assert_eq!(count, 0);
    }

    #[test]
    fn connector() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();
        let ecu = system.create_ecu_instance("ECU", &pkg).unwrap();

        // create a controller
        let controller = ecu.create_can_communication_controller("Controller").unwrap();
        assert_eq!(controller.ecu_instance().unwrap(), ecu);

        // create some physical channels
        let cluster = system.create_can_cluster("CanCluster", &pkg, None).unwrap();
        let channel1 = cluster.create_physical_channel("C1").unwrap();

        // connect the controller to channel1
        let connector = controller
            .connect_physical_channel("connection_name1", &channel1)
            .unwrap();
        assert_eq!(connector.controller().unwrap(), controller);
        assert_eq!(connector.ecu_instance().unwrap(), ecu);

        // remove the CommControllerRef from the connector and try to get the controller
        connector
            .element()
            .remove_sub_element_kind(ElementName::CommControllerRef)
            .unwrap();
        let result = connector.controller();
        assert!(result.is_err());
    }
}
