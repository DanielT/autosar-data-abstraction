use crate::{
    AbstractionElement, AutosarAbstractionError, EcuInstance, IdentifiableAbstractionElement, abstraction_element,
    communication::{AbstractCommunicationConnector, AbstractCommunicationController, LinPhysicalChannel},
};
use autosar_data::{AutosarDataError, AutosarModel, Element, ElementName, ElementsIterator, WeakElement};

//##################################################################

/// Common behaviour for LIN communication controllers
pub trait AbstractLinCommunicationController:
    AbstractCommunicationController + Into<LinCommunicationController>
{
    /// return an iterator over the [`LinPhysicalChannel`]s connected to this controller
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
    /// let lin_master = ecu_instance.create_lin_master_communication_controller("LinMaster")?;
    /// # let cluster = system.create_lin_cluster("Cluster", &package)?;
    /// # let physical_channel = cluster.create_physical_channel("Channel")?;
    /// lin_master.connect_physical_channel("connection", &physical_channel)?;
    /// for channel in lin_master.connected_channels() {
    ///     // ...
    /// }
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to get the ECU-INSTANCE
    fn connected_channels(&self) -> impl Iterator<Item = LinPhysicalChannel> + Send + use<Self> {
        if let Ok(ecu) = self.ecu_instance().map(|ecuinstance| ecuinstance.element().clone()) {
            LinCtrlChannelsIterator::new(&self.clone().into(), &ecu)
        } else {
            LinCtrlChannelsIterator {
                connector_iter: None,
                comm_controller: self.element().clone(),
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
    /// let lin_slave = ecu_instance.create_lin_slave_communication_controller("LinSlave")?;
    /// # let cluster = system.create_lin_cluster("Cluster", &package)?;
    /// # let physical_channel = cluster.create_physical_channel("Channel")?;
    /// lin_slave.connect_physical_channel("connection", &physical_channel)?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the ECU-INSTANCE
    fn connect_physical_channel(
        &self,
        connection_name: &str,
        lin_channel: &LinPhysicalChannel,
    ) -> Result<LinCommunicationConnector, AutosarAbstractionError> {
        let ecu = self.element().named_parent()?.unwrap();
        // check that there is no existing connector for this LinCommunicationController
        if let Some(connectors) = ecu.get_sub_element(ElementName::Connectors) {
            for connector in connectors.sub_elements() {
                // Does the existing connector reference this LinCommunicationController?
                // A LinCommunicationController can only connect to a single LIN cluster, so a second
                // connector cannot be created.
                if let Some(ccref) = connector.get_sub_element(ElementName::CommControllerRef)
                    && let Ok(commcontroller_of_connector) = ccref.get_reference_target()
                    && &commcontroller_of_connector == self.element()
                {
                    return Err(AutosarAbstractionError::ItemAlreadyExists);
                }
            }
        }
        // create a new connector
        let connectors = ecu.get_or_create_sub_element(ElementName::Connectors)?;
        let connector = LinCommunicationConnector::new(connection_name, &connectors, &self.clone().into())?;

        let channel_connector_refs = lin_channel
            .element()
            .get_or_create_sub_element(ElementName::CommConnectors)?;
        channel_connector_refs
            .create_sub_element(ElementName::CommunicationConnectorRefConditional)
            .and_then(|ccrc| ccrc.create_sub_element(ElementName::CommunicationConnectorRef))
            .and_then(|ccr| ccr.set_reference_target(connector.element()))?;

        Ok(connector)
    }
}

//##################################################################

/// An `EcuInstance` needs a `LinMaster` or `LinSlave` in order to connect to a LIN cluster.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinMaster(Element);
abstraction_element!(LinMaster, LinMaster);
impl IdentifiableAbstractionElement for LinMaster {}

impl LinMaster {
    // create a new LinMaster - called by EcuInstance::create_lin_master_communication_controller
    pub(crate) fn new(name: &str, ecu: &EcuInstance) -> Result<Self, AutosarAbstractionError> {
        let commcontrollers = ecu.element().get_or_create_sub_element(ElementName::CommControllers)?;
        let ctrl = commcontrollers.create_named_sub_element(ElementName::LinMaster, name)?;
        let _lincc = ctrl
            .create_sub_element(ElementName::LinMasterVariants)?
            .create_sub_element(ElementName::LinMasterConditional)?;

        Ok(Self(ctrl))
    }

    /// remove this `LinMaster` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        // remove all the connectors using this controller
        let ecu_instance = self.ecu_instance()?;
        for connector in ecu_instance
            .element()
            .get_sub_element(ElementName::Connectors)
            .iter()
            .flat_map(|connectors| connectors.sub_elements())
            .filter_map(|conn| LinCommunicationConnector::try_from(conn).ok())
        {
            if let Ok(controller_of_connector) = connector.controller()
                && controller_of_connector.element() == self.element()
            {
                connector.remove(deep)?;
            }
        }

        AbstractionElement::remove(self, deep)
    }
}

impl AbstractCommunicationController for LinMaster {}
impl AbstractLinCommunicationController for LinMaster {}

//##################################################################

/// An `EcuInstance` needs a `LinMaster` or `LinSlave` in order to connect to a LIN cluster.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinSlave(Element);
abstraction_element!(LinSlave, LinSlave);
impl IdentifiableAbstractionElement for LinSlave {}

impl LinSlave {
    // create a new LinSlave - called by EcuInstance::create_lin_slave_communication_controller
    pub(crate) fn new(name: &str, ecu: &EcuInstance) -> Result<Self, AutosarAbstractionError> {
        let commcontrollers = ecu.element().get_or_create_sub_element(ElementName::CommControllers)?;
        let ctrl = commcontrollers.create_named_sub_element(ElementName::LinSlave, name)?;
        let _linsc = ctrl
            .create_sub_element(ElementName::LinSlaveVariants)?
            .create_sub_element(ElementName::LinSlaveConditional)?;

        Ok(Self(ctrl))
    }

    /// remove this `LinSlave` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        // remove all the connectors using this controller
        let ecu_instance = self.ecu_instance()?;
        for connector in ecu_instance
            .element()
            .get_sub_element(ElementName::Connectors)
            .iter()
            .flat_map(|connectors| connectors.sub_elements())
            .filter_map(|conn| LinCommunicationConnector::try_from(conn).ok())
        {
            if let Ok(controller_of_connector) = connector.controller()
                && controller_of_connector.element() == self.element()
            {
                connector.remove(deep)?;
            }
        }

        AbstractionElement::remove(self, deep)
    }
}

impl AbstractCommunicationController for LinSlave {}
impl AbstractLinCommunicationController for LinSlave {}

//##################################################################

/// A connector between a [`LinMaster`] or [`LinSlave`] in an ECU and a [`LinPhysicalChannel`]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinCommunicationConnector(Element);
abstraction_element!(LinCommunicationConnector, LinCommunicationConnector);
impl IdentifiableAbstractionElement for LinCommunicationConnector {}

impl LinCommunicationConnector {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        controller: &LinCommunicationController,
    ) -> Result<Self, AutosarAbstractionError> {
        let connector = parent.create_named_sub_element(ElementName::LinCommunicationConnector, name)?;
        connector
            .create_sub_element(ElementName::CommControllerRef)?
            .set_reference_target(controller.element())?;
        Ok(Self(connector))
    }
}

impl AbstractCommunicationConnector for LinCommunicationConnector {
    type CommunicationControllerType = LinCommunicationController;

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
        LinCommunicationController::try_from(controller)
    }
}

//##################################################################

/// A LIN communication controller is either a [`LinMaster`] or a [`LinSlave`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LinCommunicationController {
    /// A LIN Master communication controller
    Master(LinMaster),
    /// A LIN Slave communication controller
    Slave(LinSlave),
}
impl AbstractionElement for LinCommunicationController {
    fn element(&self) -> &autosar_data::Element {
        match self {
            LinCommunicationController::Master(master) => master.element(),
            LinCommunicationController::Slave(slave) => slave.element(),
        }
    }
}
impl AbstractCommunicationController for LinCommunicationController {}

impl TryFrom<Element> for LinCommunicationController {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::LinMaster => Ok(Self::Master(LinMaster::try_from(element)?)),
            ElementName::LinSlave => Ok(Self::Slave(LinSlave::try_from(element)?)),
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "LinCommunicationController".to_string(),
            }),
        }
    }
}

impl From<LinMaster> for LinCommunicationController {
    fn from(value: LinMaster) -> Self {
        LinCommunicationController::Master(value)
    }
}

impl From<LinSlave> for LinCommunicationController {
    fn from(value: LinSlave) -> Self {
        LinCommunicationController::Slave(value)
    }
}

//##################################################################

#[doc(hidden)]
pub struct LinCtrlChannelsIterator {
    connector_iter: Option<ElementsIterator>,
    comm_controller: Element,
    model: Option<AutosarModel>,
}

impl LinCtrlChannelsIterator {
    fn new(controller: &LinCommunicationController, ecu: &Element) -> Self {
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

impl Iterator for LinCtrlChannelsIterator {
    type Item = LinPhysicalChannel;

    fn next(&mut self) -> Option<Self::Item> {
        let model = self.model.as_ref()?;
        let connector_iter = self.connector_iter.as_mut()?;
        for connector in connector_iter.by_ref() {
            if connector.element_name() == ElementName::LinCommunicationConnector
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
                    if ref_origin.element_name() == ElementName::LinPhysicalChannel {
                        return LinPhysicalChannel::try_from(ref_origin).ok();
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
    use autosar_data::AutosarVersion;

    use crate::{AutosarModelAbstraction, SystemCategory};

    use super::*;

    #[test]
    fn controller() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();
        let ecu = system.create_ecu_instance("ECU", &pkg).unwrap();

        // create a LIN master and a LIN slave communication controller
        let result = ecu.create_lin_master_communication_controller("LinMaster");
        let lin_master = result.unwrap();
        let result = ecu.create_lin_slave_communication_controller("LinSlave");
        let lin_slave = result.unwrap();

        // create some physical channels
        let cluster = system.create_lin_cluster("LinCluster", &pkg).unwrap();
        let channel = cluster.create_physical_channel("C1").unwrap();

        // connect the controllers to the channel
        let connector_m = lin_master
            .connect_physical_channel("master_connection", &channel)
            .unwrap();
        assert_eq!(connector_m.controller().unwrap(), lin_master.clone().into());
        let connector_s = lin_slave
            .connect_physical_channel("slave_connection", &channel)
            .unwrap();
        assert_eq!(connector_s.controller().unwrap(), lin_slave.clone().into());
        // can't connect to the same channel again
        let result = lin_master.connect_physical_channel("connection_name2", &channel);
        assert!(result.is_err());

        let count = lin_master.connected_channels().count();
        assert_eq!(count, 1);

        // remove the controller and try to list its connected channels again
        let ctrl_parent = lin_master.0.parent().unwrap().unwrap();
        ctrl_parent.remove_sub_element(lin_master.0.clone()).unwrap();
        let count = lin_master.connected_channels().count();
        assert_eq!(count, 0);
    }

    #[test]
    fn connector() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();
        let ecu = system.create_ecu_instance("ECU", &pkg).unwrap();

        // create a controller
        let lin_master = ecu.create_lin_master_communication_controller("Controller").unwrap();
        assert_eq!(lin_master.ecu_instance().unwrap(), ecu);

        // create some physical channels
        let cluster = system.create_lin_cluster("LinCluster", &pkg).unwrap();
        let channel = cluster.create_physical_channel("C1").unwrap();

        // connect the controller to channel1
        let connector = lin_master
            .connect_physical_channel("connection_name1", &channel)
            .unwrap();
        assert_eq!(connector.controller().unwrap(), lin_master.clone().into());
        assert_eq!(connector.ecu_instance().unwrap(), ecu);

        // remove the CommControllerRef from the connector and try to get the controller
        connector
            .element()
            .remove_sub_element_kind(ElementName::CommControllerRef)
            .unwrap();
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
        let lin_master = ecu.create_lin_master_communication_controller("Controller").unwrap();
        // create a lin cluster with a physical channel
        let cluster = system.create_lin_cluster("LinCluster", &pkg).unwrap();
        let channel = cluster.create_physical_channel("C1").unwrap();
        // connect the controller to the channel
        let connector = lin_master
            .connect_physical_channel("connection_name1", &channel)
            .unwrap();

        // remove the controller, which should also remove the connector
        lin_master.remove(true).unwrap();

        assert_eq!(ecu.communication_controllers().count(), 0);
        assert!(connector.element().path().is_err());
    }
}
