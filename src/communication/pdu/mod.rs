use crate::communication::{
    AbstractPhysicalChannel, CommunicationDirection, ISignal, ISignalGroup, ISignalTriggering, PduToFrameMapping,
    PhysicalChannel, SoConIPduIdentifier, SomeipTpConnection, TransferProperty,
};
use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, ByteOrder, EcuInstance, IdentifiableAbstractionElement,
    abstraction_element, get_reference_parents, is_used_system_element, make_unique_name,
};
use autosar_data::{AutosarDataError, Element, ElementName, EnumItem};
use std::str::FromStr;

mod container_ipdu;
mod isignal_ipdu;
mod secured_ipdu;

pub use container_ipdu::*;
pub use isignal_ipdu::*;
pub use secured_ipdu::*;

//##################################################################

/// This trait is implemented by all Pdus
pub trait AbstractPdu: AbstractionElement + Into<Pdu> {
    /// set the length of the PDU
    fn set_length(&self, length: u32) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::Length)?
            .set_character_data(length as u64)?;
        Ok(())
    }

    /// get the length of the PDU
    fn length(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::Length)?
            .character_data()?
            .parse_integer()
    }

    /// list all `PduTriggerings` that trigger this PDU
    fn pdu_triggerings(&self) -> Vec<PduTriggering> {
        let model_result = self.element().model();
        let path_result = self.element().path();
        if let (Ok(model), Ok(path)) = (model_result, path_result) {
            model
                .get_references_to(&path)
                .iter()
                .filter_map(|e| {
                    e.upgrade()
                        .and_then(|ref_elem| ref_elem.named_parent().ok().flatten())
                        .and_then(|elem| PduTriggering::try_from(elem).ok())
                })
                .collect()
        } else {
            vec![]
        }
    }
}

//##################################################################

/// for now this is a marker trait to identify `IPdus`
pub trait AbstractIpdu: AbstractPdu + Into<IPdu> {
    /// set the `ContainedIPduProps` for this `IPdu`
    ///
    /// This is only relevant for `IPdus` that will be transmitted in `ContainerIPdus`
    fn set_contained_ipdu_props(&self, props: Option<&ContainedIPduProps>) -> Result<(), AutosarAbstractionError> {
        ContainedIPduProps::set_props(self.element(), props)
    }

    /// get the `ContainedIPduProps` for this `IPdu`
    #[must_use]
    fn contained_ipdu_props(&self) -> Option<ContainedIPduProps> {
        ContainedIPduProps::get_props(self.element())
    }
}

//##################################################################

/// Network Management Pdu
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NmPdu(Element);
abstraction_element!(NmPdu, NmPdu);
impl IdentifiableAbstractionElement for NmPdu {}

impl NmPdu {
    pub(crate) fn new(name: &str, package: &ArPackage, length: u32) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_pdu = pkg_elements.create_named_sub_element(ElementName::NmPdu, name)?;
        elem_pdu
            .create_sub_element(ElementName::Length)?
            .set_character_data(length.to_string())?;

        Ok(Self(elem_pdu))
    }

    /// remove this `NmPdu` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        // remove all triggerings of this PDU
        for pdu_triggering in self.pdu_triggerings() {
            let _ = pdu_triggering.element().remove_sub_element_kind(ElementName::IPduRef);
            let _ = pdu_triggering.remove(deep);
        }

        for signal_mapping in self.mapped_signals() {
            let _ = signal_mapping.remove(deep);
        }

        let ref_parents = get_reference_parents(self.element())?;

        AbstractionElement::remove(self, deep)?;

        for (named_parent, _parent) in ref_parents {
            if named_parent.element_name() == ElementName::PduToFrameMapping
                && let Ok(pdu_to_frame_mapping) = PduToFrameMapping::try_from(named_parent)
            {
                pdu_to_frame_mapping.remove(deep)?;
            }
        }

        Ok(())
    }

    /// set the unused bit pattern for this NmPdu
    pub fn set_unused_bit_pattern(&self, pattern: u8) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::UnusedBitPattern)?
            .set_character_data(pattern.to_string())?;
        Ok(())
    }

    /// get the unused bit pattern for this NmPdu
    #[must_use]
    pub fn unused_bit_pattern(&self) -> Option<u8> {
        self.element()
            .get_sub_element(ElementName::UnusedBitPattern)?
            .character_data()?
            .parse_integer()
    }

    /// map a signal to the `ISignalIPdu`
    ///
    /// If this signal is part of a signal group, then the group must be mapped first
    pub fn map_signal(
        &self,
        signal: &ISignal,
        start_position: u32,
        byte_order: ByteOrder,
        update_bit: Option<u32>,
        transfer_property: TransferProperty,
    ) -> Result<ISignalToIPduMapping, AutosarAbstractionError> {
        let signal_name = signal
            .name()
            .ok_or(AutosarAbstractionError::InvalidParameter("invalid signal".to_string()))?;

        verify_signal_mapping(self, signal, start_position, byte_order, update_bit, &signal_name)?;

        // add a pdu triggering for the newly mapped PDU to each frame triggering of this frame
        for pt in self.pdu_triggerings() {
            let st = pt.create_signal_triggering(signal)?;
            for pdu_port in pt.pdu_ports() {
                if let (Ok(ecu), Some(direction)) = (pdu_port.ecu(), pdu_port.communication_direction()) {
                    st.connect_to_ecu(&ecu, direction)?;
                }
            }
        }

        // create and return the new mapping
        let model = self.element().model()?;
        let base_path = self.element().path()?;
        let name = make_unique_name(&model, &base_path, &signal_name);

        // the crucial difference between NmPdu and ISignalIPdu is here
        // NmPdu uses ISignalToIPduMapping, while ISignalIPdu uses ISignalToPduMapping
        let mappings = self
            .element()
            .get_or_create_sub_element(ElementName::ISignalToIPduMappings)?;

        ISignalToIPduMapping::new_with_signal(
            &name,
            &mappings,
            signal,
            start_position,
            byte_order,
            update_bit,
            transfer_property,
        )
    }

    /// map a signal group to the PDU
    pub fn map_signal_group(
        &self,
        signal_group: &ISignalGroup,
    ) -> Result<ISignalToIPduMapping, AutosarAbstractionError> {
        let signal_group_name = signal_group.name().ok_or(AutosarAbstractionError::InvalidParameter(
            "invalid signal group".to_string(),
        ))?;

        // add a pdu triggering for the newly mapped PDU to each frame triggering of this frame
        for pt in self.pdu_triggerings() {
            let st = pt.create_signal_group_triggering(signal_group)?;
            for pdu_port in pt.pdu_ports() {
                if let (Ok(ecu), Some(direction)) = (pdu_port.ecu(), pdu_port.communication_direction()) {
                    st.connect_to_ecu(&ecu, direction)?;
                }
            }
        }

        // create and return the new mapping
        let model = self.element().model()?;
        let base_path = self.element().path()?;
        let name = make_unique_name(&model, &base_path, &signal_group_name);

        // the crucial difference between NmPdu and ISignalIPdu is here
        // NmPdu uses ISignalToIPduMapping, while ISignalIPdu uses ISignalToPduMapping
        let mappings = self
            .element()
            .get_or_create_sub_element(ElementName::ISignalToIPduMappings)?;

        ISignalToIPduMapping::new_with_group(&name, &mappings, signal_group)
    }
}

impl AbstractPdu for NmPdu {}

impl SignalPdu for NmPdu {
    /// returns an iterator over all signals and signal groups mapped to the PDU
    fn mapped_signals(&self) -> impl Iterator<Item = ISignalToIPduMapping> + Send + use<> {
        self.element()
            .get_sub_element(ElementName::ISignalToIPduMappings)
            .into_iter()
            .flat_map(|mappings| mappings.sub_elements())
            .filter_map(|elem| ISignalToIPduMapping::try_from(elem).ok())
    }

    fn map_signal(
        &self,
        signal: &ISignal,
        start_position: u32,
        byte_order: ByteOrder,
        update_bit: Option<u32>,
        transfer_property: TransferProperty,
    ) -> Result<ISignalToIPduMapping, AutosarAbstractionError> {
        NmPdu::map_signal(self, signal, start_position, byte_order, update_bit, transfer_property)
    }

    fn map_signal_group(&self, signal_group: &ISignalGroup) -> Result<ISignalToIPduMapping, AutosarAbstractionError> {
        NmPdu::map_signal_group(self, signal_group)
    }
}

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
impl IdentifiableAbstractionElement for NPdu {}

impl NPdu {
    pub(crate) fn new(name: &str, package: &ArPackage, length: u32) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_pdu = pkg_elements.create_named_sub_element(ElementName::NPdu, name)?;
        elem_pdu
            .create_sub_element(ElementName::Length)?
            .set_character_data(length.to_string())?;

        Ok(Self(elem_pdu))
    }

    /// remove this `NPdu` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        // remove all triggerings of this PDU
        for pdu_triggering in self.pdu_triggerings() {
            let _ = pdu_triggering.element().remove_sub_element_kind(ElementName::IPduRef);
            let _ = pdu_triggering.remove(deep);
        }

        let ref_parents = get_reference_parents(self.element())?;

        AbstractionElement::remove(self, deep)?;

        for (named_parent, _parent) in ref_parents {
            if named_parent.element_name() == ElementName::PduToFrameMapping
                && let Ok(pdu_to_frame_mapping) = PduToFrameMapping::try_from(named_parent)
            {
                pdu_to_frame_mapping.remove(deep)?;
            }
        }

        Ok(())
    }
}

impl AbstractPdu for NPdu {}

impl AbstractIpdu for NPdu {}

impl From<NPdu> for Pdu {
    fn from(value: NPdu) -> Self {
        Pdu::NPdu(value)
    }
}

impl From<NPdu> for IPdu {
    fn from(value: NPdu) -> Self {
        IPdu::NPdu(value)
    }
}

//##################################################################

/// Represents the `IPdus` handled by Dcm
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DcmIPdu(Element);
abstraction_element!(DcmIPdu, DcmIPdu);
impl IdentifiableAbstractionElement for DcmIPdu {}

impl DcmIPdu {
    pub(crate) fn new(
        name: &str,
        package: &ArPackage,
        length: u32,
        diag_pdu_type: DiagPduType,
    ) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_pdu = pkg_elements.create_named_sub_element(ElementName::DcmIPdu, name)?;
        elem_pdu
            .create_sub_element(ElementName::Length)?
            .set_character_data(length.to_string())?;
        let dcm_ipdu = Self(elem_pdu);
        dcm_ipdu.set_diag_pdu_type(diag_pdu_type)?;

        Ok(dcm_ipdu)
    }

    /// remove this `DcmIPdu` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        // remove all triggerings of this PDU
        for pdu_triggering in self.pdu_triggerings() {
            let _ = pdu_triggering.element().remove_sub_element_kind(ElementName::IPduRef);
            let _ = pdu_triggering.remove(deep);
        }

        let ref_parents = get_reference_parents(self.element())?;

        AbstractionElement::remove(self, deep)?;

        for (named_parent, _parent) in ref_parents {
            if named_parent.element_name() == ElementName::PduToFrameMapping
                && let Ok(pdu_to_frame_mapping) = PduToFrameMapping::try_from(named_parent)
            {
                pdu_to_frame_mapping.remove(deep)?;
            }
        }

        Ok(())
    }

    /// set the type of this DcmIPdu
    pub fn set_diag_pdu_type(&self, pdu_type: DiagPduType) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::DiagPduType)?
            .set_character_data::<EnumItem>(pdu_type.into())?;
        Ok(())
    }

    /// get the type of this DcmIPdu
    #[must_use]
    pub fn diag_pdu_type(&self) -> Option<DiagPduType> {
        let enum_item = self
            .element()
            .get_sub_element(ElementName::DiagPduType)?
            .character_data()?
            .enum_value()?;
        DiagPduType::try_from(enum_item).ok()
    }
}

impl AbstractPdu for DcmIPdu {}

impl AbstractIpdu for DcmIPdu {}

impl From<DcmIPdu> for Pdu {
    fn from(value: DcmIPdu) -> Self {
        Pdu::DcmIPdu(value)
    }
}

impl From<DcmIPdu> for IPdu {
    fn from(value: DcmIPdu) -> Self {
        IPdu::DcmIPdu(value)
    }
}

//##################################################################

/// The type of a DcmIPdu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagPduType {
    /// Diagnostic Request
    DiagRequest,
    /// Diagnostic Response
    DiagResponse,
}

impl TryFrom<EnumItem> for DiagPduType {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<Self, Self::Error> {
        match value {
            EnumItem::DiagRequest => Ok(DiagPduType::DiagRequest),
            EnumItem::DiagResponse => Ok(DiagPduType::DiagResponse),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "DiagPduType".to_string(),
            }),
        }
    }
}

impl From<DiagPduType> for EnumItem {
    fn from(value: DiagPduType) -> Self {
        match value {
            DiagPduType::DiagRequest => EnumItem::DiagRequest,
            DiagPduType::DiagResponse => EnumItem::DiagResponse,
        }
    }
}

//##################################################################

/// This element is used for AUTOSAR Pdus without additional attributes that are routed by a bus interface
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GeneralPurposePdu(Element);
abstraction_element!(GeneralPurposePdu, GeneralPurposePdu);
impl IdentifiableAbstractionElement for GeneralPurposePdu {}

impl GeneralPurposePdu {
    pub(crate) fn new(
        name: &str,
        package: &ArPackage,
        length: u32,
        category: GeneralPurposePduCategory,
    ) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let pdu_elem = pkg_elements.create_named_sub_element(ElementName::GeneralPurposePdu, name)?;
        let pdu = Self(pdu_elem);

        pdu.set_length(length)?;
        pdu.set_category(category)?;

        Ok(pdu)
    }

    /// remove this `GeneralPurposePdu` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        // remove all triggerings of this PDU
        for pdu_triggering in self.pdu_triggerings() {
            let _ = pdu_triggering.element().remove_sub_element_kind(ElementName::IPduRef);
            let _ = pdu_triggering.remove(deep);
        }

        let ref_parents = get_reference_parents(self.element())?;

        AbstractionElement::remove(self, deep)?;

        for (named_parent, _parent) in ref_parents {
            if named_parent.element_name() == ElementName::PduToFrameMapping
                && let Ok(pdu_to_frame_mapping) = PduToFrameMapping::try_from(named_parent)
            {
                pdu_to_frame_mapping.remove(deep)?;
            }
        }

        Ok(())
    }

    /// set the category of this PDU
    pub fn set_category(&self, category: GeneralPurposePduCategory) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::Category)?
            .set_character_data(category.to_string())?;
        Ok(())
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
impl IdentifiableAbstractionElement for GeneralPurposeIPdu {}

impl GeneralPurposeIPdu {
    pub(crate) fn new(
        name: &str,
        package: &ArPackage,
        length: u32,
        category: GeneralPurposeIPduCategory,
    ) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let pdu_elem = pkg_elements.create_named_sub_element(ElementName::GeneralPurposeIPdu, name)?;
        let pdu = Self(pdu_elem);

        pdu.set_length(length)?;
        pdu.set_category(category)?;

        Ok(pdu)
    }

    /// remove this `GeneralPurposeIPdu` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        // remove all triggerings of this PDU
        for pdu_triggering in self.pdu_triggerings() {
            let _ = pdu_triggering.element().remove_sub_element_kind(ElementName::IPduRef);
            let _ = pdu_triggering.remove(deep);
        }

        let ref_parents = get_reference_parents(self.element())?;

        AbstractionElement::remove(self, deep)?;

        for (named_parent, _parent) in ref_parents {
            if named_parent.element_name() == ElementName::PduToFrameMapping
                && let Ok(pdu_to_frame_mapping) = PduToFrameMapping::try_from(named_parent)
            {
                pdu_to_frame_mapping.remove(deep)?;
            }
        }

        Ok(())
    }

    /// set the category of this PDU
    pub fn set_category(&self, category: GeneralPurposeIPduCategory) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::Category)?
            .set_character_data(category.to_string())?;
        Ok(())
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

impl From<GeneralPurposeIPdu> for IPdu {
    fn from(value: GeneralPurposeIPdu) -> Self {
        IPdu::GeneralPurposeIPdu(value)
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

/// The multiplexed pdu contains one of serveral signal pdus
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MultiplexedIPdu(Element);
abstraction_element!(MultiplexedIPdu, MultiplexedIPdu);
impl IdentifiableAbstractionElement for MultiplexedIPdu {}

impl MultiplexedIPdu {
    pub(crate) fn new(name: &str, package: &ArPackage, length: u32) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_pdu = pkg_elements.create_named_sub_element(ElementName::MultiplexedIPdu, name)?;
        elem_pdu
            .create_sub_element(ElementName::Length)?
            .set_character_data(length.to_string())?;

        Ok(Self(elem_pdu))
    }

    /// remove this `GeneralPurposeIPdu` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        // remove all triggerings of this PDU
        for pdu_triggering in self.pdu_triggerings() {
            let _ = pdu_triggering.element().remove_sub_element_kind(ElementName::IPduRef);
            let _ = pdu_triggering.remove(deep);
        }

        let opt_static_pdu = self.static_part();

        for dynamic_part in self.dynamic_part_alternatives() {
            dynamic_part.remove(deep)?;
        }

        let ref_parents = get_reference_parents(self.element())?;

        AbstractionElement::remove(self, deep)?;

        for (named_parent, _parent) in ref_parents {
            if named_parent.element_name() == ElementName::PduToFrameMapping
                && let Ok(pdu_to_frame_mapping) = PduToFrameMapping::try_from(named_parent)
            {
                pdu_to_frame_mapping.remove(deep)?;
            }
        }

        if let Some(ipdu) = opt_static_pdu {
            let mut triggerings = ipdu.pdu_triggerings();
            // because the multiplexed ipdu does not keep a reference to the pdu triggering of it's parts,
            // we don't know which PDU triggering to remove if there is more than one
            if triggerings.len() == 1
                && let Some(pdu_triggering) = triggerings.pop()
            {
                pdu_triggering.remove(deep)?;
            }
        }

        Ok(())
    }

    /// set the ISignalIPdu containing the static part of this multiplexed ipdu
    pub fn set_static_part(&self, static_ipdu: &ISignalIPdu) -> Result<(), AutosarAbstractionError> {
        let prev_static_part = self.static_part();

        self.element()
            .get_or_create_sub_element(ElementName::StaticParts)?
            .get_or_create_sub_element(ElementName::StaticPart)?
            .get_or_create_sub_element(ElementName::IPduRef)?
            .set_reference_target(static_ipdu.element())?;

        self.update_pdu_triggerings(prev_static_part.as_ref(), static_ipdu)?;

        Ok(())
    }

    /// get the ISignalIPdu containing the static part of this multiplexed ipdu
    #[must_use]
    pub fn static_part(&self) -> Option<ISignalIPdu> {
        let ipdu_elem = self
            .element()
            .get_sub_element(ElementName::StaticParts)?
            .get_sub_element(ElementName::StaticPart)?
            .get_sub_element(ElementName::IPduRef)?
            .get_reference_target()
            .ok()?;
        ISignalIPdu::try_from(ipdu_elem).ok()
    }

    /// add a dynamic part alternative to this multiplexed ipdu
    /// `selector_code` is the value of the multiplexor that selects this dynamic part
    /// `initial_dynamic_part` indicates whether this is the initial dynamic part; only one dynamic part can be initial
    ///
    /// All dynamic parts must have the same length
    pub fn add_dynamic_part(
        &self,
        dynamic_ipdu: &ISignalIPdu,
        selector_code: u16,
        initial_dynamic_part: bool,
    ) -> Result<DynamicPartAlternative, AutosarAbstractionError> {
        let dp_alternatives = self
            .element()
            .get_or_create_sub_element(ElementName::DynamicParts)?
            .get_or_create_sub_element(ElementName::DynamicPart)?
            .get_or_create_sub_element(ElementName::DynamicPartAlternatives)?;

        DynamicPartAlternative::new(&dp_alternatives, dynamic_ipdu, selector_code, initial_dynamic_part)
    }

    /// list all dynamic part alternatives of this multiplexed ipdu
    pub fn dynamic_part_alternatives(&self) -> impl Iterator<Item = DynamicPartAlternative> + Send + use<> {
        let dp_alternatives_elem = self
            .element()
            .get_sub_element(ElementName::DynamicParts)
            .and_then(|e| e.get_sub_element(ElementName::DynamicPart))
            .and_then(|e| e.get_sub_element(ElementName::DynamicPartAlternatives));

        dp_alternatives_elem
            .into_iter()
            .flat_map(|e| e.sub_elements())
            .filter_map(|elem| DynamicPartAlternative::try_from(elem).ok())
    }

    /// set the selector field of this multiplexed ipdu
    ///
    /// The selector field should exist as a signal in each dynamic part
    pub fn set_selector_field(
        &self,
        length: u8,
        start_position: u32,
        byte_order: ByteOrder,
    ) -> Result<(), AutosarAbstractionError> {
        if length == 0 || length > 16 {
            return Err(AutosarAbstractionError::InvalidParameter(
                "selector field length must be between 1 and 16".to_string(),
            ));
        }
        self.element()
            .get_or_create_sub_element(ElementName::SelectorFieldLength)?
            .set_character_data(length as u64)?;
        self.element()
            .get_or_create_sub_element(ElementName::SelectorFieldStartPosition)?
            .set_character_data(start_position as u64)?;
        self.element()
            .get_or_create_sub_element(ElementName::SelectorFieldByteOrder)?
            .set_character_data::<EnumItem>(byte_order.into())?;
        Ok(())
    }

    /// get the selector field of this multiplexed ipdu
    pub fn selector_field(&self) -> Option<(u8, u32, ByteOrder)> {
        let length = self
            .element()
            .get_sub_element(ElementName::SelectorFieldLength)?
            .character_data()?
            .parse_integer()?;
        let start_position = self
            .element()
            .get_sub_element(ElementName::SelectorFieldStartPosition)?
            .character_data()?
            .parse_integer()?;
        let byte_order_enum = self
            .element()
            .get_sub_element(ElementName::SelectorFieldByteOrder)?
            .character_data()?
            .enum_value()?;
        let byte_order = ByteOrder::try_from(byte_order_enum).ok()?;
        Some((length, start_position, byte_order))
    }

    // Update the pdu triggerings when the static part or a dynamic part is changed
    // Also used when the multiplexed ipdu is newly mapped to a frame
    pub(crate) fn update_pdu_triggerings(
        &self,
        old_ipdu: Option<&ISignalIPdu>,
        new_ipdu: &ISignalIPdu,
    ) -> Result<(), AutosarAbstractionError> {
        // note: usually, the MultiplexedIPdu is triggered exactly once, and thus the pdu_triggerings list has exactly one entry
        // remove the pdu triggering(s) of the previous ipdu
        if let Some(old_ipdu) = old_ipdu {
            for pt in old_ipdu.pdu_triggerings() {
                pt.remove(false)?;
            }
        }

        // create pdu triggerings for the new ipdu part in channels where this multiplexed ipdu is triggered
        for multiplex_pt in self.pdu_triggerings() {
            if let Ok(channel) = multiplex_pt.physical_channel() {
                let new_pt = PduTriggering::new(&Pdu::ISignalIPdu(new_ipdu.clone()), &channel)?;
                for pp in multiplex_pt.pdu_ports() {
                    if let (Ok(ecu), Some(direction)) = (pp.ecu(), pp.communication_direction()) {
                        let _ = new_pt.create_pdu_port(&ecu, direction);
                    }
                }
            }
        }

        Ok(())
    }
}

impl AbstractPdu for MultiplexedIPdu {}

impl AbstractIpdu for MultiplexedIPdu {}

impl From<MultiplexedIPdu> for Pdu {
    fn from(value: MultiplexedIPdu) -> Self {
        Pdu::MultiplexedIPdu(value)
    }
}

impl From<MultiplexedIPdu> for IPdu {
    fn from(value: MultiplexedIPdu) -> Self {
        IPdu::MultiplexedIPdu(value)
    }
}

//##################################################################

/// A dynamic part alternative of a multiplexed PDU
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DynamicPartAlternative(Element);
abstraction_element!(DynamicPartAlternative, DynamicPartAlternative);

impl DynamicPartAlternative {
    fn new(
        parent: &Element,
        dynamic_ipdu: &ISignalIPdu,
        selector_code: u16,
        initial_dynamic_part: bool,
    ) -> Result<Self, AutosarAbstractionError> {
        let dp_alt_elem = parent.create_sub_element(ElementName::DynamicPartAlternative)?;
        let dp_alt = Self(dp_alt_elem);
        dp_alt.set_ipdu(dynamic_ipdu)?;
        dp_alt.set_selector_field_code(selector_code)?;
        dp_alt.set_initial_dynamic_part(initial_dynamic_part)?;

        Ok(dp_alt)
    }

    /// remove this dynamic part alternative from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        // remove all triggerings of this PDU
        let opt_ipdu = self.ipdu();

        AbstractionElement::remove(self, deep)?;

        if let Some(ipdu) = opt_ipdu {
            let mut triggerings = ipdu.pdu_triggerings();
            // because the multiplexed ipdu does not keep a reference to the pdu triggering of its parts,
            // we don't know which PDU triggering to remove if there is more than one
            if triggerings.len() == 1
                && let Some(pdu_triggering) = triggerings.pop()
            {
                pdu_triggering.remove(deep)?;
            }
        }

        Ok(())
    }

    /// set the `ISignalIPdu` referenced by this dynamic part alternative
    pub fn set_ipdu(&self, ipdu: &ISignalIPdu) -> Result<(), AutosarAbstractionError> {
        let old_ipdu = self.ipdu();
        self.element()
            .get_or_create_sub_element(ElementName::IPduRef)?
            .set_reference_target(ipdu.element())?;

        self.multiplexed_ipdu()?
            .update_pdu_triggerings(old_ipdu.as_ref(), ipdu)?;

        Ok(())
    }

    /// get the `ISignalIPdu` referenced by this dynamic part alternative
    #[must_use]
    pub fn ipdu(&self) -> Option<ISignalIPdu> {
        let ipdu_elem = self
            .element()
            .get_sub_element(ElementName::IPduRef)?
            .get_reference_target()
            .ok()?;
        ISignalIPdu::try_from(ipdu_elem).ok()
    }

    /// set the selector field code of this dynamic part alternative
    pub fn set_selector_field_code(&self, code: u16) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::SelectorFieldCode)?
            .set_character_data(code as u64)?;
        Ok(())
    }

    /// get the selector field code of this dynamic part alternative
    #[must_use]
    pub fn selector_field_code(&self) -> Option<u16> {
        self.element()
            .get_sub_element(ElementName::SelectorFieldCode)?
            .character_data()?
            .parse_integer()
    }

    /// set whether this is the initial dynamic part
    pub fn set_initial_dynamic_part(&self, initial: bool) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::InitialDynamicPart)?
            .set_character_data(initial)?;
        Ok(())
    }

    /// check whether this is the initial dynamic part
    #[must_use]
    pub fn is_initial_dynamic_part(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::InitialDynamicPart)?
            .character_data()?
            .parse_bool()
    }

    /// get the multiplexed ipdu containing this dynamic part alternative
    pub fn multiplexed_ipdu(&self) -> Result<MultiplexedIPdu, AutosarAbstractionError> {
        let parent_elem = self.element().named_parent()?;
        let parent_elem = parent_elem.unwrap(); // unwrap is safe here because the parent must exist
        MultiplexedIPdu::try_from(parent_elem)
    }
}

//##################################################################

/// This element is used for user defined AUTOSAR Pdus
///
/// This PDU type is occasionally used for time-sync messages
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserDefinedPdu(Element);
abstraction_element!(UserDefinedPdu, UserDefinedPdu);
impl IdentifiableAbstractionElement for UserDefinedPdu {}

impl UserDefinedPdu {
    pub(crate) fn new(name: &str, package: &ArPackage, length: u32) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let pdu_elem = pkg_elements.create_named_sub_element(ElementName::UserDefinedPdu, name)?;
        let pdu = Self(pdu_elem);

        pdu.set_length(length)?;

        Ok(pdu)
    }

    /// remove this `UserDefinedPdu` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        // remove all triggerings of this PDU
        for pdu_triggering in self.pdu_triggerings() {
            let _ = pdu_triggering.element().remove_sub_element_kind(ElementName::IPduRef);
            let _ = pdu_triggering.remove(deep);
        }

        let ref_parents = get_reference_parents(self.element())?;

        AbstractionElement::remove(self, deep)?;

        for (named_parent, _parent) in ref_parents {
            if named_parent.element_name() == ElementName::PduToFrameMapping
                && let Ok(pdu_to_frame_mapping) = PduToFrameMapping::try_from(named_parent)
            {
                pdu_to_frame_mapping.remove(deep)?;
            }
        }

        Ok(())
    }
}

impl AbstractPdu for UserDefinedPdu {}

impl From<UserDefinedPdu> for Pdu {
    fn from(value: UserDefinedPdu) -> Self {
        Pdu::UserDefinedPdu(value)
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
    /// The Pdu is a User Defined Pdu
    UserDefinedPdu(UserDefinedPdu),
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
            Pdu::UserDefinedPdu(pdu) => pdu.element(),
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
            ElementName::UserDefinedPdu => Ok(UserDefinedPdu::try_from(element)?.into()),
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "Pdu".to_string(),
            }),
        }
    }
}

impl IdentifiableAbstractionElement for Pdu {}
impl AbstractPdu for Pdu {}

impl Pdu {
    /// remove this `Pdu` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        match self {
            Pdu::ISignalIPdu(pdu) => pdu.remove(deep),
            Pdu::NmPdu(pdu) => pdu.remove(deep),
            Pdu::NPdu(pdu) => pdu.remove(deep),
            Pdu::DcmIPdu(pdu) => pdu.remove(deep),
            Pdu::GeneralPurposePdu(pdu) => pdu.remove(deep),
            Pdu::GeneralPurposeIPdu(pdu) => pdu.remove(deep),
            Pdu::ContainerIPdu(pdu) => pdu.remove(deep),
            Pdu::SecuredIPdu(pdu) => pdu.remove(deep),
            Pdu::MultiplexedIPdu(pdu) => pdu.remove(deep),
            Pdu::UserDefinedPdu(pdu) => pdu.remove(deep),
        }
    }
}

//##################################################################

/// Wrapper for all Pdu types. It is used as a return value for functions that can return any Pdu type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IPdu {
    /// The `IPdu` is an `ISignalIPdu`
    ISignalIPdu(ISignalIPdu),
    /// The Pdu is a Transport Layer Pdu
    NPdu(NPdu),
    /// The `IPdu` is a Diagnostic Communication Management Pdu
    DcmIPdu(DcmIPdu),
    /// The `IPdu` is a general purpose Pdu
    GeneralPurposeIPdu(GeneralPurposeIPdu),
    /// The `IPdu` is a Container `IPdu`
    ContainerIPdu(ContainerIPdu),
    /// The `IPdu` is a secured `IPdu`
    SecuredIPdu(SecuredIPdu),
    /// The `IPdu` is a multiplexed `IPdu`
    MultiplexedIPdu(MultiplexedIPdu),
}

impl AbstractionElement for IPdu {
    fn element(&self) -> &Element {
        match self {
            IPdu::ISignalIPdu(pdu) => pdu.element(),
            IPdu::NPdu(pdu) => pdu.element(),
            IPdu::DcmIPdu(pdu) => pdu.element(),
            IPdu::GeneralPurposeIPdu(pdu) => pdu.element(),
            IPdu::ContainerIPdu(pdu) => pdu.element(),
            IPdu::SecuredIPdu(pdu) => pdu.element(),
            IPdu::MultiplexedIPdu(pdu) => pdu.element(),
        }
    }
}

impl TryFrom<Element> for IPdu {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::ISignalIPdu => Ok(IPdu::ISignalIPdu(ISignalIPdu::try_from(element)?)),
            ElementName::NPdu => Ok(IPdu::NPdu(NPdu::try_from(element)?)),
            ElementName::DcmIPdu => Ok(IPdu::DcmIPdu(DcmIPdu::try_from(element)?)),
            ElementName::GeneralPurposeIPdu => Ok(IPdu::GeneralPurposeIPdu(GeneralPurposeIPdu::try_from(element)?)),
            ElementName::ContainerIPdu => Ok(IPdu::ContainerIPdu(ContainerIPdu::try_from(element)?)),
            ElementName::SecuredIPdu => Ok(IPdu::SecuredIPdu(SecuredIPdu::try_from(element)?)),
            ElementName::MultiplexedIPdu => Ok(IPdu::MultiplexedIPdu(MultiplexedIPdu::try_from(element)?)),
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "IPdu".to_string(),
            }),
        }
    }
}

impl From<IPdu> for Pdu {
    fn from(value: IPdu) -> Self {
        match value {
            IPdu::ISignalIPdu(pdu) => Pdu::ISignalIPdu(pdu),
            IPdu::NPdu(pdu) => Pdu::NPdu(pdu),
            IPdu::DcmIPdu(pdu) => Pdu::DcmIPdu(pdu),
            IPdu::GeneralPurposeIPdu(pdu) => Pdu::GeneralPurposeIPdu(pdu),
            IPdu::ContainerIPdu(pdu) => Pdu::ContainerIPdu(pdu),
            IPdu::SecuredIPdu(pdu) => Pdu::SecuredIPdu(pdu),
            IPdu::MultiplexedIPdu(pdu) => Pdu::MultiplexedIPdu(pdu),
        }
    }
}

impl IdentifiableAbstractionElement for IPdu {}
impl AbstractPdu for IPdu {}
impl AbstractIpdu for IPdu {}

impl IPdu {
    /// remove this `IPdu` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        match self {
            IPdu::ISignalIPdu(pdu) => pdu.remove(deep),
            IPdu::NPdu(pdu) => pdu.remove(deep),
            IPdu::DcmIPdu(pdu) => pdu.remove(deep),
            IPdu::GeneralPurposeIPdu(pdu) => pdu.remove(deep),
            IPdu::ContainerIPdu(pdu) => pdu.remove(deep),
            IPdu::SecuredIPdu(pdu) => pdu.remove(deep),
            IPdu::MultiplexedIPdu(pdu) => pdu.remove(deep),
        }
    }
}

//##################################################################

/// a `PduTriggering` triggers a PDU in a frame or ethernet connection
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PduTriggering(Element);
abstraction_element!(PduTriggering, PduTriggering);
impl IdentifiableAbstractionElement for PduTriggering {}

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
                    pt.create_signal_triggering(&signal)?;
                } else if let Some(signal_group) = signal_mapping.signal_group() {
                    pt.create_signal_group_triggering(&signal_group)?;
                }
            }
        }

        Ok(pt)
    }

    /// remove this `PduTriggering` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        let opt_pdu = self.pdu();

        for signal_triggering in self.signal_triggerings() {
            signal_triggering.remove(deep)?;
        }

        for pdu_port in self.pdu_ports() {
            pdu_port.remove(deep)?;
        }

        // removal of PayloadRefs from SecuredIPdus is handled by AbstractionElement::remove
        // removal of ContainedPduTriggeringRefs from ContainerIPdus is handled by AbstractionElement::remove

        let ref_parents = get_reference_parents(self.element())?;

        AbstractionElement::remove(self, deep)?;

        for (named_parent, parent) in ref_parents {
            if named_parent.element_name() == ElementName::SoConIPduIdentifier
                && let Ok(socon_ipdu_identifier) = SoConIPduIdentifier::try_from(named_parent)
            {
                socon_ipdu_identifier.remove(deep)?;
            } else if parent.element_name() == ElementName::SomeipTpConnection {
                if let Ok(someip_tp_connection) = SomeipTpConnection::try_from(parent) {
                    someip_tp_connection.remove(deep)?;
                }
            } else if parent.element_name() == ElementName::PduTriggeringRefConditional
                && let Ok(Some(parent_parent)) = parent.parent()
            {
                parent_parent.remove_sub_element(parent)?;
            }
        }

        if deep && let Some(pdu) = opt_pdu {
            // check if the PDU became unused; if so remove it too
            if !is_used_system_element(pdu.element()) {
                pdu.remove(deep)?;
            }
        }

        Ok(())
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
            if let (Ok(existing_ecu), Some(existing_direction)) = (pdu_port.ecu(), pdu_port.communication_direction())
                && existing_ecu == *ecu
                && existing_direction == direction
            {
                return Ok(pdu_port);
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
    pub fn pdu_ports(&self) -> impl Iterator<Item = IPduPort> + Send + use<> {
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
    pub fn signal_triggerings(&self) -> impl Iterator<Item = ISignalTriggering> + Send + use<> {
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

    /// create a signal triggering for a signal and connect it to this `PduTriggering`
    pub(crate) fn create_signal_triggering(
        &self,
        signal: &ISignal,
    ) -> Result<ISignalTriggering, AutosarAbstractionError> {
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
            if let (Ok(ecu), Some(direction)) = (pdu_port.ecu(), pdu_port.communication_direction()) {
                st.connect_to_ecu(&ecu, direction)?;
            }
        }

        Ok(st)
    }

    /// create a signal triggering for a signal group and connect it to this `PduTriggering`
    pub(crate) fn create_signal_group_triggering(
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
            if let (Ok(ecu), Some(direction)) = (pdu_port.ecu(), pdu_port.communication_direction()) {
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
impl IdentifiableAbstractionElement for IPduPort {}

impl IPduPort {
    /// get the ECU instance that contains this `IPduPort`
    pub fn ecu(&self) -> Result<EcuInstance, AutosarAbstractionError> {
        let comm_connector_elem = self.element().named_parent()?.unwrap();
        let ecu_elem = comm_connector_elem.named_parent()?.unwrap();
        EcuInstance::try_from(ecu_elem)
    }

    /// set the communication direction of this `IPduPort`
    pub fn set_communication_direction(
        &self,
        direction: CommunicationDirection,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::CommunicationDirection)?
            .set_character_data::<EnumItem>(direction.into())?;
        Ok(())
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        AutosarModelAbstraction, ByteOrder, SystemCategory,
        communication::{AbstractFrame, AbstractFrameTriggering, CanAddressingMode, CanFrameType, TransferProperty},
    };
    use autosar_data::AutosarVersion;

    #[test]
    fn test_pdus() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let package = model.get_or_create_package("/pkg").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();

        let isignal_ipdu = system.create_isignal_ipdu("isignal_ipdu", &package, 1).unwrap();
        let nm_pdu = system.create_nm_pdu("nm_pdu", &package, 1).unwrap();
        let n_pdu = system.create_n_pdu("n_pdu", &package, 1).unwrap();
        let dcm_ipdu = system
            .create_dcm_ipdu("dcm_ipdu", &package, 1, DiagPduType::DiagRequest)
            .unwrap();
        let gp_pdu = system
            .create_general_purpose_pdu("gp_pdu", &package, 1, GeneralPurposePduCategory::Sd)
            .unwrap();
        let gp_ipdu = system
            .create_general_purpose_ipdu("gp_ipdu", &package, 1, GeneralPurposeIPduCategory::Xcp)
            .unwrap();
        let container_ipdu = system
            .create_container_ipdu(
                "container_ipdu",
                &package,
                1,
                ContainerIPduHeaderType::ShortHeader,
                RxAcceptContainedIPdu::AcceptAll,
            )
            .unwrap();
        let secured_ipdu = system
            .create_secured_ipdu("secured_ipdu", &package, 1, &SecureCommunicationProps::default())
            .unwrap();
        let multiplexed_ipdu = system.create_multiplexed_ipdu("multiplexed_ipdu", &package, 1).unwrap();

        assert_eq!(isignal_ipdu.length().unwrap(), 1);
        assert_eq!(nm_pdu.length().unwrap(), 1);
        assert_eq!(n_pdu.length().unwrap(), 1);
        assert_eq!(dcm_ipdu.length().unwrap(), 1);
        assert_eq!(gp_pdu.length().unwrap(), 1);
        assert_eq!(gp_ipdu.length().unwrap(), 1);
        assert_eq!(container_ipdu.length().unwrap(), 1);
        assert_eq!(secured_ipdu.length().unwrap(), 1);
        assert_eq!(multiplexed_ipdu.length().unwrap(), 1);

        isignal_ipdu.set_length(2).unwrap();
        assert_eq!(isignal_ipdu.length().unwrap(), 2);

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
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let package = model.get_or_create_package("/pkg").unwrap();
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
        let can_cluster = system.create_can_cluster("Cluster", &package, None).unwrap();
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
        pdu_port
            .set_communication_direction(CommunicationDirection::Out)
            .unwrap();
        assert_eq!(pdu_port.communication_direction().unwrap(), CommunicationDirection::Out);
        pdu_port.set_name("new_name").unwrap();
        assert_eq!(pdu_port.name().unwrap(), "new_name");
    }

    #[test]
    fn nm_pdu() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00052);
        let package = model.get_or_create_package("/pkg").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();

        let nm_pdu = system.create_nm_pdu("nm_pdu", &package, 1).unwrap();
        assert_eq!(nm_pdu.length().unwrap(), 1);

        nm_pdu.set_length(8).unwrap();
        assert_eq!(nm_pdu.length().unwrap(), 8);

        nm_pdu.set_unused_bit_pattern(0xff).unwrap();
        assert_eq!(nm_pdu.unused_bit_pattern().unwrap(), 0xff);

        // create a signal and map it to the PDU
        let syssignal = package.create_system_signal("sys_userdata").unwrap();
        let isignal = system
            .create_isignal("userdata", &package, 16, &syssignal, None)
            .unwrap();
        let mapping = nm_pdu
            .map_signal(
                &isignal,
                0,
                ByteOrder::MostSignificantByteFirst,
                Some(16),
                TransferProperty::Triggered,
            )
            .unwrap();
        assert_eq!(mapping.signal().unwrap(), isignal);
    }

    #[test]
    fn general_purpose_pdu() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let package = model.get_or_create_package("/pkg").unwrap();
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
    fn general_purpose_ipdu() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let package = model.get_or_create_package("/pkg").unwrap();
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

    #[test]
    fn multiplexed_ipdu() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let package = model.get_or_create_package("/pkg").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();

        let multiplexed_ipdu = system.create_multiplexed_ipdu("multiplexed_ipdu", &package, 1).unwrap();
        assert_eq!(multiplexed_ipdu.length().unwrap(), 1);

        multiplexed_ipdu.set_length(16).unwrap();
        assert_eq!(multiplexed_ipdu.length().unwrap(), 16);

        let static_ipdu = system.create_isignal_ipdu("static_ipdu", &package, 8).unwrap();
        multiplexed_ipdu.set_static_part(&static_ipdu).unwrap();
        assert_eq!(multiplexed_ipdu.static_part().unwrap(), static_ipdu);

        let dynamic_ipdu = system.create_isignal_ipdu("dynamic_ipdu", &package, 8).unwrap();
        let dp_alt = multiplexed_ipdu.add_dynamic_part(&dynamic_ipdu, 0, true).unwrap();
        assert_eq!(dp_alt.ipdu().unwrap(), dynamic_ipdu);
        assert_eq!(dp_alt.selector_field_code().unwrap(), 0);
        assert_eq!(dp_alt.is_initial_dynamic_part().unwrap(), true);
        assert_eq!(dp_alt.multiplexed_ipdu().unwrap(), multiplexed_ipdu);
        assert_eq!(multiplexed_ipdu.dynamic_part_alternatives().count(), 1);

        multiplexed_ipdu
            .set_selector_field(8, 16, ByteOrder::MostSignificantByteFirst)
            .unwrap();
        let (start_bit, length, byte_order) = multiplexed_ipdu.selector_field().unwrap();
        assert_eq!(start_bit, 8);
        assert_eq!(length, 16);
        assert_eq!(byte_order, ByteOrder::MostSignificantByteFirst);

        let cluster = system.create_can_cluster("cluster", &package, None).unwrap();
        let channel = cluster.create_physical_channel("channel").unwrap();
        let frame = system.create_can_frame("frame", &package, 64).unwrap();
        channel
            .trigger_frame(&frame, 0x33, CanAddressingMode::Standard, CanFrameType::CanFd)
            .unwrap();
        let _mapping = frame
            .map_pdu(&multiplexed_ipdu, 0, ByteOrder::MostSignificantByteLast, None)
            .unwrap();

        assert_eq!(channel.frame_triggerings().count(), 1);
        assert_eq!(channel.pdu_triggerings().count(), 3); // multiplex pdu + static part + dynamic part
    }

    #[test]
    fn ipdu() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let package = model.get_or_create_package("/pkg").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();

        let isignal_ipdu = system.create_isignal_ipdu("isignal_ipdu", &package, 1).unwrap();
        let n_pdu = system.create_n_pdu("n_pdu", &package, 1).unwrap();
        let dcm_ipdu = system
            .create_dcm_ipdu("dcm_ipdu", &package, 1, DiagPduType::DiagResponse)
            .unwrap();
        let gp_ipdu = system
            .create_general_purpose_ipdu("gp_ipdu", &package, 1, GeneralPurposeIPduCategory::Xcp)
            .unwrap();
        let container_ipdu = system
            .create_container_ipdu(
                "container_ipdu",
                &package,
                1,
                ContainerIPduHeaderType::LongHeader,
                RxAcceptContainedIPdu::AcceptConfigured,
            )
            .unwrap();
        let secured_ipdu = system
            .create_secured_ipdu("secured_ipdu", &package, 1, &SecureCommunicationProps::default())
            .unwrap();
        let multiplexed_ipdu = system.create_multiplexed_ipdu("multiplexed_ipdu", &package, 1).unwrap();

        let ipdu: IPdu = isignal_ipdu.clone().into();
        assert_eq!(ipdu.element(), isignal_ipdu.element());
        assert!(matches!(ipdu, IPdu::ISignalIPdu(_)));

        // NmPdu is not an ipdu

        let ipdu: IPdu = n_pdu.clone().into();
        assert_eq!(ipdu.element(), n_pdu.element());
        assert!(matches!(ipdu, IPdu::NPdu(_)));

        let ipdu: IPdu = dcm_ipdu.clone().into();
        assert_eq!(ipdu.element(), dcm_ipdu.element());
        assert!(matches!(ipdu, IPdu::DcmIPdu(_)));

        let ipdu: IPdu = n_pdu.clone().into();
        assert_eq!(ipdu.element(), n_pdu.element());
        assert!(matches!(ipdu, IPdu::NPdu(_)));

        // GeneralPurposePdu is not an ipdu

        let ipdu: IPdu = gp_ipdu.clone().into();
        assert_eq!(ipdu.element(), gp_ipdu.element());
        assert!(matches!(ipdu, IPdu::GeneralPurposeIPdu(_)));

        let ipdu: IPdu = container_ipdu.clone().into();
        assert_eq!(ipdu.element(), container_ipdu.element());
        assert!(matches!(ipdu, IPdu::ContainerIPdu(_)));

        let ipdu: IPdu = secured_ipdu.clone().into();
        assert_eq!(ipdu.element(), secured_ipdu.element());
        assert!(matches!(ipdu, IPdu::SecuredIPdu(_)));

        let ipdu: IPdu = multiplexed_ipdu.clone().into();
        assert_eq!(ipdu.element(), multiplexed_ipdu.element());
        assert!(matches!(ipdu, IPdu::MultiplexedIPdu(_)));

        // any Ipdu can be converted to a Pdu
        let pdu: Pdu = ipdu.clone().into();
        assert_eq!(pdu.element(), ipdu.element());
    }

    #[test]
    fn remove() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let package = model.get_or_create_package("/pkg").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();

        let isignal_ipdu = system.create_isignal_ipdu("isignal_ipdu", &package, 1).unwrap();
        let n_pdu = system.create_n_pdu("n_pdu", &package, 1).unwrap();
        let dcm_ipdu = system
            .create_dcm_ipdu("dcm_ipdu", &package, 1, DiagPduType::DiagResponse)
            .unwrap();
        let gp_ipdu = system
            .create_general_purpose_ipdu("gp_ipdu", &package, 1, GeneralPurposeIPduCategory::Xcp)
            .unwrap();
        let container_ipdu = system
            .create_container_ipdu(
                "container_ipdu",
                &package,
                1,
                ContainerIPduHeaderType::LongHeader,
                RxAcceptContainedIPdu::AcceptConfigured,
            )
            .unwrap();
        let secured_ipdu = system
            .create_secured_ipdu("secured_ipdu", &package, 1, &SecureCommunicationProps::default())
            .unwrap();
        let multiplexed_ipdu = system.create_multiplexed_ipdu("multiplexed_ipdu", &package, 1).unwrap();

        assert!(!is_used_system_element(isignal_ipdu.element()));
        assert!(!is_used_system_element(n_pdu.element()));
        assert!(!is_used_system_element(dcm_ipdu.element()));
        assert!(!is_used_system_element(gp_ipdu.element()));
        assert!(!is_used_system_element(container_ipdu.element()));
        assert!(!is_used_system_element(secured_ipdu.element()));
        assert!(!is_used_system_element(multiplexed_ipdu.element()));

        let can_cluster = system.create_can_cluster("cluster", &package, None).unwrap();
        let channel = can_cluster.create_physical_channel("channel").unwrap();
        let frame1 = system.create_can_frame("frame", &package, 64).unwrap();
        channel
            .trigger_frame(&frame1, 0x123, CanAddressingMode::Standard, CanFrameType::Can20)
            .unwrap();
        let _mapping = frame1
            .map_pdu(&isignal_ipdu, 0, ByteOrder::MostSignificantByteLast, None)
            .unwrap();
        let frame2 = system.create_can_frame("frame2", &package, 64).unwrap();
        channel
            .trigger_frame(&frame2, 0x124, CanAddressingMode::Standard, CanFrameType::Can20)
            .unwrap();
        let _mapping = frame2
            .map_pdu(&n_pdu, 0, ByteOrder::MostSignificantByteLast, None)
            .unwrap();
        let frame3 = system.create_can_frame("frame3", &package, 64).unwrap();
        channel
            .trigger_frame(&frame3, 0x125, CanAddressingMode::Standard, CanFrameType::Can20)
            .unwrap();
        let _mapping = frame3
            .map_pdu(&dcm_ipdu, 0, ByteOrder::MostSignificantByteLast, None)
            .unwrap();
        let frame4 = system.create_can_frame("frame4", &package, 64).unwrap();
        channel
            .trigger_frame(&frame4, 0x126, CanAddressingMode::Standard, CanFrameType::Can20)
            .unwrap();
        let _mapping = frame4
            .map_pdu(&gp_ipdu, 0, ByteOrder::MostSignificantByteLast, None)
            .unwrap();
        let frame5 = system.create_can_frame("frame5", &package, 64).unwrap();
        channel
            .trigger_frame(&frame5, 0x127, CanAddressingMode::Standard, CanFrameType::Can20)
            .unwrap();
        let _mapping = frame5
            .map_pdu(&container_ipdu, 0, ByteOrder::MostSignificantByteLast, None)
            .unwrap();
        let frame6 = system.create_can_frame("frame6", &package, 64).unwrap();
        channel
            .trigger_frame(&frame6, 0x128, CanAddressingMode::Standard, CanFrameType::Can20)
            .unwrap();
        let _mapping = frame6
            .map_pdu(&secured_ipdu, 0, ByteOrder::MostSignificantByteLast, None)
            .unwrap();
        let frame7 = system.create_can_frame("frame7", &package, 64).unwrap();
        channel
            .trigger_frame(&frame7, 0x129, CanAddressingMode::Standard, CanFrameType::Can20)
            .unwrap();
        let _mapping = frame7
            .map_pdu(&multiplexed_ipdu, 0, ByteOrder::MostSignificantByteLast, None)
            .unwrap();

        assert_eq!(channel.pdu_triggerings().count(), 7);

        // add static and dynamic parts to the multiplexed ipdu
        let static_part_pdu = system.create_isignal_ipdu("static_part_pdu", &package, 8).unwrap();
        multiplexed_ipdu.set_static_part(&static_part_pdu).unwrap();
        let dynamic_part_pdu = system.create_isignal_ipdu("dynamic_part_pdu", &package, 8).unwrap();
        multiplexed_ipdu.add_dynamic_part(&dynamic_part_pdu, 0, true).unwrap();

        assert_eq!(channel.pdu_triggerings().count(), 9); // static + dynamic part added

        // add a contained IPdu to the container ipdu
        let contained_ipdu = system.create_isignal_ipdu("contained_ipdu", &package, 8).unwrap();
        container_ipdu.map_ipdu(&contained_ipdu, &channel).unwrap();

        // add a payload pdu to the secured ipdu
        let payload_ipdu = system.create_isignal_ipdu("payload_ipdu", &package, 8).unwrap();
        secured_ipdu.set_payload_ipdu(&payload_ipdu, &channel).unwrap();

        assert_eq!(channel.pdu_triggerings().count(), 11); // contained + payload added

        // remove all pdus deeply
        isignal_ipdu.remove(true).unwrap();
        n_pdu.remove(true).unwrap();
        dcm_ipdu.remove(true).unwrap();
        gp_ipdu.remove(true).unwrap();
        container_ipdu.remove(true).unwrap();
        secured_ipdu.remove(true).unwrap();
        multiplexed_ipdu.remove(true).unwrap();

        // all PDU triggerings, including for contained and payload pdus, should be removed
        assert_eq!(channel.pdu_triggerings().count(), 0);
    }
}
