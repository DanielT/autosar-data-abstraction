use crate::communication::{
    CanCommunicationController, CommunicationController, EthernetCommunicationController,
    FlexrayCommunicationController,
};
use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element,
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
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the ECU-INSTANCE
    pub fn create_flexray_communication_controller(
        &self,
        name: &str,
    ) -> Result<FlexrayCommunicationController, AutosarAbstractionError> {
        FlexrayCommunicationController::new(name, self)
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
    pub fn communication_controllers(&self) -> impl Iterator<Item = CommunicationController> + Send + 'static {
        self.0
            .get_sub_element(ElementName::CommControllers)
            .into_iter()
            .flat_map(|cc| cc.sub_elements())
            .filter_map(|ccelem| CommunicationController::try_from(ccelem).ok())
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
