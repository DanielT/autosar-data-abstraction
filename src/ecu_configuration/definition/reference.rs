use crate::{
    abstraction_element,
    ecu_configuration::{EcucCommonAttributes, EcucDefinitionElement},
    AbstractionElement, AutosarAbstractionError,
};
use autosar_data::{Element, ElementName};

use super::{AbstractEcucContainerDef, EcucContainerDef, EcucDestinationUriDef};

//#########################################################

/// marker trait for all reference definitions
pub trait AbstractEcucReferenceDef: AbstractionElement + EcucCommonAttributes + EcucDefinitionElement {}

//#########################################################

/// The `EcucForeignReferenceDef` specifies a reference to an XML description of an entity
/// described in another AUTOSAR template.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucForeignReferenceDef(Element);
abstraction_element!(EcucForeignReferenceDef, EcucForeignReferenceDef);
impl EcucCommonAttributes for EcucForeignReferenceDef {}
impl EcucDefinitionElement for EcucForeignReferenceDef {}
impl AbstractEcucReferenceDef for EcucForeignReferenceDef {}

impl EcucForeignReferenceDef {
    pub(crate) fn new(name: &str, references_elem: &Element, origin: &str) -> Result<Self, AutosarAbstractionError> {
        let ecuc_foreign_reference_def_elem =
            references_elem.create_named_sub_element(ElementName::EcucForeignReferenceDef, name)?;
        let ecuc_foreign_reference_def = Self(ecuc_foreign_reference_def_elem);
        ecuc_foreign_reference_def.set_origin(origin)?;

        Ok(ecuc_foreign_reference_def)
    }

    /// set the destination type of the reference definition
    pub fn set_destination_type(&self, destination_type: Option<&str>) -> Result<(), AutosarAbstractionError> {
        if let Some(destination_type) = destination_type {
            self.element()
                .get_or_create_sub_element(ElementName::DestinationType)?
                .set_character_data(destination_type)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::DestinationType);
        }

        Ok(())
    }

    /// get the destination type of the reference definition
    pub fn destination_type(&self) -> Option<String> {
        self.element()
            .get_sub_element(ElementName::DestinationType)?
            .character_data()?
            .string_value()
    }
}

//#########################################################

/// The `EcucInstanceReferenceDef` specifies a reference to an XML description of an entity
/// described in another AUTOSAR template using INSTANCE REFERENCE semantics.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucInstanceReferenceDef(Element);
abstraction_element!(EcucInstanceReferenceDef, EcucInstanceReferenceDef);
impl EcucCommonAttributes for EcucInstanceReferenceDef {}
impl EcucDefinitionElement for EcucInstanceReferenceDef {}
impl AbstractEcucReferenceDef for EcucInstanceReferenceDef {}

impl EcucInstanceReferenceDef {
    pub(crate) fn new(name: &str, references_elem: &Element, origin: &str) -> Result<Self, AutosarAbstractionError> {
        let ecuc_instance_reference_def_elem =
            references_elem.create_named_sub_element(ElementName::EcucInstanceReferenceDef, name)?;
        let ecuc_instance_reference_def = Self(ecuc_instance_reference_def_elem);
        ecuc_instance_reference_def.set_origin(origin)?;

        Ok(ecuc_instance_reference_def)
    }

    /// set the destination type of the reference definition
    pub fn set_destination_type(&self, destination_type: Option<&str>) -> Result<(), AutosarAbstractionError> {
        if let Some(destination_type) = destination_type {
            self.element()
                .get_or_create_sub_element(ElementName::DestinationType)?
                .set_character_data(destination_type)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::DestinationType);
        }

        Ok(())
    }

    /// get the destination type of the reference definition
    pub fn destination_type(&self) -> Option<String> {
        self.element()
            .get_sub_element(ElementName::DestinationType)?
            .character_data()?
            .string_value()
    }

    /// set the destination context of the reference definition
    ///
    /// The destination context is a string of autosar element names separated by spaces.
    /// Additionally, the '*' character can be used to indicate multiple occurrences of the previous element.
    /// E.g. "SW-COMPONENT-PROTOTYPE* R-PORT-PROTOTYPE"
    pub fn set_destination_context(&self, destination_context: Option<&str>) -> Result<(), AutosarAbstractionError> {
        if let Some(destination_context) = destination_context {
            self.element()
                .get_or_create_sub_element(ElementName::DestinationContext)?
                .set_character_data(destination_context)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::DestinationContext);
        }

        Ok(())
    }

    /// get the destination context of the reference definition
    ///
    /// The destination context is a string of autosar element names separated by spaces.
    pub fn destination_context(&self) -> Option<String> {
        self.element()
            .get_sub_element(ElementName::DestinationContext)?
            .character_data()?
            .string_value()
    }
}

//#########################################################

/// The `EcucChoiceReferenceDef` specifies alternative references where only one of the specified
/// references will be used in the ECU configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucChoiceReferenceDef(Element);
abstraction_element!(EcucChoiceReferenceDef, EcucChoiceReferenceDef);
impl EcucCommonAttributes for EcucChoiceReferenceDef {}
impl EcucDefinitionElement for EcucChoiceReferenceDef {}
impl AbstractEcucReferenceDef for EcucChoiceReferenceDef {}

impl EcucChoiceReferenceDef {
    pub(crate) fn new(name: &str, references_elem: &Element, origin: &str) -> Result<Self, AutosarAbstractionError> {
        let ecu_choice_reference_def_elem =
            references_elem.create_named_sub_element(ElementName::EcucChoiceReferenceDef, name)?;
        let ecu_choice_reference_def = Self(ecu_choice_reference_def_elem);
        ecu_choice_reference_def.set_origin(origin)?;

        Ok(ecu_choice_reference_def)
    }

    /// add a reference to a destination container
    pub fn add_destination<T: AbstractEcucContainerDef>(&self, destination: &T) -> Result<(), AutosarAbstractionError> {
        let dest_refs = self.element().get_or_create_sub_element(ElementName::DestinationRefs)?;
        dest_refs
            .create_sub_element(ElementName::DestinationRef)?
            .set_reference_target(destination.element())?;

        Ok(())
    }

    /// get the references to the destination containers
    pub fn destination_refs(&self) -> impl Iterator<Item = EcucContainerDef> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::DestinationRefs)
            .into_iter()
            .flat_map(|dest_refs| dest_refs.sub_elements())
            .filter_map(|dest_ref| {
                dest_ref
                    .get_reference_target()
                    .ok()
                    .and_then(|elem| elem.try_into().ok())
            })
    }
}

//#########################################################

/// The `EcuReferenceDef` specifies references between parameters in the ECU configuration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucReferenceDef(Element);
abstraction_element!(EcucReferenceDef, EcucReferenceDef);
impl EcucCommonAttributes for EcucReferenceDef {}
impl EcucDefinitionElement for EcucReferenceDef {}
impl AbstractEcucReferenceDef for EcucReferenceDef {}

impl EcucReferenceDef {
    pub(crate) fn new(name: &str, references_elem: &Element, origin: &str) -> Result<Self, AutosarAbstractionError> {
        let ecu_reference_def_elem = references_elem.create_named_sub_element(ElementName::EcucReferenceDef, name)?;
        let ecu_reference_def = Self(ecu_reference_def_elem);
        ecu_reference_def.set_origin(origin)?;

        Ok(ecu_reference_def)
    }

    /// set the destination container of the reference
    pub fn set_destination<T: AbstractEcucContainerDef>(
        &self,
        destination: Option<&T>,
    ) -> Result<(), AutosarAbstractionError> {
        if let Some(destination) = destination {
            self.element()
                .get_or_create_sub_element(ElementName::DestinationRef)?
                .set_reference_target(destination.element())?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::DestinationRef);
        }

        Ok(())
    }

    /// get the destination container of the reference
    pub fn destination(&self) -> Option<EcucContainerDef> {
        self.element()
            .get_sub_element(ElementName::DestinationRef)
            .and_then(|dest_ref| dest_ref.get_reference_target().ok())
            .and_then(|elem| elem.try_into().ok())
    }
}

//#########################################################

/// The `EcucUriReferenceDef` defines a reference with a destination that is specified via a destinationUri
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucUriReferenceDef(Element);
abstraction_element!(EcucUriReferenceDef, EcucUriReferenceDef);
impl EcucCommonAttributes for EcucUriReferenceDef {}
impl EcucDefinitionElement for EcucUriReferenceDef {}
impl AbstractEcucReferenceDef for EcucUriReferenceDef {}

impl EcucUriReferenceDef {
    pub(crate) fn new(name: &str, references_elem: &Element, origin: &str) -> Result<Self, AutosarAbstractionError> {
        let ecu_uri_reference_def_elem =
            references_elem.create_named_sub_element(ElementName::EcucUriReferenceDef, name)?;
        let ecu_uri_reference_def = Self(ecu_uri_reference_def_elem);
        ecu_uri_reference_def.set_origin(origin)?;

        Ok(ecu_uri_reference_def)
    }

    /// set the destination uri of the reference definition
    pub fn set_destination_uri(
        &self,
        destination_uri: Option<&EcucDestinationUriDef>,
    ) -> Result<(), AutosarAbstractionError> {
        if let Some(destination_uri) = destination_uri {
            self.element()
                .get_or_create_sub_element(ElementName::DestinationUriRef)?
                .set_reference_target(destination_uri.element())?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::DestinationUriRef);
        }

        Ok(())
    }

    /// get the destination uri of the reference definition
    pub fn destination_uri(&self) -> Option<EcucDestinationUriDef> {
        self.element()
            .get_sub_element(ElementName::DestinationUriRef)?
            .get_reference_target()
            .ok()?
            .try_into()
            .ok()
    }
}

//#########################################################

/// `EcucAnyReferenceDef` is an an enum of all possible reference definitions
/// It is used as a return type by the iterator over all references in a container
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EcucAnyReferenceDef {
    /// the reference is a foreign reference (external reference)
    Foreign(EcucForeignReferenceDef),
    /// the reference is an instance reference (external reference)
    Instance(EcucInstanceReferenceDef),
    /// the reference is a choice reference (internal reference)
    Choice(EcucChoiceReferenceDef),
    /// the reference is a normal reference (internal reference)
    Normal(EcucReferenceDef),
    /// the reference is a uri reference (internal reference)
    Uri(EcucUriReferenceDef),
}

impl AbstractionElement for EcucAnyReferenceDef {
    fn element(&self) -> &Element {
        match self {
            EcucAnyReferenceDef::Foreign(elem) => elem.element(),
            EcucAnyReferenceDef::Instance(elem) => elem.element(),
            EcucAnyReferenceDef::Choice(elem) => elem.element(),
            EcucAnyReferenceDef::Normal(elem) => elem.element(),
            EcucAnyReferenceDef::Uri(elem) => elem.element(),
        }
    }
}

impl TryFrom<Element> for EcucAnyReferenceDef {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::EcucForeignReferenceDef => Ok(EcucAnyReferenceDef::Foreign(element.try_into()?)),
            ElementName::EcucInstanceReferenceDef => Ok(EcucAnyReferenceDef::Instance(element.try_into()?)),
            ElementName::EcucChoiceReferenceDef => Ok(EcucAnyReferenceDef::Choice(element.try_into()?)),
            ElementName::EcucReferenceDef => Ok(EcucAnyReferenceDef::Normal(element.try_into()?)),
            ElementName::EcucUriReferenceDef => Ok(EcucAnyReferenceDef::Uri(element.try_into()?)),
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "EcucAnyReferenceDef".to_string(),
            }),
        }
    }
}

impl EcucDefinitionElement for EcucAnyReferenceDef {}
impl EcucCommonAttributes for EcucAnyReferenceDef {}
impl AbstractEcucReferenceDef for EcucAnyReferenceDef {}

//#########################################################

#[cfg(test)]
mod test {
    use crate::{
        ecu_configuration::{EcucContainerDef, EcucDestinationUriNestingContract},
        AbstractionElement, ArPackage,
    };
    use autosar_data::{AutosarModel, AutosarVersion};

    #[test]
    fn test_foreign_reference_def() {
        let model = AutosarModel::new();
        let _file = model.create_file("file.arxml", AutosarVersion::LATEST).unwrap();
        let pkg = ArPackage::get_or_create(&model, "/pkg").unwrap();

        let ecuc_module_def = pkg.create_ecuc_module_def("module").unwrap();
        let container = ecuc_module_def.create_param_conf_container_def("container").unwrap();
        let foreign_ref = container.create_foreign_reference_def("foreign_ref", "origin").unwrap();
        assert_eq!(container.references().count(), 1);

        assert_eq!(foreign_ref.destination_type(), None);
        foreign_ref.set_destination_type(Some("type")).unwrap();
        assert_eq!(foreign_ref.destination_type(), Some("type".to_string()));
        assert_eq!(container.references().next().unwrap().element(), foreign_ref.element());
    }

    #[test]
    fn test_instance_reference_def() {
        let model = AutosarModel::new();
        let _file = model.create_file("file.arxml", AutosarVersion::LATEST).unwrap();
        let pkg = ArPackage::get_or_create(&model, "/pkg").unwrap();

        let ecuc_module_def = pkg.create_ecuc_module_def("module").unwrap();
        let container = ecuc_module_def.create_param_conf_container_def("container").unwrap();
        let instance_ref = container
            .create_instance_reference_def("instance_ref", "origin")
            .unwrap();
        assert_eq!(container.references().count(), 1);

        assert_eq!(instance_ref.destination_type(), None);
        instance_ref.set_destination_type(Some("type")).unwrap();
        assert_eq!(instance_ref.destination_type(), Some("type".to_string()));

        assert_eq!(instance_ref.destination_context(), None);
        instance_ref.set_destination_context(Some("context")).unwrap();
        assert_eq!(instance_ref.destination_context(), Some("context".to_string()));
        assert_eq!(container.references().next().unwrap().element(), instance_ref.element());
    }

    #[test]
    fn test_choice_reference_def() {
        let model = AutosarModel::new();
        let _file = model.create_file("file.arxml", AutosarVersion::LATEST).unwrap();
        let pkg = ArPackage::get_or_create(&model, "/pkg").unwrap();

        let ecuc_module_def = pkg.create_ecuc_module_def("module").unwrap();
        let container = ecuc_module_def.create_param_conf_container_def("container").unwrap();
        let choice_ref = container.create_choice_reference_def("choice_ref", "origin").unwrap();
        assert_eq!(container.references().count(), 1);

        assert_eq!(choice_ref.destination_refs().count(), 0);
        let dest = container.create_param_conf_container_def("dest").unwrap();
        choice_ref.add_destination(&dest).unwrap();
        assert_eq!(choice_ref.destination_refs().count(), 1);
        assert_eq!(container.references().next().unwrap().element(), choice_ref.element());
    }

    #[test]
    fn test_reference_def() {
        let model = AutosarModel::new();
        let _file = model.create_file("file.arxml", AutosarVersion::LATEST).unwrap();
        let pkg = ArPackage::get_or_create(&model, "/pkg").unwrap();

        let ecuc_module_def = pkg.create_ecuc_module_def("module").unwrap();
        let container = ecuc_module_def.create_param_conf_container_def("container").unwrap();
        let reference = container.create_reference_def("reference", "origin").unwrap();
        assert_eq!(container.references().count(), 1);

        assert_eq!(reference.destination(), None);
        let dest = container.create_param_conf_container_def("dest").unwrap();
        reference.set_destination(Some(&dest)).unwrap();
        assert_eq!(reference.destination().unwrap(), EcucContainerDef::ParamConf(dest));
        assert_eq!(container.references().next().unwrap().element(), reference.element());
    }

    #[test]
    fn test_uri_reference_def() {
        let model = AutosarModel::new();
        let _file = model.create_file("file.arxml", AutosarVersion::LATEST).unwrap();
        let pkg = ArPackage::get_or_create(&model, "/pkg").unwrap();

        let ecuc_module_def = pkg.create_ecuc_module_def("module").unwrap();
        let container = ecuc_module_def.create_param_conf_container_def("container").unwrap();
        let uri_ref = container.create_uri_reference_def("uri_ref", "origin").unwrap();
        assert_eq!(container.references().count(), 1);

        let uri_def_set = pkg.create_ecuc_destination_uri_def_set("uri_def").unwrap();
        let uri_def = uri_def_set
            .create_destination_uri_def("uri", EcucDestinationUriNestingContract::VertexOfTargetContainer)
            .unwrap();

        assert_eq!(uri_ref.destination_uri(), None);
        uri_ref.set_destination_uri(Some(&uri_def)).unwrap();
        assert_eq!(uri_ref.destination_uri(), Some(uri_def));
        assert_eq!(container.references().next().unwrap().element(), uri_ref.element());
    }
}
