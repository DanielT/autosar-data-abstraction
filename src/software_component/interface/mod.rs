use crate::{abstraction_element, AbstractionElement, ArPackage, AutosarAbstractionError, Element};
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

impl AbstractPortInterface for ModeSwitchInterface {}

impl ModeSwitchInterface {
    /// Create a new `ModeSwitchInterface`
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let mode_switch_interface = elements.create_named_sub_element(ElementName::ModeSwitchInterface, name)?;

        Ok(Self(mode_switch_interface))
    }
}

//##################################################################

/// A `ParameterInterface` defines a set of parameters that can be accessed
///
/// Use [`ArPackage::create_parameter_interface`] to create a new parameter interface
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParameterInterface(Element);
abstraction_element!(ParameterInterface, ParameterInterface);

impl AbstractPortInterface for ParameterInterface {}

impl ParameterInterface {
    /// Create a new `ParameterInterface`
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let parameter_interface = elements.create_named_sub_element(ElementName::ParameterInterface, name)?;

        Ok(Self(parameter_interface))
    }
}

//##################################################################

/// An `NvDataInterface` defines non-volatile data that can be accessed through the interface
///
/// Use [`ArPackage::create_nv_data_interface`] to create a new non-volatile data interface
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NvDataInterface(Element);
abstraction_element!(NvDataInterface, NvDataInterface);

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
    use crate::{software_component::AbstractSwComponentType, AutosarModelAbstraction};
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
}
