use crate::communication::{
    AbstractIpdu, AbstractPdu, AbstractPhysicalChannel, IPdu, Pdu, PduToFrameMapping, PduTriggering,
};
use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element,
    get_reference_parents,
};
use autosar_data::{Element, ElementName};

//##################################################################

/// Wraps an `IPdu` to protect it from unauthorized manipulation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SecuredIPdu(Element);
abstraction_element!(SecuredIPdu, SecuredIPdu);
impl IdentifiableAbstractionElement for SecuredIPdu {}

impl SecuredIPdu {
    pub(crate) fn new(
        name: &str,
        package: &ArPackage,
        length: u32,
        secure_props: &SecureCommunicationProps,
    ) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_pdu = pkg_elements.create_named_sub_element(ElementName::SecuredIPdu, name)?;
        let secured_ipdu = Self(elem_pdu);
        secured_ipdu.set_length(length)?;
        secured_ipdu.set_secure_communication_props(secure_props)?;

        Ok(secured_ipdu)
    }

    /// remove this `SecuredIPdu` from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        let opt_pdu_triggering = self.payload_pdu_triggering();

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

        // if there is a payload pdu triggering, remove it too
        if let Some(pdu_triggering) = opt_pdu_triggering {
            pdu_triggering.remove(deep)?;
        }

        Ok(())
    }

    /// set the properties of the secured communication
    pub fn set_secure_communication_props(
        &self,
        props: &SecureCommunicationProps,
    ) -> Result<(), AutosarAbstractionError> {
        SecureCommunicationProps::set_props(self.element(), props)
    }

    /// get the properties of the secured communication
    #[must_use]
    pub fn secure_communication_props(&self) -> Option<SecureCommunicationProps> {
        SecureCommunicationProps::get_props(self.element())
    }

    /// set or remove the `useAsCryptographicIPdu` flag
    pub fn set_use_as_cryptographic_ipdu(&self, value: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(value) = value {
            self.element()
                .get_or_create_sub_element(ElementName::UseAsCryptographicIPdu)?
                .set_character_data(value.to_string())?;
        } else {
            let _ = self
                .element()
                .remove_sub_element_kind(ElementName::UseAsCryptographicIPdu);
        }
        Ok(())
    }

    /// get the `useAsCryptographicIPdu` flag
    #[must_use]
    pub fn use_as_cryptographic_ipdu(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::UseAsCryptographicIPdu)?
            .character_data()?
            .parse_bool()
    }

    /// set the payload `PduTriggering` based on an `IPdu`
    ///
    /// This function should be used when `useAsCryptographicIPdu` is false or not set.
    /// A `PduTriggering` is created for the `Pdu`
    pub fn set_payload_ipdu<T: AbstractIpdu + AbstractPdu, U: AbstractPhysicalChannel>(
        &self,
        ipdu: &T,
        physical_channel: &U,
    ) -> Result<PduTriggering, AutosarAbstractionError> {
        if let Some(ppt) = self.payload_pdu_triggering() {
            ppt.remove(false)?;
        }
        let pdu_triggering = PduTriggering::new(&ipdu.clone().into(), &physical_channel.clone().into())?;
        self.0
            .get_or_create_sub_element(ElementName::PayloadRef)?
            .set_reference_target(pdu_triggering.element())?;

        Ok(pdu_triggering)
    }

    /// set the payload `PduTriggering` with an existing `PduTriggering`
    ///
    /// This function should be used when useAsCryptographicIPdu is true.
    /// In this case the payload is transmitted separately from the
    /// cryptographic data, so the `PduTriggering` already exists.
    pub fn set_payload_pdu_triggering(&self, pdu_triggering: &PduTriggering) -> Result<(), AutosarAbstractionError> {
        if let Some(ppt) = self.payload_pdu_triggering()
            && ppt != *pdu_triggering
        {
            ppt.remove(false)?;
        }
        self.0
            .get_or_create_sub_element(ElementName::PayloadRef)?
            .set_reference_target(pdu_triggering.element())?;
        Ok(())
    }

    /// get the payload `PduTriggering`
    #[must_use]
    pub fn payload_pdu_triggering(&self) -> Option<PduTriggering> {
        let elem = self.0.get_sub_element(ElementName::PayloadRef)?;
        PduTriggering::try_from(elem.get_reference_target().ok()?).ok()
    }
}

impl AbstractPdu for SecuredIPdu {}

impl AbstractIpdu for SecuredIPdu {}

impl From<SecuredIPdu> for Pdu {
    fn from(value: SecuredIPdu) -> Self {
        Pdu::SecuredIPdu(value)
    }
}

impl From<SecuredIPdu> for IPdu {
    fn from(value: SecuredIPdu) -> Self {
        IPdu::SecuredIPdu(value)
    }
}

//##################################################################

/// The properties of a `SecuredIPdu`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct SecureCommunicationProps {
    /// length in bits of the authentic PDU data
    pub auth_data_freshness_length: Option<u32>,
    /// start position in bits of the authentic PDU data
    pub auth_data_freshness_start_position: Option<u32>,
    /// number of authentication build attempts
    pub authentication_build_attempts: Option<u32>,
    /// number of additional authentication attempts. If this value is zero, the authentication is not repeated
    pub authentication_retries: Option<u32>,
    /// numerical identifier of the secured `IPdu`
    pub data_id: Option<u32>,
    /// id of the freshness value
    pub freshness_value_id: Option<u32>,
    /// message link length in bits
    pub message_link_length: Option<u32>,
    /// message link start position in bits
    pub message_link_position: Option<u32>,
    /// seconday freshness value id
    pub secondary_freshness_value_id: Option<u32>,
    /// length in bytes of the secure area inside the payload pdu
    pub secured_area_length: Option<u32>,
    /// start position in bytes of the secure area inside the payload pdu
    pub secured_area_offset: Option<u32>,
}

impl SecureCommunicationProps {
    pub(crate) fn set_props(
        element: &Element,
        props: &SecureCommunicationProps,
    ) -> Result<(), AutosarAbstractionError> {
        let sub_elem = element.get_or_create_sub_element(ElementName::SecureCommunicationProps)?;
        if let Some(value) = props.auth_data_freshness_length {
            sub_elem
                .create_sub_element(ElementName::AuthDataFreshnessLength)?
                .set_character_data(value as u64)?;
        }
        if let Some(value) = props.auth_data_freshness_start_position {
            sub_elem
                .create_sub_element(ElementName::AuthDataFreshnessStartPosition)?
                .set_character_data(value as u64)?;
        }
        if let Some(value) = props.authentication_build_attempts {
            sub_elem
                .create_sub_element(ElementName::AuthenticationBuildAttempts)?
                .set_character_data(value as u64)?;
        }
        if let Some(value) = props.authentication_retries {
            sub_elem
                .create_sub_element(ElementName::AuthenticationRetries)?
                .set_character_data(value as u64)?;
        }
        if let Some(value) = props.data_id {
            sub_elem
                .create_sub_element(ElementName::DataId)?
                .set_character_data(value as u64)?;
        }
        if let Some(value) = props.freshness_value_id {
            sub_elem
                .create_sub_element(ElementName::FreshnessValueId)?
                .set_character_data(value as u64)?;
        }
        if let Some(value) = props.message_link_length {
            sub_elem
                .create_sub_element(ElementName::MessageLinkLength)?
                .set_character_data(value as u64)?;
        }
        if let Some(value) = props.message_link_position {
            sub_elem
                .create_sub_element(ElementName::MessageLinkPosition)?
                .set_character_data(value as u64)?;
        }
        if let Some(value) = props.secondary_freshness_value_id {
            sub_elem
                .create_sub_element(ElementName::SecondaryFreshnessValueId)?
                .set_character_data(value as u64)?;
        }
        if let Some(value) = props.secured_area_length {
            sub_elem
                .create_sub_element(ElementName::SecuredAreaLength)?
                .set_character_data(value as u64)?;
        }
        if let Some(value) = props.secured_area_offset {
            sub_elem
                .create_sub_element(ElementName::SecuredAreaOffset)?
                .set_character_data(value as u64)?;
        }
        Ok(())
    }

    pub(crate) fn get_props(element: &Element) -> Option<SecureCommunicationProps> {
        let sub_elem = element.get_sub_element(ElementName::SecureCommunicationProps)?;
        Some(SecureCommunicationProps {
            auth_data_freshness_length: sub_elem
                .get_sub_element(ElementName::AuthDataFreshnessLength)
                .and_then(|elem| elem.character_data()?.parse_integer()),
            auth_data_freshness_start_position: sub_elem
                .get_sub_element(ElementName::AuthDataFreshnessStartPosition)
                .and_then(|elem| elem.character_data()?.parse_integer()),
            authentication_build_attempts: sub_elem
                .get_sub_element(ElementName::AuthenticationBuildAttempts)
                .and_then(|elem| elem.character_data()?.parse_integer()),
            authentication_retries: sub_elem
                .get_sub_element(ElementName::AuthenticationRetries)
                .and_then(|elem| elem.character_data()?.parse_integer()),
            data_id: sub_elem
                .get_sub_element(ElementName::DataId)
                .and_then(|elem| elem.character_data()?.parse_integer()),
            freshness_value_id: sub_elem
                .get_sub_element(ElementName::FreshnessValueId)
                .and_then(|elem| elem.character_data()?.parse_integer()),
            message_link_length: sub_elem
                .get_sub_element(ElementName::MessageLinkLength)
                .and_then(|elem| elem.character_data()?.parse_integer()),
            message_link_position: sub_elem
                .get_sub_element(ElementName::MessageLinkPosition)
                .and_then(|elem| elem.character_data()?.parse_integer()),
            secondary_freshness_value_id: sub_elem
                .get_sub_element(ElementName::SecondaryFreshnessValueId)
                .and_then(|elem| elem.character_data()?.parse_integer()),
            secured_area_length: sub_elem
                .get_sub_element(ElementName::SecuredAreaLength)
                .and_then(|elem| elem.character_data()?.parse_integer()),
            secured_area_offset: sub_elem
                .get_sub_element(ElementName::SecuredAreaOffset)
                .and_then(|elem| elem.character_data()?.parse_integer()),
        })
    }
}

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        AutosarModelAbstraction, ByteOrder, SystemCategory,
        communication::{AbstractFrame, CanAddressingMode, CanFrameType},
    };
    use autosar_data::AutosarVersion;

    #[test]
    fn test_secured_ipdu() -> Result<(), AutosarAbstractionError> {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let package = model.get_or_create_package("/pkg1")?;
        let system = package.create_system("System", SystemCategory::SystemExtract)?;
        let can_cluster = system.create_can_cluster("Cluster", &package, None)?;
        let can_channel = can_cluster.create_physical_channel("Channel")?;

        let secure_communication_props = SecureCommunicationProps {
            auth_data_freshness_length: Some(1),
            auth_data_freshness_start_position: Some(2),
            authentication_build_attempts: Some(3),
            authentication_retries: Some(4),
            data_id: Some(5),
            freshness_value_id: Some(6),
            message_link_length: Some(7),
            message_link_position: Some(8),
            secondary_freshness_value_id: Some(9),
            secured_area_length: Some(10),
            secured_area_offset: Some(11),
        };
        let secured_ipdu = system.create_secured_ipdu("SecuredIPdu", &package, 64, &secure_communication_props)?;
        assert_eq!(
            secured_ipdu.secure_communication_props(),
            Some(secure_communication_props)
        );
        assert_eq!(secured_ipdu.use_as_cryptographic_ipdu(), None);
        secured_ipdu.set_use_as_cryptographic_ipdu(Some(true))?;
        assert_eq!(secured_ipdu.use_as_cryptographic_ipdu(), Some(true));
        secured_ipdu.set_use_as_cryptographic_ipdu(Some(false))?;
        assert_eq!(secured_ipdu.use_as_cryptographic_ipdu(), Some(false));
        secured_ipdu.set_use_as_cryptographic_ipdu(None)?;
        assert_eq!(secured_ipdu.use_as_cryptographic_ipdu(), None);

        let payload_ipdu = system.create_isignal_ipdu("PayloadIPdu", &package, 64)?;
        let pdu_triggering = secured_ipdu.set_payload_ipdu(&payload_ipdu, &can_channel)?;
        assert_eq!(secured_ipdu.payload_pdu_triggering(), Some(pdu_triggering));

        let external_ipdu = system.create_isignal_ipdu("ExternalIPdu", &package, 64)?;
        let can_frame = system.create_can_frame("CanFrame", &package, 64)?;
        can_channel
            .trigger_frame(&can_frame, 0x101, CanAddressingMode::Standard, CanFrameType::CanFd)
            .unwrap();
        can_frame
            .map_pdu(&external_ipdu, 0, ByteOrder::MostSignificantByteLast, None)
            .unwrap();
        let external_pdu_triggering = &external_ipdu.pdu_triggerings()[0];
        secured_ipdu.set_payload_pdu_triggering(external_pdu_triggering)?;
        assert_eq!(
            secured_ipdu.payload_pdu_triggering(),
            Some(external_pdu_triggering.clone())
        );

        Ok(())
    }
}
