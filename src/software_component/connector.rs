use crate::{
    AbstractionElement, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element,
    software_component,
};
use autosar_data::{Element, ElementName};
use software_component::{PortInterface, PortPrototype, SwComponentPrototype};

//#########################################################

/// A `DelegationSwConnector` connects a port of a software component that is contained inside a `SwCompositionType` with a port of the `SwCompositionType`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DelegationSwConnector(Element);
abstraction_element!(DelegationSwConnector, DelegationSwConnector);
impl IdentifiableAbstractionElement for DelegationSwConnector {}

impl DelegationSwConnector {
    /// Create a new `DelegationSwConnector`
    ///
    /// # Arguments
    /// `inner_port`: A port of a software component that is contained inside the `SwCompositionType`.
    /// `outer_port`: A port of the `SwCompositionType`.
    pub(crate) fn new(
        name: &str,
        parent_element: &Element,
        inner_port: &PortPrototype,
        inner_sw_prototype: &SwComponentPrototype,
        outer_port: &PortPrototype,
    ) -> Result<Self, AutosarAbstractionError> {
        let inner_port_interface = inner_port.port_interface()?;
        // the caller (CompositionSwComponentType::add_connector) ensures that the inner and outer port both have the same kind of interface
        match &inner_port_interface {
            PortInterface::SenderReceiverInterface(_) | PortInterface::NvDataInterface(_) => {
                if (matches!(inner_port, PortPrototype::R(_)) && matches!(outer_port, PortPrototype::P(_)))
                    || (matches!(inner_port, PortPrototype::P(_)) && matches!(outer_port, PortPrototype::R(_)))
                {
                    return Err(AutosarAbstractionError::InvalidParameter(
                        "Invalid combination of provide and require ports".to_string(),
                    ));
                }
            }
            PortInterface::ClientServerInterface(_)
            | PortInterface::ModeSwitchInterface(_)
            | PortInterface::TriggerInterface(_) => {
                // table of valid combinations; Table 4.14 in AUTOSAR_TPS_SoftwareComponentTemplate.pdf:
                // inner_port\outer_port | P   | R   | PR
                // P                     | yes | no  | no
                // R                     | no  | yes | no
                // PR                    | no  | yes | no
                match (inner_port, outer_port) {
                    (PortPrototype::P(_) | PortPrototype::PR(_), PortPrototype::P(_))
                    | (PortPrototype::R(_), PortPrototype::R(_)) => {}
                    _ => {
                        return Err(AutosarAbstractionError::InvalidParameter(
                            "Invalid combination of provide and require ports".to_string(),
                        ));
                    }
                }
            }
            PortInterface::ParameterInterface(_) => { /* no specific restrictions on ParameterInterfaces */ }
        }

        let delegation_sw_connector =
            parent_element.create_named_sub_element(ElementName::DelegationSwConnector, name)?;
        let inner_iref = delegation_sw_connector.create_sub_element(ElementName::InnerPortIref)?;
        // let inner_swc = SwComponentType::try_from(inner_port.element().named_parent()?.unwrap())?;
        // The inner port is either described by an RPortInCompositionInstanceRef or a PPortInCompositionInstanceRef, depending on the port type.
        // If the inner port is a PRPortPrototype, the inner port is described by an PPortInCompositionInstanceRef, as required by [TPS_SWCT_01515].
        if matches!(inner_port, PortPrototype::R(_)) {
            // inner port is a required port
            let r_port_in_instance = inner_iref.create_sub_element(ElementName::RPortInCompositionInstanceRef)?;
            r_port_in_instance
                .create_sub_element(ElementName::TargetRPortRef)?
                .set_reference_target(inner_port.element())?;
            r_port_in_instance
                .create_sub_element(ElementName::ContextComponentRef)?
                .set_reference_target(inner_sw_prototype.element())?;
        } else {
            // inner port is a provided port or a PR port
            let p_port_in_instance = inner_iref.create_sub_element(ElementName::PPortInCompositionInstanceRef)?;
            p_port_in_instance
                .create_sub_element(ElementName::TargetPPortRef)?
                .set_reference_target(inner_port.element())?;
            p_port_in_instance
                .create_sub_element(ElementName::ContextComponentRef)?
                .set_reference_target(inner_sw_prototype.element())?;
        };
        delegation_sw_connector
            .create_sub_element(ElementName::OuterPortRef)?
            .set_reference_target(outer_port.element())?;

        Ok(Self(delegation_sw_connector))
    }

    /// Get the inner port of the delegation connector
    #[must_use]
    pub fn inner_port(&self) -> Option<PortPrototype> {
        let inner_port_iref = self.element().get_sub_element(ElementName::InnerPortIref)?;
        if let Some(r_port_in_instance) = inner_port_iref.get_sub_element(ElementName::RPortInCompositionInstanceRef) {
            let inner_port_elem = r_port_in_instance
                .get_sub_element(ElementName::TargetRPortRef)?
                .get_reference_target()
                .ok()?;
            PortPrototype::try_from(inner_port_elem).ok()
        } else if let Some(p_port_in_instance) =
            inner_port_iref.get_sub_element(ElementName::PPortInCompositionInstanceRef)
        {
            let inner_port_elem = p_port_in_instance
                .get_sub_element(ElementName::TargetPPortRef)?
                .get_reference_target()
                .ok()?;
            PortPrototype::try_from(inner_port_elem).ok()
        } else {
            None
        }
    }

    /// Get the component containing the inner port of the delegation connector
    #[must_use]
    pub fn inner_sw_component(&self) -> Option<SwComponentPrototype> {
        let inner_port_iref = self.element().get_sub_element(ElementName::InnerPortIref)?;
        if let Some(r_port_in_instance) = inner_port_iref.get_sub_element(ElementName::RPortInCompositionInstanceRef) {
            let inner_sw_component_elem = r_port_in_instance
                .get_sub_element(ElementName::ContextComponentRef)?
                .get_reference_target()
                .ok()?;
            SwComponentPrototype::try_from(inner_sw_component_elem).ok()
        } else if let Some(p_port_in_instance) =
            inner_port_iref.get_sub_element(ElementName::PPortInCompositionInstanceRef)
        {
            let inner_sw_component_elem = p_port_in_instance
                .get_sub_element(ElementName::ContextComponentRef)?
                .get_reference_target()
                .ok()?;
            SwComponentPrototype::try_from(inner_sw_component_elem).ok()
        } else {
            None
        }
    }

    /// Get the outer port of the delegation connector
    #[must_use]
    pub fn outer_port(&self) -> Option<PortPrototype> {
        let outer_port_elem = self
            .element()
            .get_sub_element(ElementName::OuterPortRef)?
            .get_reference_target()
            .ok()?;
        PortPrototype::try_from(outer_port_elem).ok()
    }
}

//##################################################################

/// An `AssemblySwConnector` connects ports of two `SwCompositionType`s.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssemblySwConnector(Element);
abstraction_element!(AssemblySwConnector, AssemblySwConnector);
impl IdentifiableAbstractionElement for AssemblySwConnector {}

impl AssemblySwConnector {
    /// Create a new `AssemblySwConnector`
    pub(crate) fn new(
        name: &str,
        parent_element: &Element,
        port_1: &PortPrototype,
        sw_prototype_1: &SwComponentPrototype,
        port_2: &PortPrototype,
        sw_prototype_2: &SwComponentPrototype,
    ) -> Result<Self, AutosarAbstractionError> {
        // find the provider and requester port; also filter out invalid combinations such as P-P and R-R
        let (provider, p_proto_swc, requester, r_proto_swc) = match (port_1, port_2) {
            (PortPrototype::P(_), PortPrototype::R(_)) => (port_1, sw_prototype_1, port_2, sw_prototype_2),
            (PortPrototype::R(_), PortPrototype::P(_)) => (port_2, sw_prototype_2, port_1, sw_prototype_1),
            (PortPrototype::P(_), PortPrototype::PR(_)) => (port_1, sw_prototype_1, port_2, sw_prototype_2),
            (PortPrototype::PR(_), PortPrototype::P(_)) => (port_2, sw_prototype_2, port_1, sw_prototype_1),
            (PortPrototype::R(_), PortPrototype::PR(_)) => (port_2, sw_prototype_2, port_1, sw_prototype_1),
            (PortPrototype::PR(_), PortPrototype::R(_)) => (port_1, sw_prototype_1, port_2, sw_prototype_2),
            (PortPrototype::PR(_), PortPrototype::PR(_)) => (port_1, sw_prototype_1, port_2, sw_prototype_2),
            _ => {
                return Err(AutosarAbstractionError::InvalidParameter(
                    "Invalid port roles".to_string(),
                ));
            }
        };

        let port_interface = provider.port_interface()?;
        // additional restrictions beyond the basic rules in the match above
        // apply to ClientServer, ModeSwitch, and Trigger interfaces
        if matches!(
            &port_interface,
            PortInterface::ClientServerInterface(_)
                | PortInterface::ModeSwitchInterface(_)
                | PortInterface::TriggerInterface(_)
        ) {
            // can only connect R to P or PR. All other combinations are forbidden.
            // As a result of the match above, we know that provider is P or PR and requester is R or PR.
            if !matches!(requester, PortPrototype::R(_)) {
                return Err(AutosarAbstractionError::InvalidParameter(
                    "Invalid combination of provide and require ports".to_string(),
                ));
            }
        }

        let assembly_sw_connector = parent_element.create_named_sub_element(ElementName::AssemblySwConnector, name)?;

        let provider_iref = assembly_sw_connector.create_sub_element(ElementName::ProviderIref)?;
        provider_iref
            .create_sub_element(ElementName::TargetPPortRef)?
            .set_reference_target(provider.element())?;
        provider_iref
            .create_sub_element(ElementName::ContextComponentRef)?
            .set_reference_target(p_proto_swc.element())?;
        let requester_iref = assembly_sw_connector.create_sub_element(ElementName::RequesterIref)?;
        requester_iref
            .create_sub_element(ElementName::TargetRPortRef)?
            .set_reference_target(requester.element())?;
        requester_iref
            .create_sub_element(ElementName::ContextComponentRef)?
            .set_reference_target(r_proto_swc.element())?;

        Ok(Self(assembly_sw_connector))
    }

    /// Get the provider port of the assembly connector
    #[must_use]
    pub fn p_port(&self) -> Option<PortPrototype> {
        let provider_iref = self.element().get_sub_element(ElementName::ProviderIref)?;
        let provider_port_elem = provider_iref
            .get_sub_element(ElementName::TargetPPortRef)?
            .get_reference_target()
            .ok()?;
        PortPrototype::try_from(provider_port_elem).ok()
    }

    /// get the component containing the `p_port` of the assembly connector
    #[must_use]
    pub fn p_sw_component(&self) -> Option<SwComponentPrototype> {
        let provider_iref = self.element().get_sub_element(ElementName::ProviderIref)?;
        let provider_swc_elem = provider_iref
            .get_sub_element(ElementName::ContextComponentRef)?
            .get_reference_target()
            .ok()?;
        SwComponentPrototype::try_from(provider_swc_elem).ok()
    }

    /// Get the requester port of the assembly connector
    #[must_use]
    pub fn r_port(&self) -> Option<PortPrototype> {
        let requester_iref = self.element().get_sub_element(ElementName::RequesterIref)?;
        let requester_port_elem = requester_iref
            .get_sub_element(ElementName::TargetRPortRef)?
            .get_reference_target()
            .ok()?;
        PortPrototype::try_from(requester_port_elem).ok()
    }

    /// get the component containing the `r_port` of the assembly connector
    #[must_use]
    pub fn r_sw_component(&self) -> Option<SwComponentPrototype> {
        let requester_iref = self.element().get_sub_element(ElementName::RequesterIref)?;
        let requester_swc_elem = requester_iref
            .get_sub_element(ElementName::ContextComponentRef)?
            .get_reference_target()
            .ok()?;
        SwComponentPrototype::try_from(requester_swc_elem).ok()
    }
}

//##################################################################

/// A `PassThroughSwConnector` connects two ports of a `SwCompositionType`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PassThroughSwConnector(Element);
abstraction_element!(PassThroughSwConnector, PassThroughSwConnector);
impl IdentifiableAbstractionElement for PassThroughSwConnector {}

impl PassThroughSwConnector {
    /// Create a new `PassThroughSwConnector`
    pub(crate) fn new(
        name: &str,
        parent_element: &Element,
        port_1: &PortPrototype,
        port_2: &PortPrototype,
    ) -> Result<Self, AutosarAbstractionError> {
        let (provided_port, required_port) = match (&port_1, &port_2) {
            (PortPrototype::P(_), PortPrototype::R(_)) => (port_1, port_2),
            (PortPrototype::R(_), PortPrototype::P(_)) => (port_2, port_1),
            (PortPrototype::P(_), PortPrototype::PR(_)) => (port_1, port_2),
            (PortPrototype::PR(_), PortPrototype::P(_)) => (port_2, port_1),
            (PortPrototype::R(_), PortPrototype::PR(_)) => (port_2, port_1),
            (PortPrototype::PR(_), PortPrototype::R(_)) => (port_1, port_2),
            (PortPrototype::PR(_), PortPrototype::PR(_)) => (port_1, port_2),
            _ => {
                return Err(AutosarAbstractionError::InvalidParameter(
                    "Invalid port roles".to_string(),
                ));
            }
        };

        let pass_through_sw_connector =
            parent_element.create_named_sub_element(ElementName::PassThroughSwConnector, name)?;

        pass_through_sw_connector
            .create_sub_element(ElementName::ProvidedOuterPortRef)?
            .set_reference_target(provided_port.element())?;
        pass_through_sw_connector
            .create_sub_element(ElementName::RequiredOuterPortRef)?
            .set_reference_target(required_port.element())?;

        Ok(Self(pass_through_sw_connector))
    }

    /// Get the provided port of the pass-through connector
    #[must_use]
    pub fn p_port(&self) -> Option<PortPrototype> {
        let provided_port_elem = self
            .element()
            .get_sub_element(ElementName::ProvidedOuterPortRef)?
            .get_reference_target()
            .ok()?;
        PortPrototype::try_from(provided_port_elem).ok()
    }

    /// Get the required port of the pass-through connector
    #[must_use]
    pub fn r_port(&self) -> Option<PortPrototype> {
        let required_port_elem = self
            .element()
            .get_sub_element(ElementName::RequiredOuterPortRef)?
            .get_reference_target()
            .ok()?;
        PortPrototype::try_from(required_port_elem).ok()
    }
}

//##################################################################

/// A `SwConnector` is a generic enum that can represent any kind of software connector.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SwConnector {
    /// The connector is a `DelegationSwConnector`
    Delegation(DelegationSwConnector),
    /// The connector is an `AssemblySwConnector`
    Assembly(AssemblySwConnector),
    /// The connector is a `PassThroughSwConnector`
    PassThrough(PassThroughSwConnector),
}

impl AbstractionElement for SwConnector {
    fn element(&self) -> &Element {
        match self {
            SwConnector::Delegation(connector) => connector.element(),
            SwConnector::Assembly(connector) => connector.element(),
            SwConnector::PassThrough(connector) => connector.element(),
        }
    }
}

impl TryFrom<Element> for SwConnector {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::DelegationSwConnector => Ok(SwConnector::Delegation(DelegationSwConnector(element))),
            ElementName::AssemblySwConnector => Ok(SwConnector::Assembly(AssemblySwConnector(element))),
            ElementName::PassThroughSwConnector => Ok(SwConnector::PassThrough(PassThroughSwConnector(element))),
            _ => Err(AutosarAbstractionError::ConversionError {
                element: element.clone(),
                dest: "SwConnector".to_string(),
            }),
        }
    }
}

impl IdentifiableAbstractionElement for SwConnector {}

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::AutosarModelAbstraction;
    use autosar_data::{AutosarVersion, ElementName};
    use software_component::{
        AbstractSwComponentType, ApplicationSwComponentType, ClientServerInterface, CompositionSwComponentType,
        SenderReceiverInterface,
    };

    #[test]
    fn test_delegation_sw_connector() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        // create interfaces for the ports
        let sr_interface = package.create_sender_receiver_interface("sr_interface").unwrap();
        let cs_interface = package.create_client_server_interface("cs_interface").unwrap();

        // create a composition and an application sw component type
        let composition = package.create_composition_sw_component_type("composition").unwrap();
        let swc_type = package.create_application_sw_component_type("app_type").unwrap();

        // create multiple ports with different interfaces and directions
        let outer_sr_p_port = composition.create_p_port("outer_sr_p_port", &sr_interface).unwrap();
        let inner_sr_p_port = swc_type.create_p_port("inner_sr_p_port", &sr_interface).unwrap();
        let outer_sr_r_port = composition.create_r_port("outer_sr_r_port", &sr_interface).unwrap();
        let inner_sr_r_port = swc_type.create_r_port("inner_sr_r_port", &sr_interface).unwrap();
        let outer_sr_pr_port = composition.create_pr_port("outer_sr_pr_port", &sr_interface).unwrap();
        let inner_sr_pr_port = swc_type.create_pr_port("inner_sr_pr_port", &sr_interface).unwrap();

        let outer_cs_p_port = composition.create_p_port("outer_cs_p_port", &cs_interface).unwrap();
        let inner_cs_p_port = swc_type.create_p_port("inner_cs_p_port", &cs_interface).unwrap();
        let outer_cs_r_port = composition.create_r_port("outer_cs_r_port", &cs_interface).unwrap();
        let inner_cs_r_port = swc_type.create_r_port("inner_cs_r_port", &cs_interface).unwrap();
        let outer_cs_pr_port = composition.create_pr_port("outer_cs_pr_port", &cs_interface).unwrap();
        let inner_cs_pr_port = swc_type.create_pr_port("inner_cs_pr_port", &cs_interface).unwrap();

        // add the application sw component type to the composition
        let app_prototype = composition.create_component("app_prototype", &swc_type).unwrap();

        // connect the outer port of the composition with the inner port of the application sw component type
        let sr_p_connector = composition
            .create_delegation_connector("sr_p_connector", &inner_sr_p_port, &app_prototype, &outer_sr_p_port)
            .unwrap();
        assert_eq!(sr_p_connector.inner_port().unwrap(), inner_sr_p_port.clone().into());
        assert_eq!(sr_p_connector.outer_port().unwrap(), outer_sr_p_port.clone().into());
        assert_eq!(sr_p_connector.inner_sw_component().unwrap(), app_prototype);
        let sr_r_connector = composition
            .create_delegation_connector("sr_r_connector", &inner_sr_r_port, &app_prototype, &outer_sr_r_port)
            .unwrap();
        assert_eq!(sr_r_connector.inner_port().unwrap(), inner_sr_r_port.clone().into());
        assert_eq!(sr_r_connector.outer_port().unwrap(), outer_sr_r_port.clone().into());
        assert_eq!(sr_r_connector.inner_sw_component().unwrap(), app_prototype);
        let _sr_pr_connector = composition
            .create_delegation_connector("sr_pr_connector", &inner_sr_pr_port, &app_prototype, &outer_sr_pr_port)
            .unwrap();
        let _cs_p_connector = composition
            .create_delegation_connector("cs_p_connector", &inner_cs_p_port, &app_prototype, &outer_cs_p_port)
            .unwrap();
        let _cs_r_connector = composition
            .create_delegation_connector("cs_r_connector", &inner_cs_r_port, &app_prototype, &outer_cs_r_port)
            .unwrap();
        // connecting a PR port to a PR port is not allowed for ClientServerInterfaces
        let cs_pr_connector_result = composition.create_delegation_connector(
            "cs_pr_connector",
            &inner_cs_pr_port,
            &app_prototype,
            &outer_cs_pr_port,
        );
        assert!(cs_pr_connector_result.is_err());

        // connecting different interfaces is not allowed
        let result = composition.create_delegation_connector(
            "invalid_connector",
            &inner_sr_p_port,
            &app_prototype,
            &outer_cs_p_port,
        );
        assert!(result.is_err());

        assert_eq!(sr_p_connector.name(), Some("sr_p_connector".to_string()));
        assert_eq!(
            sr_p_connector.element().element_name(),
            ElementName::DelegationSwConnector
        );
    }

    #[test]
    fn test_assembly_sw_connector() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        // create interfaces for the ports
        let sr_interface = SenderReceiverInterface::new("sr_interface", &package).unwrap();
        let cs_interface = ClientServerInterface::new("cs_interface", &package).unwrap();

        // create a composition and two application sw component types
        let composition = CompositionSwComponentType::new("composition", &package).unwrap();
        let swc_type_1 = ApplicationSwComponentType::new("app_type_1", &package).unwrap();
        let swc_type_2 = ApplicationSwComponentType::new("app_type_2", &package).unwrap();

        // create multiple ports with different interfaces and directions
        let swc1_sr_p_port = swc_type_1.create_p_port("swc1_sr_p_port", &sr_interface).unwrap();
        let swc2_sr_p_port = swc_type_2.create_p_port("swc2_sr_p_port", &sr_interface).unwrap();
        let swc1_sr_r_port = swc_type_1.create_r_port("swc1_sr_r_port", &sr_interface).unwrap();
        let swc2_sr_r_port = swc_type_2.create_r_port("swc2_sr_r_port", &sr_interface).unwrap();
        let swc1_sr_pr_port = swc_type_1.create_pr_port("swc1_sr_pr_port", &sr_interface).unwrap();
        let swc2_sr_pr_port = swc_type_2.create_pr_port("swc2_sr_pr_port", &sr_interface).unwrap();
        let swc1_cs_p_port = swc_type_1.create_p_port("swc1_cs_p_port", &cs_interface).unwrap();
        let swc2_cs_p_port = swc_type_2.create_p_port("swc2_cs_p_port", &cs_interface).unwrap();
        let swc1_cs_r_port = swc_type_1.create_r_port("swc1_cs_r_port", &cs_interface).unwrap();
        let swc2_cs_r_port = swc_type_2.create_r_port("swc2_cs_r_port", &cs_interface).unwrap();
        let swc1_cs_pr_port = swc_type_1.create_pr_port("swc1_cs_pr_port", &cs_interface).unwrap();
        let swc2_cs_pr_port = swc_type_2.create_pr_port("swc2_cs_pr_port", &cs_interface).unwrap();

        // add both application sw component types to the composition
        let app_prototype_1 = composition.create_component("app_prototype_1", &swc_type_1).unwrap();
        let app_prototype_2 = composition.create_component("app_prototype_2", &swc_type_2).unwrap();

        // connect the ports of the two application sw component types
        // SR: P -> R (valid)
        let sr_p_r_connector = composition
            .create_assembly_connector(
                "sr_p_r_connector",
                &swc1_sr_p_port,
                &app_prototype_1,
                &swc2_sr_r_port,
                &app_prototype_2,
            )
            .unwrap();
        assert_eq!(sr_p_r_connector.p_port().unwrap(), swc1_sr_p_port.clone().into());
        assert_eq!(sr_p_r_connector.r_port().unwrap(), swc2_sr_r_port.clone().into());
        assert_eq!(sr_p_r_connector.p_sw_component().unwrap(), app_prototype_1);
        assert_eq!(sr_p_r_connector.r_sw_component().unwrap(), app_prototype_2);
        // SR: R -> P (valid)
        let _sr_r_p_connector = composition
            .create_assembly_connector(
                "sr_r_p_connector",
                &swc1_sr_r_port,
                &app_prototype_1,
                &swc2_sr_p_port,
                &app_prototype_2,
            )
            .unwrap();
        // SR: P -> PR (valid)
        let _sr_p_pr_connector = composition
            .create_assembly_connector(
                "sr_p_pr_connector",
                &swc1_sr_p_port,
                &app_prototype_1,
                &swc2_sr_pr_port,
                &app_prototype_2,
            )
            .unwrap();
        // SR: R -> PR (valid)
        let _sr_r_pr_connector = composition
            .create_assembly_connector(
                "sr_r_pr_connector",
                &swc1_sr_r_port,
                &app_prototype_1,
                &swc2_sr_pr_port,
                &app_prototype_2,
            )
            .unwrap();
        // SR: PR -> P (valid)
        let _sr_pr_p_connector = composition
            .create_assembly_connector(
                "sr_pr_p_connector",
                &swc1_sr_pr_port,
                &app_prototype_1,
                &swc2_sr_p_port,
                &app_prototype_2,
            )
            .unwrap();
        // SR: PR -> R (valid)
        let _sr_pr_r_connector = composition
            .create_assembly_connector(
                "sr_pr_r_connector",
                &swc1_sr_pr_port,
                &app_prototype_1,
                &swc2_sr_r_port,
                &app_prototype_2,
            )
            .unwrap();
        // CS: P -> R (valid)
        let _cs_p_r_connector = composition
            .create_assembly_connector(
                "cs_p_r_connector",
                &swc1_cs_p_port,
                &app_prototype_1,
                &swc2_cs_r_port,
                &app_prototype_2,
            )
            .unwrap();
        // CS: R -> P (valid)
        let _cs_r_p_connector = composition
            .create_assembly_connector(
                "cs_r_p_connector",
                &swc1_cs_r_port,
                &app_prototype_1,
                &swc2_cs_p_port,
                &app_prototype_2,
            )
            .unwrap();
        // CS: P -> PR (invalid)
        let cs_p_pr_connector_result = composition.create_assembly_connector(
            "cs_p_pr_connector",
            &swc1_cs_p_port,
            &app_prototype_1,
            &swc2_cs_pr_port,
            &app_prototype_2,
        );
        assert!(cs_p_pr_connector_result.is_err());
        // CS: R -> PR (valid)
        let _cs_r_pr_connector = composition
            .create_assembly_connector(
                "cs_r_pr_connector",
                &swc1_cs_r_port,
                &app_prototype_1,
                &swc2_cs_pr_port,
                &app_prototype_2,
            )
            .unwrap();
        // CS: PR -> P (invalid)
        let cs_pr_p_connector_result = composition.create_assembly_connector(
            "cs_pr_p_connector",
            &swc1_cs_pr_port,
            &app_prototype_1,
            &swc2_cs_p_port,
            &app_prototype_2,
        );
        assert!(cs_pr_p_connector_result.is_err());
        // CS: PR -> R (valid)
        let _cs_pr_r_connector = composition
            .create_assembly_connector(
                "cs_pr_r_connector",
                &swc1_cs_pr_port,
                &app_prototype_1,
                &swc2_cs_r_port,
                &app_prototype_2,
            )
            .unwrap();

        // SR: P -> P (invalid)
        let sr_p_p_connector_result = composition.create_assembly_connector(
            "sr_p_p_connector",
            &swc1_sr_p_port,
            &app_prototype_1,
            &swc2_sr_p_port,
            &app_prototype_2,
        );
        assert!(sr_p_p_connector_result.is_err());
        // SR: R -> R (invalid)
        let sr_r_r_connector_result = composition.create_assembly_connector(
            "sr_r_r_connector",
            &swc1_sr_r_port,
            &app_prototype_1,
            &swc2_sr_r_port,
            &app_prototype_2,
        );
        assert!(sr_r_r_connector_result.is_err());

        // connecting different interfaces is not allowed
        let result = composition.create_assembly_connector(
            "invalid_connector",
            &swc1_sr_p_port,
            &app_prototype_1,
            &swc2_cs_r_port,
            &app_prototype_2,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_pass_through_sw_connector() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        // create interfaces for the ports
        let sr_interface = SenderReceiverInterface::new("sr_interface", &package).unwrap();
        let cs_interface = ClientServerInterface::new("cs_interface", &package).unwrap();

        // create a composition
        let composition = CompositionSwComponentType::new("composition", &package).unwrap();

        // create multiple ports with different interfaces and directions
        let sr_p_port = composition.create_p_port("sr_p_port", &sr_interface).unwrap();
        let sr_r_port = composition.create_r_port("sr_r_port", &sr_interface).unwrap();
        let cs_p_port = composition.create_p_port("cs_p_port", &cs_interface).unwrap();
        let cs_r_port = composition.create_r_port("cs_r_port", &cs_interface).unwrap();

        // connect the ports of the composition
        // SR: P -> R (valid)
        let sr_p_r_connector = composition
            .create_pass_through_connector("sr_p_r_connector", &sr_p_port, &sr_r_port)
            .unwrap();
        assert_eq!(sr_p_r_connector.p_port().unwrap(), sr_p_port.clone().into());
        assert_eq!(sr_p_r_connector.r_port().unwrap(), sr_r_port.clone().into());
        // CS: R -> P (valid)
        let _cs_r_p_connector = composition
            .create_pass_through_connector("cs_r_p_connector", &cs_r_port, &cs_p_port)
            .unwrap();

        // connecting different interfaces is not allowed
        let result = composition.create_pass_through_connector("invalid_connector", &sr_p_port, &cs_r_port);
        assert!(result.is_err());
    }
}
