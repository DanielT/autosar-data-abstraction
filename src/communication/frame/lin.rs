use crate::communication::{
    AbstractFrame, AbstractFrameTriggering, AbstractPdu, CommunicationDirection, Frame, FramePort, FrameTriggering,
    LinPhysicalChannel, Pdu, PduToFrameMapping, PduTriggering,
};
use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, ByteOrder, EcuInstance, IdentifiableAbstractionElement,
    abstraction_element, is_used_system_element, make_unique_name,
};
use autosar_data::{Element, ElementName};

//##################################################################

/// A frame on a LIN bus
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinEventTriggeredFrame(Element);
abstraction_element!(LinEventTriggeredFrame, LinEventTriggeredFrame);
impl IdentifiableAbstractionElement for LinEventTriggeredFrame {}

impl LinEventTriggeredFrame {
    pub(crate) fn new(name: &str, package: &ArPackage, byte_length: u64) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let lin_frame = pkg_elements.create_named_sub_element(ElementName::LinEventTriggeredFrame, name)?;

        lin_frame
            .create_sub_element(ElementName::FrameLength)?
            .set_character_data(byte_length.to_string())?;

        Ok(Self(lin_frame))
    }

    /// remove this `CanFrame` from the model
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

impl AbstractFrame for LinEventTriggeredFrame {
    type FrameTriggeringType = LinFrameTriggering;

    /// List all [`LinFrameTriggering`]s using this frame
    fn frame_triggerings(&self) -> Vec<LinFrameTriggering> {
        let model_result = self.element().model();
        let path_result = self.element().path();
        if let (Ok(model), Ok(path)) = (model_result, path_result) {
            model
                .get_references_to(&path)
                .iter()
                .filter_map(|e| {
                    e.upgrade()
                        .and_then(|ref_elem| ref_elem.named_parent().ok().flatten())
                        .and_then(|elem| LinFrameTriggering::try_from(elem).ok())
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
        Frame::Lin(LinFrame::EventTriggered(self.clone())).map_pdu(gen_pdu, start_position, byte_order, update_bit)
    }
}

//##################################################################

/// A frame on a LIN bus
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinSporadicFrame(Element);
abstraction_element!(LinSporadicFrame, LinSporadicFrame);
impl IdentifiableAbstractionElement for LinSporadicFrame {}

impl LinSporadicFrame {
    pub(crate) fn new(name: &str, package: &ArPackage, byte_length: u64) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let lin_frame = pkg_elements.create_named_sub_element(ElementName::LinSporadicFrame, name)?;

        lin_frame
            .create_sub_element(ElementName::FrameLength)?
            .set_character_data(byte_length.to_string())?;

        Ok(Self(lin_frame))
    }

    /// remove this `CanFrame` from the model
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

impl AbstractFrame for LinSporadicFrame {
    type FrameTriggeringType = LinFrameTriggering;

    /// List all [`LinFrameTriggering`]s using this frame
    fn frame_triggerings(&self) -> Vec<LinFrameTriggering> {
        let model_result = self.element().model();
        let path_result = self.element().path();
        if let (Ok(model), Ok(path)) = (model_result, path_result) {
            model
                .get_references_to(&path)
                .iter()
                .filter_map(|e| {
                    e.upgrade()
                        .and_then(|ref_elem| ref_elem.named_parent().ok().flatten())
                        .and_then(|elem| LinFrameTriggering::try_from(elem).ok())
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
        Frame::Lin(LinFrame::Sporadic(self.clone())).map_pdu(gen_pdu, start_position, byte_order, update_bit)
    }
}

//##################################################################

/// A frame on a LIN bus
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinUnconditionalFrame(Element);
abstraction_element!(LinUnconditionalFrame, LinUnconditionalFrame);
impl IdentifiableAbstractionElement for LinUnconditionalFrame {}

impl LinUnconditionalFrame {
    pub(crate) fn new(name: &str, package: &ArPackage, byte_length: u64) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let lin_frame = pkg_elements.create_named_sub_element(ElementName::LinUnconditionalFrame, name)?;

        lin_frame
            .create_sub_element(ElementName::FrameLength)?
            .set_character_data(byte_length.to_string())?;

        Ok(Self(lin_frame))
    }

    /// remove this `CanFrame` from the model
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

impl AbstractFrame for LinUnconditionalFrame {
    type FrameTriggeringType = LinFrameTriggering;

    /// List all [`LinFrameTriggering`]s using this frame
    fn frame_triggerings(&self) -> Vec<LinFrameTriggering> {
        let model_result = self.element().model();
        let path_result = self.element().path();
        if let (Ok(model), Ok(path)) = (model_result, path_result) {
            model
                .get_references_to(&path)
                .iter()
                .filter_map(|e| {
                    e.upgrade()
                        .and_then(|ref_elem| ref_elem.named_parent().ok().flatten())
                        .and_then(|elem| LinFrameTriggering::try_from(elem).ok())
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
        Frame::Lin(LinFrame::Unconditional(self.clone())).map_pdu(gen_pdu, start_position, byte_order, update_bit)
    }
}

//##################################################################

/// A frame on a LIN bus
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LinFrame {
    /// An event triggered LIN frame
    EventTriggered(LinEventTriggeredFrame),
    /// A sporadic LIN frame
    Sporadic(LinSporadicFrame),
    /// An unconditional LIN frame
    Unconditional(LinUnconditionalFrame),
}

impl AbstractionElement for LinFrame {
    fn element(&self) -> &autosar_data::Element {
        match self {
            LinFrame::EventTriggered(ftf) => ftf.element(),
            LinFrame::Sporadic(fs) => fs.element(),
            LinFrame::Unconditional(fu) => fu.element(),
        }
    }
}
impl IdentifiableAbstractionElement for LinFrame {}

impl TryFrom<Element> for LinFrame {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::LinEventTriggeredFrame => {
                Ok(LinFrame::EventTriggered(LinEventTriggeredFrame::try_from(element)?))
            }
            ElementName::LinSporadicFrame => Ok(LinFrame::Sporadic(LinSporadicFrame::try_from(element)?)),
            ElementName::LinUnconditionalFrame => {
                Ok(LinFrame::Unconditional(LinUnconditionalFrame::try_from(element)?))
            }
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "LinFrame".to_string(),
            }),
        }
    }
}

impl From<LinEventTriggeredFrame> for LinFrame {
    fn from(frame: LinEventTriggeredFrame) -> Self {
        LinFrame::EventTriggered(frame)
    }
}

impl From<LinSporadicFrame> for LinFrame {
    fn from(frame: LinSporadicFrame) -> Self {
        LinFrame::Sporadic(frame)
    }
}

impl From<LinUnconditionalFrame> for LinFrame {
    fn from(frame: LinUnconditionalFrame) -> Self {
        LinFrame::Unconditional(frame)
    }
}

impl AbstractFrame for LinFrame {
    type FrameTriggeringType = LinFrameTriggering;

    /// List all [`LinFrameTriggering`]s using this frame
    fn frame_triggerings(&self) -> Vec<LinFrameTriggering> {
        match self {
            LinFrame::EventTriggered(ftf) => ftf.frame_triggerings(),
            LinFrame::Sporadic(fs) => fs.frame_triggerings(),
            LinFrame::Unconditional(fu) => fu.frame_triggerings(),
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
        match self {
            LinFrame::EventTriggered(ftf) => ftf.map_pdu(gen_pdu, start_position, byte_order, update_bit),
            LinFrame::Sporadic(fs) => fs.map_pdu(gen_pdu, start_position, byte_order, update_bit),
            LinFrame::Unconditional(fu) => fu.map_pdu(gen_pdu, start_position, byte_order, update_bit),
        }
    }
}

impl LinFrame {
    /// remove this `LinFrame` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        match self {
            LinFrame::EventTriggered(ftf) => ftf.remove(deep),
            LinFrame::Sporadic(fs) => fs.remove(deep),
            LinFrame::Unconditional(fu) => fu.remove(deep),
        }
    }
}

//##################################################################

/// The frame triggering connects a frame to a physical channel
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinFrameTriggering(Element);
abstraction_element!(LinFrameTriggering, LinFrameTriggering);
impl IdentifiableAbstractionElement for LinFrameTriggering {}

impl LinFrameTriggering {
    pub(crate) fn new(
        channel: &LinPhysicalChannel,
        frame: &LinFrame,
        identifier: u32,
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
        let lin_triggering = frame_triggerings.create_named_sub_element(ElementName::LinFrameTriggering, &ft_name)?;

        lin_triggering
            .create_sub_element(ElementName::FrameRef)?
            .set_reference_target(frame.element())?;

        let ft = Self(lin_triggering);
        ft.set_identifier(identifier)?;

        for pdu_mapping in frame.mapped_pdus() {
            if let Some(pdu) = pdu_mapping.pdu() {
                ft.add_pdu_triggering(&pdu)?;
            }
        }

        Ok(ft)
    }

    /// remove this `CanFrameTriggering` from the model
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

    /// set the can id associated with this frame
    pub fn set_identifier(&self, identifier: u32) -> Result<(), AutosarAbstractionError> {
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

    pub(crate) fn add_pdu_triggering(&self, pdu: &Pdu) -> Result<PduTriggering, AutosarAbstractionError> {
        FrameTriggering::Lin(self.clone()).add_pdu_triggering(pdu)
    }

    /// get the physical channel that contains this frame triggering
    pub fn physical_channel(&self) -> Result<LinPhysicalChannel, AutosarAbstractionError> {
        let channel_elem = self.element().named_parent()?.unwrap();
        LinPhysicalChannel::try_from(channel_elem)
    }

    /// connect this frame triggering to an ECU
    ///
    /// The direction parameter specifies if the communication is incoming or outgoing
    pub fn connect_to_ecu(
        &self,
        ecu: &EcuInstance,
        direction: CommunicationDirection,
    ) -> Result<FramePort, AutosarAbstractionError> {
        FrameTriggering::Lin(self.clone()).connect_to_ecu(ecu, direction)
    }
}

impl AbstractFrameTriggering for LinFrameTriggering {
    type FrameType = LinFrame;
}

impl From<LinFrameTriggering> for FrameTriggering {
    fn from(cft: LinFrameTriggering) -> Self {
        FrameTriggering::Lin(cft)
    }
}

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::{AutosarModelAbstraction, SystemCategory, communication::AbstractPhysicalChannel};
    use autosar_data::AutosarVersion;

    #[test]
    fn create_frames() {
        let model = AutosarModelAbstraction::create("test", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        let event_triggered_frame = LinEventTriggeredFrame::new("EventTriggeredFrame", &package, 8).unwrap();
        assert_eq!(
            event_triggered_frame.element().element_name(),
            ElementName::LinEventTriggeredFrame
        );

        let sporadic_frame = LinSporadicFrame::new("SporadicFrame", &package, 8).unwrap();
        assert_eq!(sporadic_frame.element().element_name(), ElementName::LinSporadicFrame);

        let unconditional_frame = LinUnconditionalFrame::new("UnconditionalFrame", &package, 8).unwrap();
        assert_eq!(
            unconditional_frame.element().element_name(),
            ElementName::LinUnconditionalFrame
        );
    }

    #[test]
    fn remove_frame_triggering() {
        let model = AutosarModelAbstraction::create("test", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();
        let system = package.create_system("System", SystemCategory::EcuExtract).unwrap();
        let lin_cluster = system.create_lin_cluster("Cluster", &package).unwrap();
        let channel = lin_cluster.create_physical_channel("Channel").unwrap();

        let frame = system.create_lin_unconditional_frame("frame", &package, 8).unwrap();
        let pdu = system.create_isignal_ipdu("pdu", &package, 8).unwrap();

        let frame_triggering = channel.trigger_frame(&frame, 1).unwrap();

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
        let frame_triggering = channel.trigger_frame(&frame, 2).unwrap();
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
        let lin_cluster = system.create_lin_cluster("Cluster", &package).unwrap();
        let channel = lin_cluster.create_physical_channel("Channel").unwrap();
        let frame = system.create_lin_unconditional_frame("frame", &package, 8).unwrap();
        let pdu = system.create_isignal_ipdu("pdu", &package, 8).unwrap();
        let frame_triggering = channel.trigger_frame(&frame, 1).unwrap();
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
