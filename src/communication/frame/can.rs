use crate::communication::{
    AbstractFrame, AbstractFrameTriggering, AbstractPdu, CanPhysicalChannel, CommunicationDirection, Frame, FramePort,
    FrameTriggering, Pdu, PduToFrameMapping, PduTriggering,
};
use crate::{
    abstraction_element, make_unique_name, reflist_iterator, AbstractionElement, ArPackage, AutosarAbstractionError,
    ByteOrder, EcuInstance,
};
use autosar_data::{AutosarDataError, Element, ElementName, EnumItem};

/// A frame on a CAN bus
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanFrame(Element);
abstraction_element!(CanFrame, CanFrame);

impl CanFrame {
    pub(crate) fn new(name: &str, byte_length: u64, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let can_frame = pkg_elements.create_named_sub_element(ElementName::CanFrame, name)?;

        can_frame
            .create_sub_element(ElementName::FrameLength)?
            .set_character_data(byte_length.to_string())?;

        Ok(Self(can_frame))
    }
}

impl AbstractFrame for CanFrame {
    type FrameTriggeringType = CanFrameTriggering;

    /// Iterator over all [`CanFrameTriggering`]s using this frame
    fn frame_triggerings(&self) -> impl Iterator<Item = CanFrameTriggering> {
        let model_result = self.element().model();
        let path_result = self.element().path();
        if let (Ok(model), Ok(path)) = (model_result, path_result) {
            let reflist = model.get_references_to(&path);
            CanFrameTriggeringsIterator::new(reflist)
        } else {
            CanFrameTriggeringsIterator::new(vec![])
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
        Frame::Can(self.clone()).map_pdu(gen_pdu, start_position, byte_order, update_bit)
    }
}

//##################################################################

/// The frame triggering connects a frame to a physical channel
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanFrameTriggering(Element);
abstraction_element!(CanFrameTriggering, CanFrameTriggering);

impl CanFrameTriggering {
    pub(crate) fn new(
        channel: &CanPhysicalChannel,
        frame: &CanFrame,
        identifier: u32,
        addressing_mode: CanAddressingMode,
        frame_type: CanFrameType,
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
        let can_triggering = frame_triggerings.create_named_sub_element(ElementName::CanFrameTriggering, &ft_name)?;

        can_triggering
            .create_sub_element(ElementName::FrameRef)?
            .set_reference_target(frame.element())?;

        let ft = Self(can_triggering);
        ft.set_addressing_mode(addressing_mode)?;
        ft.set_frame_type(frame_type)?;
        if let Err(error) = ft.set_identifier(identifier) {
            let _ = frame_triggerings.remove_sub_element(ft.0);
            return Err(error);
        }

        for pdu_mapping in frame.mapped_pdus() {
            if let Some(pdu) = pdu_mapping.pdu() {
                ft.add_pdu_triggering(&pdu)?;
            }
        }

        Ok(ft)
    }

    /// set the can id associated with this frame
    pub fn set_identifier(&self, identifier: u32) -> Result<(), AutosarAbstractionError> {
        let amode = self.addressing_mode().unwrap_or(CanAddressingMode::Standard);
        if amode == CanAddressingMode::Standard && identifier > 0x7ff {
            return Err(AutosarAbstractionError::InvalidParameter(format!(
                "CAN-ID {identifier} is outside the 11-bit range allowed by standard addressing"
            )));
        } else if identifier > 0x1fff_ffff {
            return Err(AutosarAbstractionError::InvalidParameter(format!(
                "CAN-ID {identifier} is outside the 29-bit range allowed by extended addressing"
            )));
        }
        self.element()
            .get_or_create_sub_element(ElementName::Identifier)?
            .set_character_data(identifier.to_string())?;

        Ok(())
    }

    /// get the can id associated with this frame triggering
    #[must_use]
    pub fn identifier(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::Identifier)?
            .character_data()?
            .parse_integer()
    }

    /// set the addressing mode for this frame triggering
    pub fn set_addressing_mode(&self, addressing_mode: CanAddressingMode) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::CanAddressingMode)?
            .set_character_data::<EnumItem>(addressing_mode.into())?;

        Ok(())
    }

    /// get the addressing mode for this frame triggering
    #[must_use]
    pub fn addressing_mode(&self) -> Option<CanAddressingMode> {
        self.element()
            .get_sub_element(ElementName::CanAddressingMode)?
            .character_data()?
            .enum_value()?
            .try_into()
            .ok()
    }

    /// set the frame type for this frame triggering
    pub fn set_frame_type(&self, frame_type: CanFrameType) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::CanFrameRxBehavior)?
            .set_character_data::<EnumItem>(frame_type.into())?;
        self.element()
            .get_or_create_sub_element(ElementName::CanFrameTxBehavior)?
            .set_character_data::<EnumItem>(frame_type.into())?;

        Ok(())
    }

    /// get the frame type for this frame triggering
    #[must_use]
    pub fn frame_type(&self) -> Option<CanFrameType> {
        self.element()
            .get_sub_element(ElementName::CanFrameTxBehavior)?
            .character_data()?
            .enum_value()?
            .try_into()
            .ok()
    }

    pub(crate) fn add_pdu_triggering(&self, pdu: &Pdu) -> Result<PduTriggering, AutosarAbstractionError> {
        FrameTriggering::Can(self.clone()).add_pdu_triggering(pdu)
    }

    /// get the physical channel that contains this frame triggering
    pub fn physical_channel(&self) -> Result<CanPhysicalChannel, AutosarAbstractionError> {
        let channel_elem = self.element().named_parent()?.ok_or(AutosarDataError::ItemDeleted)?;
        CanPhysicalChannel::try_from(channel_elem)
    }

    /// connect this frame triggering to an ECU
    ///
    /// The direction parameter specifies if the communication is incoming or outgoing
    pub fn connect_to_ecu(
        &self,
        ecu: &EcuInstance,
        direction: CommunicationDirection,
    ) -> Result<FramePort, AutosarAbstractionError> {
        FrameTriggering::Can(self.clone()).connect_to_ecu(ecu, direction)
    }
}

impl AbstractFrameTriggering for CanFrameTriggering {
    type FrameType = CanFrame;
}

impl From<CanFrameTriggering> for FrameTriggering {
    fn from(cft: CanFrameTriggering) -> Self {
        FrameTriggering::Can(cft)
    }
}

//##################################################################

/// The addressing mode for a CAN frame
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanAddressingMode {
    /// Standard addressing mode: 11-bit identifier
    Standard,
    /// Extended addressing mode: 29-bit identifier
    Extended,
}

impl TryFrom<EnumItem> for CanAddressingMode {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<Self, Self::Error> {
        match value {
            EnumItem::Standard => Ok(CanAddressingMode::Standard),
            EnumItem::Extended => Ok(CanAddressingMode::Extended),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "CanAddressingMode".to_string(),
            }),
        }
    }
}

impl From<CanAddressingMode> for EnumItem {
    fn from(value: CanAddressingMode) -> Self {
        match value {
            CanAddressingMode::Standard => EnumItem::Standard,
            CanAddressingMode::Extended => EnumItem::Extended,
        }
    }
}

//##################################################################

/// The type of a CAN frame
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanFrameType {
    /// CAN 2.0 frame (max 8 bytes)
    Can20,
    /// CAN FD frame (max 64 bytes, transmitted at the `CanFD` baud rate)
    CanFd,
    /// Any CAN frame
    Any,
}

impl TryFrom<EnumItem> for CanFrameType {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<Self, Self::Error> {
        match value {
            EnumItem::Can20 => Ok(CanFrameType::Can20),
            EnumItem::CanFd => Ok(CanFrameType::CanFd),
            EnumItem::Any => Ok(CanFrameType::Any),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "CanFrameType".to_string(),
            }),
        }
    }
}

impl From<CanFrameType> for EnumItem {
    fn from(value: CanFrameType) -> Self {
        match value {
            CanFrameType::Can20 => EnumItem::Can20,
            CanFrameType::CanFd => EnumItem::CanFd,
            CanFrameType::Any => EnumItem::Any,
        }
    }
}

//##################################################################

reflist_iterator!(CanFrameTriggeringsIterator, CanFrameTriggering);

//##################################################################

#[cfg(test)]
mod test {
    use autosar_data::{AutosarModel, AutosarVersion};

    use super::*;
    use crate::{communication::CanClusterSettings, ByteOrder, SystemCategory};

    #[test]
    fn can_frame() {
        let model = AutosarModel::new();
        let _ = model.create_file("test", AutosarVersion::LATEST).unwrap();
        let package = ArPackage::get_or_create(&model, "/package").unwrap();
        let system = package.create_system("System", SystemCategory::EcuExtract).unwrap();
        let can_cluster = system
            .create_can_cluster("Cluster", &package, &CanClusterSettings::default())
            .unwrap();
        let channel = can_cluster.create_physical_channel("Channel").unwrap();

        let ecu_instance = system.create_ecu_instance("ECU", &package).unwrap();
        let can_controller = ecu_instance.create_can_communication_controller("Controller").unwrap();
        can_controller.connect_physical_channel("connection", &channel).unwrap();

        let pdu1 = system.create_isignal_ipdu("pdu1", &package, 8).unwrap();
        let pdu2 = system.create_isignal_ipdu("pdu2", &package, 8).unwrap();

        // create frames
        let frame1 = system.create_can_frame("frame1", 8, &package).unwrap();
        let frame2 = system.create_can_frame("frame2", 8, &package).unwrap();

        // map a PDU to the frame before it has been connected to the channel
        let mapping1 = frame1
            .map_pdu(&pdu1, 7, ByteOrder::MostSignificantByteFirst, None)
            .unwrap();
        assert!(frame1.mapped_pdus().count() == 1);
        assert_eq!(frame1.mapped_pdus().next().unwrap(), mapping1);

        // trigger both frames
        let frame_triggering1 = channel
            .trigger_frame(&frame1, 0x123, CanAddressingMode::Standard, CanFrameType::Can20)
            .unwrap();
        assert_eq!(frame1.frame_triggerings().count(), 1);
        let frame_triggering2 = channel
            .trigger_frame(&frame2, 0x456, CanAddressingMode::Standard, CanFrameType::Can20)
            .unwrap();
        assert_eq!(frame2.frame_triggerings().count(), 1);

        // try to set an invalid identifier
        let result = frame_triggering1.set_identifier(0xffff_ffff);
        assert!(result.is_err());

        // frame 1 already had a PDU mapped to it before it was connected to the channel, so a pdu triggering should have been created
        assert_eq!(frame_triggering1.pdu_triggerings().count(), 1);
        // frame 2 has no PDUs mapped to it, so it has no PDU triggerings
        assert_eq!(frame_triggering2.pdu_triggerings().count(), 0);

        // map a PDU to frame2 after it has been connected to the channel
        let mapping2 = frame2
            .map_pdu(&pdu2, 7, ByteOrder::MostSignificantByteFirst, None)
            .unwrap();
        assert!(frame2.mapped_pdus().count() == 1);
        assert_eq!(frame2.mapped_pdus().next().unwrap(), mapping2);

        // mapping the PDU to the connected frame should create a PDU triggering
        assert_eq!(frame_triggering2.pdu_triggerings().count(), 1);

        // connect the frame triggerings to an ECU
        let port1 = frame_triggering1
            .connect_to_ecu(&ecu_instance, CommunicationDirection::Out)
            .unwrap();
        let port2 = frame_triggering2
            .connect_to_ecu(&ecu_instance, CommunicationDirection::In)
            .unwrap();

        assert_eq!(frame_triggering1.identifier().unwrap(), 0x123);
        assert_eq!(
            frame_triggering1.addressing_mode().unwrap(),
            CanAddressingMode::Standard
        );
        assert_eq!(frame_triggering1.frame_type().unwrap(), CanFrameType::Can20);
        assert_eq!(frame_triggering1.frame().unwrap(), frame1);
        assert_eq!(frame_triggering1.physical_channel().unwrap(), channel);

        assert_eq!(mapping1.pdu().unwrap(), pdu1.into());
        assert_eq!(mapping1.byte_order().unwrap(), ByteOrder::MostSignificantByteFirst);
        assert_eq!(mapping1.start_position().unwrap(), 7);
        assert_eq!(mapping1.update_bit(), None);

        assert_eq!(port1.ecu().unwrap(), ecu_instance);
        assert_eq!(port1.communication_direction().unwrap(), CommunicationDirection::Out);
        assert_eq!(port2.ecu().unwrap(), ecu_instance);
        assert_eq!(port2.communication_direction().unwrap(), CommunicationDirection::In);
    }
}
