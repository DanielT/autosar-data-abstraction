use crate::communication::{
    AbstractCluster, DoIpTpConfig, EthernetPhysicalChannel, EthernetVlanInfo, SomeipTpConfig, UdpNmCluster,
};
use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element,
    get_reference_parents,
};
use autosar_data::{Element, ElementName};

/// An `EthernetCluster` contains all configuration items associated with an ethernet network.
/// The cluster connects multiple ECUs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EthernetCluster(Element);
abstraction_element!(EthernetCluster, EthernetCluster);
impl IdentifiableAbstractionElement for EthernetCluster {}

impl EthernetCluster {
    // create a new EthernetCluster - for internal use. User code should call System::create_ethernet_cluster
    pub(crate) fn new(cluster_name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elem_pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_cluster = elem_pkg_elements.create_named_sub_element(ElementName::EthernetCluster, cluster_name)?;
        if let Ok(cluster_content) = elem_cluster
            .create_sub_element(ElementName::EthernetClusterVariants)
            .and_then(|ecv| ecv.create_sub_element(ElementName::EthernetClusterConditional))
        {
            let _ = cluster_content.create_sub_element(ElementName::PhysicalChannels);
        }

        Ok(EthernetCluster(elem_cluster))
    }

    /// remove this `EthernetCluster` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        // remove all the physical channels
        for channel in self.physical_channels() {
            channel.remove(deep)?;
        }
        let ref_parents = get_reference_parents(self.element())?;

        // delegate to the trait implementation to clean up all other references to the element and the element itself
        AbstractionElement::remove(self, deep)?;

        // check if any DoipTpConfig, or UdpNmCluster or SomeIpTpConfig uses this EthernetCluster
        // The cluster reference is mandatory these elements, so we remove them together with the cluster
        for (named_parent, _parent) in ref_parents {
            match named_parent.element_name() {
                ElementName::DoIpTpConfig => {
                    if let Ok(doip_tp_config) = DoIpTpConfig::try_from(named_parent) {
                        doip_tp_config.remove(deep)?;
                    }
                }
                ElementName::UdpNmCluster => {
                    if let Ok(udp_nm_cluster) = UdpNmCluster::try_from(named_parent) {
                        udp_nm_cluster.remove(deep)?;
                    }
                }
                ElementName::SomeipTpConfig => {
                    if let Ok(someip_tp_config) = SomeipTpConfig::try_from(named_parent) {
                        someip_tp_config.remove(deep)?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Create a new physical channel for the cluster
    ///
    /// The supplied VLAN info must be unique - there cannot be two VLANs with the same vlan identifier.
    /// One channel may be created without VLAN information; it carries untagged traffic.
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
    /// let cluster = system.create_ethernet_cluster("Cluster", &package)?;
    /// let vlan_info = EthernetVlanInfo {
    ///     vlan_name: "VLAN_1".to_string(),
    ///     vlan_id: 1,
    /// };
    /// let channel = cluster.create_physical_channel("Channel", Some(&vlan_info))?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ItemAlreadyExists`] There is already a physical channel for this VLAN
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the ECU-INSTANCE
    pub fn create_physical_channel(
        &self,
        channel_name: &str,
        vlan_info: Option<&EthernetVlanInfo>,
    ) -> Result<EthernetPhysicalChannel, AutosarAbstractionError> {
        let phys_channels = self
            .0
            .get_or_create_sub_element(ElementName::EthernetClusterVariants)?
            .get_or_create_sub_element(ElementName::EthernetClusterConditional)?
            .get_or_create_sub_element(ElementName::PhysicalChannels)?;

        EthernetPhysicalChannel::new(channel_name, &phys_channels, vlan_info)
    }

    /// returns an iterator over all [`EthernetPhysicalChannel`]s in the cluster
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// # let package = model.get_or_create_package("/pkg1")?;
    /// # let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// # let cluster = system.create_ethernet_cluster("Cluster", &package)?;
    /// cluster.create_physical_channel("Channel", None)?;
    /// for channel in cluster.physical_channels() {
    ///     // ...
    /// }
    /// # Ok(())}
    /// ```
    pub fn physical_channels(&self) -> impl Iterator<Item = EthernetPhysicalChannel> + Send + use<> {
        self.element()
            .get_sub_element(ElementName::EthernetClusterVariants)
            .and_then(|ecv| ecv.get_sub_element(ElementName::EthernetClusterConditional))
            .and_then(|ecc| ecc.get_sub_element(ElementName::PhysicalChannels))
            .into_iter()
            .flat_map(|phys_channel| phys_channel.sub_elements())
            .filter_map(|elem| EthernetPhysicalChannel::try_from(elem).ok())
    }
}

impl AbstractCluster for EthernetCluster {}

//##################################################################

#[cfg(test)]
mod test {
    use crate::{
        AbstractionElement, AutosarModelAbstraction, SystemCategory,
        communication::{AbstractCluster, EthernetVlanInfo, UdpNmClusterSettings},
    };
    use autosar_data::AutosarVersion;

    #[test]
    fn cluster() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();

        let pkg2 = model.get_or_create_package("/ethernet").unwrap();
        // create the ethernet cluster EthCluster
        let result = system.create_ethernet_cluster("EthCluster", &pkg2);
        assert!(result.is_ok());
        let cluster = result.unwrap();
        // creating the same cluster again is not possible
        let result = system.create_ethernet_cluster("EthCluster", &pkg2);
        assert!(result.is_err());

        // system link
        let linked_system = cluster.system().unwrap();
        assert_eq!(linked_system, system);

        // create an untagged channel
        let result = cluster.create_physical_channel("Channel1", None);
        assert!(result.is_ok());
        // can't create a second untagged channel
        let result = cluster.create_physical_channel("Channel2", None);
        assert!(result.is_err());

        // create a channel for VLAN 1
        let vlan_info = EthernetVlanInfo {
            vlan_name: "VLAN_1".to_string(),
            vlan_id: 1,
        };
        let result = cluster.create_physical_channel("Channel3", Some(&vlan_info));
        assert!(result.is_ok());

        // can't create a second channel called Channel3
        let vlan_info = EthernetVlanInfo {
            vlan_name: "VLAN_2".to_string(),
            vlan_id: 2,
        };
        let result = cluster.create_physical_channel("Channel3", Some(&vlan_info));
        assert!(result.is_err());

        // create a channel for VLAN 2
        let vlan_info = EthernetVlanInfo {
            vlan_name: "VLAN_2".to_string(),
            vlan_id: 2,
        };
        let result = cluster.create_physical_channel("Channel4", Some(&vlan_info));
        assert!(result.is_ok());

        // can't create a second channel for VLAN 2
        let vlan_info = EthernetVlanInfo {
            vlan_name: "VLAN_2".to_string(),
            vlan_id: 2,
        };
        let result = cluster.create_physical_channel("Channel5", Some(&vlan_info));
        assert!(result.is_err());

        let count = cluster.physical_channels().count();
        assert_eq!(count, 3);
    }

    #[test]
    fn remove_cluster() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();
        let pkg2 = model.get_or_create_package("/ethernet").unwrap();
        let cluster = system.create_ethernet_cluster("EthCluster", &pkg2).unwrap();
        let channel = cluster.create_physical_channel("Channel1", None).unwrap();

        let doip_tp_config = system.create_doip_tp_config("DoIpTpConfig", &pkg2, &cluster).unwrap();
        let nm_config = system.create_nm_config("NmConfig", &pkg2).unwrap();
        let settings = UdpNmClusterSettings {
            nm_msg_cycle_time: 5.0,
            nm_msg_timeout_time: 5.0,
            nm_network_timeout: 5.0,
            nm_remote_sleep_indication_time: 5.0,
            nm_repeat_message_time: 5.0,
            nm_wait_bus_sleep_time: 5.0,
        };
        let udp_nm_cluster = nm_config
            .create_udp_nm_cluster("UdpNmCluster", &settings, &cluster)
            .unwrap();

        let someip_tp_config = system
            .create_someip_tp_config("SomeipTpConfig", &pkg2, &cluster)
            .unwrap();
        // remove the cluster
        let result = cluster.remove(true);
        assert!(result.is_ok());
        // check that the channel, DoIpTpConfig, UdpNmCluster and SomeIpTpConfig are also removed
        assert!(channel.element().path().is_err());
        assert!(doip_tp_config.element().path().is_err());
        assert!(udp_nm_cluster.element().path().is_err());
        assert!(someip_tp_config.element().path().is_err());
    }
}
