use crate::communication::{
    CanCluster, CanFrame, CanTpConfig, Cluster, ContainerIPdu, ContainerIPduHeaderType, DcmIPdu, DoIpTpConfig,
    EthernetCluster, EventGroupControlType, FlexrayArTpConfig, FlexrayCluster, FlexrayClusterSettings, FlexrayFrame,
    FlexrayTpConfig, Frame, GeneralPurposeIPdu, GeneralPurposeIPduCategory, GeneralPurposePdu,
    GeneralPurposePduCategory, ISignal, ISignalGroup, ISignalIPdu, MultiplexedIPdu, NPdu, NmConfig, NmPdu, Pdu,
    RxAcceptContainedIPdu, SecureCommunicationProps, SecuredIPdu, ServiceInstanceCollectionSet, SoAdRoutingGroup,
    SocketConnectionIpduIdentifierSet, SomeipTpConfig, SystemSignal, SystemSignalGroup,
};
use crate::datatype::SwBaseType;
use crate::software_component::{CompositionSwComponentType, RootSwCompositionPrototype};
use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, EcuInstance, IdentifiableAbstractionElement,
    abstraction_element,
};
use autosar_data::{AutosarModel, Element, ElementName, WeakElement};
use std::iter::FusedIterator;

mod mapping;

pub use mapping::*;

/// The System is the top level of a system template
///
/// It defines how ECUs communicate with each other over various networks.
/// It also contains the mapping of software components to ECUs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct System(Element);
abstraction_element!(System, System);
impl IdentifiableAbstractionElement for System {}

impl System {
    // find an existing \<SYSTEM\> in the model, if it exists
    #[must_use]
    pub(crate) fn find(model: &AutosarModel) -> Option<Self> {
        let elem = model
            .identifiable_elements()
            .filter_map(|(_, weak)| weak.upgrade())
            .find(|elem| elem.element_name() == ElementName::System)?;
        Some(Self(elem))
    }

    /// Create a new SYSTEM in the given AR-PACKAGE
    ///
    /// Note that an Autosar model should ony contain one SYSTEM. This is not checked here.
    ///
    /// Use [`ArPackage::create_system`] to create a new system.
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/my/pkg")?;
    /// let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// assert!(model.get_element_by_path("/my/pkg/System").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the SYSTEM element
    pub(crate) fn new(
        name: &str,
        package: &ArPackage,
        category: SystemCategory,
    ) -> Result<Self, AutosarAbstractionError> {
        let pkg_elem_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;

        let system_elem = pkg_elem_elements.create_named_sub_element(ElementName::System, name)?;
        let system = System(system_elem);
        system.set_category(category)?;

        Ok(system)
    }

    /// set the category of the system
    pub fn set_category(&self, category: SystemCategory) -> Result<(), AutosarAbstractionError> {
        self.0
            .get_or_create_sub_element(ElementName::Category)?
            .set_character_data(category.to_string())?;
        Ok(())
    }

    /// get the category of the system
    #[must_use]
    pub fn category(&self) -> Option<SystemCategory> {
        self.0
            .get_sub_element(ElementName::Category)?
            .character_data()?
            .string_value()?
            .parse()
            .ok()
    }

    /// set the pncVectorLength of the system
    pub fn set_pnc_vector_length(&self, length: Option<u32>) -> Result<(), AutosarAbstractionError> {
        if let Some(length) = length {
            self.0
                .get_or_create_sub_element(ElementName::PncVectorLength)?
                .set_character_data(length as u64)?;
        } else {
            let _ = self.0.remove_sub_element_kind(ElementName::PncVectorLength);
        }
        Ok(())
    }

    /// get the pncVectorLength of the system
    #[must_use]
    pub fn pnc_vector_length(&self) -> Option<u32> {
        self.0
            .get_sub_element(ElementName::PncVectorLength)?
            .character_data()?
            .parse_integer()
    }

    /// set the pncVectorOffset of the system
    pub fn set_pnc_vector_offset(&self, offset: Option<u32>) -> Result<(), AutosarAbstractionError> {
        if let Some(offset) = offset {
            self.0
                .get_or_create_sub_element(ElementName::PncVectorOffset)?
                .set_character_data(offset as u64)?;
        } else {
            let _ = self.0.remove_sub_element_kind(ElementName::PncVectorOffset);
        }
        Ok(())
    }

    /// get the pncVectorOffset of the system
    #[must_use]
    pub fn pnc_vector_offset(&self) -> Option<u32> {
        self.0
            .get_sub_element(ElementName::PncVectorOffset)?
            .character_data()?
            .parse_integer()
    }

    /// create an `EcuInstance` that is connected to this System
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// # let package1 = model.get_or_create_package("/pkg1")?;
    /// let system = package1.create_system("System", SystemCategory::SystemExtract)?;
    /// # let package2 = model.get_or_create_package("/pkg2")?;
    /// let ecu_instance = system.create_ecu_instance("ecu_name", &package2)?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the ECU-INSTANCE
    pub fn create_ecu_instance(&self, name: &str, package: &ArPackage) -> Result<EcuInstance, AutosarAbstractionError> {
        let ecu_instance = EcuInstance::new(name, package)?;
        self.create_fibex_element_ref_unchecked(ecu_instance.element())?;

        Ok(ecu_instance)
    }

    /// get an iterator over all ECU-INSTANCEs in this SYSTEM
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// # let package = model.get_or_create_package("/pkg1")?;
    /// let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// system.create_ecu_instance("ecu_name1", &package)?;
    /// system.create_ecu_instance("ecu_name2", &package)?;
    /// for ecu in system.ecu_instances() {
    ///     // do something
    /// }
    /// assert_eq!(system.ecu_instances().count(), 2);
    /// # Ok(())}
    /// ```
    pub fn ecu_instances(&self) -> impl Iterator<Item = EcuInstance> + Send + 'static {
        EcuInstanceIterator::new(self)
    }

    /// create a new CAN-CLUSTER
    ///
    /// The cluster must have a channel to be valid, but this channel is not created automatically.
    /// Call [`CanCluster::create_physical_channel`] to create it.
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
    /// let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// let cluster = system.create_can_cluster("can_cluster", &package, None)?;
    /// cluster.create_physical_channel("can_channel");
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the can cluster
    pub fn create_can_cluster(
        &self,
        cluster_name: &str,
        package: &ArPackage,
        can_baudrate: Option<u32>,
    ) -> Result<CanCluster, AutosarAbstractionError> {
        let cluster = CanCluster::new(cluster_name, package, can_baudrate)?;
        self.create_fibex_element_ref_unchecked(cluster.element())?;

        Ok(cluster)
    }

    /// create a new ETHERNET-CLUSTER and connect it to the SYSTEM
    ///
    /// The cluster must have at least one channel to be valid.
    /// Call [`EthernetCluster::create_physical_channel`] to create it.
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
    /// let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// let cluster = system.create_ethernet_cluster("ethernet_cluster", &package)?;
    /// let vlan_info = EthernetVlanInfo { vlan_name: "VLAN_1".to_string(), vlan_id: 1};
    /// cluster.create_physical_channel("ethernet_channel", Some(&vlan_info));
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the ethernet cluster
    pub fn create_ethernet_cluster(
        &self,
        cluster_name: &str,
        package: &ArPackage,
    ) -> Result<EthernetCluster, AutosarAbstractionError> {
        let cluster = EthernetCluster::new(cluster_name, package)?;
        self.create_fibex_element_ref_unchecked(cluster.element())?;

        Ok(cluster)
    }

    /// create a new FLEXRAY-CLUSTER and connect it to the SYSTEM
    ///
    /// A `FlexrayClusterSettings` structure containing the timings and parameters for the Flexray cluster must be provided.
    ///
    /// The cluster must have at least one channel to be valid.
    /// Call [`FlexrayCluster::create_physical_channel`] to create it.
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
    /// let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// let cluster = system.create_flexray_cluster("flexray_cluster", &package, &FlexrayClusterSettings::default())?;
    /// cluster.create_physical_channel("flexray_channel", FlexrayChannelName::A);
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the flexray cluster
    pub fn create_flexray_cluster(
        &self,
        cluster_name: &str,
        package: &ArPackage,
        settings: &FlexrayClusterSettings,
    ) -> Result<FlexrayCluster, AutosarAbstractionError> {
        let cluster = FlexrayCluster::new(cluster_name, package, settings)?;
        self.create_fibex_element_ref_unchecked(cluster.element())?;

        Ok(cluster)
    }

    /// Create an iterator over all clusters connected to the SYSTEM
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
    /// let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// system.create_can_cluster("can_cluster", &package, None)?;
    /// system.create_flexray_cluster("flexray_cluster", &package, &FlexrayClusterSettings::default())?;
    /// for cluster in system.clusters() {
    ///     // do something
    /// }
    /// assert_eq!(system.clusters().count(), 2);
    /// # Ok(())}
    /// ```
    pub fn clusters(&self) -> impl Iterator<Item = Cluster> + Send + 'static {
        self.0
            .get_sub_element(ElementName::FibexElements)
            .into_iter()
            .flat_map(|fibexelems| fibexelems.sub_elements())
            .filter_map(|ferc| {
                ferc.get_sub_element(ElementName::FibexElementRef)
                    .and_then(|fer| fer.get_reference_target().ok())
                    .and_then(|elem| Cluster::try_from(elem).ok())
            })
    }

    /// create a new [`CanFrame`]
    ///
    /// This new frame needs to be linked to a `CanPhysicalChannel`
    pub fn create_can_frame(
        &self,
        name: &str,
        package: &ArPackage,
        byte_length: u64,
    ) -> Result<CanFrame, AutosarAbstractionError> {
        let can_frame = CanFrame::new(name, package, byte_length)?;
        self.create_fibex_element_ref_unchecked(can_frame.element())?;

        Ok(can_frame)
    }

    /// create a new [`FlexrayFrame`]
    ///
    /// This new frame needs to be linked to a `FlexrayPhysicalChannel`
    pub fn create_flexray_frame(
        &self,
        name: &str,
        package: &ArPackage,
        byte_length: u64,
    ) -> Result<FlexrayFrame, AutosarAbstractionError> {
        let flexray_frame = FlexrayFrame::new(name, package, byte_length)?;
        self.create_fibex_element_ref_unchecked(flexray_frame.element())?;

        Ok(flexray_frame)
    }

    /// iterate over all Frames in the System
    ///
    /// This iterator returns all CAN and Flexray frames that are connected to the System using a FibexElementRef.
    pub fn frames(&self) -> impl Iterator<Item = Frame> + Send + 'static {
        self.0
            .get_sub_element(ElementName::FibexElements)
            .into_iter()
            .flat_map(|fibexelems| fibexelems.sub_elements())
            .filter_map(|ferc| {
                ferc.get_sub_element(ElementName::FibexElementRef)
                    .and_then(|fer| fer.get_reference_target().ok())
                    .and_then(|elem| Frame::try_from(elem).ok())
            })
    }

    /// create a new isignal in the [`System`]
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
    /// let sig_package = model.get_or_create_package("/ISignals")?;
    /// let sys_package = model.get_or_create_package("/SystemSignals")?;
    /// let system_signal = sys_package.create_system_signal("signal1")?;
    /// system.create_isignal("signal1", &sig_package, 32, &system_signal, None)?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::InvalidParameter`] `sig_package` and `sys_package` may not be identical
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create elements
    pub fn create_isignal(
        &self,
        name: &str,
        package: &ArPackage,
        bit_length: u64,
        syssignal: &SystemSignal,
        datatype: Option<&SwBaseType>,
    ) -> Result<ISignal, AutosarAbstractionError> {
        let i_signal = ISignal::new(name, package, bit_length, syssignal, datatype)?;

        self.create_fibex_element_ref_unchecked(i_signal.element())?;

        Ok(i_signal)
    }

    /// iterate over all ISignals in the System
    ///
    /// This iterator returns all ISignals that are connected to the System using a FibexElementRef.
    pub fn isignals(&self) -> impl Iterator<Item = ISignal> + Send + 'static {
        self.0
            .get_sub_element(ElementName::FibexElements)
            .into_iter()
            .flat_map(|fibexelems| fibexelems.sub_elements())
            .filter_map(|ferc| {
                ferc.get_sub_element(ElementName::FibexElementRef)
                    .and_then(|fer| fer.get_reference_target().ok())
                    .and_then(|elem| ISignal::try_from(elem).ok())
            })
    }

    /// create a new signal group in the [`System`]
    ///
    /// `I-SIGNAL-GROUP` and `SYSTEM-SIGNAL-GROUP` are created using the same name; therefore they must be placed in
    /// different packages: `sig_package` and `sys_package` may not be identical.
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
    /// let sig_package = model.get_or_create_package("/ISignals")?;
    /// let sys_package = model.get_or_create_package("/SystemSignals")?;
    /// let system_signal_group = sys_package.create_system_signal_group("signalgroup")?;
    /// system.create_isignal_group("signal_group", &sig_package, &system_signal_group)?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::InvalidParameter`] `sig_package` and `sys_package` may not be identical
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create elements
    pub fn create_isignal_group(
        &self,
        name: &str,
        package: &ArPackage,
        system_signal_group: &SystemSignalGroup,
    ) -> Result<ISignalGroup, AutosarAbstractionError> {
        let i_signal_group = ISignalGroup::new(name, package, system_signal_group)?;

        self.create_fibex_element_ref_unchecked(i_signal_group.element())?;

        Ok(i_signal_group)
    }

    /// iterate over all ISignalGroups in the System
    ///
    /// This iterator returns all ISignalGroups that are connected to the System using a FibexElementRef.
    pub fn isignal_groups(&self) -> impl Iterator<Item = ISignalGroup> + Send + 'static {
        self.0
            .get_sub_element(ElementName::FibexElements)
            .into_iter()
            .flat_map(|fibexelems| fibexelems.sub_elements())
            .filter_map(|ferc| {
                ferc.get_sub_element(ElementName::FibexElementRef)
                    .and_then(|fer| fer.get_reference_target().ok())
                    .and_then(|elem| ISignalGroup::try_from(elem).ok())
            })
    }

    /// create an [`ISignalIPdu`] in the [`System`]
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
    /// let package = model.get_or_create_package("/Pdus")?;
    /// system.create_isignal_ipdu("pdu", &package, 42)?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create elements
    pub fn create_isignal_ipdu(
        &self,
        name: &str,
        package: &ArPackage,
        length: u32,
    ) -> Result<ISignalIPdu, AutosarAbstractionError> {
        let pdu = ISignalIPdu::new(name, package, length)?;
        self.create_fibex_element_ref_unchecked(pdu.element())?;

        Ok(pdu)
    }

    /// create an [`NmPdu`] in the [`System`]
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
    /// let package = model.get_or_create_package("/Pdus")?;
    /// system.create_nm_pdu("pdu", &package, 42)?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create elements
    pub fn create_nm_pdu(
        &self,
        name: &str,
        package: &ArPackage,
        length: u32,
    ) -> Result<NmPdu, AutosarAbstractionError> {
        let pdu = NmPdu::new(name, package, length)?;
        self.create_fibex_element_ref_unchecked(pdu.element())?;

        Ok(pdu)
    }

    /// create an [`NPdu`] in the [`System`]
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
    /// let package = model.get_or_create_package("/Pdus")?;
    /// system.create_n_pdu("pdu", &package, 42)?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create elements
    pub fn create_n_pdu(&self, name: &str, package: &ArPackage, length: u32) -> Result<NPdu, AutosarAbstractionError> {
        let pdu = NPdu::new(name, package, length)?;
        self.create_fibex_element_ref_unchecked(pdu.element())?;

        Ok(pdu)
    }

    /// create a [`DcmIPdu`] in the [`System`]
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
    /// let package = model.get_or_create_package("/Pdus")?;
    /// system.create_dcm_ipdu("pdu", &package, 42)?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create elements
    pub fn create_dcm_ipdu(
        &self,
        name: &str,
        package: &ArPackage,
        length: u32,
    ) -> Result<DcmIPdu, AutosarAbstractionError> {
        let pdu = DcmIPdu::new(name, package, length)?;
        self.create_fibex_element_ref_unchecked(pdu.element())?;

        Ok(pdu)
    }

    /// create a [`GeneralPurposePdu`] in the [`System`]
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
    /// let package = model.get_or_create_package("/Pdus")?;
    /// system.create_general_purpose_pdu("pdu", &package, 42, GeneralPurposePduCategory::GlobalTime)?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create elements
    pub fn create_general_purpose_pdu(
        &self,
        name: &str,
        package: &ArPackage,
        length: u32,
        category: GeneralPurposePduCategory,
    ) -> Result<GeneralPurposePdu, AutosarAbstractionError> {
        let pdu = GeneralPurposePdu::new(name, package, length, category)?;
        self.create_fibex_element_ref_unchecked(pdu.element())?;

        Ok(pdu)
    }

    /// create a [`GeneralPurposeIPdu`] in the [`System`]
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
    /// let package = model.get_or_create_package("/Pdus")?;
    /// system.create_general_purpose_ipdu("pdu", &package, 42, GeneralPurposeIPduCategory::Xcp)?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create elements
    pub fn create_general_purpose_ipdu(
        &self,
        name: &str,
        package: &ArPackage,
        length: u32,
        category: GeneralPurposeIPduCategory,
    ) -> Result<GeneralPurposeIPdu, AutosarAbstractionError> {
        let pdu = GeneralPurposeIPdu::new(name, package, length, category)?;
        self.create_fibex_element_ref_unchecked(pdu.element())?;

        Ok(pdu)
    }

    /// create a [`ContainerIPdu`] in the [`System`]
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
    /// let package = model.get_or_create_package("/Pdus")?;
    /// system.create_container_ipdu("pdu", &package, 42, ContainerIPduHeaderType::ShortHeader, RxAcceptContainedIPdu::AcceptAll)?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create elements
    pub fn create_container_ipdu(
        &self,
        name: &str,
        package: &ArPackage,
        length: u32,
        header_type: ContainerIPduHeaderType,
        rx_accept: RxAcceptContainedIPdu,
    ) -> Result<ContainerIPdu, AutosarAbstractionError> {
        let pdu = ContainerIPdu::new(name, package, length, header_type, rx_accept)?;
        self.create_fibex_element_ref_unchecked(pdu.element())?;

        Ok(pdu)
    }

    /// create a [`SecuredIPdu`] in the [`System`]
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
    /// let package = model.get_or_create_package("/Pdus")?;
    /// let secure_communication_props = SecureCommunicationProps::default();
    /// system.create_secured_ipdu("pdu", &package, 42, &secure_communication_props)?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create elements
    pub fn create_secured_ipdu(
        &self,
        name: &str,
        package: &ArPackage,
        length: u32,
        secure_props: &SecureCommunicationProps,
    ) -> Result<SecuredIPdu, AutosarAbstractionError> {
        let pdu = SecuredIPdu::new(name, package, length, secure_props)?;
        self.create_fibex_element_ref_unchecked(pdu.element())?;

        Ok(pdu)
    }

    /// create a [`MultiplexedIPdu`] in the [`System`]
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
    /// let package = model.get_or_create_package("/Pdus")?;
    /// system.create_multiplexed_ipdu("pdu", &package, 42)?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create elements
    pub fn create_multiplexed_ipdu(
        &self,
        name: &str,
        package: &ArPackage,
        length: u32,
    ) -> Result<MultiplexedIPdu, AutosarAbstractionError> {
        let pdu = MultiplexedIPdu::new(name, package, length)?;
        self.create_fibex_element_ref_unchecked(pdu.element())?;

        Ok(pdu)
    }

    /// iterate over all PDUs in the System
    ///
    /// This iterator returns all PDUs that are connected to the System using a FibexElementRef.
    pub fn pdus(&self) -> impl Iterator<Item = Pdu> + Send + 'static {
        self.0
            .get_sub_element(ElementName::FibexElements)
            .into_iter()
            .flat_map(|fibexelems| fibexelems.sub_elements())
            .filter_map(|ferc| {
                ferc.get_sub_element(ElementName::FibexElementRef)
                    .and_then(|fer| fer.get_reference_target().ok())
                    .and_then(|elem| Pdu::try_from(elem).ok())
            })
    }

    /// Create a `SocketConnectionIpduIdentifierSet` in the SYSTEM
    ///
    /// `SocketConnectionIpduIdentifierSet` are part of the new ethernet modeling that was introduced in Autosar 4.5.0 (`AUTOSAR_00048`).
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
    /// let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// let set = system.create_socket_connection_ipdu_identifier_set("set", &package)?;
    /// # Ok(())}
    /// ```
    pub fn create_socket_connection_ipdu_identifier_set(
        &self,
        name: &str,
        package: &ArPackage,
    ) -> Result<SocketConnectionIpduIdentifierSet, AutosarAbstractionError> {
        let set = SocketConnectionIpduIdentifierSet::new(name, package)?;
        self.create_fibex_element_ref_unchecked(set.element())?;

        Ok(set)
    }

    /// Create a `SoAdRoutingGroup` in the SYSTEM
    ///
    /// `SoAdRoutingGroup` are part of the old ethernet modeling that was used prior to Autosar 4.5.0 (`AUTOSAR_00048`).
    /// The elements are still present (but obsolete) in newer versions of the standard.
    /// Old and new elements may not be mixed in the same model.
    pub fn create_so_ad_routing_group(
        &self,
        name: &str,
        package: &ArPackage,
        control_type: Option<EventGroupControlType>,
    ) -> Result<SoAdRoutingGroup, AutosarAbstractionError> {
        let group = SoAdRoutingGroup::new(name, package, control_type)?;
        self.create_fibex_element_ref_unchecked(group.element())?;

        Ok(group)
    }

    /// Create a `ServiceInstanceCollectionSet` in the SYSTEM
    ///
    /// `ServiceInstanceCollectionSet`s are part of the new ethernet modeling that was introduced in Autosar 4.5.0 (`AUTOSAR_00048`).
    pub fn create_service_instance_collection_set(
        &self,
        name: &str,
        package: &ArPackage,
    ) -> Result<ServiceInstanceCollectionSet, AutosarAbstractionError> {
        let set = ServiceInstanceCollectionSet::new(name, package)?;
        self.create_fibex_element_ref_unchecked(set.element())?;

        Ok(set)
    }

    /// Create a `SomeipTpConfig` in the SYSTEM
    ///
    /// `SomeipTpConfig`s contain the configuration how to segment or reassemble large `SomeipTp` PDUs.
    pub fn create_someip_tp_config<T: Into<Cluster> + Clone>(
        &self,
        name: &str,
        package: &ArPackage,
        cluster: &T,
    ) -> Result<SomeipTpConfig, AutosarAbstractionError> {
        let config = SomeipTpConfig::new(name, package, &cluster.clone().into())?;
        self.create_fibex_element_ref_unchecked(config.element())?;

        Ok(config)
    }

    /// Create a `CanTpConfig` in the SYSTEM
    ///
    /// `CanTpConfig`s contain the configuration how to segment or reassemble diagnostic messages on a CAN bus.
    pub fn create_can_tp_config(
        &self,
        name: &str,
        package: &ArPackage,
        can_cluster: &CanCluster,
    ) -> Result<CanTpConfig, AutosarAbstractionError> {
        let config = CanTpConfig::new(name, package, can_cluster)?;
        self.create_fibex_element_ref_unchecked(config.element())?;

        Ok(config)
    }

    /// Create a `DoIpTpConfig` in the SYSTEM
    ///
    /// `DoIpTpConfig`s contain the configuration how to transmit diagnostic messages over IP networks.
    pub fn create_doip_tp_config(
        &self,
        name: &str,
        package: &ArPackage,
        eth_cluster: &EthernetCluster,
    ) -> Result<DoIpTpConfig, AutosarAbstractionError> {
        let config = DoIpTpConfig::new(name, package, eth_cluster)?;
        self.create_fibex_element_ref_unchecked(config.element())?;

        Ok(config)
    }

    /// Create a `FlexRayTpConfig` in the SYSTEM
    ///
    /// `FlexRayTpConfig`s describe how to segment or reassemble diagnostic messages on a `FlexRay` bus.
    /// This configuration type is used for Flexray ISO TP communication.
    pub fn create_flexray_tp_config(
        &self,
        name: &str,
        package: &ArPackage,
        flexray_cluster: &FlexrayCluster,
    ) -> Result<FlexrayTpConfig, AutosarAbstractionError> {
        let config = FlexrayTpConfig::new(name, package, flexray_cluster)?;
        self.create_fibex_element_ref_unchecked(config.element())?;

        Ok(config)
    }

    /// Create a `FlexrayArTpConfig` in the SYSTEM
    ///
    /// `FlexrayArTpConfig`s describe how to segment or reassemble diagnostic messages on a `FlexRay` bus.
    /// This configuration type is used for Flexray AUTOSAR TP communication.
    pub fn create_flexray_ar_tp_config(
        &self,
        name: &str,
        package: &ArPackage,
        flexray_cluster: &FlexrayCluster,
    ) -> Result<FlexrayArTpConfig, AutosarAbstractionError> {
        let config = FlexrayArTpConfig::new(name, package, flexray_cluster)?;
        self.create_fibex_element_ref_unchecked(config.element())?;

        Ok(config)
    }

    /// Create a new `NmConfig` in the SYSTEM
    ///
    /// `NmConfig`s contain the configuration for network management.
    /// The System may contain zero or one `NmConfig`s.
    pub fn create_nm_config(&self, name: &str, package: &ArPackage) -> Result<NmConfig, AutosarAbstractionError> {
        let config = NmConfig::new(name, package)?;
        self.create_fibex_element_ref_unchecked(config.element())?;

        Ok(config)
    }

    /// Get the `NmConfig` of the SYSTEM, if any
    ///
    /// The System may contain zero or one `NmConfig`s.
    #[must_use]
    pub fn nm_config(&self) -> Option<NmConfig> {
        self.0
            .get_sub_element(ElementName::FibexElements)
            .into_iter()
            .flat_map(|fibexelems| fibexelems.sub_elements())
            .find_map(|ferc| {
                ferc.get_sub_element(ElementName::FibexElementRef)
                    .and_then(|fer| fer.get_reference_target().ok())
                    .and_then(|elem| NmConfig::try_from(elem).ok())
            })
    }

    /// connect an element to the SYSTEM by creating a FIBEX-ELEMENT-REF
    ///
    /// If there is already a FIBEX-ELEMENT-REF, this function does nothing, successfully.
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
    /// let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// # let pkg_elements = package.element().get_sub_element(ElementName::Elements).unwrap();
    /// let can_cluster = pkg_elements.create_named_sub_element(ElementName::CanCluster, "Cluster")?;
    /// system.create_fibex_element_ref(&can_cluster)?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model
    pub fn create_fibex_element_ref(&self, elem: &Element) -> Result<(), AutosarAbstractionError> {
        let model = elem.model()?;
        let refs = model.get_references_to(&elem.path()?);
        for reference in refs.iter().filter_map(WeakElement::upgrade) {
            if reference.element_name() == ElementName::FibexElementRef {
                // a FIBEX-ELEMENT-REF for this elem already exists.
                return Ok(());
            }
        }
        self.create_fibex_element_ref_unchecked(elem)
    }

    fn create_fibex_element_ref_unchecked(&self, elem: &Element) -> Result<(), AutosarAbstractionError> {
        let fibex_elements = self.0.get_or_create_sub_element(ElementName::FibexElements)?;
        let fibex_element_ref = fibex_elements
            .create_sub_element(ElementName::FibexElementRefConditional)?
            .create_sub_element(ElementName::FibexElementRef)?;
        fibex_element_ref.set_reference_target(elem)?;
        Ok(())
    }

    /// set the root software composition of the system
    ///
    /// When the root software composition is set, a root sw composition prototype is created for it.
    /// This function will remove any existing root sw composition prototype
    pub fn set_root_sw_composition(
        &self,
        name: &str,
        composition_type: &CompositionSwComponentType,
    ) -> Result<RootSwCompositionPrototype, AutosarAbstractionError> {
        let root_compositions = self
            .0
            .get_or_create_sub_element(ElementName::RootSoftwareCompositions)?;

        if let Some(existing_composition) = root_compositions.get_sub_element(ElementName::RootSwCompositionPrototype) {
            root_compositions.remove_sub_element(existing_composition)?;
        }
        RootSwCompositionPrototype::new(name, &root_compositions, composition_type)
    }

    /// get the root software composition of the system
    #[must_use]
    pub fn root_sw_composition(&self) -> Option<RootSwCompositionPrototype> {
        let root_compositions = self.element().get_sub_element(ElementName::RootSoftwareCompositions)?;
        let root_composition = root_compositions.get_sub_element(ElementName::RootSwCompositionPrototype)?;
        RootSwCompositionPrototype::try_from(root_composition).ok()
    }

    /// get or create a mapping for this system
    ///
    /// There does not seem to be any benefit to having multiple mappings for a single system, so this function
    /// will return the first mapping if it exists. Otherwise a new mapping will be created with the provided name.
    pub fn get_or_create_mapping(&self, name: &str) -> Result<SystemMapping, AutosarAbstractionError> {
        if let Some(mapping) = self.0.get_sub_element(ElementName::Mappings) {
            if let Some(mapping) = mapping.get_sub_element(ElementName::SystemMapping) {
                return SystemMapping::try_from(mapping);
            }
        }
        SystemMapping::new(name, self)
    }
}

//#########################################################

/// The category of a System
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemCategory {
    /// The `System` is used to describe system constraints
    SystemConstraints,
    /// The `System` is used to describe the system configuration of a complete AUTOSAR system
    SystemDescription,
    /// The `System` is used to describe a subsystem specific view on the complete system description
    SystemExtract,
    /// The `System` is used to describe the ECU specific view on the complete system description
    EcuExtract,
    /// The `System` is used to describe a functional (solution-independent/abstract) system design
    AbstractSystemDescription,
    /// The `System` is used to describe the closed view on one ECU
    EcuSystemDescription,
    /// The `System` describes the content of one `CpSoftwareCluster`
    SwClusterSystemDescription,
    /// `System` which describes the rapid prototyping algorithm in the format of AUTOSAR Software Components
    RptSystem,
}

impl std::fmt::Display for SystemCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemCategory::SystemConstraints => f.write_str("SYSTEM_CONSTRAINTS"),
            SystemCategory::SystemDescription => f.write_str("SYSTEM_DESCRIPTION"),
            SystemCategory::SystemExtract => f.write_str("SYSTEM_EXTRACT"),
            SystemCategory::EcuExtract => f.write_str("ECU_EXTRACT"),
            SystemCategory::AbstractSystemDescription => f.write_str("ABSTRACT_SYSTEM_DESCRIPTION"),
            SystemCategory::EcuSystemDescription => f.write_str("ECU_SYSTEM_DESCRIPTION"),
            SystemCategory::SwClusterSystemDescription => f.write_str("SW_CLUSTER_SYSTEM_DESCRIPTION"),
            SystemCategory::RptSystem => f.write_str("RPT_SYSTEM"),
        }
    }
}

impl std::str::FromStr for SystemCategory {
    type Err = AutosarAbstractionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SYSTEM_CONSTRAINTS" => Ok(SystemCategory::SystemConstraints),
            "SYSTEM_DESCRIPTION" => Ok(SystemCategory::SystemDescription),
            "SYSTEM_EXTRACT" => Ok(SystemCategory::SystemExtract),
            "ECU_EXTRACT" => Ok(SystemCategory::EcuExtract),
            "ABSTRACT_SYSTEM_DESCRIPTION" => Ok(SystemCategory::AbstractSystemDescription),
            "ECU_SYSTEM_DESCRIPTION" => Ok(SystemCategory::EcuSystemDescription),
            "SW_CLUSTER_SYSTEM_DESCRIPTION" => Ok(SystemCategory::SwClusterSystemDescription),
            "RPT_SYSTEM" => Ok(SystemCategory::RptSystem),
            _ => Err(AutosarAbstractionError::InvalidParameter(s.to_string())),
        }
    }
}

//#########################################################

/// An iterator over all `EcuInstances` in a System
pub struct EcuInstanceIterator {
    fibex_elements: Option<Element>,
    position: usize,
}

impl EcuInstanceIterator {
    pub(crate) fn new(system: &System) -> Self {
        let fibex_elements = system.0.get_sub_element(ElementName::FibexElements);

        EcuInstanceIterator {
            fibex_elements,
            position: 0,
        }
    }
}

impl Iterator for EcuInstanceIterator {
    type Item = EcuInstance;

    fn next(&mut self) -> Option<Self::Item> {
        let fibelem = self.fibex_elements.as_ref()?;

        while let Some(fibrefcond) = fibelem.get_sub_element_at(self.position) {
            self.position += 1;
            if let Some(ecuinstance) = fibrefcond
                .get_sub_element(ElementName::FibexElementRef)
                .and_then(|r| r.get_reference_target().ok())
                .and_then(|target| EcuInstance::try_from(target).ok())
            {
                return Some(ecuinstance);
            }
        }
        self.fibex_elements = None;
        None
    }
}

impl FusedIterator for EcuInstanceIterator {}

//#########################################################

#[cfg(test)]
mod test {
    use crate::{
        AbstractionElement, AutosarModelAbstraction, IdentifiableAbstractionElement, System,
        communication::{
            ContainerIPduHeaderType, FlexrayClusterSettings, GeneralPurposeIPduCategory, GeneralPurposePduCategory,
            RxAcceptContainedIPdu, SecureCommunicationProps,
        },
        software_component::CompositionSwComponentType,
        system::SystemCategory,
    };
    use autosar_data::{AutosarVersion, ElementName};

    #[test]
    fn system() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);

        // try to find a system in the empty model
        let result = model.find_system();
        assert!(result.is_none());

        // create a System
        let package = model.get_or_create_package("/SYSTEM").unwrap();
        let system = package.create_system("System", SystemCategory::SystemExtract).unwrap();

        // find the newly created system
        let system_2 = model.find_system().unwrap();
        assert_eq!(system, system_2);

        // name
        assert_eq!(system.name().unwrap(), "System");
        system.set_name("NewName").unwrap();
        assert_eq!(system.name().unwrap(), "NewName");

        // category
        assert_eq!(system.category().unwrap(), SystemCategory::SystemExtract);
        system.set_category(SystemCategory::EcuExtract).unwrap();
        assert_eq!(system.category().unwrap(), SystemCategory::EcuExtract);

        // pnc vector length
        assert!(system.pnc_vector_length().is_none());
        system.set_pnc_vector_length(Some(42)).unwrap();
        assert_eq!(system.pnc_vector_length().unwrap(), 42);
        system.set_pnc_vector_length(None).unwrap();
        assert!(system.pnc_vector_length().is_none());

        // pnc vector offset
        assert!(system.pnc_vector_offset().is_none());
        system.set_pnc_vector_offset(Some(42)).unwrap();
        assert_eq!(system.pnc_vector_offset().unwrap(), 42);
        system.set_pnc_vector_offset(None).unwrap();
        assert!(system.pnc_vector_offset().is_none());
    }

    #[test]
    fn system_category() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/SYSTEM").unwrap();
        System::new("System", &package, SystemCategory::AbstractSystemDescription).unwrap();

        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/SYSTEM").unwrap();
        System::new("System", &package, SystemCategory::EcuExtract).unwrap();

        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/SYSTEM").unwrap();
        System::new("System", &package, SystemCategory::EcuSystemDescription).unwrap();

        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/SYSTEM").unwrap();
        System::new("System", &package, SystemCategory::RptSystem).unwrap();

        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/SYSTEM").unwrap();
        System::new("System", &package, SystemCategory::SwClusterSystemDescription).unwrap();

        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/SYSTEM").unwrap();
        System::new("System", &package, SystemCategory::SystemConstraints).unwrap();

        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/SYSTEM").unwrap();
        System::new("System", &package, SystemCategory::SystemDescription).unwrap();

        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/SYSTEM").unwrap();
        System::new("System", &package, SystemCategory::SystemExtract).unwrap();
    }

    #[test]
    fn fibex_ref() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/SYSTEM").unwrap();
        let system = package
            .create_system("System", SystemCategory::SystemDescription)
            .unwrap();

        let el_elements = package
            .element()
            .get_or_create_sub_element(ElementName::Elements)
            .unwrap();
        let el_ecuinst = el_elements
            .create_named_sub_element(ElementName::EcuInstance, "Ecu")
            .unwrap();

        let el_fibex_elements = system
            .element()
            .get_or_create_sub_element(ElementName::FibexElements)
            .unwrap();
        assert_eq!(el_fibex_elements.sub_elements().count(), 0);

        // create one reference
        system.create_fibex_element_ref(&el_ecuinst).unwrap();
        assert_eq!(el_fibex_elements.sub_elements().count(), 1);
        // find the existing reference and do nothing
        system.create_fibex_element_ref(&el_ecuinst).unwrap();
        assert_eq!(el_fibex_elements.sub_elements().count(), 1);
    }

    #[test]
    fn ecu_instance_iterator() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package_1 = model.get_or_create_package("/SYSTEM").unwrap();
        let system = package_1
            .create_system("System", SystemCategory::SystemExtract)
            .unwrap();
        let package_2 = model.get_or_create_package("/ECU").unwrap();
        system.create_ecu_instance("Ecu_1", &package_2).unwrap();
        system.create_ecu_instance("Ecu_2", &package_2).unwrap();
        system.create_ecu_instance("Ecu_3", &package_2).unwrap();

        let mut iter = system.ecu_instances();
        let item = iter.next().unwrap();
        assert_eq!(item.name().unwrap(), "Ecu_1");
        assert_eq!(model.get_element_by_path("/ECU/Ecu_1").unwrap(), *item.element());
        let item = iter.next().unwrap();
        assert_eq!(item.name().unwrap(), "Ecu_2");
        assert_eq!(model.get_element_by_path("/ECU/Ecu_2").unwrap(), *item.element());
        let item = iter.next().unwrap();
        assert_eq!(item.name().unwrap(), "Ecu_3");
        assert_eq!(model.get_element_by_path("/ECU/Ecu_3").unwrap(), *item.element());

        assert!(iter.next().is_none());
        // after returning none the iterator continues to return none
        assert!(iter.next().is_none());
    }

    #[test]
    fn cluster_iterator() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package_1 = model.get_or_create_package("/SYSTEM").unwrap();
        let system = package_1
            .create_system("System", SystemCategory::SystemExtract)
            .unwrap();
        let package_2 = model.get_or_create_package("/Clusters").unwrap();

        system.create_can_cluster("CanCluster", &package_2, None).unwrap();

        let settings = FlexrayClusterSettings::new();
        system
            .create_flexray_cluster("FlexrayCluster", &package_2, &settings)
            .unwrap();

        system.create_ethernet_cluster("EthernetCluster", &package_2).unwrap();

        // the ecu-instance is a fourth item in the FIBEX-ELEMENTS of the system, which should not be picked up by the iterator
        let package_3 = model.get_or_create_package("/ECU").unwrap();
        system.create_ecu_instance("Ecu_1", &package_3).unwrap();

        assert_eq!(system.clusters().count(), 3);
    }

    #[test]
    fn frames_iterator() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package_1 = model.get_or_create_package("/SYSTEM").unwrap();
        let system = package_1
            .create_system("System", SystemCategory::SystemExtract)
            .unwrap();
        let package_2 = model.get_or_create_package("/Frames").unwrap();

        system.create_can_frame("CanFrame", &package_2, 8).unwrap();
        system.create_flexray_frame("FlexrayFrame", &package_2, 8).unwrap();

        // the ecu-instance is a third item in the FIBEX-ELEMENTS of the system, which should not be picked up by the iterator
        let package_3 = model.get_or_create_package("/ECU").unwrap();
        system.create_ecu_instance("Ecu_1", &package_3).unwrap();

        assert_eq!(system.frames().count(), 2);
    }

    #[test]
    fn signals_iterator() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package_1 = model.get_or_create_package("/SYSTEM").unwrap();
        let system = package_1
            .create_system("System", SystemCategory::SystemExtract)
            .unwrap();
        let package_2 = model.get_or_create_package("/Signals").unwrap();

        let syssig1 = package_2.create_system_signal("syssig1").unwrap();
        system.create_isignal("Sig1", &package_2, 8, &syssig1, None).unwrap();
        let syssig2 = package_2.create_system_signal("syssig2").unwrap();
        system.create_isignal("Sig2", &package_2, 8, &syssig2, None).unwrap();

        // the ecu-instance is a third item in the FIBEX-ELEMENTS of the system, which should not be picked up by the iterator
        let package_3 = model.get_or_create_package("/ECU").unwrap();
        system.create_ecu_instance("Ecu_1", &package_3).unwrap();

        assert_eq!(system.isignals().count(), 2);
    }

    #[test]
    fn isignal_groups_iterator() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package_1 = model.get_or_create_package("/SYSTEM").unwrap();
        let system = package_1
            .create_system("System", SystemCategory::SystemExtract)
            .unwrap();
        let package_2 = model.get_or_create_package("/SignalGroups").unwrap();

        let sysgroup1 = package_2.create_system_signal_group("sysgroup1").unwrap();
        system
            .create_isignal_group("siggroup1", &package_2, &sysgroup1)
            .unwrap();
        let sysgroup2 = package_2.create_system_signal_group("sysgroup2").unwrap();
        system
            .create_isignal_group("siggroup2", &package_2, &sysgroup2)
            .unwrap();

        // the ecu-instance is a third item in the FIBEX-ELEMENTS of the system, which should not be picked up by the iterator
        let package_3 = model.get_or_create_package("/ECU").unwrap();
        system.create_ecu_instance("Ecu_1", &package_3).unwrap();

        assert_eq!(system.isignal_groups().count(), 2);
    }

    #[test]
    fn pdus_iterator() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package_1 = model.get_or_create_package("/SYSTEM").unwrap();
        let system = package_1
            .create_system("System", SystemCategory::SystemExtract)
            .unwrap();
        let package_2 = model.get_or_create_package("/Pdus").unwrap();

        system.create_dcm_ipdu("DcmIpdu", &package_2, 8).unwrap();
        system
            .create_general_purpose_pdu("GeneralPurposePdu", &package_2, 8, GeneralPurposePduCategory::DoIp)
            .unwrap();
        system
            .create_general_purpose_ipdu("GeneralPurposeIpdu", &package_2, 8, GeneralPurposeIPduCategory::Xcp)
            .unwrap();
        system
            .create_container_ipdu(
                "ContainerIpdu",
                &package_2,
                8,
                ContainerIPduHeaderType::NoHeader,
                RxAcceptContainedIPdu::AcceptAll,
            )
            .unwrap();
        system
            .create_secured_ipdu("SecuredIpdu", &package_2, 8, &SecureCommunicationProps::default())
            .unwrap();
        system
            .create_multiplexed_ipdu("MultiplexedIpdu", &package_2, 8)
            .unwrap();

        // the EcuInstance is a seventh item in the FIBEX-ELEMENTS of the system, which should not be picked up by the iterator
        let package_3 = model.get_or_create_package("/ECU").unwrap();
        system.create_ecu_instance("Ecu_1", &package_3).unwrap();

        assert_eq!(system.pdus().count(), 6);
    }

    #[test]
    fn nm_config() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let sys_package = model.get_or_create_package("/SYSTEM").unwrap();
        let system = sys_package
            .create_system("System", SystemCategory::SystemExtract)
            .unwrap();

        assert!(system.nm_config().is_none());

        let nm_package = model.get_or_create_package("/Nm").unwrap();
        let nm_config = system.create_nm_config("NmConfig", &nm_package).unwrap();

        assert!(system.nm_config().is_some());
        assert_eq!(system.nm_config().unwrap(), nm_config);
    }

    #[test]
    fn sw_mapping() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package_1 = model.get_or_create_package("/SYSTEM").unwrap();
        let system = package_1
            .create_system("System", SystemCategory::SystemExtract)
            .unwrap();
        let package_2 = model.get_or_create_package("/SWC").unwrap();
        let package_3 = model.get_or_create_package("/ECU").unwrap();

        let root_composition = CompositionSwComponentType::new("RootComposition", &package_2).unwrap();
        let context_composition = CompositionSwComponentType::new("ContextComposition", &package_2).unwrap();
        let ecu_composition = CompositionSwComponentType::new("EcuComposition", &package_2).unwrap();
        let _root_proto = system
            .set_root_sw_composition("RootComposition", &root_composition)
            .unwrap();
        assert_eq!(system.root_sw_composition().unwrap(), _root_proto);

        let context_proto = root_composition
            .create_component("ContextComposition", &context_composition.clone())
            .unwrap();
        let ecu_proto = context_composition
            .create_component("EcuComposition", &ecu_composition)
            .unwrap();
        let ecu = system.create_ecu_instance("Ecu", &package_3).unwrap();

        let mapping = system.get_or_create_mapping("Mapping").unwrap();
        mapping.map_swc_to_ecu("SwcToEcu1", &context_proto, &ecu).unwrap();
        let swc_to_ecu = mapping.map_swc_to_ecu("SwcToEcu2", &ecu_proto, &ecu).unwrap();

        assert_eq!(swc_to_ecu.target_component().unwrap(), ecu_proto);
        assert_eq!(swc_to_ecu.ecu_instance().unwrap(), ecu);

        // println!("{}", _file.serialize().unwrap());
    }
}
