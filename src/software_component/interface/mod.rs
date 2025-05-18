use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, Element, IdentifiableAbstractionElement,
    abstraction_element,
    datatype::{AbstractAutosarDataType, ValueSpecification},
    software_component::ModeDeclarationGroup,
};
use autosar_data::ElementName;

mod clientserver;
mod senderreceiver;

pub use clientserver::*;
pub use senderreceiver::*;

//##################################################################

/// A `ModeSwitchInterface` defines a set of modes that can be switched
///
/// Use [`ArPackage::create_mode_switch_interface`] to create a new mode switch interface
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModeSwitchInterface(Element);
abstraction_element!(ModeSwitchInterface, ModeSwitchInterface);
impl IdentifiableAbstractionElement for ModeSwitchInterface {}
impl AbstractPortInterface for ModeSwitchInterface {}

impl ModeSwitchInterface {
    /// Create a new `ModeSwitchInterface`
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let mode_switch_interface = elements.create_named_sub_element(ElementName::ModeSwitchInterface, name)?;

        Ok(Self(mode_switch_interface))
    }

    /// Create a mode group in this `ModeSwitchInterface`
    ///
    /// The `ModeSwitchInterface` can contain one mode group
    pub fn create_mode_group(
        &self,
        name: &str,
        mode_declaration_group: &ModeDeclarationGroup,
    ) -> Result<ModeGroup, AutosarAbstractionError> {
        ModeGroup::new(name, &self.element(), mode_declaration_group)
    }

    /// Get the mode group for this `ModeSwitchInterface`
    #[must_use]
    pub fn mode_group(&self) -> Option<ModeGroup> {
        let mode_group_elem = self.element().get_sub_element(ElementName::ModeGroup)?;
        ModeGroup::try_from(mode_group_elem).ok()
    }
}

//##################################################################

/// A `ModeGroup` represents a mode group in a `ModeSwitchInterface`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModeGroup(Element);
abstraction_element!(ModeGroup, ModeGroup);
impl IdentifiableAbstractionElement for ModeGroup {}

impl ModeGroup {
    /// Create a new `ModeGroup`
    fn new(
        name: &str,
        parent_element: &Element,
        mode_declaration_group: &ModeDeclarationGroup,
    ) -> Result<Self, AutosarAbstractionError> {
        let mode_group_elem = parent_element.create_named_sub_element(ElementName::ModeGroup, name)?;
        let mode_group = Self(mode_group_elem);
        mode_group.set_mode_declaration_group(mode_declaration_group)?;

        Ok(mode_group)
    }

    /// Set the mode declaration group for this `ModeGroup`
    pub fn set_mode_declaration_group(
        &self,
        mode_declaration_group: &ModeDeclarationGroup,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::TypeTref)?
            .set_reference_target(mode_declaration_group.element())?;
        Ok(())
    }

    /// Get the mode declaration group for this `ModeGroup`
    #[must_use]
    pub fn mode_declaration_group(&self) -> Option<ModeDeclarationGroup> {
        let mode_declaration_group_elem = self
            .element()
            .get_sub_element(ElementName::TypeTref)?
            .get_reference_target()
            .ok()?;
        ModeDeclarationGroup::try_from(mode_declaration_group_elem).ok()
    }
}

//##################################################################

/// A `ParameterInterface` defines a set of parameters that can be accessed
///
/// Use [`ArPackage::create_parameter_interface`] to create a new parameter interface
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParameterInterface(Element);
abstraction_element!(ParameterInterface, ParameterInterface);
impl IdentifiableAbstractionElement for ParameterInterface {}
impl AbstractPortInterface for ParameterInterface {}

impl ParameterInterface {
    /// Create a new `ParameterInterface`
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let parameter_interface = elements.create_named_sub_element(ElementName::ParameterInterface, name)?;

        Ok(Self(parameter_interface))
    }

    /// Create a new `ParameterDataPrototype` in this `ParameterInterface`
    pub fn create_parameter<T: AbstractAutosarDataType>(
        &self,
        name: &str,
        data_type: &T,
    ) -> Result<ParameterDataPrototype, AutosarAbstractionError> {
        let parameters = self.element().get_or_create_sub_element(ElementName::Parameters)?;
        ParameterDataPrototype::new(name, &parameters, data_type.element())
    }

    /// iterate over all `ParameterDataPrototype` in this `ParameterInterface`
    pub fn parameters(&self) -> impl Iterator<Item = ParameterDataPrototype> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::Parameters)
            .into_iter()
            .flat_map(|parameters| parameters.sub_elements())
            .filter_map(|param| ParameterDataPrototype::try_from(param).ok())
    }
}

//##################################################################

/// A `ParameterDataPrototype` defines a read-only parameter.
///
/// Typically such a parameter can be calibrated, but this is not required.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParameterDataPrototype(Element);
abstraction_element!(ParameterDataPrototype, ParameterDataPrototype);
impl IdentifiableAbstractionElement for ParameterDataPrototype {}

impl ParameterDataPrototype {
    /// Create a new `ParameterDataPrototype`
    fn new(name: &str, parent_element: &Element, data_type: &Element) -> Result<Self, AutosarAbstractionError> {
        let pdp = parent_element.create_named_sub_element(ElementName::ParameterDataPrototype, name)?;
        pdp.create_sub_element(ElementName::TypeTref)?
            .set_reference_target(data_type)?;

        Ok(Self(pdp))
    }

    /// set the init value for this signal
    pub fn set_init_value<T: Into<ValueSpecification>>(&self, value_spec: T) -> Result<(), AutosarAbstractionError> {
        let value_spec: ValueSpecification = value_spec.into();
        let init_value_elem = self.element().get_or_create_sub_element(ElementName::InitValue)?;
        value_spec.store(&init_value_elem)?;
        Ok(())
    }

    /// get the init value for this signal
    #[must_use]
    pub fn init_value(&self) -> Option<ValueSpecification> {
        let init_value_elem = self
            .element()
            .get_sub_element(ElementName::InitValue)?
            .get_sub_element_at(0)?;
        ValueSpecification::load(&init_value_elem)
    }
}

//##################################################################

/// An `NvDataInterface` defines non-volatile data that can be accessed through the interface
///
/// Use [`ArPackage::create_nv_data_interface`] to create a new non-volatile data interface
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NvDataInterface(Element);
abstraction_element!(NvDataInterface, NvDataInterface);
impl IdentifiableAbstractionElement for NvDataInterface {}
impl AbstractPortInterface for NvDataInterface {}

impl NvDataInterface {
    /// Create a new `NvDataInterface`
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let nv_data_interface = elements.create_named_sub_element(ElementName::NvDataInterface, name)?;

        Ok(Self(nv_data_interface))
    }
}

//##################################################################

/// A `TriggerInterface` declares a number of triggers that can be sent by an trigger source
///
/// Use [`ArPackage::create_trigger_interface`] to create a new trigger interface
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TriggerInterface(Element);
abstraction_element!(TriggerInterface, TriggerInterface);
impl IdentifiableAbstractionElement for TriggerInterface {}
impl AbstractPortInterface for TriggerInterface {}

impl TriggerInterface {
    /// Create a new `TriggerInterface`
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let trigger_interface = elements.create_named_sub_element(ElementName::TriggerInterface, name)?;

        Ok(Self(trigger_interface))
    }
}

//##################################################################

/// The `AbstractPortInterface` trait is a marker trait for all port interfaces
pub trait AbstractPortInterface: AbstractionElement {}

//##################################################################

/// The `PortInterface` enum represents all possible port interfaces
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PortInterface {
    /// The interface is a sender-receiver interface
    SenderReceiverInterface(SenderReceiverInterface),
    /// The interface is a client-server interface
    ClientServerInterface(ClientServerInterface),
    /// The interface is a mode switch interface
    ModeSwitchInterface(ModeSwitchInterface),
    /// The interface is a parameter interface
    ParameterInterface(ParameterInterface),
    /// The interface is a non-volatile data interface
    NvDataInterface(NvDataInterface),
    /// The interface is a trigger interface
    TriggerInterface(TriggerInterface),
}

impl AbstractionElement for PortInterface {
    fn element(&self) -> &Element {
        match self {
            PortInterface::SenderReceiverInterface(sender_receiver_interface) => sender_receiver_interface.element(),
            PortInterface::ClientServerInterface(client_server_interface) => client_server_interface.element(),
            PortInterface::ModeSwitchInterface(mode_switch_interface) => mode_switch_interface.element(),
            PortInterface::ParameterInterface(parameter_interface) => parameter_interface.element(),
            PortInterface::NvDataInterface(nv_data_interface) => nv_data_interface.element(),
            PortInterface::TriggerInterface(trigger_interface) => trigger_interface.element(),
        }
    }
}

impl IdentifiableAbstractionElement for PortInterface {}
impl AbstractPortInterface for PortInterface {}

impl TryFrom<Element> for PortInterface {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::SenderReceiverInterface => {
                Ok(PortInterface::SenderReceiverInterface(SenderReceiverInterface(element)))
            }
            ElementName::ClientServerInterface => {
                Ok(PortInterface::ClientServerInterface(ClientServerInterface(element)))
            }
            ElementName::ModeSwitchInterface => Ok(PortInterface::ModeSwitchInterface(ModeSwitchInterface(element))),
            ElementName::ParameterInterface => Ok(PortInterface::ParameterInterface(ParameterInterface(element))),
            ElementName::NvDataInterface => Ok(PortInterface::NvDataInterface(NvDataInterface(element))),
            ElementName::TriggerInterface => Ok(PortInterface::TriggerInterface(TriggerInterface(element))),
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "PortInterface".to_string(),
            }),
        }
    }
}

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        AutosarModelAbstraction,
        datatype::{BaseTypeEncoding, ImplementationDataTypeSettings, TextValueSpecification},
        software_component::AbstractSwComponentType,
    };
    use autosar_data::AutosarVersion;

    #[test]
    fn test_interfaces() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        let sender_receiver_interface = package
            .create_sender_receiver_interface("sender_receiver_interface")
            .unwrap();
        let client_server_interface = package
            .create_client_server_interface("client_server_interface")
            .unwrap();
        let mode_switch_interface = package.create_mode_switch_interface("mode_switch_interface").unwrap();
        let parameter_interface = package.create_parameter_interface("parameter_interface").unwrap();
        let nv_data_interface = package.create_nv_data_interface("nv_data_interface").unwrap();
        let trigger_interface = package.create_trigger_interface("trigger_interface").unwrap();

        let composition = package.create_composition_sw_component_type("composition").unwrap();

        let port_1 = composition.create_p_port("port_1", &sender_receiver_interface).unwrap();
        assert!(matches!(
            port_1.port_interface(),
            Ok(PortInterface::SenderReceiverInterface(interface)) if interface == sender_receiver_interface
        ));
        assert_eq!(
            port_1.port_interface().unwrap().element(),
            sender_receiver_interface.element()
        );

        let port_2 = composition.create_p_port("port_2", &client_server_interface).unwrap();
        assert!(matches!(
            port_2.port_interface(),
            Ok(PortInterface::ClientServerInterface(interface)) if interface == client_server_interface
        ));
        assert_eq!(
            port_2.port_interface().unwrap().element(),
            client_server_interface.element()
        );

        let port_3 = composition.create_p_port("port_3", &mode_switch_interface).unwrap();
        assert!(matches!(
            port_3.port_interface(),
            Ok(PortInterface::ModeSwitchInterface(interface)) if interface == mode_switch_interface
        ));
        assert_eq!(
            port_3.port_interface().unwrap().element(),
            mode_switch_interface.element()
        );

        let port_4 = composition.create_p_port("port_4", &parameter_interface).unwrap();
        assert!(matches!(
            port_4.port_interface(),
            Ok(PortInterface::ParameterInterface(interface)) if interface == parameter_interface
        ));
        assert_eq!(
            port_4.port_interface().unwrap().element(),
            parameter_interface.element()
        );

        let port_5 = composition.create_p_port("port_5", &nv_data_interface).unwrap();
        assert!(matches!(
            port_5.port_interface(),
            Ok(PortInterface::NvDataInterface(interface)) if interface == nv_data_interface
        ));
        assert_eq!(port_5.port_interface().unwrap().element(), nv_data_interface.element());

        let port_6 = composition.create_p_port("port_6", &trigger_interface).unwrap();
        assert!(matches!(
            port_6.port_interface(),
            Ok(PortInterface::TriggerInterface(interface)) if interface == trigger_interface
        ));
        assert_eq!(port_6.port_interface().unwrap().element(), trigger_interface.element());
    }

    #[test]
    fn parameter_interface() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        let parameter_interface = package.create_parameter_interface("parameter_interface").unwrap();
        let base_type = package
            .create_sw_base_type("base", 32, BaseTypeEncoding::None, None, None, None)
            .unwrap();
        let datatype = package
            .create_implementation_data_type(&ImplementationDataTypeSettings::Value {
                name: "ImplementationValue".to_string(),
                base_type: base_type.clone(),
                compu_method: None,
                data_constraint: None,
            })
            .unwrap();

        let parameter = parameter_interface.create_parameter("parameter", &datatype).unwrap();
        assert_eq!(parameter.name().as_deref().unwrap(), "parameter");

        assert_eq!(parameter_interface.parameters().count(), 1);

        let value_spec = TextValueSpecification {
            label: None,
            value: "42".to_string(),
        };
        parameter.set_init_value(value_spec).unwrap();
        assert_eq!(
            parameter.init_value().unwrap(),
            TextValueSpecification {
                label: None,
                value: "42".to_string()
            }
            .into()
        );
    }

    #[test]
    fn mode_switch_interface() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        let mode_declaration_group = package
            .create_mode_declaration_group("mode_declaration_group", None)
            .unwrap();

        let mode_switch_interface = package.create_mode_switch_interface("mode_switch_interface").unwrap();
        let mode_group = mode_switch_interface
            .create_mode_group("mode_group", &mode_declaration_group)
            .unwrap();
        assert_eq!(mode_switch_interface.mode_group().unwrap(), mode_group);
        assert_eq!(mode_group.mode_declaration_group().unwrap(), mode_declaration_group);
    }
}
