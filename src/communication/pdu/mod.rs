use crate::communication::{
    AbstractPhysicalChannel, CommunicationDirection, ISignal, ISignalGroup, ISignalTriggering, PhysicalChannel,
};
use crate::{
    abstraction_element, make_unique_name, reflist_iterator, AbstractionElement, ArPackage, AutosarAbstractionError,
    EcuInstance,
};
use autosar_data::{AutosarDataError, Element, ElementName, EnumItem};
use std::str::FromStr;

mod isignal_ipdu;

pub use isignal_ipdu::*;

//##################################################################

/// This trait is implemented by all Pdus
pub trait AbstractPdu: AbstractionElement + Into<Pdu> {
    /// get the length of the PDU
    fn length(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::Length)?
            .character_data()?
            .parse_integer()
    }

    /// iterate over the `PduTriggerings` that trigger this PDU
    fn pdu_triggerings(&self) -> impl Iterator<Item = PduTriggering> + Send + 'static {
        let model_result = self.element().model();
        let path_result = self.element().path();
        if let (Ok(model), Ok(path)) = (model_result, path_result) {
            let reflist = model.get_references_to(&path);
            PduTriggeringsIterator::new(reflist)
        } else {
            PduTriggeringsIterator::new(vec![])
        }
    }
}

//##################################################################

/// for now this is a marker trait to identify `IPdus`
pub trait AbstractIpdu: AbstractPdu {}

//##################################################################

/// Network Management Pdu
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NmPdu(Element);
abstraction_element!(NmPdu, NmPdu);

impl NmPdu {
    pub(crate) fn new(name: &str, package: &ArPackage, length: u32) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_pdu = pkg_elements.create_named_sub_element(ElementName::NmPdu, name)?;
        elem_pdu
            .create_sub_element(ElementName::Length)?
            .set_character_data(length.to_string())?;

        Ok(Self(elem_pdu))
    }
}

impl AbstractPdu for NmPdu {}

impl From<NmPdu> for Pdu {
    fn from(value: NmPdu) -> Self {
        Pdu::NmPdu(value)
    }
}

//##################################################################

/// This is a Pdu of the transport layer. The main purpose of the TP layer is to segment and reassemble `IPdus`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NPdu(Element);
abstraction_element!(NPdu, NPdu);

impl NPdu {
    pub(crate) fn new(name: &str, package: &ArPackage, length: u32) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_pdu = pkg_elements.create_named_sub_element(ElementName::NPdu, name)?;
        elem_pdu
            .create_sub_element(ElementName::Length)?
            .set_character_data(length.to_string())?;

        Ok(Self(elem_pdu))
    }
}

impl AbstractPdu for NPdu {}

impl AbstractIpdu for NPdu {}

impl From<NPdu> for Pdu {
    fn from(value: NPdu) -> Self {
        Pdu::NPdu(value)
    }
}

//##################################################################

/// Represents the `IPdus` handled by Dcm
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DcmIPdu(Element);
abstraction_element!(DcmIPdu, DcmIPdu);

impl DcmIPdu {
    pub(crate) fn new(name: &str, package: &ArPackage, length: u32) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_pdu = pkg_elements.create_named_sub_element(ElementName::DcmIPdu, name)?;
        elem_pdu
            .create_sub_element(ElementName::Length)?
            .set_character_data(length.to_string())?;

        Ok(Self(elem_pdu))
    }
}

impl AbstractPdu for DcmIPdu {}

impl AbstractIpdu for DcmIPdu {}

impl From<DcmIPdu> for Pdu {
    fn from(value: DcmIPdu) -> Self {
        Pdu::DcmIPdu(value)
    }
}

//##################################################################

/// This element is used for AUTOSAR Pdus without additional attributes that are routed by a bus interface
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GeneralPurposePdu(Element);
abstraction_element!(GeneralPurposePdu, GeneralPurposePdu);

impl GeneralPurposePdu {
    pub(crate) fn new(
        name: &str,
        package: &ArPackage,
        length: u32,
        category: GeneralPurposePduCategory,
    ) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_pdu = pkg_elements.create_named_sub_element(ElementName::GeneralPurposePdu, name)?;
        elem_pdu
            .create_sub_element(ElementName::Category)?
            .set_character_data(category.to_string())?;
        elem_pdu
            .create_sub_element(ElementName::Length)?
            .set_character_data(length.to_string())?;

        Ok(Self(elem_pdu))
    }

    /// get the category of this PDU
    #[must_use]
    pub fn category(&self) -> Option<GeneralPurposePduCategory> {
        let category_string = self
            .element()
            .get_sub_element(ElementName::Category)?
            .character_data()?
            .string_value()?;
        GeneralPurposePduCategory::from_str(&category_string).ok()
    }
}

impl AbstractPdu for GeneralPurposePdu {}

impl From<GeneralPurposePdu> for Pdu {
    fn from(value: GeneralPurposePdu) -> Self {
        Pdu::GeneralPurposePdu(value)
    }
}

//##################################################################

/// The category of a `GeneralPurposePdu`
///
/// The Autosar standard defines the following categories:
/// - `SD`
/// - `GLOBAL_TIME`
/// - `DOIP`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeneralPurposePduCategory {
    /// Service Discovery
    Sd,
    /// Global Time Synchronization
    GlobalTime,
    /// Diagnostic over IP
    DoIp,
}

impl std::fmt::Display for GeneralPurposePduCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeneralPurposePduCategory::Sd => write!(f, "SD"),
            GeneralPurposePduCategory::GlobalTime => write!(f, "GLOBAL_TIME"),
            GeneralPurposePduCategory::DoIp => write!(f, "DOIP"),
        }
    }
}

impl std::str::FromStr for GeneralPurposePduCategory {
    type Err = AutosarAbstractionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SD" => Ok(GeneralPurposePduCategory::Sd),
            "GLOBAL_TIME" => Ok(GeneralPurposePduCategory::GlobalTime),
            "DOIP" => Ok(GeneralPurposePduCategory::DoIp),
            _ => Err(AutosarAbstractionError::InvalidParameter(s.to_string())),
        }
    }
}

//##################################################################

/// This element is used for AUTOSAR Pdus without attributes that are routed by the `PduR`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GeneralPurposeIPdu(Element);
abstraction_element!(GeneralPurposeIPdu, GeneralPurposeIPdu);

impl GeneralPurposeIPdu {
    pub(crate) fn new(
        name: &str,
        package: &ArPackage,
        length: u32,
        category: GeneralPurposeIPduCategory,
    ) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_pdu = pkg_elements.create_named_sub_element(ElementName::GeneralPurposeIPdu, name)?;
        elem_pdu
            .create_sub_element(ElementName::Length)?
            .set_character_data(length.to_string())?;
        elem_pdu
            .create_sub_element(ElementName::Category)?
            .set_character_data(category.to_string())?;

        Ok(Self(elem_pdu))
    }

    /// get the category of this PDU
    #[must_use]
    pub fn category(&self) -> Option<GeneralPurposeIPduCategory> {
        let category_string = self
            .element()
            .get_sub_element(ElementName::Category)?
            .character_data()?
            .string_value()?;
        GeneralPurposeIPduCategory::from_str(&category_string).ok()
    }
}

impl AbstractPdu for GeneralPurposeIPdu {}

impl AbstractIpdu for GeneralPurposeIPdu {}

impl From<GeneralPurposeIPdu> for Pdu {
    fn from(value: GeneralPurposeIPdu) -> Self {
        Pdu::GeneralPurposeIPdu(value)
    }
}

//##################################################################

/// The category of a `GeneralPurposeIPdu`
///
/// The Autosar standard defines the following categories:
/// - XCP
/// - `SOMEIP_SEGMENTED_IPDU`
/// - DLT
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeneralPurposeIPduCategory {
    /// XCP
    Xcp,
    /// SOME/IP Segmented `IPdu`
    SomeipSegmentedIpdu,
    /// Diagnostic Log and Trace
    Dlt,
}

impl std::fmt::Display for GeneralPurposeIPduCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeneralPurposeIPduCategory::Xcp => write!(f, "XCP"),
            GeneralPurposeIPduCategory::SomeipSegmentedIpdu => write!(f, "SOMEIP_SEGMENTED_IPDU"),
            GeneralPurposeIPduCategory::Dlt => write!(f, "DLT"),
        }
    }
}

impl std::str::FromStr for GeneralPurposeIPduCategory {
    type Err = AutosarAbstractionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "XCP" => Ok(GeneralPurposeIPduCategory::Xcp),
            "SOMEIP_SEGMENTED_IPDU" => Ok(GeneralPurposeIPduCategory::SomeipSegmentedIpdu),
            "DLT" => Ok(GeneralPurposeIPduCategory::Dlt),
            _ => Err(AutosarAbstractionError::InvalidParameter(s.to_string())),
        }
    }
}

//##################################################################

/// Several `IPdus` can be collected in one `ContainerIPdu` based on the headerType
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContainerIPdu(Element);
abstraction_element!(ContainerIPdu, ContainerIPdu);

impl ContainerIPdu {
    pub(crate) fn new(name: &str, package: &ArPackage, length: u32) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_pdu = pkg_elements.create_named_sub_element(ElementName::ContainerIPdu, name)?;
        elem_pdu
            .create_sub_element(ElementName::Length)?
            .set_character_data(length.to_string())?;

        Ok(Self(elem_pdu))
    }
}

impl AbstractPdu for ContainerIPdu {}

impl AbstractIpdu for ContainerIPdu {}

impl From<ContainerIPdu> for Pdu {
    fn from(value: ContainerIPdu) -> Self {
        Pdu::ContainerIPdu(value)
    }
}

//##################################################################

/// Wraps an `IPdu` to protect it from unauthorized manipulation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SecuredIPdu(Element);
abstraction_element!(SecuredIPdu, SecuredIPdu);

impl SecuredIPdu {
    pub(crate) fn new(name: &str, package: &ArPackage, length: u32) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_pdu = pkg_elements.create_named_sub_element(ElementName::SecuredIPdu, name)?;
        elem_pdu
            .create_sub_element(ElementName::Length)?
            .set_character_data(length.to_string())?;

        Ok(Self(elem_pdu))
    }
}

impl AbstractPdu for SecuredIPdu {}

impl AbstractIpdu for SecuredIPdu {}

impl From<SecuredIPdu> for Pdu {
    fn from(value: SecuredIPdu) -> Self {
        Pdu::SecuredIPdu(value)
    }
}

//##################################################################

/// The multiplexed pdu contains one of serveral signal pdus
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MultiplexedIPdu(Element);
abstraction_element!(MultiplexedIPdu, MultiplexedIPdu);

impl MultiplexedIPdu {
    pub(crate) fn new(name: &str, package: &ArPackage, length: u32) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_pdu = pkg_elements.create_named_sub_element(ElementName::MultiplexedIPdu, name)?;
        elem_pdu
            .create_sub_element(ElementName::Length)?
            .set_character_data(length.to_string())?;

        Ok(Self(elem_pdu))
    }
}

impl AbstractPdu for MultiplexedIPdu {}

impl AbstractIpdu for MultiplexedIPdu {}

impl From<MultiplexedIPdu> for Pdu {
    fn from(value: MultiplexedIPdu) -> Self {
        Pdu::MultiplexedIPdu(value)
    }
}

//##################################################################

/// Wrapper for all Pdu types. It is used as a return value for functions that can return any Pdu type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pdu {
    /// The Pdu is an `ISignalIPdu`
    ISignalIPdu(ISignalIPdu),
    /// The Pdu is a Network Management Pdu
    NmPdu(NmPdu),
    /// The Pdu is a Transport Layer Pdu
    NPdu(NPdu),
    /// The Pdu is a Diagnostic Communication Management Pdu
    DcmIPdu(DcmIPdu),
    /// The Pdu is a General Purpose Pdu
    GeneralPurposePdu(GeneralPurposePdu),
    /// The Pdu is a General Purpose `IPdu`
    GeneralPurposeIPdu(GeneralPurposeIPdu),
    /// The Pdu is a Container `IPdu`
    ContainerIPdu(ContainerIPdu),
    /// The Pdu is a Secured `IPdu`
    SecuredIPdu(SecuredIPdu),
    /// The Pdu is a Multiplexed `IPdu`
    MultiplexedIPdu(MultiplexedIPdu),
}

impl AbstractionElement for Pdu {
    fn element(&self) -> &Element {
        match self {
            Pdu::ISignalIPdu(pdu) => pdu.element(),
            Pdu::NmPdu(pdu) => pdu.element(),
            Pdu::NPdu(pdu) => pdu.element(),
            Pdu::DcmIPdu(pdu) => pdu.element(),
            Pdu::GeneralPurposePdu(pdu) => pdu.element(),
            Pdu::GeneralPurposeIPdu(pdu) => pdu.element(),
            Pdu::ContainerIPdu(pdu) => pdu.element(),
            Pdu::SecuredIPdu(pdu) => pdu.element(),
            Pdu::MultiplexedIPdu(pdu) => pdu.element(),
        }
    }
}

impl TryFrom<Element> for Pdu {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::ISignalIPdu => Ok(ISignalIPdu::try_from(element)?.into()),
            ElementName::NmPdu => Ok(NmPdu::try_from(element)?.into()),
            ElementName::NPdu => Ok(NPdu::try_from(element)?.into()),
            ElementName::DcmIPdu => Ok(DcmIPdu::try_from(element)?.into()),
            ElementName::GeneralPurposePdu => Ok(GeneralPurposePdu::try_from(element)?.into()),
            ElementName::GeneralPurposeIPdu => Ok(GeneralPurposeIPdu::try_from(element)?.into()),
            ElementName::ContainerIPdu => Ok(ContainerIPdu::try_from(element)?.into()),
            ElementName::SecuredIPdu => Ok(SecuredIPdu::try_from(element)?.into()),
            ElementName::MultiplexedIPdu => Ok(MultiplexedIPdu::try_from(element)?.into()),
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "Pdu".to_string(),
            }),
        }
    }
}

impl AbstractPdu for Pdu {}

//##################################################################

/// a `PduTriggering` triggers a PDU in a frame or ethernet connection
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PduTriggering(Element);
abstraction_element!(PduTriggering, PduTriggering);

impl PduTriggering {
    pub(crate) fn new(pdu: &Pdu, channel: &PhysicalChannel) -> Result<Self, AutosarAbstractionError> {
        let model = channel.element().model()?;
        let base_path = channel.element().path()?;
        let pdu_name = pdu
            .name()
            .ok_or(AutosarAbstractionError::InvalidParameter("invalid pdu".to_string()))?;
        let pt_name = format!("PT_{pdu_name}");
        let pt_name = make_unique_name(&model, &base_path, &pt_name);

        let triggerings = channel
            .element()
            .get_or_create_sub_element(ElementName::PduTriggerings)?;
        let pt_elem = triggerings.create_named_sub_element(ElementName::PduTriggering, &pt_name)?;
        pt_elem
            .create_sub_element(ElementName::IPduRef)?
            .set_reference_target(pdu.element())?;

        let pt = Self(pt_elem);

        if let Pdu::ISignalIPdu(isignal_ipdu) = pdu {
            for signal_mapping in isignal_ipdu.mapped_signals() {
                if let Some(signal) = signal_mapping.signal() {
                    pt.add_signal_triggering(&signal)?;
                } else if let Some(signal_group) = signal_mapping.signal_group() {
                    pt.add_signal_group_triggering(&signal_group)?;
                }
            }
        }

        Ok(pt)
    }

    /// get the Pdu that is triggered by this pdu triggering
    #[must_use]
    pub fn pdu(&self) -> Option<Pdu> {
        let pdu_elem = self
            .element()
            .get_sub_element(ElementName::IPduRef)?
            .get_reference_target()
            .ok()?;
        Pdu::try_from(pdu_elem).ok()
    }

    /// get the physical channel that contains this pdu triggering
    pub fn physical_channel(&self) -> Result<PhysicalChannel, AutosarAbstractionError> {
        let channel_elem = self.element().named_parent()?.ok_or(AutosarDataError::ItemDeleted)?;
        PhysicalChannel::try_from(channel_elem)
    }

    /// create an `IPduPort` to connect a `PduTriggering` to an `EcuInstance`
    pub fn create_pdu_port(
        &self,
        ecu: &EcuInstance,
        direction: CommunicationDirection,
    ) -> Result<IPduPort, AutosarAbstractionError> {
        for pdu_port in self.pdu_ports() {
            if let (Some(existing_ecu), Some(existing_direction)) = (pdu_port.ecu(), pdu_port.communication_direction())
            {
                if existing_ecu == *ecu && existing_direction == direction {
                    return Ok(pdu_port);
                }
            }
        }

        let channel = self.physical_channel()?;
        let connector = channel
            .ecu_connector(ecu)
            .ok_or(AutosarAbstractionError::InvalidParameter(
                "The ECU is not connected to the channel".to_string(),
            ))?;

        let name = self.name().ok_or(AutosarDataError::ItemDeleted)?;
        let suffix = match direction {
            CommunicationDirection::In => "Rx",
            CommunicationDirection::Out => "Tx",
        };
        let port_name = format!("{name}_{suffix}",);
        let pp_elem = connector
            .element()
            .get_or_create_sub_element(ElementName::EcuCommPortInstances)?
            .create_named_sub_element(ElementName::IPduPort, &port_name)?;
        pp_elem
            .create_sub_element(ElementName::CommunicationDirection)?
            .set_character_data::<EnumItem>(direction.into())?;

        self.element()
            .get_or_create_sub_element(ElementName::IPduPortRefs)?
            .create_sub_element(ElementName::IPduPortRef)?
            .set_reference_target(&pp_elem)?;

        for st in self.signal_triggerings() {
            st.connect_to_ecu(ecu, direction)?;
        }

        Ok(IPduPort(pp_elem))
    }

    /// create an iterator over the `IPduPorts` that are connected to this `PduTriggering`
    pub fn pdu_ports(&self) -> impl Iterator<Item = IPduPort> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::IPduPortRefs)
            .into_iter()
            .flat_map(|ipprefs| ipprefs.sub_elements())
            .filter_map(|ippref| {
                ippref
                    .get_reference_target()
                    .ok()
                    .and_then(|elem| IPduPort::try_from(elem).ok())
            })
    }

    /// create an iterator over the `ISignalTriggerings` that are triggered by this `PduTriggering`
    pub fn signal_triggerings(&self) -> impl Iterator<Item = ISignalTriggering> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::ISignalTriggerings)
            .into_iter()
            .flat_map(|ists| ists.sub_elements())
            .filter_map(|ist| {
                ist.get_sub_element(ElementName::ISignalTriggeringRef)
                    .and_then(|str| str.get_reference_target().ok())
                    .and_then(|elem| ISignalTriggering::try_from(elem).ok())
            })
    }

    /// add a signal triggering for a signal to this `PduTriggering`
    pub fn add_signal_triggering(&self, signal: &ISignal) -> Result<ISignalTriggering, AutosarAbstractionError> {
        let channel = self.physical_channel()?;
        let st = ISignalTriggering::new(signal, &channel)?;
        let triggerings = self
            .element()
            .get_or_create_sub_element(ElementName::ISignalTriggerings)?;
        triggerings
            .create_sub_element(ElementName::ISignalTriggeringRefConditional)?
            .create_sub_element(ElementName::ISignalTriggeringRef)?
            .set_reference_target(st.element())?;

        for pdu_port in self.pdu_ports() {
            if let (Some(ecu), Some(direction)) = (pdu_port.ecu(), pdu_port.communication_direction()) {
                st.connect_to_ecu(&ecu, direction)?;
            }
        }

        Ok(st)
    }

    /// add a signal triggering for a signal group to this `PduTriggering`
    pub fn add_signal_group_triggering(
        &self,
        signal_group: &ISignalGroup,
    ) -> Result<ISignalTriggering, AutosarAbstractionError> {
        let channel = self.physical_channel()?;
        let st = ISignalTriggering::new_group(signal_group, &channel)?;
        let triggerings = self
            .element()
            .get_or_create_sub_element(ElementName::ISignalTriggerings)?;
        triggerings
            .create_sub_element(ElementName::ISignalTriggeringRefConditional)?
            .create_sub_element(ElementName::ISignalTriggeringRef)?
            .set_reference_target(st.element())?;

        for pdu_port in self.pdu_ports() {
            if let (Some(ecu), Some(direction)) = (pdu_port.ecu(), pdu_port.communication_direction()) {
                st.connect_to_ecu(&ecu, direction)?;
            }
        }

        Ok(st)
    }
}

//##################################################################

/// The `IPduPort` allows an ECU to send or receive a PDU
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IPduPort(Element);
abstraction_element!(IPduPort, IPduPort);

impl IPduPort {
    /// get the ECU instance that contains this `IPduPort`
    #[must_use]
    pub fn ecu(&self) -> Option<EcuInstance> {
        let comm_connector_elem = self.element().named_parent().ok()??;
        let ecu_elem = comm_connector_elem.named_parent().ok()??;
        EcuInstance::try_from(ecu_elem).ok()
    }

    /// get the communication direction of this `IPduPort`
    #[must_use]
    pub fn communication_direction(&self) -> Option<CommunicationDirection> {
        self.element()
            .get_sub_element(ElementName::CommunicationDirection)?
            .character_data()?
            .enum_value()?
            .try_into()
            .ok()
    }
}

//##################################################################

/// The collction trigger defines whether a Pdu contributes to the triggering
/// of the data transmission if Pdu collection is enabled
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PduCollectionTrigger {
    /// Pdu will trigger the transmission of the data.
    Always,
    /// Pdu will be buffered and will not trigger the transmission of the data
    Never,
}

impl From<PduCollectionTrigger> for EnumItem {
    fn from(value: PduCollectionTrigger) -> Self {
        match value {
            PduCollectionTrigger::Always => EnumItem::Always,
            PduCollectionTrigger::Never => EnumItem::Never,
        }
    }
}

impl TryFrom<EnumItem> for PduCollectionTrigger {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<Self, Self::Error> {
        match value {
            EnumItem::Always => Ok(PduCollectionTrigger::Always),
            EnumItem::Never => Ok(PduCollectionTrigger::Never),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "PduCollectionTrigger".to_string(),
            }),
        }
    }
}

//##################################################################

reflist_iterator!(PduTriggeringsIterator, PduTriggering);

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        communication::{
            AbstractFrame, AbstractFrameTriggering, CanAddressingMode, CanClusterSettings, CanFrameType,
            TransferProperty,
        },
        ByteOrder, SystemCategory,
    };
    use autosar_data::{AutosarModel, AutosarVersion};

    #[test]
    fn test_pdus() {
        let model = AutosarModel::new();
        let _file = model.create_file("filename", AutosarVersion::Autosar_00048).unwrap();
        let package = ArPackage::get_or_create(&model, "/pkg").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();

        let isignal_ipdu = system.create_isignal_ipdu("isignal_ipdu", &package, 1).unwrap();
        let nm_pdu = system.create_nm_pdu("nm_pdu", &package, 1).unwrap();
        let n_pdu = system.create_n_pdu("n_pdu", &package, 1).unwrap();
        let dcm_ipdu = system.create_dcm_ipdu("dcm_ipdu", &package, 1).unwrap();
        let gp_pdu = system
            .create_general_purpose_pdu("gp_pdu", &package, 1, GeneralPurposePduCategory::Sd)
            .unwrap();
        let gp_ipdu = system
            .create_general_purpose_ipdu("gp_ipdu", &package, 1, GeneralPurposeIPduCategory::Xcp)
            .unwrap();
        let container_ipdu = system.create_container_ipdu("container_ipdu", &package, 1).unwrap();
        let secured_ipdu = system.create_secured_ipdu("secured_ipdu", &package, 1).unwrap();
        let multiplexed_ipdu = system.create_multiplexed_ipdu("multiplexed_ipdu", &package, 1).unwrap();

        let frame = system.create_flexray_frame("frame1", &package, 64).unwrap();
        frame
            .map_pdu(&isignal_ipdu, 0, ByteOrder::MostSignificantByteLast, None)
            .unwrap();
        frame
            .map_pdu(&nm_pdu, 8, ByteOrder::MostSignificantByteLast, None)
            .unwrap();
        frame
            .map_pdu(&n_pdu, 16, ByteOrder::MostSignificantByteLast, None)
            .unwrap();
        frame
            .map_pdu(&dcm_ipdu, 24, ByteOrder::MostSignificantByteLast, None)
            .unwrap();
        frame
            .map_pdu(&gp_pdu, 32, ByteOrder::MostSignificantByteLast, None)
            .unwrap();
        frame
            .map_pdu(&gp_ipdu, 40, ByteOrder::MostSignificantByteLast, None)
            .unwrap();
        frame
            .map_pdu(&container_ipdu, 48, ByteOrder::MostSignificantByteLast, None)
            .unwrap();
        frame
            .map_pdu(&secured_ipdu, 56, ByteOrder::MostSignificantByteLast, None)
            .unwrap();
        frame
            .map_pdu(&multiplexed_ipdu, 64, ByteOrder::MostSignificantByteLast, None)
            .unwrap();

        let mut pdus_iter = frame.mapped_pdus();
        assert_eq!(pdus_iter.next().unwrap().name().unwrap(), "isignal_ipdu");
        assert_eq!(pdus_iter.next().unwrap().name().unwrap(), "nm_pdu");
        assert_eq!(pdus_iter.next().unwrap().name().unwrap(), "n_pdu");
        assert_eq!(pdus_iter.next().unwrap().name().unwrap(), "dcm_ipdu");
        assert_eq!(pdus_iter.next().unwrap().name().unwrap(), "gp_pdu");
        assert_eq!(pdus_iter.next().unwrap().name().unwrap(), "gp_ipdu");
        assert_eq!(pdus_iter.next().unwrap().name().unwrap(), "container_ipdu");
        assert_eq!(pdus_iter.next().unwrap().name().unwrap(), "secured_ipdu");
        assert_eq!(pdus_iter.next().unwrap().name().unwrap(), "multiplexed_ipdu");
        assert!(pdus_iter.next().is_none());
    }

    #[test]
    fn test_pdu_triggering() {
        let model = AutosarModel::new();
        let _file = model.create_file("filename", AutosarVersion::Autosar_00048).unwrap();
        let package = ArPackage::get_or_create(&model, "/pkg").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();

        // create an ISignalIPdu with a signal
        let isignal_ipdu = system.create_isignal_ipdu("isignal_ipdu", &package, 1).unwrap();
        let syssignal = package.create_system_signal("syssignal").unwrap();
        let isignal = system.create_isignal("isignal", &package, 1, &syssignal, None).unwrap();
        isignal_ipdu
            .map_signal(
                &isignal,
                0,
                ByteOrder::MostSignificantByteLast,
                None,
                TransferProperty::Triggered,
            )
            .unwrap();
        // create an ISignalGroup with a second signal
        let syssignal_group = package.create_system_signal_group("syssignal_group").unwrap();
        let isignal_group = system
            .create_isignal_group("isignal_group", &package, &syssignal_group)
            .unwrap();
        let syssignal2 = package.create_system_signal("syssignal2").unwrap();
        let isignal2 = system
            .create_isignal("isignal2", &package, 1, &syssignal2, None)
            .unwrap();
        isignal_ipdu.map_signal_group(&isignal_group).unwrap();
        isignal_ipdu
            .map_signal(
                &isignal2,
                1,
                ByteOrder::MostSignificantByteLast,
                None,
                TransferProperty::Triggered,
            )
            .unwrap();

        // create a frame and map the ISignalIPdu to it
        let can_cluster = system
            .create_can_cluster("Cluster", &package, &CanClusterSettings::default())
            .unwrap();
        let channel = can_cluster.create_physical_channel("Channel").unwrap();
        let frame = system.create_can_frame("frame", &package, 8).unwrap();
        let frame_triggering = channel
            .trigger_frame(&frame, 0x123, CanAddressingMode::Standard, CanFrameType::Can20)
            .unwrap();
        let _mapping = frame
            .map_pdu(&isignal_ipdu, 0, ByteOrder::MostSignificantByteLast, None)
            .unwrap();

        // create an EcuInstance, and connect it to the channel. The frame is reeived by the ECU
        let ecu = system.create_ecu_instance("ecu", &package).unwrap();
        let controller = ecu.create_can_communication_controller("controller").unwrap();
        controller.connect_physical_channel("connection", &channel).unwrap();
        frame_triggering
            .connect_to_ecu(&ecu, CommunicationDirection::In)
            .unwrap();

        let pdu_triggering = frame_triggering.pdu_triggerings().next().unwrap();
        assert_eq!(pdu_triggering.pdu_ports().count(), 1);
        assert_eq!(pdu_triggering.signal_triggerings().count(), 3); // one for each signal, and another for the signal group

        let pdu_port = pdu_triggering.pdu_ports().next().unwrap();
        assert_eq!(pdu_port.ecu().unwrap().name().unwrap(), "ecu");
        assert_eq!(pdu_port.communication_direction().unwrap(), CommunicationDirection::In);
    }

    #[test]
    fn general_purpose_pdu() {
        let model = AutosarModel::new();
        let _file = model.create_file("filename", AutosarVersion::Autosar_00048).unwrap();
        let package = ArPackage::get_or_create(&model, "/pkg").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();

        let gp_pdu1 = system
            .create_general_purpose_pdu("gp_pdu1", &package, 1, GeneralPurposePduCategory::Sd)
            .unwrap();
        assert_eq!(gp_pdu1.category().unwrap(), GeneralPurposePduCategory::Sd);

        let gp_pdu2 = system
            .create_general_purpose_pdu("gp_pdu2", &package, 1, GeneralPurposePduCategory::GlobalTime)
            .unwrap();
        assert_eq!(gp_pdu2.category().unwrap(), GeneralPurposePduCategory::GlobalTime);

        let gp_pdu3 = system
            .create_general_purpose_pdu("gp_pdu3", &package, 1, GeneralPurposePduCategory::DoIp)
            .unwrap();
        assert_eq!(gp_pdu3.category().unwrap(), GeneralPurposePduCategory::DoIp);

        // conversion of category to string and back
        assert_eq!(
            GeneralPurposePduCategory::from_str("SD").unwrap(),
            GeneralPurposePduCategory::Sd
        );
        assert_eq!(
            GeneralPurposePduCategory::from_str("GLOBAL_TIME").unwrap(),
            GeneralPurposePduCategory::GlobalTime
        );
        assert_eq!(
            GeneralPurposePduCategory::from_str("DOIP").unwrap(),
            GeneralPurposePduCategory::DoIp
        );
        assert!(GeneralPurposePduCategory::from_str("invalid").is_err());
    }

    #[test]
    fn create_general_purpose_ipdu() {
        let model = AutosarModel::new();
        let _file = model.create_file("filename", AutosarVersion::Autosar_00048).unwrap();
        let package = ArPackage::get_or_create(&model, "/pkg").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();

        let gp_ipdu1 = system
            .create_general_purpose_ipdu("gp_ipdu1", &package, 1, GeneralPurposeIPduCategory::Xcp)
            .unwrap();
        assert_eq!(gp_ipdu1.category().unwrap(), GeneralPurposeIPduCategory::Xcp);

        let gp_ipdu2 = system
            .create_general_purpose_ipdu("gp_ipdu2", &package, 1, GeneralPurposeIPduCategory::SomeipSegmentedIpdu)
            .unwrap();
        assert_eq!(
            gp_ipdu2.category().unwrap(),
            GeneralPurposeIPduCategory::SomeipSegmentedIpdu
        );

        let gp_ipdu3 = system
            .create_general_purpose_ipdu("gp_ipdu3", &package, 1, GeneralPurposeIPduCategory::Dlt)
            .unwrap();
        assert_eq!(gp_ipdu3.category().unwrap(), GeneralPurposeIPduCategory::Dlt);

        // conversion of category to string and back
        assert_eq!(
            GeneralPurposeIPduCategory::from_str("XCP").unwrap(),
            GeneralPurposeIPduCategory::Xcp
        );
        assert_eq!(
            GeneralPurposeIPduCategory::from_str("SOMEIP_SEGMENTED_IPDU").unwrap(),
            GeneralPurposeIPduCategory::SomeipSegmentedIpdu
        );
        assert_eq!(
            GeneralPurposeIPduCategory::from_str("DLT").unwrap(),
            GeneralPurposeIPduCategory::Dlt
        );
        assert!(GeneralPurposeIPduCategory::from_str("invalid").is_err());
    }
}
