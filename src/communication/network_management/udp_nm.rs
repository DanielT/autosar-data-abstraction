use crate::communication::{
    AbstractNmCluster, AbstractNmClusterCoupling, AbstractNmNode, EthernetCluster, EthernetCommunicationController,
    EthernetPhysicalChannel, NmEcu,
};
use crate::{abstraction_element, AbstractionElement, AutosarAbstractionError, IdentifiableAbstractionElement};
use autosar_data::{Element, ElementName};

//##################################################################

/// Udp / Ethernet specific `NmCluster`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UdpNmCluster(Element);
abstraction_element!(UdpNmCluster, UdpNmCluster);
impl IdentifiableAbstractionElement for UdpNmCluster {}

impl UdpNmCluster {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        settings: &UdpNmClusterSettings,
        ethernet_cluster: &EthernetCluster,
    ) -> Result<Self, AutosarAbstractionError> {
        let nm_cluster = parent.create_named_sub_element(ElementName::UdpNmCluster, name)?;
        let udp_nm_cluster = Self(nm_cluster);
        udp_nm_cluster.set_communication_cluster(ethernet_cluster)?;
        udp_nm_cluster.set_nm_msg_cycle_time(settings.nm_msg_cycle_time)?;
        udp_nm_cluster.set_nm_message_timeout_time(settings.nm_msg_timeout_time)?;
        udp_nm_cluster.set_nm_network_timeout(settings.nm_network_timeout)?;
        udp_nm_cluster.set_nm_remote_sleep_indication_time(settings.nm_remote_sleep_indication_time)?;
        udp_nm_cluster.set_nm_repeat_message_time(settings.nm_repeat_message_time)?;
        udp_nm_cluster.set_nm_wait_bus_sleep_time(settings.nm_wait_bus_sleep_time)?;

        Ok(udp_nm_cluster)
    }

    /// set the nmMsgCycleTime
    pub fn set_nm_msg_cycle_time(&self, cycle_time: f64) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmMsgCycleTime)?
            .set_character_data(cycle_time)?;
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

    /// set the nmMessageTimeoutTime
    pub fn set_nm_message_timeout_time(&self, timeout_time: f64) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmMessageTimeoutTime)?
            .set_character_data(timeout_time)?;
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

    /// set the `NmNetworkTimeout`
    pub fn set_nm_network_timeout(&self, timeout: f64) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmNetworkTimeout)?
            .set_character_data(timeout)?;
        Ok(())
    }

    /// get the `NmNetworkTimeout`
    #[must_use]
    pub fn nm_network_timeout(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::NmNetworkTimeout)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_float())
    }

    /// set the `NmRemoteSleepIndicationTime`
    pub fn set_nm_remote_sleep_indication_time(&self, time: f64) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmRemoteSleepIndicationTime)?
            .set_character_data(time)?;
        Ok(())
    }

    /// get the `NmRemoteSleepIndicationTime`
    #[must_use]
    pub fn nm_remote_sleep_indication_time(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::NmRemoteSleepIndicationTime)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_float())
    }

    /// set the `NmRepeatMessageTime`
    pub fn set_nm_repeat_message_time(&self, time: f64) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmRepeatMessageTime)?
            .set_character_data(time)?;
        Ok(())
    }

    /// get the `NmRepeatMessageTime`
    #[must_use]
    pub fn nm_repeat_message_time(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::NmRepeatMessageTime)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_float())
    }

    /// set the `NmWaitBusSleepTime`
    pub fn set_nm_wait_bus_sleep_time(&self, time: f64) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmWaitBusSleepTime)?
            .set_character_data(time)?;
        Ok(())
    }

    /// get the `NmWaitBusSleepTime`
    #[must_use]
    pub fn nm_wait_bus_sleep_time(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::NmWaitBusSleepTime)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_float())
    }

    /// add a `UdpNmNode` to the cluster
    pub fn create_udp_nm_node(
        &self,
        name: &str,
        controller: &EthernetCommunicationController,
        nm_ecu: &NmEcu,
        nm_msg_cycle_offset: f64,
    ) -> Result<UdpNmNode, AutosarAbstractionError> {
        let nm_nodes = self.element().get_or_create_sub_element(ElementName::NmNodes)?;
        UdpNmNode::new(name, &nm_nodes, controller, nm_ecu, nm_msg_cycle_offset)
    }

    /// set or delete the Vlan associated with the cluster through an `EthernetPhysicalChannel` reference
    ///
    /// If `vlan` is `Some`, the Vlan is set to the value of `vlan`. If `vlan` is `None`, the Vlan is removed.
    pub fn set_vlan(&self, vlan: Option<&EthernetPhysicalChannel>) -> Result<(), AutosarAbstractionError> {
        if let Some(vlan) = vlan {
            self.element()
                .get_or_create_sub_element(ElementName::VlanRef)?
                .set_reference_target(vlan.element())?;
        } else {
            self.element().remove_sub_element_kind(ElementName::VlanRef)?;
        }
        Ok(())
    }

    /// get the Vlan associated with the cluster
    #[must_use]
    pub fn vlan(&self) -> Option<EthernetPhysicalChannel> {
        self.element()
            .get_sub_element(ElementName::VlanRef)
            .and_then(|vlan_ref| vlan_ref.get_reference_target().ok())
            .and_then(|elem| elem.try_into().ok())
    }

    /// set or delete the value nmImmediateNmTransmissions
    pub fn set_nm_immediate_nm_transmissions(&self, value: Option<u32>) -> Result<(), AutosarAbstractionError> {
        if let Some(value) = value {
            self.element()
                .get_or_create_sub_element(ElementName::NmImmediateNmTransmissions)?
                .set_character_data(u64::from(value))?;
        } else {
            self.element()
                .remove_sub_element_kind(ElementName::NmImmediateNmTransmissions)?;
        }
        Ok(())
    }

    /// get the value of nmImmediateNmTransmissions
    #[must_use]
    pub fn nm_immediate_nm_transmissions(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::NmImmediateNmTransmissions)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set or delete the value nmCbvPosition
    pub fn set_nm_cbv_position(&self, value: Option<u32>) -> Result<(), AutosarAbstractionError> {
        if let Some(value) = value {
            self.element()
                .get_or_create_sub_element(ElementName::NmCbvPosition)?
                .set_character_data(u64::from(value))?;
        } else {
            self.element().remove_sub_element_kind(ElementName::NmCbvPosition)?;
        }
        Ok(())
    }

    /// get the value of nmCbvPosition
    #[must_use]
    pub fn nm_cbv_position(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::NmCbvPosition)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set or delete the value nmNidPosition
    pub fn set_nm_nid_position(&self, value: Option<u32>) -> Result<(), AutosarAbstractionError> {
        if let Some(value) = value {
            self.element()
                .get_or_create_sub_element(ElementName::NmNidPosition)?
                .set_character_data(u64::from(value))?;
        } else {
            self.element().remove_sub_element_kind(ElementName::NmNidPosition)?;
        }
        Ok(())
    }

    /// get the value of nmNidPosition
    #[must_use]
    pub fn nm_nid_position(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::NmNidPosition)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }
}

impl AbstractNmCluster for UdpNmCluster {
    type CommunicationClusterType = EthernetCluster;
    type NmNodeType = UdpNmNode;
}

/// `UdpNmClusterSettings` encapsulates the mandatory settings for a `UdpNmCluster`
#[derive(Debug, Clone, PartialEq)]
pub struct UdpNmClusterSettings {
    /// Period of an `NmPdu` in seconds
    pub nm_msg_cycle_time: f64,
    /// Timeout of a `NmPdu` in seconds
    pub nm_msg_timeout_time: f64,
    /// Network Timeout for `NmPdus` in seconds
    pub nm_network_timeout: f64,
    /// Timeout for Remote Sleep Indication in seconds
    pub nm_remote_sleep_indication_time: f64,
    /// Timeout for Repeat Message State in seconds
    pub nm_repeat_message_time: f64,
    /// Timeout for bus calm down phase in seconds
    pub nm_wait_bus_sleep_time: f64,
}

//##################################################################

/// Udp / Ethernet specific `NmClusterCoupling`
///
/// It couples multiple `UdpNmCluster`s and provides UdpNm-specific settings
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UdpNmClusterCoupling(Element);
abstraction_element!(UdpNmClusterCoupling, UdpNmClusterCoupling);

impl UdpNmClusterCoupling {
    pub(crate) fn new(parent: &Element) -> Result<Self, AutosarAbstractionError> {
        let nm_cluster_coupling = parent.create_sub_element(ElementName::UdpNmClusterCoupling)?;
        Ok(Self(nm_cluster_coupling))
    }

    /// set or remove the nmImmediateRestartEnabled flag
    ///
    /// If `enabled` is `Some`, the flag is set to the value of `enabled`.
    /// If `enabled` is `None`, the flag is removed.
    pub fn set_nm_immediate_restart_enabled(&self, enabled: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(enabled) = enabled {
            self.element()
                .get_or_create_sub_element(ElementName::NmImmediateRestartEnabled)?
                .set_character_data(enabled)?;
        } else {
            self.element()
                .remove_sub_element_kind(ElementName::NmImmediateRestartEnabled)?;
        }
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

impl AbstractNmClusterCoupling for UdpNmClusterCoupling {
    type NmClusterType = UdpNmCluster;
}

//##################################################################

/// Udp / Ethernet specific `NmNode`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UdpNmNode(Element);
abstraction_element!(UdpNmNode, UdpNmNode);
impl IdentifiableAbstractionElement for UdpNmNode {}

impl UdpNmNode {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        controller: &EthernetCommunicationController,
        nm_ecu: &NmEcu,
        nm_msg_cycle_offset: f64,
    ) -> Result<Self, AutosarAbstractionError> {
        let udp_nm_node_elem = parent.create_named_sub_element(ElementName::UdpNmNode, name)?;
        let udp_nm_node = Self(udp_nm_node_elem);
        udp_nm_node.set_communication_controller(controller)?;
        udp_nm_node.set_nm_ecu(nm_ecu)?;
        udp_nm_node.set_nm_msg_cycle_offset(nm_msg_cycle_offset)?;

        Ok(udp_nm_node)
    }

    /// set the `NmMsgCycleOffset`
    pub fn set_nm_msg_cycle_offset(&self, offset: f64) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NmMsgCycleOffset)?
            .set_character_data(offset)?;
        Ok(())
    }

    /// get the `NmMsgCycleOffset`
    #[must_use]
    pub fn nm_msg_cycle_offset(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::NmMsgCycleOffset)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_float())
    }

    /// set ot remove the allNmMessagesKeepAwake flag
    ///
    /// If `enabled` is `Some`, the flag is set to the value of `enabled`. If `enabled` is `None`, the flag is removed.
    pub fn set_all_nm_messages_keep_awake(&self, enabled: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(enabled) = enabled {
            self.element()
                .get_or_create_sub_element(ElementName::AllNmMessagesKeepAwake)?
                .set_character_data(enabled)?;
        } else {
            self.element()
                .remove_sub_element_kind(ElementName::AllNmMessagesKeepAwake)?;
        }
        Ok(())
    }

    /// get the allNmMessagesKeepAwake flag
    #[must_use]
    pub fn all_nm_messages_keep_awake(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::AllNmMessagesKeepAwake)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_bool())
    }
}

impl AbstractNmNode for UdpNmNode {
    type CommunicationControllerType = EthernetCommunicationController;
}
