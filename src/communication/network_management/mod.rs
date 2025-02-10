use crate::communication::{
    AbstractCluster, AbstractCommunicationController, CanCluster, EthernetCluster, FlexrayCluster, NmPdu,
};
use crate::{
    abstraction_element, AbstractionElement, ArPackage, AutosarAbstractionError, EcuInstance,
    IdentifiableAbstractionElement,
};
use autosar_data::{Element, ElementName};

mod can_nm;
mod flexray_nm;
mod udp_nm;

pub use can_nm::*;
pub use flexray_nm::*;
pub use udp_nm::*;

//##################################################################

/// The `NmConfig` is the root element for the network management configuration.
///
/// Only one config may exist per `System`, and this configuration may contain multiple `NmClusters` for different bus types.
///
/// Use [`System::create_nm_config`](crate::System::create_nm_config) to create a new `NmConfig` in a `System`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NmConfig(Element);
abstraction_element!(NmConfig, NmConfig);
impl IdentifiableAbstractionElement for NmConfig {}

impl NmConfig {
    /// create a new `NmConfig` in the given `ArPackage`
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let nm_config = Self(elements.create_named_sub_element(ElementName::NmConfig, name)?);
        nm_config
            .element()
            .create_sub_element(ElementName::Category)?
            .set_character_data("NM_CONFIG")?;
        Ok(nm_config)
    }

    /// create a new `CanNmCluster`
    pub fn create_can_nm_cluster(
        &self,
        name: &str,
        settings: &CanNmClusterSettings,
        can_cluster: &CanCluster,
    ) -> Result<CanNmCluster, AutosarAbstractionError> {
        let nmclusters = self.element().get_or_create_sub_element(ElementName::NmClusters)?;
        CanNmCluster::new(name, &nmclusters, settings, can_cluster)
    }

    /// create a new `FlexrayNmCluster`
    pub fn create_flexray_nm_cluster(
        &self,
        name: &str,
        settings: &FlexrayNmClusterSettings,
        flexray_cluster: &FlexrayCluster,
    ) -> Result<FlexrayNmCluster, AutosarAbstractionError> {
        let nmclusters = self.element().get_or_create_sub_element(ElementName::NmClusters)?;
        FlexrayNmCluster::new(name, &nmclusters, settings, flexray_cluster)
    }

    /// create a new `UdpNmCluster`
    pub fn create_udp_nm_cluster(
        &self,
        name: &str,
        settings: &UdpNmClusterSettings,
        ethernet_cluster: &EthernetCluster,
    ) -> Result<UdpNmCluster, AutosarAbstractionError> {
        let nmclusters = self.element().get_or_create_sub_element(ElementName::NmClusters)?;
        UdpNmCluster::new(name, &nmclusters, settings, ethernet_cluster)
    }

    /// get all `NmClusters`
    pub fn nm_clusters(&self) -> impl Iterator<Item = NmCluster> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::NmClusters)
            .into_iter()
            .flat_map(|clusters| clusters.sub_elements())
            .filter_map(|elem| elem.try_into().ok())
    }

    /// create a new `CanNmClusterCoupling`
    pub fn create_can_nm_cluster_coupling(
        &self,
        nm_busload_reduction_enabled: bool,
        nm_immediate_restart_enabled: bool,
    ) -> Result<CanNmClusterCoupling, AutosarAbstractionError> {
        let nmcluster_couplings = self
            .element()
            .get_or_create_sub_element(ElementName::NmClusterCouplings)?;
        CanNmClusterCoupling::new(
            &nmcluster_couplings,
            nm_busload_reduction_enabled,
            nm_immediate_restart_enabled,
        )
    }

    /// create a new `FlexrayNmClusterCoupling`
    pub fn create_flexray_nm_cluster_coupling(
        &self,
        nm_schedule_variant: FlexrayNmScheduleVariant,
    ) -> Result<FlexrayNmClusterCoupling, AutosarAbstractionError> {
        let nmcluster_couplings = self
            .element()
            .get_or_create_sub_element(ElementName::NmClusterCouplings)?;
        FlexrayNmClusterCoupling::new(&nmcluster_couplings, nm_schedule_variant)
    }

    /// create a new `UdpNmClusterCoupling`
    pub fn create_udp_nm_cluster_coupling(&self) -> Result<UdpNmClusterCoupling, AutosarAbstractionError> {
        let nmcluster_couplings = self
            .element()
            .get_or_create_sub_element(ElementName::NmClusterCouplings)?;
        UdpNmClusterCoupling::new(&nmcluster_couplings)
    }

    /// iterate over all `NmClusterCouplings`
    pub fn nm_cluster_couplings(&self) -> impl Iterator<Item = NmClusterCoupling> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::NmClusterCouplings)
            .into_iter()
            .flat_map(|couplings| couplings.sub_elements())
            .filter_map(|elem| elem.try_into().ok())
    }

    /// create a new `NmEcu`
    pub fn create_nm_ecu(&self, name: &str, ecu_instance: &EcuInstance) -> Result<NmEcu, AutosarAbstractionError> {
        let nm_ecus = self.element().get_or_create_sub_element(ElementName::NmIfEcus)?;
        NmEcu::new(name, &nm_ecus, ecu_instance)
    }

    /// iterate over all `NmEcus`
    pub fn nm_ecus(&self) -> impl Iterator<Item = NmEcu> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::NmIfEcus)
            .into_iter()
            .flat_map(|ecus| ecus.sub_elements())
            .filter_map(|elem| elem.try_into().ok())
    }
}

//##################################################################

/// An NM cluster is a set of NM nodes coordinated by the NM algorithm.
/// The `AbstractNmCluster` is a common interface for all bus specific NM clusters and provides common functionality.
pub trait AbstractNmCluster: AbstractionElement {
    /// type of the communication cluster on which this NM cluster is based
    type CommunicationClusterType: AbstractCluster;
    /// type of the NM node in this cluster, e.g. `CanNmNode` for a `CanNmCluster`
    type NmNodeType: AbstractNmNode;

    /// set the referenced `CommunicationCluster`
    fn set_communication_cluster(
        &self,
        cluster: &Self::CommunicationClusterType,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .create_sub_element(ElementName::CommunicationClusterRef)?
            .set_reference_target(cluster.element())?;
        Ok(())
    }

    /// get the referenced `CommunicationCluster`
    fn communication_cluster(&self) -> Option<Self::CommunicationClusterType> {
        self.element()
            .get_sub_element(ElementName::CommunicationClusterRef)
            .and_then(|ccref| ccref.get_reference_target().ok())
            .and_then(|elem| elem.try_into().ok())
    }

    // Note: can't add NmNodes in the trait, because different types of NmNodes need different parameters

    /// iterate over all `NmNodes` in this cluster
    fn nm_nodes(&self) -> impl Iterator<Item = Self::NmNodeType> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::NmNodes)
            .into_iter()
            .flat_map(|nodes| nodes.sub_elements())
            .filter_map(|elem| elem.try_into().ok())
    }

    /// set or remove the nmChannelSleepMaster flag
    fn set_channel_sleep_master(&self, value: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(value) = value {
            self.element()
                .create_sub_element(ElementName::NmChannelSleepMaster)?
                .set_character_data(value)?;
        } else {
            self.element()
                .remove_sub_element_kind(ElementName::NmChannelSleepMaster)?;
        }
        Ok(())
    }

    /// get the nmChannelSleepMaster flag
    fn channel_sleep_master(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::NmChannelSleepMaster)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_bool())
    }

    /// set the nmNodeDetectionEnabled flag
    fn set_node_detection_enabled(&self, value: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(value) = value {
            self.element()
                .create_sub_element(ElementName::NmNodeDetectionEnabled)?
                .set_character_data(value)?;
        } else {
            self.element()
                .remove_sub_element_kind(ElementName::NmNodeDetectionEnabled)?;
        }
        Ok(())
    }

    /// get the nmNodeDetectionEnabled flag
    fn node_detection_enabled(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::NmNodeDetectionEnabled)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_bool())
    }

    /// set the nmNodeIdEnabled flag
    fn set_node_id_enabled(&self, value: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(value) = value {
            self.element()
                .create_sub_element(ElementName::NmNodeIdEnabled)?
                .set_character_data(value)?;
        } else {
            self.element().remove_sub_element_kind(ElementName::NmNodeIdEnabled)?;
        }
        Ok(())
    }

    /// get the nmNodeIdEnabled flag
    fn node_id_enabled(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::NmNodeIdEnabled)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_bool())
    }

    /// set the nmPncParticipation flag
    fn set_pnc_participation(&self, value: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(value) = value {
            self.element()
                .create_sub_element(ElementName::NmPncParticipation)?
                .set_character_data(value)?;
        } else {
            self.element()
                .remove_sub_element_kind(ElementName::NmPncParticipation)?;
        }
        Ok(())
    }

    /// get the nmPncParticipation flag
    fn pnc_participation(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::NmPncParticipation)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_bool())
    }

    /// set the nmRepeatMsgIndEnabled flag
    fn set_repeat_msg_ind_enabled(&self, value: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(value) = value {
            self.element()
                .create_sub_element(ElementName::NmRepeatMsgIndEnabled)?
                .set_character_data(value)?;
        } else {
            self.element()
                .remove_sub_element_kind(ElementName::NmRepeatMsgIndEnabled)?;
        }
        Ok(())
    }

    /// get the nmRepeatMsgIndEnabled flag
    fn repeat_msg_ind_enabled(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::NmRepeatMsgIndEnabled)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_bool())
    }

    /// set the nmSynchronizingNetwork flag
    fn set_synchronizing_network(&self, value: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(value) = value {
            self.element()
                .create_sub_element(ElementName::NmSynchronizingNetwork)?
                .set_character_data(value)?;
        } else {
            self.element()
                .remove_sub_element_kind(ElementName::NmSynchronizingNetwork)?;
        }
        Ok(())
    }

    /// get the nmSynchronizingNetwork flag
    fn synchronizing_network(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::NmSynchronizingNetwork)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_bool())
    }

    /// set the pncClusterVectorLength
    fn set_pnc_cluster_vector_length(&self, value: Option<u8>) -> Result<(), AutosarAbstractionError> {
        if let Some(value) = value {
            self.element()
                .create_sub_element(ElementName::PncClusterVectorLength)?
                .set_character_data(u64::from(value))?;
        } else {
            self.element()
                .remove_sub_element_kind(ElementName::PncClusterVectorLength)?;
        }
        Ok(())
    }

    /// get the pncClusterVectorLength
    fn pnc_cluster_vector_length(&self) -> Option<u8> {
        self.element()
            .get_sub_element(ElementName::PncClusterVectorLength)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }
}

//##################################################################

/// The `NmCluster` encapsulates the bus specific NM clusters.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NmCluster {
    /// the NM cluster is a `CanNmCluster`
    CanNm(CanNmCluster),
    /// the NM cluster is a `FlexrayNmCluster`
    FlexrayNm(FlexrayNmCluster),
    /// the NM cluster is a `UdpNmCluster`
    UdpNm(UdpNmCluster),
}

impl TryFrom<Element> for NmCluster {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::CanNmCluster => CanNmCluster::try_from(element).map(NmCluster::CanNm),
            ElementName::FlexrayNmCluster => FlexrayNmCluster::try_from(element).map(NmCluster::FlexrayNm),
            ElementName::UdpNmCluster => UdpNmCluster::try_from(element).map(NmCluster::UdpNm),
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "NmCluster".to_string(),
            }),
        }
    }
}

impl AbstractionElement for NmCluster {
    fn element(&self) -> &Element {
        match self {
            NmCluster::CanNm(cluster) => cluster.element(),
            NmCluster::FlexrayNm(cluster) => cluster.element(),
            NmCluster::UdpNm(cluster) => cluster.element(),
        }
    }
}

impl IdentifiableAbstractionElement for NmCluster {}

//##################################################################

/// The `NmClusterCoupling` is used to couple two `NmClusters` together.
///
/// `AbstractNmClusterCoupling` is a common interface for all bus specific
/// NM cluster couplings and provides common functionality.
pub trait AbstractNmClusterCoupling: AbstractionElement {
    /// type of the coupled `NmCluster`s
    type NmClusterType: AbstractNmCluster;

    /// add a reference to a coupled `NmCluster`
    fn add_coupled_cluster(&self, cluster: &Self::NmClusterType) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::CoupledClusterRefs)?
            .create_sub_element(ElementName::CoupledClusterRef)?
            .set_reference_target(cluster.element())?;
        Ok(())
    }

    /// iterate over all coupled `NmClusters`
    fn coupled_clusters(&self) -> impl Iterator<Item = Self::NmClusterType> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::CoupledClusterRefs)
            .into_iter()
            .flat_map(|clusters| clusters.sub_elements())
            .filter_map(|refelem| {
                refelem
                    .get_reference_target()
                    .ok()
                    .and_then(|elem| elem.try_into().ok())
            })
    }
}

//##################################################################

/// Wrapper for the different types of `NmClusterCoupling`; this type is returned by the iterator over `NmClusterCouplings`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NmClusterCoupling {
    /// the coupling is a `CanNmClusterCoupling`
    CanNmClusterCoupling(CanNmClusterCoupling),
    /// the coupling is a `FlexrayNmClusterCoupling`
    FlexrayNmClusterCoupling(FlexrayNmClusterCoupling),
    /// the coupling is a `UdpNmClusterCoupling`
    UdpNmClusterCoupling(UdpNmClusterCoupling),
}

impl TryFrom<Element> for NmClusterCoupling {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::CanNmClusterCoupling => {
                CanNmClusterCoupling::try_from(element).map(NmClusterCoupling::CanNmClusterCoupling)
            }
            ElementName::FlexrayNmClusterCoupling => {
                FlexrayNmClusterCoupling::try_from(element).map(NmClusterCoupling::FlexrayNmClusterCoupling)
            }
            ElementName::UdpNmClusterCoupling => {
                UdpNmClusterCoupling::try_from(element).map(NmClusterCoupling::UdpNmClusterCoupling)
            }
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "NmClusterCoupling".to_string(),
            }),
        }
    }
}

impl AbstractionElement for NmClusterCoupling {
    fn element(&self) -> &Element {
        match self {
            NmClusterCoupling::CanNmClusterCoupling(coupling) => coupling.element(),
            NmClusterCoupling::FlexrayNmClusterCoupling(coupling) => coupling.element(),
            NmClusterCoupling::UdpNmClusterCoupling(coupling) => coupling.element(),
        }
    }
}

impl IdentifiableAbstractionElement for NmClusterCoupling {}

//##################################################################

/// The `NmEcu` represents an `EcuInstance` wich participates in network management.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NmEcu(Element);
abstraction_element!(NmEcu, NmEcu);
impl IdentifiableAbstractionElement for NmEcu {}

impl NmEcu {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        ecu_instance: &EcuInstance,
    ) -> Result<Self, AutosarAbstractionError> {
        let nm_ecu = parent.create_named_sub_element(ElementName::NmEcu, name)?;
        nm_ecu
            .create_sub_element(ElementName::EcuInstanceRef)?
            .set_reference_target(ecu_instance.element())?;
        Ok(Self(nm_ecu))
    }

    /// get the referenced `EcuInstance`
    #[must_use]
    pub fn ecu_instance(&self) -> Option<EcuInstance> {
        self.element()
            .get_sub_element(ElementName::EcuInstanceRef)
            .and_then(|eiref| eiref.get_reference_target().ok())
            .and_then(|elem| elem.try_into().ok())
    }

    /// set the nmBusSynchronizationEnabled flag
    ///
    /// This flag is optional; if it is set to `Some()` the value is created, if it is set to None the value is removed.
    pub fn set_nm_bus_synchronization_enabled(&self, value: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(value) = value {
            self.element()
                .create_sub_element(ElementName::NmBusSynchronizationEnabled)?
                .set_character_data(value)?;
        } else {
            self.element()
                .remove_sub_element_kind(ElementName::NmBusSynchronizationEnabled)?;
        }
        Ok(())
    }

    /// get the nmBusSynchronizationEnabled flag
    #[must_use]
    pub fn nm_bus_synchronization_enabled(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::NmBusSynchronizationEnabled)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_bool())
    }

    /// set the nmComControlEnabled flag
    ///
    /// This flag is optional; if it is set to `Some()` the value is created, if it is set to None the value is removed.
    pub fn set_nm_com_control_enabled(&self, value: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(value) = value {
            self.element()
                .create_sub_element(ElementName::NmComControlEnabled)?
                .set_character_data(value)?;
        } else {
            self.element()
                .remove_sub_element_kind(ElementName::NmComControlEnabled)?;
        }
        Ok(())
    }

    /// get the nmComControlEnabled flag
    #[must_use]
    pub fn nm_com_control_enabled(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::NmComControlEnabled)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_bool())
    }

    /// set or remove the nmCycletimeMainFunction value
    ///
    /// This value is optional; if it is set to Some(x) the value is created, if it is set to None the value is removed.
    pub fn set_cycle_time_main_function(&self, value: Option<f64>) -> Result<(), AutosarAbstractionError> {
        if let Some(value) = value {
            self.element()
                .create_sub_element(ElementName::NmCycletimeMainFunction)?
                .set_character_data(value)?;
        } else {
            self.element()
                .remove_sub_element_kind(ElementName::NmCycletimeMainFunction)?;
        }
        Ok(())
    }

    /// get the nmCycletimeMainFunction value
    #[must_use]
    pub fn cycle_time_main_function(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::NmCycletimeMainFunction)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_float())
    }
}

//##################################################################

/// `AbstractNmNode` is a common interface for all bus specific NM nodes and provides common functionality.
///
/// The `NmNode` represents a node in the network management.
/// Each `NmNode` is connected to a `CommunicationController` and an `NmEcu`.
pub trait AbstractNmNode: AbstractionElement {
    /// type of the communication controller connected to this node
    type CommunicationControllerType: AbstractCommunicationController;

    /// set the referenced `CommunicationController`
    fn set_communication_controller(
        &self,
        controller: &Self::CommunicationControllerType,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .create_sub_element(ElementName::ControllerRef)?
            .set_reference_target(controller.element())?;
        Ok(())
    }

    /// get the referenced `CommunicationController`
    fn communication_controller(&self) -> Option<Self::CommunicationControllerType> {
        self.element()
            .get_sub_element(ElementName::ControllerRef)
            .and_then(|ccref| ccref.get_reference_target().ok())
            .and_then(|elem| elem.try_into().ok())
    }

    /// set the referenced `NmEcu`
    fn set_nm_ecu(&self, ecu: &NmEcu) -> Result<(), AutosarAbstractionError> {
        self.element()
            .create_sub_element(ElementName::NmIfEcuRef)?
            .set_reference_target(ecu.element())?;
        Ok(())
    }

    /// get the referenced `NmEcu`
    fn nm_ecu(&self) -> Option<NmEcu> {
        self.element()
            .get_sub_element(ElementName::NmIfEcuRef)
            .and_then(|eiref| eiref.get_reference_target().ok())
            .and_then(|elem| elem.try_into().ok())
    }

    /// set the nmNodeId
    /// This value is optional; if it is set to Some(x) the value is created, if it is set to None the value is removed.
    fn set_node_id(&self, value: Option<u32>) -> Result<(), AutosarAbstractionError> {
        if let Some(value) = value {
            self.element()
                .create_sub_element(ElementName::NmNodeId)?
                .set_character_data(u64::from(value))?;
        } else {
            self.element().remove_sub_element_kind(ElementName::NmNodeId)?;
        }
        Ok(())
    }

    /// get the nmNodeId
    fn node_id(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::NmNodeId)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set ot remove the nmPassiveModeEnabled flag
    ///
    /// This flag is optional; if it is set to Some(x) the value is created, if it is set to None the value is removed.
    fn set_passive_mode(&self, value: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(value) = value {
            self.element()
                .create_sub_element(ElementName::NmPassiveModeEnabled)?
                .set_character_data(value)?;
        } else {
            self.element()
                .remove_sub_element_kind(ElementName::NmPassiveModeEnabled)?;
        }
        Ok(())
    }

    /// get the nmPassiveModeEnabled flag
    fn passive_mode(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::NmPassiveModeEnabled)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_bool())
    }

    /// add an Rx `NmPdu`
    ///
    /// Every `NmNode` must have at least one Rx `NmPdu`
    fn add_rx_nm_pdu(&self, nm_pdu: &NmPdu) -> Result<(), AutosarAbstractionError> {
        let rx_pdus = self.element().get_or_create_sub_element(ElementName::RxNmPduRefs)?;
        rx_pdus
            .create_sub_element(ElementName::RxNmPduRef)?
            .set_reference_target(nm_pdu.element())?;
        Ok(())
    }

    /// iterate over all RX `NmPdus`
    fn rx_nm_pdus(&self) -> impl Iterator<Item = NmPdu> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::RxNmPduRefs)
            .into_iter()
            .flat_map(|rx_pdus| rx_pdus.sub_elements())
            .filter_map(|refelem| {
                refelem
                    .get_reference_target()
                    .ok()
                    .and_then(|elem| elem.try_into().ok())
            })
    }

    /// add a Tx `NmPdu`
    ///
    /// Active `NmNodes` must have at least one Tx `NmPdu`, while passive `NmNodes` may have none.
    fn add_tx_nm_pdu(&self, nm_pdu: &NmPdu) -> Result<(), AutosarAbstractionError> {
        let tx_pdus = self.element().get_or_create_sub_element(ElementName::TxNmPduRefs)?;
        tx_pdus
            .create_sub_element(ElementName::TxNmPduRef)?
            .set_reference_target(nm_pdu.element())?;
        Ok(())
    }

    /// iterate over all TX `NmPdus`
    fn tx_nm_pdus(&self) -> impl Iterator<Item = NmPdu> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::TxNmPduRefs)
            .into_iter()
            .flat_map(|tx_pdus| tx_pdus.sub_elements())
            .filter_map(|refelem| {
                refelem
                    .get_reference_target()
                    .ok()
                    .and_then(|elem| elem.try_into().ok())
            })
    }
}

//##################################################################

#[cfg(test)]
mod test {
    use crate::communication::*;
    use crate::*;
    use autosar_data::AutosarVersion;

    #[test]
    fn test_can_nm() {
        let model = AutosarModelAbstraction::create("test", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();
        let system = package.create_system("System", SystemCategory::SystemExtract).unwrap();

        let can_cluster = system
            .create_can_cluster("can_cluster", &package, &CanClusterSettings::default())
            .unwrap();
        let can_physical_channel = can_cluster.create_physical_channel("can_channel").unwrap();
        let ecu1 = system.create_ecu_instance("ecu1", &package).unwrap();
        let ecu2 = system.create_ecu_instance("ecu2", &package).unwrap();

        let ecu1_communication_controller = ecu1.create_can_communication_controller("can_controller_1").unwrap();
        let ecu2_communication_controller = ecu2.create_can_communication_controller("can_controller_2").unwrap();

        let _connector1 = ecu1_communication_controller
            .connect_physical_channel("Ecu1_connection", &can_physical_channel)
            .unwrap();
        let _connector2 = ecu2_communication_controller
            .connect_physical_channel("Ecu2_connection", &can_physical_channel)
            .unwrap();

        let nm_pdu1 = system.create_nm_pdu("NmPdu1", &package, 8).unwrap();
        let nm_pdu2 = system.create_nm_pdu("NmPdu2", &package, 8).unwrap();

        //========= NM Config ==========

        let nm_config = system.create_nm_config("NmConfig", &package).unwrap();

        // ------ CAN NM cluster ------
        let can_nm_cluster_settings = CanNmClusterSettings {
            nm_busload_reduction_active: false,
            nm_immediate_nm_transmissions: 22,
            nm_message_timeout_time: 4.5,
            nm_msg_cycle_time: 1.0,
            nm_network_timeout: 9.0,
            nm_remote_sleep_indication_time: 2.0,
            nm_repeat_message_time: 2.0,
            nm_wait_bus_sleep_time: 2.0,
        };
        let can_nm_cluster = nm_config
            .create_can_nm_cluster("can_nm_cluster", &can_nm_cluster_settings, &can_cluster)
            .unwrap();
        assert_eq!(nm_config.nm_clusters().count(), 1);
        let generic_cluster = nm_config.nm_clusters().next().unwrap();
        assert_eq!(generic_cluster.element(), can_nm_cluster.element());
        assert_eq!(can_nm_cluster.communication_cluster(), Some(can_cluster));
        // verify settings
        assert_eq!(can_nm_cluster.nm_busload_reduction_active(), Some(false));
        assert_eq!(can_nm_cluster.nm_immediate_nm_transmissions(), Some(22));
        assert_eq!(can_nm_cluster.nm_message_timeout_time(), Some(4.5));
        assert_eq!(can_nm_cluster.nm_msg_cycle_time(), Some(1.0));
        assert_eq!(can_nm_cluster.nm_network_timeout(), Some(9.0));
        assert_eq!(can_nm_cluster.nm_remote_sleep_indication_time(), Some(2.0));
        assert_eq!(can_nm_cluster.nm_repeat_message_time(), Some(2.0));
        assert_eq!(can_nm_cluster.nm_wait_bus_sleep_time(), Some(2.0));
        // test additional CanNmCluster properties
        can_nm_cluster.set_channel_sleep_master(Some(true)).unwrap();
        assert_eq!(can_nm_cluster.channel_sleep_master(), Some(true));
        can_nm_cluster.set_node_detection_enabled(Some(false)).unwrap();
        assert_eq!(can_nm_cluster.node_detection_enabled(), Some(false));
        can_nm_cluster.set_node_id_enabled(Some(true)).unwrap();
        assert_eq!(can_nm_cluster.node_id_enabled(), Some(true));
        can_nm_cluster.set_pnc_participation(Some(true)).unwrap();
        assert_eq!(can_nm_cluster.pnc_participation(), Some(true));
        can_nm_cluster.set_repeat_msg_ind_enabled(Some(true)).unwrap();
        assert_eq!(can_nm_cluster.repeat_msg_ind_enabled(), Some(true));
        can_nm_cluster.set_synchronizing_network(Some(true)).unwrap();
        assert_eq!(can_nm_cluster.synchronizing_network(), Some(true));
        can_nm_cluster.set_pnc_cluster_vector_length(Some(3)).unwrap();
        assert_eq!(can_nm_cluster.pnc_cluster_vector_length(), Some(3));
        // remove optional values
        can_nm_cluster.set_channel_sleep_master(None).unwrap();
        assert_eq!(can_nm_cluster.channel_sleep_master(), None);
        can_nm_cluster.set_node_detection_enabled(None).unwrap();
        assert_eq!(can_nm_cluster.node_detection_enabled(), None);
        can_nm_cluster.set_node_id_enabled(None).unwrap();
        assert_eq!(can_nm_cluster.node_id_enabled(), None);
        can_nm_cluster.set_pnc_participation(None).unwrap();
        assert_eq!(can_nm_cluster.pnc_participation(), None);
        can_nm_cluster.set_repeat_msg_ind_enabled(None).unwrap();
        assert_eq!(can_nm_cluster.repeat_msg_ind_enabled(), None);
        can_nm_cluster.set_synchronizing_network(None).unwrap();
        assert_eq!(can_nm_cluster.synchronizing_network(), None);
        can_nm_cluster.set_pnc_cluster_vector_length(None).unwrap();
        assert_eq!(can_nm_cluster.pnc_cluster_vector_length(), None);

        // ------ CAN NM ecu ------
        let nm_ecu1 = nm_config.create_nm_ecu("nm_ecu1", &ecu1).unwrap();
        let nm_ecu2 = nm_config.create_nm_ecu("nm_ecu2", &ecu2).unwrap();
        assert_eq!(nm_config.nm_ecus().count(), 2);
        assert_eq!(nm_ecu1.ecu_instance(), Some(ecu1));
        assert_eq!(nm_ecu2.ecu_instance(), Some(ecu2));
        nm_ecu1.set_nm_bus_synchronization_enabled(Some(true)).unwrap();
        assert_eq!(nm_ecu1.nm_bus_synchronization_enabled(), Some(true));
        nm_ecu1.set_nm_com_control_enabled(Some(true)).unwrap();
        assert_eq!(nm_ecu1.nm_com_control_enabled(), Some(true));
        nm_ecu1.set_cycle_time_main_function(Some(0.1)).unwrap();
        assert_eq!(nm_ecu1.cycle_time_main_function(), Some(0.1));
        // remove optional values
        nm_ecu1.set_nm_bus_synchronization_enabled(None).unwrap();
        assert_eq!(nm_ecu1.nm_bus_synchronization_enabled(), None);
        nm_ecu1.set_nm_com_control_enabled(None).unwrap();
        assert_eq!(nm_ecu1.nm_com_control_enabled(), None);
        nm_ecu1.set_cycle_time_main_function(None).unwrap();
        assert_eq!(nm_ecu1.cycle_time_main_function(), None);

        // ------ CAN NM node ------
        let nm_node1 = can_nm_cluster
            .create_can_nm_node("can_nm_node1", &ecu1_communication_controller, &nm_ecu1)
            .unwrap();
        assert_eq!(nm_node1.communication_controller(), Some(ecu1_communication_controller));
        assert_eq!(nm_node1.nm_ecu(), Some(nm_ecu1));
        nm_node1.set_node_id(Some(1)).unwrap();
        assert_eq!(nm_node1.node_id(), Some(1));
        nm_node1.set_passive_mode(Some(false)).unwrap();
        assert_eq!(nm_node1.passive_mode(), Some(false));

        let nm_node2 = can_nm_cluster
            .create_can_nm_node("can_nm_node2", &ecu2_communication_controller, &nm_ecu2)
            .unwrap();
        assert_eq!(can_nm_cluster.nm_nodes().count(), 2);
        assert_eq!(nm_node2.communication_controller(), Some(ecu2_communication_controller));

        nm_node1.add_rx_nm_pdu(&nm_pdu1).unwrap();
        nm_node1.add_tx_nm_pdu(&nm_pdu2).unwrap();
        assert_eq!(nm_node1.rx_nm_pdus().count(), 1);
        assert_eq!(nm_node1.tx_nm_pdus().count(), 1);
        nm_node2.add_rx_nm_pdu(&nm_pdu2).unwrap();
        nm_node2.add_tx_nm_pdu(&nm_pdu1).unwrap();
        assert_eq!(nm_node2.rx_nm_pdus().count(), 1);
        assert_eq!(nm_node2.tx_nm_pdus().count(), 1);

        assert_eq!(can_nm_cluster.nm_nodes().next().unwrap(), nm_node1);

        // remove optional values
        nm_node1.set_node_id(None).unwrap();
        assert_eq!(nm_node1.node_id(), None);
        nm_node1.set_passive_mode(None).unwrap();
        assert_eq!(nm_node1.passive_mode(), None);

        // ------ CAN NM Cluster Coupling ------
        let cluster_coupling = nm_config.create_can_nm_cluster_coupling(true, true).unwrap();
        assert_eq!(nm_config.nm_cluster_couplings().count(), 1);
        assert_eq!(
            nm_config.nm_cluster_couplings().next().unwrap().element(),
            cluster_coupling.element()
        );
        assert_eq!(cluster_coupling.nm_busload_reduction_enabled(), Some(true));
        assert_eq!(cluster_coupling.nm_immediate_restart_enabled(), Some(true));
        cluster_coupling.add_coupled_cluster(&can_nm_cluster).unwrap();
        assert_eq!(cluster_coupling.coupled_clusters().count(), 1);

        // conversions
        let can_nm_cluster2 = CanNmCluster::try_from(can_nm_cluster.element().clone()).unwrap();
        assert_eq!(can_nm_cluster2, can_nm_cluster);
        let nm_cluster2 = NmCluster::try_from(can_nm_cluster.element().clone()).unwrap();
        assert_eq!(nm_cluster2, NmCluster::CanNm(can_nm_cluster));
        let result = NmCluster::try_from(model.root_element());
        assert!(result.is_err());
        let cluster_coupling2 = CanNmClusterCoupling::try_from(cluster_coupling.element().clone()).unwrap();
        assert_eq!(cluster_coupling2, cluster_coupling);
        let coupling2 = NmClusterCoupling::try_from(cluster_coupling.element().clone()).unwrap();
        assert_eq!(coupling2, NmClusterCoupling::CanNmClusterCoupling(cluster_coupling));
        let result = NmClusterCoupling::try_from(model.root_element());
        assert!(result.is_err());
    }

    #[test]
    fn test_flexray_nm() {
        let model = AutosarModelAbstraction::create("test", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();
        let system = package.create_system("System", SystemCategory::SystemExtract).unwrap();

        let flexray_cluster = system
            .create_flexray_cluster("flexray_cluster", &package, &FlexrayClusterSettings::default())
            .unwrap();
        let flexray_physical_channel = flexray_cluster
            .create_physical_channel("flexray_channel", FlexrayChannelName::A)
            .unwrap();
        let ecu1 = system.create_ecu_instance("ecu1", &package).unwrap();
        let ecu2 = system.create_ecu_instance("ecu2", &package).unwrap();

        let ecu1_communication_controller = ecu1
            .create_flexray_communication_controller("flexray_controller_1")
            .unwrap();
        let ecu2_communication_controller = ecu2
            .create_flexray_communication_controller("flexray_controller_2")
            .unwrap();

        let _connector1 = ecu1_communication_controller
            .connect_physical_channel("Ecu1_connection", &flexray_physical_channel)
            .unwrap();
        let _connector2 = ecu2_communication_controller
            .connect_physical_channel("Ecu2_connection", &flexray_physical_channel)
            .unwrap();

        let nm_pdu1 = system.create_nm_pdu("NmPdu1", &package, 8).unwrap();
        let nm_pdu2 = system.create_nm_pdu("NmPdu2", &package, 8).unwrap();

        //========= NM Config ==========

        let nm_config = system.create_nm_config("NmConfig", &package).unwrap();

        // ------ Flexray NM cluster ------
        let flexray_nm_cluster_settings = FlexrayNmClusterSettings {
            nm_data_cycle: 1,
            nm_remote_sleep_indication_time: 2.0,
            nm_repeat_message_time: 3.0,
            nm_repetition_cycle: 4,
            nm_voting_cycle: 5,
        };
        let flexray_nm_cluster = nm_config
            .create_flexray_nm_cluster("flexray_nm_cluster", &flexray_nm_cluster_settings, &flexray_cluster)
            .unwrap();
        assert_eq!(nm_config.nm_clusters().count(), 1);
        let generic_cluster = nm_config.nm_clusters().next().unwrap();
        assert_eq!(generic_cluster.element(), flexray_nm_cluster.element());
        assert_eq!(flexray_nm_cluster.communication_cluster(), Some(flexray_cluster));
        // verify settings
        assert_eq!(flexray_nm_cluster.nm_data_cycle(), Some(1));
        assert_eq!(flexray_nm_cluster.nm_remote_sleep_indication_time(), Some(2.0));
        assert_eq!(flexray_nm_cluster.nm_repeat_message_time(), Some(3.0));
        assert_eq!(flexray_nm_cluster.nm_repetition_cycle(), Some(4));
        assert_eq!(flexray_nm_cluster.nm_voting_cycle(), Some(5));
        // test additional FlexrayNmCluster properties
        flexray_nm_cluster.set_channel_sleep_master(Some(true)).unwrap();
        assert_eq!(flexray_nm_cluster.channel_sleep_master(), Some(true));
        flexray_nm_cluster.set_node_detection_enabled(Some(false)).unwrap();
        assert_eq!(flexray_nm_cluster.node_detection_enabled(), Some(false));
        flexray_nm_cluster.set_node_id_enabled(Some(true)).unwrap();
        assert_eq!(flexray_nm_cluster.node_id_enabled(), Some(true));
        flexray_nm_cluster.set_pnc_participation(Some(true)).unwrap();
        assert_eq!(flexray_nm_cluster.pnc_participation(), Some(true));
        flexray_nm_cluster.set_repeat_msg_ind_enabled(Some(true)).unwrap();
        assert_eq!(flexray_nm_cluster.repeat_msg_ind_enabled(), Some(true));
        flexray_nm_cluster.set_synchronizing_network(Some(true)).unwrap();
        assert_eq!(flexray_nm_cluster.synchronizing_network(), Some(true));
        flexray_nm_cluster.set_pnc_cluster_vector_length(Some(3)).unwrap();
        assert_eq!(flexray_nm_cluster.pnc_cluster_vector_length(), Some(3));
        // remove optional values
        flexray_nm_cluster.set_channel_sleep_master(None).unwrap();
        assert_eq!(flexray_nm_cluster.channel_sleep_master(), None);
        flexray_nm_cluster.set_node_detection_enabled(None).unwrap();
        assert_eq!(flexray_nm_cluster.node_detection_enabled(), None);
        flexray_nm_cluster.set_node_id_enabled(None).unwrap();
        assert_eq!(flexray_nm_cluster.node_id_enabled(), None);
        flexray_nm_cluster.set_pnc_participation(None).unwrap();
        assert_eq!(flexray_nm_cluster.pnc_participation(), None);
        flexray_nm_cluster.set_repeat_msg_ind_enabled(None).unwrap();
        assert_eq!(flexray_nm_cluster.repeat_msg_ind_enabled(), None);
        flexray_nm_cluster.set_synchronizing_network(None).unwrap();
        assert_eq!(flexray_nm_cluster.synchronizing_network(), None);
        flexray_nm_cluster.set_pnc_cluster_vector_length(None).unwrap();
        assert_eq!(flexray_nm_cluster.pnc_cluster_vector_length(), None);

        // ------ Flexray NM ecu ------
        let nm_ecu1 = nm_config.create_nm_ecu("nm_ecu1", &ecu1).unwrap();
        let nm_ecu2 = nm_config.create_nm_ecu("nm_ecu2", &ecu2).unwrap();
        assert_eq!(nm_config.nm_ecus().count(), 2);
        assert_eq!(nm_ecu1.ecu_instance(), Some(ecu1));
        assert_eq!(nm_ecu2.ecu_instance(), Some(ecu2));
        nm_ecu1.set_nm_bus_synchronization_enabled(Some(true)).unwrap();
        assert_eq!(nm_ecu1.nm_bus_synchronization_enabled(), Some(true));
        nm_ecu1.set_nm_com_control_enabled(Some(true)).unwrap();
        assert_eq!(nm_ecu1.nm_com_control_enabled(), Some(true));
        nm_ecu1.set_cycle_time_main_function(Some(0.1)).unwrap();
        assert_eq!(nm_ecu1.cycle_time_main_function(), Some(0.1));

        // ------ Flexray NM node ------
        let nm_node1 = flexray_nm_cluster
            .create_flexray_nm_node("flexray_nm_node1", &ecu1_communication_controller, &nm_ecu1)
            .unwrap();
        assert_eq!(nm_node1.communication_controller(), Some(ecu1_communication_controller));
        assert_eq!(nm_node1.nm_ecu(), Some(nm_ecu1));
        nm_node1.set_node_id(Some(1)).unwrap();
        assert_eq!(nm_node1.node_id(), Some(1));
        nm_node1.set_passive_mode(Some(false)).unwrap();
        assert_eq!(nm_node1.passive_mode(), Some(false));

        let nm_node2 = flexray_nm_cluster
            .create_flexray_nm_node("flexray_nm_node2", &ecu2_communication_controller, &nm_ecu2)
            .unwrap();
        assert_eq!(flexray_nm_cluster.nm_nodes().count(), 2);
        assert_eq!(nm_node2.communication_controller(), Some(ecu2_communication_controller));

        nm_node1.add_rx_nm_pdu(&nm_pdu1).unwrap();
        nm_node1.add_tx_nm_pdu(&nm_pdu2).unwrap();
        assert_eq!(nm_node1.rx_nm_pdus().count(), 1);
        assert_eq!(nm_node1.tx_nm_pdus().count(), 1);
        nm_node2.add_rx_nm_pdu(&nm_pdu2).unwrap();
        nm_node2.add_tx_nm_pdu(&nm_pdu1).unwrap();
        assert_eq!(nm_node2.rx_nm_pdus().count(), 1);
        assert_eq!(nm_node2.tx_nm_pdus().count(), 1);

        assert_eq!(flexray_nm_cluster.nm_nodes().next().unwrap(), nm_node1);

        // remove optional values
        nm_node1.set_node_id(None).unwrap();
        assert_eq!(nm_node1.node_id(), None);
        nm_node1.set_passive_mode(None).unwrap();
        assert_eq!(nm_node1.passive_mode(), None);

        // ------ Flexray NM Cluster Coupling ------
        let cluster_coupling = nm_config
            .create_flexray_nm_cluster_coupling(FlexrayNmScheduleVariant::ScheduleVariant6)
            .unwrap();
        assert_eq!(nm_config.nm_cluster_couplings().count(), 1);
        assert_eq!(
            cluster_coupling.nm_schedule_variant(),
            Some(FlexrayNmScheduleVariant::ScheduleVariant6)
        );
        cluster_coupling.add_coupled_cluster(&flexray_nm_cluster).unwrap();
        assert_eq!(cluster_coupling.coupled_clusters().count(), 1);

        // ------ Flexray Schedule Variant ------
        assert_eq!(
            EnumItem::from(FlexrayNmScheduleVariant::ScheduleVariant1),
            EnumItem::ScheduleVariant1
        );
        assert_eq!(
            EnumItem::from(FlexrayNmScheduleVariant::ScheduleVariant2),
            EnumItem::ScheduleVariant2
        );
        assert_eq!(
            EnumItem::from(FlexrayNmScheduleVariant::ScheduleVariant3),
            EnumItem::ScheduleVariant3
        );
        assert_eq!(
            EnumItem::from(FlexrayNmScheduleVariant::ScheduleVariant4),
            EnumItem::ScheduleVariant4
        );
        assert_eq!(
            EnumItem::from(FlexrayNmScheduleVariant::ScheduleVariant5),
            EnumItem::ScheduleVariant5
        );
        assert_eq!(
            EnumItem::from(FlexrayNmScheduleVariant::ScheduleVariant6),
            EnumItem::ScheduleVariant6
        );
        assert_eq!(
            EnumItem::from(FlexrayNmScheduleVariant::ScheduleVariant7),
            EnumItem::ScheduleVariant7
        );

        assert_eq!(
            FlexrayNmScheduleVariant::try_from(EnumItem::ScheduleVariant1).unwrap(),
            FlexrayNmScheduleVariant::ScheduleVariant1
        );
        assert_eq!(
            FlexrayNmScheduleVariant::try_from(EnumItem::ScheduleVariant2).unwrap(),
            FlexrayNmScheduleVariant::ScheduleVariant2
        );
        assert_eq!(
            FlexrayNmScheduleVariant::try_from(EnumItem::ScheduleVariant3).unwrap(),
            FlexrayNmScheduleVariant::ScheduleVariant3
        );
        assert_eq!(
            FlexrayNmScheduleVariant::try_from(EnumItem::ScheduleVariant4).unwrap(),
            FlexrayNmScheduleVariant::ScheduleVariant4
        );
        assert_eq!(
            FlexrayNmScheduleVariant::try_from(EnumItem::ScheduleVariant5).unwrap(),
            FlexrayNmScheduleVariant::ScheduleVariant5
        );
        assert_eq!(
            FlexrayNmScheduleVariant::try_from(EnumItem::ScheduleVariant6).unwrap(),
            FlexrayNmScheduleVariant::ScheduleVariant6
        );
        assert_eq!(
            FlexrayNmScheduleVariant::try_from(EnumItem::ScheduleVariant7).unwrap(),
            FlexrayNmScheduleVariant::ScheduleVariant7
        );
        assert!(FlexrayNmScheduleVariant::try_from(EnumItem::Aa).is_err());

        // Cluster Coupling conversions
        let cluster_coupling2 = nm_config.nm_cluster_couplings().next().unwrap();
        assert_eq!(cluster_coupling.element(), cluster_coupling2.element());
    }

    #[test]
    fn test_udp_nm() {
        let model = AutosarModelAbstraction::create("test", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();
        let system = package.create_system("System", SystemCategory::SystemExtract).unwrap();

        let ethernet_cluster = system.create_ethernet_cluster("ethernet_cluster", &package).unwrap();
        let ethernet_physical_channel = ethernet_cluster
            .create_physical_channel("ethernet_channel", None)
            .unwrap();
        let ecu1 = system.create_ecu_instance("ecu1", &package).unwrap();
        let ecu2 = system.create_ecu_instance("ecu2", &package).unwrap();

        let ecu1_communication_controller = ecu1
            .create_ethernet_communication_controller("udp_controller_1", None)
            .unwrap();
        let ecu2_communication_controller = ecu2
            .create_ethernet_communication_controller("udp_controller_2", None)
            .unwrap();

        let _connector1 = ecu1_communication_controller
            .connect_physical_channel("Ecu1_connection", &ethernet_physical_channel)
            .unwrap();
        let _connector2 = ecu2_communication_controller
            .connect_physical_channel("Ecu2_connection", &ethernet_physical_channel)
            .unwrap();

        let nm_pdu1 = system.create_nm_pdu("NmPdu1", &package, 8).unwrap();
        let nm_pdu2 = system.create_nm_pdu("NmPdu2", &package, 8).unwrap();

        //========= NM Config ==========

        let nm_config = system.create_nm_config("NmConfig", &package).unwrap();

        // ------ UDP NM cluster ------
        let udp_nm_cluster_settings = UdpNmClusterSettings {
            nm_msg_cycle_time: 1.0,
            nm_msg_timeout_time: 2.0,
            nm_network_timeout: 3.0,
            nm_remote_sleep_indication_time: 4.0,
            nm_repeat_message_time: 5.0,
            nm_wait_bus_sleep_time: 6.0,
        };
        let udp_nm_cluster = nm_config
            .create_udp_nm_cluster("udp_nm_cluster", &udp_nm_cluster_settings, &ethernet_cluster)
            .unwrap();
        assert_eq!(nm_config.nm_clusters().count(), 1);
        let generic_cluster = nm_config.nm_clusters().next().unwrap();
        assert_eq!(generic_cluster.element(), udp_nm_cluster.element());
        assert_eq!(udp_nm_cluster.communication_cluster(), Some(ethernet_cluster));
        // verify settings
        assert_eq!(udp_nm_cluster.nm_msg_cycle_time(), Some(1.0));
        assert_eq!(udp_nm_cluster.nm_message_timeout_time(), Some(2.0));
        assert_eq!(udp_nm_cluster.nm_network_timeout(), Some(3.0));
        assert_eq!(udp_nm_cluster.nm_remote_sleep_indication_time(), Some(4.0));
        assert_eq!(udp_nm_cluster.nm_repeat_message_time(), Some(5.0));
        assert_eq!(udp_nm_cluster.nm_wait_bus_sleep_time(), Some(6.0));
        // test additional UdpNmCluster properties
        udp_nm_cluster.set_channel_sleep_master(Some(true)).unwrap();
        assert_eq!(udp_nm_cluster.channel_sleep_master(), Some(true));
        udp_nm_cluster.set_node_detection_enabled(Some(false)).unwrap();
        assert_eq!(udp_nm_cluster.node_detection_enabled(), Some(false));
        udp_nm_cluster.set_nm_cbv_position(Some(33)).unwrap();
        assert_eq!(udp_nm_cluster.nm_cbv_position(), Some(33));
        udp_nm_cluster.set_nm_immediate_nm_transmissions(Some(11)).unwrap();
        assert_eq!(udp_nm_cluster.nm_immediate_nm_transmissions(), Some(11));
        udp_nm_cluster.set_vlan(Some(&ethernet_physical_channel)).unwrap();
        assert_eq!(udp_nm_cluster.vlan(), Some(ethernet_physical_channel));
        udp_nm_cluster.set_nm_nid_position(Some(3)).unwrap();
        assert_eq!(udp_nm_cluster.nm_nid_position(), Some(3));
        udp_nm_cluster.set_node_id_enabled(Some(true)).unwrap();
        assert_eq!(udp_nm_cluster.node_id_enabled(), Some(true));
        udp_nm_cluster.set_pnc_participation(Some(true)).unwrap();
        assert_eq!(udp_nm_cluster.pnc_participation(), Some(true));
        udp_nm_cluster.set_repeat_msg_ind_enabled(Some(true)).unwrap();
        assert_eq!(udp_nm_cluster.repeat_msg_ind_enabled(), Some(true));
        udp_nm_cluster.set_synchronizing_network(Some(true)).unwrap();
        assert_eq!(udp_nm_cluster.synchronizing_network(), Some(true));
        udp_nm_cluster.set_pnc_cluster_vector_length(Some(3)).unwrap();
        assert_eq!(udp_nm_cluster.pnc_cluster_vector_length(), Some(3));
        // remove optional values
        udp_nm_cluster.set_channel_sleep_master(None).unwrap();
        assert_eq!(udp_nm_cluster.channel_sleep_master(), None);
        udp_nm_cluster.set_node_detection_enabled(None).unwrap();
        assert_eq!(udp_nm_cluster.node_detection_enabled(), None);
        udp_nm_cluster.set_nm_cbv_position(None).unwrap();
        assert_eq!(udp_nm_cluster.nm_cbv_position(), None);
        udp_nm_cluster.set_nm_immediate_nm_transmissions(None).unwrap();
        assert_eq!(udp_nm_cluster.nm_immediate_nm_transmissions(), None);
        udp_nm_cluster.set_vlan(None).unwrap();
        assert_eq!(udp_nm_cluster.vlan(), None);
        udp_nm_cluster.set_nm_nid_position(None).unwrap();
        assert_eq!(udp_nm_cluster.nm_nid_position(), None);
        udp_nm_cluster.set_node_id_enabled(None).unwrap();
        assert_eq!(udp_nm_cluster.node_id_enabled(), None);
        udp_nm_cluster.set_pnc_participation(None).unwrap();
        assert_eq!(udp_nm_cluster.pnc_participation(), None);
        udp_nm_cluster.set_repeat_msg_ind_enabled(None).unwrap();
        assert_eq!(udp_nm_cluster.repeat_msg_ind_enabled(), None);
        udp_nm_cluster.set_synchronizing_network(None).unwrap();
        assert_eq!(udp_nm_cluster.synchronizing_network(), None);
        udp_nm_cluster.set_pnc_cluster_vector_length(None).unwrap();
        assert_eq!(udp_nm_cluster.pnc_cluster_vector_length(), None);

        // ------ UDP NM ecu ------
        let nm_ecu1 = nm_config.create_nm_ecu("nm_ecu1", &ecu1).unwrap();
        let nm_ecu2 = nm_config.create_nm_ecu("nm_ecu2", &ecu2).unwrap();
        assert_eq!(nm_config.nm_ecus().count(), 2);
        assert_eq!(nm_ecu1.ecu_instance(), Some(ecu1));
        assert_eq!(nm_ecu2.ecu_instance(), Some(ecu2));
        nm_ecu1.set_nm_bus_synchronization_enabled(Some(true)).unwrap();
        assert_eq!(nm_ecu1.nm_bus_synchronization_enabled(), Some(true));
        nm_ecu1.set_nm_com_control_enabled(Some(true)).unwrap();
        assert_eq!(nm_ecu1.nm_com_control_enabled(), Some(true));
        nm_ecu1.set_cycle_time_main_function(Some(0.1)).unwrap();
        assert_eq!(nm_ecu1.cycle_time_main_function(), Some(0.1));

        // ------ UDP NM node ------
        let nm_node1 = udp_nm_cluster
            .create_udp_nm_node("udp_nm_node1", &ecu1_communication_controller, &nm_ecu1, 0.1)
            .unwrap();
        assert_eq!(nm_node1.communication_controller(), Some(ecu1_communication_controller));
        assert_eq!(nm_node1.nm_ecu(), Some(nm_ecu1));
        assert_eq!(nm_node1.nm_msg_cycle_offset(), Some(0.1));
        nm_node1.set_node_id(Some(1)).unwrap();
        assert_eq!(nm_node1.node_id(), Some(1));
        nm_node1.set_passive_mode(Some(false)).unwrap();
        assert_eq!(nm_node1.passive_mode(), Some(false));
        nm_node1.set_all_nm_messages_keep_awake(Some(true)).unwrap();
        assert_eq!(nm_node1.all_nm_messages_keep_awake(), Some(true));

        let nm_node2 = udp_nm_cluster
            .create_udp_nm_node("udp_nm_node2", &ecu2_communication_controller, &nm_ecu2, 0.1)
            .unwrap();
        assert_eq!(udp_nm_cluster.nm_nodes().count(), 2);
        assert_eq!(nm_node2.communication_controller(), Some(ecu2_communication_controller));

        nm_node1.add_rx_nm_pdu(&nm_pdu1).unwrap();
        nm_node1.add_tx_nm_pdu(&nm_pdu2).unwrap();
        assert_eq!(nm_node1.rx_nm_pdus().count(), 1);
        assert_eq!(nm_node1.tx_nm_pdus().count(), 1);
        nm_node2.add_rx_nm_pdu(&nm_pdu2).unwrap();
        nm_node2.add_tx_nm_pdu(&nm_pdu1).unwrap();
        assert_eq!(nm_node2.rx_nm_pdus().count(), 1);
        assert_eq!(nm_node2.tx_nm_pdus().count(), 1);

        assert_eq!(udp_nm_cluster.nm_nodes().next().unwrap(), nm_node1);

        // remove optional values
        nm_node1.set_node_id(None).unwrap();
        assert_eq!(nm_node1.node_id(), None);
        nm_node1.set_passive_mode(None).unwrap();
        assert_eq!(nm_node1.passive_mode(), None);
        nm_node1.set_all_nm_messages_keep_awake(None).unwrap();
        assert_eq!(nm_node1.all_nm_messages_keep_awake(), None);

        // ------ UDP NM Cluster Coupling ------

        let cluster_coupling = nm_config.create_udp_nm_cluster_coupling().unwrap();
        assert_eq!(nm_config.nm_cluster_couplings().count(), 1);
        cluster_coupling.add_coupled_cluster(&udp_nm_cluster).unwrap();
        assert_eq!(cluster_coupling.coupled_clusters().count(), 1);
        cluster_coupling.set_nm_immediate_restart_enabled(Some(true)).unwrap();
        assert_eq!(cluster_coupling.nm_immediate_restart_enabled(), Some(true));
        // remove optional values
        cluster_coupling.set_nm_immediate_restart_enabled(None).unwrap();
        assert_eq!(cluster_coupling.nm_immediate_restart_enabled(), None);

        // Cluster Coupling conversions
        let cluster_coupling2 = nm_config.nm_cluster_couplings().next().unwrap();
        assert_eq!(cluster_coupling.element(), cluster_coupling2.element());
    }
}
