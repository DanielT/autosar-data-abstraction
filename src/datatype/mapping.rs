use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, Element, IdentifiableAbstractionElement,
    abstraction_element, datatype,
};
use autosar_data::ElementName;
use datatype::{ApplicationDataType, ImplementationDataType};

/// A [`DataTypeMappingSet`] contains `DataTypeMap`s
///
/// Use [`ArPackage::create_data_type_mapping_set`] to create a new `DataTypeMappingSet`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataTypeMappingSet(Element);
abstraction_element!(DataTypeMappingSet, DataTypeMappingSet);
impl IdentifiableAbstractionElement for DataTypeMappingSet {}

impl DataTypeMappingSet {
    /// Create a new `DataTypeMappingSet`
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let mapping_set = elements.create_named_sub_element(ElementName::DataTypeMappingSet, name)?;

        Ok(Self(mapping_set))
    }

    /// Create a new `DataTypeMap` in the `DataTypeMappingSet`
    pub fn create_data_type_map<T: Into<ApplicationDataType> + Clone>(
        &self,
        implementation_data_type: &ImplementationDataType,
        application_data_type: &T,
    ) -> Result<DataTypeMap, AutosarAbstractionError> {
        let application_data_type = application_data_type.clone().into();
        let data_type_map = DataTypeMap::new(self.element(), implementation_data_type, &application_data_type)?;
        Ok(data_type_map)
    }

    /// Get an iterator over the `DataTypeMap`s in the `DataTypeMappingSet`
    pub fn data_type_maps(&self) -> impl Iterator<Item = DataTypeMap> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::DataTypeMaps)
            .into_iter()
            .flat_map(|maps| maps.sub_elements())
            .filter_map(|elem| DataTypeMap::try_from(elem).ok())
    }
}

//#########################################################

/// A `DataTypeMap` maps an `ImplementationDataType` to an `ApplicationDataType`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataTypeMap(Element);
abstraction_element!(DataTypeMap, DataTypeMap);

impl DataTypeMap {
    /// Create a new `DataTypeMap`
    fn new(
        parent: &Element,
        implementation_data_type: &ImplementationDataType,
        application_data_type: &ApplicationDataType,
    ) -> Result<Self, AutosarAbstractionError> {
        let maps = parent.get_or_create_sub_element(ElementName::DataTypeMaps)?;
        let data_type_map = maps.create_sub_element(ElementName::DataTypeMap)?;

        data_type_map
            .create_sub_element(ElementName::ApplicationDataTypeRef)?
            .set_reference_target(application_data_type.element())?;
        data_type_map
            .create_sub_element(ElementName::ImplementationDataTypeRef)?
            .set_reference_target(implementation_data_type.element())?;

        Ok(Self(data_type_map))
    }

    /// Get the `ImplementationDataType` of the `DataTypeMap`
    #[must_use]
    pub fn implementation_data_type(&self) -> Option<ImplementationDataType> {
        self.element()
            .get_sub_element(ElementName::ImplementationDataTypeRef)?
            .get_reference_target()
            .ok()?
            .try_into()
            .ok()
    }

    /// Get the `ApplicationDataType` of the `DataTypeMap`
    #[must_use]
    pub fn application_data_type(&self) -> Option<ApplicationDataType> {
        self.element()
            .get_sub_element(ElementName::ApplicationDataTypeRef)?
            .get_reference_target()
            .ok()?
            .try_into()
            .ok()
    }
}

//#########################################################

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AutosarModelAbstraction;
    use autosar_data::AutosarVersion;
    use datatype::{
        ApplicationPrimitiveCategory, ApplicationPrimitiveDataType, BaseTypeEncoding, ImplementationDataTypeSettings,
        SwBaseType,
    };

    #[test]
    fn test_data_type_mapping_set() {
        let model = AutosarModelAbstraction::create("test.arxml", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/DataTypeMappingSets").unwrap();
        let mapping_set = DataTypeMappingSet::new("MappingSet", &package).unwrap();

        // create an implementation data type
        let base_type =
            SwBaseType::new("uint8", &package, 8, BaseTypeEncoding::None, None, None, Some("uint8")).unwrap();
        let impl_data_type = ImplementationDataType::new(
            &package,
            &ImplementationDataTypeSettings::Value {
                name: "ImplDataType".to_string(),
                base_type: base_type.clone(),
                compu_method: None,
                data_constraint: None,
            },
        )
        .unwrap();
        // create an application data type
        let app_data_type = ApplicationPrimitiveDataType::new(
            "AppDataType",
            &package,
            ApplicationPrimitiveCategory::Value,
            None,
            None,
            None,
        )
        .unwrap()
        .into();

        let data_type_map = mapping_set
            .create_data_type_map(&impl_data_type, &app_data_type)
            .unwrap();

        assert_eq!(data_type_map.implementation_data_type().unwrap(), impl_data_type);
        assert_eq!(data_type_map.application_data_type().unwrap(), app_data_type);

        assert_eq!(mapping_set.data_type_maps().count(), 1);
    }
}
