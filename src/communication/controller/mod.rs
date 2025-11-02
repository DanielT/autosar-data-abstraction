use crate::{AbstractionElement, AutosarAbstractionError, EcuInstance, IdentifiableAbstractionElement};
use autosar_data::{AutosarDataError, Element, ElementName};

mod can;
mod ethernet;
mod flexray;
mod lin;

pub use can::*;
pub use ethernet::*;
pub use flexray::*;
pub use lin::*;

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
    /// The `CommunicationController` is a [`LinMaster`] CommunicationController
    LinMaster(LinMaster),
    /// The `CommunicationController` is a [`LinSlave`] CommunicationController
    LinSlave(LinSlave),
}

impl AbstractionElement for CommunicationController {
    fn element(&self) -> &autosar_data::Element {
        match self {
            CommunicationController::Can(ccc) => ccc.element(),
            CommunicationController::Ethernet(ecc) => ecc.element(),
            CommunicationController::Flexray(fcc) => fcc.element(),
            CommunicationController::LinMaster(lcc) => lcc.element(),
            CommunicationController::LinSlave(lcc) => lcc.element(),
        }
    }
}

impl TryFrom<Element> for CommunicationController {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::CanCommunicationController => Ok(Self::Can(CanCommunicationController::try_from(element)?)),
            ElementName::EthernetCommunicationController => {
                Ok(Self::Ethernet(EthernetCommunicationController::try_from(element)?))
            }
            ElementName::FlexrayCommunicationController => {
                Ok(Self::Flexray(FlexrayCommunicationController::try_from(element)?))
            }
            ElementName::LinMaster => Ok(Self::LinMaster(LinMaster::try_from(element)?)),
            ElementName::LinSlave => Ok(Self::LinSlave(LinSlave::try_from(element)?)),
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "CommunicationController".to_string(),
            }),
        }
    }
}

impl IdentifiableAbstractionElement for CommunicationController {}
impl AbstractCommunicationController for CommunicationController {}

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

impl From<LinMaster> for CommunicationController {
    fn from(value: LinMaster) -> Self {
        CommunicationController::LinMaster(value)
    }
}

impl From<LinSlave> for CommunicationController {
    fn from(value: LinSlave) -> Self {
        CommunicationController::LinSlave(value)
    }
}

//##################################################################

/// A trait for all communication controllers
pub trait AbstractCommunicationController: AbstractionElement {
    /// Get the `EcuInstance` that contains this `CommunicationController`
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::{*, communication::*};
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// # let package = model.get_or_create_package("/pkg1")?;
    /// # let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// # let ecu_instance = system.create_ecu_instance("ecu_name", &package)?;
    /// let can_controller = ecu_instance.create_can_communication_controller("CanCtrl")?;
    /// assert_eq!(can_controller.ecu_instance()?, ecu_instance);
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to get the ECU-INSTANCE
    fn ecu_instance(&self) -> Result<EcuInstance, AutosarAbstractionError> {
        // Note: it is always OK to unwrap the result of named_parent() because
        // the parent of a CommunicationController is always an EcuInstance
        // named_parent() can only return Ok(None) for an ArPackage
        self.element().named_parent()?.unwrap().try_into()
    }
}

//##################################################################

/// A trait for all communication connectors
pub trait AbstractCommunicationConnector: AbstractionElement {
    /// The controller type of the `CommunicationConnector`
    type CommunicationControllerType: AbstractCommunicationController;

    /// Get the `EcuInstance` that contains this `CommunicationConnector`
    fn ecu_instance(&self) -> Result<EcuInstance, AutosarAbstractionError> {
        // Note: it is always OK to unwrap the result of named_parent() because
        // the parent of a CommunicationConnector is always an EcuInstance
        // named_parent() can only return Ok(None) for an ArPackage
        self.element().named_parent()?.unwrap().try_into()
    }

    /// Get the controller of the `CommunicationConnector`
    fn controller(&self) -> Result<Self::CommunicationControllerType, AutosarAbstractionError>;
}

//##################################################################

/// wraps all different kinds of communication connector
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommunicationConnector {
    /// The `CommunicationConnector` is a [`CanCommunicationConnector`]
    Can(CanCommunicationConnector),
    /// The `CommunicationConnector` is an [`EthernetCommunicationConnector`]
    Ethernet(EthernetCommunicationConnector),
    /// The `CommunicationConnector` is a [`FlexrayCommunicationConnector`]
    Flexray(FlexrayCommunicationConnector),
    /// The `CommunicationConnector` is a [`LinCommunicationConnector`]
    Lin(LinCommunicationConnector),
}

impl AbstractionElement for CommunicationConnector {
    fn element(&self) -> &autosar_data::Element {
        match self {
            CommunicationConnector::Can(cc) => cc.element(),
            CommunicationConnector::Ethernet(ec) => ec.element(),
            CommunicationConnector::Flexray(fc) => fc.element(),
            CommunicationConnector::Lin(lc) => lc.element(),
        }
    }
}

impl TryFrom<Element> for CommunicationConnector {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::CanCommunicationConnector => Ok(Self::Can(CanCommunicationConnector::try_from(element)?)),
            ElementName::EthernetCommunicationConnector => {
                Ok(Self::Ethernet(EthernetCommunicationConnector::try_from(element)?))
            }
            ElementName::FlexrayCommunicationConnector => {
                Ok(Self::Flexray(FlexrayCommunicationConnector::try_from(element)?))
            }
            ElementName::LinCommunicationConnector => Ok(Self::Lin(LinCommunicationConnector::try_from(element)?)),
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "CommunicationConnector".to_string(),
            }),
        }
    }
}

impl IdentifiableAbstractionElement for CommunicationConnector {}

impl AbstractCommunicationConnector for CommunicationConnector {
    type CommunicationControllerType = CommunicationController;

    fn controller(&self) -> Result<Self::CommunicationControllerType, AutosarAbstractionError> {
        let Some(controller_ref) = self.element().get_sub_element(ElementName::CommControllerRef) else {
            return Err(AutosarAbstractionError::ModelError(AutosarDataError::ElementNotFound {
                target: ElementName::CommControllerRef,
                parent: self.element().element_name(),
            }));
        };
        let controller = controller_ref.get_reference_target()?;
        Self::CommunicationControllerType::try_from(controller)
    }
}

//##################################################################

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AutosarModelAbstraction, SystemCategory,
        communication::{FlexrayChannelName, FlexrayClusterSettings},
    };
    use autosar_data::AutosarVersion;

    #[test]
    fn test_communication_controller() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/test").unwrap();
        let system = package.create_system("System", SystemCategory::SystemExtract).unwrap();

        let ecu = system.create_ecu_instance("ecu", &package).unwrap();
        let can_ctrl = ecu.create_can_communication_controller("can").unwrap();
        let ethernet_ctrl = ecu.create_ethernet_communication_controller("ethernet", None).unwrap();
        let flexray_ctrl = ecu.create_flexray_communication_controller("flexray").unwrap();

        let can_cc: CommunicationController = can_ctrl.clone().into();
        let ethernet_cc: CommunicationController = ethernet_ctrl.clone().into();
        let flexray_cc: CommunicationController = flexray_ctrl.clone().into();

        let can_cluster = system.create_can_cluster("can_cluster", &package, None).unwrap();
        let ethernet_cluster = system.create_ethernet_cluster("ethernet_cluster", &package).unwrap();
        let flexray_cluster = system
            .create_flexray_cluster("flexray_cluster", &package, &FlexrayClusterSettings::default())
            .unwrap();

        let can_channel = can_cluster.create_physical_channel("can_channel").unwrap();
        let ethernet_channel = ethernet_cluster
            .create_physical_channel("ethernet_channel", None)
            .unwrap();
        let flexray_channel = flexray_cluster
            .create_physical_channel("flexray_channel", FlexrayChannelName::A)
            .unwrap();

        let can_connector = can_ctrl
            .connect_physical_channel("can_connector", &can_channel)
            .unwrap();
        let ethernet_connector = ethernet_ctrl
            .connect_physical_channel("ethernet_connector", &ethernet_channel)
            .unwrap();
        let flexray_connector = flexray_ctrl
            .connect_physical_channel("flexray_connector", &flexray_channel)
            .unwrap();

        let connector: CommunicationConnector = CommunicationConnector::Can(can_connector.clone());
        assert_eq!(connector.controller().unwrap(), can_cc);
        let connector = CommunicationConnector::Ethernet(ethernet_connector.clone());
        assert_eq!(connector.controller().unwrap(), ethernet_cc);
        let connector = CommunicationConnector::Flexray(flexray_connector.clone());
        assert_eq!(connector.controller().unwrap(), flexray_cc);

        assert_eq!(can_cc.element().item_name().unwrap(), "can");
        assert_eq!(ethernet_cc.element().item_name().unwrap(), "ethernet");
        assert_eq!(flexray_cc.element().item_name().unwrap(), "flexray");
    }
}
