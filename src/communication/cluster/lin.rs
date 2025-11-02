use crate::communication::{AbstractCluster, LinPhysicalChannel};
use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element,
};
use autosar_data::{Element, ElementName};

//##################################################################

/// A `LinCluster` contains all configuration items associated with a LIN network.
/// The cluster connects multiple ECUs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinCluster(Element);
abstraction_element!(LinCluster, LinCluster);
impl IdentifiableAbstractionElement for LinCluster {}

impl LinCluster {
    // create a new LinCluster - for internal use. User code should call System::create_lin_cluster
    pub(crate) fn new(cluster_name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elem_pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_cluster = elem_pkg_elements.create_named_sub_element(ElementName::LinCluster, cluster_name)?;
        if let Ok(cluster_content) = elem_cluster
            .create_sub_element(ElementName::LinClusterVariants)
            .and_then(|ccv| ccv.create_sub_element(ElementName::LinClusterConditional))
        {
            let _ = cluster_content
                .create_sub_element(ElementName::ProtocolName)
                .and_then(|pn| pn.set_character_data("CAN"));

            let _ = cluster_content.create_sub_element(ElementName::PhysicalChannels);
        }

        let lin_cluster = LinCluster(elem_cluster);

        Ok(lin_cluster)
    }

    /// Create a new physical channel for the cluster
    ///
    /// A LIN cluster must contain exactly one physical channel; trying to add a second one triggers an error.
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # use autosar_data_abstraction::communication::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00051);
    /// # let package = model.get_or_create_package("/pkg1")?;
    /// # let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// let cluster = system.create_lin_cluster("Cluster", &package)?;
    /// let channel = cluster.create_physical_channel("Channel")?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ItemAlreadyExists`] There is already a physical channel in this LIN cluster
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the ECU-INSTANCE
    pub fn create_physical_channel(&self, channel_name: &str) -> Result<LinPhysicalChannel, AutosarAbstractionError> {
        let phys_channels = self
            .0
            .get_or_create_sub_element(ElementName::LinClusterVariants)?
            .get_or_create_sub_element(ElementName::LinClusterConditional)?
            .get_or_create_sub_element(ElementName::PhysicalChannels)?;

        if phys_channels.sub_elements().count() != 0 {
            return Err(AutosarAbstractionError::ItemAlreadyExists);
        }

        let channel = phys_channels.create_named_sub_element(ElementName::LinPhysicalChannel, channel_name)?;

        LinPhysicalChannel::try_from(channel)
    }

    /// return the `LinPhysicalChannel` of the Cluster, if it has been created
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
    pub fn physical_channel(&self) -> Option<LinPhysicalChannel> {
        let channel = self
            .0
            .get_sub_element(ElementName::LinClusterVariants)?
            .get_sub_element(ElementName::LinClusterConditional)?
            .get_sub_element(ElementName::PhysicalChannels)?
            .get_sub_element(ElementName::LinPhysicalChannel)?;
        LinPhysicalChannel::try_from(channel).ok()
    }
}

impl AbstractCluster for LinCluster {}

//##################################################################

#[cfg(test)]
mod test {
    use crate::{AutosarModelAbstraction, SystemCategory, communication::AbstractCluster};
    use autosar_data::AutosarVersion;

    #[test]
    fn cluster() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00051);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();

        let pkg2 = model.get_or_create_package("/lin").unwrap();
        // create the LIN cluster LinCluster
        let result = system.create_lin_cluster("LinCluster", &pkg2);
        assert!(result.is_ok());
        let cluster = result.unwrap();
        // creating the same cluster again is not possible
        let result = system.create_lin_cluster("LinCluster", &pkg2);
        assert!(result.is_err());

        // system link
        let linked_system = cluster.system().unwrap();
        assert_eq!(linked_system, system);

        // create a channel
        let result = cluster.create_physical_channel("Channel1");
        assert!(result.is_ok());
        // can't create a second channel
        let result = cluster.create_physical_channel("Channel2");
        assert!(result.is_err());

        let pc = cluster.physical_channel();
        assert!(pc.is_some());
    }
}
