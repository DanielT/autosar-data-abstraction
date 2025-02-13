use crate::communication::{
    Cluster, EventGroupControlType, GeneralPurposeIPduCategory, ISignalIPdu, Pdu, PduTriggering, SoConIPduIdentifier,
    SocketAddress, TpConfig,
};
use crate::{
    abstraction_element, AbstractionElement, ArPackage, AutosarAbstractionError, IdentifiableAbstractionElement,
};
use autosar_data::{Element, ElementName, EnumItem};

//##################################################################

/// A `ServiceInstanceCollectionSet` contains `ServiceInstance`s that are provided or consumed by an ECU
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceInstanceCollectionSet(Element);
abstraction_element!(ServiceInstanceCollectionSet, ServiceInstanceCollectionSet);
impl IdentifiableAbstractionElement for ServiceInstanceCollectionSet {}

impl ServiceInstanceCollectionSet {
    /// create a new `ServiceInstanceCollectionSet`
    ///
    /// This is a Fibex element, so this function is not exported in the API.
    /// Users should call `System::create_service_instance_collection_set` instead, which also creates the required `FibexElementRef`.
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let sic = package
            .element()
            .get_or_create_sub_element(ElementName::Elements)?
            .create_named_sub_element(ElementName::ServiceInstanceCollectionSet, name)?;
        Ok(Self(sic))
    }

    /// create a new `ProvidedServiceInstance` in this `ServiceInstanceCollectionSet`
    pub fn create_provided_service_instance(
        &self,
        name: &str,
        service_identifier: u16,
        instance_identifier: u16,
        major_version: u32,
        minor_version: u32,
    ) -> Result<ProvidedServiceInstance, AutosarAbstractionError> {
        let instances = self
            .element()
            .get_or_create_sub_element(ElementName::ServiceInstances)?;

        ProvidedServiceInstance::new(
            name,
            &instances,
            service_identifier,
            instance_identifier,
            major_version,
            minor_version,
        )
    }

    /// create a new `ConsumedServiceInstance` in this `ServiceInstanceCollectionSet`
    pub fn create_consumed_service_instance(
        &self,
        name: &str,
        service_identifier: u16,
        instance_identifier: u16,
        major_version: u32,
        minor_version: &str,
    ) -> Result<ConsumedServiceInstance, AutosarAbstractionError> {
        let instances = self
            .element()
            .get_or_create_sub_element(ElementName::ServiceInstances)?;

        ConsumedServiceInstance::new(
            name,
            &instances,
            service_identifier,
            instance_identifier,
            major_version,
            minor_version,
        )
    }

    /// create an iterator over all `ServiceInstances` in this set
    pub fn service_instances(&self) -> impl Iterator<Item = ServiceInstance> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::ServiceInstances)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|elem| ServiceInstance::try_from(elem).ok())
    }
}

//##################################################################

/// A `ServiceInstance` is a service that is provided or consumed by an ECU
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ServiceInstance {
    /// A service that is provided by an ECU
    Provided(ProvidedServiceInstance),
    /// A service that is consumed by an ECU
    Consumed(ConsumedServiceInstance),
}

impl AbstractionElement for ServiceInstance {
    fn element(&self) -> &Element {
        match self {
            ServiceInstance::Provided(psi) => psi.element(),
            ServiceInstance::Consumed(csi) => csi.element(),
        }
    }
}

impl IdentifiableAbstractionElement for ServiceInstance {}

impl TryFrom<Element> for ServiceInstance {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::ProvidedServiceInstance => Ok(ServiceInstance::Provided(ProvidedServiceInstance(element))),
            ElementName::ConsumedServiceInstance => Ok(ServiceInstance::Consumed(ConsumedServiceInstance(element))),
            _ => Err(AutosarAbstractionError::InvalidParameter(
                "Element is not a ServiceInstance".to_string(),
            )),
        }
    }
}

//##################################################################

/// A `ProvidedServiceInstance` is a service that is provided by an ECU
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProvidedServiceInstance(Element);

impl AbstractionElement for ProvidedServiceInstance {
    fn element(&self) -> &Element {
        &self.0
    }
}

impl IdentifiableAbstractionElement for ProvidedServiceInstance {}

impl TryFrom<Element> for ProvidedServiceInstance {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        // hierarchy: ServiceInstances -> ProvidedServiceInstance
        let parent_name = element.parent()?.map(|p| p.element_name());
        if !matches!(parent_name, Some(ElementName::ServiceInstances)) {
            return Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "ProvidedServiceInstance".to_string(),
            });
        }

        if element.element_name() == ElementName::ProvidedServiceInstance {
            Ok(Self(element))
        } else {
            Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "ProvidedServiceInstance".to_string(),
            })
        }
    }
}

impl ProvidedServiceInstance {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        service_identifier: u16,
        instance_identifier: u16,
        major_version: u32,
        minor_version: u32,
    ) -> Result<Self, AutosarAbstractionError> {
        let psi_elem = parent.create_named_sub_element(ElementName::ProvidedServiceInstance, name)?;
        let psi = Self(psi_elem);

        psi.set_service_identifier(service_identifier)?;
        psi.set_instance_identifier(instance_identifier)?;
        psi.set_major_version(major_version)?;
        psi.set_minor_version(minor_version)?;

        Ok(psi)
    }

    /// set the service identifier of this `ProvidedServiceInstance`
    pub fn set_service_identifier(&self, service_identifier: u16) -> Result<(), AutosarAbstractionError> {
        self.0
            .get_or_create_sub_element(ElementName::ServiceIdentifier)?
            .set_character_data(u64::from(service_identifier))?;
        Ok(())
    }

    /// get the service identifier of this `ProvidedServiceInstance`
    #[must_use]
    pub fn service_identifier(&self) -> Option<u16> {
        self.0
            .get_sub_element(ElementName::ServiceIdentifier)
            .and_then(|si| si.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set the instance identifier of this `ProvidedServiceInstance`
    pub fn set_instance_identifier(&self, instance_identifier: u16) -> Result<(), AutosarAbstractionError> {
        self.0
            .get_or_create_sub_element(ElementName::InstanceIdentifier)?
            .set_character_data(u64::from(instance_identifier))?;
        Ok(())
    }

    /// get the instance identifier of this `ProvidedServiceInstance`
    #[must_use]
    pub fn instance_identifier(&self) -> Option<u16> {
        self.0
            .get_sub_element(ElementName::InstanceIdentifier)
            .and_then(|ii| ii.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set the major version of this `ProvidedServiceInstance`
    pub fn set_major_version(&self, major_version: u32) -> Result<(), AutosarAbstractionError> {
        self.0
            .get_or_create_sub_element(ElementName::MajorVersion)?
            .set_character_data(u64::from(major_version))?;
        Ok(())
    }

    /// get the major version of this `ProvidedServiceInstance`
    #[must_use]
    pub fn major_version(&self) -> Option<u32> {
        self.0
            .get_sub_element(ElementName::MajorVersion)
            .and_then(|mv| mv.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set the minor version of this `ProvidedServiceInstance`
    pub fn set_minor_version(&self, minor_version: u32) -> Result<(), AutosarAbstractionError> {
        self.0
            .get_or_create_sub_element(ElementName::MinorVersion)?
            .set_character_data(u64::from(minor_version))?;
        Ok(())
    }

    /// get the minor version of this `ProvidedServiceInstance`
    #[must_use]
    pub fn minor_version(&self) -> Option<u32> {
        self.0
            .get_sub_element(ElementName::MinorVersion)
            .and_then(|mv| mv.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// create a new `EventHandler` in this `ProvidedServiceInstance`
    pub fn create_event_handler(
        &self,
        name: &str,
        event_group_identifier: u32,
    ) -> Result<EventHandler, AutosarAbstractionError> {
        let ehs = self.element().get_or_create_sub_element(ElementName::EventHandlers)?;
        EventHandler::new(name, &ehs, event_group_identifier)
    }

    /// get the `EventHandler`s in this `ProvidedServiceInstance`
    pub fn event_handlers(&self) -> impl Iterator<Item = EventHandler> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::EventHandlers)
            .into_iter()
            .flat_map(|ehs| ehs.sub_elements())
            .filter_map(|eh| EventHandler::try_from(eh).ok())
    }

    /// set a local unicast address for this `ProvidedServiceInstance`
    ///
    /// The PSI may use two local unicast addresses, one each for UDP and TCP.
    /// The unicast address is used to assign the service to a specific ECU, and may not be empty.
    pub fn set_local_unicast_address(&self, address: &SocketAddress) -> Result<(), AutosarAbstractionError> {
        set_local_unicast_address(self.element(), address)
    }

    /// get the local unicast addresses
    pub fn local_unicast_addresses(&self) -> impl Iterator<Item = LocalUnicastAddress> + Send + 'static {
        local_unicast_addresses_iter(self.element())
    }

    /// set the SD server instance configuration for this `ProvidedServiceInstance`
    pub fn set_sd_server_instance_config(
        &self,
        config: &SomeipSdServerServiceInstanceConfig,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::SdServerTimerConfigs)?
            .create_sub_element(ElementName::SomeipSdServerServiceInstanceConfigRefConditional)?
            .create_sub_element(ElementName::SomeipSdServerServiceInstanceConfigRef)?
            .set_reference_target(config.element())?;
        Ok(())
    }

    /// get the SD server instance configuration for this `ProvidedServiceInstance`
    #[must_use]
    pub fn sd_server_instance_config(&self) -> Option<SomeipSdServerServiceInstanceConfig> {
        let ref_elem = self
            .element()
            .get_sub_element(ElementName::SdServerTimerConfigs)?
            .get_sub_element(ElementName::SomeipSdServerServiceInstanceConfigRefConditional)?
            .get_sub_element(ElementName::SomeipSdServerServiceInstanceConfigRef)?
            .get_reference_target()
            .ok()?;
        SomeipSdServerServiceInstanceConfig::try_from(ref_elem).ok()
    }
}

//##################################################################

/// An `EventHandler` describes the handling of a single event in a `ProvidedServiceInstance`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventHandler(Element);

impl AbstractionElement for EventHandler {
    fn element(&self) -> &Element {
        &self.0
    }
}

impl IdentifiableAbstractionElement for EventHandler {}

impl TryFrom<Element> for EventHandler {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        // hierarchy: ServiceInstanceCollectionSet(named) -> ServiceInstances -> ProvidedServiceInstance(named) -> EventHandlers -> EventHandler(named)
        let parent_name = element
            .named_parent()?
            .and_then(|p| p.named_parent().ok().flatten())
            .map(|p| p.element_name());
        if !matches!(parent_name, Some(ElementName::ServiceInstanceCollectionSet)) {
            return Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "EventHandler".to_string(),
            });
        }

        if element.element_name() == ElementName::EventHandler {
            Ok(Self(element))
        } else {
            Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "EventHandler".to_string(),
            })
        }
    }
}

impl EventHandler {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        event_group_identifier: u32,
    ) -> Result<Self, AutosarAbstractionError> {
        let evgrp_elem = parent.create_named_sub_element(ElementName::EventHandler, name)?;
        let evgrp = Self(evgrp_elem);

        evgrp.set_event_group_identifier(event_group_identifier)?;

        Ok(evgrp)
    }

    /// set the event group identifier of this `EventHandler`
    pub fn set_event_group_identifier(&self, event_group_identifier: u32) -> Result<(), AutosarAbstractionError> {
        self.0
            .get_or_create_sub_element(ElementName::EventGroupIdentifier)?
            .set_character_data(u64::from(event_group_identifier))?;
        Ok(())
    }

    /// get the event group identifier of this `EventHandler`
    #[must_use]
    pub fn event_group_identifier(&self) -> Option<u32> {
        self.0
            .get_sub_element(ElementName::EventGroupIdentifier)
            .and_then(|egi| egi.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// create a new `PduActivationRoutingGroup` in this `EventHandler`
    pub fn create_pdu_activation_routing_group(
        &self,
        name: &str,
        event_group_control_type: EventGroupControlType,
    ) -> Result<PduActivationRoutingGroup, AutosarAbstractionError> {
        let parent = self
            .element()
            .get_or_create_sub_element(ElementName::PduActivationRoutingGroups)?;
        PduActivationRoutingGroup::new(name, &parent, event_group_control_type)
    }

    /// get the `PduActivationRoutingGroup`s in this `EventHandler`
    pub fn pdu_activation_routing_groups(&self) -> impl Iterator<Item = PduActivationRoutingGroup> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::PduActivationRoutingGroups)
            .into_iter()
            .flat_map(|pargs| pargs.sub_elements())
            .filter_map(|parg| PduActivationRoutingGroup::try_from(parg).ok())
    }

    /// set the SD server event group timing configuration for this `EventHandler`
    pub fn set_sd_server_event_group_timing_config(
        &self,
        config: &SomeipSdServerEventGroupTimingConfig,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::SdServerEgTimingConfigs)?
            .create_sub_element(ElementName::SomeipSdServerEventGroupTimingConfigRefConditional)?
            .create_sub_element(ElementName::SomeipSdServerEventGroupTimingConfigRef)?
            .set_reference_target(config.element())?;
        Ok(())
    }

    /// get the SD server event group timing configuration for this `EventHandler`
    #[must_use]
    pub fn sd_server_event_group_timing_config(&self) -> Option<SomeipSdServerEventGroupTimingConfig> {
        let ref_elem = self
            .element()
            .get_sub_element(ElementName::SdServerEgTimingConfigs)?
            .get_sub_element(ElementName::SomeipSdServerEventGroupTimingConfigRefConditional)?
            .get_sub_element(ElementName::SomeipSdServerEventGroupTimingConfigRef)?
            .get_reference_target()
            .ok()?;
        SomeipSdServerEventGroupTimingConfig::try_from(ref_elem).ok()
    }
}

//##################################################################

/// A `ConsumedServiceInstance` is a service that is consumed by an ECU
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConsumedServiceInstance(Element);

impl AbstractionElement for ConsumedServiceInstance {
    fn element(&self) -> &Element {
        &self.0
    }
}

impl IdentifiableAbstractionElement for ConsumedServiceInstance {}

impl TryFrom<Element> for ConsumedServiceInstance {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        // hierarchy: ServiceInstances -> ConsumedServiceInstance
        let parent_name = element.parent()?.map(|p| p.element_name());
        if !matches!(parent_name, Some(ElementName::ServiceInstances)) {
            return Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "ConsumedServiceInstance".to_string(),
            });
        }

        if element.element_name() == ElementName::ConsumedServiceInstance {
            Ok(Self(element))
        } else {
            Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "ConsumedServiceInstance".to_string(),
            })
        }
    }
}

impl ConsumedServiceInstance {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        service_identifier: u16,
        instance_identifier: u16,
        major_version: u32,
        minor_version: &str,
    ) -> Result<Self, AutosarAbstractionError> {
        let csi_elem = parent.create_named_sub_element(ElementName::ConsumedServiceInstance, name)?;
        let csi = Self(csi_elem);

        csi.set_service_identifier(service_identifier)?;
        csi.set_instance_identifier(instance_identifier)?;
        csi.set_major_version(major_version)?;
        csi.set_minor_version(minor_version)?;

        Ok(csi)
    }

    /// set the service identifier of this `ConsumedServiceInstance`
    pub fn set_service_identifier(&self, service_identifier: u16) -> Result<(), AutosarAbstractionError> {
        self.0
            .get_or_create_sub_element(ElementName::ServiceIdentifier)?
            .set_character_data(u64::from(service_identifier))?;
        Ok(())
    }

    /// get the service identifier of this `ConsumedServiceInstance`
    #[must_use]
    pub fn service_identifier(&self) -> Option<u16> {
        self.0
            .get_sub_element(ElementName::ServiceIdentifier)
            .and_then(|si| si.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set the instance identifier of this `ConsumedServiceInstance`
    pub fn set_instance_identifier(&self, instance_identifier: u16) -> Result<(), AutosarAbstractionError> {
        self.0
            .get_or_create_sub_element(ElementName::InstanceIdentifier)?
            .set_character_data(u64::from(instance_identifier))?;
        Ok(())
    }

    /// get the instance identifier of this `ConsumedServiceInstance`
    #[must_use]
    pub fn instance_identifier(&self) -> Option<u16> {
        self.0
            .get_sub_element(ElementName::InstanceIdentifier)
            .and_then(|ii| ii.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set the major version of this `ConsumedServiceInstance`
    pub fn set_major_version(&self, major_version: u32) -> Result<(), AutosarAbstractionError> {
        self.0
            .get_or_create_sub_element(ElementName::MajorVersion)?
            .set_character_data(u64::from(major_version))?;
        Ok(())
    }

    /// get the major version of this `ConsumedServiceInstance`
    #[must_use]
    pub fn major_version(&self) -> Option<u32> {
        self.0
            .get_sub_element(ElementName::MajorVersion)
            .and_then(|mv| mv.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set the minor version of this `ConsumedServiceInstance`
    ///
    /// The minor version can be a number or the String "ANY".
    pub fn set_minor_version(&self, minor_version: &str) -> Result<(), AutosarAbstractionError> {
        self.0
            .get_or_create_sub_element(ElementName::MinorVersion)?
            .set_character_data(minor_version)?;
        Ok(())
    }

    /// get the minor version of this `ConsumedServiceInstance`
    ///
    /// The minor version can be a number or the String "ANY".
    #[must_use]
    pub fn minor_version(&self) -> Option<String> {
        self.0
            .get_sub_element(ElementName::MinorVersion)
            .and_then(|mv| mv.character_data())
            .and_then(|cdata| cdata.string_value())
    }

    /// create a new `ConsumedEventGrup` in this `ConsumedServiceInstance`
    pub fn create_consumed_event_group(
        &self,
        name: &str,
        event_group_identifier: u32,
    ) -> Result<ConsumedEventGroup, AutosarAbstractionError> {
        let cegs = self
            .element()
            .get_or_create_sub_element(ElementName::ConsumedEventGroups)?;
        ConsumedEventGroup::new(name, &cegs, event_group_identifier)
    }

    /// get the `ConsumedEventGroup`s in this `ConsumedServiceInstance`
    pub fn consumed_event_groups(&self) -> impl Iterator<Item = ConsumedEventGroup> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::ConsumedEventGroups)
            .into_iter()
            .flat_map(|cegs| cegs.sub_elements())
            .filter_map(|ceg| ConsumedEventGroup::try_from(ceg).ok())
    }

    /// set a local unicast address for this `ConsumedServiceInstance`
    ///
    /// The CSI may use two local unicast addresses, one each for UDP and TCP.
    /// If the consumed service instance does not specify a local unicast address
    /// because it only receives multicast messages, then the `ConsumedEventGroup`
    /// must have an eventMulticastAddress.
    pub fn set_local_unicast_address(&self, address: &SocketAddress) -> Result<(), AutosarAbstractionError> {
        set_local_unicast_address(self.element(), address)
    }

    /// get the local unicast addresses
    pub fn local_unicast_addresses(&self) -> impl Iterator<Item = LocalUnicastAddress> + Send + 'static {
        local_unicast_addresses_iter(self.element())
    }

    /// set the SD client instance configuration for this `ConsumedServiceInstance`
    pub fn set_sd_client_instance_config(
        &self,
        config: &SomeipSdClientServiceInstanceConfig,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::SdClientTimerConfigs)?
            .create_sub_element(ElementName::SomeipSdClientServiceInstanceConfigRefConditional)?
            .create_sub_element(ElementName::SomeipSdClientServiceInstanceConfigRef)?
            .set_reference_target(config.element())?;
        Ok(())
    }

    /// get the SD client instance configuration for this `ConsumedServiceInstance`
    #[must_use]
    pub fn sd_client_instance_config(&self) -> Option<SomeipSdClientServiceInstanceConfig> {
        let ref_elem = self
            .element()
            .get_sub_element(ElementName::SdClientTimerConfigs)?
            .get_sub_element(ElementName::SomeipSdClientServiceInstanceConfigRefConditional)?
            .get_sub_element(ElementName::SomeipSdClientServiceInstanceConfigRef)?
            .get_reference_target()
            .ok()?;
        SomeipSdClientServiceInstanceConfig::try_from(ref_elem).ok()
    }
}

//##################################################################

/// A `ConsumedEventGroup` is a group of events in a `ConsumedServiceInstance` that are consumed by an ECU
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConsumedEventGroup(Element);

impl AbstractionElement for ConsumedEventGroup {
    fn element(&self) -> &Element {
        &self.0
    }
}

impl IdentifiableAbstractionElement for ConsumedEventGroup {}

impl TryFrom<Element> for ConsumedEventGroup {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        // hierarchy: ServiceInstanceCollectionSet(named) -> ServiceInstances -> ConsumedServiceInstance(named) -> ConsumedEventGroups -> ConsumedEventGroup(named)
        let parent_name = element
            .named_parent()?
            .and_then(|p| p.named_parent().ok().flatten())
            .map(|p| p.element_name());
        if !matches!(parent_name, Some(ElementName::ServiceInstanceCollectionSet)) {
            return Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "ConsumedEventGroup".to_string(),
            });
        }

        if element.element_name() == ElementName::ConsumedEventGroup {
            Ok(Self(element))
        } else {
            Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "ConsumedEventGroup".to_string(),
            })
        }
    }
}

impl ConsumedEventGroup {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        event_group_identifier: u32,
    ) -> Result<Self, AutosarAbstractionError> {
        let ceg_elem = parent.create_named_sub_element(ElementName::ConsumedEventGroup, name)?;
        let ceg = Self(ceg_elem);
        ceg.set_event_group_identifier(event_group_identifier)?;

        Ok(ceg)
    }

    /// set the event group identifier of this `ConsumedEventGroup`
    pub fn set_event_group_identifier(&self, event_group_identifier: u32) -> Result<(), AutosarAbstractionError> {
        self.0
            .get_or_create_sub_element(ElementName::EventGroupIdentifier)?
            .set_character_data(u64::from(event_group_identifier))?;
        Ok(())
    }

    /// get the event group identifier of this `ConsumedEventGroup`
    #[must_use]
    pub fn event_group_identifier(&self) -> Option<u32> {
        self.0
            .get_sub_element(ElementName::EventGroupIdentifier)
            .and_then(|egi| egi.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// create a new `PduActivationRoutingGroup` in this `ConsumedEventGroup`
    pub fn create_pdu_activation_routing_group(
        &self,
        name: &str,
        event_group_control_type: EventGroupControlType,
    ) -> Result<PduActivationRoutingGroup, AutosarAbstractionError> {
        let parent = self
            .element()
            .get_or_create_sub_element(ElementName::PduActivationRoutingGroups)?;
        PduActivationRoutingGroup::new(name, &parent, event_group_control_type)
    }

    /// iterate over the `PduActivationRoutingGroup`s in this `ConsumedEventGroup`
    pub fn pdu_activation_routing_groups(&self) -> impl Iterator<Item = PduActivationRoutingGroup> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::PduActivationRoutingGroups)
            .into_iter()
            .flat_map(|pargs| pargs.sub_elements())
            .filter_map(|parg| PduActivationRoutingGroup::try_from(parg).ok())
    }

    /// add an event multicast address to this `ConsumedEventGroup`
    pub fn add_event_multicast_address(&self, address: &SocketAddress) -> Result<(), AutosarAbstractionError> {
        let Some(application_endpoint) = address.element().get_sub_element(ElementName::ApplicationEndpoint) else {
            return Err(AutosarAbstractionError::InvalidParameter(
                "Can't add the event multicast address: The target SocketAddress does not have an ApplicationEndpoint, so it can't be used".to_string(),
            ));
        };

        let container = self
            .element()
            .get_or_create_sub_element(ElementName::EventMulticastAddresss)?;
        container
            .create_sub_element(ElementName::ApplicationEndpointRefConditional)?
            .create_sub_element(ElementName::ApplicationEndpointRef)?
            .set_reference_target(&application_endpoint)?;

        Ok(())
    }

    /// get the event multicast addresses
    pub fn event_multicast_addresses(&self) -> impl Iterator<Item = SocketAddress> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::EventMulticastAddresss)
            .into_iter()
            .flat_map(|addresses| addresses.sub_elements())
            .filter_map(|app_endpoint_ref_cond| {
                app_endpoint_ref_cond.get_sub_element(ElementName::ApplicationEndpointRef)
            })
            .filter_map(|app_endpoint_ref| app_endpoint_ref.get_reference_target().ok())
            .filter_map(|app_endpoint| app_endpoint.named_parent().ok().flatten())
            .filter_map(|sockaddr| SocketAddress::try_from(sockaddr).ok())
    }

    /// set the SD client timer configuration for this `ConsumedEventGroup`
    pub fn set_sd_client_timer_config(
        &self,
        config: &SomeipSdClientEventGroupTimingConfig,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::SdClientTimerConfigs)?
            .create_sub_element(ElementName::SomeipSdClientEventGroupTimingConfigRefConditional)?
            .create_sub_element(ElementName::SomeipSdClientEventGroupTimingConfigRef)?
            .set_reference_target(config.element())?;
        Ok(())
    }

    /// get the SD client timer configuration for this `ConsumedEventGroup`
    #[must_use]
    pub fn sd_client_timer_config(&self) -> Option<SomeipSdClientEventGroupTimingConfig> {
        let ref_elem = self
            .element()
            .get_sub_element(ElementName::SdClientTimerConfigs)?
            .get_sub_element(ElementName::SomeipSdClientEventGroupTimingConfigRefConditional)?
            .get_sub_element(ElementName::SomeipSdClientEventGroupTimingConfigRef)?
            .get_reference_target()
            .ok()?;
        SomeipSdClientEventGroupTimingConfig::try_from(ref_elem).ok()
    }
}

//##################################################################

/// A `LocalUnicastAddress` is a local address (TCP or UDP) that can be used for a `ProvidedServiceInstance` or `ConsumedServiceInstance`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LocalUnicastAddress {
    /// A UDP address
    Udp(SocketAddress),
    /// A TCP address
    Tcp(SocketAddress),
}

/// helper function for both `ConsumedServiceInstance` and `ProvidedServiceInstance`
fn set_local_unicast_address(parent: &Element, target_socket: &SocketAddress) -> Result<(), AutosarAbstractionError> {
    let Some(target_appendpoint) = target_socket
        .element()
        .get_sub_element(ElementName::ApplicationEndpoint)
    else {
        return Err(AutosarAbstractionError::InvalidParameter(
            "Can't set the local address: The target SocketAddress does not have an ApplicationEndpoint, so it can't be used".to_string(),
        ));
    };
    let Some(tp_config) = target_socket.tp_config() else {
        return Err(AutosarAbstractionError::InvalidParameter(
            "Can't set the local address: The target SocketAddress does not have a TP configuration, so it can't be used".to_string(),
        ));
    };

    let addresses_container = match parent.get_sub_element(ElementName::LocalUnicastAddresss) {
        Some(addresses_container) => {
            // LOCAL-UNICAST-ADDRESSS -> collection of(APPLICATION-ENDPOINT-REF-CONDITIONAL -> APPLICATION-ENDPOINT-REF)
            // check if an address of the given type already exists and remove it
            for existing_ref_elem in addresses_container.sub_elements() {
                // existing_ref_elem is an APPLICATION-ENDPOINT-REF-CONDITIONAL
                // find the referenced APPLICATION-ENDPOINT, go up to the parent SocketAddress, and get the
                // TP configuration in order to determine if this is a TCP or UDP address
                if let Some(existing_tp_config) = existing_ref_elem
                    .get_sub_element(ElementName::ApplicationEndpointRef)
                    .and_then(|aer| aer.get_reference_target().ok())
                    .and_then(|ae| ae.named_parent().ok().flatten())
                    .and_then(|sockaddr| SocketAddress::try_from(sockaddr).ok())
                    .and_then(|sa| sa.tp_config())
                {
                    // if the target socket address has the same type as the new address, remove the old one
                    if matches!(
                        (existing_tp_config, &tp_config),
                        (TpConfig::TcpTp { .. }, &TpConfig::TcpTp { .. })
                            | (TpConfig::UdpTp { .. }, &TpConfig::UdpTp { .. })
                    ) {
                        addresses_container.remove_sub_element(existing_ref_elem)?;
                    }
                }
            }
            addresses_container
        }
        None => parent.create_sub_element(ElementName::LocalUnicastAddresss)?,
    };

    // no distinction between TCP and UDP addresses is needed when creating the new reference
    addresses_container
        .create_sub_element(ElementName::ApplicationEndpointRefConditional)?
        .create_sub_element(ElementName::ApplicationEndpointRef)?
        .set_reference_target(&target_appendpoint)?;
    Ok(())
}

fn local_unicast_addresses_iter(element: &Element) -> impl Iterator<Item = LocalUnicastAddress> + Send + 'static {
    // first, build an iterator over all the ApplicationEndpoint Elements referenced by the LocalUnicastAddresss container
    let app_endpoint_iter = element
        .get_sub_element(ElementName::LocalUnicastAddresss)
        .into_iter()
        .flat_map(|addresses| addresses.sub_elements())
        .filter_map(|app_endpoint_ref_cond| app_endpoint_ref_cond.get_sub_element(ElementName::ApplicationEndpointRef))
        .filter_map(|app_endpoint_ref| app_endpoint_ref.get_reference_target().ok());

    // (split for readability to avoid a long chain of filter_map calls)

    // for each ApplicationEndpoint, get the containing SocketAddress, and then wrap it in a LocalUnicastAddress of the correct type
    app_endpoint_iter
        .filter_map(|ae| ae.named_parent().ok().flatten())
        .filter_map(|sockaddr| SocketAddress::try_from(sockaddr).ok())
        .filter_map(|sa| {
            sa.tp_config().map(|tp_config| match tp_config {
                TpConfig::TcpTp { .. } => LocalUnicastAddress::Tcp(sa),
                TpConfig::UdpTp { .. } => LocalUnicastAddress::Udp(sa),
            })
        })
}

//##################################################################

/// A group of Pdus that can be activated or deactivated for transmission over a socket connection.
/// It is used by `EventHandler`s in `ProvidedServiceInstance`s and `ConsumedServiceInstance`s.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PduActivationRoutingGroup(Element);
abstraction_element!(PduActivationRoutingGroup, PduActivationRoutingGroup);
impl IdentifiableAbstractionElement for PduActivationRoutingGroup {}

impl PduActivationRoutingGroup {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        event_group_control_type: EventGroupControlType,
    ) -> Result<Self, AutosarAbstractionError> {
        let elem = parent.create_named_sub_element(ElementName::PduActivationRoutingGroup, name)?;
        elem.create_sub_element(ElementName::EventGroupControlType)?
            .set_character_data::<EnumItem>(event_group_control_type.into())?;
        Ok(Self(elem))
    }

    /// get the event group control type of this `PduActivationRoutingGroup`
    #[must_use]
    pub fn event_group_control_type(&self) -> Option<EventGroupControlType> {
        self.0
            .get_sub_element(ElementName::EventGroupControlType)
            .and_then(|egct| egct.character_data())
            .and_then(|cdata| cdata.enum_value())
            .and_then(|val| EventGroupControlType::try_from(val).ok())
    }

    /// add a reference to a `SoConIPduIdentifier` for UDP communication to this `PduActivationRoutingGroup`
    pub fn add_ipdu_identifier_udp(
        &self,
        ipdu_identifier: &SoConIPduIdentifier,
    ) -> Result<(), AutosarAbstractionError> {
        let elem = self
            .element()
            .get_or_create_sub_element(ElementName::IPduIdentifierUdpRefs)?;
        elem.create_sub_element(ElementName::IPduIdentifierUdpRef)?
            .set_reference_target(ipdu_identifier.element())?;
        Ok(())
    }

    /// get all `SoConIPduIdentifier`s for UDP communication in this `PduActivationRoutingGroup`
    pub fn ipdu_identifiers_udp(&self) -> impl Iterator<Item = SoConIPduIdentifier> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::IPduIdentifierUdpRefs)
            .into_iter()
            .flat_map(|refs| refs.sub_elements())
            .filter_map(|ref_elem| ref_elem.get_reference_target().ok())
            .filter_map(|target| SoConIPduIdentifier::try_from(target).ok())
    }

    /// add a reference to a `SoConIPduIdentifier` for TCP communication to this `PduActivationRoutingGroup`
    pub fn add_ipdu_identifier_tcp(
        &self,
        ipdu_identifier: &SoConIPduIdentifier,
    ) -> Result<(), AutosarAbstractionError> {
        let elem = self
            .element()
            .get_or_create_sub_element(ElementName::IPduIdentifierTcpRefs)?;
        elem.create_sub_element(ElementName::IPduIdentifierTcpRef)?
            .set_reference_target(ipdu_identifier.element())?;
        Ok(())
    }

    /// get all `SoConIPduIdentifier`s for TCP communication in this `PduActivationRoutingGroup`
    pub fn ipdu_identifiers_tcp(&self) -> impl Iterator<Item = SoConIPduIdentifier> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::IPduIdentifierTcpRefs)
            .into_iter()
            .flat_map(|refs| refs.sub_elements())
            .filter_map(|ref_elem| ref_elem.get_reference_target().ok())
            .filter_map(|target| SoConIPduIdentifier::try_from(target).ok())
    }
}

//##################################################################

/// A `SomeipSdServerServiceInstanceConfig` is a configuration for a `ProvidedServiceInstance`
///
/// This configuration is a named element that is created separately and can be used by multiple `ProvidedServiceInstance`s.
///
/// Use [`ArPackage::create_someip_sd_server_service_instance_config`] to create a new `SomeipSdServerServiceInstanceConfig`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SomeipSdServerServiceInstanceConfig(Element);
abstraction_element!(SomeipSdServerServiceInstanceConfig, SomeipSdServerServiceInstanceConfig);
impl IdentifiableAbstractionElement for SomeipSdServerServiceInstanceConfig {}

impl SomeipSdServerServiceInstanceConfig {
    /// create a new `SomeipSdServerServiceInstanceConfig` in the given package
    pub(crate) fn new(name: &str, package: &ArPackage, ttl: u32) -> Result<Self, AutosarAbstractionError> {
        let pkg_elem = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem = pkg_elem.create_named_sub_element(ElementName::SomeipSdServerServiceInstanceConfig, name)?;

        elem.create_sub_element(ElementName::ServiceOfferTimeToLive)?
            .set_character_data(u64::from(ttl))?;

        Ok(Self(elem))
    }

    /// set the service offer time to live of this `SomeipSdServerServiceInstanceConfig`
    pub fn set_service_offer_time_to_live(&self, ttl: u32) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::ServiceOfferTimeToLive)?
            .set_character_data(u64::from(ttl))?;
        Ok(())
    }

    /// get the service offer time to live of this `SomeipSdServerServiceInstanceConfig`
    #[must_use]
    pub fn service_offer_time_to_live(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::ServiceOfferTimeToLive)
            .and_then(|ttl| ttl.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set the offer cyclic delay of this `SomeipSdServerServiceInstanceConfig`
    pub fn set_offer_cyclic_delay(&self, delay: f64) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::OfferCyclicDelay)?
            .set_character_data(delay)?;
        Ok(())
    }

    /// get the offer cyclic delay of this `SomeipSdServerServiceInstanceConfig`
    #[must_use]
    pub fn offer_cyclic_delay(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::OfferCyclicDelay)
            .and_then(|ocd| ocd.character_data())
            .and_then(|cdata| cdata.parse_float())
    }

    /// set the priority of this `SomeipSdServerServiceInstanceConfig`
    ///
    /// Available since R21-11 (`AUTOSAR_00050`)
    pub fn set_priority(&self, priority: u8) -> Result<(), AutosarAbstractionError> {
        // try to set it, but ignore the error if the element is not available
        if let Ok(priority_element) = self.element().get_or_create_sub_element(ElementName::Priority) {
            priority_element.set_character_data(u64::from(priority))?;
        }
        Ok(())
    }

    /// get the priority of this `SomeipSdServerServiceInstanceConfig`
    #[must_use]
    pub fn priority(&self) -> Option<u8> {
        self.element()
            .get_sub_element(ElementName::Priority)
            .and_then(|p| p.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set the initial offer behavior of this `SomeipSdServerServiceInstanceConfig`
    pub fn set_initial_offer_behavior(
        &self,
        initial_offer_behavior: &InitialSdDelayConfig,
    ) -> Result<(), AutosarAbstractionError> {
        let iob = self
            .element()
            .get_or_create_sub_element(ElementName::InitialOfferBehavior)?;
        initial_offer_behavior.set(&iob)?;
        Ok(())
    }

    /// get the initial offer behavior of this `SomeipSdServerServiceInstanceConfig`
    #[must_use]
    pub fn initial_offer_behavior(&self) -> Option<InitialSdDelayConfig> {
        let iob = self.element().get_sub_element(ElementName::InitialOfferBehavior)?;
        InitialSdDelayConfig::get(&iob)
    }

    /// set the request response delay of this `SomeipSdServerServiceInstanceConfig`
    pub fn set_request_response_delay(
        &self,
        request_response_delay: &RequestResponseDelay,
    ) -> Result<(), AutosarAbstractionError> {
        let rrd = self
            .element()
            .get_or_create_sub_element(ElementName::RequestResponseDelay)?;
        request_response_delay.set(&rrd)?;
        Ok(())
    }

    /// get the request response delay of this `SomeipSdServerServiceInstanceConfig`
    #[must_use]
    pub fn request_response_delay(&self) -> Option<RequestResponseDelay> {
        let rrd = self.element().get_sub_element(ElementName::RequestResponseDelay)?;
        RequestResponseDelay::get(&rrd)
    }
}

//##################################################################

/// A `SomeipSdServerEventGroupTimingConfig` contains the configuration for the timing of an `EventHandler`
///
/// This configuration is a named element that is created separately and can be used by multiple `EventHandler`s.
///
/// Use [`ArPackage::create_someip_sd_server_event_group_timing_config`] to create a new `SomeipSdServerEventGroupTimingConfig`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SomeipSdServerEventGroupTimingConfig(Element);
abstraction_element!(
    SomeipSdServerEventGroupTimingConfig,
    SomeipSdServerEventGroupTimingConfig
);
impl IdentifiableAbstractionElement for SomeipSdServerEventGroupTimingConfig {}

impl SomeipSdServerEventGroupTimingConfig {
    /// create a new `SomeipSdServerEventGroupTimingConfig` in the given package
    pub(crate) fn new(
        name: &str,
        package: &ArPackage,
        request_response_delay: &RequestResponseDelay,
    ) -> Result<Self, AutosarAbstractionError> {
        let pkg_elem = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem = pkg_elem.create_named_sub_element(ElementName::SomeipSdServerEventGroupTimingConfig, name)?;
        let rrd = elem.create_sub_element(ElementName::RequestResponseDelay)?;
        request_response_delay.set(&rrd)?;

        Ok(Self(elem))
    }

    /// set the request response delay of this `SomeipSdServerEventGroupTimingConfig`
    pub fn set_request_response_delay(
        &self,
        request_response_delay: &RequestResponseDelay,
    ) -> Result<(), AutosarAbstractionError> {
        let rrd = self
            .element()
            .get_or_create_sub_element(ElementName::RequestResponseDelay)?;
        request_response_delay.set(&rrd)?;
        Ok(())
    }

    /// get the request response delay of this `SomeipSdServerEventGroupTimingConfig`
    #[must_use]
    pub fn request_response_delay(&self) -> Option<RequestResponseDelay> {
        let rrd = self.element().get_sub_element(ElementName::RequestResponseDelay)?;
        RequestResponseDelay::get(&rrd)
    }
}

//##################################################################

/// A `SomeipSdClientServiceInstanceConfig` is a configuration for a `ConsumedServiceInstance`
///
/// This configuration is a named element that is created separately and can be used by multiple `ConsumedServiceInstance`s.
///
/// Use [`ArPackage::create_someip_sd_client_service_instance_config`] to create a new `SomeipSdClientServiceInstanceConfig`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SomeipSdClientServiceInstanceConfig(Element);
abstraction_element!(SomeipSdClientServiceInstanceConfig, SomeipSdClientServiceInstanceConfig);
impl IdentifiableAbstractionElement for SomeipSdClientServiceInstanceConfig {}

impl SomeipSdClientServiceInstanceConfig {
    /// create a new `SomeipSdClientServiceInstanceConfig` in the given package
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let pkg_elem = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem = pkg_elem.create_named_sub_element(ElementName::SomeipSdClientServiceInstanceConfig, name)?;

        Ok(Self(elem))
    }

    /// set the initial find behavior of this `SomeipSdClientServiceInstanceConfig`
    pub fn set_initial_find_behavior(
        &self,
        initial_find_behavior: &InitialSdDelayConfig,
    ) -> Result<(), AutosarAbstractionError> {
        let ifb = self
            .element()
            .get_or_create_sub_element(ElementName::InitialFindBehavior)?;
        initial_find_behavior.set(&ifb)?;
        Ok(())
    }

    /// get the initial find behavior of this `SomeipSdClientServiceInstanceConfig`
    #[must_use]
    pub fn initial_find_behavior(&self) -> Option<InitialSdDelayConfig> {
        let ifb = self.element().get_sub_element(ElementName::InitialFindBehavior)?;
        InitialSdDelayConfig::get(&ifb)
    }

    /// set the priority of this `SomeipSdClientServiceInstanceConfig`
    ///
    /// Available since R21-11 (`AUTOSAR_00050`)
    pub fn set_priority(&self, priority: u8) -> Result<(), AutosarAbstractionError> {
        // try to set it, but ignore the error if the element is not available
        if let Ok(priority_element) = self.element().get_or_create_sub_element(ElementName::Priority) {
            priority_element.set_character_data(u64::from(priority))?;
        }
        Ok(())
    }

    /// get the priority of this `SomeipSdClientServiceInstanceConfig`
    #[must_use]
    pub fn priority(&self) -> Option<u8> {
        self.element()
            .get_sub_element(ElementName::Priority)
            .and_then(|p| p.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }
}

//##################################################################

/// A `SomeipSdClientEventGroupTimingConfig` contains the configuration for the timing of a `ConsumedEventGroup`
///
/// This configuration is a named element that is created separately and can be used by multiple `ConsumedEventGroup`s.
///
/// Use [`ArPackage::create_someip_sd_client_event_group_timing_config`] to create a new `SomeipSdClientEventGroupTimingConfig`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SomeipSdClientEventGroupTimingConfig(Element);
abstraction_element!(
    SomeipSdClientEventGroupTimingConfig,
    SomeipSdClientEventGroupTimingConfig
);
impl IdentifiableAbstractionElement for SomeipSdClientEventGroupTimingConfig {}

impl SomeipSdClientEventGroupTimingConfig {
    /// create a new `SomeipSdClientEventGroupTimingConfig` in the given package
    pub(crate) fn new(name: &str, package: &ArPackage, time_to_live: u32) -> Result<Self, AutosarAbstractionError> {
        let pkg_elem = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem = pkg_elem.create_named_sub_element(ElementName::SomeipSdClientEventGroupTimingConfig, name)?;
        elem.create_sub_element(ElementName::TimeToLive)?
            .set_character_data(u64::from(time_to_live))?;

        Ok(Self(elem))
    }

    /// set the time to live of this `SomeipSdClientEventGroupTimingConfig`
    pub fn set_time_to_live(&self, time_to_live: u32) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::TimeToLive)?
            .set_character_data(u64::from(time_to_live))?;
        Ok(())
    }

    /// get the time to live of this `SomeipSdClientEventGroupTimingConfig`
    #[must_use]
    pub fn time_to_live(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::TimeToLive)
            .and_then(|ttl| ttl.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set the request response delay of this `SomeipSdClientEventGroupTimingConfig`
    pub fn set_request_response_delay(
        &self,
        request_response_delay: &RequestResponseDelay,
    ) -> Result<(), AutosarAbstractionError> {
        let rrd = self
            .element()
            .get_or_create_sub_element(ElementName::RequestResponseDelay)?;
        request_response_delay.set(&rrd)?;
        Ok(())
    }

    /// get the request response delay of this `SomeipSdClientEventGroupTimingConfig`
    #[must_use]
    pub fn request_response_delay(&self) -> Option<RequestResponseDelay> {
        let rrd = self.element().get_sub_element(ElementName::RequestResponseDelay)?;
        RequestResponseDelay::get(&rrd)
    }

    /// set the subscribe eventgroup retry delay of this `SomeipSdClientEventGroupTimingConfig`
    pub fn set_subscribe_eventgroup_retry_delay(
        &self,
        subscribe_eventgroup_retry_delay: f64,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::SubscribeEventgroupRetryDelay)?
            .set_character_data(subscribe_eventgroup_retry_delay)?;
        Ok(())
    }

    /// get the subscribe eventgroup retry delay of this `SomeipSdClientEventGroupTimingConfig`
    #[must_use]
    pub fn subscribe_eventgroup_retry_delay(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::SubscribeEventgroupRetryDelay)
            .and_then(|sgrd| sgrd.character_data())
            .and_then(|cdata| cdata.parse_float())
    }

    /// set subscribe eventgroup retry max of this `SomeipSdClientEventGroupTimingConfig`
    pub fn set_subscribe_eventgroup_retry_max(
        &self,
        subscribe_eventgroup_retry_max: u32,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::SubscribeEventgroupRetryMax)?
            .set_character_data(u64::from(subscribe_eventgroup_retry_max))?;
        Ok(())
    }

    /// get the value of subscribe eventgroup retry max of this `SomeipSdClientEventGroupTimingConfig`
    #[must_use]
    pub fn subscribe_eventgroup_retry_max(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::SubscribeEventgroupRetryMax)
            .and_then(|sgrm| sgrm.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }
}

//##################################################################

/// A `RequestResponseDelay` contains the minimum and maximum delay for a request-response cycle
#[derive(Debug, Clone, PartialEq)]
pub struct RequestResponseDelay {
    /// minimum value of the delay in seconds
    pub min_value: f64,
    /// maximum value of the delay in seconds
    pub max_value: f64,
}

impl RequestResponseDelay {
    fn set(&self, element: &Element) -> Result<(), AutosarAbstractionError> {
        element
            .get_or_create_sub_element(ElementName::MinValue)?
            .set_character_data(self.min_value)?;
        element
            .get_or_create_sub_element(ElementName::MaxValue)?
            .set_character_data(self.max_value)?;
        Ok(())
    }

    fn get(element: &Element) -> Option<Self> {
        let min_value = element
            .get_sub_element(ElementName::MinValue)
            .and_then(|rrd| rrd.character_data())
            .and_then(|cdata| cdata.parse_float())
            .unwrap_or(0.0);
        let max_value = element
            .get_sub_element(ElementName::MaxValue)
            .and_then(|rrd| rrd.character_data())
            .and_then(|cdata| cdata.parse_float())
            .unwrap_or(0.0);
        Some(Self { min_value, max_value })
    }
}

//##################################################################

/// A `InitialSdDelayConfig` contains the configuration for the initial delay of an SD client or server
#[derive(Debug, Clone, PartialEq)]
pub struct InitialSdDelayConfig {
    /// maximum value of the randomized delay in seconds
    pub initial_delay_max_value: f64,
    /// minimum value of the randomized delay in seconds
    pub initial_delay_min_value: f64,
    /// base delay for repetitions in seconds
    pub initial_repetitions_base_delay: Option<f64>,
    /// maximum number of repetitions
    pub initial_repetitions_max: Option<u32>,
}

impl InitialSdDelayConfig {
    fn set(&self, element: &Element) -> Result<(), AutosarAbstractionError> {
        element
            .get_or_create_sub_element(ElementName::InitialDelayMaxValue)?
            .set_character_data(self.initial_delay_max_value)?;
        element
            .get_or_create_sub_element(ElementName::InitialDelayMinValue)?
            .set_character_data(self.initial_delay_min_value)?;
        if let Some(base_delay) = self.initial_repetitions_base_delay {
            element
                .get_or_create_sub_element(ElementName::InitialRepetitionsBaseDelay)?
                .set_character_data(base_delay)?;
        }
        if let Some(max_repetitions) = self.initial_repetitions_max {
            element
                .get_or_create_sub_element(ElementName::InitialRepetitionsMax)?
                .set_character_data(u64::from(max_repetitions))?;
        }
        Ok(())
    }

    fn get(element: &Element) -> Option<Self> {
        let initial_delay_max_value = element
            .get_sub_element(ElementName::InitialDelayMaxValue)
            .and_then(|rrd| rrd.character_data())
            .and_then(|cdata| cdata.parse_float())?;
        let initial_delay_min_value = element
            .get_sub_element(ElementName::InitialDelayMinValue)
            .and_then(|rrd| rrd.character_data())
            .and_then(|cdata| cdata.parse_float())?;
        let initial_repetitions_base_delay = element
            .get_sub_element(ElementName::InitialRepetitionsBaseDelay)
            .and_then(|rrd| rrd.character_data())
            .and_then(|cdata| cdata.parse_float());
        let initial_repetitions_max = element
            .get_sub_element(ElementName::InitialRepetitionsMax)
            .and_then(|rrd| rrd.character_data())
            .and_then(|cdata| cdata.parse_integer());
        Some(Self {
            initial_delay_max_value,
            initial_delay_min_value,
            initial_repetitions_base_delay,
            initial_repetitions_max,
        })
    }
}

//##################################################################

/// A `SomipTpConfig` contains the configuration of individual `SomeIp` TP connections
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SomeipTpConfig(Element);
abstraction_element!(SomeipTpConfig, SomeipTpConfig);
impl IdentifiableAbstractionElement for SomeipTpConfig {}

impl SomeipTpConfig {
    pub(crate) fn new(name: &str, package: &ArPackage, cluster: &Cluster) -> Result<Self, AutosarAbstractionError> {
        let pkg_elem = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem = pkg_elem.create_named_sub_element(ElementName::SomeipTpConfig, name)?;

        elem.create_sub_element(ElementName::CommunicationClusterRef)?
            .set_reference_target(cluster.element())?;

        Ok(Self(elem))
    }

    /// get the communication cluster of this `SomeipTpConfig`
    #[must_use]
    pub fn cluster(&self) -> Option<Cluster> {
        self.element()
            .get_sub_element(ElementName::CommunicationClusterRef)
            .and_then(|ccr| ccr.get_reference_target().ok())
            .and_then(|target| Cluster::try_from(target).ok())
    }

    /// create a new `SomeipTpChannel` in this `SomeipTpConfig`
    ///
    /// version >= `AUTOSAR_00046`
    pub fn create_someip_tp_channel(&self, name: &str) -> Result<SomeipTpChannel, AutosarAbstractionError> {
        let channels = self.element().get_or_create_sub_element(ElementName::TpChannels)?;
        SomeipTpChannel::new(name, &channels)
    }

    /// iterate over all `SomeipTpChannel`s in this `SomeipTpConfig`
    pub fn someip_tp_channels(&self) -> impl Iterator<Item = SomeipTpChannel> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::TpChannels)
            .into_iter()
            .flat_map(|channels| channels.sub_elements())
            .filter_map(|channel| SomeipTpChannel::try_from(channel).ok())
    }

    /// create a new `SomeIp` TP connection in this `SomeipTpConfig`
    ///
    /// returns the `PduTriggering` that is created for the `TpSdu`
    pub fn create_someip_tp_connection(
        &self,
        tp_sdu: &ISignalIPdu,
        transport_pdu_triggering: &PduTriggering,
        tp_channel: Option<SomeipTpChannel>,
    ) -> Result<SomeipTpConnection, AutosarAbstractionError> {
        let connections = self.element().get_or_create_sub_element(ElementName::TpConnections)?;
        SomeipTpConnection::new(&connections, tp_sdu, transport_pdu_triggering, tp_channel)
    }

    /// get all `SomeipTpConnection`s in this `SomeipTpConfig`
    pub fn someip_tp_connections(&self) -> impl Iterator<Item = SomeipTpConnection> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::TpConnections)
            .into_iter()
            .flat_map(|connections| connections.sub_elements())
            .filter_map(|conn| SomeipTpConnection::try_from(conn).ok())
    }
}

//##################################################################

/// A `SomeipTpConnection` contains the configuration of a single `SomeIp` TP connection
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SomeipTpConnection(Element);
abstraction_element!(SomeipTpConnection, SomeipTpConnection);

impl SomeipTpConnection {
    pub(crate) fn new(
        parent: &Element,
        tp_sdu: &ISignalIPdu,
        transport_pdu_triggering: &PduTriggering,
        tp_channel: Option<SomeipTpChannel>,
    ) -> Result<Self, AutosarAbstractionError> {
        let conn_elem = parent.create_sub_element(ElementName::SomeipTpConnection)?;
        let conn = Self(conn_elem);

        conn.set_transport_pdu_triggering(transport_pdu_triggering)?;
        conn.set_tp_sdu(tp_sdu)?;
        conn.set_tp_channel(tp_channel)?;

        Ok(conn)
    }

    /// get the `SomeipTpConfig` that contains this `SomeipTpConnection`
    pub fn someip_tp_config(&self) -> Result<SomeipTpConfig, AutosarAbstractionError> {
        let parent = self.element().named_parent()?.unwrap();
        SomeipTpConfig::try_from(parent)
    }

    /// set the `PduTriggering` for the transport PDU of this `SomeipTpConnection`
    pub fn set_transport_pdu_triggering(
        &self,
        transport_pdu_triggering: &PduTriggering,
    ) -> Result<(), AutosarAbstractionError> {
        // check if the transport PDU is a GeneralPurposeIPdu
        let Some(Pdu::GeneralPurposeIPdu(gp_ipdu)) = transport_pdu_triggering.pdu() else {
            return Err(AutosarAbstractionError::InvalidParameter(
                "Invalid transport PDU for the SomeIpTpConnection: it must be a GeneralPurposeIPdu".to_string(),
            ));
        };

        // check the category of the GeneralPurposeIPdu: according to the AUTOSAR standard, it must be SOMEIP_SEGMENTED_IPDU
        if gp_ipdu.category() != Some(GeneralPurposeIPduCategory::SomeipSegmentedIpdu) {
            return Err(AutosarAbstractionError::InvalidParameter(
                "Invalid transport PDU for the SomeIpTpConnection: it must be a segmented IPDU".to_string(),
            ));
        }

        // get the physical channel of the transport PDU; this is currently the only link to the channel
        let channel = transport_pdu_triggering.physical_channel()?;
        // get the cluster of the physical channel and check if it matches the cluster of the SomeIpTpConfig
        let Some(channel_cluster) = channel
            .element()
            .named_parent()?
            .and_then(|p| Cluster::try_from(p).ok())
        else {
            return Err(AutosarAbstractionError::InvalidParameter(
                "Invalid physical channel or cluster of the transport PDU".to_string(),
            ));
        };
        let Some(cluster) = self.someip_tp_config()?.cluster() else {
            return Err(AutosarAbstractionError::InvalidParameter(
                "Invalid SomeIpTpConfig: missing cluster reference".to_string(),
            ));
        };
        if channel_cluster != cluster {
            return Err(AutosarAbstractionError::InvalidParameter(
                "The transport PDU must be in the same cluster as the SomeIpTpConfig".to_string(),
            ));
        }

        self.element()
            .create_sub_element(ElementName::TransportPduRef)?
            .set_reference_target(transport_pdu_triggering.element())?;
        Ok(())
    }

    /// get the `PduTriggering` for the transport PDU of this `SomeipTpConnection`
    #[must_use]
    pub fn transport_pdu_triggering(&self) -> Option<PduTriggering> {
        self.element()
            .get_sub_element(ElementName::TransportPduRef)
            .and_then(|ref_elem| ref_elem.get_reference_target().ok())
            .and_then(|target| PduTriggering::try_from(target).ok())
    }

    /// set the `TpSdu` of this `SomeipTpConnection`
    pub fn set_tp_sdu(&self, tp_sdu: &ISignalIPdu) -> Result<(), AutosarAbstractionError> {
        let Some(transport_pdu_triggering) = self.transport_pdu_triggering() else {
            return Err(AutosarAbstractionError::InvalidParameter(
                "The transport PDU of the SomeipTpConnection is missing, so the TP-SDU can't be created".to_string(),
            ));
        };
        let channel = transport_pdu_triggering.physical_channel()?;

        // create the PduTriggering for the TpSdu in the same cluster as the transport PDU
        let pt_tp_sdu = PduTriggering::new(&tp_sdu.clone().into(), &channel)?;

        self.element()
            .create_sub_element(ElementName::TpSduRef)?
            .set_reference_target(pt_tp_sdu.element())?;
        Ok(())
    }

    /// get the `TpSdu` of this `SomeipTpConnection`
    #[must_use]
    pub fn tp_sdu(&self) -> Option<ISignalIPdu> {
        let tp_sdu_triggering_elem = self
            .element()
            .get_sub_element(ElementName::TpSduRef)?
            .get_reference_target()
            .ok()?;
        let tp_sdu_triggering = PduTriggering::try_from(tp_sdu_triggering_elem).ok()?;

        if let Some(Pdu::ISignalIPdu(tp_sdu)) = tp_sdu_triggering.pdu() {
            Some(tp_sdu)
        } else {
            None
        }
    }

    /// set the `TpChannel` of this `SomeipTpConnection`
    pub fn set_tp_channel(&self, tp_channel: Option<SomeipTpChannel>) -> Result<(), AutosarAbstractionError> {
        if let Some(tp_channel) = tp_channel {
            self.element()
                .get_or_create_sub_element(ElementName::TpChannelRef)?
                .set_reference_target(tp_channel.element())?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::TpChannelRef);
        }
        Ok(())
    }

    /// get the `TpChannel` of this `SomeipTpConnection`
    #[must_use]
    pub fn tp_channel(&self) -> Option<SomeipTpChannel> {
        self.element()
            .get_sub_element(ElementName::TpChannelRef)
            .and_then(|ref_elem| ref_elem.get_reference_target().ok())
            .and_then(|target| SomeipTpChannel::try_from(target).ok())
    }
}

//##################################################################

/// General settings for a `SomeIp` TP channel
///
/// version >= `AUTOSAR_00046`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SomeipTpChannel(Element);
abstraction_element!(SomeipTpChannel, SomeipTpChannel);
impl IdentifiableAbstractionElement for SomeipTpChannel {}

impl SomeipTpChannel {
    pub(crate) fn new(name: &str, parent: &Element) -> Result<Self, AutosarAbstractionError> {
        let elem = parent.create_named_sub_element(ElementName::SomeipTpChannel, name)?;
        Ok(Self(elem))
    }

    /// set the rxTimeoutTime for the `SomeIpTpChannel`
    pub fn set_rx_timeout_time(&self, rx_timeout_time: f64) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::RxTimeoutTime)?
            .set_character_data(rx_timeout_time)?;
        Ok(())
    }

    /// get the rxTimeoutTime for the `SomeIpTpChannel`
    #[must_use]
    pub fn rx_timeout_time(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::RxTimeoutTime)
            .and_then(|rtt| rtt.character_data())
            .and_then(|cdata| cdata.parse_float())
    }

    /// set the separationTime for the `SomeIpTpChannel`
    pub fn set_separation_time(&self, separation_time: f64) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::SeparationTime)?
            .set_character_data(separation_time)?;
        Ok(())
    }

    /// get the separationTime for the `SomeIpTpChannel`
    #[must_use]
    pub fn separation_time(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::SeparationTime)
            .and_then(|st| st.character_data())
            .and_then(|cdata| cdata.parse_float())
    }
}

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;
    use autosar_data::AutosarVersion;
    use communication::{EthernetVlanInfo, NetworkEndpointAddress, PduCollectionTrigger, SocketAddressType};

    /// helper function to create a test setup with:
    /// - a system
    /// - an ethernet cluster
    ///   - a physical channel
    ///   - a network endpoint
    ///   - a socket address
    fn helper_create_test_objects(model: &AutosarModelAbstraction) -> SocketAddress {
        let package = model.get_or_create_package("/ethernet").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();
        let cluster = system.create_ethernet_cluster("ethcluster", &package).unwrap();
        let channel = cluster
            .create_physical_channel(
                "channel",
                Some(EthernetVlanInfo {
                    vlan_name: "VLAN_02".to_string(),
                    vlan_id: 2,
                }),
            )
            .unwrap();
        let network_endpoint_address = NetworkEndpointAddress::IPv4 {
            address: Some("192.168.2.222".to_string()),
            address_source: None,
            default_gateway: None,
            network_mask: None,
        };
        let network_endpoint = channel
            .create_network_endpoint("endpoint", network_endpoint_address, None)
            .unwrap();
        let tp_config = TpConfig::UdpTp {
            port_number: Some(1234),
            port_dynamically_assigned: None,
        };
        channel
            .create_socket_address(
                "socket",
                &network_endpoint,
                &tp_config,
                SocketAddressType::Unicast(None),
            )
            .unwrap()
    }

    #[test]
    fn test_service_instance_collection_set() {
        let model = AutosarModelAbstraction::create("file", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();

        let si_set = system
            .create_service_instance_collection_set("service_instance_collection_set", &package)
            .unwrap();
        assert_eq!(si_set.name().unwrap(), "service_instance_collection_set");

        let psi = si_set
            .create_provided_service_instance("ProvidedInstance", 1, 1, 1, 0)
            .unwrap();
        let csi = si_set
            .create_consumed_service_instance("ConsumedInstance", 1, 1, 1, "1")
            .unwrap();

        assert_eq!(si_set.service_instances().count(), 2);
        let service_instances: Vec<ServiceInstance> = si_set.service_instances().collect();
        assert_eq!(service_instances[0].element(), psi.element());
        assert_eq!(service_instances[0], ServiceInstance::Provided(psi));
        assert_eq!(service_instances[1].element(), csi.element());
        assert_eq!(service_instances[1], ServiceInstance::Consumed(csi));
    }

    #[test]
    fn test_provided_service_instance() {
        let model = AutosarModelAbstraction::create("file", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();

        let socket = helper_create_test_objects(&model);

        let si_set = system
            .create_service_instance_collection_set("service_instance_collection_set", &package)
            .unwrap();
        let psi = si_set
            .create_provided_service_instance("ProvidedInstance", 1, 1, 1, 0)
            .unwrap();

        psi.set_local_unicast_address(&socket).unwrap();

        assert_eq!(psi.service_identifier().unwrap(), 1);
        assert_eq!(psi.instance_identifier().unwrap(), 1);
        assert_eq!(psi.major_version().unwrap(), 1);
        assert_eq!(psi.minor_version().unwrap(), 0);
        assert_eq!(
            psi.local_unicast_addresses().next(),
            Some(LocalUnicastAddress::Udp(socket.clone()))
        );

        let eh = psi.create_event_handler("EventHandler", 1).unwrap();
        assert_eq!(eh.event_group_identifier().unwrap(), 1);

        let sd_config_package = model.get_or_create_package("/SomeipSdTimingConfigs").unwrap();
        let server_service_instance_config =
            SomeipSdServerServiceInstanceConfig::new("ssssic", &sd_config_package, 10).unwrap();
        psi.set_sd_server_instance_config(&server_service_instance_config)
            .unwrap();
        assert_eq!(psi.sd_server_instance_config().unwrap(), server_service_instance_config);
    }

    #[test]
    fn test_event_handler() {
        let model = AutosarModelAbstraction::create("file", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();

        let si_set = system
            .create_service_instance_collection_set("service_instance_collection_set", &package)
            .unwrap();
        let psi = si_set
            .create_provided_service_instance("ProvidedInstance", 1, 1, 1, 0)
            .unwrap();
        assert_eq!(psi.event_handlers().count(), 0);
        let eh = psi.create_event_handler("EventHandler", 1).unwrap();
        assert_eq!(psi.event_handlers().count(), 1);
        assert_eq!(eh.event_group_identifier().unwrap(), 1);

        let prg = eh
            .create_pdu_activation_routing_group("PduActivationRoutingGroup", EventGroupControlType::ActivationUnicast)
            .unwrap();
        assert_eq!(eh.pdu_activation_routing_groups().count(), 1);
        assert_eq!(
            prg.event_group_control_type().unwrap(),
            EventGroupControlType::ActivationUnicast
        );

        let sd_config_package = model.get_or_create_package("/SomeipSdTimingConfigs").unwrap();
        let rrd = RequestResponseDelay {
            min_value: 1.0,
            max_value: 2.0,
        };
        let server_event_group_timing_config =
            SomeipSdServerEventGroupTimingConfig::new("segtc", &sd_config_package, &rrd).unwrap();
        eh.set_sd_server_event_group_timing_config(&server_event_group_timing_config)
            .unwrap();
        assert_eq!(
            eh.sd_server_event_group_timing_config().unwrap(),
            server_event_group_timing_config
        );
    }

    #[test]
    fn server_sd_config() {
        let model = AutosarModelAbstraction::create("file", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();

        let si_set = system
            .create_service_instance_collection_set("service_instance_collection_set", &package)
            .unwrap();
        let psi = si_set
            .create_provided_service_instance("ProvidedInstance", 1, 1, 1, 0)
            .unwrap();
        let eh = psi.create_event_handler("EventHandler", 1).unwrap();

        let rrd = RequestResponseDelay {
            min_value: 1.0,
            max_value: 2.0,
        };

        // SD server instance config for the ProvidedServiceInstance
        let sd_server_instance_config = SomeipSdServerServiceInstanceConfig::new("ssssic", &package, 10).unwrap();
        assert_eq!(sd_server_instance_config.service_offer_time_to_live().unwrap(), 10);
        sd_server_instance_config.set_service_offer_time_to_live(20).unwrap();
        assert_eq!(sd_server_instance_config.service_offer_time_to_live().unwrap(), 20);
        sd_server_instance_config.set_offer_cyclic_delay(1.0).unwrap();
        assert_eq!(sd_server_instance_config.offer_cyclic_delay().unwrap(), 1.0);
        // priority is only available since R21-11 (AUTOSAR_00050)! in older versions it will not be set
        sd_server_instance_config.set_priority(5).unwrap();
        assert_eq!(sd_server_instance_config.priority().unwrap(), 5);

        let initial_offer_behavior = InitialSdDelayConfig {
            initial_delay_max_value: 1.0,
            initial_delay_min_value: 0.5,
            initial_repetitions_base_delay: Some(0.1),
            initial_repetitions_max: Some(5),
        };
        sd_server_instance_config
            .set_initial_offer_behavior(&initial_offer_behavior)
            .unwrap();
        assert_eq!(
            sd_server_instance_config.initial_offer_behavior().unwrap(),
            initial_offer_behavior
        );
        sd_server_instance_config.set_request_response_delay(&rrd).unwrap();
        assert_eq!(sd_server_instance_config.request_response_delay().unwrap(), rrd);

        psi.set_sd_server_instance_config(&sd_server_instance_config).unwrap();
        assert_eq!(psi.sd_server_instance_config().unwrap(), sd_server_instance_config);

        // SD server event group timing config for the EventHandler
        let sd_server_event_group_timing_config =
            SomeipSdServerEventGroupTimingConfig::new("segtc", &package, &rrd).unwrap();
        assert_eq!(
            sd_server_event_group_timing_config.request_response_delay().unwrap(),
            rrd
        );
        sd_server_event_group_timing_config
            .set_request_response_delay(&rrd)
            .unwrap();
        assert_eq!(
            sd_server_event_group_timing_config.request_response_delay().unwrap(),
            rrd
        );

        eh.set_sd_server_event_group_timing_config(&sd_server_event_group_timing_config)
            .unwrap();
        assert_eq!(
            eh.sd_server_event_group_timing_config().unwrap(),
            sd_server_event_group_timing_config
        );
    }

    #[test]
    fn test_consumed_service_instance() {
        let model = AutosarModelAbstraction::create("file", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();

        let socket = helper_create_test_objects(&model);

        let si_set = system
            .create_service_instance_collection_set("service_instance_collection_set", &package)
            .unwrap();
        let csi = si_set
            .create_consumed_service_instance("ConsumedInstance", 1, 1, 1, "1")
            .unwrap();

        csi.set_local_unicast_address(&socket).unwrap();

        assert_eq!(csi.service_identifier().unwrap(), 1);
        assert_eq!(csi.instance_identifier().unwrap(), 1);
        assert_eq!(csi.major_version().unwrap(), 1);
        assert_eq!(csi.minor_version().unwrap(), "1");
        assert_eq!(
            csi.local_unicast_addresses().next(),
            Some(LocalUnicastAddress::Udp(socket.clone()))
        );

        let ceg = csi.create_consumed_event_group("EventGroup", 1).unwrap();
        assert_eq!(ceg.event_group_identifier().unwrap(), 1);

        let sd_config_package = model.get_or_create_package("/SomeipSdTimingConfigs").unwrap();
        let client_service_instance_config =
            SomeipSdClientServiceInstanceConfig::new("cscic", &sd_config_package).unwrap();
        csi.set_sd_client_instance_config(&client_service_instance_config)
            .unwrap();
        assert_eq!(csi.sd_client_instance_config().unwrap(), client_service_instance_config);
    }

    #[test]
    fn test_consumed_event_group() {
        let model = AutosarModelAbstraction::create("file", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();

        let socket = helper_create_test_objects(&model);

        let si_set = system
            .create_service_instance_collection_set("service_instance_collection_set", &package)
            .unwrap();
        let csi = si_set
            .create_consumed_service_instance("ConsumedInstance", 1, 1, 1, "1")
            .unwrap();
        assert_eq!(csi.consumed_event_groups().count(), 0);
        let ceg = csi.create_consumed_event_group("EventGroup", 1).unwrap();
        assert_eq!(csi.consumed_event_groups().count(), 1);
        assert_eq!(ceg.event_group_identifier().unwrap(), 1);

        ceg.add_event_multicast_address(&socket).unwrap();
        assert_eq!(ceg.event_multicast_addresses().next().unwrap(), socket);

        let prg = ceg
            .create_pdu_activation_routing_group(
                "PduActivationRoutingGroup",
                EventGroupControlType::ActivationMulticast,
            )
            .unwrap();
        assert_eq!(ceg.pdu_activation_routing_groups().count(), 1);
        assert_eq!(
            prg.event_group_control_type().unwrap(),
            EventGroupControlType::ActivationMulticast
        );

        let sd_config_package = model.get_or_create_package("/SomeipSdTimingConfigs").unwrap();
        let client_event_group_timing_config =
            SomeipSdClientEventGroupTimingConfig::new("cegtc", &sd_config_package, 10).unwrap();
        ceg.set_sd_client_timer_config(&client_event_group_timing_config)
            .unwrap();
        assert_eq!(ceg.sd_client_timer_config().unwrap(), client_event_group_timing_config);
    }

    #[test]
    fn client_sd_config() {
        let model = AutosarModelAbstraction::create("file", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();

        let si_set = system
            .create_service_instance_collection_set("service_instance_collection_set", &package)
            .unwrap();
        let csi = si_set
            .create_consumed_service_instance("ConsumedInstance", 1, 1, 1, "1")
            .unwrap();
        let ceg = csi.create_consumed_event_group("EventGroup", 1).unwrap();

        let rrd = RequestResponseDelay {
            min_value: 1.0,
            max_value: 2.0,
        };

        // SD client instance config for the ConsumedServiceInstance
        let sd_client_instance_config = SomeipSdClientServiceInstanceConfig::new("cscic", &package).unwrap();
        csi.set_sd_client_instance_config(&sd_client_instance_config).unwrap();
        assert_eq!(csi.sd_client_instance_config().unwrap(), sd_client_instance_config);

        let initial_find_behavior = InitialSdDelayConfig {
            initial_delay_max_value: 1.0,
            initial_delay_min_value: 0.5,
            initial_repetitions_base_delay: Some(0.1),
            initial_repetitions_max: Some(5),
        };
        sd_client_instance_config
            .set_initial_find_behavior(&initial_find_behavior)
            .unwrap();
        assert_eq!(
            sd_client_instance_config.initial_find_behavior().unwrap(),
            initial_find_behavior
        );
        // priority is only available since R21-11 (AUTOSAR_00050)! in older versions it will not be set
        sd_client_instance_config.set_priority(5).unwrap();
        assert_eq!(sd_client_instance_config.priority().unwrap(), 5);

        // SD client event group timing config for the ConsumedEventGroup
        let sd_client_event_group_timing_config =
            SomeipSdClientEventGroupTimingConfig::new("cegtc", &package, 10).unwrap();
        assert_eq!(sd_client_event_group_timing_config.time_to_live().unwrap(), 10);
        sd_client_event_group_timing_config.set_time_to_live(20).unwrap();
        assert_eq!(sd_client_event_group_timing_config.time_to_live().unwrap(), 20);
        sd_client_event_group_timing_config
            .set_request_response_delay(&rrd)
            .unwrap();
        assert_eq!(
            sd_client_event_group_timing_config.request_response_delay().unwrap(),
            rrd
        );
        sd_client_event_group_timing_config
            .set_subscribe_eventgroup_retry_delay(1.0)
            .unwrap();
        assert_eq!(
            sd_client_event_group_timing_config
                .subscribe_eventgroup_retry_delay()
                .unwrap(),
            1.0
        );
        sd_client_event_group_timing_config
            .set_subscribe_eventgroup_retry_max(5)
            .unwrap();
        assert_eq!(
            sd_client_event_group_timing_config
                .subscribe_eventgroup_retry_max()
                .unwrap(),
            5
        );

        ceg.set_sd_client_timer_config(&sd_client_event_group_timing_config)
            .unwrap();
        assert_eq!(
            ceg.sd_client_timer_config().unwrap(),
            sd_client_event_group_timing_config
        );
    }

    #[test]
    fn test_local_unicast_addresses() {
        let model = AutosarModelAbstraction::create("file", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();

        let udp_socket = helper_create_test_objects(&model);
        let network_endpoint = udp_socket.network_endpoint().unwrap();
        let channel = udp_socket.physical_channel().unwrap();
        let tcp_socket = channel
            .create_socket_address(
                "tcp_socket",
                &network_endpoint,
                &TpConfig::TcpTp {
                    port_number: Some(1234),
                    port_dynamically_assigned: None,
                },
                SocketAddressType::Unicast(None),
            )
            .unwrap();
        let udp_socket_2 = channel
            .create_socket_address(
                "udp_socket_2",
                &network_endpoint,
                &TpConfig::UdpTp {
                    port_number: Some(1235),
                    port_dynamically_assigned: None,
                },
                SocketAddressType::Unicast(None),
            )
            .unwrap();

        let si_set = system
            .create_service_instance_collection_set("service_instance_collection_set", &package)
            .unwrap();
        let psi = si_set
            .create_provided_service_instance("ProvidedInstance", 1, 1, 1, 0)
            .unwrap();

        // there are no local unicast addresses yet
        assert_eq!(psi.local_unicast_addresses().count(), 0);

        // add the UDP socket
        psi.set_local_unicast_address(&udp_socket).unwrap();
        assert_eq!(psi.local_unicast_addresses().count(), 1);

        // add the TCP socket
        psi.set_local_unicast_address(&tcp_socket).unwrap();
        assert_eq!(psi.local_unicast_addresses().count(), 2);

        // add the second UDP socket, replacing the first one
        psi.set_local_unicast_address(&udp_socket_2).unwrap();
        assert_eq!(psi.local_unicast_addresses().count(), 2);
    }

    #[test]
    fn test_pdus() {
        let model = AutosarModelAbstraction::create("file", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();
        let cluster = system.create_ethernet_cluster("ethcluster", &package).unwrap();
        let channel = cluster
            .create_physical_channel(
                "channel",
                Some(EthernetVlanInfo {
                    vlan_name: "VLAN_02".to_string(),
                    vlan_id: 2,
                }),
            )
            .unwrap();
        let ipdu = system.create_isignal_ipdu("pdu", &package, 222).unwrap();

        let si_set = system
            .create_service_instance_collection_set("service_instance_collection_set", &package)
            .unwrap();
        let psi = si_set
            .create_provided_service_instance("ProvidedInstance", 1, 1, 1, 0)
            .unwrap();
        let eh = psi.create_event_handler("EventHandler", 1).unwrap();
        let csi = si_set
            .create_consumed_service_instance("ConsumedInstance", 1, 1, 1, "1")
            .unwrap();
        let ceg = csi.create_consumed_event_group("EventGroup", 1).unwrap();

        let psi_prg = eh
            .create_pdu_activation_routing_group("PduActivationRoutingGroup", EventGroupControlType::ActivationUnicast)
            .unwrap();
        let _csi_pgr = ceg
            .create_pdu_activation_routing_group("PduActivationRoutingGroup", EventGroupControlType::ActivationUnicast)
            .unwrap();

        let ipdu_identifier_set = system
            .create_socket_connection_ipdu_identifier_set("socon_ipdu_id", &package)
            .unwrap();
        let ipdu_identifier = ipdu_identifier_set
            .create_socon_ipdu_identifier(
                "ipdu_id",
                &ipdu,
                &channel,
                Some(1),
                None,
                Some(PduCollectionTrigger::Always),
            )
            .unwrap();
        assert_eq!(ipdu_identifier_set.socon_ipdu_identifiers().count(), 1);
        psi_prg.add_ipdu_identifier_udp(&ipdu_identifier).unwrap();
        assert_eq!(psi_prg.ipdu_identifiers_udp().count(), 1);
        psi_prg.add_ipdu_identifier_tcp(&ipdu_identifier).unwrap();
        assert_eq!(psi_prg.ipdu_identifiers_tcp().count(), 1);
    }

    #[test]
    fn test_conversion() {
        let model = AutosarModelAbstraction::create("file", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();

        let si_set = system
            .create_service_instance_collection_set("service_instance_collection_set", &package)
            .unwrap();
        let psi = si_set
            .create_provided_service_instance("ProvidedInstance", 1, 1, 1, 0)
            .unwrap();
        let element = psi.element().clone();
        let psi2 = ProvidedServiceInstance::try_from(element).unwrap();
        assert_eq!(psi, psi2);

        let eh = psi.create_event_handler("EventHandler", 1).unwrap();
        let element = eh.element().clone();
        let eh2 = EventHandler::try_from(element).unwrap();
        assert_eq!(eh, eh2);

        let csi = si_set
            .create_consumed_service_instance("ConsumedInstance", 1, 1, 1, "1")
            .unwrap();
        let element = csi.element().clone();
        let csi2 = ConsumedServiceInstance::try_from(element).unwrap();
        assert_eq!(csi, csi2);

        let ceg = csi.create_consumed_event_group("EventGroup", 1).unwrap();
        let element = ceg.element().clone();
        let ceg2 = ConsumedEventGroup::try_from(element).unwrap();
        assert_eq!(ceg, ceg2);

        // create someip configuration items using the old structure
        let socket = helper_create_test_objects(&model);
        let ae = socket
            .element()
            .get_sub_element(ElementName::ApplicationEndpoint)
            .unwrap();

        // prove that the conversion of an old ProvidedServiceInstance fails
        let psi_old_elem = ae
            .create_sub_element(ElementName::ProvidedServiceInstances)
            .unwrap()
            .create_named_sub_element(ElementName::ProvidedServiceInstance, "PSI")
            .unwrap();
        let result = ProvidedServiceInstance::try_from(psi_old_elem.clone());
        assert!(result.is_err());

        // prove that the conversion of an old EventHandler fails
        let eh_old_elem = psi_old_elem
            .create_sub_element(ElementName::EventHandlers)
            .unwrap()
            .create_named_sub_element(ElementName::EventHandler, "EH")
            .unwrap();
        let result = EventHandler::try_from(eh_old_elem);
        assert!(result.is_err());

        // prove that the conversion of an old ConsumedServiceInstance fails
        let csi_old_elem = ae
            .create_sub_element(ElementName::ConsumedServiceInstances)
            .unwrap()
            .create_named_sub_element(ElementName::ConsumedServiceInstance, "CSI")
            .unwrap();
        let result = ConsumedServiceInstance::try_from(csi_old_elem.clone());
        assert!(result.is_err());

        // prove that the conversion of an old ConsumedEventGroup fails
        let ceg_old_elem = csi_old_elem
            .create_sub_element(ElementName::ConsumedEventGroups)
            .unwrap()
            .create_named_sub_element(ElementName::ConsumedEventGroup, "CEG")
            .unwrap();
        let result = ConsumedEventGroup::try_from(ceg_old_elem);
        assert!(result.is_err());
    }

    #[test]
    fn someip_tp() {
        let model = AutosarModelAbstraction::create("file", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();
        let cluster = system.create_ethernet_cluster("ethcluster", &package).unwrap();
        let channel = cluster
            .create_physical_channel(
                "channel",
                Some(EthernetVlanInfo {
                    vlan_name: "VLAN_02".to_string(),
                    vlan_id: 2,
                }),
            )
            .unwrap();

        let gp_ipdu = system
            .create_general_purpose_ipdu(
                "gp_ipdu",
                &package,
                1400,
                GeneralPurposeIPduCategory::SomeipSegmentedIpdu,
            )
            .unwrap();
        let isignal_ipdu = system.create_isignal_ipdu("isignal_ipdu", &package, 12000).unwrap();

        let ipdu_identifier_set = system
            .create_socket_connection_ipdu_identifier_set("socon_ipdu_id", &package)
            .unwrap();
        let ipdu_identifier = ipdu_identifier_set
            .create_socon_ipdu_identifier(
                "ipdu_id",
                &gp_ipdu,
                &channel,
                Some(1),
                None,
                Some(PduCollectionTrigger::Always),
            )
            .unwrap();
        let transport_pdu_triggering = ipdu_identifier.pdu_triggering().unwrap();

        let tp_config = system
            .create_somip_tp_config("someip_tp_config", &package, &cluster)
            .unwrap();

        let tp_channel = tp_config.create_someip_tp_channel("someip_tp_channel").unwrap();
        assert_eq!(tp_config.someip_tp_channels().count(), 1);
        tp_channel.set_rx_timeout_time(0.33).unwrap();
        assert_eq!(tp_channel.rx_timeout_time().unwrap(), 0.33);
        tp_channel.set_separation_time(0.44).unwrap();
        assert_eq!(tp_channel.separation_time().unwrap(), 0.44);

        let tp_conn = tp_config
            .create_someip_tp_connection(&isignal_ipdu, &transport_pdu_triggering, Some(tp_channel.clone()))
            .unwrap();
        assert_eq!(tp_config.someip_tp_connections().count(), 1);
        assert_eq!(tp_config.someip_tp_connections().next().unwrap(), tp_conn);
        assert_eq!(tp_conn.tp_sdu(), Some(isignal_ipdu));
        assert_eq!(tp_conn.tp_channel(), Some(tp_channel));
        assert_eq!(tp_conn.transport_pdu_triggering(), Some(transport_pdu_triggering));
        assert_eq!(tp_conn.someip_tp_config().unwrap(), tp_config);
    }
}
