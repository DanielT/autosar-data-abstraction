use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, Element, IdentifiableAbstractionElement,
    abstraction_element,
    datatype::{AbstractAutosarDataType, AutosarDataType, ValueSpecification},
    software_component::AbstractPortInterface,
};
use autosar_data::ElementName;

//##################################################################

/// A `SenderReceiverInterface` defines a set of data elements that can be sent and received
///
/// Use [`ArPackage::create_sender_receiver_interface`] to create a new sender receiver interface
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SenderReceiverInterface(pub(crate) Element);
abstraction_element!(SenderReceiverInterface, SenderReceiverInterface);
impl IdentifiableAbstractionElement for SenderReceiverInterface {}
impl AbstractPortInterface for SenderReceiverInterface {}

impl SenderReceiverInterface {
    /// Create a new `SenderReceiverInterface`
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let sender_receiver_interface =
            elements.create_named_sub_element(ElementName::SenderReceiverInterface, name)?;

        Ok(Self(sender_receiver_interface))
    }

    /// Add a new data element to the sender receiver interface
    pub fn create_data_element<T: AbstractAutosarDataType>(
        &self,
        name: &str,
        data_type: &T,
    ) -> Result<VariableDataPrototype, AutosarAbstractionError> {
        let data_elements = self.element().get_or_create_sub_element(ElementName::DataElements)?;
        VariableDataPrototype::new(name, &data_elements, data_type.element())
    }

    /// iterate over all data elements
    pub fn data_elements(&self) -> impl Iterator<Item = VariableDataPrototype> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::DataElements)
            .into_iter()
            .flat_map(|data_elements| data_elements.sub_elements())
            .filter_map(|elem| VariableDataPrototype::try_from(elem).ok())
    }
}

//##################################################################

/// A `VariableDataPrototype` represents a data element in a `SenderReceiverInterface`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariableDataPrototype(Element);
abstraction_element!(VariableDataPrototype, VariableDataPrototype);
impl IdentifiableAbstractionElement for VariableDataPrototype {}

impl VariableDataPrototype {
    /// Create a new `VariableDataPrototype`
    fn new(name: &str, parent_element: &Element, data_type: &Element) -> Result<Self, AutosarAbstractionError> {
        let vdp = parent_element.create_named_sub_element(ElementName::VariableDataPrototype, name)?;
        vdp.create_sub_element(ElementName::TypeTref)?
            .set_reference_target(data_type)?;

        Ok(Self(vdp))
    }

    /// Get the interface containing the data element
    pub fn interface(&self) -> Result<SenderReceiverInterface, AutosarAbstractionError> {
        let named_parent = self.element().named_parent()?.unwrap();
        SenderReceiverInterface::try_from(named_parent)
    }

    /// Set the data type of the data element
    pub fn set_data_type<T: AbstractAutosarDataType>(&self, data_type: &T) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::TypeTref)?
            .set_reference_target(data_type.element())?;
        Ok(())
    }

    /// Get the data type of the data element
    #[must_use]
    pub fn data_type(&self) -> Option<AutosarDataType> {
        let type_tref = self.element().get_sub_element(ElementName::TypeTref)?;
        AutosarDataType::try_from(type_tref.get_reference_target().ok()?).ok()
    }

    /// Set the init value of the data element
    pub fn set_init_value<T: Into<ValueSpecification>>(
        &self,
        value_spec: Option<T>,
    ) -> Result<(), AutosarAbstractionError> {
        if let Some(value_spec) = value_spec {
            let value_spec: ValueSpecification = value_spec.into();
            let init_value_elem = self.element().get_or_create_sub_element(ElementName::InitValue)?;
            value_spec.store(&init_value_elem)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::InitValue);
        }
        Ok(())
    }

    /// Get the init value of the data element
    #[must_use]
    pub fn init_value(&self) -> Option<ValueSpecification> {
        let init_value_elem = self
            .element()
            .get_sub_element(ElementName::InitValue)?
            .get_sub_element_at(0)?;
        ValueSpecification::load(&init_value_elem)
    }
}

//##################################################################

#[cfg(test)]
mod test {
    use crate::{
        AutosarModelAbstraction,
        datatype::{
            AutosarDataType, BaseTypeEncoding, ImplementationDataTypeSettings, NumericalValueSpecification,
            ValueSpecification,
        },
        software_component::AbstractPortInterface,
    };
    use autosar_data::AutosarVersion;

    #[test]
    fn sender_receiver_interface() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        let sr_interface = package
            .create_sender_receiver_interface("SenderReceiverInterface")
            .unwrap();

        let base_type = package
            .create_sw_base_type("base", 32, BaseTypeEncoding::None, None, None, None)
            .unwrap();
        let impl_settings = ImplementationDataTypeSettings::Value {
            name: "ImplementationValue".to_string(),
            base_type: base_type.clone(),
            compu_method: None,
            data_constraint: None,
        };
        let datatype = package.create_implementation_data_type(&impl_settings).unwrap();
        let impl_settings2 = ImplementationDataTypeSettings::Value {
            name: "ImplementationValue2".to_string(),
            base_type,
            compu_method: None,
            data_constraint: None,
        };
        let datatype2 = package.create_implementation_data_type(&impl_settings2).unwrap();

        let data_element = sr_interface.create_data_element("data_element", &datatype).unwrap();
        assert_eq!(sr_interface.data_elements().count(), 1);
        assert_eq!(data_element.interface().unwrap(), sr_interface);
        assert_eq!(
            data_element.data_type().unwrap(),
            AutosarDataType::ImplementationDataType(datatype)
        );
        data_element.set_data_type(&datatype2).unwrap();
        assert_eq!(
            data_element.data_type().unwrap(),
            AutosarDataType::ImplementationDataType(datatype2)
        );

        let value_spec = NumericalValueSpecification {
            label: None,
            value: 42.0,
        };
        data_element.set_init_value(Some(value_spec)).unwrap();
        assert_eq!(
            data_element.init_value().unwrap(),
            NumericalValueSpecification {
                label: None,
                value: 42.0
            }
            .into()
        );

        data_element.set_init_value::<ValueSpecification>(None).unwrap();
        assert!(data_element.init_value().is_none());

        sr_interface.set_is_service(Some(false)).unwrap();
        assert!(!sr_interface.is_service().unwrap());
        sr_interface.set_is_service(Some(true)).unwrap();
        assert!(sr_interface.is_service().unwrap());
        sr_interface.set_is_service(None).unwrap();
        assert_eq!(sr_interface.is_service(), None);
    }
}
