use crate::communication::EthernetPhysicalChannel;
use crate::{AbstractionElement, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element};
use autosar_data::{CharacterData, Element, ElementName, EnumItem};

/// A network endpoint contains address information for a connection
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NetworkEndpoint(Element);
abstraction_element!(NetworkEndpoint, NetworkEndpoint);
impl IdentifiableAbstractionElement for NetworkEndpoint {}

impl NetworkEndpoint {
    pub(crate) fn new(
        name: &str,
        channel: &EthernetPhysicalChannel,
        address: NetworkEndpointAddress,
    ) -> Result<Self, AutosarAbstractionError> {
        let el_network_endpoint = channel
            .element()
            .get_or_create_sub_element(ElementName::NetworkEndpoints)?
            .create_named_sub_element(ElementName::NetworkEndpoint, name)?;

        let network_endpoint = Self(el_network_endpoint);
        let result = network_endpoint.add_network_endpoint_address(address);
        if let Err(error) = result {
            let _ = channel.element().remove_sub_element(network_endpoint.0);
            return Err(error);
        }

        Ok(network_endpoint)
    }

    /// add a network endpoint address to this `NetworkEndpoint`
    ///
    /// A `NetworkEndpoint` may have multiple sets of address information. The following restrictions apply:
    ///
    /// - all addresses must have the same type, i.e. all IPv4 or all IPv6
    /// - only one of them may be a `Fixed` address, all others must be dynamic (DHCP, automatic link local, etc.)
    pub fn add_network_endpoint_address(&self, address: NetworkEndpointAddress) -> Result<(), AutosarAbstractionError> {
        let mut fixedcount = 0;
        if matches!(address, NetworkEndpointAddress::IPv4 { address_source, .. } if address_source == Some(IPv4AddressSource::Fixed))
            || matches!(address, NetworkEndpointAddress::IPv6 { address_source, .. } if address_source == Some(IPv6AddressSource::Fixed))
        {
            fixedcount = 1;
        }
        for existing_address in self.addresses() {
            if std::mem::discriminant(&existing_address) != std::mem::discriminant(&address) {
                return Err(AutosarAbstractionError::InvalidParameter(
                    "you cannot mix IPv4 and IPv6 inside one NetworkEndpoint".to_string(),
                ));
            }
            if matches!(existing_address, NetworkEndpointAddress::IPv4 { address_source, .. } if address_source == Some(IPv4AddressSource::Fixed))
                || matches!(existing_address, NetworkEndpointAddress::IPv6 { address_source, .. } if address_source == Some(IPv6AddressSource::Fixed))
            {
                fixedcount += 1;
            }
        }
        if fixedcount > 1 {
            return Err(AutosarAbstractionError::InvalidParameter(
                "Only one NetworkEndpointAddress can be a fixed address".to_string(),
            ));
        }

        let addresses = self
            .0
            .get_or_create_sub_element(ElementName::NetworkEndpointAddresses)?;
        match address {
            NetworkEndpointAddress::IPv4 {
                address,
                address_source,
                default_gateway,
                network_mask,
            } => {
                let cfg = addresses.create_sub_element(ElementName::Ipv4Configuration)?;
                if let Some(addr) = address {
                    cfg.create_sub_element(ElementName::Ipv4Address)?
                        .set_character_data(addr)?;
                }
                if let Some(addr_src) = address_source {
                    cfg.create_sub_element(ElementName::Ipv4AddressSource)?
                        .set_character_data::<EnumItem>(addr_src.into())?;
                }
                if let Some(defgw) = default_gateway {
                    cfg.create_sub_element(ElementName::DefaultGateway)?
                        .set_character_data(defgw)?;
                }
                if let Some(netmask) = network_mask {
                    cfg.create_sub_element(ElementName::NetworkMask)?
                        .set_character_data(netmask)?;
                }
            }
            NetworkEndpointAddress::IPv6 {
                address,
                address_source,
                default_router,
            } => {
                let cfg = addresses.create_sub_element(ElementName::Ipv6Configuration)?;
                if let Some(addr) = address {
                    cfg.create_sub_element(ElementName::Ipv6Address)?
                        .set_character_data(addr)?;
                }
                if let Some(addr_src) = address_source {
                    cfg.create_sub_element(ElementName::Ipv6AddressSource)?
                        .set_character_data(addr_src.to_cdata())?;
                }
                if let Some(dr) = default_router {
                    cfg.create_sub_element(ElementName::DefaultRouter)?
                        .set_character_data(dr)?;
                }
            }
        }
        Ok(())
    }

    /// iterator over all addresses in the `NetworkEndpoint`
    pub fn addresses(&self) -> impl Iterator<Item = NetworkEndpointAddress> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::NetworkEndpointAddresses)
            .into_iter()
            .flat_map(|addresses| addresses.sub_elements())
            .filter_map(|elem| NetworkEndpointAddress::try_from(elem).ok())
    }
}

//##################################################################

/// address information for a network endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkEndpointAddress {
    /// IPv4 addressing information
    IPv4 {
        /// IPv4 address in the form "a.b.c.d". This is used if the address source is FIXED
        address: Option<String>,
        /// defines how the address is obtained
        address_source: Option<IPv4AddressSource>,
        /// ip address of the default gateway
        default_gateway: Option<String>,
        /// Network mask in the form "a.b.c.d"
        network_mask: Option<String>,
    },
    /// IPv6 addressing information
    IPv6 {
        /// IPv6 address, without abbreviation
        address: Option<String>,
        /// defines how the address is obtained
        address_source: Option<IPv6AddressSource>,
        /// IP address of the default router
        default_router: Option<String>,
    },
}

impl TryFrom<Element> for NetworkEndpointAddress {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::Ipv4Configuration => {
                let address = element
                    .get_sub_element(ElementName::Ipv4Address)
                    .and_then(|i4a| i4a.character_data())
                    .and_then(|cdata| cdata.string_value());
                let address_source = element
                    .get_sub_element(ElementName::Ipv4AddressSource)
                    .and_then(|i4as| i4as.character_data())
                    .and_then(IPv4AddressSource::from_cdata);
                let default_gateway = element
                    .get_sub_element(ElementName::DefaultGateway)
                    .and_then(|dg| dg.character_data())
                    .and_then(|cdata| cdata.string_value());
                let network_mask = element
                    .get_sub_element(ElementName::NetworkMask)
                    .and_then(|nm| nm.character_data())
                    .and_then(|cdata| cdata.string_value());

                Ok(NetworkEndpointAddress::IPv4 {
                    address,
                    address_source,
                    default_gateway,
                    network_mask,
                })
            }
            ElementName::Ipv6Configuration => {
                let address = element
                    .get_sub_element(ElementName::Ipv6Address)
                    .and_then(|i6a| i6a.character_data())
                    .and_then(|cdata| cdata.string_value());
                let address_source = element
                    .get_sub_element(ElementName::Ipv6AddressSource)
                    .and_then(|i6as| i6as.character_data())
                    .and_then(IPv6AddressSource::from_cdata);
                let default_router = element
                    .get_sub_element(ElementName::DefaultRouter)
                    .and_then(|dr| dr.character_data())
                    .and_then(|cdata| cdata.string_value());

                Ok(NetworkEndpointAddress::IPv6 {
                    address,
                    address_source,
                    default_router,
                })
            }
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "NetwworkEndpointAddress".to_string(),
            }),
        }
    }
}

/// `IPv4AddressSource` defines how the address of an IPv4 `NetworkEndpoint` is obtained
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IPv4AddressSource {
    /// use `AutoIp` (aka APIPA) to assign a link-local address
    AutoIp,
    /// use `AutoIp` with `DoIp` settings to assign a link-local address
    AutoIpDoIp,
    /// dynamic assignment using DHCP
    DHCPv4,
    /// static IP address configuration - the address must be specified in `NetworkEndpointAddress`
    Fixed,
}

impl IPv4AddressSource {
    fn from_cdata(cdata: CharacterData) -> Option<Self> {
        match cdata {
            CharacterData::Enum(EnumItem::AutoIp) => Some(Self::AutoIp),
            CharacterData::Enum(EnumItem::AutoIpDoip) => Some(Self::AutoIpDoIp),
            CharacterData::Enum(EnumItem::Dhcpv4) => Some(Self::DHCPv4),
            CharacterData::Enum(EnumItem::Fixed) => Some(Self::Fixed),
            _ => None,
        }
    }
}

impl From<IPv4AddressSource> for EnumItem {
    fn from(value: IPv4AddressSource) -> Self {
        match value {
            IPv4AddressSource::AutoIp => EnumItem::AutoIp,
            IPv4AddressSource::AutoIpDoIp => EnumItem::AutoIpDoip,
            IPv4AddressSource::DHCPv4 => EnumItem::Dhcpv4,
            IPv4AddressSource::Fixed => EnumItem::Fixed,
        }
    }
}

/// `IPv6AddressSource` defines how the address of an IPv6 `NetworkEndpoint` is obtained
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IPv6AddressSource {
    /// dynamic assignment using DHCP
    DHCPv6,
    /// static IP address configuration - the address must be specified in `NetworkEndpointAddress`
    Fixed,
    /// automatic link local address assignment
    LinkLocal,
    /// automatic link local address assignment using doip parameters
    LinkLocalDoIp,
    /// IPv6 stateless autoconfiguration
    RouterAdvertisement,
}

impl IPv6AddressSource {
    fn from_cdata(cdata: CharacterData) -> Option<Self> {
        match cdata {
            CharacterData::Enum(EnumItem::Dhcpv6) => Some(Self::DHCPv6),
            CharacterData::Enum(EnumItem::Fixed) => Some(Self::Fixed),
            CharacterData::Enum(EnumItem::LinkLocal) => Some(Self::LinkLocal),
            CharacterData::Enum(EnumItem::LinkLocalDoip) => Some(Self::LinkLocalDoIp),
            CharacterData::Enum(EnumItem::RouterAdvertisement) => Some(Self::RouterAdvertisement),
            _ => None,
        }
    }

    fn to_cdata(self) -> CharacterData {
        match self {
            Self::DHCPv6 => CharacterData::Enum(EnumItem::Dhcpv6),
            Self::Fixed => CharacterData::Enum(EnumItem::Fixed),
            Self::LinkLocal => CharacterData::Enum(EnumItem::LinkLocal),
            Self::LinkLocalDoIp => CharacterData::Enum(EnumItem::LinkLocalDoip),
            Self::RouterAdvertisement => CharacterData::Enum(EnumItem::RouterAdvertisement),
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
    fn test_network_endpoint_ipv4() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();
        let cluster = system.create_ethernet_cluster("EthCluster", &pkg).unwrap();
        let channel = cluster.create_physical_channel("Channel", None).unwrap();

        // create a static socket connection between the local_socket and the remote_socket
        let address1 = NetworkEndpointAddress::IPv4 {
            address: Some("192.168.0.1".to_string()),
            address_source: Some(IPv4AddressSource::Fixed),
            default_gateway: Some("192.168.0.2".to_string()),
            network_mask: Some("255.255.0.0".to_string()),
        };
        let network_endpoint = channel
            .create_network_endpoint("RemoteAddress", address1.clone(), None)
            .unwrap();
        assert_eq!(network_endpoint.addresses().count(), 1);
        assert_eq!(network_endpoint.addresses().next().unwrap(), address1);

        let address2 = NetworkEndpointAddress::IPv4 {
            address: None,
            address_source: Some(IPv4AddressSource::AutoIp),
            default_gateway: None,
            network_mask: None,
        };
        network_endpoint.add_network_endpoint_address(address2).unwrap();
        assert_eq!(network_endpoint.addresses().count(), 2);
    }

    #[test]
    fn test_network_endpoint_ipv6() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();
        let cluster = system.create_ethernet_cluster("EthCluster", &pkg).unwrap();
        let channel = cluster.create_physical_channel("Channel", None).unwrap();

        // create a static socket connection between the local_socket and the remote_socket
        let address1 = NetworkEndpointAddress::IPv6 {
            address: Some("2001:0db8:0000:0000:0000:0000:0000:0001".to_string()),
            address_source: Some(IPv6AddressSource::Fixed),
            default_router: Some("2001:0db8:0000:0000:0000:0000:0000:0002".to_string()),
        };
        let network_endpoint = channel
            .create_network_endpoint("RemoteAddress", address1.clone(), None)
            .unwrap();
        assert_eq!(network_endpoint.addresses().count(), 1);
        assert_eq!(network_endpoint.addresses().next().unwrap(), address1);

        let address2 = NetworkEndpointAddress::IPv6 {
            address: None,
            address_source: Some(IPv6AddressSource::LinkLocal),
            default_router: None,
        };
        network_endpoint.add_network_endpoint_address(address2).unwrap();
        assert_eq!(network_endpoint.addresses().count(), 2);
    }
}
