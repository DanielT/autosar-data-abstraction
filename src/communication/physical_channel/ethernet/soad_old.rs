use crate::communication::{
    AbstractPdu, EthernetPhysicalChannel, EventGroupControlType, Pdu, PduCollectionTrigger, PduTriggering,
    PhysicalChannel, SocketAddress, TpConfig,
};
use crate::{
    abstraction_element, AbstractionElement, ArPackage, AutosarAbstractionError, IdentifiableAbstractionElement,
};
use autosar_data::{Element, ElementName, EnumItem};

//##################################################################

/// A `SocketConnectionBundle` describes a connection between a server port and multiple client ports.
/// It contains multiple bundled connections, each transporting one or more PDUs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SocketConnectionBundle(Element);
abstraction_element!(SocketConnectionBundle, SocketConnectionBundle);
impl IdentifiableAbstractionElement for SocketConnectionBundle {}

impl SocketConnectionBundle {
    pub(crate) fn new(
        name: &str,
        server_port: &SocketAddress,
        connections_elem: &Element,
    ) -> Result<Self, AutosarAbstractionError> {
        let scb = connections_elem.create_named_sub_element(ElementName::SocketConnectionBundle, name)?;

        scb.create_sub_element(ElementName::ServerPortRef)?
            .set_reference_target(server_port.element())?;

        Ok(Self(scb))
    }

    /// get the physical channel containing this socket connection bundle
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # use autosar_data_abstraction::communication::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// # let package = model.get_or_create_package("/pkg1")?;
    /// # let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// # let cluster = system.create_ethernet_cluster("Cluster", &package)?;
    /// # let channel = cluster.create_physical_channel("Channel", None)?;
    /// # let server_endpoint = channel.create_network_endpoint("ServerAddress", NetworkEndpointAddress::IPv4 {
    /// #    address: Some("192.168.0.1".to_string()),
    /// #    address_source: Some(IPv4AddressSource::Fixed),
    /// #    default_gateway: None,
    /// #    network_mask: None
    /// # }, None)?;
    /// # let server_socket = channel.create_socket_address("ServerSocket", &server_endpoint, &TpConfig::TcpTp { port_number: Some(1234), port_dynamically_assigned: None }, SocketAddressType::Unicast(None))?;
    /// # let client_endpoint = channel.create_network_endpoint("ClientAddress", NetworkEndpointAddress::IPv4 {
    /// #    address: Some("192.168.0.2".to_string()),
    /// #    address_source: Some(IPv4AddressSource::Fixed),
    /// #    default_gateway: None,
    /// #    network_mask: None
    /// # }, None)?;
    /// # let client_socket = channel.create_socket_address("ClientSocket", &client_endpoint, &TpConfig::TcpTp { port_number: Some(1235), port_dynamically_assigned: None }, SocketAddressType::Unicast(None))?;
    /// let bundle = channel.create_socket_connection_bundle("Bundle", &server_socket)?;
    /// assert_eq!(channel, bundle.physical_channel()?);
    /// # Ok(())}
    /// ```
    pub fn physical_channel(&self) -> Result<EthernetPhysicalChannel, AutosarAbstractionError> {
        let channel = self.element().named_parent()?.unwrap();
        EthernetPhysicalChannel::try_from(channel)
    }

    /// get the server port of this socket connection bundle
    #[must_use]
    pub fn server_port(&self) -> Option<SocketAddress> {
        self.element()
            .get_sub_element(ElementName::ServerPortRef)
            .and_then(|spr| spr.get_reference_target().ok())
            .and_then(|sp| SocketAddress::try_from(sp).ok())
    }

    /// create a bundled `SocketConnection` between the server port and a client port
    pub fn create_bundled_connection(
        &self,
        client_port: &SocketAddress,
    ) -> Result<SocketConnection, AutosarAbstractionError> {
        let Some(server_port) = self.server_port() else {
            return Err(AutosarAbstractionError::InvalidParameter(
                "SocketConnectionBundle has no server port".to_string(),
            ));
        };
        let own_tp_config = server_port.tp_config();
        let remote_tp_config = client_port.tp_config();
        match (own_tp_config, remote_tp_config) {
            (Some(TpConfig::TcpTp { .. }), Some(TpConfig::TcpTp { .. }))
            | (Some(TpConfig::UdpTp { .. }), Some(TpConfig::UdpTp { .. }))
            | (None, None) => { /* ok */ }
            _ => {
                return Err(AutosarAbstractionError::InvalidParameter(
                    "Both SocketAddresses must use the same transport protocol".to_string(),
                ))
            }
        }
        let conn = self
            .0
            .get_or_create_sub_element(ElementName::BundledConnections)?
            .create_sub_element(ElementName::SocketConnection)?;

        SocketConnection::new(conn, client_port)
    }

    /// create an iterator over all bundled connections in this socket connection bundle
    pub fn bundled_connections(&self) -> impl Iterator<Item = SocketConnection> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::BundledConnections)
            .into_iter()
            .flat_map(|bc| bc.sub_elements())
            .filter_map(|elem| SocketConnection::try_from(elem).ok())
    }
}

//##################################################################

/// A socketConnection inside a `SocketConnectionBundle` describes a single connection to a specific client port.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SocketConnection(Element);
abstraction_element!(SocketConnection, SocketConnection);

impl SocketConnection {
    /// The PDU header id for SD messages must always be set to `0xFFFF_8100`
    pub const SD_HEADER_ID: u32 = 0xFFFF_8100;

    // create a new SocketConnection (internal)
    pub(crate) fn new(conn: Element, client_port: &SocketAddress) -> Result<Self, AutosarAbstractionError> {
        conn.create_sub_element(ElementName::ClientPortRef)?
            .set_reference_target(client_port.element())?;

        Ok(Self(conn))
    }

    /// get the socket connection bundle containing this socket connection
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # use autosar_data_abstraction::communication::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// # let package = model.get_or_create_package("/pkg1")?;
    /// # let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// # let cluster = system.create_ethernet_cluster("Cluster", &package)?;
    /// # let channel = cluster.create_physical_channel("Channel", None)?;
    /// # let server_endpoint = channel.create_network_endpoint("ServerAddress", NetworkEndpointAddress::IPv4 {
    /// #    address: Some("192.168.0.1".to_string()),
    /// #    address_source: Some(IPv4AddressSource::Fixed),
    /// #    default_gateway: None,
    /// #    network_mask: None
    /// # }, None)?;
    /// # let server_socket = channel.create_socket_address("ServerSocket", &server_endpoint, &TpConfig::TcpTp { port_number: Some(1234), port_dynamically_assigned: None }, SocketAddressType::Unicast(None))?;
    /// # let client_endpoint = channel.create_network_endpoint("ClientAddress", NetworkEndpointAddress::IPv4 {
    /// #    address: Some("192.168.0.2".to_string()),
    /// #    address_source: Some(IPv4AddressSource::Fixed),
    /// #    default_gateway: None,
    /// #    network_mask: None
    /// # }, None)?;
    /// # let client_socket = channel.create_socket_address("ClientSocket", &client_endpoint, &TpConfig::TcpTp { port_number: Some(1235), port_dynamically_assigned: None }, SocketAddressType::Unicast(None))?;
    /// let bundle = channel.create_socket_connection_bundle("Bundle", &server_socket)?;
    /// let connection = bundle.create_bundled_connection(&client_socket)?;
    /// assert_eq!(bundle, connection.socket_connection_bundle()?);
    /// # Ok(())}
    /// ```
    pub fn socket_connection_bundle(&self) -> Result<SocketConnectionBundle, AutosarAbstractionError> {
        let bundle = self.element().named_parent()?.unwrap();
        SocketConnectionBundle::try_from(bundle)
    }

    /// get the client port of this socket connection
    #[must_use]
    pub fn client_port(&self) -> Option<SocketAddress> {
        self.element()
            .get_sub_element(ElementName::ClientPortRef)
            .and_then(|cpr| cpr.get_reference_target().ok())
            .and_then(|cp| SocketAddress::try_from(cp).ok())
    }

    /// add a PDU to the socket connection, returning a `PduTriggering`
    pub fn trigger_pdu<T: AbstractPdu>(
        &self,
        pdu: &T,
        header_id: u32,
        timeout: Option<f64>,
        collection_trigger: Option<PduCollectionTrigger>,
    ) -> Result<PduTriggering, AutosarAbstractionError> {
        let pdu: Pdu = pdu.clone().into();
        self.trigger_pdu_internal(&pdu, header_id, timeout, collection_trigger)
    }

    /// add a PDU to the socket connection, returning a `PduTriggering`
    /// Split off an internal function to keep the binary size down, since the rust compiler duplicates the generic functions for each type
    fn trigger_pdu_internal(
        &self,
        pdu: &Pdu,
        header_id: u32,
        timeout: Option<f64>,
        collection_trigger: Option<PduCollectionTrigger>,
    ) -> Result<PduTriggering, AutosarAbstractionError> {
        let scii = self
            .0
            .get_or_create_sub_element(ElementName::Pdus)?
            .create_sub_element(ElementName::SocketConnectionIpduIdentifier)?;
        scii.create_sub_element(ElementName::HeaderId)?
            .set_character_data(header_id.to_string())?;
        if let Some(timeout) = timeout {
            scii.create_sub_element(ElementName::PduCollectionPduTimeout)?
                .set_character_data(timeout)?;
        }
        if let Some(collection_trigger) = collection_trigger {
            scii.create_sub_element(ElementName::PduCollectionTrigger)?
                .set_character_data::<EnumItem>(collection_trigger.into())?;
        }

        let pt = PduTriggering::new(
            pdu,
            &PhysicalChannel::Ethernet(self.socket_connection_bundle()?.physical_channel()?),
        )?;
        scii.create_sub_element(ElementName::PduTriggeringRef)?
            .set_reference_target(pt.element())?;

        Ok(pt)
    }

    /// set the header id for a PDU in this socket connection
    pub fn set_header_id(&self, pdu_triggering: &PduTriggering, header_id: u64) -> Result<(), AutosarAbstractionError> {
        for scii in self
            .element()
            .get_or_create_sub_element(ElementName::Pdus)?
            .sub_elements()
        {
            if let Some(pt_ref) = scii
                .get_sub_element(ElementName::PduTriggeringRef)
                .and_then(|ptref| ptref.get_reference_target().ok())
                .and_then(|pt| PduTriggering::try_from(pt).ok())
            {
                if pt_ref == *pdu_triggering {
                    scii.get_or_create_sub_element(ElementName::HeaderId)?
                        .set_character_data(header_id)?;
                }
            }
        }
        Ok(())
    }

    /// get the header id for a PDU in this socket connection
    #[must_use]
    pub fn header_id(&self, pdu_triggering: &PduTriggering) -> Option<u64> {
        for scii in self.element().get_sub_element(ElementName::Pdus)?.sub_elements() {
            if let Some(pt_ref) = scii
                .get_sub_element(ElementName::PduTriggeringRef)
                .and_then(|ptref| ptref.get_reference_target().ok())
                .and_then(|pt| PduTriggering::try_from(pt).ok())
            {
                if pt_ref == *pdu_triggering {
                    return scii
                        .get_sub_element(ElementName::HeaderId)?
                        .character_data()?
                        .parse_integer();
                }
            }
        }
        None
    }

    /// set the timeout for a PDU in this socket connection
    pub fn set_timeout(&self, pdu_triggering: &PduTriggering, timeout: f64) -> Result<(), AutosarAbstractionError> {
        for scii in self
            .element()
            .get_or_create_sub_element(ElementName::Pdus)?
            .sub_elements()
        {
            if let Some(pt_ref) = scii
                .get_sub_element(ElementName::PduTriggeringRef)
                .and_then(|ptref| ptref.get_reference_target().ok())
                .and_then(|pt| PduTriggering::try_from(pt).ok())
            {
                if pt_ref == *pdu_triggering {
                    scii.get_or_create_sub_element(ElementName::PduCollectionPduTimeout)?
                        .set_character_data(timeout)?;
                }
            }
        }
        Ok(())
    }

    /// get the timeout for a PDU in this socket connection
    #[must_use]
    pub fn timeout(&self, pdu_triggering: &PduTriggering) -> Option<f64> {
        for scii in self.element().get_sub_element(ElementName::Pdus)?.sub_elements() {
            if let Some(pt_ref) = scii
                .get_sub_element(ElementName::PduTriggeringRef)
                .and_then(|ptref| ptref.get_reference_target().ok())
                .and_then(|pt| PduTriggering::try_from(pt).ok())
            {
                if pt_ref == *pdu_triggering {
                    return scii
                        .get_sub_element(ElementName::PduCollectionPduTimeout)?
                        .character_data()?
                        .float_value();
                }
            }
        }
        None
    }

    /// set the collection trigger for a PDU in this socket connection
    pub fn set_collection_trigger(
        &self,
        pdu_triggering: &PduTriggering,
        trigger: PduCollectionTrigger,
    ) -> Result<(), AutosarAbstractionError> {
        for scii in self
            .element()
            .get_or_create_sub_element(ElementName::Pdus)?
            .sub_elements()
        {
            if let Some(pt_ref) = scii
                .get_sub_element(ElementName::PduTriggeringRef)
                .and_then(|ptref| ptref.get_reference_target().ok())
                .and_then(|pt| PduTriggering::try_from(pt).ok())
            {
                if pt_ref == *pdu_triggering {
                    scii.get_or_create_sub_element(ElementName::PduCollectionTrigger)?
                        .set_character_data::<EnumItem>(trigger.into())?;
                }
            }
        }
        Ok(())
    }

    /// get the collection trigger for a PDU in this socket connection
    #[must_use]
    pub fn collection_trigger(&self, pdu_triggering: &PduTriggering) -> Option<PduCollectionTrigger> {
        for scii in self.element().get_sub_element(ElementName::Pdus)?.sub_elements() {
            if let Some(pt_ref) = scii
                .get_sub_element(ElementName::PduTriggeringRef)
                .and_then(|ptref| ptref.get_reference_target().ok())
                .and_then(|pt| PduTriggering::try_from(pt).ok())
            {
                if pt_ref == *pdu_triggering {
                    return scii
                        .get_sub_element(ElementName::PduCollectionTrigger)?
                        .character_data()?
                        .enum_value()?
                        .try_into()
                        .ok();
                }
            }
        }
        None
    }

    /// add a `SoAdRoutingGroup` for a PDU in this socket connection
    pub fn add_routing_group(
        &self,
        pdu_triggering: &PduTriggering,
        routing_group: &SoAdRoutingGroup,
    ) -> Result<(), AutosarAbstractionError> {
        let Some(scii) = self
            .element()
            .get_or_create_sub_element(ElementName::Pdus)?
            .sub_elements()
            .find(|scii| {
                scii.get_sub_element(ElementName::PduTriggeringRef)
                    .and_then(|ptref| ptref.get_reference_target().ok())
                    .and_then(|pt| PduTriggering::try_from(pt).ok())
                    .is_some_and(|pt| pt == *pdu_triggering)
            })
        else {
            return Err(AutosarAbstractionError::InvalidParameter(
                "Could not add SoAdRoutingGroup - PduTriggering not found".to_string(),
            ));
        };

        scii.get_or_create_sub_element(ElementName::RoutingGroupRefs)?
            .create_sub_element(ElementName::RoutingGroupRef)?
            .set_reference_target(routing_group.element())?;

        Ok(())
    }

    /// create an iterator over all PDU triggerings in this socket connection
    pub fn pdu_triggerings(&self) -> impl Iterator<Item = PduTriggering> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::Pdus)
            .into_iter()
            .flat_map(|pdus| pdus.sub_elements())
            .filter_map(|scii: Element| {
                scii.get_sub_element(ElementName::PduTriggeringRef)
                    .and_then(|pt| pt.get_reference_target().ok())
                    .and_then(|pt| PduTriggering::try_from(pt).ok())
            })
    }

    /// set or remove the `client_ip_addr_from_connection_request` attribute for this socket connection
    ///
    /// if the value is Some(true), the attribute is set to "true"
    /// if the value is Some(false), the attribute is set to "false"
    /// if the value is None, the attribute is removed
    pub fn set_client_ip_addr_from_connection_request(
        &self,
        value: Option<bool>,
    ) -> Result<(), AutosarAbstractionError> {
        if let Some(value) = value {
            self.element()
                .get_or_create_sub_element(ElementName::ClientIpAddrFromConnectionRequest)?
                .set_character_data(value.to_string())?;
        } else {
            let _ = self
                .element()
                .remove_sub_element_kind(ElementName::ClientIpAddrFromConnectionRequest);
        }
        Ok(())
    }

    /// get the value of the `client_ip_addr_from_connection_request` attribute for this socket connection
    #[must_use]
    pub fn client_ip_addr_from_connection_request(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::ClientIpAddrFromConnectionRequest)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_bool())
    }

    /// set or remove the `client_port_from_connection_request` attribute for this socket connection
    ///
    /// if the value is Some(true), the attribute is set to "true"
    /// if the value is Some(false), the attribute is set to "false"
    /// if the value is None, the attribute is removed
    pub fn set_client_port_from_connection_request(&self, value: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(value) = value {
            self.element()
                .get_or_create_sub_element(ElementName::ClientPortFromConnectionRequest)?
                .set_character_data(value.to_string())?;
        } else {
            let _ = self
                .element()
                .remove_sub_element_kind(ElementName::ClientPortFromConnectionRequest);
        }
        Ok(())
    }

    /// get the value of the `client_port_from_connection_request` attribute for this socket connection
    #[must_use]
    pub fn client_port_from_connection_request(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::ClientPortFromConnectionRequest)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_bool())
    }

    /// set or remove the RuntimeIpAddressConfiguration/RuntimePortConfiguration attributes for this socket connection
    ///
    /// If `state` is true, the attributes are set to "Sd"
    /// If `state` is false, the attributes are removed
    pub fn set_runtime_address_configuration(&self, state: bool) -> Result<(), AutosarAbstractionError> {
        if state {
            self.element()
                .get_or_create_sub_element(ElementName::RuntimeIpAddressConfiguration)?
                .set_character_data(EnumItem::Sd)?;
            self.element()
                .get_or_create_sub_element(ElementName::RuntimePortConfiguration)?
                .set_character_data(EnumItem::Sd)?;
        } else {
            let _ = self
                .element()
                .remove_sub_element_kind(ElementName::RuntimeIpAddressConfiguration);
            let _ = self
                .element()
                .remove_sub_element_kind(ElementName::RuntimePortConfiguration);
        }
        Ok(())
    }

    /// get the value of the RuntimeIpAddressConfiguration attribute for this socket connection
    #[must_use]
    pub fn runtime_ip_address_configuration(&self) -> bool {
        let enum_value = self
            .element()
            .get_sub_element(ElementName::RuntimeIpAddressConfiguration)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.enum_value());
        enum_value == Some(EnumItem::Sd)
    }

    /// get the value of the RuntimePortConfiguration attribute for this socket connection
    #[must_use]
    pub fn runtime_port_configuration(&self) -> bool {
        let enum_value = self
            .element()
            .get_sub_element(ElementName::RuntimePortConfiguration)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.enum_value());
        enum_value == Some(EnumItem::Sd)
    }
}

//##################################################################

/// A `SoAdRoutingGroup` is used to link `SomeIp` settings in Consumed/ProvidedServiceInstances
/// to the `SocketConnectionBundles` used for transmission.
/// `SoAdRoutingGroups` are part of the old way of configuring Ethernet communication in AUTOSAR.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SoAdRoutingGroup(Element);
abstraction_element!(SoAdRoutingGroup, SoAdRoutingGroup);
impl IdentifiableAbstractionElement for SoAdRoutingGroup {}

impl SoAdRoutingGroup {
    /// create a new `SoAdRoutingGroup`
    ///
    /// `SoAdRoutingGroups` are used to link `SomeIp` settings in Consumed/ProvidedServiceInstances
    /// to the `SocketConnectionBundles` used for transmission.
    /// `SoAdRoutingGroups` are part of the old way of configuring Ethernet communication in AUTOSAR.
    pub(crate) fn new(
        name: &str,
        package: &ArPackage,
        control_type: Option<EventGroupControlType>,
    ) -> Result<Self, AutosarAbstractionError> {
        let srg_elem: Element = package
            .element()
            .get_or_create_sub_element(ElementName::Elements)?
            .create_named_sub_element(ElementName::SoAdRoutingGroup, name)?;
        let srg = Self(srg_elem);

        if let Some(control_type) = control_type {
            srg.set_control_type(control_type)?;
        }

        Ok(srg)
    }

    /// set the `EventGroupControlType` of this `SoAdRoutingGroup`
    pub fn set_control_type(&self, control_type: EventGroupControlType) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::EventGroupControlType)?
            .set_character_data::<EnumItem>(control_type.into())?;
        Ok(())
    }

    /// get the `EventGroupControlType` of this `SoAdRoutingGroup`
    #[must_use]
    pub fn control_type(&self) -> Option<EventGroupControlType> {
        self.element()
            .get_sub_element(ElementName::EventGroupControlType)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.enum_value())
            .and_then(|eval| eval.try_into().ok())
    }
}

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        communication::{IPv4AddressSource, NetworkEndpointAddress, SocketAddressType},
        AutosarModelAbstraction, SystemCategory,
    };
    use autosar_data::AutosarVersion;

    #[test]
    fn test_socket_connection_bundle() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let package = model.get_or_create_package("/pkg1").unwrap();
        let system = package.create_system("System", SystemCategory::SystemExtract).unwrap();
        let cluster = system.create_ethernet_cluster("Cluster", &package).unwrap();
        let channel = cluster.create_physical_channel("Channel", None).unwrap();
        let server_endpoint = channel
            .create_network_endpoint(
                "ServerAddress",
                NetworkEndpointAddress::IPv4 {
                    address: Some("192.168.0.1".to_string()),
                    address_source: Some(IPv4AddressSource::Fixed),
                    default_gateway: None,
                    network_mask: None,
                },
                None,
            )
            .unwrap();
        let server_socket = channel
            .create_socket_address(
                "ServerSocket",
                &server_endpoint,
                &TpConfig::TcpTp {
                    port_number: Some(1234),
                    port_dynamically_assigned: None,
                },
                SocketAddressType::Unicast(None),
            )
            .unwrap();
        let bundle = channel
            .create_socket_connection_bundle("Bundle", &server_socket)
            .unwrap();
        assert_eq!(channel.socket_connection_bundles().next(), Some(bundle.clone()));
        assert_eq!(channel.socket_connection_bundles().count(), 1);
        assert_eq!(channel, bundle.physical_channel().unwrap());
        assert_eq!(Some(server_socket), bundle.server_port());

        let client_endpoint = channel
            .create_network_endpoint(
                "ClientAddress",
                NetworkEndpointAddress::IPv4 {
                    address: Some("192.168.0.2".to_string()),
                    address_source: Some(IPv4AddressSource::Fixed),
                    default_gateway: None,
                    network_mask: None,
                },
                None,
            )
            .unwrap();
        let client_socket = channel
            .create_socket_address(
                "ClientSocket",
                &client_endpoint,
                &TpConfig::TcpTp {
                    port_number: Some(1235),
                    port_dynamically_assigned: None,
                },
                SocketAddressType::Unicast(None),
            )
            .unwrap();
        let connection = bundle.create_bundled_connection(&client_socket).unwrap();

        let pdu = system.create_isignal_ipdu("Pdu", &package, 8).unwrap();
        let pt = connection
            .trigger_pdu(&pdu, 0x1234, Some(0.5), Some(PduCollectionTrigger::Always))
            .unwrap();
        assert_eq!(Some(pt.clone()), connection.pdu_triggerings().next());
        assert_eq!(Some(0x1234), connection.header_id(&pt));
        assert_eq!(Some(0.5), connection.timeout(&pt));
        assert_eq!(Some(PduCollectionTrigger::Always), connection.collection_trigger(&pt));

        connection.set_header_id(&pt, 0x5678).unwrap();
        assert_eq!(Some(0x5678), connection.header_id(&pt));
        connection.set_timeout(&pt, 1.5).unwrap();
        assert_eq!(Some(1.5), connection.timeout(&pt));
        connection
            .set_collection_trigger(&pt, PduCollectionTrigger::Never)
            .unwrap();
        assert_eq!(Some(PduCollectionTrigger::Never), connection.collection_trigger(&pt));
        connection
            .set_client_ip_addr_from_connection_request(Some(true))
            .unwrap();
        assert_eq!(connection.client_ip_addr_from_connection_request(), Some(true));
        connection.set_client_ip_addr_from_connection_request(None).unwrap();
        assert_eq!(connection.client_ip_addr_from_connection_request(), None);
        connection.set_client_port_from_connection_request(Some(false)).unwrap();
        assert_eq!(connection.client_port_from_connection_request(), Some(false));
        connection.set_client_port_from_connection_request(None).unwrap();
        assert_eq!(connection.client_port_from_connection_request(), None);
        connection.set_runtime_address_configuration(true).unwrap();
        assert_eq!(connection.runtime_ip_address_configuration(), true);
        assert_eq!(connection.runtime_port_configuration(), true);
        connection.set_runtime_address_configuration(false).unwrap();
        assert_eq!(connection.runtime_ip_address_configuration(), false);
        assert_eq!(connection.runtime_port_configuration(), false);

        let routing_group = system
            .create_so_ad_routing_group("RoutingGroup", &package, None)
            .unwrap();
        connection.add_routing_group(&pt, &routing_group).unwrap();

        assert_eq!(routing_group.control_type(), None);
        routing_group
            .set_control_type(EventGroupControlType::TriggerUnicast)
            .unwrap();
        assert_eq!(
            routing_group.control_type(),
            Some(EventGroupControlType::TriggerUnicast)
        );
    }
}
