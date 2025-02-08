use crate::{
    abstraction_element,
    ecu_configuration::{
        EcucAddInfoParamDef, EcucAnyReferenceDef, EcucBooleanParamDef, EcucChoiceReferenceDef, EcucDefinitionElement,
        EcucEnumerationParamDef, EcucFloatParamDef, EcucForeignReferenceDef, EcucFunctionNameDef,
        EcucInstanceReferenceDef, EcucIntegerParamDef, EcucLinkerSymbolDef, EcucMultilineStringParamDef,
        EcucParameterDef, EcucReferenceDef, EcucStringParamDef, EcucUriReferenceDef,
    },
    AbstractionElement, AutosarAbstractionError,
};
use autosar_data::{Element, ElementName};

//#########################################################

/// Marker trait for container definitions
pub trait AbstractEcucContainerDef: EcucDefinitionElement {}

//#########################################################

/// The `EcucChoiceContainerDef` is used to define configuration containers
/// that provide a choice between several EcucParamConfContainerDef
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucChoiceContainerDef(Element);
abstraction_element!(EcucChoiceContainerDef, EcucChoiceContainerDef);
impl EcucDefinitionElement for EcucChoiceContainerDef {}
impl AbstractEcucContainerDef for EcucChoiceContainerDef {}

impl EcucChoiceContainerDef {
    pub(crate) fn new(name: &str, containers_elem: &Element) -> Result<Self, AutosarAbstractionError> {
        let choice_container_def_elem =
            containers_elem.create_named_sub_element(ElementName::EcucChoiceContainerDef, name)?;

        Ok(Self(choice_container_def_elem))
    }

    /// create a new `EcucParamConfContainerDef` as one of the choices in this choice container
    pub fn create_param_conf_container_def(
        &self,
        name: &str,
    ) -> Result<EcucParamConfContainerDef, AutosarAbstractionError> {
        let containers_elem = self.element().get_or_create_sub_element(ElementName::Choices)?;
        EcucParamConfContainerDef::new(name, &containers_elem)
    }

    /// iterate over the choices in the container
    pub fn choices(&self) -> impl Iterator<Item = EcucParamConfContainerDef> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::Choices)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|elem| EcucParamConfContainerDef::try_from(elem).ok())
    }
}

//#########################################################

/// The `EcucParamConfContainerDef` is used to define configuration containers
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucParamConfContainerDef(Element);
abstraction_element!(EcucParamConfContainerDef, EcucParamConfContainerDef);
impl EcucDefinitionElement for EcucParamConfContainerDef {}
impl AbstractEcucContainerDef for EcucParamConfContainerDef {}

impl EcucParamConfContainerDef {
    pub(crate) fn new(name: &str, containers_elem: &Element) -> Result<Self, AutosarAbstractionError> {
        let param_conf_container_def_elem =
            containers_elem.create_named_sub_element(ElementName::EcucParamConfContainerDef, name)?;

        Ok(Self(param_conf_container_def_elem))
    }

    /// create a new `EcucChoiceContainerDef` as a sub-container
    pub fn create_choice_container_def(&self, name: &str) -> Result<EcucChoiceContainerDef, AutosarAbstractionError> {
        let containers_elem = self.element().get_or_create_sub_element(ElementName::SubContainers)?;
        EcucChoiceContainerDef::new(name, &containers_elem)
    }

    /// create a new `EcucParamConfContainerDef` as a sub-container
    pub fn create_param_conf_container_def(
        &self,
        name: &str,
    ) -> Result<EcucParamConfContainerDef, AutosarAbstractionError> {
        let containers_elem = self.element().get_or_create_sub_element(ElementName::SubContainers)?;
        EcucParamConfContainerDef::new(name, &containers_elem)
    }

    /// iterate over the sub-containers
    pub fn sub_containers(&self) -> impl Iterator<Item = EcucContainerDef> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::SubContainers)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|elem| EcucContainerDef::try_from(elem).ok())
    }

    /// create a new EcucAddInfoParamDef in the container
    pub fn create_add_info_param_def(
        &self,
        name: &str,
        origin: &str,
    ) -> Result<EcucAddInfoParamDef, AutosarAbstractionError> {
        let parameters_elem = self.element().get_or_create_sub_element(ElementName::Parameters)?;
        EcucAddInfoParamDef::new(name, &parameters_elem, origin)
    }

    /// create a new EcucBooleanParamDef in the container
    pub fn create_boolean_param_def(
        &self,
        name: &str,
        origin: &str,
    ) -> Result<EcucBooleanParamDef, AutosarAbstractionError> {
        let parameters_elem = self.element().get_or_create_sub_element(ElementName::Parameters)?;
        EcucBooleanParamDef::new(name, &parameters_elem, origin)
    }

    /// create a new EcucEnumerationParamDef in the container
    pub fn create_enumeration_param_def(
        &self,
        name: &str,
        origin: &str,
    ) -> Result<EcucEnumerationParamDef, AutosarAbstractionError> {
        let parameters_elem = self.element().get_or_create_sub_element(ElementName::Parameters)?;
        EcucEnumerationParamDef::new(name, &parameters_elem, origin)
    }

    /// create a new EcucFloatParamDef in the container
    pub fn create_float_param_def(
        &self,
        name: &str,
        origin: &str,
    ) -> Result<EcucFloatParamDef, AutosarAbstractionError> {
        let parameters_elem = self.element().get_or_create_sub_element(ElementName::Parameters)?;
        EcucFloatParamDef::new(name, &parameters_elem, origin)
    }

    /// create a new EcucIntegerParamDef in the container
    pub fn create_integer_param_def(
        &self,
        name: &str,
        origin: &str,
    ) -> Result<EcucIntegerParamDef, AutosarAbstractionError> {
        let parameters_elem = self.element().get_or_create_sub_element(ElementName::Parameters)?;
        EcucIntegerParamDef::new(name, &parameters_elem, origin)
    }

    /// create a new EcucFunctionNameDef in the container
    pub fn create_function_name_def(
        &self,
        name: &str,
        origin: &str,
    ) -> Result<EcucFunctionNameDef, AutosarAbstractionError> {
        let parameters_elem = self.element().get_or_create_sub_element(ElementName::Parameters)?;
        EcucFunctionNameDef::new(name, &parameters_elem, origin)
    }

    /// create a new EcucLinkerSymbolDef in the container
    pub fn create_linker_symbol_def(
        &self,
        name: &str,
        origin: &str,
    ) -> Result<EcucLinkerSymbolDef, AutosarAbstractionError> {
        let parameters_elem = self.element().get_or_create_sub_element(ElementName::Parameters)?;
        EcucLinkerSymbolDef::new(name, &parameters_elem, origin)
    }

    /// create a new EcucMultilineStringParamDef in the container
    pub fn create_multiline_string_param_def(
        &self,
        name: &str,
        origin: &str,
    ) -> Result<EcucMultilineStringParamDef, AutosarAbstractionError> {
        let parameters_elem = self.element().get_or_create_sub_element(ElementName::Parameters)?;
        EcucMultilineStringParamDef::new(name, &parameters_elem, origin)
    }

    /// create a new EcucStringParamDef in the container
    pub fn create_string_param_def(
        &self,
        name: &str,
        origin: &str,
    ) -> Result<EcucStringParamDef, AutosarAbstractionError> {
        let parameters_elem = self.element().get_or_create_sub_element(ElementName::Parameters)?;
        EcucStringParamDef::new(name, &parameters_elem, origin)
    }

    /// get the parameters in the container
    pub fn parameters(&self) -> impl Iterator<Item = EcucParameterDef> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::Parameters)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|elem| EcucParameterDef::try_from(elem).ok())
    }

    /// create a new EcucForeignReferenceDef in the container
    pub fn create_foreign_reference_def(
        &self,
        name: &str,
        origin: &str,
    ) -> Result<EcucForeignReferenceDef, AutosarAbstractionError> {
        let foreign_references_elem = self.element().get_or_create_sub_element(ElementName::References)?;
        EcucForeignReferenceDef::new(name, &foreign_references_elem, origin)
    }

    /// create a new EcucInstanceReferenceDef in the container
    pub fn create_instance_reference_def(
        &self,
        name: &str,
        origin: &str,
    ) -> Result<EcucInstanceReferenceDef, AutosarAbstractionError> {
        let instance_references_elem = self.element().get_or_create_sub_element(ElementName::References)?;
        EcucInstanceReferenceDef::new(name, &instance_references_elem, origin)
    }

    /// create a new EcucChoiceReferenceDef in the container
    pub fn create_choice_reference_def(
        &self,
        name: &str,
        origin: &str,
    ) -> Result<EcucChoiceReferenceDef, AutosarAbstractionError> {
        let choice_references_elem = self.element().get_or_create_sub_element(ElementName::References)?;
        EcucChoiceReferenceDef::new(name, &choice_references_elem, origin)
    }

    /// create a new EcucReferenceDef in the container
    pub fn create_reference_def(&self, name: &str, origin: &str) -> Result<EcucReferenceDef, AutosarAbstractionError> {
        let references_elem = self.element().get_or_create_sub_element(ElementName::References)?;
        EcucReferenceDef::new(name, &references_elem, origin)
    }

    /// create a new EcucUriReferenceDef in the container
    pub fn create_uri_reference_def(
        &self,
        name: &str,
        origin: &str,
    ) -> Result<EcucUriReferenceDef, AutosarAbstractionError> {
        let uri_references_elem = self.element().get_or_create_sub_element(ElementName::References)?;
        EcucUriReferenceDef::new(name, &uri_references_elem, origin)
    }

    /// get the references in the container
    pub fn references(&self) -> impl Iterator<Item = EcucAnyReferenceDef> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::References)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|elem| EcucAnyReferenceDef::try_from(elem).ok())
    }
}

//#########################################################

/// `EcucContainerDef` is an enum of both container definitions, which is used as a return type by iterators
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EcucContainerDef {
    /// the container is a choice container
    Choice(EcucChoiceContainerDef),
    /// the container is a parameter container
    ParamConf(EcucParamConfContainerDef),
}

impl AbstractionElement for EcucContainerDef {
    fn element(&self) -> &Element {
        match self {
            EcucContainerDef::Choice(elem) => elem.element(),
            EcucContainerDef::ParamConf(elem) => elem.element(),
        }
    }
}

impl TryFrom<Element> for EcucContainerDef {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::EcucChoiceContainerDef => Ok(EcucContainerDef::Choice(element.try_into()?)),
            ElementName::EcucParamConfContainerDef => Ok(EcucContainerDef::ParamConf(element.try_into()?)),
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "EcucContainerDef".to_string(),
            }),
        }
    }
}

impl EcucDefinitionElement for EcucContainerDef {}
impl AbstractEcucContainerDef for EcucContainerDef {}

//#########################################################

#[cfg(test)]
mod test {
    use crate::{AbstractionElement, AutosarModelAbstraction};
    use autosar_data::AutosarVersion;

    #[test]
    fn container() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST).unwrap();
        let package = model.get_or_create_package("/pkg1").unwrap();
        let ecuc_module = package.create_ecuc_module_def("ecuc_module").unwrap();
        assert_eq!(ecuc_module.containers().count(), 0);

        let choice_container = ecuc_module.create_choice_container_def("ChoiceContainer").unwrap();
        assert_eq!(choice_container.choices().count(), 0);
        choice_container
            .create_param_conf_container_def("ParamConfContainerChoice1")
            .unwrap();
        choice_container
            .create_param_conf_container_def("ParamConfContainerChoice2")
            .unwrap();
        assert_eq!(choice_container.choices().count(), 2);

        let param_conf_container = ecuc_module
            .create_param_conf_container_def("ParamConfContainer")
            .unwrap();
        assert_eq!(param_conf_container.sub_containers().count(), 0);
        param_conf_container
            .create_choice_container_def("ChoiceContainer")
            .unwrap();
        assert_eq!(param_conf_container.sub_containers().count(), 1);

        assert_eq!(param_conf_container.parameters().count(), 0);
        param_conf_container
            .create_add_info_param_def("AddInfoParam", "origin")
            .unwrap();
        param_conf_container
            .create_boolean_param_def("BooleanParam", "origin")
            .unwrap();
        param_conf_container
            .create_enumeration_param_def("EnumerationParam", "origin")
            .unwrap();
        param_conf_container
            .create_float_param_def("FloatParam", "origin")
            .unwrap();
        param_conf_container
            .create_integer_param_def("IntegerParam", "origin")
            .unwrap();
        param_conf_container
            .create_function_name_def("FunctionName", "origin")
            .unwrap();
        param_conf_container
            .create_linker_symbol_def("LinkerSymbol", "origin")
            .unwrap();
        param_conf_container
            .create_multiline_string_param_def("MultilineStringParam", "origin")
            .unwrap();
        param_conf_container
            .create_string_param_def("StringParam", "origin")
            .unwrap();
        assert_eq!(param_conf_container.parameters().count(), 9);

        assert_eq!(param_conf_container.references().count(), 0);
        param_conf_container
            .create_foreign_reference_def("ForeignReference", "origin")
            .unwrap();
        param_conf_container
            .create_instance_reference_def("InstanceReference", "origin")
            .unwrap();
        param_conf_container
            .create_choice_reference_def("ChoiceReference", "origin")
            .unwrap();
        param_conf_container
            .create_reference_def("Reference", "origin")
            .unwrap();
        param_conf_container
            .create_uri_reference_def("UriReference", "origin")
            .unwrap();
        assert_eq!(param_conf_container.references().count(), 5);

        assert_eq!(ecuc_module.containers().count(), 2);
        let mut containers_iter = ecuc_module.containers();
        assert_eq!(containers_iter.next().unwrap().element(), choice_container.element());
        assert_eq!(
            containers_iter.next().unwrap().element(),
            param_conf_container.element()
        );
    }
}
