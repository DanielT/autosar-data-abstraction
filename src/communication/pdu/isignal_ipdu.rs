use crate::communication::{AbstractIpdu, AbstractPdu, IPdu, ISignal, ISignalGroup, Pdu, TransferProperty};
use crate::{
    abstraction_element, make_unique_name, AbstractionElement, ArPackage, AutosarAbstractionError, ByteOrder,
    IdentifiableAbstractionElement,
};
use autosar_data::{Element, ElementName, EnumItem};

//##################################################################

/// Represents the `IPdus` handled by Com
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ISignalIPdu(Element);
abstraction_element!(ISignalIPdu, ISignalIPdu);
impl IdentifiableAbstractionElement for ISignalIPdu {}

impl ISignalIPdu {
    pub(crate) fn new(name: &str, package: &ArPackage, length: u32) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_pdu = pkg_elements.create_named_sub_element(ElementName::ISignalIPdu, name)?;
        elem_pdu
            .create_sub_element(ElementName::Length)?
            .set_character_data(length.to_string())?;

        Ok(Self(elem_pdu))
    }

    /// returns an iterator over all signals mapped to the PDU
    pub fn mapped_signals(&self) -> impl Iterator<Item = ISignalToIPduMapping> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::ISignalToPduMappings)
            .into_iter()
            .flat_map(|mappings| mappings.sub_elements())
            .filter_map(|elem| ISignalToIPduMapping::try_from(elem).ok())
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

        let length = self.length().unwrap_or(0);
        let mut validator = SignalMappingValidator::new(length);
        // build a bitmap of all signals that are already mapped in this pdu
        for mapping in self.mapped_signals() {
            if let (Some(m_signal), Some(m_start_pos), Some(m_byte_order)) =
                (mapping.signal(), mapping.start_position(), mapping.byte_order())
            {
                let len = m_signal.length().unwrap_or(0);
                validator.add_signal(m_start_pos, len, m_byte_order, mapping.update_bit());
            }
        }
        // add the new signal to the validator bitmap to see if it overlaps any exisiting signals
        if !validator.add_signal(start_position, signal.length().unwrap_or(0), byte_order, update_bit) {
            return Err(AutosarAbstractionError::InvalidParameter(format!(
                "Cannot map signal {signal_name} to an overalapping position in the pdu"
            )));
        }

        if let Some(signal_group) = signal.signal_group() {
            if !self
                .mapped_signals()
                .filter_map(|mapping| mapping.signal_group())
                .any(|grp| grp == signal_group)
            {
                return Err(AutosarAbstractionError::InvalidParameter(
                    "Cannot map signal to pdu, because it is part of an unmapped signal group.".to_string(),
                ));
            }
        }

        // add a pdu triggering for the newly mapped PDU to each frame triggering of this frame
        for pt in self.pdu_triggerings() {
            let st = pt.add_signal_triggering(signal)?;
            for pdu_port in pt.pdu_ports() {
                if let (Some(ecu), Some(direction)) = (pdu_port.ecu(), pdu_port.communication_direction()) {
                    st.connect_to_ecu(&ecu, direction)?;
                }
            }
        }

        // create and return the new mapping
        let model = self.element().model()?;
        let base_path = self.element().path()?;
        let name = make_unique_name(&model, &base_path, &signal_name);

        let mappings = self
            .element()
            .get_or_create_sub_element(ElementName::ISignalToPduMappings)?;

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
            let st = pt.add_signal_group_triggering(signal_group)?;
            for pdu_port in pt.pdu_ports() {
                if let (Some(ecu), Some(direction)) = (pdu_port.ecu(), pdu_port.communication_direction()) {
                    st.connect_to_ecu(&ecu, direction)?;
                }
            }
        }

        // create and return the new mapping
        let model = self.element().model()?;
        let base_path = self.element().path()?;
        let name = make_unique_name(&model, &base_path, &signal_group_name);

        let mappings = self
            .element()
            .get_or_create_sub_element(ElementName::ISignalToPduMappings)?;

        ISignalToIPduMapping::new_with_group(&name, &mappings, signal_group)
    }

    /// set the transmission timing of the PDU
    pub fn set_timing(&self, timing_spec: &IpduTiming) -> Result<(), AutosarAbstractionError> {
        let _ = self
            .element()
            .remove_sub_element_kind(ElementName::IPduTimingSpecifications);

        let timing_elem = self
            .element()
            .create_sub_element(ElementName::IPduTimingSpecifications)?
            .create_sub_element(ElementName::IPduTiming)?;
        if let Some(min_delay) = timing_spec.minimum_delay {
            timing_elem
                .create_sub_element(ElementName::MinimumDelay)?
                .set_character_data(min_delay)?;
        }
        if let Some(transmission_mode_true_timing) = &timing_spec.transmission_mode_true_timing {
            let tmtt_elem = timing_elem
                .get_or_create_sub_element(ElementName::TransmissionModeDeclaration)?
                .create_sub_element(ElementName::TransmissionModeTrueTiming)?;
            Self::set_transmission_mode_timinig(tmtt_elem, transmission_mode_true_timing)?;
        }
        if let Some(transmission_mode_false_timing) = &timing_spec.transmission_mode_false_timing {
            let tmtf_elem = timing_elem
                .get_or_create_sub_element(ElementName::TransmissionModeDeclaration)?
                .create_sub_element(ElementName::TransmissionModeFalseTiming)?;
            Self::set_transmission_mode_timinig(tmtf_elem, transmission_mode_false_timing)?;
        }

        Ok(())
    }

    /// Helper function to set the transmission mode timing, used by `ISignalIPdu::set_timing` for both true and false timing
    fn set_transmission_mode_timinig(
        timing_element: Element,
        transmission_mode_timing: &TransmissionModeTiming,
    ) -> Result<(), AutosarAbstractionError> {
        if let Some(cyclic_timing) = &transmission_mode_timing.cyclic_timing {
            let ct_elem = timing_element.create_sub_element(ElementName::CyclicTiming)?;
            ct_elem
                .create_sub_element(ElementName::TimePeriod)?
                .create_sub_element(ElementName::Value)?
                .set_character_data(cyclic_timing.time_period)?;
            if let Some(time_offset) = cyclic_timing.time_offset {
                ct_elem
                    .create_sub_element(ElementName::TimeOffset)?
                    .create_sub_element(ElementName::Value)?
                    .set_character_data(time_offset)?;
            }
        }
        if let Some(event_controlled_timing) = &transmission_mode_timing.event_controlled_timing {
            let ect_elem = timing_element.create_sub_element(ElementName::EventControlledTiming)?;
            ect_elem
                .create_sub_element(ElementName::NumberOfRepetitions)?
                .set_character_data(u64::from(event_controlled_timing.number_of_repetitions))?;
            if let Some(repetition_period) = event_controlled_timing.repetition_period {
                ect_elem
                    .create_sub_element(ElementName::RepetitionPeriod)?
                    .create_sub_element(ElementName::Value)?
                    .set_character_data(repetition_period)?;
            }
        }

        Ok(())
    }

    /// get the transmission timing of the PDU
    #[must_use]
    pub fn timing(&self) -> Option<IpduTiming> {
        let timing_elem = self
            .element()
            .get_sub_element(ElementName::IPduTimingSpecifications)?
            .get_sub_element(ElementName::IPduTiming)?;
        let minimum_delay = timing_elem
            .get_sub_element(ElementName::MinimumDelay)
            .and_then(|md| md.character_data())
            .and_then(|cdata| cdata.parse_float());
        let transmission_mode_true_timing = timing_elem
            .get_sub_element(ElementName::TransmissionModeDeclaration)
            .and_then(|tmd| tmd.get_sub_element(ElementName::TransmissionModeTrueTiming))
            .and_then(|tmtt| Self::transmission_mode_timing(&tmtt));
        let transmission_mode_false_timing = timing_elem
            .get_sub_element(ElementName::TransmissionModeDeclaration)
            .and_then(|tmd| tmd.get_sub_element(ElementName::TransmissionModeFalseTiming))
            .and_then(|tmtf| Self::transmission_mode_timing(&tmtf));

        Some(IpduTiming {
            minimum_delay,
            transmission_mode_true_timing,
            transmission_mode_false_timing,
        })
    }

    /// Helper function to get the transmission mode timing, used by `ISignalIPdu::timing` for both true and false modes
    fn transmission_mode_timing(timing_elem: &Element) -> Option<TransmissionModeTiming> {
        let cyclic_timing = timing_elem.get_sub_element(ElementName::CyclicTiming).and_then(|ct| {
            let time_period = ct
                .get_sub_element(ElementName::TimePeriod)
                .and_then(|tp| tp.get_sub_element(ElementName::Value))
                .and_then(|val| val.character_data())
                .and_then(|cdata| cdata.parse_float());
            let time_offset = ct
                .get_sub_element(ElementName::TimeOffset)
                .and_then(|to| to.get_sub_element(ElementName::Value))
                .and_then(|val| val.character_data())
                .and_then(|cdata| cdata.parse_float());
            time_period.map(|tp| CyclicTiming {
                time_period: tp,
                time_offset,
            })
        });
        let event_controlled_timing = timing_elem
            .get_sub_element(ElementName::EventControlledTiming)
            .and_then(|ect| {
                let number_of_repetitions = ect
                    .get_sub_element(ElementName::NumberOfRepetitions)
                    .and_then(|nr| nr.character_data())
                    .and_then(|cdata| cdata.parse_integer());
                let repetition_period = ect
                    .get_sub_element(ElementName::RepetitionPeriod)
                    .and_then(|rp| rp.get_sub_element(ElementName::Value))
                    .and_then(|val| val.character_data())
                    .and_then(|cdata| cdata.parse_float());
                number_of_repetitions.map(|nr| EventControlledTiming {
                    number_of_repetitions: nr,
                    repetition_period,
                })
            });

        Some(TransmissionModeTiming {
            cyclic_timing,
            event_controlled_timing,
        })
    }
}

impl AbstractPdu for ISignalIPdu {}

impl AbstractIpdu for ISignalIPdu {}

impl From<ISignalIPdu> for Pdu {
    fn from(value: ISignalIPdu) -> Self {
        Pdu::ISignalIPdu(value)
    }
}

impl From<ISignalIPdu> for IPdu {
    fn from(value: ISignalIPdu) -> Self {
        IPdu::ISignalIPdu(value)
    }
}

//##################################################################

/// `ISignalToIPduMapping` connects an `ISignal` or `ISignalGroup` to an `ISignalToIPdu`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ISignalToIPduMapping(Element);
abstraction_element!(ISignalToIPduMapping, ISignalToIPduMapping);
impl IdentifiableAbstractionElement for ISignalToIPduMapping {}

impl ISignalToIPduMapping {
    fn new_with_signal(
        name: &str,
        mappings: &Element,
        signal: &ISignal,
        start_position: u32,
        byte_order: ByteOrder,
        update_bit: Option<u32>,
        transfer_property: TransferProperty,
    ) -> Result<Self, AutosarAbstractionError> {
        let signal_mapping = mappings.create_named_sub_element(ElementName::ISignalToIPduMapping, name)?;
        signal_mapping
            .create_sub_element(ElementName::ISignalRef)?
            .set_reference_target(signal.element())?;
        signal_mapping
            .create_sub_element(ElementName::PackingByteOrder)?
            .set_character_data::<EnumItem>(byte_order.into())?;
        signal_mapping
            .create_sub_element(ElementName::StartPosition)?
            .set_character_data(u64::from(start_position))?;
        signal_mapping
            .create_sub_element(ElementName::TransferProperty)?
            .set_character_data::<EnumItem>(transfer_property.into())?;
        if let Some(update_bit_pos) = update_bit {
            signal_mapping
                .create_sub_element(ElementName::UpdateIndicationBitPosition)?
                .set_character_data(u64::from(update_bit_pos))?;
        }

        Ok(Self(signal_mapping))
    }

    fn new_with_group(
        name: &str,
        mappings: &Element,
        signal_group: &ISignalGroup,
    ) -> Result<Self, AutosarAbstractionError> {
        let signal_mapping = mappings.create_named_sub_element(ElementName::ISignalToIPduMapping, name)?;
        signal_mapping
            .create_sub_element(ElementName::ISignalGroupRef)?
            .set_reference_target(signal_group.element())?;

        Ok(Self(signal_mapping))
    }

    /// Reference to the signal that is mapped to the PDU.
    /// Every mapping contains either a signal or a signal group.
    #[must_use]
    pub fn signal(&self) -> Option<ISignal> {
        self.element()
            .get_sub_element(ElementName::ISignalRef)
            .and_then(|sigref| sigref.get_reference_target().ok())
            .and_then(|signal_elem| ISignal::try_from(signal_elem).ok())
    }

    /// Set the byte order of the data in the mapped signal.
    pub fn set_byte_order(&self, byte_order: ByteOrder) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::PackingByteOrder)?
            .set_character_data::<EnumItem>(byte_order.into())?;
        Ok(())
    }

    /// Byte order of the data in the signal.
    #[must_use]
    pub fn byte_order(&self) -> Option<ByteOrder> {
        self.element()
            .get_sub_element(ElementName::PackingByteOrder)
            .and_then(|pbo| pbo.character_data())
            .and_then(|cdata| cdata.enum_value())
            .and_then(|enumval| enumval.try_into().ok())
    }

    /// Start position of the signal data within the PDU (bit position).
    /// The start position is mandatory if the mapping describes a signal.
    #[must_use]
    pub fn start_position(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::StartPosition)
            .and_then(|sp_elem| sp_elem.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// Bit position of the update bit for the mapped signal. Not all signals use an update bit.
    /// This is never used for signal groups
    #[must_use]
    pub fn update_bit(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::UpdateIndicationBitPosition)
            .and_then(|uibp| uibp.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// Set the transfer property of the mapped signal
    pub fn set_transfer_property(&self, transfer_property: TransferProperty) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::TransferProperty)?
            .set_character_data::<EnumItem>(transfer_property.into())?;
        Ok(())
    }

    /// Get the transfer property of the mapped signal
    #[must_use]
    pub fn transfer_property(&self) -> Option<TransferProperty> {
        self.element()
            .get_sub_element(ElementName::TransferProperty)
            .and_then(|pbo| pbo.character_data())
            .and_then(|cdata| cdata.enum_value())
            .and_then(|enumval| enumval.try_into().ok())
    }

    /// Reference to the signal group that is mapped to the PDU.
    /// Every mapping contains either a signal or a signal group.
    #[must_use]
    pub fn signal_group(&self) -> Option<ISignalGroup> {
        self.element()
            .get_sub_element(ElementName::ISignalGroupRef)
            .and_then(|sgref| sgref.get_reference_target().ok())
            .and_then(|siggrp_elem| ISignalGroup::try_from(siggrp_elem).ok())
    }
}

//##################################################################

/// Timing specification for an IPDU
#[derive(Debug, Clone, PartialEq)]
pub struct IpduTiming {
    /// minimum delay in seconds between two transmissions of the PDU
    pub minimum_delay: Option<f64>,
    /// timing specification if the COM transmission mode is true
    pub transmission_mode_true_timing: Option<TransmissionModeTiming>,
    /// timing specification if the COM transmission mode is false
    pub transmission_mode_false_timing: Option<TransmissionModeTiming>,
}

/// Cyclic and event controlled timing parameters for an IPDU
#[derive(Debug, Clone, PartialEq)]
pub struct TransmissionModeTiming {
    /// cyclic timing parameters
    pub cyclic_timing: Option<CyclicTiming>,
    /// event controlled timing parameters
    pub event_controlled_timing: Option<EventControlledTiming>,
}

/// Cyclic timing parameters for an IPDU
#[derive(Debug, Clone, PartialEq)]
pub struct CyclicTiming {
    /// period of repetition in seconds
    pub time_period: f64,
    /// delay until the first transmission of the PDU in seconds
    pub time_offset: Option<f64>,
}

/// Event controlled timing parameters for an IPDU
#[derive(Debug, Clone, PartialEq)]
pub struct EventControlledTiming {
    /// The PDU will be sent (number of repetitions + 1) times. If number of repetitions is 0, then the PDU is sent exactly once.
    pub number_of_repetitions: u32,
    /// time in seconds between two transmissions of the PDU
    pub repetition_period: Option<f64>,
}

//##################################################################

/// Helper struct to validate signal mappings
pub struct SignalMappingValidator {
    bitmap: Vec<u8>,
}

impl SignalMappingValidator {
    /// Create a new validator for a PDU with the given length
    #[must_use]
    pub fn new(length: u32) -> Self {
        Self {
            bitmap: vec![0; length as usize],
        }
    }

    /// add a signal to the validator
    ///
    /// This will mark the bits in the bitmap that are used by the signal.
    /// If the signal overlaps with any previously added signal, then the method will return false.
    pub fn add_signal(
        &mut self,
        bit_position: u32,
        bit_length: u64,
        byte_order: ByteOrder,
        update_bit: Option<u32>,
    ) -> bool {
        let bit_position = u64::from(bit_position);
        let first_byte = (bit_position / 8) as usize;
        let bit_offset = bit_position % 8; // bit position inside the first byte
        let first_byte_bits; // number of bits in the first byte
        let mut first_mask;

        if byte_order == ByteOrder::MostSignificantByteFirst {
            // MostSignificantByteFirst / big endian
            // bit-position: 5, length: 10
            // byte   |               0               |               1               |
            // bit    | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
            // signal | 4   5   6   7   8   9                           0   1   2   3
            first_byte_bits = (bit_offset + 1).min(bit_length);
            first_mask = ((1u16 << (bit_offset + 1)) - 1) as u8;
            if bit_offset + 1 != first_byte_bits {
                let pos2 = bit_offset - first_byte_bits;
                let subtract_mask = (1u8 << pos2) - 1;
                first_mask -= subtract_mask;
            }
        } else {
            // MostSignificantByteLast / little endian
            // bit-position: 5, length: 10
            // byte   |               0               |               1               |
            // bit    | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
            // signal |                     0   1   2   3   4   5   6   7   8   9
            first_byte_bits = (8 - bit_offset).min(bit_length); // value range 1 - 8
            first_mask = !((1u16 << bit_offset) - 1) as u8; // 0b1111_1110, 0b1111_1100, 0b1111_1000, ..., 0b1000_0000
            if bit_offset + first_byte_bits < 8 {
                let pos2 = bit_offset + first_byte_bits;
                let subtract_mask = !((1u8 << pos2) - 1);
                first_mask -= subtract_mask;
            }
        }
        let full_bytes = (bit_length - first_byte_bits) as usize / 8;
        let end_bits = (bit_length - first_byte_bits) % 8;

        let mut result = self.apply_mask(first_mask, first_byte);
        result &= self.apply_full_bytes(first_byte + 1, full_bytes);

        // handle any bits in a partial trailing byte
        if end_bits > 0 {
            let end_mask = if byte_order == ByteOrder::MostSignificantByteFirst {
                !((1u8 << end_bits) - 1)
            } else {
                (1u8 << end_bits) - 1
            };
            result &= self.apply_mask(end_mask, first_byte + full_bytes + 1);
        }

        // handle the update bit, if any
        if let Some(update_bit) = update_bit {
            let position = (update_bit / 8) as usize;
            let bit_pos = update_bit % 8;
            let mask = 1 << bit_pos;
            result &= self.apply_mask(mask, position);
        }

        result
    }

    fn apply_mask(&mut self, mask: u8, position: usize) -> bool {
        if position < self.bitmap.len() {
            // check if any of the masked bits were already set
            let result = self.bitmap[position] & mask == 0;
            // set the masked bits
            self.bitmap[position] |= mask;
            result
        } else {
            false
        }
    }

    fn apply_full_bytes(&mut self, position: usize, count: usize) -> bool {
        let mut result = true;
        if count > 0 {
            let limit = self.bitmap.len().min(position + count);
            for idx in position..limit {
                result &= self.apply_mask(0xff, idx);
            }
            // make sure all of the signal bytes are inside the pdu length
            result &= limit == position + count;
        }
        result
    }
}

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::{AutosarModelAbstraction, ByteOrder, SystemCategory};
    use autosar_data::AutosarVersion;

    #[test]
    fn isignal_ipdu() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let package = model.get_or_create_package("/pkg").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();

        let pdu = system.create_isignal_ipdu("isignal_ipdu", &package, 8).unwrap();
        assert_eq!(pdu.name().unwrap(), "isignal_ipdu");
        assert_eq!(pdu.length().unwrap(), 8);

        // create a signal and map it to the PDU
        let syssignal = package.create_system_signal("syssignal").unwrap();
        let isignal = system.create_isignal("isignal", &package, 4, &syssignal, None).unwrap();
        let mapping = pdu
            .map_signal(
                &isignal,
                0,
                ByteOrder::MostSignificantByteFirst,
                Some(5),
                TransferProperty::Triggered,
            )
            .unwrap();
        assert_eq!(mapping.signal().unwrap(), isignal);
        assert_eq!(mapping.start_position().unwrap(), 0);
        assert_eq!(mapping.update_bit(), Some(5));
        assert_eq!(mapping.byte_order().unwrap(), ByteOrder::MostSignificantByteFirst);
        mapping.set_byte_order(ByteOrder::MostSignificantByteLast).unwrap();
        assert_eq!(mapping.byte_order().unwrap(), ByteOrder::MostSignificantByteLast);
        assert_eq!(mapping.transfer_property().unwrap(), TransferProperty::Triggered);
        mapping.set_transfer_property(TransferProperty::Pending).unwrap();
        assert_eq!(mapping.transfer_property().unwrap(), TransferProperty::Pending);

        // create a signal group which contains a signal
        let syssignal_group = package.create_system_signal_group("syssignal_group").unwrap();
        let signal_group = system
            .create_isignal_group("signal_group", &package, &syssignal_group)
            .unwrap();
        let grouped_syssignal = package.create_system_signal("groups_syssignal").unwrap();
        syssignal_group.add_signal(&grouped_syssignal).unwrap();
        let grouped_isignal = system
            .create_isignal("grouped_isignal", &package, 4, &grouped_syssignal, None)
            .unwrap();
        signal_group.add_signal(&grouped_isignal).unwrap();
        assert_eq!(grouped_isignal.signal_group().unwrap(), signal_group);

        // map the signal to the PDU - this should fail, because the signal is part of an unmapped signal group
        let result = pdu.map_signal(
            &grouped_isignal,
            9,
            ByteOrder::MostSignificantByteFirst,
            None,
            TransferProperty::Triggered,
        );
        assert!(result.is_err());

        // map the signal group to the PDU
        let mapping = pdu.map_signal_group(&signal_group).unwrap();
        assert_eq!(mapping.signal_group().unwrap(), signal_group);

        // map the grouped signal to the PDU - this should now work
        let _mapping = pdu
            .map_signal(
                &grouped_isignal,
                9,
                ByteOrder::MostSignificantByteFirst,
                None,
                TransferProperty::Triggered,
            )
            .unwrap();
    }

    #[test]
    fn validate_signal_mapping() {
        // create a validator and add a 2-bit signal
        let mut validator = SignalMappingValidator::new(4);
        let result = validator.add_signal(0, 2, ByteOrder::MostSignificantByteLast, None);
        assert!(result);
        assert_eq!(validator.bitmap[0], 0x03);

        // create a validator and add a little-endian 9-bit signal
        let mut validator = SignalMappingValidator::new(4);
        let result = validator.add_signal(5, 10, ByteOrder::MostSignificantByteLast, None);
        assert!(result);
        assert_eq!(validator.bitmap[0], 0xE0);
        assert_eq!(validator.bitmap[1], 0x7F);

        // create a validator and add a big-endian 9-bit signal
        let mut validator = SignalMappingValidator::new(4);
        let result = validator.add_signal(5, 10, ByteOrder::MostSignificantByteFirst, None);
        assert!(result);
        assert_eq!(validator.bitmap[0], 0x3F);
        assert_eq!(validator.bitmap[1], 0xF0);

        // add another signal to the existing validator
        let result = validator.add_signal(5, 10, ByteOrder::MostSignificantByteLast, None);
        // it returns false (the signals overlap), but the bitmap is still updated
        assert!(!result);
        assert_eq!(validator.bitmap[0], 0xFF);
        assert_eq!(validator.bitmap[1], 0xFF);

        // create a validator and add a 32-bit signal
        let mut validator = SignalMappingValidator::new(4);
        let result = validator.add_signal(0, 32, ByteOrder::MostSignificantByteLast, None);
        assert!(result);
        assert_eq!(validator.bitmap[0], 0xFF);
        assert_eq!(validator.bitmap[1], 0xFF);
        assert_eq!(validator.bitmap[2], 0xFF);
        assert_eq!(validator.bitmap[3], 0xFF);

        // create a validator and add a big-endian 32-bit signal
        let mut validator = SignalMappingValidator::new(4);
        let result = validator.add_signal(7, 32, ByteOrder::MostSignificantByteFirst, None);
        assert!(result);
        assert_eq!(validator.bitmap[0], 0xFF);
        assert_eq!(validator.bitmap[1], 0xFF);
        assert_eq!(validator.bitmap[2], 0xFF);
        assert_eq!(validator.bitmap[3], 0xFF);

        // multiple mixed signals
        let mut validator = SignalMappingValidator::new(8);
        let result = validator.add_signal(7, 16, ByteOrder::MostSignificantByteFirst, Some(60));
        assert!(result);
        let result = validator.add_signal(16, 3, ByteOrder::MostSignificantByteLast, Some(61));
        assert!(result);
        let result = validator.add_signal(19, 7, ByteOrder::MostSignificantByteLast, Some(62));
        assert!(result);
        let result = validator.add_signal(26, 30, ByteOrder::MostSignificantByteLast, Some(63));
        assert!(result);
        let result = validator.add_signal(59, 4, ByteOrder::MostSignificantByteFirst, None);
        assert!(result);
        assert_eq!(validator.bitmap, [0xFF; 8]);
    }

    #[test]
    fn ipdu_timing() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let package = model.get_or_create_package("/pkg").unwrap();
        let pdu = ISignalIPdu::new("pdu_name", &package, 8).unwrap();

        let timing_spec = IpduTiming {
            minimum_delay: Some(0.1),
            transmission_mode_true_timing: Some(TransmissionModeTiming {
                cyclic_timing: Some(CyclicTiming {
                    time_period: 0.2,
                    time_offset: Some(0.3),
                }),
                event_controlled_timing: Some(EventControlledTiming {
                    number_of_repetitions: 4,
                    repetition_period: Some(0.5),
                }),
            }),
            transmission_mode_false_timing: Some(TransmissionModeTiming {
                cyclic_timing: Some(CyclicTiming {
                    time_period: 0.6,
                    time_offset: Some(0.7),
                }),
                event_controlled_timing: Some(EventControlledTiming {
                    number_of_repetitions: 8,
                    repetition_period: Some(0.9),
                }),
            }),
        };
        pdu.set_timing(&timing_spec).unwrap();
        let timing_spec2 = pdu.timing().unwrap();
        assert_eq!(timing_spec, timing_spec2);
    }
}
