use crate::communication::{AbstractCluster, CanNmCluster, CanPhysicalChannel};
use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element,
    get_reference_parents,
};
use autosar_data::{Element, ElementName};

/// A `CanCluster` contains all configuration items associated with a CAN network.
/// The cluster connects multiple ECUs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanCluster(Element);
abstraction_element!(CanCluster, CanCluster);
impl IdentifiableAbstractionElement for CanCluster {}

impl CanCluster {
    // create a new CanCluster - for internal use. User code should call System::create_can_cluster
    pub(crate) fn new(
        cluster_name: &str,
        package: &ArPackage,
        can_baudrate: Option<u32>,
    ) -> Result<Self, AutosarAbstractionError> {
        let elem_pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_cluster = elem_pkg_elements.create_named_sub_element(ElementName::CanCluster, cluster_name)?;
        if let Ok(cluster_content) = elem_cluster
            .create_sub_element(ElementName::CanClusterVariants)
            .and_then(|ccv| ccv.create_sub_element(ElementName::CanClusterConditional))
        {
            let _ = cluster_content
                .create_sub_element(ElementName::ProtocolName)
                .and_then(|pn| pn.set_character_data("CAN"));

            let _ = cluster_content.create_sub_element(ElementName::PhysicalChannels);
        }

        let can_cluster = CanCluster(elem_cluster);
        can_cluster.set_baudrate(can_baudrate.unwrap_or(500_000))?;

        Ok(can_cluster)
    }

    /// remove this `CanCluster` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        // remove the physical channel, if existing
        if let Some(channel) = self.physical_channel() {
            channel.remove(deep)?;
        }
        let ref_parents = get_reference_parents(&self.0)?;

        // delegate to the trait implementation to clean up all other references to the element and the element itself
        AbstractionElement::remove(self, deep)?;

        // check if any CanTpConfig or CanNmCluster uses this CanCluster
        // In both the cluster reference is mandatory, so we remove them together with the cluster
        for (named_parent, _parent) in ref_parents {
            match named_parent.element_name() {
                ElementName::CanTpConfig => {
                    if let Ok(can_tp_config) = crate::communication::CanTpConfig::try_from(named_parent) {
                        can_tp_config.remove(deep)?;
                    }
                }
                ElementName::CanNmCluster => {
                    if let Ok(can_nm_cluster) = CanNmCluster::try_from(named_parent) {
                        can_nm_cluster.remove(deep)?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// set the baudrate for this `CanCluster`
    pub fn set_baudrate(&self, baudrate: u32) -> Result<(), AutosarAbstractionError> {
        self.0
            .get_or_create_sub_element(ElementName::CanClusterVariants)
            .and_then(|ccv| ccv.get_or_create_sub_element(ElementName::CanClusterConditional))
            .and_then(|cc| cc.get_or_create_sub_element(ElementName::Baudrate))
            .and_then(|br| br.set_character_data(baudrate as u64))?;
        Ok(())
    }

    /// get the baudrate for this `CanCluster`
    #[must_use]
    pub fn baudrate(&self) -> Option<u32> {
        self.0
            .get_sub_element(ElementName::CanClusterVariants)
            .and_then(|ccv| ccv.get_sub_element(ElementName::CanClusterConditional))
            .and_then(|cc| cc.get_sub_element(ElementName::Baudrate))
            .and_then(|br| br.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set the baudrate for CAN FD for this `CanCluster`
    pub fn set_can_fd_baudrate(&self, baudrate: Option<u32>) -> Result<(), AutosarAbstractionError> {
        if let Some(baudrate) = baudrate {
            self.0
                .get_or_create_sub_element(ElementName::CanClusterVariants)
                .and_then(|ccv| ccv.get_or_create_sub_element(ElementName::CanClusterConditional))
                .and_then(|cc| cc.get_or_create_sub_element(ElementName::CanFdBaudrate))
                .and_then(|br| br.set_character_data(baudrate as u64))?;
        } else {
            let _ = self
                .0
                .get_sub_element(ElementName::CanClusterVariants)
                .and_then(|ccv| ccv.get_sub_element(ElementName::CanClusterConditional))
                .and_then(|cc| cc.remove_sub_element_kind(ElementName::CanFdBaudrate).ok());
        }
        Ok(())
    }

    /// get the baudrate for CAN FD for this `CanCluster`
    #[must_use]
    pub fn can_fd_baudrate(&self) -> Option<u32> {
        self.0
            .get_sub_element(ElementName::CanClusterVariants)
            .and_then(|ccv| ccv.get_sub_element(ElementName::CanClusterConditional))
            .and_then(|cc| cc.get_sub_element(ElementName::CanFdBaudrate))
            .and_then(|br| br.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set the baudrate for CAN XL for this `CanCluster`
    pub fn set_can_xl_baudrate(&self, baudrate: Option<u32>) -> Result<(), AutosarAbstractionError> {
        if let Some(baudrate) = baudrate {
            self.0
                .get_or_create_sub_element(ElementName::CanClusterVariants)
                .and_then(|ccv| ccv.get_or_create_sub_element(ElementName::CanClusterConditional))
                .and_then(|cc| cc.get_or_create_sub_element(ElementName::CanXlBaudrate))
                .and_then(|br| br.set_character_data(baudrate as u64))?;
        } else {
            let _ = self
                .0
                .get_sub_element(ElementName::CanClusterVariants)
                .and_then(|ccv| ccv.get_sub_element(ElementName::CanClusterConditional))
                .and_then(|cc| cc.remove_sub_element_kind(ElementName::CanXlBaudrate).ok());
        }
        Ok(())
    }

    /// get the baudrate for CAN XL for this `CanCluster`
    #[must_use]
    pub fn can_xl_baudrate(&self) -> Option<u32> {
        self.0
            .get_sub_element(ElementName::CanClusterVariants)
            .and_then(|ccv| ccv.get_sub_element(ElementName::CanClusterConditional))
            .and_then(|cc| cc.get_sub_element(ElementName::CanXlBaudrate))
            .and_then(|br| br.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// Create a new physical channel for the cluster
    ///
    /// A can cluster must contain exactly one physical channel; trying to add a second one triggers an error.
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
    /// let cluster = system.create_can_cluster("Cluster", &package, None)?;
    /// let channel = cluster.create_physical_channel("Channel")?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ItemAlreadyExists`] There is already a physical channel in this CAN cluster
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the ECU-INSTANCE
    pub fn create_physical_channel(&self, channel_name: &str) -> Result<CanPhysicalChannel, AutosarAbstractionError> {
        let phys_channels = self
            .0
            .get_or_create_sub_element(ElementName::CanClusterVariants)?
            .get_or_create_sub_element(ElementName::CanClusterConditional)?
            .get_or_create_sub_element(ElementName::PhysicalChannels)?;

        if phys_channels.sub_elements().count() != 0 {
            return Err(AutosarAbstractionError::ItemAlreadyExists);
        }

        let channel = phys_channels.create_named_sub_element(ElementName::CanPhysicalChannel, channel_name)?;

        CanPhysicalChannel::try_from(channel)
    }

    /// return the `CanPhysicalChannel` of the Cluster, if it has been created
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
    /// # let cluster = system.create_can_cluster("Cluster", &package, None)?;
    /// # let can_channel = cluster.create_physical_channel("Channel")?;
    /// if let Some(channel) = cluster.physical_channel() {
    /// #   assert_eq!(channel, can_channel);
    /// }
    /// # Ok(())}
    /// ```
    #[must_use]
    pub fn physical_channel(&self) -> Option<CanPhysicalChannel> {
        let channel = self
            .0
            .get_sub_element(ElementName::CanClusterVariants)?
            .get_sub_element(ElementName::CanClusterConditional)?
            .get_sub_element(ElementName::PhysicalChannels)?
            .get_sub_element(ElementName::CanPhysicalChannel)?;
        CanPhysicalChannel::try_from(channel).ok()
    }
}

impl AbstractCluster for CanCluster {}

//##################################################################

#[cfg(test)]
mod test {
    use crate::{
        AbstractionElement, AutosarModelAbstraction, SystemCategory,
        communication::{AbstractCluster, CanNmClusterSettings},
    };
    use autosar_data::AutosarVersion;

    #[test]
    fn cluster() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00051);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();

        let pkg2 = model.get_or_create_package("/can").unwrap();
        // create the CAN cluster CanCluster
        let result = system.create_can_cluster("CanCluster", &pkg2, None);
        assert!(result.is_ok());
        let cluster = result.unwrap();
        // creating the same cluster again is not possible
        let result = system.create_can_cluster("CanCluster", &pkg2, None);
        assert!(result.is_err());

        // system link
        let linked_system = cluster.system().unwrap();
        assert_eq!(linked_system, system);

        // settings for CanFd
        cluster.set_baudrate(250_000).unwrap();
        assert_eq!(cluster.baudrate().unwrap(), 250_000);
        cluster.set_can_fd_baudrate(Some(2_000_000)).unwrap();
        assert_eq!(cluster.can_fd_baudrate().unwrap(), 2_000_000);
        cluster.set_can_xl_baudrate(Some(10_000_000)).unwrap();
        assert_eq!(cluster.can_xl_baudrate().unwrap(), 10_000_000);
        // remove CanFd settings
        cluster.set_can_fd_baudrate(None).unwrap();
        assert!(cluster.can_fd_baudrate().is_none());
        // remove CanXl settings
        cluster.set_can_xl_baudrate(None).unwrap();
        assert!(cluster.can_xl_baudrate().is_none());

        // create a channel
        let result = cluster.create_physical_channel("Channel1");
        assert!(result.is_ok());
        // can't create a second channel
        let result = cluster.create_physical_channel("Channel2");
        assert!(result.is_err());

        let pc = cluster.physical_channel();
        assert!(pc.is_some());
    }

    #[test]
    fn remove_cluster() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00051);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();

        let pkg2 = model.get_or_create_package("/can").unwrap();
        let cluster = system.create_can_cluster("CanCluster", &pkg2, None).unwrap();
        let channel = cluster.create_physical_channel("Channel1").unwrap();

        let nm_config = system.create_nm_config("NmConfig", &pkg2).unwrap();
        let can_nm_cluster_settings = CanNmClusterSettings {
            nm_busload_reduction_active: false,
            nm_immediate_nm_transmissions: 5,
            nm_message_timeout_time: 5.0,
            nm_msg_cycle_time: 5.0,
            nm_network_timeout: 8.0,
            nm_remote_sleep_indication_time: 3.0,
            nm_repeat_message_time: 1.0,
            nm_wait_bus_sleep_time: 1.0,
        };
        let can_nm_cluster = nm_config
            .create_can_nm_cluster("CanNmCluster", &can_nm_cluster_settings, &cluster)
            .unwrap();
        let can_tp_config = system.create_can_tp_config("CanTpConfig", &pkg2, &cluster).unwrap();

        // remove the cluster
        let result = cluster.remove(true);
        assert!(result.is_ok());

        // check that the channel, CanNmCluster and CanTpConfig are also removed
        assert!(channel.element().path().is_err());
        assert!(can_nm_cluster.element().path().is_err());
        assert!(can_tp_config.element().path().is_err());
    }
}
