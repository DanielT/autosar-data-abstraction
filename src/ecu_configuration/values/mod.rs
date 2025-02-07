use crate::{
    abstraction_element,
    ecu_configuration::{
        AbstractEcucContainerDef, AbstractEcucReferenceDef, EcucContainerDef, EcucInstanceReferenceDef, EcucModuleDef,
        EcucNumericalParamDef, EcucTextualParamDef,
    },
    AbstractionElement, ArPackage, AutosarAbstractionError, System,
};
use autosar_data::{Element, ElementName};

mod parameter;
mod reference;

pub use parameter::*;
pub use reference::*;

//#########################################################

/// `EcucValueCollection` collects references to all the separate modules that form the ECU configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucValueCollection(Element);
abstraction_element!(EcucValueCollection, EcucValueCollection);

impl EcucValueCollection {
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let ecuc_value_collection_elem = elements.create_named_sub_element(ElementName::EcucValueCollection, name)?;

        Ok(Self(ecuc_value_collection_elem))
    }

    /// Add a reference to a module configuration to the collection
    pub fn add_module_configuration(
        &self,
        module_configuration: &EcucModuleConfigurationValues,
    ) -> Result<(), AutosarAbstractionError> {
        let ecuc_values_elem = self.element().get_or_create_sub_element(ElementName::EcucValues)?;
        ecuc_values_elem
            .create_sub_element(ElementName::EcucModuleConfigurationValuesRefConditional)?
            .create_sub_element(ElementName::EcucModuleConfigurationValuesRef)?
            .set_reference_target(module_configuration.element())?;

        Ok(())
    }

    /// Get the module configurations in the collection
    pub fn module_configurations(&self) -> impl Iterator<Item = EcucModuleConfigurationValues> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::EcucValues)
            .into_iter()
            .flat_map(|values_elem| values_elem.sub_elements())
            .filter_map(|ref_cond| {
                ref_cond
                    .get_sub_element(ElementName::EcucModuleConfigurationValuesRef)
                    .and_then(|module_config_ref| module_config_ref.get_reference_target().ok())
                    .and_then(|module_elem| EcucModuleConfigurationValues::try_from(module_elem).ok())
            })
    }

    /// Set the ecu extract reference, which links a `System` to the ECU configuration
    pub fn set_ecu_extract_reference(&self, system: &System) -> Result<(), AutosarAbstractionError> {
        self.element()
            .create_sub_element(ElementName::EcuExtractRef)?
            .set_reference_target(system.element())?;

        Ok(())
    }

    /// Get the system that the ECU configuration is linked to
    pub fn ecu_extract_reference(&self) -> Option<System> {
        let system_elem = self
            .element()
            .get_sub_element(ElementName::EcuExtractRef)?
            .get_reference_target()
            .ok()?;
        System::try_from(system_elem).ok()
    }
}

//#########################################################

/// The `EcucModuleConfigurationValues` is a container for the configuration of a single base software module
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucModuleConfigurationValues(Element);
abstraction_element!(EcucModuleConfigurationValues, EcucModuleConfigurationValues);

impl EcucModuleConfigurationValues {
    pub(crate) fn new(
        name: &str,
        package: &ArPackage,
        module_definition: &EcucModuleDef,
    ) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let module_config_elem = elements.create_named_sub_element(ElementName::EcucModuleConfigurationValues, name)?;

        let module_config = Self(module_config_elem);
        module_config.set_definition(module_definition)?;

        Ok(module_config)
    }

    /// set the module definition reference
    pub fn set_definition(&self, module_definition: &EcucModuleDef) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::DefinitionRef)?
            .set_reference_target(module_definition.element())?;

        Ok(())
    }

    /// get the module definition
    ///
    /// This function returns the definition as an `EcucModuleDef` object.
    /// If the definition is not loaded, use `definition_ref()` instead.
    pub fn definition(&self) -> Option<EcucModuleDef> {
        let definition_elem = self
            .element()
            .get_sub_element(ElementName::DefinitionRef)?
            .get_reference_target()
            .ok()?;
        EcucModuleDef::try_from(definition_elem).ok()
    }

    /// get the definition reference as a string
    ///
    /// This function is an alternative to `definition()`; it is useful when the
    /// referenced definition is not loaded and can't be resolved.
    pub fn definition_ref(&self) -> Option<String> {
        self.element()
            .get_sub_element(ElementName::DefinitionRef)?
            .character_data()?
            .string_value()
    }

    /// Create a new `EcucContainerValue` in the module configuration
    pub fn create_container_value<T: AbstractEcucContainerDef>(
        &self,
        name: &str,
        definition: &T,
    ) -> Result<EcucContainerValue, AutosarAbstractionError> {
        let containers_elem = self.element().get_or_create_sub_element(ElementName::Containers)?;
        EcucContainerValue::new(name, &containers_elem, definition)
    }

    /// create an iterator over the container values in the module configuration
    pub fn container_values(&self) -> impl Iterator<Item = EcucContainerValue> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::Containers)
            .into_iter()
            .flat_map(|containers_elem| containers_elem.sub_elements())
            .filter_map(|container_elem| EcucContainerValue::try_from(container_elem).ok())
    }
}

//#########################################################

/// The `EcucContainerValue` is a container in the ECU configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucContainerValue(Element);
abstraction_element!(EcucContainerValue, EcucContainerValue);

impl EcucContainerValue {
    pub(crate) fn new<T: AbstractEcucContainerDef>(
        name: &str,
        parent: &Element,
        definition: &T,
    ) -> Result<Self, AutosarAbstractionError> {
        let container_value_elem = parent.create_named_sub_element(ElementName::EcucContainerValue, name)?;
        let container_value = Self(container_value_elem);

        container_value.set_definition(definition)?;

        Ok(container_value)
    }

    /// set the container definition reference
    pub fn set_definition<T: AbstractEcucContainerDef>(&self, definition: &T) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::DefinitionRef)?
            .set_reference_target(definition.element())?;

        Ok(())
    }

    /// get the container definition
    ///
    /// This function returns the definition as an `EcucContainerDef` object.
    /// If the definition is not loaded, use `definition_ref()` instead.
    pub fn definition(&self) -> Option<EcucContainerDef> {
        let definition_elem = self
            .element()
            .get_sub_element(ElementName::DefinitionRef)?
            .get_reference_target()
            .ok()?;
        EcucContainerDef::try_from(definition_elem).ok()
    }

    /// get the definition reference as a string
    ///
    /// This function is an alternative to `definition()`; it is useful when the
    /// referenced definition is not loaded and can't be resolved.
    pub fn definition_ref(&self) -> Option<String> {
        self.element()
            .get_sub_element(ElementName::DefinitionRef)?
            .character_data()?
            .string_value()
    }

    /// create a sub-container
    pub fn create_sub_container<T: AbstractEcucContainerDef>(
        &self,
        name: &str,
        definition: &T,
    ) -> Result<EcucContainerValue, AutosarAbstractionError> {
        let sub_containers_elem = self.element().get_or_create_sub_element(ElementName::SubContainers)?;
        EcucContainerValue::new(name, &sub_containers_elem, definition)
    }

    /// iterate over the sub-containers in this container
    pub fn sub_containers(&self) -> impl Iterator<Item = EcucContainerValue> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::SubContainers)
            .into_iter()
            .flat_map(|sub_containers| sub_containers.sub_elements())
            .filter_map(|elem| EcucContainerValue::try_from(elem).ok())
    }

    /// set the index of the container
    ///
    /// If the container definition has `requiresIndex` set to `true`, then the container
    /// must have an index. Otherwise the index is meaningless.
    pub fn set_index(&self, index: Option<u64>) -> Result<(), AutosarAbstractionError> {
        if let Some(index) = index {
            self.element()
                .get_or_create_sub_element(ElementName::Index)?
                .set_character_data(index)?;
        } else {
            self.element().remove_sub_element_kind(ElementName::Index)?;
        }

        Ok(())
    }

    /// get the index of the container
    ///
    /// If the container definition has `requiresIndex` set to `true`, then the container
    /// must have an index. Otherwise the index is meaningless.
    pub fn index(&self) -> Option<u64> {
        self.element()
            .get_sub_element(ElementName::Index)?
            .character_data()?
            .parse_integer()
    }

    /// create a new `EcucNumericalParamValue` in the container
    pub fn create_numerical_param_value<T: EcucNumericalParamDef>(
        &self,
        definition: &T,
        value: &str,
    ) -> Result<EcucNumericalParamValue, AutosarAbstractionError> {
        let parameter_values_elem = self.element().get_or_create_sub_element(ElementName::ParameterValues)?;
        EcucNumericalParamValue::new(&parameter_values_elem, definition, value)
    }

    /// create a new `EcucTextualParamValue` in the container
    pub fn create_textual_param_value<T: EcucTextualParamDef>(
        &self,
        definition: &T,
        value: &str,
    ) -> Result<EcucTextualParamValue, AutosarAbstractionError> {
        let parameter_values_elem = self.element().get_or_create_sub_element(ElementName::ParameterValues)?;
        EcucTextualParamValue::new(&parameter_values_elem, definition, value)
    }

    /// iterate over the parameter values in the container
    pub fn parameter_values(&self) -> impl Iterator<Item = EcucParameterValue> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::ParameterValues)
            .into_iter()
            .flat_map(|param_values_elem| param_values_elem.sub_elements())
            .filter_map(|param_elem| EcucParameterValue::try_from(param_elem).ok())
    }

    /// create a new instance reference value in the container
    pub fn create_instance_reference(
        &self,
        definition: &EcucInstanceReferenceDef,
        target_context: &[&Element],
        target: &Element,
    ) -> Result<EcucInstanceReferenceValue, AutosarAbstractionError> {
        let reference_values_elem = self.element().get_or_create_sub_element(ElementName::ReferenceValues)?;
        EcucInstanceReferenceValue::new(&reference_values_elem, definition, target_context, target)
    }

    /// create a new reference value in the container
    pub fn create_reference_value<T: AbstractEcucReferenceDef>(
        &self,
        definition: &T,
        target: &Element,
    ) -> Result<EcucReferenceValue, AutosarAbstractionError> {
        let reference_values_elem = self.element().get_or_create_sub_element(ElementName::ReferenceValues)?;
        EcucReferenceValue::new(&reference_values_elem, definition, target)
    }

    /// iterate over the reference values in the container
    pub fn reference_values(&self) -> impl Iterator<Item = EcucAnyReferenceValue> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::ReferenceValues)
            .into_iter()
            .flat_map(|reference_values_elem| reference_values_elem.sub_elements())
            .filter_map(|reference_elem| EcucAnyReferenceValue::try_from(reference_elem).ok())
    }
}

//#########################################################

#[cfg(test)]
mod test {
    use crate::{system, AbstractionElement, ArPackage};
    use autosar_data::{AutosarModel, AutosarVersion, ElementName};

    #[test]
    fn test_ecu_configuration_values() {
        let definition_model = AutosarModel::new();
        let _file = definition_model
            .create_file("definition.arxml", AutosarVersion::LATEST)
            .unwrap();
        let def_package = ArPackage::get_or_create(&definition_model, "/def_package").unwrap();

        let values_model = AutosarModel::new();
        let _file = values_model
            .create_file("values.arxml", AutosarVersion::LATEST)
            .unwrap();
        let val_package = ArPackage::get_or_create(&values_model, "/val_package").unwrap();

        // create a definition for the ECU configuration
        let module_def = def_package.create_ecuc_module_def("ModuleDef").unwrap();
        let container_def = module_def.create_param_conf_container_def("ContainerDef").unwrap();

        // create an ecu configuration based on the definition model
        let ecuc_value_collection = val_package.create_ecuc_value_collection("EcucValues").unwrap();
        assert_eq!(ecuc_value_collection.ecu_extract_reference(), None);
        let system = val_package
            .create_system("System", system::SystemCategory::EcuExtract)
            .unwrap();
        ecuc_value_collection.set_ecu_extract_reference(&system).unwrap();
        assert_eq!(ecuc_value_collection.ecu_extract_reference().unwrap(), system);
        assert_eq!(ecuc_value_collection.module_configurations().count(), 0);

        let ecuc_config_values = val_package
            .create_ecuc_module_configuration_values("Module", &module_def)
            .unwrap();
        ecuc_value_collection
            .add_module_configuration(&ecuc_config_values)
            .unwrap();
        assert_eq!(ecuc_value_collection.module_configurations().count(), 1);
        assert_eq!(ecuc_config_values.definition_ref(), module_def.element().path().ok());
        // the definition is not loaded in the same model, so we can't get it
        assert!(ecuc_config_values.definition().is_none());

        let container_values = ecuc_config_values
            .create_container_value("Container", &container_def)
            .unwrap();
        assert_eq!(container_values.definition_ref(), container_def.element().path().ok());
        assert!(container_values.definition().is_none());
        assert_eq!(container_values.index(), None);
        container_values.set_index(Some(0)).unwrap();
        assert_eq!(container_values.index(), Some(0));
        assert_eq!(ecuc_config_values.container_values().count(), 1);

        let sub_container_value = container_values
            .create_sub_container("SubContainer", &container_def)
            .unwrap();
        assert_eq!(
            sub_container_value.definition_ref(),
            container_def.element().path().ok()
        );
        assert_eq!(container_values.sub_containers().count(), 1);

        // copy the definition into the value model
        // once the definition and values are in the same model, we can get the definition directly
        values_model
            .root_element()
            .get_sub_element(ElementName::ArPackages)
            .unwrap()
            .create_copied_sub_element(def_package.element())
            .unwrap();
        // get the definitions from the value model
    }
}
