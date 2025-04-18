use crate::communication::{
    AbstractPhysicalChannel, ConsumedServiceInstanceV1, EthernetPhysicalChannel, NetworkEndpoint,
    ProvidedServiceInstanceV1, StaticSocketConnection, TcpRole,
};
use crate::{
    AbstractionElement, AutosarAbstractionError, EcuInstance, IdentifiableAbstractionElement, abstraction_element,
};
use autosar_data::{Element, ElementName};

//##################################################################

/// A socket address establishes the link between one or more ECUs and a `NetworkEndpoint`.
/// It contains all settings that are relevant for this combination.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SocketAddress(Element);
abstraction_element!(SocketAddress, SocketAddress);
impl IdentifiableAbstractionElement for SocketAddress {}

impl SocketAddress {
    pub(crate) fn new(
        name: &str,
        channel: &EthernetPhysicalChannel,
        network_endpoint: &NetworkEndpoint,
        tp_config: &TpConfig,
        sa_type: SocketAddressType,
    ) -> Result<Self, AutosarAbstractionError> {
        let channel_elem = channel.element();
        let (unicast, ecu_instances) = match sa_type {
            SocketAddressType::Unicast(Some(ecu_instance)) => (true, vec![ecu_instance]),
            SocketAddressType::Unicast(None) => (true, vec![]),
            SocketAddressType::Multicast(ecu_instances) => (false, ecu_instances),
        };

        // TCP connections don't work using multicast IP addresses
        if !unicast && matches!(tp_config, TpConfig::TcpTp { .. }) {
            return Err(AutosarAbstractionError::InvalidParameter(
                "TCP is incomptible with multicasting".to_string(),
            ));
        }
        // extension: check if the address is valid for multicasting?
        // IPv4: 224.0.0.0 - 239.255.255.255
        // IPv6: FFxx:/12

        // get the connector for each ECU in advance, so that nothing needs to be cleaned up if there is a problem here
        let connectors = ecu_instances
            .iter()
            .filter_map(|ecu_instance| channel.ecu_connector(ecu_instance).map(|conn| conn.element().clone()))
            .collect::<Vec<_>>();
        if connectors.len() != ecu_instances.len() {
            return Err(AutosarAbstractionError::InvalidParameter(
                "All EcuInstances must be connected to the EthernetPhysicalChannel".to_string(),
            ));
        }

        let elem = channel_elem
            .get_or_create_sub_element(ElementName::SoAdConfig)?
            .get_or_create_sub_element(ElementName::SocketAddresss)?
            .create_named_sub_element(ElementName::SocketAddress, name)?;

        if unicast {
            if !connectors.is_empty() {
                elem.create_sub_element(ElementName::ConnectorRef)
                    .unwrap()
                    .set_reference_target(&connectors[0])
                    .unwrap();
            }
        } else {
            let mc_connectors = elem.create_sub_element(ElementName::MulticastConnectorRefs)?;
            for conn in &connectors {
                mc_connectors
                    .create_sub_element(ElementName::MulticastConnectorRef)?
                    .set_reference_target(conn)?;
            }
        }

        let ae_name = format!("{name}_AE");
        let ae = elem.create_named_sub_element(ElementName::ApplicationEndpoint, &ae_name)?;
        ae.create_sub_element(ElementName::NetworkEndpointRef)?
            .set_reference_target(network_endpoint.element())?;
        let tp_configuration = ae.create_sub_element(ElementName::TpConfiguration)?;
        match tp_config {
            TpConfig::TcpTp {
                port_number,
                port_dynamically_assigned,
            } => {
                let tcptp = tp_configuration.create_sub_element(ElementName::TcpTp)?;
                let tcptp_port = tcptp.create_sub_element(ElementName::TcpTpPort)?;
                // PortNumber and DynamicallyAssigned are mutually exclusive.
                // The attribute DynamicallyAssigned is deprecated starting in Autosar 4.5.0
                if let Some(portnum) = port_number {
                    tcptp_port
                        .create_sub_element(ElementName::PortNumber)?
                        .set_character_data(portnum.to_string())?;
                } else if let Some(dyn_assign) = port_dynamically_assigned {
                    tcptp_port
                        .create_sub_element(ElementName::DynamicallyAssigned)?
                        .set_character_data(*dyn_assign)?;
                }
            }
            TpConfig::UdpTp {
                port_number,
                port_dynamically_assigned,
            } => {
                let udptp_port = tp_configuration
                    .create_sub_element(ElementName::UdpTp)?
                    .create_sub_element(ElementName::UdpTpPort)?;
                // PortNumber and DynamicallyAssigned are mutually exclusive.
                // The attribute DynamicallyAssigned is deprecated starting in Autosar 4.5.0
                if let Some(portnum) = port_number {
                    udptp_port
                        .create_sub_element(ElementName::PortNumber)?
                        .set_character_data(portnum.to_string())?;
                } else if let Some(dyn_assign) = port_dynamically_assigned {
                    let boolstr = if *dyn_assign { "true" } else { "false" };
                    udptp_port
                        .create_sub_element(ElementName::DynamicallyAssigned)?
                        .set_character_data(boolstr)?;
                }
            }
        }

        Ok(Self(elem))
    }

    /// get the network endpoint of this `SocketAddress`
    #[must_use]
    pub fn network_endpoint(&self) -> Option<NetworkEndpoint> {
        let ne = self
            .element()
            .get_sub_element(ElementName::ApplicationEndpoint)?
            .get_sub_element(ElementName::NetworkEndpointRef)?
            .get_reference_target()
            .ok()?;
        ne.try_into().ok()
    }

    /// get the socket address type: unicast / multicast, as well as the connected ecus
    #[must_use]
    pub fn socket_address_type(&self) -> Option<SocketAddressType> {
        if let Some(connector_ref) = self.0.get_sub_element(ElementName::ConnectorRef) {
            let ecu = EcuInstance::try_from(connector_ref.get_reference_target().ok()?.named_parent().ok()??).ok()?;
            Some(SocketAddressType::Unicast(Some(ecu)))
        } else if let Some(mcr) = self.0.get_sub_element(ElementName::MulticastConnectorRefs) {
            let ecus = mcr
                .sub_elements()
                .filter_map(|cr| {
                    cr.get_reference_target()
                        .ok()
                        .and_then(|conn| conn.named_parent().ok().flatten())
                })
                .filter_map(|ecu_elem| EcuInstance::try_from(ecu_elem).ok())
                .collect::<Vec<_>>();
            Some(SocketAddressType::Multicast(ecus))
        } else {
            None
        }
    }

    /// add an `EcuInstance` to this multicast `SocketAddress`
    pub fn add_multicast_ecu(&self, ecu: &EcuInstance) -> Result<(), AutosarAbstractionError> {
        let socket_type = self.socket_address_type();
        match socket_type {
            Some(SocketAddressType::Multicast(multicast_ecus)) => {
                // extend the list of multicast EcuInstances if needed
                if !multicast_ecus.contains(ecu) {
                    let Some(connector) = self.physical_channel()?.ecu_connector(ecu) else {
                        return Err(AutosarAbstractionError::InvalidParameter(
                            "EcuInstance is not connected to the EthernetPhysicalChannel".to_string(),
                        ));
                    };
                    let mcr = self.0.get_or_create_sub_element(ElementName::MulticastConnectorRefs)?;
                    let mc_ref = mcr.create_sub_element(ElementName::MulticastConnectorRef)?;
                    mc_ref.set_reference_target(connector.element())?;
                }
            }
            None => {
                // add the first EcuInstance to this multicast SocketAddress
                let Some(connector) = self.physical_channel()?.ecu_connector(ecu) else {
                    return Err(AutosarAbstractionError::InvalidParameter(
                        "EcuInstance is not connected to the EthernetPhysicalChannel".to_string(),
                    ));
                };
                let mcr = self.0.get_or_create_sub_element(ElementName::MulticastConnectorRefs)?;
                let mc_ref = mcr.create_sub_element(ElementName::MulticastConnectorRef)?;
                mc_ref.set_reference_target(connector.element())?;
            }
            Some(SocketAddressType::Unicast(_)) => {
                return Err(AutosarAbstractionError::InvalidParameter(
                    "This SocketAddress is not a multicast socket".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// set the `EcuInstance` for this unicast `SocketAddress`
    pub fn set_unicast_ecu(&self, ecu: &EcuInstance) -> Result<(), AutosarAbstractionError> {
        let socket_type = self.socket_address_type();
        match socket_type {
            None | Some(SocketAddressType::Unicast(_)) => {
                let channel = self.physical_channel()?;
                let Some(connector) = channel.ecu_connector(ecu) else {
                    return Err(AutosarAbstractionError::InvalidParameter(
                        "EcuInstance is not connected to the EthernetPhysicalChannel".to_string(),
                    ));
                };
                self.0
                    .get_or_create_sub_element(ElementName::ConnectorRef)?
                    .set_reference_target(connector.element())?;
            }
            Some(SocketAddressType::Multicast(_)) => {
                return Err(AutosarAbstractionError::InvalidParameter(
                    "This SocketAddress is not a unicast socket".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// get the transport protocol settings for this `SocketAddress`
    #[must_use]
    pub fn tp_config(&self) -> Option<TpConfig> {
        let tp = self
            .0
            .get_sub_element(ElementName::ApplicationEndpoint)?
            .get_sub_element(ElementName::TpConfiguration)?;

        if let Some(tcp_tp) = tp.get_sub_element(ElementName::TcpTp) {
            let port = tcp_tp.get_sub_element(ElementName::TcpTpPort)?;
            let (port_number, port_dynamically_assigned) = Self::port_config(&port);
            Some(TpConfig::TcpTp {
                port_number,
                port_dynamically_assigned,
            })
        } else if let Some(udp_tp) = tp.get_sub_element(ElementName::UdpTp) {
            let port = udp_tp.get_sub_element(ElementName::UdpTpPort)?;
            let (port_number, port_dynamically_assigned) = Self::port_config(&port);
            Some(TpConfig::UdpTp {
                port_number,
                port_dynamically_assigned,
            })
        } else {
            None
        }
    }

    // get the port number and dynamic assignment setting from a port element
    fn port_config(port_element: &Element) -> (Option<u16>, Option<bool>) {
        let port_number = port_element
            .get_sub_element(ElementName::PortNumber)
            .and_then(|pn| pn.character_data())
            .and_then(|cdata| cdata.parse_integer());
        let port_dynamically_assigned = port_element
            .get_sub_element(ElementName::DynamicallyAssigned)
            .and_then(|da| da.character_data())
            .and_then(|cdata| cdata.string_value())
            .map(|val| val == "true" || val == "1");
        (port_number, port_dynamically_assigned)
    }

    /// create a new `StaticSocketConnection` from this `SocketAddress` to a remote `SocketAddress`
    pub fn create_static_socket_connection(
        &self,
        name: &str,
        remote_address: &SocketAddress,
        tcp_role: Option<TcpRole>,
        tcp_connect_timeout: Option<f64>,
    ) -> Result<StaticSocketConnection, AutosarAbstractionError> {
        let own_tp_config = self.tp_config();
        let remote_tp_config = remote_address.tp_config();
        match (own_tp_config, remote_tp_config) {
            (Some(TpConfig::TcpTp { .. }), Some(TpConfig::TcpTp { .. })) => {
                StaticSocketConnection::new(name, self.element(), remote_address, tcp_role, tcp_connect_timeout)
            }
            (Some(TpConfig::UdpTp { .. }), Some(TpConfig::UdpTp { .. })) | (None, None) => {
                StaticSocketConnection::new(name, self.element(), remote_address, None, None)
            }
            _ => Err(AutosarAbstractionError::InvalidParameter(
                "Both SocketAddresses must use the same transport protocol".to_string(),
            )),
        }
    }

    /// get the `PhysicalChannel` containing this `SocketAddress`
    pub fn physical_channel(&self) -> Result<EthernetPhysicalChannel, AutosarAbstractionError> {
        let named_parent = self.0.named_parent()?.unwrap();
        named_parent.try_into()
    }

    /// iterate over all `StaticSocketConnection`s in this `SocketAddress`
    pub fn static_socket_connections(&self) -> impl Iterator<Item = StaticSocketConnection> + Send + 'static {
        self.0
            .get_sub_element(ElementName::StaticSocketConnections)
            .into_iter()
            .flat_map(|ssc| ssc.sub_elements())
            .filter_map(|ssc| StaticSocketConnection::try_from(ssc).ok())
    }

    /// create a `ProvidedServiceInstanceV1` in this `SocketAddress`
    ///
    /// Creating a `ProvidedServiceInstanceV1` in a `SocketAddress` is part of the old way of defining services (<= Autosar 4.5.0).
    /// It is obsolete in newer versions of the standard.
    ///
    /// When using the new way of defining services, a `ProvidedServiceInstance` should be created in a `ServiceInstanceCollectionSet` instead.
    pub fn create_provided_service_instance(
        &self,
        name: &str,
        service_identifier: u16,
        instance_identifier: u16,
    ) -> Result<ProvidedServiceInstanceV1, AutosarAbstractionError> {
        let socket_name = self.name().unwrap_or_default();
        let ae_name = format!("{socket_name}_AE");
        let ae = self
            .element()
            .get_or_create_named_sub_element(ElementName::ApplicationEndpoint, &ae_name)?;
        let psis = ae.get_or_create_sub_element(ElementName::ProvidedServiceInstances)?;

        ProvidedServiceInstanceV1::new(name, &psis, service_identifier, instance_identifier)
    }

    /// get the `ProvidedServiceInstanceV1`s in this `SocketAddress`
    pub fn provided_service_instances(&self) -> impl Iterator<Item = ProvidedServiceInstanceV1> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::ApplicationEndpoint)
            .and_then(|ae| ae.get_sub_element(ElementName::ProvidedServiceInstances))
            .into_iter()
            .flat_map(|psis| psis.sub_elements())
            .filter_map(|psi| ProvidedServiceInstanceV1::try_from(psi).ok())
    }

    /// create a `ConsumedServiceInstanceV1` in this `SocketAddress`
    ///
    /// Creating a `ConsumedServiceInstanceV1` in a `SocketAddress` is part of the old way of defining services (<= Autosar 4.5.0).
    /// It is obsolete in newer versions of the standard.
    ///
    /// When using the new way of defining services, a `ConsumedServiceInstance` should be created in a `ServiceInstanceCollectionSet` instead.
    pub fn create_consumed_service_instance(
        &self,
        name: &str,
        provided_service_instance: &ProvidedServiceInstanceV1,
    ) -> Result<ConsumedServiceInstanceV1, AutosarAbstractionError> {
        let socket_name = self.name().unwrap_or_default();
        let ae_name = format!("{socket_name}_AE");
        let ae = self
            .element()
            .get_or_create_named_sub_element(ElementName::ApplicationEndpoint, &ae_name)?;
        let csis = ae.get_or_create_sub_element(ElementName::ConsumedServiceInstances)?;
        ConsumedServiceInstanceV1::new(name, &csis, provided_service_instance)
    }

    /// get the `ConsumedServiceInstance`s in this `SocketAddress`
    pub fn consumed_service_instances(&self) -> impl Iterator<Item = ConsumedServiceInstanceV1> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::ApplicationEndpoint)
            .and_then(|ae| ae.get_sub_element(ElementName::ConsumedServiceInstances))
            .into_iter()
            .flat_map(|csis| csis.sub_elements())
            .filter_map(|csi| ConsumedServiceInstanceV1::try_from(csi).ok())
    }
}

//##################################################################

/// transport protocol settings of a [`SocketAddress`]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TpConfig {
    /// The socket uses TCP
    TcpTp {
        /// The port number used by the socket
        port_number: Option<u16>,
        /// If the port number is dynamically assigned. Obsolete; set the port number to None instead
        port_dynamically_assigned: Option<bool>,
        // additional TCP options: currently not supported
    },
    /// The socket uses UDP
    UdpTp {
        /// The port number used by the socket
        port_number: Option<u16>,
        /// If the port number is dynamically assigned. Obsolete; set the port number to None instead
        port_dynamically_assigned: Option<bool>,
    },
    // RtpTp, Ieee1722Tp, HttpTp: currently not supported
}

//##################################################################

/// Describes if a [`SocketAddress`] is used for unicast or multicast
#[derive(Debug, Clone, PartialEq)]
pub enum SocketAddressType {
    /// The socket is used for unicast communication with a single ECU
    Unicast(Option<EcuInstance>),
    /// The socket is used for multicast communication with multiple ECUs
    Multicast(Vec<EcuInstance>),
}

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::communication::{IPv4AddressSource, NetworkEndpointAddress};
    use crate::{AutosarModelAbstraction, SystemCategory};
    use autosar_data::AutosarVersion;

    #[test]
    fn socket_address() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_4_3_0);
        let package = model.get_or_create_package("/pkg1").unwrap();
        let system = package.create_system("System", SystemCategory::SystemExtract).unwrap();
        let cluster = system.create_ethernet_cluster("Cluster", &package).unwrap();
        let channel = cluster.create_physical_channel("Channel", None).unwrap();

        let ecu_instance = system.create_ecu_instance("Ecu", &package).unwrap();
        let controller = ecu_instance
            .create_ethernet_communication_controller("EthCtrl", None)
            .unwrap();
        controller.connect_physical_channel("connection", &channel).unwrap();

        let ecu_instance2 = system.create_ecu_instance("Ecu2", &package).unwrap();
        let controller2 = ecu_instance2
            .create_ethernet_communication_controller("EthCtrl", None)
            .unwrap();
        controller2.connect_physical_channel("connection", &channel).unwrap();

        let ecu_instance3 = system.create_ecu_instance("Ecu3", &package).unwrap();
        let controller3 = ecu_instance3
            .create_ethernet_communication_controller("EthCtrl", None)
            .unwrap();
        controller3.connect_physical_channel("connection", &channel).unwrap();

        let endpoint_address = NetworkEndpointAddress::IPv4 {
            address: Some("192.168.0.1".to_string()),
            address_source: Some(IPv4AddressSource::Fixed),
            default_gateway: Some("192.168.0.2".to_string()),
            network_mask: Some("255.255.255.0".to_string()),
        };
        let network_endpoint = channel
            .create_network_endpoint("Address", endpoint_address, Some(&ecu_instance))
            .unwrap();
        let tcp_port = TpConfig::UdpTp {
            port_number: Some(1234),
            port_dynamically_assigned: None,
        };

        // create a unicast socket with an EcuInstance
        let socket_type: SocketAddressType = SocketAddressType::Unicast(Some(ecu_instance.clone()));
        let unicast_socket_address = channel
            .create_socket_address("Socket", &network_endpoint, &tcp_port, socket_type.clone())
            .unwrap();
        assert_eq!(channel.socket_addresses().count(), 1);
        assert_eq!(unicast_socket_address.network_endpoint().unwrap(), network_endpoint);
        assert_eq!(unicast_socket_address.socket_address_type().unwrap(), socket_type);
        // replace the EcuInstance in the socket
        unicast_socket_address.set_unicast_ecu(&ecu_instance2).unwrap();
        assert_eq!(
            unicast_socket_address.socket_address_type().unwrap(),
            SocketAddressType::Unicast(Some(ecu_instance2.clone()))
        );

        // create a unicast socket without an EcuInstance
        let socket_type: SocketAddressType = SocketAddressType::Unicast(None);
        let unicast_socket_address2 = channel
            .create_socket_address("Socket2", &network_endpoint, &tcp_port, socket_type.clone())
            .unwrap();
        // set the EcuInstance and verify that it is set
        unicast_socket_address2.set_unicast_ecu(&ecu_instance).unwrap();
        assert_eq!(
            unicast_socket_address2.socket_address_type().unwrap(),
            SocketAddressType::Unicast(Some(ecu_instance.clone()))
        );

        // create a multicast socket with multiple EcuInstances
        let socket_type: SocketAddressType =
            SocketAddressType::Multicast(vec![ecu_instance.clone(), ecu_instance2.clone()]);
        let multicast_socket_address = channel
            .create_socket_address("Socket3", &network_endpoint, &tcp_port, socket_type.clone())
            .unwrap();
        assert_eq!(multicast_socket_address.socket_address_type().unwrap(), socket_type);
        // add an EcuInstance to the multicast socket
        multicast_socket_address.add_multicast_ecu(&ecu_instance3).unwrap();
        assert_eq!(
            multicast_socket_address.socket_address_type().unwrap(),
            SocketAddressType::Multicast(vec![ecu_instance.clone(), ecu_instance2.clone(), ecu_instance3.clone()])
        );
    }

    #[test]
    fn socket_sd_config() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_4_3_0);
        let package = model.get_or_create_package("/pkg1").unwrap();
        let system = package.create_system("System", SystemCategory::SystemExtract).unwrap();
        let cluster = system.create_ethernet_cluster("Cluster", &package).unwrap();
        let channel = cluster.create_physical_channel("Channel", None).unwrap();

        // let ecu_instance = system.create_ecu_instance("Ecu", &package).unwrap();
        // let controller = ecu_instance
        //     .create_ethernet_communication_controller("EthCtrl", None)
        //     .unwrap();
        // controller.connect_physical_channel("connection", &channel).unwrap();

        let endpoint_address = NetworkEndpointAddress::IPv4 {
            address: Some("192.168.0.1".to_string()),
            address_source: Some(IPv4AddressSource::Fixed),
            default_gateway: None,
            network_mask: None,
        };
        let network_endpoint = channel
            .create_network_endpoint("Address", endpoint_address, None)
            .unwrap();
        let tcp_port = TpConfig::TcpTp {
            port_number: Some(1234),
            port_dynamically_assigned: None,
        };
        let socket_type: SocketAddressType = SocketAddressType::Unicast(None);
        let socket = channel
            .create_socket_address("Socket", &network_endpoint, &tcp_port, socket_type.clone())
            .unwrap();

        let provided_service_instance = socket.create_provided_service_instance("psi", 1, 2).unwrap();
        let consumed_service_instance = socket
            .create_consumed_service_instance("csi", &provided_service_instance)
            .unwrap();

        assert_eq!(socket.provided_service_instances().count(), 1);
        assert_eq!(
            socket.provided_service_instances().next().unwrap(),
            provided_service_instance
        );
        assert_eq!(socket.consumed_service_instances().count(), 1);
        assert_eq!(
            socket.consumed_service_instances().next().unwrap(),
            consumed_service_instance
        );
    }
}
