use crate::communication::{
    AbstractFrame, AbstractFrameTriggering, AbstractPdu, Frame, FrameTriggering, PduToFrameMapping,
};
use crate::{
    AbstractionElement, AutosarAbstractionError, ByteOrder, IdentifiableAbstractionElement, abstraction_element,
};
use autosar_data::{Element, ElementName};

//##################################################################

/// A frame on a LIN bus
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinEventTriggeredFrame(Element);
abstraction_element!(LinEventTriggeredFrame, LinEventTriggeredFrame);
impl IdentifiableAbstractionElement for LinEventTriggeredFrame {}

impl LinEventTriggeredFrame {}

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

impl LinSporadicFrame {}

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

impl LinUnconditionalFrame {}

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

//##################################################################

/// The frame triggering connects a frame to a physical channel
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinFrameTriggering(Element);
abstraction_element!(LinFrameTriggering, LinFrameTriggering);
impl IdentifiableAbstractionElement for LinFrameTriggering {}

impl LinFrameTriggering {}

impl AbstractFrameTriggering for LinFrameTriggering {
    type FrameType = LinFrame;
}

impl From<LinFrameTriggering> for FrameTriggering {
    fn from(cft: LinFrameTriggering) -> Self {
        FrameTriggering::Lin(cft)
    }
}
