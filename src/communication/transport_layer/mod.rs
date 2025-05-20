use crate::{AbstractionElement, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element};
use autosar_data::{Element, ElementName};

mod can_tp;
mod doip_tp;
mod flexray_ar_tp;
mod flexray_tp;

pub use can_tp::*;
pub use doip_tp::*;
pub use flexray_ar_tp::*;
pub use flexray_tp::*;

//##################################################################

/// Represents an ECUs transport layer address on the referenced channel
///
/// The `TpAddress` element is used by `FlexrayArTpConfig` and `FlexrayTpConfig`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TpAddress(Element);
abstraction_element!(TpAddress, TpAddress);
impl IdentifiableAbstractionElement for TpAddress {}

impl TpAddress {
    pub(crate) fn new(name: &str, parent: &Element, address: u32) -> Result<Self, AutosarAbstractionError> {
        let tp_address_elem = parent.create_named_sub_element(ElementName::TpAddress, name)?;
        let tp_address = Self(tp_address_elem);
        tp_address.set_address(address)?;

        Ok(tp_address)
    }

    /// set the value of the address
    pub fn set_address(&self, address: u32) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::TpAddress)?
            .set_character_data(u64::from(address))?;
        Ok(())
    }

    /// get the value of the address
    #[must_use]
    pub fn address(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::TpAddress)?
            .character_data()?
            .parse_integer()
    }
}
