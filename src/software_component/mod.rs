//! Software component types and compositions
//!
//! This module contains the definition of software component types and instances.
//! It also contains the definition of the composition hierarchy, and the connectors between components.

use crate::{
    abstraction_element, datatype, element_iterator, reflist_iterator, AbstractionElement, ArPackage,
    AutosarAbstractionError, Element,
};
use autosar_data::ElementName;
use datatype::DataTypeMappingSet;

mod connector;
mod interface;
mod port;

pub use connector::*;
pub use interface::*;
pub use port::*;

//##################################################################

/// The `AbstractSwComponentType` is the common interface for all types of software components
pub trait AbstractSwComponentType: AbstractionElement {
    /// iterator over the instances of the component type
    fn instances(&self) -> ComponentPrototypeIterator {
        let model_result = self.element().model();
        let path_result = self.element().path();
        if let (Ok(model), Ok(path)) = (model_result, path_result) {
            let reflist = model.get_references_to(&path);
            ComponentPrototypeIterator::new(reflist)
        } else {
            ComponentPrototypeIterator::new(vec![])
        }
    }

    /// iterator over all compositions containing instances of the component type
    fn parent_compositions(&self) -> impl Iterator<Item = CompositionSwComponentType> {
        self.instances()
            .filter_map(|swcp| swcp.element().named_parent().ok().flatten())
            .filter_map(|elem| CompositionSwComponentType::try_from(elem).ok())
    }

    /// add a data type mapping to the SWC, by referencing an existing `DataTypeMappingSet`
    fn add_data_type_mapping(&self, data_type_mapping_set: &DataTypeMappingSet) -> Result<(), AutosarAbstractionError> {
        // this default implementation applies to component variants that have internal behaviors.
        // specifically, this means that it is NOT valid for the CompositionSwComponentType
        let name = self.name().unwrap();
        let data_type_mapping_refs = self
            .element()
            .get_or_create_sub_element(ElementName::InternalBehaviors)?
            .get_or_create_named_sub_element(ElementName::SwcInternalBehavior, &format!("{name}_InternalBehavior"))?
            .get_or_create_sub_element(ElementName::DataTypeMappingRefs)?;
        data_type_mapping_refs
            .create_sub_element(ElementName::DataTypeMappingRef)?
            .set_reference_target(data_type_mapping_set.element())?;
        Ok(())
    }

    /// create a new required port with the given name and port interface
    fn create_r_port<T: Into<PortInterface> + AbstractionElement>(
        &self,
        name: &str,
        port_interface: &T,
    ) -> Result<RPortPrototype, AutosarAbstractionError> {
        let ports = self.element().get_or_create_sub_element(ElementName::Ports)?;
        RPortPrototype::new(name, &ports, port_interface)
    }

    /// create a new provided port with the given name and port interface
    fn create_p_port<T: Into<PortInterface> + AbstractionElement>(
        &self,
        name: &str,
        port_interface: &T,
    ) -> Result<PPortPrototype, AutosarAbstractionError> {
        let ports = self.element().get_or_create_sub_element(ElementName::Ports)?;
        PPortPrototype::new(name, &ports, port_interface)
    }

    /// create a new provided required port with the given name and port interface
    fn create_pr_port<T: Into<PortInterface> + AbstractionElement>(
        &self,
        name: &str,
        port_interface: &T,
    ) -> Result<PRPortPrototype, AutosarAbstractionError> {
        let ports = self.element().get_or_create_sub_element(ElementName::Ports)?;
        PRPortPrototype::new(name, &ports, port_interface)
    }

    /// get an iterator over the ports of the component
    fn ports(&self) -> PortPrototypeIterator {
        PortPrototypeIterator::new(self.element().get_sub_element(ElementName::Ports))
    }

    /// create a new port group
    fn create_port_group(&self, name: &str) -> Result<PortGroup, AutosarAbstractionError> {
        let port_groups = self.element().get_or_create_sub_element(ElementName::PortGroups)?;
        PortGroup::new(name, &port_groups)
    }
}

//##################################################################

/// A `CompositionSwComponentType` is a software component that contains other software components
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompositionSwComponentType(Element);
abstraction_element!(CompositionSwComponentType, CompositionSwComponentType);

impl CompositionSwComponentType {
    /// create a new composition component with the given name
    pub fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let composition = elements.create_named_sub_element(ElementName::CompositionSwComponentType, name)?;
        Ok(Self(composition))
    }

    /// check if the composition is a parent (or grand-parent, etc.) of the component
    pub fn is_parent_of<T: AbstractSwComponentType>(&self, other: &T) -> bool {
        // the expectation is that in normal cases each component has only one parent
        // additionally there should never be any cycles in the composition hierarchy
        let mut work_items = other.parent_compositions().collect::<Vec<_>>();
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
    pub fn components(&self) -> impl Iterator<Item = SwComponentType> {
        CompositionComponentsIter::new(self.element().get_sub_element(ElementName::Components))
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
}

impl AbstractSwComponentType for CompositionSwComponentType {
    /// add a data type mapping, by referencing an existing `DataTypeMappingSet`
    fn add_data_type_mapping(&self, data_type_mapping_set: &DataTypeMappingSet) -> Result<(), AutosarAbstractionError> {
        let data_type_mapping_refs = self
            .element()
            .get_or_create_sub_element(ElementName::DataTypeMappingRefs)?;
        data_type_mapping_refs
            .create_sub_element(ElementName::DataTypeMappingRef)?
            .set_reference_target(data_type_mapping_set.element())?;
        Ok(())
    }
}

//##################################################################

/// An `ApplicationSwComponentType` is a software component that provides application functionality
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApplicationSwComponentType(Element);
abstraction_element!(ApplicationSwComponentType, ApplicationSwComponentType);

impl ApplicationSwComponentType {
    /// create a new application component with the given name
    pub fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let application = elements.create_named_sub_element(ElementName::ApplicationSwComponentType, name)?;
        Ok(Self(application))
    }
}

impl AbstractSwComponentType for ApplicationSwComponentType {}

//##################################################################

/// A `ComplexDeviceDriverSwComponentType` is a software component that provides complex device driver functionality
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComplexDeviceDriverSwComponentType(Element);
abstraction_element!(ComplexDeviceDriverSwComponentType, ComplexDeviceDriverSwComponentType);

impl ComplexDeviceDriverSwComponentType {
    /// create a new complex device driver component with the given name
    pub fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let cdd = elements.create_named_sub_element(ElementName::ComplexDeviceDriverSwComponentType, name)?;
        Ok(Self(cdd))
    }
}

impl AbstractSwComponentType for ComplexDeviceDriverSwComponentType {}

//##################################################################

/// `ServiceSwComponentType` is used for configuring services for a given ECU. Instances of this class should only
/// be created in ECU Configuration phase for the specific purpose of the service configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceSwComponentType(Element);
abstraction_element!(ServiceSwComponentType, ServiceSwComponentType);

impl ServiceSwComponentType {
    /// create a new service component with the given name
    pub fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let service = elements.create_named_sub_element(ElementName::ServiceSwComponentType, name)?;
        Ok(Self(service))
    }
}

impl AbstractSwComponentType for ServiceSwComponentType {}

//##################################################################

/// `SensorActuatorSwComponentType` is used to connect sensor/acutator devices to the ECU configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SensorActuatorSwComponentType(Element);
abstraction_element!(SensorActuatorSwComponentType, SensorActuatorSwComponentType);

impl SensorActuatorSwComponentType {
    /// create a new sensor/actuator component with the given name
    pub fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let sensor_actuator = elements.create_named_sub_element(ElementName::SensorActuatorSwComponentType, name)?;
        Ok(Self(sensor_actuator))
    }
}

impl AbstractSwComponentType for SensorActuatorSwComponentType {}

//##################################################################

/// The `ECUAbstraction` is a special `AtomicSwComponentType` that resides between a software-component
/// that wants to access ECU periphery and the Microcontroller Abstraction
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcuAbstractionSwComponentType(Element);
abstraction_element!(EcuAbstractionSwComponentType, EcuAbstractionSwComponentType);

impl EcuAbstractionSwComponentType {
    /// create a new ECU abstraction component with the given name
    pub fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let ecu_abstraction = elements.create_named_sub_element(ElementName::EcuAbstractionSwComponentType, name)?;
        Ok(Self(ecu_abstraction))
    }
}

impl AbstractSwComponentType for EcuAbstractionSwComponentType {}

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

impl AbstractSwComponentType for SwComponentType {
    fn add_data_type_mapping(&self, data_type_mapping_set: &DataTypeMappingSet) -> Result<(), AutosarAbstractionError> {
        match self {
            SwComponentType::Composition(comp) => comp.add_data_type_mapping(data_type_mapping_set),
            SwComponentType::Application(app) => app.add_data_type_mapping(data_type_mapping_set),
            SwComponentType::ComplexDeviceDriver(cdd) => cdd.add_data_type_mapping(data_type_mapping_set),
            SwComponentType::Service(service) => service.add_data_type_mapping(data_type_mapping_set),
            SwComponentType::SensorActuator(sensor_actuator) => {
                sensor_actuator.add_data_type_mapping(data_type_mapping_set)
            }
            SwComponentType::EcuAbstraction(ecu_abstraction) => {
                ecu_abstraction.add_data_type_mapping(data_type_mapping_set)
            }
        }
    }
}

//##################################################################

/// A `SwComponentPrototype` is an instance of a software component type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SwComponentPrototype(Element);
abstraction_element!(SwComponentPrototype, SwComponentPrototype);

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

element_iterator!(CompositionComponentsIter, SwComponentType, Some);

//##################################################################

reflist_iterator!(ComponentPrototypeIterator, ComponentPrototype);

//##################################################################

element_iterator!(PortPrototypeIterator, PortPrototype, Some);

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::{System, SystemCategory};
    use autosar_data::{AutosarModel, AutosarVersion};

    #[test]
    fn software_compositions() {
        let model = AutosarModel::new();
        let _file = model.create_file("filename", AutosarVersion::LATEST).unwrap();
        let package = ArPackage::get_or_create(&model, "/package").unwrap();

        let comp1 = CompositionSwComponentType::new("comp1", &package).unwrap();
        let comp2 = CompositionSwComponentType::new("comp2", &package).unwrap();
        let comp3 = CompositionSwComponentType::new("comp3", &package).unwrap();
        let comp4 = CompositionSwComponentType::new("comp4", &package).unwrap();

        comp1.create_component("comp2", &comp2.clone()).unwrap();
        comp2.create_component("comp3", &comp3.clone()).unwrap();
        comp3.create_component("comp4", &comp4.clone()).unwrap();

        assert_eq!(comp1.instances().count(), 0);
        assert_eq!(comp2.instances().count(), 1);
        assert_eq!(comp3.instances().count(), 1);
        assert_eq!(comp4.instances().count(), 1);

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
        let model = AutosarModel::new();
        let _file = model.create_file("filename", AutosarVersion::LATEST).unwrap();
        let package = ArPackage::get_or_create(&model, "/package").unwrap();

        let system = System::new("system", &package, SystemCategory::EcuExtract).unwrap();
        let comp = CompositionSwComponentType::new("comp", &package).unwrap();
        let root_sw_component_prototype = system.set_root_sw_composition("root", &comp).unwrap();

        assert_eq!(
            ComponentPrototype::RootComposition(root_sw_component_prototype),
            comp.instances().next().unwrap()
        );
        assert_eq!(comp.instances().count(), 1);
    }

    #[test]
    fn data_type_mapping() {
        let model = AutosarModel::new();
        let _file = model.create_file("filename", AutosarVersion::LATEST).unwrap();
        let package = ArPackage::get_or_create(&model, "/package").unwrap();

        let mapping_set = DataTypeMappingSet::new("mapping_set", &package).unwrap();
        let composition = CompositionSwComponentType::new("comp", &package).unwrap();
        composition.add_data_type_mapping(&mapping_set).unwrap();

        let app = ApplicationSwComponentType::new("app", &package).unwrap();
        app.add_data_type_mapping(&mapping_set).unwrap();

        let cdd = ComplexDeviceDriverSwComponentType::new("cdd", &package).unwrap();
        cdd.add_data_type_mapping(&mapping_set).unwrap();

        let service = ServiceSwComponentType::new("service", &package).unwrap();
        service.add_data_type_mapping(&mapping_set).unwrap();

        let sensor_actuator = SensorActuatorSwComponentType::new("sensor_actuator", &package).unwrap();
        sensor_actuator.add_data_type_mapping(&mapping_set).unwrap();

        let ecu_abstraction = EcuAbstractionSwComponentType::new("ecu_abstraction", &package).unwrap();
        ecu_abstraction.add_data_type_mapping(&mapping_set).unwrap();
    }

    #[test]
    fn components() {
        let model = AutosarModel::new();
        let _file = model.create_file("filename", AutosarVersion::LATEST).unwrap();
        let package = ArPackage::get_or_create(&model, "/package").unwrap();

        let comp = CompositionSwComponentType::new("comp", &package).unwrap();
        let app = ApplicationSwComponentType::new("app", &package).unwrap();
        let cdd = ComplexDeviceDriverSwComponentType::new("cdd", &package).unwrap();
        let service = ServiceSwComponentType::new("service", &package).unwrap();
        let sensor_actuator = SensorActuatorSwComponentType::new("sensor_actuator", &package).unwrap();
        let ecu_abstraction = EcuAbstractionSwComponentType::new("ecu_abstraction", &package).unwrap();

        let container_comp = CompositionSwComponentType::new("container_comp", &package).unwrap();
        let _comp_prototype = container_comp.create_component("comp", &comp.clone()).unwrap();
        let _app_prototype = container_comp.create_component("app", &app.clone()).unwrap();
        let _cdd_prototype = container_comp.create_component("cdd", &cdd.clone()).unwrap();
        let _service_prototype = container_comp.create_component("service", &service.clone()).unwrap();
        let _sensor_actuator_prototype = container_comp
            .create_component("sensor_actuator", &sensor_actuator.clone())
            .unwrap();
        let _ecu_abstraction_prototype = container_comp
            .create_component("ecu_abstraction", &ecu_abstraction.clone())
            .unwrap();
    }
}
