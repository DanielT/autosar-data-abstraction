use crate::communication::{
    AbstractFrame, AbstractFrameTriggering, AbstractPdu, CommunicationDirection, FlexrayPhysicalChannel, Frame,
    FramePort, FrameTriggering, Pdu, PduToFrameMapping, PduTriggering,
};
use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, ByteOrder, EcuInstance, IdentifiableAbstractionElement,
    abstraction_element, is_used_system_element, make_unique_name,
};
use autosar_data::{Element, ElementName, EnumItem};

//##################################################################

/// a Flexray frame
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlexrayFrame(Element);
abstraction_element!(FlexrayFrame, FlexrayFrame);
impl IdentifiableAbstractionElement for FlexrayFrame {}

impl FlexrayFrame {
    pub(crate) fn new(name: &str, package: &ArPackage, byte_length: u64) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let fr_frame = pkg_elements.create_named_sub_element(ElementName::FlexrayFrame, name)?;

        fr_frame
            .create_sub_element(ElementName::FrameLength)?
            .set_character_data(byte_length.to_string())?;

        Ok(Self(fr_frame))
    }

    /// remove this `FlexrayFrame` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        for pdu_mapping in self.mapped_pdus() {
            pdu_mapping.remove(deep)?;
        }

        // get all frame triggerings using this frame
        let frame_triggerings = self.frame_triggerings();

        // remove the element itself
        AbstractionElement::remove(self, deep)?;

        // remove the frame triggerings
        for ft in frame_triggerings {
            ft.remove(deep)?;
        }

        Ok(())
    }
}

impl AbstractFrame for FlexrayFrame {
    type FrameTriggeringType = FlexrayFrameTriggering;

    /// Iterator over all [`FlexrayFrameTriggering`]s using this frame
    fn frame_triggerings(&self) -> Vec<FlexrayFrameTriggering> {
        let model_result = self.element().model();
        let path_result = self.element().path();
        if let (Ok(model), Ok(path)) = (model_result, path_result) {
            model
                .get_references_to(&path)
                .iter()
                .filter_map(|e| {
                    e.upgrade()
                        .and_then(|ref_elem| ref_elem.named_parent().ok().flatten())
                        .and_then(|elem| FlexrayFrameTriggering::try_from(elem).ok())
                })
                .collect()
        } else {
            vec![]
        }
    }

    /// map a PDU to the frame
    fn map_pdu<T: AbstractPdu>(
        &self,
        gen_pdu: &T,
        start_position: u32,
        byte_order: ByteOrder,
        update_bit: Option<u32>,
    ) -> Result<PduToFrameMapping, AutosarAbstractionError> {
        Frame::Flexray(self.clone()).map_pdu(gen_pdu, start_position, byte_order, update_bit)
    }
}

//##################################################################

/// The frame triggering connects a frame to a physical channel
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlexrayFrameTriggering(Element);
abstraction_element!(FlexrayFrameTriggering, FlexrayFrameTriggering);
impl IdentifiableAbstractionElement for FlexrayFrameTriggering {}

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

    /// remove this `FlexrayFrameTriggering` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        let opt_frame = self.frame();

        // remove all pdu triggerings of this frame triggering
        for pt in self.pdu_triggerings() {
            pt.remove(deep)?;
        }
        for frame_port in self.frame_ports() {
            frame_port.remove(deep)?;
        }

        AbstractionElement::remove(self, deep)?;

        // if deep, check if the frame became unused because of this frame triggering removal
        // if so remove it too
        if deep && let Some(frame) = opt_frame {
            // check if any frame became unused because of this frame triggering removal
            // if so remove it too
            if !is_used_system_element(frame.element()) {
                frame.remove(deep)?;
            }
        }

        Ok(())
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
                let _ = timings_elem.remove_sub_element_kind(ElementName::CycleRepetition);
                timings_elem
                    .get_or_create_sub_element(ElementName::CycleCounter)?
                    .get_or_create_sub_element(ElementName::CycleCounter)?
                    .set_character_data(cycle_counter.to_string())?;
            }
            FlexrayCommunicationCycle::Repetition {
                base_cycle,
                cycle_repetition,
            } => {
                let _ = timings_elem.remove_sub_element_kind(ElementName::CycleCounter);
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

    pub(crate) fn add_pdu_triggering(&self, pdu: &Pdu) -> Result<PduTriggering, AutosarAbstractionError> {
        FrameTriggering::Flexray(self.clone()).add_pdu_triggering(pdu)
    }

    /// get the physical channel that contains this frame triggering
    pub fn physical_channel(&self) -> Result<FlexrayPhysicalChannel, AutosarAbstractionError> {
        let channel_elem = self.element().named_parent()?.unwrap();
        FlexrayPhysicalChannel::try_from(channel_elem)
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
}

impl AbstractFrameTriggering for FlexrayFrameTriggering {
    type FrameType = FlexrayFrame;
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        AutosarModelAbstraction, ByteOrder, SystemCategory,
        communication::{AbstractPhysicalChannel, FlexrayChannelName, FlexrayClusterSettings},
    };
    use autosar_data::AutosarVersion;

    #[test]
    fn fr_frame() {
        let model = AutosarModelAbstraction::create("test", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();
        let system = package.create_system("System", SystemCategory::EcuExtract).unwrap();
        let flexray_cluster = system
            .create_flexray_cluster("Cluster", &package, &FlexrayClusterSettings::default())
            .unwrap();
        let channel = flexray_cluster
            .create_physical_channel("Channel", FlexrayChannelName::A)
            .unwrap();

        let ecu_instance = system.create_ecu_instance("ECU", &package).unwrap();
        let can_controller = ecu_instance
            .create_flexray_communication_controller("Controller")
            .unwrap();
        can_controller.connect_physical_channel("connection", &channel).unwrap();

        let pdu1 = system.create_isignal_ipdu("pdu1", &package, 8).unwrap();
        let pdu2 = system.create_isignal_ipdu("pdu2", &package, 8).unwrap();

        // create two frames
        let frame1 = system.create_flexray_frame("frame1", &package, 64).unwrap();
        let frame2 = system.create_flexray_frame("frame2", &package, 64).unwrap();

        assert_eq!(frame1.length().unwrap(), 64);
        frame1.set_length(60).unwrap();
        assert_eq!(frame1.length().unwrap(), 60);

        // map a PDU to frame1 before it has been connected to the channel
        let mapping = frame1
            .map_pdu(&pdu1, 7, ByteOrder::MostSignificantByteFirst, Some(8))
            .unwrap();
        assert!(frame1.mapped_pdus().count() == 1);
        assert_eq!(frame1.mapped_pdus().next().unwrap(), mapping);

        // trigger both frames
        let frame_triggering1 = channel
            .trigger_frame(
                &frame1,
                1,
                &FlexrayCommunicationCycle::Repetition {
                    base_cycle: 1,
                    cycle_repetition: CycleRepetition::C1,
                },
            )
            .unwrap();
        assert_eq!(frame1.frame_triggerings().len(), 1);
        let frame_triggering2 = channel
            .trigger_frame(&frame2, 2, &FlexrayCommunicationCycle::Counter { cycle_counter: 2 })
            .unwrap();
        assert_eq!(frame2.frame_triggerings().len(), 1);
        assert_eq!(channel.frame_triggerings().count(), 2);

        // a pdu triggering for the mapped pdu should be created when the frame is connected to the channel
        assert_eq!(frame_triggering1.pdu_triggerings().count(), 1);

        // map another PDU to the frame after it has been connected to the channel
        let _ = frame1
            .map_pdu(&pdu2, 71, ByteOrder::MostSignificantByteFirst, None)
            .unwrap();
        assert!(frame1.mapped_pdus().count() == 2);

        // mapping the PDU to the connected frame should create a PDU triggering
        assert_eq!(frame_triggering1.pdu_triggerings().count(), 2);

        // connect the frame triggering to an ECU
        let frame_port = frame_triggering1
            .connect_to_ecu(&ecu_instance, CommunicationDirection::Out)
            .unwrap();
        assert_eq!(frame_port.ecu().unwrap(), ecu_instance);
        assert_eq!(
            frame_port.communication_direction().unwrap(),
            CommunicationDirection::Out
        );
        frame_port.set_name("port").unwrap();
        assert_eq!(frame_port.name().unwrap(), "port");

        assert_eq!(frame_triggering1.frame().unwrap(), frame1);
        assert_eq!(frame_triggering1.slot().unwrap(), 1);
        assert_eq!(
            frame_triggering1.timing().unwrap(),
            FlexrayCommunicationCycle::Repetition {
                base_cycle: 1,
                cycle_repetition: CycleRepetition::C1
            }
        );
        assert_eq!(frame_triggering1.physical_channel().unwrap(), channel);
        assert_eq!(frame_triggering2.frame().unwrap(), frame2);
        assert_eq!(frame_triggering2.slot().unwrap(), 2);
        assert_eq!(
            frame_triggering2.timing().unwrap(),
            FlexrayCommunicationCycle::Counter { cycle_counter: 2 }
        );
        assert_eq!(frame_triggering2.physical_channel().unwrap(), channel);

        assert_eq!(mapping.pdu().unwrap(), pdu1.into());
        assert_eq!(mapping.byte_order().unwrap(), ByteOrder::MostSignificantByteFirst);
        assert_eq!(mapping.start_position().unwrap(), 7);
        assert_eq!(mapping.update_bit(), Some(8));
    }

    #[test]
    fn cycle_repetition() {
        assert_eq!(EnumItem::CycleRepetition1, CycleRepetition::C1.into());
        assert_eq!(EnumItem::CycleRepetition2, CycleRepetition::C2.into());
        assert_eq!(EnumItem::CycleRepetition4, CycleRepetition::C4.into());
        assert_eq!(EnumItem::CycleRepetition5, CycleRepetition::C5.into());
        assert_eq!(EnumItem::CycleRepetition8, CycleRepetition::C8.into());
        assert_eq!(EnumItem::CycleRepetition10, CycleRepetition::C10.into());
        assert_eq!(EnumItem::CycleRepetition16, CycleRepetition::C16.into());
        assert_eq!(EnumItem::CycleRepetition20, CycleRepetition::C20.into());
        assert_eq!(EnumItem::CycleRepetition32, CycleRepetition::C32.into());
        assert_eq!(EnumItem::CycleRepetition40, CycleRepetition::C40.into());
        assert_eq!(EnumItem::CycleRepetition50, CycleRepetition::C50.into());
        assert_eq!(EnumItem::CycleRepetition64, CycleRepetition::C64.into());

        assert_eq!(CycleRepetition::C1, EnumItem::CycleRepetition1.try_into().unwrap());
        assert_eq!(CycleRepetition::C2, EnumItem::CycleRepetition2.try_into().unwrap());
        assert_eq!(CycleRepetition::C4, EnumItem::CycleRepetition4.try_into().unwrap());
        assert_eq!(CycleRepetition::C5, EnumItem::CycleRepetition5.try_into().unwrap());
        assert_eq!(CycleRepetition::C8, EnumItem::CycleRepetition8.try_into().unwrap());
        assert_eq!(CycleRepetition::C10, EnumItem::CycleRepetition10.try_into().unwrap());
        assert_eq!(CycleRepetition::C16, EnumItem::CycleRepetition16.try_into().unwrap());
        assert_eq!(CycleRepetition::C20, EnumItem::CycleRepetition20.try_into().unwrap());
        assert_eq!(CycleRepetition::C32, EnumItem::CycleRepetition32.try_into().unwrap());
        assert_eq!(CycleRepetition::C40, EnumItem::CycleRepetition40.try_into().unwrap());
        assert_eq!(CycleRepetition::C50, EnumItem::CycleRepetition50.try_into().unwrap());
        assert_eq!(CycleRepetition::C64, EnumItem::CycleRepetition64.try_into().unwrap());

        let result: Result<CycleRepetition, _> = EnumItem::Aa.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn remove_frame_triggering() {
        let model = AutosarModelAbstraction::create("test", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();
        let system = package.create_system("System", SystemCategory::EcuExtract).unwrap();
        let flexray_cluster = system
            .create_flexray_cluster("Cluster", &package, &FlexrayClusterSettings::default())
            .unwrap();
        let channel = flexray_cluster
            .create_physical_channel("Channel", FlexrayChannelName::A)
            .unwrap();

        let frame = system.create_flexray_frame("frame", &package, 8).unwrap();
        let pdu = system.create_isignal_ipdu("pdu", &package, 8).unwrap();

        let frame_triggering = channel
            .trigger_frame(&frame, 0x123, &FlexrayCommunicationCycle::Counter { cycle_counter: 1 })
            .unwrap();

        let _mapping = frame
            .map_pdu(&pdu, 0, ByteOrder::MostSignificantByteLast, None)
            .unwrap();

        assert_eq!(frame.mapped_pdus().count(), 1);
        assert_eq!(frame.frame_triggerings().len(), 1);
        assert_eq!(channel.frame_triggerings().count(), 1);

        // remove the frame triggering
        frame_triggering.remove(false).unwrap();
        // the frame remains because we did a shallow removal
        assert_eq!(system.frames().count(), 1);

        // re-create the frame triggering
        let frame_triggering = channel
            .trigger_frame(&frame, 0x123, &FlexrayCommunicationCycle::Counter { cycle_counter: 1 })
            .unwrap();
        // remove the frame triggering with deep=true
        frame_triggering.remove(true).unwrap();

        // the frame triggering should be removed
        assert_eq!(channel.frame_triggerings().count(), 0);
        // the frame should be removed because it became unused
        assert_eq!(system.frames().count(), 0);
        // the mapping should be removed because the frame was removed
        assert_eq!(frame.mapped_pdus().count(), 0);
        // the pdu was also removed, because it became unused and we did a deep removal
        assert_eq!(system.pdus().count(), 0);

        assert_eq!(channel.frame_triggerings().count(), 0);
        assert_eq!(channel.pdu_triggerings().count(), 0);
    }

    #[test]
    fn remove_frame() {
        let model = AutosarModelAbstraction::create("test", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();
        let system = package.create_system("System", SystemCategory::EcuExtract).unwrap();
        let flexray_cluster = system
            .create_flexray_cluster("Cluster", &package, &FlexrayClusterSettings::default())
            .unwrap();
        let channel = flexray_cluster
            .create_physical_channel("Channel", FlexrayChannelName::A)
            .unwrap();
        let frame = system.create_flexray_frame("frame", &package, 8).unwrap();
        let pdu = system.create_isignal_ipdu("pdu", &package, 8).unwrap();
        let frame_triggering = channel
            .trigger_frame(&frame, 0x123, &FlexrayCommunicationCycle::Counter { cycle_counter: 1 })
            .unwrap();
        let mapping = frame
            .map_pdu(&pdu, 0, ByteOrder::MostSignificantByteLast, None)
            .unwrap();
        assert_eq!(frame.mapped_pdus().count(), 1);
        assert_eq!(frame.frame_triggerings().len(), 1);
        assert_eq!(channel.frame_triggerings().count(), 1);
        // remove the frame with deep=false
        frame.remove(false).unwrap();
        // the frame should be removed
        assert_eq!(system.frames().count(), 0);
        // the mapping should be removed
        assert!(mapping.element().path().is_err());
        // the pdu should still exist
        assert_eq!(system.pdus().count(), 1);
        // the frame triggering should be removed
        assert!(frame_triggering.element().path().is_err());
        assert_eq!(channel.frame_triggerings().count(), 0);
        assert_eq!(channel.pdu_triggerings().count(), 0);
    }
}
