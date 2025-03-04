use crate::{
    AbstractionElement, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element,
    software_component,
};
use autosar_data::{Element, ElementName};
use software_component::{AbstractPortInterface, PortInterface, SwComponentType};

//#########################################################

/// `RPortPrototype` represents a required port prototype
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RPortPrototype(Element);
abstraction_element!(RPortPrototype, RPortPrototype);
impl IdentifiableAbstractionElement for RPortPrototype {}

impl RPortPrototype {
    /// Create a new `RPortPrototype`
    pub(crate) fn new<T: AbstractPortInterface>(
        name: &str,
        parent_element: &Element,
        interface: &T,
    ) -> Result<Self, AutosarAbstractionError> {
        let r_port_prototype = parent_element.create_named_sub_element(ElementName::RPortPrototype, name)?;
        r_port_prototype
            .create_sub_element(ElementName::RequiredInterfaceTref)?
            .set_reference_target(interface.element())?;

        Ok(Self(r_port_prototype))
    }

    /// Get the port interface of the port prototype
    pub fn port_interface(&self) -> Result<PortInterface, AutosarAbstractionError> {
        let interface_elem = self
            .element()
            .get_sub_element(ElementName::RequiredInterfaceTref)
            .and_then(|r| r.get_reference_target().ok())
            .ok_or(AutosarAbstractionError::InvalidParameter(
                "RPortPrototype is incomplete: RequiredInterfaceTref is missing".to_string(),
            ))?;
        PortInterface::try_from(interface_elem)
    }

    /// Get the component type containing the port prototype
    pub fn component_type(&self) -> Result<SwComponentType, AutosarAbstractionError> {
        let component_type_elem = self.element().named_parent()?.unwrap();
        SwComponentType::try_from(component_type_elem)
    }
}

//##################################################################

/// `PPortPrototype` represents a provided port prototype
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PPortPrototype(Element);
abstraction_element!(PPortPrototype, PPortPrototype);
impl IdentifiableAbstractionElement for PPortPrototype {}

impl PPortPrototype {
    /// Create a new `PPortPrototype`
    pub(crate) fn new<T: AbstractPortInterface>(
        name: &str,
        parent_element: &Element,
        interface: &T,
    ) -> Result<Self, AutosarAbstractionError> {
        let p_port_prototype = parent_element.create_named_sub_element(ElementName::PPortPrototype, name)?;
        p_port_prototype
            .create_sub_element(ElementName::ProvidedInterfaceTref)?
            .set_reference_target(interface.element())?;

        Ok(Self(p_port_prototype))
    }

    /// Get the port interface of the port prototype
    pub fn port_interface(&self) -> Result<PortInterface, AutosarAbstractionError> {
        let interface_elem = self
            .element()
            .get_sub_element(ElementName::ProvidedInterfaceTref)
            .and_then(|r| r.get_reference_target().ok())
            .ok_or(AutosarAbstractionError::InvalidParameter(
                "PPortPrototype is incomplete: ProvidedInterfaceTref is missing".to_string(),
            ))?;
        PortInterface::try_from(interface_elem)
    }

    /// Get the component type containing the port prototype
    pub fn component_type(&self) -> Result<SwComponentType, AutosarAbstractionError> {
        let component_type_elem = self.element().named_parent()?.unwrap();
        SwComponentType::try_from(component_type_elem)
    }
}

//##################################################################

/// `PRPortPrototype` represents a provided and required port prototype
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PRPortPrototype(Element);
abstraction_element!(PRPortPrototype, PrPortPrototype);
impl IdentifiableAbstractionElement for PRPortPrototype {}

impl PRPortPrototype {
    /// Create a new `PRPortPrototype`
    pub(crate) fn new<T: AbstractPortInterface>(
        name: &str,
        parent_element: &Element,
        interface: &T,
    ) -> Result<Self, AutosarAbstractionError> {
        if interface.element().element_name() == ElementName::ParameterInterface {
            return Err(AutosarAbstractionError::InvalidParameter(
                "ParameterInterface is not allowed for PRPortPrototype".to_string(),
            ));
        }

        let pr_port_prototype = parent_element.create_named_sub_element(ElementName::PrPortPrototype, name)?;
        pr_port_prototype
            .create_sub_element(ElementName::ProvidedRequiredInterfaceTref)?
            .set_reference_target(interface.element())?;

        Ok(Self(pr_port_prototype))
    }

    /// Get the port interface of the port prototype
    pub fn port_interface(&self) -> Result<PortInterface, AutosarAbstractionError> {
        let interface_elem = self
            .element()
            .get_sub_element(ElementName::ProvidedRequiredInterfaceTref)
            .and_then(|r| r.get_reference_target().ok())
            .ok_or(AutosarAbstractionError::InvalidParameter(
                "PRPortPrototype is incomplete: ProvidedRequiredInterfaceTref is missing".to_string(),
            ))?;
        PortInterface::try_from(interface_elem)
    }

    /// Get the component type containing the port prototype
    pub fn component_type(&self) -> Result<SwComponentType, AutosarAbstractionError> {
        let component_type_elem = self.element().named_parent()?.unwrap();
        SwComponentType::try_from(component_type_elem)
    }
}

//##################################################################

/// The `PortPrototype` enum represents all possible kinds of port prototypes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PortPrototype {
    /// A required port prototype
    R(RPortPrototype),
    /// A provided port prototype
    P(PPortPrototype),
    /// A provided and required port prototype
    PR(PRPortPrototype),
}

impl AbstractionElement for PortPrototype {
    fn element(&self) -> &Element {
        match self {
            PortPrototype::R(port) => port.element(),
            PortPrototype::P(port) => port.element(),
            PortPrototype::PR(port) => port.element(),
        }
    }
}

impl IdentifiableAbstractionElement for PortPrototype {}

impl From<RPortPrototype> for PortPrototype {
    fn from(port: RPortPrototype) -> Self {
        PortPrototype::R(port)
    }
}

impl From<PPortPrototype> for PortPrototype {
    fn from(port: PPortPrototype) -> Self {
        PortPrototype::P(port)
    }
}

impl From<PRPortPrototype> for PortPrototype {
    fn from(port: PRPortPrototype) -> Self {
        PortPrototype::PR(port)
    }
}

impl TryFrom<Element> for PortPrototype {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::RPortPrototype => Ok(PortPrototype::R(RPortPrototype(element))),
            ElementName::PPortPrototype => Ok(PortPrototype::P(PPortPrototype(element))),
            ElementName::PrPortPrototype => Ok(PortPrototype::PR(PRPortPrototype(element))),
            _ => Err(AutosarAbstractionError::ConversionError {
                element: element.clone(),
                dest: "PortPrototype".to_string(),
            }),
        }
    }
}

impl PortPrototype {
    /// Get the port interface of the port prototype
    pub fn port_interface(&self) -> Result<PortInterface, AutosarAbstractionError> {
        match self {
            PortPrototype::R(port) => port.port_interface(),
            PortPrototype::P(port) => port.port_interface(),
            PortPrototype::PR(port) => port.port_interface(),
        }
    }

    /// Get the component type containing the port prototype
    pub fn component_type(&self) -> Result<SwComponentType, AutosarAbstractionError> {
        let component_type_elem = self.element().named_parent()?.unwrap();
        SwComponentType::try_from(component_type_elem)
    }
}

//##################################################################

/// `PortGroup` represents a group of ports
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PortGroup(Element);
abstraction_element!(PortGroup, PortGroup);
impl IdentifiableAbstractionElement for PortGroup {}

impl PortGroup {
    /// Create a new `PortGroup`
    pub(crate) fn new(name: &str, parent_element: &Element) -> Result<Self, AutosarAbstractionError> {
        let port_group = parent_element.create_named_sub_element(ElementName::PortGroup, name)?;

        Ok(Self(port_group))
    }
}

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::AutosarModelAbstraction;
    use autosar_data::AutosarVersion;
    use software_component::AbstractSwComponentType;

    #[test]
    fn ports() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        let comp = package.create_composition_sw_component_type("comp").unwrap();

        let port_interface = package.create_sender_receiver_interface("sr_interface").unwrap();
        let r_port = comp.create_r_port("sr_r_port", &port_interface).unwrap();
        let p_port = comp.create_p_port("sr_p_port", &port_interface).unwrap();
        let pr_port = comp.create_pr_port("sr_pr_port", &port_interface).unwrap();

        assert_eq!(comp.ports().count(), 3);
        let ports: Vec<PortPrototype> = comp.ports().collect();
        assert_eq!(ports[0], r_port.clone().into());
        assert_eq!(ports[1], p_port.clone().into());
        assert_eq!(ports[2], pr_port.clone().into());
        assert_eq!(r_port.component_type().unwrap(), comp.clone().into());
        assert_eq!(p_port.component_type().unwrap(), comp.clone().into());
        assert_eq!(pr_port.component_type().unwrap(), comp.clone().into());
        assert_eq!(ports[0].component_type().unwrap(), comp.clone().into());

        let port_interface = package.create_client_server_interface("cs_interface").unwrap();
        let r_port = comp.create_r_port("cs_r_port", &port_interface).unwrap();
        let p_port = comp.create_p_port("cs_p_port", &port_interface).unwrap();
        let pr_port = comp.create_pr_port("cs_pr_port", &port_interface).unwrap();

        assert_eq!(comp.ports().count(), 6);
        let ports: Vec<PortPrototype> = comp.ports().collect();
        assert_eq!(ports[3], r_port.into());
        assert_eq!(ports[4], p_port.into());
        assert_eq!(ports[5], pr_port.into());

        let port_interface = package.create_mode_switch_interface("ms_interface").unwrap();
        let r_port = comp.create_r_port("ms_r_port", &port_interface).unwrap();
        let p_port = comp.create_p_port("ms_p_port", &port_interface).unwrap();
        let pr_port = comp.create_pr_port("ms_pr_port", &port_interface).unwrap();

        assert_eq!(comp.ports().count(), 9);
        let ports: Vec<PortPrototype> = comp.ports().collect();
        assert_eq!(ports[6], r_port.into());
        assert_eq!(ports[7], p_port.into());
        assert_eq!(ports[8], pr_port.into());

        let port_interface = package.create_nv_data_interface("nv_interface").unwrap();
        let r_port = comp.create_r_port("nv_r_port", &port_interface).unwrap();
        let p_port = comp.create_p_port("nv_p_port", &port_interface).unwrap();
        let pr_port = comp.create_pr_port("nv_pr_port", &port_interface).unwrap();

        assert_eq!(comp.ports().count(), 12);
        let ports: Vec<PortPrototype> = comp.ports().collect();
        assert_eq!(ports[9], r_port.into());
        assert_eq!(ports[10], p_port.into());
        assert_eq!(ports[11], pr_port.into());

        let port_interface = package.create_parameter_interface("param_interface").unwrap();
        let r_port = comp.create_r_port("param_r_port", &port_interface).unwrap();
        let p_port = comp.create_p_port("param_p_port", &port_interface).unwrap();
        let pr_port_result = comp.create_pr_port("param_pr_port", &port_interface);
        assert!(pr_port_result.is_err());

        assert_eq!(comp.ports().count(), 14);
        let ports: Vec<PortPrototype> = comp.ports().collect();
        assert_eq!(ports[12], r_port.into());
        assert_eq!(ports[13], p_port.into());

        let port_interface = package.create_trigger_interface("trigger_interface").unwrap();
        let r_port = comp.create_r_port("trigger_r_port", &port_interface).unwrap();
        let p_port = comp.create_p_port("trigger_p_port", &port_interface).unwrap();
        let pr_port = comp.create_pr_port("trigger_pr_port", &port_interface).unwrap();

        assert_eq!(comp.ports().count(), 17);
        let ports: Vec<PortPrototype> = comp.ports().collect();
        assert_eq!(ports[14], r_port.into());
        assert_eq!(ports[15], p_port.into());
        assert_eq!(ports[16], pr_port.into());
    }
}
