use crate::{
    AbstractionElement, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element,
    communication::{
        AbstractPhysicalChannel, LinCluster, LinCommunicationConnector, LinFrame, LinFrameTriggering, PhysicalChannel,
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

    /// remove this `LinPhysicalChannel` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        // remove all frame triggerings of this physical channel
        for ft in self.frame_triggerings() {
            ft.remove(deep)?;
        }

        // remove all pdu triggerings of this physical channel
        for pt in self.pdu_triggerings() {
            pt.remove(deep)?;
        }

        // remove all signal triggerings of this physical channel
        for st in self.signal_triggerings() {
            st.remove(deep)?;
        }

        // remove all connectors using this physical channel
        for connector in self.connectors() {
            connector.remove(deep)?;
        }

        AbstractionElement::remove(self, deep)
    }

    /// add a trigger for a LIN frame in this physical channel
    pub fn trigger_frame<T: Into<LinFrame> + AbstractionElement>(
        &self,
        frame: &T,
        identifier: u32,
    ) -> Result<LinFrameTriggering, AutosarAbstractionError> {
        LinFrameTriggering::new(self, &frame.clone().into(), identifier)
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

//##################################################################

#[cfg(test)]
mod test {
    use crate::{
        AbstractionElement, AutosarModelAbstraction, ByteOrder, SystemCategory,
        communication::{AbstractFrame, AbstractPhysicalChannel, PhysicalChannel},
    };
    use autosar_data::AutosarVersion;

    #[test]
    fn channel() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();
        let cluster = system.create_lin_cluster("CanCluster", &pkg).unwrap();

        let channel = cluster.create_physical_channel("channel_name").unwrap();
        let c2 = channel.cluster().unwrap();
        assert_eq!(cluster, c2);

        let wrapped_channel: PhysicalChannel = channel.clone().into();
        assert_eq!(wrapped_channel, PhysicalChannel::Lin(channel));
    }

    #[test]
    fn remove_channel() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();
        let cluster = system.create_lin_cluster("LinCluster", &pkg).unwrap();

        let channel = cluster.create_physical_channel("channel_name").unwrap();

        let frame = system.create_lin_unconditional_frame("LinFrame", &pkg, 8).unwrap();
        let _ = channel.trigger_frame(&frame, 0x123).unwrap();
        let isignal_ipdu = system.create_isignal_ipdu("ISignalIPdu", &pkg, 8).unwrap();
        let _ = frame
            .map_pdu(&isignal_ipdu, 0, ByteOrder::MostSignificantByteLast, None)
            .unwrap();

        assert_eq!(channel.frame_triggerings().count(), 1);
        assert_eq!(channel.pdu_triggerings().count(), 1);

        channel.remove(true).unwrap();
        assert!(cluster.physical_channel().is_none());
        // the PDU was removed, because it was unused and deep removal was requested
        assert!(isignal_ipdu.element().parent().is_err());
    }
}
