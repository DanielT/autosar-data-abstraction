use crate::communication::{AbstractCluster, EthernetPhysicalChannel, EthernetVlanInfo};
use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element,
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
        AutosarModelAbstraction, SystemCategory,
        communication::{AbstractCluster, EthernetVlanInfo},
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
}
