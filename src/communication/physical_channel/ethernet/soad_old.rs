use crate::communication::{
    AbstractPdu, EthernetPhysicalChannel, EventGroupControlType, Pdu, PduCollectionTrigger, PduTriggering,
    PhysicalChannel, SocketAddress, TpConfig,
};
use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element,
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
        let scb_elem = connections_elem.create_named_sub_element(ElementName::SocketConnectionBundle, name)?;
        let scb = Self(scb_elem);

        scb.set_server_port(server_port)?;

        Ok(scb)
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

    ///set the server port of this socket connection bundle
    pub fn set_server_port(&self, server_port: &SocketAddress) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::ServerPortRef)?
            .set_reference_target(server_port.element())?;
        Ok(())
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
                ));
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
        let conn = Self(conn);
        conn.set_client_port(client_port)?;

        Ok(conn)
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

    /// set the client port of this socket connection
    pub fn set_client_port(&self, client_port: &SocketAddress) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::ClientPortRef)?
            .set_reference_target(client_port.element())?;
        Ok(())
    }

    /// get the client port of this socket connection
    #[must_use]
    pub fn client_port(&self) -> Option<SocketAddress> {
        self.element()
            .get_sub_element(ElementName::ClientPortRef)
            .and_then(|cpr| cpr.get_reference_target().ok())
            .and_then(|cp| SocketAddress::try_from(cp).ok())
    }

    /// Create a new `SocketConnectionIpduIdentifier` in this socket connection
    ///
    /// The `SocketConnectionIpduIdentifier` is used to trigger a PDU, and contains associated settings
    /// The function returns a tuple of the new `SocketConnectionIpduIdentifier` and the associated `PduTriggering`
    /// since most callers only need the `PduTriggering`.
    pub fn create_socket_connection_ipdu_identifier<T: AbstractPdu>(
        &self,
        pdu: &T,
        header_id: u32,
        timeout: Option<f64>,
        collection_trigger: Option<PduCollectionTrigger>,
    ) -> Result<(SocketConnectionIpduIdentifier, PduTriggering), AutosarAbstractionError> {
        SocketConnectionIpduIdentifier::new(
            self.element(),
            &pdu.clone().into(),
            header_id,
            timeout,
            collection_trigger,
        )
    }

    /// create an iterator over all `SocketConnectionIpduIdentifiers` in this socket connection
    pub fn socket_connection_ipdu_identifiers(
        &self,
    ) -> impl Iterator<Item = SocketConnectionIpduIdentifier> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::Pdus)
            .into_iter()
            .flat_map(|pdus| pdus.sub_elements())
            .filter_map(|elem| SocketConnectionIpduIdentifier::try_from(elem).ok())
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

    /// set the `RuntimeIpAddressConfiguration` attribute for this socket connection
    ///
    /// If `state` is true, the attribute is set to "Sd"
    /// If `state` is false, the attribute is removed
    pub fn set_runtime_ip_address_configuration(&self, state: bool) -> Result<(), AutosarAbstractionError> {
        if state {
            self.element()
                .get_or_create_sub_element(ElementName::RuntimeIpAddressConfiguration)?
                .set_character_data(EnumItem::Sd)?;
        } else {
            let _ = self
                .element()
                .remove_sub_element_kind(ElementName::RuntimeIpAddressConfiguration);
        }
        Ok(())
    }

    /// check if the value of the `RuntimeIpAddressConfiguration` attribute is "SD"
    #[must_use]
    pub fn runtime_ip_address_configuration(&self) -> bool {
        let enum_value = self
            .element()
            .get_sub_element(ElementName::RuntimeIpAddressConfiguration)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.enum_value());
        enum_value == Some(EnumItem::Sd)
    }

    /// set the `RuntimePortConfiguration` attributes for this socket connection
    ///
    /// If `state` is true, the attribute is set to "Sd"
    /// If `state` is false, the attributes is removed
    pub fn set_runtime_port_configuration(&self, state: bool) -> Result<(), AutosarAbstractionError> {
        if state {
            self.element()
                .get_or_create_sub_element(ElementName::RuntimePortConfiguration)?
                .set_character_data(EnumItem::Sd)?;
        } else {
            let _ = self
                .element()
                .remove_sub_element_kind(ElementName::RuntimePortConfiguration);
        }
        Ok(())
    }

    /// check if the value of the `RuntimePortConfiguration` attribute is "SD"
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

/// A `SocketConnectionIpduIdentifier` is used to trigger a PDU in a `SocketConnection`.
///
/// In addition to the Pdu Triggering, it also contains associated settings like the
/// header id, timeout and collection trigger.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SocketConnectionIpduIdentifier(Element);
abstraction_element!(SocketConnectionIpduIdentifier, SocketConnectionIpduIdentifier);

impl SocketConnectionIpduIdentifier {
    pub(crate) fn new<T: AbstractPdu>(
        parent: &Element,
        pdu: &T,
        header_id: u32,
        timeout: Option<f64>,
        collection_trigger: Option<PduCollectionTrigger>,
    ) -> Result<(Self, PduTriggering), AutosarAbstractionError> {
        let scii_elem = parent
            .get_or_create_sub_element(ElementName::Pdus)?
            .create_sub_element(ElementName::SocketConnectionIpduIdentifier)?;
        let scii = Self(scii_elem);
        let pt = scii.trigger_pdu(&pdu.clone().into())?;
        scii.set_header_id(header_id)?;
        scii.set_timeout(timeout)?;
        scii.set_collection_trigger(collection_trigger)?;

        Ok((scii, pt))
    }

    /// get the `SocketConnection` containing this `SocketConnectionIpduIdentifier`
    pub fn socket_connection(&self) -> Result<SocketConnection, AutosarAbstractionError> {
        // SOCKET-CONNECTION > PDUS > SOCKET-CONNECTION-IPDU-IDENTIFIER
        let socket_connection_elem = self.element().parent()?.unwrap().parent()?.unwrap();
        SocketConnection::try_from(socket_connection_elem)
    }

    /// trigger a PDU in this `SocketConnectionIpduIdentifier`, creating a `PduTriggering`
    pub fn trigger_pdu(&self, pdu: &Pdu) -> Result<PduTriggering, AutosarAbstractionError> {
        if let Some(old_pt) = self.pdu_triggering() {
            // there is already a PduTriggering in this SocketConnectionIpduIdentifier
            // remove it first -- ideally this should be old_pt.remove() but that doesn't exist yet
            if let Ok(Some(parent)) = old_pt.element().parent() {
                parent.remove_sub_element(old_pt.element().clone())?;
            }
        }
        let channel = self
            .socket_connection()?
            .socket_connection_bundle()?
            .physical_channel()?;
        let pt = PduTriggering::new(pdu, &PhysicalChannel::Ethernet(channel))?;

        self.element()
            .get_or_create_sub_element(ElementName::PduTriggeringRef)?
            .set_reference_target(pt.element())?;
        Ok(pt)
    }

    /// get the `PduTriggering` associated with this `SocketConnectionIpduIdentifier`
    #[must_use]
    pub fn pdu_triggering(&self) -> Option<PduTriggering> {
        self.element()
            .get_sub_element(ElementName::PduTriggeringRef)
            .and_then(|elem| elem.get_reference_target().ok())
            .and_then(|pt| PduTriggering::try_from(pt).ok())
    }

    /// set the header id for this `SocketConnectionIpduIdentifier`
    pub fn set_header_id(&self, header_id: u32) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::HeaderId)?
            .set_character_data(header_id.to_string())?;
        Ok(())
    }

    /// get the header id for this `SocketConnectionIpduIdentifier`
    #[must_use]
    pub fn header_id(&self) -> Option<u64> {
        self.element()
            .get_sub_element(ElementName::HeaderId)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set the timeout for this `SocketConnectionIpduIdentifier`
    pub fn set_timeout(&self, timeout: Option<f64>) -> Result<(), AutosarAbstractionError> {
        if let Some(timeout) = timeout {
            self.element()
                .get_or_create_sub_element(ElementName::PduCollectionPduTimeout)?
                .set_character_data(timeout)?;
        } else {
            let _ = self
                .element()
                .remove_sub_element_kind(ElementName::PduCollectionPduTimeout);
        }
        Ok(())
    }

    /// get the timeout for this `SocketConnectionIpduIdentifier`
    #[must_use]
    pub fn timeout(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::PduCollectionPduTimeout)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.float_value())
    }

    /// set the collection trigger for this `SocketConnectionIpduIdentifier`
    pub fn set_collection_trigger(&self, trigger: Option<PduCollectionTrigger>) -> Result<(), AutosarAbstractionError> {
        if let Some(trigger) = trigger {
            self.element()
                .get_or_create_sub_element(ElementName::PduCollectionTrigger)?
                .set_character_data::<EnumItem>(trigger.into())?;
        } else {
            let _ = self
                .element()
                .remove_sub_element_kind(ElementName::PduCollectionTrigger);
        }
        Ok(())
    }

    /// get the collection trigger for this `SocketConnectionIpduIdentifier`
    #[must_use]
    pub fn collection_trigger(&self) -> Option<PduCollectionTrigger> {
        self.element()
            .get_sub_element(ElementName::PduCollectionTrigger)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.enum_value())
            .and_then(|eval| eval.try_into().ok())
    }

    /// add a reference to a `SoAdRoutingGroup` to this `SocketConnectionIpduIdentifier`
    pub fn add_routing_group(&self, routing_group: &SoAdRoutingGroup) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::RoutingGroupRefs)?
            .create_sub_element(ElementName::RoutingGroupRef)?
            .set_reference_target(routing_group.element())?;
        Ok(())
    }

    /// create an iterator over all `SoAdRoutingGroups` referenced by this `SocketConnectionIpduIdentifier`
    pub fn routing_groups(&self) -> impl Iterator<Item = SoAdRoutingGroup> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::RoutingGroupRefs)
            .into_iter()
            .flat_map(|rgr| rgr.sub_elements())
            .filter_map(|ref_elem| {
                ref_elem
                    .get_reference_target()
                    .ok()
                    .and_then(|elem| SoAdRoutingGroup::try_from(elem).ok())
            })
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
        AutosarModelAbstraction, SystemCategory,
        communication::{IPv4AddressSource, NetworkEndpointAddress, SocketAddressType},
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
        let (scii, pt) = connection
            .create_socket_connection_ipdu_identifier(&pdu, 0x1234, Some(0.5), Some(PduCollectionTrigger::Always))
            .unwrap();
        assert_eq!(connection.socket_connection_ipdu_identifiers().count(), 1);
        assert_eq!(pt, scii.pdu_triggering().unwrap());
        assert_eq!(Some(pt.clone()), connection.pdu_triggerings().next());
        assert_eq!(Some(0x1234), scii.header_id());
        assert_eq!(Some(0.5), scii.timeout());
        assert_eq!(Some(PduCollectionTrigger::Always), scii.collection_trigger());

        scii.set_header_id(0x5678).unwrap();
        assert_eq!(Some(0x5678), scii.header_id());
        scii.set_timeout(Some(1.5)).unwrap();
        assert_eq!(Some(1.5), scii.timeout());
        scii.set_collection_trigger(Some(PduCollectionTrigger::Never)).unwrap();
        assert_eq!(Some(PduCollectionTrigger::Never), scii.collection_trigger());
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
        connection.set_runtime_ip_address_configuration(true).unwrap();
        assert!(connection.runtime_ip_address_configuration());
        connection.set_runtime_port_configuration(true).unwrap();
        assert!(connection.runtime_port_configuration());
        connection.set_runtime_ip_address_configuration(false).unwrap();
        assert!(!connection.runtime_ip_address_configuration());
        connection.set_runtime_port_configuration(false).unwrap();
        assert!(!connection.runtime_port_configuration());

        let routing_group = system
            .create_so_ad_routing_group("RoutingGroup", &package, None)
            .unwrap();
        scii.add_routing_group(&routing_group).unwrap();
        assert_eq!(scii.routing_groups().next(), Some(routing_group.clone()));
        assert_eq!(scii.routing_groups().count(), 1);

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
