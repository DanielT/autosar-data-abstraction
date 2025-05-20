use crate::communication::{
    AbstractNmCluster, AbstractNmClusterCoupling, AbstractNmNode, FlexrayCluster, FlexrayCommunicationController, NmEcu,
};
use crate::{AbstractionElement, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element};
use autosar_data::{Element, ElementName, EnumItem};

//##################################################################

/// Flexray specific `NmCluster`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlexrayNmCluster(Element);
abstraction_element!(FlexrayNmCluster, FlexrayNmCluster);
impl IdentifiableAbstractionElement for FlexrayNmCluster {}

impl FlexrayNmCluster {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        settings: &FlexrayNmClusterSettings,
        flexray_cluster: &FlexrayCluster,
    ) -> Result<Self, AutosarAbstractionError> {
        let fr_nm_cluster_elem = parent.create_named_sub_element(ElementName::FlexrayNmCluster, name)?;
        let fr_nm_cluster = Self(fr_nm_cluster_elem);

        fr_nm_cluster.set_communication_cluster(flexray_cluster)?;
        fr_nm_cluster.set_nm_data_cycle(settings.nm_data_cycle)?;
        fr_nm_cluster.set_nm_remote_sleep_indication_time(settings.nm_remote_sleep_indication_time)?;
        fr_nm_cluster.set_nm_repeat_message_time(settings.nm_repeat_message_time)?;
        fr_nm_cluster.set_nm_repetition_cycle(settings.nm_repetition_cycle)?;
        fr_nm_cluster.set_nm_voting_cycle(settings.nm_voting_cycle)?;

        Ok(fr_nm_cluster)
    }

    /// set the nmDataCycle
    ///
    /// Number of `FlexRay` Communication Cycles needed to transmit the Nm Data PDUs of all `FlexRay` Nm Ecus of this `FlexRayNmCluster`.
    pub fn set_nm_data_cycle(&self, nm_data_cycle: u32) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmDataCycle)?
            .set_character_data(u64::from(nm_data_cycle))?;
        Ok(())
    }

    /// get the nmDataCycle
    ///
    /// Number of `FlexRay` Communication Cycles needed to transmit the Nm Data PDUs of all `FlexRay` Nm Ecus of this `FlexRayNmCluster`.
    #[must_use]
    pub fn nm_data_cycle(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::NmDataCycle)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set the nmRemoteSleepIndicationTime
    ///
    /// Timeout for Remote Sleep Indication in seconds.
    pub fn set_nm_remote_sleep_indication_time(
        &self,
        nm_remote_sleep_indication_time: f64,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmRemoteSleepIndicationTime)?
            .set_character_data(nm_remote_sleep_indication_time)?;
        Ok(())
    }

    /// get the nmRemoteSleepIndicationTime
    ///
    /// Timeout for Remote Sleep Indication in seconds.
    #[must_use]
    pub fn nm_remote_sleep_indication_time(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::NmRemoteSleepIndicationTime)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_float())
    }

    /// set the nmRepeatMessageTime
    ///
    /// Timeout for Repeat Message State in seconds.
    pub fn set_nm_repeat_message_time(&self, nm_repeat_message_time: f64) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmRepeatMessageTime)?
            .set_character_data(nm_repeat_message_time)?;
        Ok(())
    }

    /// get the nmRepeatMessageTime
    ///
    /// Timeout for Repeat Message State in seconds.
    #[must_use]
    pub fn nm_repeat_message_time(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::NmRepeatMessageTime)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_float())
    }

    /// set the nmRepetitionCycle
    ///
    /// Number of `FlexRay` Communication Cycles used to repeat the transmission of the Nm vote Pdus of all
    /// `FlexRay` `NmEcus` of this `FlexRayNmCluster`. This value shall be an integral multiple of nmVotingCycle.
    pub fn set_nm_repetition_cycle(&self, nm_repetition_cycle: u32) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmRepetitionCycle)?
            .set_character_data(u64::from(nm_repetition_cycle))?;
        Ok(())
    }

    /// get the nmRepetitionCycle
    ///
    /// Number of `FlexRay` Communication Cycles used to repeat the transmission of the Nm vote Pdus of all
    /// `FlexRay` `NmEcus` of this `FlexRayNmCluster`. This value shall be an integral multiple of nmVotingCycle.
    #[must_use]
    pub fn nm_repetition_cycle(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::NmRepetitionCycle)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set the nmVotingCycle
    ///
    /// The number of `FlexRay` Communication Cycles used to transmit the Nm Vote PDUs of all `FlexRay` Nm Ecus of this `FlexRayNmCluster`.
    pub fn set_nm_voting_cycle(&self, nm_voting_cycle: u32) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmVotingCycle)?
            .set_character_data(u64::from(nm_voting_cycle))?;
        Ok(())
    }

    /// get the nmVotingCycle
    ///
    /// The number of `FlexRay` Communication Cycles used to transmit the Nm Vote PDUs of all `FlexRay` Nm Ecus of this `FlexRayNmCluster`.
    #[must_use]
    pub fn nm_voting_cycle(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::NmVotingCycle)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// add a `FlexrayNmNode` to the cluster
    pub fn create_flexray_nm_node(
        &self,
        name: &str,
        controller: &FlexrayCommunicationController,
        nm_ecu: &NmEcu,
    ) -> Result<FlexrayNmNode, AutosarAbstractionError> {
        let nm_nodes = self.element().get_or_create_sub_element(ElementName::NmNodes)?;
        FlexrayNmNode::new(name, &nm_nodes, controller, nm_ecu)
    }
}

impl AbstractNmCluster for FlexrayNmCluster {
    type CommunicationClusterType = FlexrayCluster;
    type NmNodeType = FlexrayNmNode;
}

//##################################################################

/// Mandatory settings for a `FlexrayNmCluster`
///
/// These settings must be provided when creating a new `FlexrayNmCluster`.
/// Additional optional settings can be set using `FlexrayNmCluster` methods.
#[derive(Debug, Clone, PartialEq)]
pub struct FlexrayNmClusterSettings {
    /// nmDataCycle: Number of `FlexRay` Communication Cycles needed to transmit the Nm Data PDUs of all `FlexRay` Nm Ecus of this `FlexrayNmCluster`.
    pub nm_data_cycle: u32,
    /// nmRemoteSleepIndicationTime: Timeout for Remote Sleep Indication in seconds.
    pub nm_remote_sleep_indication_time: f64,
    /// nmRepeatMessageTime: Timeout for Repeat Message State in seconds.
    pub nm_repeat_message_time: f64,
    /// nmRepetitionCycle: Number of `FlexRay` Communication Cycles used to repeat the transmission of the Nm vote Pdus of all
    /// `FlexRay` `NmEcus` of this `FlexrayNmCluster`. This value shall be an integral multiple of nmVotingCycle.
    pub nm_repetition_cycle: u32,
    /// nmVotingCycle: The number of `FlexRay` Communication Cycles used to transmit the Nm Vote PDUs of all `FlexRay` Nm Ecus of this `FlexrayNmCluster`.
    pub nm_voting_cycle: u32,
}

//##################################################################

/// A `FlexRayNmClusterCoupling` couples multiple `FlexrayNmCluster`s.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlexrayNmClusterCoupling(Element);
abstraction_element!(FlexrayNmClusterCoupling, FlexrayNmClusterCoupling);

impl FlexrayNmClusterCoupling {
    pub(crate) fn new(
        parent: &Element,
        nm_schedule_variant: FlexrayNmScheduleVariant,
    ) -> Result<Self, AutosarAbstractionError> {
        let nm_cluster_coupling_elem = parent.create_sub_element(ElementName::FlexrayNmClusterCoupling)?;
        let nm_cluster_coupling = Self(nm_cluster_coupling_elem);
        nm_cluster_coupling.set_nm_schedule_variant(nm_schedule_variant)?;

        Ok(nm_cluster_coupling)
    }

    /// set the nmScheduleVariant
    pub fn set_nm_schedule_variant(
        &self,
        nm_schedule_variant: FlexrayNmScheduleVariant,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmScheduleVariant)?
            .set_character_data::<EnumItem>(nm_schedule_variant.into())?;
        Ok(())
    }

    /// get the nmScheduleVariant
    #[must_use]
    pub fn nm_schedule_variant(&self) -> Option<FlexrayNmScheduleVariant> {
        self.element()
            .get_sub_element(ElementName::NmScheduleVariant)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.enum_value())
            .and_then(|enum_item| FlexrayNmScheduleVariant::try_from(enum_item).ok())
    }
}

impl AbstractNmClusterCoupling for FlexrayNmClusterCoupling {
    type NmClusterType = FlexrayNmCluster;
}

//##################################################################

/// The `FlexrayNmScheduleVariant` defines the way the NM-Vote and NM-Data are transmitted within the `FlexRay` network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlexrayNmScheduleVariant {
    /// NM-Vote and NM Data transmitted within one PDU in static segment. The NM-Vote has to be realized as separate bit within the PDU.
    ScheduleVariant1,
    /// NM-Vote and NM-Data transmitted within one PDU in dynamic segment. The presence (or non-presence) of the PDU corresponds to the NM-Vote
    ScheduleVariant2,
    /// NM-Vote and NM-Data are transmitted in the static segment in separate PDUs. This alternative is not recommended => Alternative 1 should be used instead.
    ScheduleVariant3,
    /// NM-Vote transmitted in static and NM-Data transmitted in dynamic segment.
    ScheduleVariant4,
    /// NM-Vote is transmitted in dynamic and NM-Data is transmitted in static segment. This alternative is not recommended => Variants 2 or 6 should be used instead.
    ScheduleVariant5,
    /// NM-Vote and NM-Data are transmitted in the dynamic segment in separate PDUs.
    ScheduleVariant6,
    /// NM-Vote and a copy of the CBV are transmitted in the static segment (using the `FlexRay` NM Vector support) and NM-Data is transmitted in the dynamic segment
    ScheduleVariant7,
}

impl From<FlexrayNmScheduleVariant> for EnumItem {
    fn from(value: FlexrayNmScheduleVariant) -> Self {
        match value {
            FlexrayNmScheduleVariant::ScheduleVariant1 => EnumItem::ScheduleVariant1,
            FlexrayNmScheduleVariant::ScheduleVariant2 => EnumItem::ScheduleVariant2,
            FlexrayNmScheduleVariant::ScheduleVariant3 => EnumItem::ScheduleVariant3,
            FlexrayNmScheduleVariant::ScheduleVariant4 => EnumItem::ScheduleVariant4,
            FlexrayNmScheduleVariant::ScheduleVariant5 => EnumItem::ScheduleVariant5,
            FlexrayNmScheduleVariant::ScheduleVariant6 => EnumItem::ScheduleVariant6,
            FlexrayNmScheduleVariant::ScheduleVariant7 => EnumItem::ScheduleVariant7,
        }
    }
}

impl TryFrom<EnumItem> for FlexrayNmScheduleVariant {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<Self, Self::Error> {
        match value {
            EnumItem::ScheduleVariant1 => Ok(FlexrayNmScheduleVariant::ScheduleVariant1),
            EnumItem::ScheduleVariant2 => Ok(FlexrayNmScheduleVariant::ScheduleVariant2),
            EnumItem::ScheduleVariant3 => Ok(FlexrayNmScheduleVariant::ScheduleVariant3),
            EnumItem::ScheduleVariant4 => Ok(FlexrayNmScheduleVariant::ScheduleVariant4),
            EnumItem::ScheduleVariant5 => Ok(FlexrayNmScheduleVariant::ScheduleVariant5),
            EnumItem::ScheduleVariant6 => Ok(FlexrayNmScheduleVariant::ScheduleVariant6),
            EnumItem::ScheduleVariant7 => Ok(FlexrayNmScheduleVariant::ScheduleVariant7),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "FlexrayNmScheduleVariant".to_string(),
            }),
        }
    }
}

//##################################################################

/// A `FlexrayNmNode` represents a `FlexRay` specific `NmNode`.
///
/// It connects a `FlexrayCommunicationController` with a `NmEcu`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlexrayNmNode(Element);
abstraction_element!(FlexrayNmNode, FlexrayNmNode);
impl IdentifiableAbstractionElement for FlexrayNmNode {}

impl FlexrayNmNode {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        controller: &FlexrayCommunicationController,
        nm_ecu: &NmEcu,
    ) -> Result<Self, AutosarAbstractionError> {
        let fr_nm_node_elem = parent.create_named_sub_element(ElementName::FlexrayNmNode, name)?;
        let fr_nm_node = Self(fr_nm_node_elem);
        fr_nm_node.set_communication_controller(controller)?;
        fr_nm_node.set_nm_ecu(nm_ecu)?;

        Ok(fr_nm_node)
    }
}

impl AbstractNmNode for FlexrayNmNode {
    type CommunicationControllerType = FlexrayCommunicationController;
}
