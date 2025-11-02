use crate::{
    AbstractionElement, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element,
    communication::{
        AbstractPhysicalChannel, LinCluster, LinCommunicationConnector, LinFrameTriggering, PhysicalChannel,
    },
};
use autosar_data::{Element, ElementName};

//##################################################################

/// The `LinPhysicalChannel` contains all of the communication on a LIN network
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinPhysicalChannel(Element);
abstraction_element!(LinPhysicalChannel, LinPhysicalChannel);
impl IdentifiableAbstractionElement for LinPhysicalChannel {}

impl LinPhysicalChannel {
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
    /// # let cluster = system.create_lin_cluster("Cluster", &package)?;
    /// let channel = cluster.create_physical_channel("Channel")?;
    /// let cluster_2 = channel.cluster()?;
    /// assert_eq!(cluster, cluster_2);
    /// # Ok(())}
    /// ```
    pub fn cluster(&self) -> Result<LinCluster, AutosarAbstractionError> {
        let cluster_elem = self.0.named_parent()?.unwrap();
        LinCluster::try_from(cluster_elem)
    }

    /// iterate over all frame triggerings of this physical channel
    pub fn frame_triggerings(&self) -> impl Iterator<Item = LinFrameTriggering> + Send + use<> {
        self.0
            .get_sub_element(ElementName::FrameTriggerings)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|elem| LinFrameTriggering::try_from(elem).ok())
    }
}

impl From<LinPhysicalChannel> for PhysicalChannel {
    fn from(channel: LinPhysicalChannel) -> Self {
        PhysicalChannel::Lin(channel)
    }
}

impl AbstractPhysicalChannel for LinPhysicalChannel {
    type CommunicationConnectorType = LinCommunicationConnector;
}
