use crate::communication::{
    AbstractPdu, AbstractPhysicalChannel, CommunicationDirection, Pdu, PduTriggering, PhysicalChannel,
};
use crate::{
    AbstractionElement, AutosarAbstractionError, ByteOrder, EcuInstance, IdentifiableAbstractionElement,
    abstraction_element, make_unique_name,
};

mod can;
mod flexray;
// ethernet does not use frames. PDUs are transmitted over SomeIp or static SocketConnections

use autosar_data::{AutosarDataError, Element, ElementName, EnumItem};
pub use can::*;
pub use flexray::*;

//##################################################################

/// A trait for all frame types
pub trait AbstractFrame: AbstractionElement {
    /// The bus-specific frame triggering type
    type FrameTriggeringType: AbstractFrameTriggering;

    /// returns an iterator over all PDUs in the frame
    fn mapped_pdus(&self) -> impl Iterator<Item = PduToFrameMapping> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::PduToFrameMappings)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|elem| PduToFrameMapping::try_from(elem).ok())
    }

    /// Iterator over all [`FrameTriggering`]s using this frame
    fn frame_triggerings(&self) -> Vec<Self::FrameTriggeringType>;

    /// map a PDU to the frame
    fn map_pdu<T: AbstractPdu>(
        &self,
        gen_pdu: &T,
        start_position: u32,
        byte_order: ByteOrder,
        update_bit: Option<u32>,
    ) -> Result<PduToFrameMapping, AutosarAbstractionError>;

    /// set the length of the frame
    fn set_length(&self, length: u32) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::FrameLength)?
            .set_character_data(u64::from(length))?;
        Ok(())
    }

    /// get the length of the frame
    fn length(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::FrameLength)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }
}

//##################################################################

/// A wrapper for CAN and `FlexRay` frames (Ethernet does not use frames)
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Frame {
    /// The frame is a CAN frame
    Can(CanFrame),
    /// The frame is a `FlexRay` frame
    Flexray(FlexrayFrame),
}

impl AbstractionElement for Frame {
    fn element(&self) -> &autosar_data::Element {
        match self {
            Self::Can(cf) => cf.element(),
            Self::Flexray(ff) => ff.element(),
        }
    }
}

impl IdentifiableAbstractionElement for Frame {}

impl AbstractFrame for Frame {
    type FrameTriggeringType = FrameTriggering;

    fn frame_triggerings(&self) -> Vec<FrameTriggering> {
        let model_result = self.element().model();
        let path_result = self.element().path();
        if let (Ok(model), Ok(path)) = (model_result, path_result) {
            model
                .get_references_to(&path)
                .iter()
                .filter_map(|e| {
                    e.upgrade()
                        .and_then(|ref_elem| ref_elem.named_parent().ok().flatten())
                        .and_then(|elem| FrameTriggering::try_from(elem).ok())
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
        let pdu = gen_pdu.clone().into();
        Self::map_pdu_internal(self, &pdu, start_position, byte_order, update_bit)
    }
}

impl TryFrom<Element> for Frame {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::CanFrame => Ok(Self::Can(CanFrame::try_from(element)?)),
            ElementName::FlexrayFrame => Ok(Self::Flexray(FlexrayFrame::try_from(element)?)),
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "Frame".to_string(),
            }),
        }
    }
}

impl Frame {
    fn map_pdu_internal(
        &self,
        pdu: &Pdu,
        start_position: u32,
        byte_order: ByteOrder,
        update_bit: Option<u32>,
    ) -> Result<PduToFrameMapping, AutosarAbstractionError> {
        let pdu_name = pdu
            .name()
            .ok_or(AutosarAbstractionError::InvalidParameter("invalid PDU".to_string()))?;
        for mapping in self.mapped_pdus() {
            // verify that all PDU mappings in this frame use the same byte order
            if let Some(mapped_byte_order) = mapping.byte_order() {
                if mapped_byte_order != byte_order {
                    return Err(AutosarAbstractionError::InvalidParameter(
                        "All mapped PDUs must use the same byte order".to_string(),
                    ));
                }
            }

            // todo? check if the new PDU overlaps any existing ones
        }

        // add a pdu triggering for the newly mapped PDU to each frame triggering of this frame
        for ft in self.frame_triggerings() {
            let pt = ft.add_pdu_triggering(pdu)?;
            for frame_port in ft.frame_ports() {
                if let (Ok(ecu), Some(direction)) = (frame_port.ecu(), frame_port.communication_direction()) {
                    pt.create_pdu_port(&ecu, direction)?;
                }
            }
        }

        // create and return the new mapping
        let model = self.element().model()?;
        let base_path = self.element().path()?;
        let name = make_unique_name(&model, &base_path, &pdu_name);

        let mappings = self
            .element()
            .get_or_create_sub_element(ElementName::PduToFrameMappings)?;

        PduToFrameMapping::new(&name, &mappings, pdu, start_position, byte_order, update_bit)
    }
}

//##################################################################

/// A trait for all frame triggerings
pub trait AbstractFrameTriggering: AbstractionElement {
    /// The frame type triggered by this `FrameTriggering`
    type FrameType: AbstractFrame;

    /// get the frame triggered by this `FrameTriggering`
    #[must_use]
    fn frame(&self) -> Option<Self::FrameType> {
        Self::FrameType::try_from(
            self.element()
                .get_sub_element(ElementName::FrameRef)?
                .get_reference_target()
                .ok()?,
        )
        .ok()
    }

    /// iterate over all frame ports referenced by this frame triggering
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::{*, communication::*};
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// # let package = model.get_or_create_package("/pkg")?;
    /// # let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// # let ecu = system.create_ecu_instance("ECU", &package)?;
    /// # let cluster = system.create_can_cluster("Cluster", &package, None)?;
    /// # let channel = cluster.create_physical_channel("Channel")?;
    /// # let ecu_instance = system.create_ecu_instance("Ecu", &package)?;
    /// # let canctrl = ecu_instance.create_can_communication_controller("CanCtrl")?;
    /// # canctrl.connect_physical_channel("Connector", &channel)?;
    /// let frame = system.create_can_frame("Frame", &package, 8)?;
    /// let frame_triggering = channel.trigger_frame(&frame, 0x100, CanAddressingMode::Standard, CanFrameType::Can20)?;
    /// let frame_port = frame_triggering.connect_to_ecu(&ecu_instance, CommunicationDirection::In)?;
    /// for fp in frame_triggering.frame_ports() {
    ///    // ...
    /// }
    /// assert_eq!(frame_triggering.frame_ports().count(), 1);
    /// # Ok(())}
    /// ```
    fn frame_ports(&self) -> impl Iterator<Item = FramePort> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::FramePortRefs)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|fpref| {
                fpref
                    .get_reference_target()
                    .ok()
                    .and_then(|fp| FramePort::try_from(fp).ok())
            })
    }

    /// iterate over all PDU triggerings used by this frame triggering
    fn pdu_triggerings(&self) -> impl Iterator<Item = PduTriggering> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::PduTriggerings)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|element| {
                element
                    .get_sub_element(ElementName::PduTriggeringRef)
                    .and_then(|ptr| ptr.get_reference_target().ok())
                    .and_then(|ptelem| PduTriggering::try_from(ptelem).ok())
            })
    }

    /// get the physical channel that contains this frame triggering
    fn physical_channel(&self) -> Result<PhysicalChannel, AutosarAbstractionError> {
        let channel_elem = self.element().named_parent()?.ok_or(AutosarDataError::ItemDeleted)?;
        PhysicalChannel::try_from(channel_elem)
    }
}

//##################################################################

/// A wrapper for CAN and `FlexRay` frame triggerings
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FrameTriggering {
    /// a CAN frame triggering
    Can(CanFrameTriggering),
    /// a `FlexRay` frame triggering
    Flexray(FlexrayFrameTriggering),
}

impl AbstractionElement for FrameTriggering {
    fn element(&self) -> &autosar_data::Element {
        match self {
            Self::Can(cft) => cft.element(),
            Self::Flexray(fft) => fft.element(),
        }
    }
}

impl IdentifiableAbstractionElement for FrameTriggering {}

impl AbstractFrameTriggering for FrameTriggering {
    type FrameType = Frame;
}

impl TryFrom<Element> for FrameTriggering {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::CanFrameTriggering => Ok(CanFrameTriggering::try_from(element)?.into()),
            ElementName::FlexrayFrameTriggering => Ok(FlexrayFrameTriggering::try_from(element)?.into()),
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "FrameTriggering".to_string(),
            }),
        }
    }
}

impl FrameTriggering {
    /// connect this `FrameTriggering` to an `EcuInstance`
    ///
    /// The `EcuInstance` must already be connected to the `PhysicalChannel` that contains the `FrameTriggering`.
    pub fn connect_to_ecu(
        &self,
        ecu: &EcuInstance,
        direction: CommunicationDirection,
    ) -> Result<FramePort, AutosarAbstractionError> {
        for frame_port in self.frame_ports() {
            if let (Ok(existing_ecu), Some(existing_direction)) =
                (frame_port.ecu(), frame_port.communication_direction())
            {
                if existing_ecu == *ecu && existing_direction == direction {
                    return Ok(frame_port);
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
        let fp_elem = connector
            .element()
            .get_or_create_sub_element(ElementName::EcuCommPortInstances)?
            .create_named_sub_element(ElementName::FramePort, &port_name)?;
        fp_elem
            .create_sub_element(ElementName::CommunicationDirection)?
            .set_character_data::<EnumItem>(direction.into())?;

        self.element()
            .get_or_create_sub_element(ElementName::FramePortRefs)?
            .create_sub_element(ElementName::FramePortRef)?
            .set_reference_target(&fp_elem)?;

        for pt in self.pdu_triggerings() {
            pt.create_pdu_port(ecu, direction)?;
        }

        Ok(FramePort(fp_elem))
    }

    fn add_pdu_triggering(&self, pdu: &Pdu) -> Result<PduTriggering, AutosarAbstractionError> {
        let channel = self.physical_channel()?;
        let pt = PduTriggering::new(pdu, &channel)?;
        let triggerings = self.element().get_or_create_sub_element(ElementName::PduTriggerings)?;
        triggerings
            .create_sub_element(ElementName::PduTriggeringRefConditional)?
            .create_sub_element(ElementName::PduTriggeringRef)?
            .set_reference_target(pt.element())?;

        for frame_port in self.frame_ports() {
            if let (Ok(ecu), Some(direction)) = (frame_port.ecu(), frame_port.communication_direction()) {
                pt.create_pdu_port(&ecu, direction)?;
            }
        }

        Ok(pt)
    }
}

//##################################################################

/// `PduToFrameMapping` connects a PDU to a frame
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PduToFrameMapping(Element);
abstraction_element!(PduToFrameMapping, PduToFrameMapping);
impl IdentifiableAbstractionElement for PduToFrameMapping {}

impl PduToFrameMapping {
    fn new(
        name: &str,
        mappings: &Element,
        pdu: &Pdu,
        start_position: u32,
        byte_order: ByteOrder,
        update_bit: Option<u32>,
    ) -> Result<Self, AutosarAbstractionError> {
        let pdumapping_elem = mappings.create_named_sub_element(ElementName::PduToFrameMapping, name)?;
        pdumapping_elem
            .create_sub_element(ElementName::PduRef)?
            .set_reference_target(pdu.element())?;

        let pdumapping = Self(pdumapping_elem);

        pdumapping.set_byte_order(byte_order)?;
        pdumapping.set_start_position(start_position)?;
        pdumapping.set_update_bit(update_bit)?;

        Ok(pdumapping)
    }

    /// Reference to the PDU that is mapped into the frame. The PDU reference is mandatory.
    #[must_use]
    pub fn pdu(&self) -> Option<Pdu> {
        self.element()
            .get_sub_element(ElementName::PduRef)
            .and_then(|pduref| pduref.get_reference_target().ok())
            .and_then(|pdu_elem| Pdu::try_from(pdu_elem).ok())
    }

    /// set the byte order of the data in the PDU.
    ///
    /// All `PduToFrameMappings` within a frame must have the same byte order.
    /// PDUs may not use the byte order value `Opaque`.
    ///
    /// Note: If the byte order is swapped, then the start position must be adjusted accordingly.
    pub fn set_byte_order(&self, byte_order: ByteOrder) -> Result<(), AutosarAbstractionError> {
        if byte_order == ByteOrder::Opaque {
            return Err(AutosarAbstractionError::InvalidParameter(
                "Byte order: opaque is not allowed for PDUs".to_string(),
            ));
        }
        self.element()
            .get_or_create_sub_element(ElementName::PackingByteOrder)?
            .set_character_data::<EnumItem>(byte_order.into())?;
        Ok(())
    }

    /// get the byte order of the data in the PDU.
    ///
    /// All `PduToFrameMappings` within a frame must have the same byte order.
    /// PDUs may not use the byte order value `Opaque`.
    #[must_use]
    pub fn byte_order(&self) -> Option<ByteOrder> {
        self.element()
            .get_sub_element(ElementName::PackingByteOrder)
            .and_then(|pbo| pbo.character_data())
            .and_then(|cdata| cdata.enum_value())
            .and_then(|enumval| enumval.try_into().ok())
    }

    /// set the start position of the PDU data within the frame (bit position).
    ///
    /// PDUs are byte aligned.
    /// For little-endian data the values 0, 8, 16, ... are allowed;
    /// for big-endian data the value 7, 15, 23, ... are allowed.
    ///
    /// Note: if you intend to change both the byte order and the start position, then you should change the byte order first.
    /// New values set here must match the configured byte order.
    pub fn set_start_position(&self, start_position: u32) -> Result<(), AutosarAbstractionError> {
        if (self.byte_order() == Some(ByteOrder::MostSignificantByteFirst) && (start_position % 8 != 7))
            || (self.byte_order() == Some(ByteOrder::MostSignificantByteLast) && (start_position % 8 != 0))
        {
            return Err(AutosarAbstractionError::InvalidParameter(
                "PDUs must be byte-aligned".to_string(),
            ));
        }
        self.element()
            .get_or_create_sub_element(ElementName::StartPosition)?
            .set_character_data(u64::from(start_position))?;
        Ok(())
    }

    /// Start position of the PDU data within the frame (bit position). The start position is mandatory.
    ///
    /// PDUs are byte aligned.
    /// For little-endian data the values 0, 8, 16, ... are allowed;
    /// for big-endian data the value 7, 15, 23, ... are allowed.
    #[must_use]
    pub fn start_position(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::StartPosition)
            .and_then(|pbo| pbo.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set or clear the bit position of the update bit for the mapped PDU.
    pub fn set_update_bit(&self, update_bit: Option<u32>) -> Result<(), AutosarAbstractionError> {
        if let Some(update_bit) = update_bit {
            self.element()
                .get_or_create_sub_element(ElementName::UpdateIndicationBitPosition)?
                .set_character_data(u64::from(update_bit))?;
        } else {
            let _ = self
                .element()
                .remove_sub_element_kind(ElementName::UpdateIndicationBitPosition);
        }
        Ok(())
    }

    /// Bit position of the update bit for the mapped PDU. Not all PDUs use an update bit.
    #[must_use]
    pub fn update_bit(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::UpdateIndicationBitPosition)
            .and_then(|pbo| pbo.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }
}

//##################################################################

/// The `FramePort` allows an ECU to send or receive a frame
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FramePort(Element);
abstraction_element!(FramePort, FramePort);
impl IdentifiableAbstractionElement for FramePort {}

impl FramePort {
    /// get the ECU instance that contains this frame port
    pub fn ecu(&self) -> Result<EcuInstance, AutosarAbstractionError> {
        let comm_connector_elem = self.element().named_parent()?.unwrap();
        let ecu_elem = comm_connector_elem.named_parent()?.unwrap();
        EcuInstance::try_from(ecu_elem)
    }

    /// set the communication direction of the frame port
    pub fn set_communication_direction(
        &self,
        direction: CommunicationDirection,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::CommunicationDirection)?
            .set_character_data::<EnumItem>(direction.into())?;
        Ok(())
    }

    /// get the communication direction of the frame port
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

#[cfg(test)]
mod test {
    use crate::{AutosarModelAbstraction, SystemCategory};

    use super::*;

    #[test]
    fn frame() {
        let model = AutosarModelAbstraction::create("filename", autosar_data::AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();
        let system = package.create_system("System", SystemCategory::SystemExtract).unwrap();

        let can_frame = system.create_can_frame("CanFrame", &package, 8).unwrap();
        let flexray_frame = system.create_flexray_frame("FlexrayFrame", &package, 32).unwrap();

        let frame_1 = Frame::try_from(can_frame.element().clone()).unwrap();
        assert_eq!(frame_1.element().element_name(), autosar_data::ElementName::CanFrame);
        let frame_2 = Frame::try_from(flexray_frame.element().clone()).unwrap();
        assert_eq!(
            frame_2.element().element_name(),
            autosar_data::ElementName::FlexrayFrame
        );

        let err = Frame::try_from(model.root_element().clone());
        assert!(err.is_err());
    }
}
