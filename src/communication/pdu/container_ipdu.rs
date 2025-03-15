use crate::communication::{AbstractIpdu, AbstractPdu, AbstractPhysicalChannel, IPdu, Pdu};
use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element,
};
use autosar_data::{Element, ElementName, EnumItem};

use super::{PduCollectionTrigger, PduTriggering};

//##################################################################

/// Several `IPdus` can be collected in one `ContainerIPdu` based on the headerType
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContainerIPdu(Element);
abstraction_element!(ContainerIPdu, ContainerIPdu);
impl IdentifiableAbstractionElement for ContainerIPdu {}

impl ContainerIPdu {
    pub(crate) fn new(
        name: &str,
        package: &ArPackage,
        length: u32,
        header_type: ContainerIPduHeaderType,
        rx_accept: RxAcceptContainedIPdu,
    ) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_pdu = pkg_elements.create_named_sub_element(ElementName::ContainerIPdu, name)?;
        let container_ipdu = Self(elem_pdu);
        container_ipdu.set_length(length)?;
        container_ipdu.set_header_type(header_type)?;
        container_ipdu.set_rx_accept_contained_ipdu(rx_accept)?;

        Ok(container_ipdu)
    }

    /// set the header type of this `ContainerIPdu`
    pub fn set_header_type(&self, header_type: ContainerIPduHeaderType) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::HeaderType)?
            .set_character_data::<EnumItem>(header_type.into())?;
        Ok(())
    }

    /// get the header type of this `ContainerIPdu`
    #[must_use]
    pub fn header_type(&self) -> Option<ContainerIPduHeaderType> {
        self.element()
            .get_sub_element(ElementName::HeaderType)?
            .character_data()?
            .enum_value()?
            .try_into()
            .ok()
    }

    /// set the rx accept of this `ContainerIPdu`
    pub fn set_rx_accept_contained_ipdu(
        &self,
        rx_accept: RxAcceptContainedIPdu,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::RxAcceptContainedIPdu)?
            .set_character_data::<EnumItem>(rx_accept.into())?;
        Ok(())
    }

    /// get the rx accept of this `ContainerIPdu`
    #[must_use]
    pub fn rx_accept_contained_ipdu(&self) -> Option<RxAcceptContainedIPdu> {
        self.element()
            .get_sub_element(ElementName::RxAcceptContainedIPdu)?
            .character_data()?
            .enum_value()?
            .try_into()
            .ok()
    }

    /// set the container timeout of this `ContainerIPdu`
    pub fn set_container_timeout(&self, timeout: Option<f64>) -> Result<(), AutosarAbstractionError> {
        if let Some(timeout) = timeout {
            self.element()
                .get_or_create_sub_element(ElementName::ContainerTimeout)?
                .set_character_data(timeout)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::ContainerTimeout);
        }
        Ok(())
    }

    /// get the container timeout of this `ContainerIPdu`
    #[must_use]
    pub fn container_timeout(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::ContainerTimeout)?
            .character_data()?
            .parse_float()
    }

    /// set the container trigger of this `ContainerIPdu`
    pub fn set_container_trigger(&self, trigger: Option<ContainerIPduTrigger>) -> Result<(), AutosarAbstractionError> {
        if let Some(trigger) = trigger {
            self.element()
                .get_or_create_sub_element(ElementName::ContainerTrigger)?
                .set_character_data::<EnumItem>(trigger.into())?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::ContainerTrigger);
        }
        Ok(())
    }

    /// get the container trigger of this `ContainerIPdu`
    #[must_use]
    pub fn container_trigger(&self) -> Option<ContainerIPduTrigger> {
        self.element()
            .get_sub_element(ElementName::ContainerTrigger)?
            .character_data()?
            .enum_value()?
            .try_into()
            .ok()
    }

    /// map an IPdu to this `ContainerIPdu`, and create a PduTriggering for it in the physical channel
    pub fn map_ipdu<T: AbstractIpdu, U: AbstractPhysicalChannel>(
        &self,
        ipdu: &T,
        physical_channel: &U,
    ) -> Result<PduTriggering, AutosarAbstractionError> {
        let contained_pdu_triggering_refs_elem = self
            .element()
            .get_or_create_sub_element(ElementName::ContainedPduTriggeringRefs)?;
        let pdu_triggering = PduTriggering::new(&ipdu.clone().into(), &physical_channel.clone().into())?;

        contained_pdu_triggering_refs_elem
            .create_sub_element(ElementName::ContainedPduTriggeringRef)?
            .set_reference_target(pdu_triggering.element())?;

        Ok(pdu_triggering)
    }

    /// iterate over all contained IPdu triggerings
    pub fn contained_ipdu_triggerings(&self) -> impl Iterator<Item = PduTriggering> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::ContainedPduTriggeringRefs)
            .into_iter()
            .flat_map(|triggerings| triggerings.sub_elements())
            .filter_map(|triggering_ref| triggering_ref.get_reference_target().ok())
            .filter_map(|triggering| PduTriggering::try_from(triggering).ok())
    }
}

impl AbstractPdu for ContainerIPdu {}

impl AbstractIpdu for ContainerIPdu {}

impl From<ContainerIPdu> for Pdu {
    fn from(value: ContainerIPdu) -> Self {
        Pdu::ContainerIPdu(value)
    }
}

impl From<ContainerIPdu> for IPdu {
    fn from(value: ContainerIPdu) -> Self {
        IPdu::ContainerIPdu(value)
    }
}

//##################################################################

/// The header type of a `ContainerIPdu`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContainerIPduHeaderType {
    /// Header size is 64 bit: Header id is 32 bit, dlc is 32 bit
    LongHeader,
    /// no header is used, the locations of the contained PDUs are fixed
    NoHeader,
    /// Header size is 32 bit: Header id is 24 bit, dlc is 8 bit
    ShortHeader,
}

impl From<ContainerIPduHeaderType> for EnumItem {
    fn from(value: ContainerIPduHeaderType) -> Self {
        match value {
            ContainerIPduHeaderType::LongHeader => EnumItem::LongHeader,
            ContainerIPduHeaderType::NoHeader => EnumItem::NoHeader,
            ContainerIPduHeaderType::ShortHeader => EnumItem::ShortHeader,
        }
    }
}

impl TryFrom<EnumItem> for ContainerIPduHeaderType {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<Self, Self::Error> {
        match value {
            EnumItem::LongHeader => Ok(ContainerIPduHeaderType::LongHeader),
            EnumItem::NoHeader => Ok(ContainerIPduHeaderType::NoHeader),
            EnumItem::ShortHeader => Ok(ContainerIPduHeaderType::ShortHeader),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "ContainerIPduHeaderType".to_string(),
            }),
        }
    }
}

//##################################################################

/// The `RxAcceptContainedIPdu` enum defines whether a fixed set of contained IPdus is accepted or all contained IPdus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RxAcceptContainedIPdu {
    /// All contained IPdus are accepted
    AcceptAll,
    /// Only the configured contained IPdus are accepted
    AcceptConfigured,
}

impl From<RxAcceptContainedIPdu> for EnumItem {
    fn from(value: RxAcceptContainedIPdu) -> Self {
        match value {
            RxAcceptContainedIPdu::AcceptAll => EnumItem::AcceptAll,
            RxAcceptContainedIPdu::AcceptConfigured => EnumItem::AcceptConfigured,
        }
    }
}

impl TryFrom<EnumItem> for RxAcceptContainedIPdu {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<Self, Self::Error> {
        match value {
            EnumItem::AcceptAll => Ok(RxAcceptContainedIPdu::AcceptAll),
            EnumItem::AcceptConfigured => Ok(RxAcceptContainedIPdu::AcceptConfigured),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "RxAcceptContainedIPdu".to_string(),
            }),
        }
    }
}

//##################################################################

/// Defines when the transmission of the ContainerIPdu shall be requested
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContainerIPduTrigger {
    /// transmission of the ContainerIPdu shall be requested when the default trigger conditions apply
    DefaultTrigger,
    /// transmission of the ContainerIPdu shall be requested right after the first Contained
    /// IPdu was put into the ContainerIPdu
    FirstContainedTrigger,
}

impl From<ContainerIPduTrigger> for EnumItem {
    fn from(value: ContainerIPduTrigger) -> Self {
        match value {
            ContainerIPduTrigger::DefaultTrigger => EnumItem::DefaultTrigger,
            ContainerIPduTrigger::FirstContainedTrigger => EnumItem::FirstContainedTrigger,
        }
    }
}

impl TryFrom<EnumItem> for ContainerIPduTrigger {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<Self, Self::Error> {
        match value {
            EnumItem::DefaultTrigger => Ok(ContainerIPduTrigger::DefaultTrigger),
            EnumItem::FirstContainedTrigger => Ok(ContainerIPduTrigger::FirstContainedTrigger),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "ContainerIPduTrigger".to_string(),
            }),
        }
    }
}

//##################################################################

/// Properties for an IPdu that is transmitted in a container IPdu
#[derive(Debug, Clone, PartialEq)]
pub struct ContainedIPduProps {
    /// collection semantics: LastIsBest or Queued
    pub collection_semantics: Option<ContainedIPduCollectionSemantics>,
    /// header id of the contained IPdu, used when the header type is LongHeader
    pub header_id_long: Option<u32>,
    /// header id of the contained IPdu, used when the header type is ShortHeader
    pub header_id_short: Option<u32>, // 24 bit
    /// offset of the contained IPdu in the container IPdu, used when the header type is NoHeader
    pub offset: Option<u32>,
    /// priority of the contained IPdu. 255: lowest, 0: highest
    pub priority: Option<u8>,
    /// sender timeout. Ignored on the receiver side
    pub timeout: Option<f64>,
    /// defines whether the contained IPdu triggers transmission of the container IPdu
    pub trigger: Option<PduCollectionTrigger>,
    /// update indication bit position of the contained IPdu
    pub update_indication_bit_position: Option<u32>,
}

impl ContainedIPduProps {
    pub(crate) fn get_props(parent_elem: &Element) -> Option<Self> {
        let props_elem = parent_elem.get_sub_element(ElementName::ContainedIPduProps)?;
        let collection_semantics = props_elem
            .get_sub_element(ElementName::CollectionSemantics)
            .and_then(|elem| elem.character_data()?.enum_value()?.try_into().ok());
        let header_id_long = props_elem
            .get_sub_element(ElementName::HeaderIdLongHeader)
            .and_then(|elem| elem.character_data()?.parse_integer());
        let header_id_short = props_elem
            .get_sub_element(ElementName::HeaderIdShortHeader)
            .and_then(|elem| elem.character_data()?.parse_integer());
        let offset = props_elem
            .get_sub_element(ElementName::Offset)
            .and_then(|elem| elem.character_data()?.parse_integer());
        let priority = props_elem
            .get_sub_element(ElementName::Priority)
            .and_then(|elem| elem.character_data()?.parse_integer());
        let timeout = props_elem
            .get_sub_element(ElementName::Timeout)
            .and_then(|elem| elem.character_data()?.parse_float());
        let trigger = props_elem
            .get_sub_element(ElementName::Trigger)
            .and_then(|elem| elem.character_data()?.enum_value()?.try_into().ok());
        let update_indication_bit_position = props_elem
            .get_sub_element(ElementName::UpdateIndicationBitPosition)
            .and_then(|elem| elem.character_data()?.parse_integer());

        Some(Self {
            collection_semantics,
            header_id_long,
            header_id_short,
            offset,
            priority,
            timeout,
            trigger,
            update_indication_bit_position,
        })
    }

    pub(crate) fn set_props(parent_elem: &Element, props: Option<&Self>) -> Result<(), AutosarAbstractionError> {
        if let Some(props) = props {
            let props_elem = parent_elem.get_or_create_sub_element(ElementName::ContainedIPduProps)?;
            if let Some(collection_semantics) = props.collection_semantics {
                props_elem
                    .get_or_create_sub_element(ElementName::CollectionSemantics)?
                    .set_character_data::<EnumItem>(collection_semantics.into())?;
            }
            if let Some(header_id_long) = props.header_id_long {
                props_elem
                    .get_or_create_sub_element(ElementName::HeaderIdLongHeader)?
                    .set_character_data(header_id_long as u64)?;
            }
            if let Some(header_id_short) = props.header_id_short {
                props_elem
                    .get_or_create_sub_element(ElementName::HeaderIdShortHeader)?
                    .set_character_data(header_id_short as u64)?;
            }
            if let Some(offset) = props.offset {
                props_elem
                    .get_or_create_sub_element(ElementName::Offset)?
                    .set_character_data(offset as u64)?;
            }
            if let Some(priority) = props.priority {
                props_elem
                    .get_or_create_sub_element(ElementName::Priority)?
                    .set_character_data(priority as u64)?;
            }
            if let Some(timeout) = props.timeout {
                props_elem
                    .get_or_create_sub_element(ElementName::Timeout)?
                    .set_character_data(timeout)?;
            }
            if let Some(trigger) = props.trigger {
                props_elem
                    .get_or_create_sub_element(ElementName::Trigger)?
                    .set_character_data::<EnumItem>(trigger.into())?;
            }
            if let Some(update_indication_bit_position) = props.update_indication_bit_position {
                props_elem
                    .get_or_create_sub_element(ElementName::UpdateIndicationBitPosition)?
                    .set_character_data(update_indication_bit_position as u64)?;
            }
        } else {
            let _ = parent_elem.remove_sub_element_kind(ElementName::ContainedIPduProps);
        }
        Ok(())
    }
}

//##################################################################

/// collection semantics for the ContainedIPdu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContainedIPduCollectionSemantics {
    /// The ContainedIPdu data will be fetched via TriggerTransmit just before the transmission executes.
    LastIsBest,
    /// The ContainedIPdu data will instantly be stored to the ContainerIPdu in the context of the Transmit call
    Queued,
}

impl From<ContainedIPduCollectionSemantics> for EnumItem {
    fn from(value: ContainedIPduCollectionSemantics) -> Self {
        match value {
            ContainedIPduCollectionSemantics::LastIsBest => EnumItem::LastIsBest,
            ContainedIPduCollectionSemantics::Queued => EnumItem::Queued,
        }
    }
}

impl TryFrom<EnumItem> for ContainedIPduCollectionSemantics {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<Self, Self::Error> {
        match value {
            EnumItem::LastIsBest => Ok(ContainedIPduCollectionSemantics::LastIsBest),
            EnumItem::Queued => Ok(ContainedIPduCollectionSemantics::Queued),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "ContainedIPduCollectionSemantics".to_string(),
            }),
        }
    }
}

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        AutosarModelAbstraction, SystemCategory,
        communication::{FlexrayChannelName, FlexrayClusterSettings},
    };
    use autosar_data::AutosarVersion;

    #[test]
    fn test_container_ipdu() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let package = model.get_or_create_package("/pkg").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();
        let flexray_cluster = system
            .create_flexray_cluster("FlxCluster", &package, &FlexrayClusterSettings::new())
            .unwrap();
        let flexray_channel = flexray_cluster
            .create_physical_channel("FlxChannel", FlexrayChannelName::A)
            .unwrap();

        let container_ipdu = system
            .create_container_ipdu(
                "container_ipdu",
                &package,
                64,
                ContainerIPduHeaderType::ShortHeader,
                RxAcceptContainedIPdu::AcceptAll,
            )
            .unwrap();
        assert_eq!(
            container_ipdu.header_type().unwrap(),
            ContainerIPduHeaderType::ShortHeader
        );
        assert_eq!(
            container_ipdu.rx_accept_contained_ipdu().unwrap(),
            RxAcceptContainedIPdu::AcceptAll
        );

        container_ipdu
            .set_header_type(ContainerIPduHeaderType::LongHeader)
            .unwrap();
        assert_eq!(
            container_ipdu.header_type().unwrap(),
            ContainerIPduHeaderType::LongHeader
        );

        container_ipdu
            .set_rx_accept_contained_ipdu(RxAcceptContainedIPdu::AcceptConfigured)
            .unwrap();
        assert_eq!(
            container_ipdu.rx_accept_contained_ipdu().unwrap(),
            RxAcceptContainedIPdu::AcceptConfigured
        );

        container_ipdu.set_container_timeout(Some(0.1)).unwrap();
        assert_eq!(container_ipdu.container_timeout().unwrap(), 0.1);
        container_ipdu.set_container_timeout(None).unwrap();
        assert_eq!(container_ipdu.container_timeout(), None);

        container_ipdu
            .set_container_trigger(Some(ContainerIPduTrigger::DefaultTrigger))
            .unwrap();
        assert_eq!(
            container_ipdu.container_trigger().unwrap(),
            ContainerIPduTrigger::DefaultTrigger
        );
        container_ipdu
            .set_container_trigger(Some(ContainerIPduTrigger::FirstContainedTrigger))
            .unwrap();
        assert_eq!(
            container_ipdu.container_trigger().unwrap(),
            ContainerIPduTrigger::FirstContainedTrigger
        );
        container_ipdu.set_container_trigger(None).unwrap();
        assert_eq!(container_ipdu.container_trigger(), None);

        let contained_ipdu = system.create_isignal_ipdu("ISignalIpdu", &package, 8).unwrap();
        let contained_props = ContainedIPduProps {
            collection_semantics: Some(ContainedIPduCollectionSemantics::LastIsBest),
            header_id_long: Some(0x12345678),
            header_id_short: Some(0x123456),
            offset: Some(0x10),
            priority: Some(0x10),
            timeout: Some(0.1),
            trigger: Some(PduCollectionTrigger::Always),
            update_indication_bit_position: Some(0x10),
        };
        contained_ipdu.set_contained_ipdu_props(Some(&contained_props)).unwrap();
        assert_eq!(contained_ipdu.contained_ipdu_props().unwrap(), contained_props);
        contained_ipdu.set_contained_ipdu_props(None).unwrap();
        assert_eq!(contained_ipdu.contained_ipdu_props(), None);

        let pdu_triggering = container_ipdu.map_ipdu(&contained_ipdu, &flexray_channel).unwrap();
        assert_eq!(container_ipdu.contained_ipdu_triggerings().count(), 1);
        assert_eq!(container_ipdu.contained_ipdu_triggerings().next(), Some(pdu_triggering));
    }
}
