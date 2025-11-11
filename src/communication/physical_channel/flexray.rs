use crate::{
    AbstractionElement, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element,
    communication::{
        AbstractPhysicalChannel, FlexrayCluster, FlexrayCommunicationConnector, FlexrayCommunicationCycle,
        FlexrayFrame, FlexrayFrameTriggering, PhysicalChannel,
    },
};
use autosar_data::{Element, ElementName, EnumItem};

/// the `FlexrayPhysicalChannel` represents either channel A or B of Flexray cluster
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlexrayPhysicalChannel(Element);
abstraction_element!(FlexrayPhysicalChannel, FlexrayPhysicalChannel);
impl IdentifiableAbstractionElement for FlexrayPhysicalChannel {}

impl FlexrayPhysicalChannel {
    /// get the channel name of a `FlexrayPhysicalChannel`
    #[must_use]
    pub fn channel_name(&self) -> Option<FlexrayChannelName> {
        let cn = self
            .0
            .get_sub_element(ElementName::ChannelName)?
            .character_data()?
            .enum_value()?;
        match cn {
            EnumItem::ChannelA => Some(FlexrayChannelName::A),
            EnumItem::ChannelB => Some(FlexrayChannelName::B),
            _ => None,
        }
    }

    /// remove this `FlexrayPhysicalChannel` from the model
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
    /// # let cluster = system.create_flexray_cluster("Cluster", &package, &FlexrayClusterSettings::default())?;
    /// let channel = cluster.create_physical_channel("Channel", FlexrayChannelName::A)?;
    /// let cluster_2 = channel.cluster()?;
    /// assert_eq!(cluster, cluster_2);
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model
    pub fn cluster(&self) -> Result<FlexrayCluster, AutosarAbstractionError> {
        let cluster_elem = self.0.named_parent()?.unwrap();
        FlexrayCluster::try_from(cluster_elem)
    }

    /// add a trigger for a flexray frame in this physical channel
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
    /// # let cluster = system.create_flexray_cluster("Cluster", &package, &FlexrayClusterSettings::default())?;
    /// let channel = cluster.create_physical_channel("Channel", FlexrayChannelName::A)?;
    /// let frame = system.create_flexray_frame("Frame", &frame_package, 64)?;
    /// let timing = FlexrayCommunicationCycle::Repetition {base_cycle: 1, cycle_repetition: CycleRepetition::C1};
    /// channel.trigger_frame(&frame, 1, &timing)?;
    /// # Ok(())}
    /// ```
    pub fn trigger_frame(
        &self,
        frame: &FlexrayFrame,
        slot_id: u16,
        timing: &FlexrayCommunicationCycle,
    ) -> Result<FlexrayFrameTriggering, AutosarAbstractionError> {
        FlexrayFrameTriggering::new(self, frame, slot_id, timing)
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
    /// # let cluster = system.create_flexray_cluster("Cluster", &package, &FlexrayClusterSettings::default())?;
    /// # let channel = cluster.create_physical_channel("Channel", FlexrayChannelName::A)?;
    /// # let frame = system.create_flexray_frame("Frame", &package, 64)?;
    /// # let timing = FlexrayCommunicationCycle::Repetition {base_cycle: 1, cycle_repetition: CycleRepetition::C1};
    /// channel.trigger_frame(&frame, 1, &timing)?;
    /// for ft in channel.frame_triggerings() {
    ///     println!("Frame triggering: {:?}", ft);
    /// }
    /// # assert_eq!(channel.frame_triggerings().count(), 1);
    /// # Ok(())}
    pub fn frame_triggerings(&self) -> impl Iterator<Item = FlexrayFrameTriggering> + Send + use<> {
        self.0
            .get_sub_element(ElementName::FrameTriggerings)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|elem| FlexrayFrameTriggering::try_from(elem).ok())
    }
}

impl From<FlexrayPhysicalChannel> for PhysicalChannel {
    fn from(channel: FlexrayPhysicalChannel) -> Self {
        PhysicalChannel::Flexray(channel)
    }
}

impl AbstractPhysicalChannel for FlexrayPhysicalChannel {
    type CommunicationConnectorType = FlexrayCommunicationConnector;
}

//##################################################################

/// A flexray cluster may contain the channels A and/or B.
///
/// This enum is an abstraction over the \<CHANNEL-NAME\> element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexrayChannelName {
    /// Channel A
    A,
    /// Channel B
    B,
}

//##################################################################

#[cfg(test)]
mod test {
    use crate::{
        AbstractionElement, AutosarModelAbstraction, ByteOrder, SystemCategory,
        communication::{AbstractFrame, FlexrayChannelName, FlexrayClusterSettings},
    };
    use autosar_data::{AutosarVersion, ElementName};

    #[test]
    fn channel() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();
        let settings = FlexrayClusterSettings::default();
        let cluster = system.create_flexray_cluster("FlxCluster", &pkg, &settings).unwrap();

        let channel = cluster
            .create_physical_channel("channel_name", FlexrayChannelName::A)
            .unwrap();
        let c2 = channel.cluster().unwrap();
        assert_eq!(cluster, c2);

        let wrapped_channel: super::PhysicalChannel = channel.clone().into();
        assert_eq!(wrapped_channel, super::PhysicalChannel::Flexray(channel.clone()));

        // damage the channel info by removing the channel name
        let elem_channelname = channel.element().get_sub_element(ElementName::ChannelName).unwrap();
        elem_channelname.remove_character_data().unwrap();
        assert!(channel.channel_name().is_none());

        // now there is no longer a channel A
        let channel2 = cluster.create_physical_channel("channel_name2", FlexrayChannelName::A);
        assert!(channel2.is_ok());
    }

    #[test]
    fn remove_channel() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let pkg = model.get_or_create_package("/test").unwrap();
        let system = pkg.create_system("System", SystemCategory::SystemDescription).unwrap();
        let settings = FlexrayClusterSettings::default();
        let cluster = system.create_flexray_cluster("FlxCluster", &pkg, &settings).unwrap();

        let channel = cluster
            .create_physical_channel("channel_name", FlexrayChannelName::A)
            .unwrap();

        let frame = system.create_flexray_frame("FlexrayFrame", &pkg, 8).unwrap();
        let timing = crate::communication::FlexrayCommunicationCycle::Repetition {
            base_cycle: 1,
            cycle_repetition: crate::communication::CycleRepetition::C1,
        };
        let _ = channel.trigger_frame(&frame, 1, &timing).unwrap();
        let isignal_ipdu = system.create_isignal_ipdu("ISignalIPdu", &pkg, 8).unwrap();
        let _ = frame
            .map_pdu(&isignal_ipdu, 0, ByteOrder::MostSignificantByteLast, None)
            .unwrap();

        channel.remove(true).unwrap();

        assert!(cluster.physical_channels().channel_a.is_none());
        assert!(cluster.physical_channels().channel_b.is_none());

        // the PDU was removed, because it was unused and deep removal was requested
        assert!(isignal_ipdu.element().parent().is_err());
    }
}
