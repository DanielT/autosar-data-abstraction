use crate::communication::{AbstractCommunicationConnector, AbstractCommunicationController, FlexrayPhysicalChannel};
use crate::{
    AbstractionElement, AutosarAbstractionError, EcuInstance, IdentifiableAbstractionElement, abstraction_element,
};
use autosar_data::{AutosarDataError, AutosarModel, Element, ElementName, ElementsIterator, WeakElement};

/// An `EcuInstance` needs a `FlexrayCommunicationController` in order to connect to a Flexray cluster.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlexrayCommunicationController(Element);
abstraction_element!(FlexrayCommunicationController, FlexrayCommunicationController);
impl IdentifiableAbstractionElement for FlexrayCommunicationController {}

impl FlexrayCommunicationController {
    // create a new FlexrayCommunicationController - called by EcuInstance::create_flexray_communication_controller
    pub(crate) fn new(name: &str, ecu: &EcuInstance) -> Result<Self, AutosarAbstractionError> {
        let commcontrollers = ecu.element().get_or_create_sub_element(ElementName::CommControllers)?;
        let ctrl = commcontrollers.create_named_sub_element(ElementName::FlexrayCommunicationController, name)?;
        let _flxccc = ctrl
            .create_sub_element(ElementName::FlexrayCommunicationControllerVariants)?
            .create_sub_element(ElementName::FlexrayCommunicationControllerConditional)?;

        Ok(Self(ctrl))
    }

    /// remove this `FlexrayCommunicationController` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        // remove all the connectors using this controller
        let ecu_instance = self.ecu_instance()?;
        for connector in ecu_instance
            .element()
            .get_sub_element(ElementName::Connectors)
            .iter()
            .flat_map(|connectors| connectors.sub_elements())
            .filter_map(|conn| FlexrayCommunicationConnector::try_from(conn).ok())
        {
            if let Ok(controller_of_connector) = connector.controller()
                && controller_of_connector == self
            {
                connector.remove(deep)?;
            }
        }

        AbstractionElement::remove(self, deep)
    }

    /// return an iterator over the [`FlexrayPhysicalChannel`]s connected to this controller
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
    /// let flexray_controller = ecu_instance.create_flexray_communication_controller("FRCtrl")?;
    /// # let cluster = system.create_flexray_cluster("Cluster", &package, &FlexrayClusterSettings::default())?;
    /// # let physical_channel = cluster.create_physical_channel("Channel", FlexrayChannelName::A)?;
    /// flexray_controller.connect_physical_channel("connection", &physical_channel)?;
    /// for channel in flexray_controller.connected_channels() {
    ///     // ...
    /// }
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model
    pub fn connected_channels(&self) -> impl Iterator<Item = FlexrayPhysicalChannel> + Send + use<> {
        if let Ok(ecu) = self.ecu_instance().map(|ecuinstance| ecuinstance.element().clone()) {
            FlexrayCtrlChannelsIterator::new(self, &ecu)
        } else {
            FlexrayCtrlChannelsIterator {
                connector_iter: None,
                comm_controller: self.0.clone(),
                model: None,
            }
        }
    }

    /// Connect this [`FlexrayCommunicationController`] inside an [`EcuInstance`] to a [`FlexrayPhysicalChannel`] in the [`crate::System`]
    ///
    /// Creates a `FlexrayCommunicationConnector` in the [`EcuInstance`] that contains this [`FlexrayCommunicationController`].
    ///
    /// This function establishes the relationships:
    ///  - [`FlexrayPhysicalChannel`] -> `FlexrayCommunicationConnector`
    ///  - `FlexrayCommunicationConnector` -> [`FlexrayCommunicationController`]
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
    /// let flexray_controller = ecu_instance.create_flexray_communication_controller("FlxCtrl")?;
    /// # let cluster = system.create_flexray_cluster("Cluster", &package, &FlexrayClusterSettings::default())?;
    /// # let physical_channel = cluster.create_physical_channel("Channel", FlexrayChannelName::A)?;
    /// flexray_controller.connect_physical_channel("connection", &physical_channel)?;
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the ECU-INSTANCE
    pub fn connect_physical_channel(
        &self,
        connection_name: &str,
        flx_channel: &FlexrayPhysicalChannel,
    ) -> Result<FlexrayCommunicationConnector, AutosarAbstractionError> {
        let ecu = self.0.named_parent()?.unwrap();

        for existing_channel in self.connected_channels() {
            if existing_channel == *flx_channel {
                return Err(AutosarAbstractionError::ItemAlreadyExists);
            }
        }

        // create a new connector
        let connectors = ecu.get_or_create_sub_element(ElementName::Connectors)?;
        let connector = FlexrayCommunicationConnector::new(connection_name, &connectors, self)?;

        let channel_connctor_refs = flx_channel
            .element()
            .get_or_create_sub_element(ElementName::CommConnectors)?;
        channel_connctor_refs
            .create_sub_element(ElementName::CommunicationConnectorRefConditional)
            .and_then(|ccrc| ccrc.create_sub_element(ElementName::CommunicationConnectorRef))
            .and_then(|ccr| ccr.set_reference_target(connector.element()))?;

        Ok(connector)
    }
}

impl AbstractCommunicationController for FlexrayCommunicationController {}

//##################################################################

/// A connector between a [`FlexrayCommunicationController`] in an ECU and a [`FlexrayPhysicalChannel`]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlexrayCommunicationConnector(Element);
abstraction_element!(FlexrayCommunicationConnector, FlexrayCommunicationConnector);
impl IdentifiableAbstractionElement for FlexrayCommunicationConnector {}

impl FlexrayCommunicationConnector {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        controller: &FlexrayCommunicationController,
    ) -> Result<Self, AutosarAbstractionError> {
        let connector = parent.create_named_sub_element(ElementName::FlexrayCommunicationConnector, name)?;
        connector
            .create_sub_element(ElementName::CommControllerRef)
            .and_then(|refelem| refelem.set_reference_target(controller.element()))?;

        Ok(Self(connector))
    }
}

impl AbstractCommunicationConnector for FlexrayCommunicationConnector {
    type CommunicationControllerType = FlexrayCommunicationController;

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
        FlexrayCommunicationController::try_from(controller)
    }
}

//##################################################################

#[doc(hidden)]
pub struct FlexrayCtrlChannelsIterator {
    connector_iter: Option<ElementsIterator>,
    comm_controller: Element,
    model: Option<AutosarModel>,
}

impl FlexrayCtrlChannelsIterator {
    fn new(controller: &FlexrayCommunicationController, ecu: &Element) -> Self {
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

impl Iterator for FlexrayCtrlChannelsIterator {
    type Item = FlexrayPhysicalChannel;

    fn next(&mut self) -> Option<Self::Item> {
        let model = self.model.as_ref()?;
        let connector_iter = self.connector_iter.as_mut()?;
        for connector in connector_iter.by_ref() {
            if connector.element_name() == ElementName::FlexrayCommunicationConnector
                && let Some(commcontroller_of_connector) = connector
                    .get_sub_element(ElementName::CommControllerRef)
                    .and_then(|ccr| ccr.get_reference_target().ok())
                && commcontroller_of_connector == self.comm_controller
            {
                for ref_origin in model
                    .get_references_to(&connector.path().ok()?)
                    .iter()
                    .filter_map(WeakElement::upgrade)
                    .filter_map(|elem| elem.named_parent().ok().flatten())
                {
                    // This assumes that each connector will only ever be referenced by at most one
                    // PhysicalChannel, which is true for well-formed files.
                    if ref_origin.element_name() == ElementName::FlexrayPhysicalChannel {
                        return FlexrayPhysicalChannel::try_from(ref_origin).ok();
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
    use crate::{
        AutosarModelAbstraction, SystemCategory,
        communication::{FlexrayChannelName, FlexrayClusterSettings},
    };

    use super::*;
    use autosar_data::AutosarVersion;

    #[test]
    fn controller() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();
        let ecu = system.create_ecu_instance("ECU", &pkg).unwrap();

        // create a controller
        let result = ecu.create_flexray_communication_controller("Controller");
        let controller = result.unwrap();

        // create some physical channels
        let settings = FlexrayClusterSettings::default();
        let cluster = system.create_flexray_cluster("FlxCluster", &pkg, &settings).unwrap();
        let channel1 = cluster.create_physical_channel("C1", FlexrayChannelName::A).unwrap();

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
        let controller = ecu.create_flexray_communication_controller("Controller").unwrap();
        assert_eq!(controller.ecu_instance().unwrap(), ecu);

        // create some physical channels
        let settings = FlexrayClusterSettings::default();
        let cluster = system.create_flexray_cluster("FlxCluster", &pkg, &settings).unwrap();
        let channel1 = cluster.create_physical_channel("C1", FlexrayChannelName::A).unwrap();

        // connect the controller to channel1
        let connector = controller
            .connect_physical_channel("connection_name1", &channel1)
            .unwrap();
        assert_eq!(connector.controller().unwrap(), controller);
        assert_eq!(connector.ecu_instance().unwrap(), ecu);

        // remove the connector and try to get the controller again
        let conn_parent = connector.0.parent().unwrap().unwrap();
        conn_parent.remove_sub_element(connector.0.clone()).unwrap();
        let result = connector.controller();
        assert!(result.is_err());
    }

    #[test]
    fn remove_controller() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();
        let ecu = system.create_ecu_instance("ECU", &pkg).unwrap();
        // create a controller
        let controller = ecu.create_flexray_communication_controller("Controller").unwrap();
        let cluster = system
            .create_flexray_cluster("FlxCluster", &pkg, &FlexrayClusterSettings::default())
            .unwrap();
        let channel_a = cluster.create_physical_channel("C1", FlexrayChannelName::A).unwrap();
        let connector = controller
            .connect_physical_channel("connection_name1", &channel_a)
            .unwrap();

        // remove the controller (also removes the connector)
        controller.remove(true).unwrap();

        assert_eq!(ecu.communication_controllers().count(), 0);
        assert!(connector.element().path().is_err());
    }
}
