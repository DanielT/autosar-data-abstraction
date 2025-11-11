use crate::communication::{
    CanCommunicationController, CanTpEcu, CommunicationController, EthernetCommunicationController,
    FlexrayCommunicationController, FlexrayTpEcu, ISignalIPduGroup, LinMaster, LinSlave, NmEcu,
};
use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element,
    get_reference_parents,
};
use autosar_data::{Element, ElementName};

/// The `EcuInstance` represents one ECU in a `System`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcuInstance(Element);
abstraction_element!(EcuInstance, EcuInstance);
impl IdentifiableAbstractionElement for EcuInstance {}

impl EcuInstance {
    // Create a new EcuInstance
    //
    // This new EcuInstance will not be connected to a System.
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<EcuInstance, AutosarAbstractionError> {
        let elem_pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_ecu_instance = elem_pkg_elements.create_named_sub_element(ElementName::EcuInstance, name)?;

        Ok(EcuInstance(elem_ecu_instance))
    }

    /// remove this `EcuInstance` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        // remove all communication controllers of this ECU
        for controller in self.communication_controllers() {
            controller.remove(deep)?;
        }

        let ref_parents = get_reference_parents(&self.0)?;

        AbstractionElement::remove(self, deep)?;

        for (_named_parent, parent) in ref_parents {
            match parent.element_name() {
                ElementName::NmEcu => {
                    if let Ok(nm_ecu) = NmEcu::try_from(parent) {
                        nm_ecu.remove(deep)?;
                    };
                }
                ElementName::CanTpEcu => {
                    if let Ok(can_tp_ecu) = CanTpEcu::try_from(parent) {
                        can_tp_ecu.remove(deep)?;
                    };
                }
                ElementName::FlexrayTpEcu => {
                    if let Ok(frtp_ecu) = FlexrayTpEcu::try_from(parent) {
                        frtp_ecu.remove(deep)?;
                    };
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Create a CAN-COMMUNICATION-CONTROLLER for this ECU-INSTANCE
    ///
    /// The ECU must have one controller per bus it communicates on.
    /// For example, if it communicates on two CAN buses, then two CAN-COMMUNICATION-CONTROLLERs are needed.
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
    /// let ecu_instance = system.create_ecu_instance("ecu_name", &package)?;
    /// let can_controller = ecu_instance.create_can_communication_controller("CanCtrl")?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the ECU-INSTANCE
    pub fn create_can_communication_controller(
        &self,
        name: &str,
    ) -> Result<CanCommunicationController, AutosarAbstractionError> {
        CanCommunicationController::new(name, self)
    }

    /// Create an ETHERNET-COMMUNICATION-CONTROLLER for this ECU-INSTANCE
    ///
    /// The ECU must have one controller per bus it communicates on.
    /// For example, if it communicates on two CAN buses, then two CAN-COMMUNICATION-CONTROLLERs are needed.
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
    /// let ecu_instance = system.create_ecu_instance("ecu_name", &package)?;
    /// let ethernet_controller = ecu_instance
    ///     .create_ethernet_communication_controller("EthCtrl", Some("ab:cd:ef:01:02:03".to_string()))?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the ECU-INSTANCE
    pub fn create_ethernet_communication_controller(
        &self,
        name: &str,
        mac_address: Option<String>,
    ) -> Result<EthernetCommunicationController, AutosarAbstractionError> {
        EthernetCommunicationController::new(name, self, mac_address)
    }

    /// Create a FLEXRAY-COMMUNICATION-CONTROLLER for this ECU-INSTANCE
    ///
    /// The ECU must have one controller per bus it communicates on.
    /// For example, if it communicates on two CAN buses, then two CAN-COMMUNICATION-CONTROLLERs are needed.
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
    /// let ecu_instance = system.create_ecu_instance("ecu_name", &package)?;
    /// let flexray_controller = ecu_instance
    ///     .create_flexray_communication_controller("FlexrayCtrl")?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to
    ///   create the FLEXRAY-COMMUNICATION-CONTROLLER
    pub fn create_flexray_communication_controller(
        &self,
        name: &str,
    ) -> Result<FlexrayCommunicationController, AutosarAbstractionError> {
        FlexrayCommunicationController::new(name, self)
    }

    /// Create a LIN-MASTER communication controller for this ECU-INSTANCE
    ///
    /// The ECU must have one controller per bus it communicates on.
    /// For example, if it communicates on two LIN buses, then two LIN-MASTER or
    /// LIN-SLAVE communication controllers are needed.
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
    /// let ecu_instance = system.create_ecu_instance("ecu_name", &package)?;
    /// let can_controller = ecu_instance.create_lin_master_communication_controller("LinMasterCtrl")?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the LIN-MASTER
    pub fn create_lin_master_communication_controller(&self, name: &str) -> Result<LinMaster, AutosarAbstractionError> {
        LinMaster::new(name, self)
    }

    /// Create a LIN-SLAVE communication controller for this ECU-INSTANCE
    ///
    /// The ECU must have one controller per bus it communicates on.
    /// For example, if it communicates on two LIN buses, then two LIN-MASTER or
    /// LIN-SLAVE communication controllers are needed.
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
    /// let ecu_instance = system.create_ecu_instance("ecu_name", &package)?;
    /// let can_controller = ecu_instance.create_lin_slave_communication_controller("LinSlaveCtrl")?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the LIN-MASTER
    pub fn create_lin_slave_communication_controller(&self, name: &str) -> Result<LinSlave, AutosarAbstractionError> {
        LinSlave::new(name, self)
    }

    /// return an interator over all communication controllers in this `EcuInstance`
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
    /// let ecu_instance = system.create_ecu_instance("ecu_name", &package)?;
    /// ecu_instance.create_flexray_communication_controller("FlexrayCtrl")?;
    /// ecu_instance.create_can_communication_controller("CanCtrl")?;
    /// for ctrl in ecu_instance.communication_controllers() {
    ///     // ...
    /// }
    /// # assert_eq!(ecu_instance.communication_controllers().count(), 2);
    /// # Ok(())}
    /// ```
    pub fn communication_controllers(&self) -> impl Iterator<Item = CommunicationController> + Send + use<> {
        self.0
            .get_sub_element(ElementName::CommControllers)
            .into_iter()
            .flat_map(|cc| cc.sub_elements())
            .filter_map(|ccelem| CommunicationController::try_from(ccelem).ok())
    }

    /// Add a reference to an associated COM IPdu group
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
    /// # let group = system.create_isignal_ipdu_group("PduGroup", &package, CommunicationDirection::In)?;
    /// let ecu_instance = system.create_ecu_instance("ecu_name", &package)?;
    /// ecu_instance.add_associated_com_ipdu_group(&group)?;
    /// for group in ecu_instance.associated_com_ipdu_groups() {
    ///     // ...
    /// }
    /// # assert_eq!(ecu_instance.associated_com_ipdu_groups().count(), 1);
    /// # Ok(())}
    /// ```
    pub fn add_associated_com_ipdu_group(
        &self,
        group: &ISignalIPduGroup,
    ) -> Result<(), AutosarAbstractionError> {
        self.0
            .get_or_create_sub_element(ElementName::AssociatedComIPduGroupRefs)?
            .create_sub_element(ElementName::AssociatedComIPduGroupRef)?
            .set_reference_target(group.element())?;
        Ok(())
    }

    /// Return an iterator over all associated COM IPdu groups
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
    /// # let group = system.create_isignal_ipdu_group("PduGroup", &package, CommunicationDirection::In)?;
    /// let ecu_instance = system.create_ecu_instance("ecu_name", &package)?;
    /// # ecu_instance.add_associated_com_ipdu_group(&group)?;
    /// for group in ecu_instance.associated_com_ipdu_groups() {
    ///     // ...
    /// }
    /// # assert_eq!(ecu_instance.associated_com_ipdu_groups().count(), 1);
    /// # Ok(())}
    /// ```
    pub fn associated_com_ipdu_groups(&self) -> impl Iterator<Item = ISignalIPduGroup> + Send + use<> {
        self.0
            .get_sub_element(ElementName::AssociatedComIPduGroupRefs)
            .into_iter()
            .flat_map(|acigr| acigr.sub_elements())
            .filter_map(|acigrf| {
                acigrf
                    .get_reference_target()
                    .ok()
                    .and_then(|elem| ISignalIPduGroup::try_from(elem).ok())
            })
    }
}

//##################################################################

#[cfg(test)]
mod test {
    use crate::*;
    use autosar_data::AutosarVersion;

    #[test]
    fn ecu() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let package = model.get_or_create_package("/pkg1").unwrap();
        let system = package.create_system("System", SystemCategory::SystemExtract).unwrap();
        let ecu_instance = system.create_ecu_instance("ecu_name", &package).unwrap();
        ecu_instance
            .create_flexray_communication_controller("FlexrayCtrl")
            .unwrap();
        ecu_instance.create_can_communication_controller("CanCtrl").unwrap();
        ecu_instance
            .create_ethernet_communication_controller("EthCtrl", None)
            .unwrap();
        assert_eq!(ecu_instance.communication_controllers().count(), 3);
    }
}
