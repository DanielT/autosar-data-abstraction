use crate::{
    AbstractionElement, AutosarAbstractionError, EcuInstance, IdentifiableAbstractionElement, abstraction_element,
    communication::{AbstractCommunicationConnector, AbstractCommunicationController},
};
use autosar_data::{AutosarDataError, Element, ElementName};

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
}

impl AbstractCommunicationController for LinMaster {}

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
}

impl AbstractCommunicationController for LinSlave {}

//##################################################################

/// A connector between a [`LinMaster`] or [`LinSlave`] in an ECU and a [`LinPhysicalChannel`]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinCommunicationConnector(Element);
abstraction_element!(LinCommunicationConnector, LinCommunicationConnector);
impl IdentifiableAbstractionElement for LinCommunicationConnector {}

impl LinCommunicationConnector {}

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
