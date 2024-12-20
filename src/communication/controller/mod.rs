use crate::{AbstractionElement, AutosarAbstractionError, EcuInstance};
use autosar_data::{Element, ElementName};

mod can;
mod ethernet;
mod flexray;

pub use can::*;
pub use ethernet::*;
pub use flexray::*;

//##################################################################

/// wraps all different kinds of communication controller
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CommunicationController {
    /// The `CommunicationController` is a [`CanCommunicationController`]
    Can(CanCommunicationController),
    /// The `CommunicationController` is an [`EthernetCommunicationController`]
    Ethernet(EthernetCommunicationController),
    /// The `CommunicationController` is a [`FlexrayCommunicationController`]
    Flexray(FlexrayCommunicationController),
}

impl AbstractionElement for CommunicationController {
    fn element(&self) -> &autosar_data::Element {
        match self {
            CommunicationController::Can(ccc) => ccc.element(),
            CommunicationController::Ethernet(ecc) => ecc.element(),
            CommunicationController::Flexray(fcc) => fcc.element(),
        }
    }
}

impl TryFrom<Element> for CommunicationController {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::CanCommunicationController => Ok(CanCommunicationController::try_from(element)?.into()),
            ElementName::EthernetCommunicationController => {
                Ok(EthernetCommunicationController::try_from(element)?.into())
            }
            ElementName::FlexrayCommunicationController => {
                Ok(FlexrayCommunicationController::try_from(element)?.into())
            }
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "CommunicationController".to_string(),
            }),
        }
    }
}

impl From<CanCommunicationController> for CommunicationController {
    fn from(value: CanCommunicationController) -> Self {
        CommunicationController::Can(value)
    }
}

impl From<EthernetCommunicationController> for CommunicationController {
    fn from(value: EthernetCommunicationController) -> Self {
        CommunicationController::Ethernet(value)
    }
}

impl From<FlexrayCommunicationController> for CommunicationController {
    fn from(value: FlexrayCommunicationController) -> Self {
        CommunicationController::Flexray(value)
    }
}

//##################################################################

/// A trait for all communication controllers
pub trait AbstractCommunicationController: AbstractionElement {
    /// Get the `EcuInstance` that contains this `CommunicationController`
    fn ecu_instance(&self) -> Result<EcuInstance, AutosarAbstractionError> {
        // Note: it is always OK to unwrap the result of named_parent() because
        // the parent of a CommunicationController is always an EcuInstance
        // named_parent() can only return Ok(None) for an ArPackage
        self.element().named_parent()?.unwrap().try_into()
    }
}

//##################################################################

/// A trait for all communication connectors
pub trait CommunicationConnector: AbstractionElement {
    /// The controller type of the `CommunicationConnector`
    type Controller;

    /// Get the `EcuInstance` that contains this `CommunicationConnector`
    fn ecu_instance(&self) -> Result<EcuInstance, AutosarAbstractionError> {
        // Note: it is always OK to unwrap the result of named_parent() because
        // the parent of a CommunicationConnector is always an EcuInstance
        // named_parent() can only return Ok(None) for an ArPackage
        self.element().named_parent()?.unwrap().try_into()
    }

    /// Get the controller of the `CommunicationConnector`
    fn controller(&self) -> Result<Self::Controller, AutosarAbstractionError>;
}

//##################################################################

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ArPackage, EcuInstance};
    use autosar_data::{AutosarModel, AutosarVersion};

    #[test]
    fn test_communication_controller() {
        let model = AutosarModel::new();
        let _file = model.create_file("test.arxml", AutosarVersion::LATEST).unwrap();
        let package = ArPackage::get_or_create(&model, "/test").unwrap();
        let ecu = EcuInstance::new("ecu", &package).unwrap();
        let can = CanCommunicationController::new("can", &ecu).unwrap();
        let ethernet = EthernetCommunicationController::new("ethernet", &ecu, None).unwrap();
        let flexray = FlexrayCommunicationController::new("flexray", &ecu).unwrap();

        let can_cc: CommunicationController = can.into();
        let ethernet_cc: CommunicationController = ethernet.into();
        let flexray_cc: CommunicationController = flexray.into();

        assert_eq!(can_cc.element().item_name().unwrap(), "can");
        assert_eq!(ethernet_cc.element().item_name().unwrap(), "ethernet");
        assert_eq!(flexray_cc.element().item_name().unwrap(), "flexray");
    }
}
