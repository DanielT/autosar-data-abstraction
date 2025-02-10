use crate::{
    abstraction_element,
    communication::{
        AbstractPhysicalChannel, CanAddressingMode, CanCluster, CanCommunicationConnector, CanFrame,
        CanFrameTriggering, CanFrameType,
    },
    AbstractionElement, AutosarAbstractionError, IdentifiableAbstractionElement,
};
use autosar_data::{Element, ElementName};

/// The `CanPhysicalChannel contains all of the communication on a CAN network
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanPhysicalChannel(Element);
abstraction_element!(CanPhysicalChannel, CanPhysicalChannel);
impl IdentifiableAbstractionElement for CanPhysicalChannel {}

impl CanPhysicalChannel {
    /// get the cluster containing this physical channel
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
    /// # let cluster = system.create_can_cluster("Cluster", &package, &CanClusterSettings::default())?;
    /// let channel = cluster.create_physical_channel("Channel")?;
    /// let cluster_2 = channel.cluster()?;
    /// assert_eq!(cluster, cluster_2);
    /// # Ok(())}
    /// ```
    pub fn cluster(&self) -> Result<CanCluster, AutosarAbstractionError> {
        let cluster_elem = self.0.named_parent()?.unwrap();
        CanCluster::try_from(cluster_elem)
    }

    /// add a trigger for a CAN frame in this physical channel
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
    /// # let frame_package = model.get_or_create_package("/Frames")?;
    /// # let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// # let cluster = system.create_can_cluster("Cluster", &package, &CanClusterSettings::default())?;
    /// let channel = cluster.create_physical_channel("Channel")?;
    /// let frame = system.create_can_frame("Frame", &frame_package, 8)?;
    /// channel.trigger_frame(&frame, 0x100, CanAddressingMode::Standard, CanFrameType::Can20)?;
    /// # Ok(())}
    /// ```
    pub fn trigger_frame(
        &self,
        frame: &CanFrame,
        identifier: u32,
        addressing_mode: CanAddressingMode,
        frame_type: CanFrameType,
    ) -> Result<CanFrameTriggering, AutosarAbstractionError> {
        CanFrameTriggering::new(self, frame, identifier, addressing_mode, frame_type)
    }

    /// iterate over all frame triggerings of this physical channel
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::{*, communication::*};
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
    /// # let package = model.get_or_create_package("/pkg1")?;
    /// # let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// # let cluster = system.create_can_cluster("Cluster", &package, &CanClusterSettings::default())?;
    /// # let channel = cluster.create_physical_channel("Channel")?;
    /// # let frame = system.create_can_frame("Frame", &package, 8)?;
    /// channel.trigger_frame(&frame, 0x100, CanAddressingMode::Standard, CanFrameType::Can20)?;
    /// for ft in channel.frame_triggerings() {
    ///     println!("Frame triggering: {:?}", ft);
    /// }
    /// # assert_eq!(channel.frame_triggerings().count(), 1);
    /// # Ok(())}
    pub fn frame_triggerings(&self) -> impl Iterator<Item = CanFrameTriggering> + Send + 'static {
        self.0
            .get_sub_element(ElementName::FrameTriggerings)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|elem| CanFrameTriggering::try_from(elem).ok())
    }
}

impl AbstractPhysicalChannel for CanPhysicalChannel {
    type CommunicationConnectorType = CanCommunicationConnector;
}

//##################################################################

#[cfg(test)]
mod test {
    use crate::{communication::CanClusterSettings, AutosarModelAbstraction, SystemCategory};
    use autosar_data::AutosarVersion;

    #[test]
    fn channel() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();
        let settings = CanClusterSettings::default();
        let cluster = system.create_can_cluster("CanCluster", &pkg, &settings).unwrap();

        let channel = cluster.create_physical_channel("channel_name").unwrap();
        let c2 = channel.cluster().unwrap();
        assert_eq!(cluster, c2);
    }
}
