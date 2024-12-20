use crate::communication::{
    AbstractPdu, CommunicationDirection, FlexrayPhysicalChannel, Frame, FramePort, FrameTriggering, Pdu,
    PduToFrameMapping, PduToFrameMappingIterator, PduTriggering,
};
use crate::{
    abstraction_element, make_unique_name, reflist_iterator, AbstractionElement, ArPackage, AutosarAbstractionError,
    ByteOrder, EcuInstance,
};
use autosar_data::{Element, ElementName, EnumItem};

/// a Flexray frame
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlexrayFrame(Element);
abstraction_element!(FlexrayFrame, FlexrayFrame);

impl FlexrayFrame {
    pub(crate) fn new(name: &str, byte_length: u64, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let fr_frame = pkg_elements.create_named_sub_element(ElementName::FlexrayFrame, name)?;

        fr_frame
            .create_sub_element(ElementName::FrameLength)?
            .set_character_data(byte_length.to_string())?;

        Ok(Self(fr_frame))
    }

    /// returns an iterator over all PDUs in the frame
    #[must_use]
    pub fn mapped_pdus(&self) -> PduToFrameMappingIterator {
        PduToFrameMappingIterator::new(self.element().get_sub_element(ElementName::PduToFrameMappings))
    }

    /// Iterator over all [`FlexrayFrameTriggering`]s using this frame
    pub fn frame_triggerings(&self) -> impl Iterator<Item = FlexrayFrameTriggering> {
        let model_result = self.element().model();
        let path_result = self.element().path();
        if let (Ok(model), Ok(path)) = (model_result, path_result) {
            let reflist = model.get_references_to(&path);
            FlexrayFrameTriggeringsIterator::new(reflist)
        } else {
            FlexrayFrameTriggeringsIterator::new(vec![])
        }
    }

    /// map a PDU to this frame
    pub fn map_pdu<T: AbstractPdu>(
        &self,
        pdu: &T,
        start_position: u32,
        byte_order: ByteOrder,
        update_bit: Option<u32>,
    ) -> Result<PduToFrameMapping, AutosarAbstractionError> {
        Frame::from(self.clone()).map_pdu(pdu, start_position, byte_order, update_bit)
    }
}

impl From<FlexrayFrame> for Frame {
    fn from(frame: FlexrayFrame) -> Self {
        Frame::Flexray(frame)
    }
}

//##################################################################

/// The frame triggering connects a frame to a physical channel
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlexrayFrameTriggering(Element);
abstraction_element!(FlexrayFrameTriggering, FlexrayFrameTriggering);

impl FlexrayFrameTriggering {
    pub(crate) fn new(
        channel: &FlexrayPhysicalChannel,
        frame: &FlexrayFrame,
        slot_id: u16,
        timing: &FlexrayCommunicationCycle,
    ) -> Result<Self, AutosarAbstractionError> {
        let model = channel.element().model()?;
        let base_path = channel.element().path()?;
        let frame_name = frame
            .name()
            .ok_or(AutosarAbstractionError::InvalidParameter("invalid frame".to_string()))?;
        let ft_name = format!("FT_{frame_name}");
        let ft_name = make_unique_name(&model, &base_path, &ft_name);

        let frame_triggerings = channel
            .element()
            .get_or_create_sub_element(ElementName::FrameTriggerings)?;
        let fr_triggering =
            frame_triggerings.create_named_sub_element(ElementName::FlexrayFrameTriggering, &ft_name)?;

        fr_triggering
            .create_sub_element(ElementName::FrameRef)?
            .set_reference_target(frame.element())?;

        let ft = Self(fr_triggering);
        ft.set_slot(slot_id)?;
        ft.set_timing(timing)?;

        for pdu_mapping in frame.mapped_pdus() {
            if let Some(pdu) = pdu_mapping.pdu() {
                ft.add_pdu_triggering(&pdu)?;
            }
        }

        Ok(ft)
    }

    /// set the slot id for the flexray frame triggering
    pub fn set_slot(&self, slot_id: u16) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::AbsolutelyScheduledTimings)?
            .get_or_create_sub_element(ElementName::FlexrayAbsolutelyScheduledTiming)?
            .get_or_create_sub_element(ElementName::SlotId)?
            .set_character_data(slot_id.to_string())?;
        Ok(())
    }

    /// get the slot id of the flexray frame triggering
    ///
    /// In a well-formed file this always returns Some(value); None should only be seen if the file is incomplete.
    #[must_use]
    pub fn slot(&self) -> Option<u16> {
        self.element()
            .get_sub_element(ElementName::AbsolutelyScheduledTimings)?
            .get_sub_element(ElementName::FlexrayAbsolutelyScheduledTiming)?
            .get_sub_element(ElementName::SlotId)?
            .character_data()?
            .parse_integer()
    }

    /// set the timing of the flexray frame
    pub fn set_timing(&self, timing: &FlexrayCommunicationCycle) -> Result<(), AutosarAbstractionError> {
        let timings_elem = self
            .element()
            .get_or_create_sub_element(ElementName::AbsolutelyScheduledTimings)?
            .get_or_create_sub_element(ElementName::FlexrayAbsolutelyScheduledTiming)?
            .get_or_create_sub_element(ElementName::CommunicationCycle)?;
        match timing {
            FlexrayCommunicationCycle::Counter { cycle_counter } => {
                if let Some(repetition) = timings_elem.get_sub_element(ElementName::CycleRepetition) {
                    let _ = timings_elem.remove_sub_element(repetition);
                }
                timings_elem
                    .get_or_create_sub_element(ElementName::CycleCounter)?
                    .get_or_create_sub_element(ElementName::CycleCounter)?
                    .set_character_data(cycle_counter.to_string())?;
            }
            FlexrayCommunicationCycle::Repetition {
                base_cycle,
                cycle_repetition,
            } => {
                if let Some(counter) = timings_elem.get_sub_element(ElementName::CycleCounter) {
                    let _ = timings_elem.remove_sub_element(counter);
                }
                let repetition = timings_elem.get_or_create_sub_element(ElementName::CycleRepetition)?;
                repetition
                    .get_or_create_sub_element(ElementName::BaseCycle)?
                    .set_character_data(base_cycle.to_string())?;
                repetition
                    .get_or_create_sub_element(ElementName::CycleRepetition)?
                    .set_character_data::<EnumItem>((*cycle_repetition).into())?;
            }
        }
        Ok(())
    }

    /// get the timing of the flexray frame
    ///
    /// In a well-formed file this should always return Some(...)
    #[must_use]
    pub fn timing(&self) -> Option<FlexrayCommunicationCycle> {
        let timings = self
            .element()
            .get_sub_element(ElementName::AbsolutelyScheduledTimings)?
            .get_sub_element(ElementName::FlexrayAbsolutelyScheduledTiming)?
            .get_sub_element(ElementName::CommunicationCycle)?;

        if let Some(counter_based) = timings.get_sub_element(ElementName::CycleCounter) {
            let cycle_counter = counter_based
                .get_sub_element(ElementName::CycleCounter)?
                .character_data()?
                .parse_integer()?;
            Some(FlexrayCommunicationCycle::Counter { cycle_counter })
        } else if let Some(repetition) = timings.get_sub_element(ElementName::CycleRepetition) {
            let base_cycle = repetition
                .get_sub_element(ElementName::BaseCycle)?
                .character_data()?
                .parse_integer()?;
            let cycle_repetition = repetition
                .get_sub_element(ElementName::CycleRepetition)?
                .character_data()?
                .enum_value()?
                .try_into()
                .ok()?;

            Some(FlexrayCommunicationCycle::Repetition {
                base_cycle,
                cycle_repetition,
            })
        } else {
            None
        }
    }

    /// get the frame triggered by this `FrameTriggering`
    #[must_use]
    pub fn frame(&self) -> Option<FlexrayFrame> {
        FlexrayFrame::try_from(
            self.element()
                .get_sub_element(ElementName::FrameRef)?
                .get_reference_target()
                .ok()?,
        )
        .ok()
    }

    pub(crate) fn add_pdu_triggering(&self, pdu: &Pdu) -> Result<PduTriggering, AutosarAbstractionError> {
        FrameTriggering::Flexray(self.clone()).add_pdu_triggering(pdu)
    }

    /// connect this frame triggering to an ECU
    ///
    /// The frame triggering may be connected to any number of ECUs.
    pub fn connect_to_ecu(
        &self,
        ecu: &EcuInstance,
        direction: CommunicationDirection,
    ) -> Result<FramePort, AutosarAbstractionError> {
        FrameTriggering::Flexray(self.clone()).connect_to_ecu(ecu, direction)
    }

    /// iterator over all `PduTriggerings` used by this `FrameTriggering`
    pub fn pdu_triggerings(&self) -> impl Iterator<Item = PduTriggering> {
        FrameTriggering::Flexray(self.clone()).pdu_triggerings()
    }
}

impl From<FlexrayFrameTriggering> for FrameTriggering {
    fn from(fft: FlexrayFrameTriggering) -> Self {
        FrameTriggering::Flexray(fft)
    }
}

//##################################################################

/// The timing settings of a Flexray frame
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexrayCommunicationCycle {
    /// The frame is sent every `cycle_counter` cycles
    Counter {
        /// the cycle counter
        cycle_counter: u8,
    },
    /// The frame is sent every `base_cycle` cycles and repeated every `cycle_repetition` cycles
    Repetition {
        /// the base cycle - typically 64
        base_cycle: u8,
        /// the cycle repetition
        cycle_repetition: CycleRepetition,
    },
}

/// The cycle repetition of a Flexray frame, from the Flexray standard
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleRepetition {
    /// 1 - sent every cycle
    C1,
    /// 2 - sent every second cycle
    C2,
    /// 4 - sent every fourth cycle
    C4,
    /// 5 - sent every fifth cycle (Flexray 3.0 only)
    C5,
    /// 8 - sent every eighth cycle
    C8,
    /// 10 - sent every tenth cycle (Flexray 3.0 only)
    C10,
    /// 16 - sent every sixteenth cycle
    C16,
    /// 20 - sent every twentieth cycle (Flexray 3.0 only)
    C20,
    /// 32 - sent every thirty-second cycle
    C32,
    /// 40 - sent every fortieth cycle (Flexray 3.0 only)
    C40,
    /// 50 - sent every fiftieth cycle (Flexray 3.0 only)
    C50,
    /// 64 - sent every sixty-fourth cycle
    C64,
}

impl TryFrom<EnumItem> for CycleRepetition {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<Self, Self::Error> {
        match value {
            EnumItem::CycleRepetition1 => Ok(Self::C1),
            EnumItem::CycleRepetition2 => Ok(Self::C2),
            EnumItem::CycleRepetition4 => Ok(Self::C4),
            EnumItem::CycleRepetition5 => Ok(Self::C5),
            EnumItem::CycleRepetition8 => Ok(Self::C8),
            EnumItem::CycleRepetition10 => Ok(Self::C10),
            EnumItem::CycleRepetition16 => Ok(Self::C16),
            EnumItem::CycleRepetition20 => Ok(Self::C20),
            EnumItem::CycleRepetition32 => Ok(Self::C32),
            EnumItem::CycleRepetition40 => Ok(Self::C40),
            EnumItem::CycleRepetition50 => Ok(Self::C50),
            EnumItem::CycleRepetition64 => Ok(Self::C64),

            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "CycleRepetitionType".to_string(),
            }),
        }
    }
}

impl From<CycleRepetition> for EnumItem {
    fn from(value: CycleRepetition) -> Self {
        match value {
            CycleRepetition::C1 => EnumItem::CycleRepetition1,
            CycleRepetition::C2 => EnumItem::CycleRepetition2,
            CycleRepetition::C4 => EnumItem::CycleRepetition4,
            CycleRepetition::C5 => EnumItem::CycleRepetition5,
            CycleRepetition::C8 => EnumItem::CycleRepetition8,
            CycleRepetition::C10 => EnumItem::CycleRepetition10,
            CycleRepetition::C16 => EnumItem::CycleRepetition16,
            CycleRepetition::C20 => EnumItem::CycleRepetition20,
            CycleRepetition::C32 => EnumItem::CycleRepetition32,
            CycleRepetition::C40 => EnumItem::CycleRepetition40,
            CycleRepetition::C50 => EnumItem::CycleRepetition50,
            CycleRepetition::C64 => EnumItem::CycleRepetition64,
        }
    }
}

//##################################################################

reflist_iterator!(FlexrayFrameTriggeringsIterator, FlexrayFrameTriggering);
