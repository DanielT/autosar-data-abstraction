use crate::communication::{
    AbstractNmCluster, AbstractNmClusterCoupling, AbstractNmNode, CanCluster, CanCommunicationController, NmEcu,
};
use crate::{AbstractionElement, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element};
use autosar_data::{Element, ElementName};

//##################################################################

/// Can specific `NmCluster` attributes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanNmCluster(Element);
abstraction_element!(CanNmCluster, CanNmCluster);
impl IdentifiableAbstractionElement for CanNmCluster {}

impl CanNmCluster {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        settings: &CanNmClusterSettings,
        can_cluster: &CanCluster,
    ) -> Result<Self, AutosarAbstractionError> {
        let can_nm_cluster_elem = parent.create_named_sub_element(ElementName::CanNmCluster, name)?;
        let can_nm_cluster = Self(can_nm_cluster_elem);

        can_nm_cluster.set_communication_cluster(can_cluster)?;
        can_nm_cluster.set_nm_busload_reduction_active(settings.nm_busload_reduction_active)?;
        can_nm_cluster.set_nm_immediate_nm_transmissions(settings.nm_immediate_nm_transmissions)?;
        can_nm_cluster.set_nm_message_timeout_time(settings.nm_message_timeout_time)?;
        can_nm_cluster.set_nm_msg_cycle_time(settings.nm_msg_cycle_time)?;
        can_nm_cluster.set_nm_network_timeout(settings.nm_network_timeout)?;
        can_nm_cluster.set_nm_remote_sleep_indication_time(settings.nm_remote_sleep_indication_time)?;
        can_nm_cluster.set_nm_repeat_message_time(settings.nm_repeat_message_time)?;
        can_nm_cluster.set_nm_wait_bus_sleep_time(settings.nm_wait_bus_sleep_time)?;

        Ok(can_nm_cluster)
    }

    /// set the nmBusloadReductionActive flag
    pub fn set_nm_busload_reduction_active(
        &self,
        nm_busload_reduction_active: bool,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmBusloadReductionActive)?
            .set_character_data(nm_busload_reduction_active)?;
        Ok(())
    }

    /// get the nmBusloadReductionActive flag
    #[must_use]
    pub fn nm_busload_reduction_active(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::NmBusloadReductionActive)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_bool())
    }

    /// set the nmImmediateNmTransmissions value
    pub fn set_nm_immediate_nm_transmissions(
        &self,
        nm_immediate_nm_transmissions: u32,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmImmediateNmTransmissions)?
            .set_character_data(u64::from(nm_immediate_nm_transmissions))?;
        Ok(())
    }

    /// get the nmImmediateNmTransmissions value
    #[must_use]
    pub fn nm_immediate_nm_transmissions(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::NmImmediateNmTransmissions)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set the nmMessageTimeoutTime
    pub fn set_nm_message_timeout_time(&self, nm_message_timeout_time: f64) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmMessageTimeoutTime)?
            .set_character_data(nm_message_timeout_time)?;
        Ok(())
    }

    /// get the nmMessageTimeoutTime
    #[must_use]
    pub fn nm_message_timeout_time(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::NmMessageTimeoutTime)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_float())
    }

    /// set the nmMsgCycleTime
    pub fn set_nm_msg_cycle_time(&self, nm_msg_cycle_time: f64) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmMsgCycleTime)?
            .set_character_data(nm_msg_cycle_time)?;
        Ok(())
    }

    /// get the nmMsgCycleTime
    #[must_use]
    pub fn nm_msg_cycle_time(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::NmMsgCycleTime)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_float())
    }

    /// set the nmNetworkTimeout
    pub fn set_nm_network_timeout(&self, nm_network_timeout: f64) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmNetworkTimeout)?
            .set_character_data(nm_network_timeout)?;
        Ok(())
    }

    /// get the nmNetworkTimeout
    #[must_use]
    pub fn nm_network_timeout(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::NmNetworkTimeout)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_float())
    }

    /// set the nmRemoteSleepIndicationTime
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
    #[must_use]
    pub fn nm_remote_sleep_indication_time(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::NmRemoteSleepIndicationTime)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_float())
    }

    /// set the nmRepeatMessageTime
    pub fn set_nm_repeat_message_time(&self, nm_repeat_message_time: f64) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmRepeatMessageTime)?
            .set_character_data(nm_repeat_message_time)?;
        Ok(())
    }

    /// get the nmRepeatMessageTime
    #[must_use]
    pub fn nm_repeat_message_time(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::NmRepeatMessageTime)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_float())
    }

    /// set the nmWaitBusSleepTime
    pub fn set_nm_wait_bus_sleep_time(&self, nm_wait_bus_sleep_time: f64) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmWaitBusSleepTime)?
            .set_character_data(nm_wait_bus_sleep_time)?;
        Ok(())
    }

    /// get the nmWaitBusSleepTime
    #[must_use]
    pub fn nm_wait_bus_sleep_time(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::NmWaitBusSleepTime)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_float())
    }

    /// add a `CanNmNode` to the cluster
    pub fn create_can_nm_node(
        &self,
        name: &str,
        controller: &CanCommunicationController,
        nm_ecu: &NmEcu,
    ) -> Result<CanNmNode, AutosarAbstractionError> {
        let nm_nodes = self.element().get_or_create_sub_element(ElementName::NmNodes)?;
        CanNmNode::new(name, &nm_nodes, controller, nm_ecu)
    }
}

impl AbstractNmCluster for CanNmCluster {
    type CommunicationClusterType = CanCluster;
    type NmNodeType = CanNmNode;
}

//##################################################################

/// Mandatory settings for a `CanNmCluster`
///
/// These settings are mandatory for a `CanNmCluster` and must be set during creation.
/// Additional optional settings can be set using the `CanNmCluster` methods.
#[derive(Debug, Clone, PartialEq)]
pub struct CanNmClusterSettings {
    /// nmBusloadReductionActive: Determines if bus load reduction for the respective `CanNm` channel is active.
    pub nm_busload_reduction_active: bool,
    /// nmImmediateNmTransmissions: Defines the number of immediate `NmPdus` which shall be transmitted.
    /// If the value is zero no immediate `NmPdus` are transmitted.
    pub nm_immediate_nm_transmissions: u32,
    /// nmMessageTimeoutTime: Timeout of an `NmPdu` in seconds.
    pub nm_message_timeout_time: f64,
    /// nmMsgCycleTime: Period of a `NmPdu` in seconds
    pub nm_msg_cycle_time: f64,
    /// nmNetworkTimeout: Network Timeout for `NmPdus` in seconds.
    pub nm_network_timeout: f64,
    /// nmRemoteSleepIndicationTime: Timeout for Remote Sleep Indication in seconds.
    pub nm_remote_sleep_indication_time: f64,
    /// nmRepeatMessageTime: Timeout for Repeat Message State in seconds.
    pub nm_repeat_message_time: f64,
    /// nmWaitBusSleepTime: Timeout for bus calm down phase in seconds.
    pub nm_wait_bus_sleep_time: f64,
}

//##################################################################

/// A `CanNmClusterCoupling` couples multiple `CanNmCluster`s, and contains CAN specific settings.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanNmClusterCoupling(Element);
abstraction_element!(CanNmClusterCoupling, CanNmClusterCoupling);

impl CanNmClusterCoupling {
    pub(crate) fn new(
        parent: &Element,
        nm_busload_reduction_enabled: bool,
        nm_immediate_restart_enabled: bool,
    ) -> Result<Self, AutosarAbstractionError> {
        let nm_cluster_coupling_elem = parent.create_sub_element(ElementName::CanNmClusterCoupling)?;
        let nm_cluster_coupling = Self(nm_cluster_coupling_elem);
        nm_cluster_coupling.set_nm_busload_reduction_enabled(nm_busload_reduction_enabled)?;
        nm_cluster_coupling.set_nm_immediate_restart_enabled(nm_immediate_restart_enabled)?;

        Ok(nm_cluster_coupling)
    }

    /// set the nmBusloadReductionEnabled flag
    pub fn set_nm_busload_reduction_enabled(
        &self,
        nm_busload_reduction_enabled: bool,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmBusloadReductionEnabled)?
            .set_character_data(nm_busload_reduction_enabled)?;
        Ok(())
    }

    /// get the nmBusloadReductionEnabled flag
    #[must_use]
    pub fn nm_busload_reduction_enabled(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::NmBusloadReductionEnabled)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_bool())
    }

    /// set the nmImmediateRestartEnabled flag
    pub fn set_nm_immediate_restart_enabled(
        &self,
        nm_immediate_restart_enabled: bool,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmImmediateRestartEnabled)?
            .set_character_data(nm_immediate_restart_enabled)?;
        Ok(())
    }

    /// get the nmImmediateRestartEnabled flag
    #[must_use]
    pub fn nm_immediate_restart_enabled(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::NmImmediateRestartEnabled)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_bool())
    }
}

impl AbstractNmClusterCoupling for CanNmClusterCoupling {
    type NmClusterType = CanNmCluster;
}

//##################################################################

/// A `CanNmNode` represents a node in a `CanNmCluster`.
///
/// The node connects to a `CanCommunicationController` and an `NmEcu`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanNmNode(Element);
abstraction_element!(CanNmNode, CanNmNode);
impl IdentifiableAbstractionElement for CanNmNode {}

impl CanNmNode {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        controller: &CanCommunicationController,
        nm_ecu: &NmEcu,
    ) -> Result<Self, AutosarAbstractionError> {
        let can_nm_node_elem = parent.create_named_sub_element(ElementName::CanNmNode, name)?;
        let can_nm_ecu = Self(can_nm_node_elem);
        can_nm_ecu.set_communication_controller(controller)?;
        can_nm_ecu.set_nm_ecu(nm_ecu)?;

        Ok(can_nm_ecu)
    }
}

impl AbstractNmNode for CanNmNode {
    type CommunicationControllerType = CanCommunicationController;
}
