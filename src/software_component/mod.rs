//! Software component types and compositions
//!
//! This module contains the definition of software component types and instances.
//! It also contains the definition of the composition hierarchy, and the connectors between components.

use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, Element, IdentifiableAbstractionElement,
    abstraction_element,
};
use autosar_data::ElementName;

mod connector;
mod interface;
mod internal_behavior;
mod port;

pub use connector::*;
pub use interface::*;
pub use internal_behavior::*;
pub use port::*;

//##################################################################

/// The `AbstractSwComponentType` is the common interface for all types of software components
pub trait AbstractSwComponentType: IdentifiableAbstractionElement {
    /// iterator over the instances of the component type
    fn instances(&self) -> Vec<ComponentPrototype> {
        let model_result = self.element().model();
        let path_result = self.element().path();
        if let (Ok(model), Ok(path)) = (model_result, path_result) {
            model
                .get_references_to(&path)
                .iter()
                .filter_map(|e| {
                    e.upgrade()
                        .and_then(|ref_elem| ref_elem.named_parent().ok().flatten())
                        .and_then(|elem| ComponentPrototype::try_from(elem).ok())
                })
                .collect()
        } else {
            vec![]
        }
    }

    /// list all compositions containing instances of the component type
    fn parent_compositions(&self) -> Vec<CompositionSwComponentType> {
        self.instances()
            .iter()
            .filter_map(|swcp| swcp.element().named_parent().ok().flatten())
            .filter_map(|elem| CompositionSwComponentType::try_from(elem).ok())
            .collect()
    }

    /// create a new required port with the given name and port interface
    fn create_r_port<T: AbstractPortInterface>(
        &self,
        name: &str,
        port_interface: &T,
    ) -> Result<RPortPrototype, AutosarAbstractionError> {
        let ports = self.element().get_or_create_sub_element(ElementName::Ports)?;
        RPortPrototype::new(name, &ports, port_interface)
    }

    /// create a new provided port with the given name and port interface
    fn create_p_port<T: AbstractPortInterface>(
        &self,
        name: &str,
        port_interface: &T,
    ) -> Result<PPortPrototype, AutosarAbstractionError> {
        let ports = self.element().get_or_create_sub_element(ElementName::Ports)?;
        PPortPrototype::new(name, &ports, port_interface)
    }

    /// create a new provided required port with the given name and port interface
    fn create_pr_port<T: AbstractPortInterface>(
        &self,
        name: &str,
        port_interface: &T,
    ) -> Result<PRPortPrototype, AutosarAbstractionError> {
        let ports = self.element().get_or_create_sub_element(ElementName::Ports)?;
        PRPortPrototype::new(name, &ports, port_interface)
    }

    /// get an iterator over the ports of the component
    fn ports(&self) -> impl Iterator<Item = PortPrototype> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::Ports)
            .into_iter()
            .flat_map(|ports| ports.sub_elements())
            .filter_map(|elem| PortPrototype::try_from(elem).ok())
    }

    /// create a new port group
    fn create_port_group(&self, name: &str) -> Result<PortGroup, AutosarAbstractionError> {
        let port_groups = self.element().get_or_create_sub_element(ElementName::PortGroups)?;
        PortGroup::new(name, &port_groups)
    }
}

//##################################################################

/// Shared trait identifiying atomic software components
///
/// An atomic software component is atomic in the sense that it cannot be further decomposed
pub trait AtomicSwComponentType: AbstractSwComponentType {
    /// create an SwcInternalBehavior for the component
    ///
    /// A component can have only one internal behavior, but since the internal behavior is a variation point,
    /// more than one internal behavior can be created. In this case the variation point settings must ensure that only one
    /// internal behavior is active.
    fn create_swc_internal_behavior(&self, name: &str) -> Result<SwcInternalBehavior, AutosarAbstractionError> {
        let internal_behaviors = self
            .element()
            .get_or_create_sub_element(ElementName::InternalBehaviors)?;
        SwcInternalBehavior::new(name, &internal_behaviors)
    }

    /// iterate over all swc internal behaviors - typically zero or one
    fn swc_internal_behaviors(&self) -> impl Iterator<Item = SwcInternalBehavior> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::InternalBehaviors)
            .into_iter()
            .flat_map(|internal_behaviors| internal_behaviors.sub_elements())
            .filter_map(|elem| SwcInternalBehavior::try_from(elem).ok())
    }
}

//##################################################################

/// A `CompositionSwComponentType` is a software component that contains other software components
///
/// Use [`ArPackage::create_composition_sw_component_type`] to create a new composition sw component type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompositionSwComponentType(Element);
abstraction_element!(CompositionSwComponentType, CompositionSwComponentType);
impl IdentifiableAbstractionElement for CompositionSwComponentType {}

impl CompositionSwComponentType {
    /// create a new composition component with the given name
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let composition = elements.create_named_sub_element(ElementName::CompositionSwComponentType, name)?;
        Ok(Self(composition))
    }

    /// check if the composition is a parent (or grand-parent, etc.) of the component
    pub fn is_parent_of<T: AbstractSwComponentType>(&self, other: &T) -> bool {
        // the expectation is that in normal cases each component has only one parent
        // additionally there should never be any cycles in the composition hierarchy
        let mut work_items = other.parent_compositions();
        let mut counter = 1000; // just to prevent infinite loops, since I don't trust files generated by other tools
        while !work_items.is_empty() && counter > 0 {
            counter -= 1;
            if work_items.contains(self) {
                return true;
            }
            // the uses of pop here makes this a depth-first search in the case where there are multiple parents
            let item = work_items.pop().unwrap();
            work_items.extend(item.parent_compositions());
        }

        false
    }

    /// create a component of type `component_type` in the composition
    ///
    /// It is not allowed to form cycles in the composition hierarchy, and this will return an error
    pub fn create_component<T: Into<SwComponentType> + Clone>(
        &self,
        name: &str,
        component_type: &T,
    ) -> Result<SwComponentPrototype, AutosarAbstractionError> {
        let component_type = component_type.clone().into();
        if let SwComponentType::Composition(composition_component) = &component_type {
            if composition_component.is_parent_of(self) {
                return Err(AutosarAbstractionError::InvalidParameter(
                    "Creating a cycle in the composition hierarchy".to_string(),
                ));
            }
        }

        let components = self.element().get_or_create_sub_element(ElementName::Components)?;
        SwComponentPrototype::new(name, &components, &component_type)
    }

    /// get an iterator over the components of the composition
    pub fn components(&self) -> impl Iterator<Item = SwComponentPrototype> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::Components)
            .into_iter()
            .flat_map(|components| components.sub_elements())
            .filter_map(|elem| SwComponentPrototype::try_from(elem).ok())
    }

    /// create a new delegation connector between an inner port and an outer port
    ///
    /// The two ports must be compatible.
    pub fn create_delegation_connector<T1: Into<PortPrototype> + Clone, T2: Into<PortPrototype> + Clone>(
        &self,
        name: &str,
        inner_port: &T1,
        inner_sw_prototype: &SwComponentPrototype,
        outer_port: &T2,
    ) -> Result<DelegationSwConnector, AutosarAbstractionError> {
        self.create_delegation_connector_internal(
            name,
            &inner_port.clone().into(),
            inner_sw_prototype,
            &outer_port.clone().into(),
        )
    }

    /// create a new delegation connector between an inner port and an outer port
    /// this is the actual implementation of the public method, but without the generic parameters
    fn create_delegation_connector_internal(
        &self,
        name: &str,
        inner_port: &PortPrototype,
        inner_sw_prototype: &SwComponentPrototype,
        outer_port: &PortPrototype,
    ) -> Result<DelegationSwConnector, AutosarAbstractionError> {
        // check the compatibility of the interfaces
        let interface_1 = inner_port.port_interface()?;
        let interface_2 = outer_port.port_interface()?;
        if std::mem::discriminant(&interface_1) != std::mem::discriminant(&interface_2) {
            return Err(AutosarAbstractionError::InvalidParameter(
                "The interfaces of the two ports are not compatible".to_string(),
            ));
        }

        // check that the inner port is part of the inner component
        let inner_swc_from_port = SwComponentType::try_from(inner_port.element().named_parent()?.unwrap())?;
        let inner_swc_from_component =
            inner_sw_prototype
                .component_type()
                .ok_or(AutosarAbstractionError::InvalidParameter(
                    "The inner component is incomplete and lacks a type reference".to_string(),
                ))?;
        if inner_swc_from_port != inner_swc_from_component {
            return Err(AutosarAbstractionError::InvalidParameter(
                "The inner port must be part of the inner component".to_string(),
            ));
        }

        let swc_self = self.clone().into();
        let outer_swc_from_port = SwComponentType::try_from(outer_port.element().named_parent()?.unwrap())?;
        if outer_swc_from_port != swc_self {
            return Err(AutosarAbstractionError::InvalidParameter(
                "The outer port must be part of the composition".to_string(),
            ));
        }

        // create the delegation connector
        let connectors = self.element().get_or_create_sub_element(ElementName::Connectors)?;

        DelegationSwConnector::new(
            name,
            &connectors,
            inner_port, // inner port = port of the contained component
            inner_sw_prototype,
            outer_port, // outer port = port of the composition
        )
    }

    /// create a new assembly connector between two ports of contained software components
    ///
    /// The two ports must be compatible.
    pub fn create_assembly_connector<T1: Into<PortPrototype> + Clone, T2: Into<PortPrototype> + Clone>(
        &self,
        name: &str,
        port_1: &T1,
        sw_prototype_1: &SwComponentPrototype,
        port_2: &T2,
        sw_prototype_2: &SwComponentPrototype,
    ) -> Result<AssemblySwConnector, AutosarAbstractionError> {
        self.create_assembly_connector_internal(
            name,
            &port_1.clone().into(),
            sw_prototype_1,
            &port_2.clone().into(),
            sw_prototype_2,
        )
    }

    fn create_assembly_connector_internal(
        &self,
        name: &str,
        port_1: &PortPrototype,
        sw_prototype_1: &SwComponentPrototype,
        port_2: &PortPrototype,
        sw_prototype_2: &SwComponentPrototype,
    ) -> Result<AssemblySwConnector, AutosarAbstractionError> {
        // check the compatibility of the interfaces
        let interface_1 = port_1.port_interface()?;
        let interface_2 = port_2.port_interface()?;
        if std::mem::discriminant(&interface_1) != std::mem::discriminant(&interface_2) {
            return Err(AutosarAbstractionError::InvalidParameter(
                "The interfaces of the two ports are not compatible".to_string(),
            ));
        }

        // check that the ports are part of the correct components
        let swc_1_from_port = SwComponentType::try_from(port_1.element().named_parent()?.unwrap())?;
        let swc_1_from_component = sw_prototype_1
            .component_type()
            .ok_or(AutosarAbstractionError::InvalidParameter(
                "SW component prototype 1 is incomplete and lacks a type reference".to_string(),
            ))?;
        if swc_1_from_port != swc_1_from_component {
            return Err(AutosarAbstractionError::InvalidParameter(
                "The first port must be part of the first software component".to_string(),
            ));
        }

        let swc_2_from_port = SwComponentType::try_from(port_2.element().named_parent()?.unwrap())?;
        let swc_2_from_component = sw_prototype_2
            .component_type()
            .ok_or(AutosarAbstractionError::InvalidParameter(
                "SW component prototype 2 is incomplete and lacks a type reference".to_string(),
            ))?;
        if swc_2_from_port != swc_2_from_component {
            return Err(AutosarAbstractionError::InvalidParameter(
                "The second port must be part of the second software component".to_string(),
            ));
        }

        // check that both SWCs are part of the composition
        if &sw_prototype_1.parent_composition()? != self {
            return Err(AutosarAbstractionError::InvalidParameter(
                "The first software component must be part of the composition".to_string(),
            ));
        }
        if &sw_prototype_2.parent_composition()? != self {
            return Err(AutosarAbstractionError::InvalidParameter(
                "The second software component must be part of the composition".to_string(),
            ));
        }

        // create the assembly connector
        let connectors = self.element().get_or_create_sub_element(ElementName::Connectors)?;
        AssemblySwConnector::new(name, &connectors, port_1, sw_prototype_1, port_2, sw_prototype_2)
    }

    /// create a new passthrough connector between two outer ports of the composition
    ///
    /// The two ports must be compatible.
    pub fn create_pass_through_connector<T1: Into<PortPrototype> + Clone, T2: Into<PortPrototype> + Clone>(
        &self,
        name: &str,
        port_1: &T1,
        port_2: &T2,
    ) -> Result<PassThroughSwConnector, AutosarAbstractionError> {
        self.create_pass_through_connector_internal(name, &port_1.clone().into(), &port_2.clone().into())
    }

    fn create_pass_through_connector_internal(
        &self,
        name: &str,
        port_1: &PortPrototype,
        port_2: &PortPrototype,
    ) -> Result<PassThroughSwConnector, AutosarAbstractionError> {
        // check the compatibility of the interfaces
        let interface_1 = port_1.port_interface()?;
        let interface_2 = port_2.port_interface()?;
        if std::mem::discriminant(&interface_1) != std::mem::discriminant(&interface_2) {
            return Err(AutosarAbstractionError::InvalidParameter(
                "The interfaces of the two ports are not compatible".to_string(),
            ));
        }

        // decide what kind of connector to create
        let swc_1 = SwComponentType::try_from(port_1.element().named_parent()?.unwrap())?;
        let swc_2 = SwComponentType::try_from(port_2.element().named_parent()?.unwrap())?;
        let swc_self = self.clone().into();

        // both ports must be part of the composition
        if swc_1 != swc_self || swc_2 != swc_self {
            return Err(AutosarAbstractionError::InvalidParameter(
                "The ports must be part of the composition".to_string(),
            ));
        }

        let connectors = self.element().get_or_create_sub_element(ElementName::Connectors)?;
        PassThroughSwConnector::new(name, &connectors, port_1, port_2)
    }

    /// iterate over all connectors
    pub fn connectors(&self) -> impl Iterator<Item = SwConnector> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::Connectors)
            .into_iter()
            .flat_map(|connectors| connectors.sub_elements())
            .filter_map(|elem| SwConnector::try_from(elem).ok())
    }
}

impl AbstractSwComponentType for CompositionSwComponentType {}

//##################################################################

/// An `ApplicationSwComponentType` is a software component that provides application functionality
///
/// Use [`ArPackage::create_application_sw_component_type`] to create a new application sw component type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApplicationSwComponentType(Element);
abstraction_element!(ApplicationSwComponentType, ApplicationSwComponentType);
impl IdentifiableAbstractionElement for ApplicationSwComponentType {}

impl ApplicationSwComponentType {
    /// create a new application component with the given name
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let application = elements.create_named_sub_element(ElementName::ApplicationSwComponentType, name)?;
        Ok(Self(application))
    }
}

impl AbstractSwComponentType for ApplicationSwComponentType {}
impl AtomicSwComponentType for ApplicationSwComponentType {}

//##################################################################

/// A `ComplexDeviceDriverSwComponentType` is a software component that provides complex device driver functionality
///
/// Use [`ArPackage::create_complex_device_driver_sw_component_type`] to create a new complex device driver sw component type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComplexDeviceDriverSwComponentType(Element);
abstraction_element!(ComplexDeviceDriverSwComponentType, ComplexDeviceDriverSwComponentType);
impl IdentifiableAbstractionElement for ComplexDeviceDriverSwComponentType {}

impl ComplexDeviceDriverSwComponentType {
    /// create a new complex device driver component with the given name
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let cdd = elements.create_named_sub_element(ElementName::ComplexDeviceDriverSwComponentType, name)?;
        Ok(Self(cdd))
    }
}

impl AbstractSwComponentType for ComplexDeviceDriverSwComponentType {}
impl AtomicSwComponentType for ComplexDeviceDriverSwComponentType {}

//##################################################################

/// `ServiceSwComponentType` is used for configuring services for a given ECU. Instances of this class should only
/// be created in ECU Configuration phase for the specific purpose of the service configuration.
///
/// Use [`ArPackage::create_service_sw_component_type`] to create a new service sw component type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceSwComponentType(Element);
abstraction_element!(ServiceSwComponentType, ServiceSwComponentType);
impl IdentifiableAbstractionElement for ServiceSwComponentType {}

impl ServiceSwComponentType {
    /// create a new service component with the given name
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let service = elements.create_named_sub_element(ElementName::ServiceSwComponentType, name)?;
        Ok(Self(service))
    }
}

impl AbstractSwComponentType for ServiceSwComponentType {}
impl AtomicSwComponentType for ServiceSwComponentType {}

//##################################################################

/// `SensorActuatorSwComponentType` is used to connect sensor/acutator devices to the ECU configuration
///
/// Use [`ArPackage::create_sensor_actuator_sw_component_type`] to create a new sensor/actuator sw component type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SensorActuatorSwComponentType(Element);
abstraction_element!(SensorActuatorSwComponentType, SensorActuatorSwComponentType);
impl IdentifiableAbstractionElement for SensorActuatorSwComponentType {}

impl SensorActuatorSwComponentType {
    /// create a new sensor/actuator component with the given name
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let sensor_actuator = elements.create_named_sub_element(ElementName::SensorActuatorSwComponentType, name)?;
        Ok(Self(sensor_actuator))
    }
}

impl AbstractSwComponentType for SensorActuatorSwComponentType {}
impl AtomicSwComponentType for SensorActuatorSwComponentType {}

//##################################################################

/// The `ECUAbstraction` is a special `AtomicSwComponentType` that resides between a software-component
/// that wants to access ECU periphery and the Microcontroller Abstraction
///
/// Use [`ArPackage::create_ecu_abstraction_sw_component_type`] to create a new ECU abstraction sw component type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcuAbstractionSwComponentType(Element);
abstraction_element!(EcuAbstractionSwComponentType, EcuAbstractionSwComponentType);
impl IdentifiableAbstractionElement for EcuAbstractionSwComponentType {}

impl EcuAbstractionSwComponentType {
    /// create a new ECU abstraction component with the given name
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let ecu_abstraction = elements.create_named_sub_element(ElementName::EcuAbstractionSwComponentType, name)?;
        Ok(Self(ecu_abstraction))
    }
}

impl AbstractSwComponentType for EcuAbstractionSwComponentType {}
impl AtomicSwComponentType for EcuAbstractionSwComponentType {}

//##################################################################

/// The `SwComponentType` enum represents all possible types of software components
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SwComponentType {
    /// the component is `CompositionSwComponentType`
    Composition(CompositionSwComponentType),
    /// the component is `ApplicationSwComponentType`
    Application(ApplicationSwComponentType),
    /// the component is `ComplexDeviceDriverSwComponentType`
    ComplexDeviceDriver(ComplexDeviceDriverSwComponentType),
    /// the component is `ServiceSwComponentType`
    Service(ServiceSwComponentType),
    /// the component is `SensorActuatorSwComponentType`
    SensorActuator(SensorActuatorSwComponentType),
    /// the component is `EcuAbstractionSwComponentType`
    EcuAbstraction(EcuAbstractionSwComponentType),
}

impl AbstractionElement for SwComponentType {
    fn element(&self) -> &Element {
        match self {
            SwComponentType::Composition(comp) => comp.element(),
            SwComponentType::Application(app) => app.element(),
            SwComponentType::ComplexDeviceDriver(cdd) => cdd.element(),
            SwComponentType::Service(service) => service.element(),
            SwComponentType::SensorActuator(sensor_actuator) => sensor_actuator.element(),
            SwComponentType::EcuAbstraction(ecu_abstraction) => ecu_abstraction.element(),
        }
    }
}

impl IdentifiableAbstractionElement for SwComponentType {}

impl TryFrom<Element> for SwComponentType {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::CompositionSwComponentType => {
                Ok(SwComponentType::Composition(CompositionSwComponentType(element)))
            }
            ElementName::ApplicationSwComponentType => {
                Ok(SwComponentType::Application(ApplicationSwComponentType(element)))
            }
            ElementName::ComplexDeviceDriverSwComponentType => Ok(SwComponentType::ComplexDeviceDriver(
                ComplexDeviceDriverSwComponentType(element),
            )),
            ElementName::ServiceSwComponentType => Ok(SwComponentType::Service(ServiceSwComponentType(element))),
            ElementName::SensorActuatorSwComponentType => {
                Ok(SwComponentType::SensorActuator(SensorActuatorSwComponentType(element)))
            }
            ElementName::EcuAbstractionSwComponentType => {
                Ok(SwComponentType::EcuAbstraction(EcuAbstractionSwComponentType(element)))
            }
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "SwComponentType".to_string(),
            }),
        }
    }
}

impl From<CompositionSwComponentType> for SwComponentType {
    fn from(comp: CompositionSwComponentType) -> Self {
        SwComponentType::Composition(comp)
    }
}

impl From<ApplicationSwComponentType> for SwComponentType {
    fn from(app: ApplicationSwComponentType) -> Self {
        SwComponentType::Application(app)
    }
}

impl From<ComplexDeviceDriverSwComponentType> for SwComponentType {
    fn from(cdd: ComplexDeviceDriverSwComponentType) -> Self {
        SwComponentType::ComplexDeviceDriver(cdd)
    }
}

impl From<ServiceSwComponentType> for SwComponentType {
    fn from(service: ServiceSwComponentType) -> Self {
        SwComponentType::Service(service)
    }
}

impl From<SensorActuatorSwComponentType> for SwComponentType {
    fn from(sensor_actuator: SensorActuatorSwComponentType) -> Self {
        SwComponentType::SensorActuator(sensor_actuator)
    }
}

impl From<EcuAbstractionSwComponentType> for SwComponentType {
    fn from(ecu_abstraction: EcuAbstractionSwComponentType) -> Self {
        SwComponentType::EcuAbstraction(ecu_abstraction)
    }
}

impl AbstractSwComponentType for SwComponentType {}

//##################################################################

/// A `SwComponentPrototype` is an instance of a software component type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SwComponentPrototype(Element);
abstraction_element!(SwComponentPrototype, SwComponentPrototype);
impl IdentifiableAbstractionElement for SwComponentPrototype {}

impl SwComponentPrototype {
    fn new(
        name: &str,
        components: &Element,
        component_type: &SwComponentType,
    ) -> Result<Self, AutosarAbstractionError> {
        let component = components.create_named_sub_element(ElementName::SwComponentPrototype, name)?;
        component
            .create_sub_element(ElementName::TypeTref)?
            .set_reference_target(component_type.element())?;

        Ok(Self(component))
    }

    /// get the sw component type that this prototype is based on
    #[must_use]
    pub fn component_type(&self) -> Option<SwComponentType> {
        let component_elem = self
            .element()
            .get_sub_element(ElementName::TypeTref)?
            .get_reference_target()
            .ok()?;
        SwComponentType::try_from(component_elem).ok()
    }

    /// get the composition containing this component
    pub fn parent_composition(&self) -> Result<CompositionSwComponentType, AutosarAbstractionError> {
        let parent = self.element().named_parent()?.unwrap();
        CompositionSwComponentType::try_from(parent)
    }
}

//##################################################################

/// The `RootSwCompositionPrototype` is a special kind of `SwComponentPrototype` that represents the root of the composition hierarchy
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RootSwCompositionPrototype(Element);
abstraction_element!(RootSwCompositionPrototype, RootSwCompositionPrototype);
impl IdentifiableAbstractionElement for RootSwCompositionPrototype {}

impl RootSwCompositionPrototype {
    pub(crate) fn new(
        name: &str,
        root_compositions: &Element,
        composition_type: &CompositionSwComponentType,
    ) -> Result<Self, AutosarAbstractionError> {
        let root_composition =
            root_compositions.create_named_sub_element(ElementName::RootSwCompositionPrototype, name)?;
        root_composition
            .create_sub_element(ElementName::SoftwareCompositionTref)?
            .set_reference_target(composition_type.element())?;

        Ok(Self(root_composition))
    }

    /// get the composition that this root component is based on
    #[must_use]
    pub fn composition(&self) -> Option<CompositionSwComponentType> {
        let composition_elem = self
            .element()
            .get_sub_element(ElementName::SoftwareCompositionTref)?
            .get_reference_target()
            .ok()?;
        CompositionSwComponentType::try_from(composition_elem).ok()
    }
}

//##################################################################

/// The `ComponentPrototype` enum represents all possible types of software component prototypes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComponentPrototype {
    /// the component prototype is a `SwComponentPrototype`
    SwComponent(SwComponentPrototype),
    /// the component prototype is a `RootSwCompositionPrototype`
    RootComposition(RootSwCompositionPrototype),
}

impl AbstractionElement for ComponentPrototype {
    fn element(&self) -> &Element {
        match self {
            ComponentPrototype::SwComponent(swcp) => swcp.element(),
            ComponentPrototype::RootComposition(root) => root.element(),
        }
    }
}

impl TryFrom<Element> for ComponentPrototype {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::SwComponentPrototype => Ok(ComponentPrototype::SwComponent(SwComponentPrototype(element))),
            ElementName::RootSwCompositionPrototype => {
                Ok(ComponentPrototype::RootComposition(RootSwCompositionPrototype(element)))
            }
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "ComponentPrototype".to_string(),
            }),
        }
    }
}

impl IdentifiableAbstractionElement for ComponentPrototype {}

impl ComponentPrototype {
    #[must_use]
    /// get the sw component type that this prototype is based on
    pub fn component_type(&self) -> Option<SwComponentType> {
        match self {
            ComponentPrototype::SwComponent(swcp) => swcp.component_type(),
            ComponentPrototype::RootComposition(rc) => rc.composition().map(std::convert::Into::into),
        }
    }

    /// get the composition containing this component
    ///
    /// if the component is a root composition, this will always return None
    pub fn parent_composition(&self) -> Result<Option<CompositionSwComponentType>, AutosarAbstractionError> {
        match self {
            ComponentPrototype::SwComponent(swcp) => swcp.parent_composition().map(Some),
            ComponentPrototype::RootComposition(_) => Ok(None),
        }
    }
}

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::{AutosarModelAbstraction, SystemCategory};
    use autosar_data::AutosarVersion;

    #[test]
    fn software_compositions() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        let comp1 = CompositionSwComponentType::new("comp1", &package).unwrap();
        let comp2 = CompositionSwComponentType::new("comp2", &package).unwrap();
        let comp3 = CompositionSwComponentType::new("comp3", &package).unwrap();
        let comp4 = CompositionSwComponentType::new("comp4", &package).unwrap();

        comp1.create_component("comp2", &comp2.clone()).unwrap();
        comp2.create_component("comp3", &comp3.clone()).unwrap();
        comp3.create_component("comp4", &comp4.clone()).unwrap();

        assert_eq!(comp1.instances().len(), 0);
        assert_eq!(comp2.instances().len(), 1);
        assert_eq!(comp3.instances().len(), 1);
        assert_eq!(comp4.instances().len(), 1);

        assert!(comp1.is_parent_of(&comp2));
        assert!(comp1.is_parent_of(&comp3));
        assert!(comp1.is_parent_of(&comp4));

        assert!(!comp2.is_parent_of(&comp1));
        assert!(comp2.is_parent_of(&comp3));
        assert!(comp2.is_parent_of(&comp4));

        assert!(!comp3.is_parent_of(&comp1));
        assert!(!comp3.is_parent_of(&comp2));
        assert!(comp3.is_parent_of(&comp4));

        assert!(!comp4.is_parent_of(&comp1));
        assert!(!comp4.is_parent_of(&comp2));
        assert!(!comp4.is_parent_of(&comp3));
    }

    #[test]
    fn root_composition() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();
        let comp = CompositionSwComponentType::new("comp", &package).unwrap();
        let root_sw_component_prototype = system.set_root_sw_composition("root", &comp).unwrap();

        assert_eq!(
            ComponentPrototype::RootComposition(root_sw_component_prototype),
            comp.instances()[0]
        );
        assert_eq!(comp.instances().len(), 1);
    }

    #[test]
    fn components() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        let comp = CompositionSwComponentType::new("comp", &package).unwrap();
        let app = ApplicationSwComponentType::new("app", &package).unwrap();
        let cdd = ComplexDeviceDriverSwComponentType::new("cdd", &package).unwrap();
        let service = ServiceSwComponentType::new("service", &package).unwrap();
        let sensor_actuator = SensorActuatorSwComponentType::new("sensor_actuator", &package).unwrap();
        let ecu_abstraction = EcuAbstractionSwComponentType::new("ecu_abstraction", &package).unwrap();

        let container_comp = CompositionSwComponentType::new("container_comp", &package).unwrap();
        let comp_prototype = container_comp.create_component("comp", &comp.clone()).unwrap();
        let _app_prototype = container_comp.create_component("app", &app.clone()).unwrap();
        let _cdd_prototype = container_comp.create_component("cdd", &cdd.clone()).unwrap();
        let _service_prototype = container_comp.create_component("service", &service.clone()).unwrap();
        let _sensor_actuator_prototype = container_comp
            .create_component("sensor_actuator", &sensor_actuator.clone())
            .unwrap();
        let _ecu_abstraction_prototype = container_comp
            .create_component("ecu_abstraction", &ecu_abstraction.clone())
            .unwrap();

        assert_eq!(container_comp.components().count(), 6);
        let mut comp_prototype_iter = container_comp.components();
        assert_eq!(
            comp_prototype_iter.next().unwrap().component_type().unwrap(),
            comp.clone().into()
        );
        assert_eq!(
            comp_prototype_iter.next().unwrap().component_type().unwrap(),
            app.into()
        );
        assert_eq!(
            comp_prototype_iter.next().unwrap().component_type().unwrap(),
            cdd.into()
        );
        assert_eq!(
            comp_prototype_iter.next().unwrap().component_type().unwrap(),
            service.into()
        );
        assert_eq!(
            comp_prototype_iter.next().unwrap().component_type().unwrap(),
            sensor_actuator.into()
        );
        assert_eq!(
            comp_prototype_iter.next().unwrap().component_type().unwrap(),
            ecu_abstraction.into()
        );
        assert!(comp_prototype_iter.next().is_none());

        let component_prototype = ComponentPrototype::SwComponent(comp_prototype);
        assert_eq!(component_prototype.component_type().unwrap(), comp.into());
    }

    #[test]
    fn ports_and_connectors() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        // create some components:
        // comp_parent contains comp_child, swc1, swc2
        let comp_parent_type = package.create_composition_sw_component_type("comp_parent").unwrap();
        let comp_child_type = package.create_composition_sw_component_type("comp_child").unwrap();
        let swc_type = package.create_application_sw_component_type("swc_type1").unwrap();

        let comp_child_proto = comp_parent_type.create_component("comp2", &comp_child_type).unwrap();
        let swc_proto = comp_parent_type.create_component("swc1", &swc_type).unwrap();

        // create port interfaces: S/R and C/S
        let port_interface_sr = package.create_sender_receiver_interface("sr").unwrap();
        let port_interface_cs = package.create_client_server_interface("cs").unwrap();

        // connect S/R ports:
        // - comp_parent R port to swc R port (delegation)
        // - swc P port to comp_child R port (assembly)
        // - comp_child R port to comp_child p port (passthrough)
        let comp_parent_r_port = comp_parent_type.create_r_port("port_r", &port_interface_sr).unwrap();
        let swc_r_port = swc_type.create_r_port("port_r", &port_interface_sr).unwrap();
        let swc_p_port = swc_type.create_p_port("port_p", &port_interface_sr).unwrap();
        let comp_child_r_port = comp_child_type.create_r_port("port_r", &port_interface_sr).unwrap();
        let comp_child_p_port = comp_child_type.create_p_port("port_p", &port_interface_sr).unwrap();

        comp_parent_type
            .create_delegation_connector("sr_delegation", &swc_r_port, &swc_proto, &comp_parent_r_port)
            .unwrap();
        comp_parent_type
            .create_assembly_connector(
                "sr_assembly",
                &swc_p_port,
                &swc_proto,
                &comp_child_r_port,
                &comp_child_proto,
            )
            .unwrap();
        comp_child_type
            .create_pass_through_connector("sr_passthrough", &comp_child_r_port, &comp_child_p_port)
            .unwrap();

        // connect C/S ports:
        // - comp_parent S port to swc S port (delegation)
        // - swc C port to comp_child S port (assembly)
        // - comp_child S port to comp_child C port (passthrough)
        let comp_parent_s_port = comp_parent_type.create_p_port("port_s", &port_interface_cs).unwrap();
        let swc_s_port = swc_type.create_p_port("port_s", &port_interface_cs).unwrap();
        let swc_c_port = swc_type.create_r_port("port_c", &port_interface_cs).unwrap();
        let comp_child_s_port = comp_child_type.create_p_port("port_s", &port_interface_cs).unwrap();
        let comp_child_c_port = comp_child_type.create_r_port("port_c", &port_interface_cs).unwrap();

        comp_parent_type
            .create_delegation_connector("cs_delegation", &swc_s_port, &swc_proto, &comp_parent_s_port)
            .unwrap();
        comp_parent_type
            .create_assembly_connector(
                "cs_assembly",
                &swc_c_port,
                &swc_proto,
                &comp_child_s_port,
                &comp_child_proto,
            )
            .unwrap();
        comp_child_type
            .create_pass_through_connector("cs_passthrough", &comp_child_s_port, &comp_child_c_port)
            .unwrap();

        // check the connectors
        let mut parent_connectors = comp_parent_type.connectors();
        assert_eq!(parent_connectors.next().unwrap().name().unwrap(), "sr_delegation");
        assert_eq!(parent_connectors.next().unwrap().name().unwrap(), "sr_assembly");
        assert_eq!(parent_connectors.next().unwrap().name().unwrap(), "cs_delegation");
        assert_eq!(parent_connectors.next().unwrap().name().unwrap(), "cs_assembly");
        assert!(parent_connectors.next().is_none());

        let mut child_connectors = comp_child_type.connectors();
        assert_eq!(child_connectors.next().unwrap().name().unwrap(), "sr_passthrough");
        assert_eq!(child_connectors.next().unwrap().name().unwrap(), "cs_passthrough");
        assert!(child_connectors.next().is_none());

        // create a port group
        comp_parent_type.create_port_group("group").unwrap();
    }
}
