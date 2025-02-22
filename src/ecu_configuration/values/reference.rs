use crate::ecu_configuration::{AbstractEcucReferenceDef, EcucAnyReferenceDef, EcucInstanceReferenceDef};
use crate::{AbstractionElement, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element};
use autosar_data::{Element, ElementName};

//#########################################################

/// An `EcucInstanceReferenceValue` provides the mechanism to reference an instance of a prototype
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucInstanceReferenceValue(Element);
abstraction_element!(EcucInstanceReferenceValue, EcucInstanceReferenceValue);

impl EcucInstanceReferenceValue {
    pub(crate) fn new(
        parent: &Element,
        definition: &EcucInstanceReferenceDef,
        target_context: &[&Element],
        target: &Element,
    ) -> Result<Self, AutosarAbstractionError> {
        let instance_ref_elem = parent.create_sub_element(ElementName::EcucInstanceReferenceValue)?;
        let instance_ref = Self(instance_ref_elem);

        instance_ref.set_definition(definition)?;
        instance_ref.set_target(target_context, target)?;

        Ok(instance_ref)
    }

    /// set the parameter definition reference
    pub fn set_definition(&self, definition: &EcucInstanceReferenceDef) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::DefinitionRef)?
            .set_reference_target(definition.element())?;

        Ok(())
    }

    /// get the parameter definition
    ///
    /// This function returns the definition as an `EcucParameterDef` enum, which
    /// could contain either an `EcucFloatParamDef` or an `EcucIntegerParamDef`.
    /// If the definition is not loaded, use `definition_ref()` instead.
    pub fn definition(&self) -> Option<EcucInstanceReferenceDef> {
        let definition_elem = self
            .element()
            .get_sub_element(ElementName::DefinitionRef)?
            .get_reference_target()
            .ok()?;
        EcucInstanceReferenceDef::try_from(definition_elem).ok()
    }

    /// get the parameter definition reference as a string
    ///
    /// This function is an alternative to `definition()`; it is useful when the
    /// referenced definition is not loaded and can't be resolved.
    pub fn definition_ref(&self) -> Option<String> {
        self.element()
            .get_sub_element(ElementName::DefinitionRef)?
            .character_data()?
            .string_value()
    }

    /// Set the target of the reference
    ///
    /// An instance reference targets a specific instance of a prototype. In order to uniquely identify the target,
    /// the target context is required. The target context is a list of elements that are the parent elements of the
    /// target element. The instance reference definition specifies which context elements are required.
    pub fn set_target(&self, taget_context: &[&Element], target: &Element) -> Result<(), AutosarAbstractionError> {
        // remove existing target elements
        let _ = self.element().remove_sub_element_kind(ElementName::ValueIref);
        // create the target context elements

        let value_iref_elem = self.element().create_sub_element(ElementName::ValueIref)?;
        for context_elem in taget_context {
            value_iref_elem
                .create_sub_element(ElementName::ContextElementRef)?
                .set_reference_target(context_elem)?;
        }
        // create the target element
        value_iref_elem
            .create_sub_element(ElementName::TargetRef)?
            .set_reference_target(target)?;

        Ok(())
    }

    /// Get the target of the reference
    ///
    /// Returns the targt element of the instance reference, as well as the context elements that are needed to uniquely
    /// identify the target.
    pub fn target(&self) -> Option<(Vec<Element>, Element)> {
        let value_iref_elem = self.element().get_sub_element(ElementName::ValueIref)?;
        let target = value_iref_elem
            .get_sub_element(ElementName::TargetRef)?
            .get_reference_target()
            .ok()?;

        let context_elements: Vec<_> = value_iref_elem
            .sub_elements()
            .filter(|elem| elem.element_name() == ElementName::ContextElementRef)
            .filter_map(|context_ref| context_ref.get_reference_target().ok())
            .collect();

        Some((context_elements, target))
    }

    /// set the index of the reference
    ///
    /// If the reference definition has `requiresIndex` set to `true`, then the reference
    /// must have an index. Otherwise the index is meaningless.
    pub fn set_index(&self, index: Option<u64>) -> Result<(), AutosarAbstractionError> {
        if let Some(index) = index {
            self.element()
                .get_or_create_sub_element(ElementName::Index)?
                .set_character_data(index)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::Index);
        }

        Ok(())
    }

    /// get the index of the reference
    ///
    /// If the reference definition has `requiresIndex` set to `true`, then the reference
    /// must have an index. Otherwise the index is meaningless.
    pub fn index(&self) -> Option<u64> {
        self.element()
            .get_sub_element(ElementName::Index)?
            .character_data()?
            .parse_integer()
    }

    /// set the isAutoValue flag
    ///
    /// If the reference definition has `withAuto` set to `true`, then the reference is allowed to have an auto value.
    pub fn set_is_auto_value(&self, is_auto_value: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(is_auto_value) = is_auto_value {
            self.element()
                .get_or_create_sub_element(ElementName::IsAutoValue)?
                .set_character_data(is_auto_value)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::IsAutoValue);
        }

        Ok(())
    }

    /// get the isAutoValue flag
    pub fn is_auto_value(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::IsAutoValue)?
            .character_data()?
            .parse_bool()
    }
}

//#########################################################

/// An `EcucReferenceValue` allows the ecu tonfiguration to refer to any identifiable element in the Autosar model
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucReferenceValue(Element);
abstraction_element!(EcucReferenceValue, EcucReferenceValue);

impl EcucReferenceValue {
    pub(crate) fn new<T: AbstractEcucReferenceDef>(
        parent: &Element,
        definition: &T,
        target: &Element,
    ) -> Result<Self, AutosarAbstractionError> {
        let reference_elem = parent.create_sub_element(ElementName::EcucReferenceValue)?;
        let reference = Self(reference_elem);

        reference.set_definition(definition)?;
        reference.set_target(target)?;

        Ok(reference)
    }

    /// set the parameter definition reference
    pub fn set_definition<T: AbstractEcucReferenceDef>(&self, definition: &T) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::DefinitionRef)?
            .set_reference_target(definition.element())?;

        Ok(())
    }

    /// get the reference definition
    ///
    /// This function returns the definition as an `EcucParameterDef` enum, which
    /// could contain either an `EcucFloatParamDef` or an `EcucIntegerParamDef`.
    /// If the definition is not loaded, use `definition_ref()` instead.
    pub fn definition(&self) -> Option<EcucAnyReferenceDef> {
        let definition_elem = self
            .element()
            .get_sub_element(ElementName::DefinitionRef)?
            .get_reference_target()
            .ok()?;
        EcucAnyReferenceDef::try_from(definition_elem).ok()
    }

    /// get the referenced definition ref as a string
    ///
    /// This function is an alternative to `definition()`; it is useful when the
    /// referenced definition is not loaded and can't be resolved.
    pub fn definition_ref(&self) -> Option<String> {
        self.element()
            .get_sub_element(ElementName::DefinitionRef)?
            .character_data()?
            .string_value()
    }

    /// Set the target of the reference
    pub fn set_target(&self, target: &Element) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::ValueRef)?
            .set_reference_target(target)?;

        Ok(())
    }

    /// Get the target of the reference
    pub fn target(&self) -> Option<Element> {
        self.element()
            .get_sub_element(ElementName::ValueRef)?
            .get_reference_target()
            .ok()
    }

    /// set the index of the reference
    ///
    /// If the reference definition has `requiresIndex` set to `true`, then the reference
    /// must have an index. Otherwise the index is meaningless.
    pub fn set_index(&self, index: Option<u64>) -> Result<(), AutosarAbstractionError> {
        if let Some(index) = index {
            self.element()
                .get_or_create_sub_element(ElementName::Index)?
                .set_character_data(index)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::Index);
        }

        Ok(())
    }

    /// get the index of the reference
    ///
    /// If the reference definition has `requiresIndex` set to `true`, then the reference
    /// must have an index. Otherwise the index is meaningless.
    pub fn index(&self) -> Option<u64> {
        self.element()
            .get_sub_element(ElementName::Index)?
            .character_data()?
            .parse_integer()
    }

    /// set the isAutoValue flag
    ///
    /// If the reference definition has `withAuto` set to `true`, then the reference is allowed to have an auto value.
    pub fn set_is_auto_value(&self, is_auto_value: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(is_auto_value) = is_auto_value {
            self.element()
                .get_or_create_sub_element(ElementName::IsAutoValue)?
                .set_character_data(is_auto_value)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::IsAutoValue);
        }

        Ok(())
    }

    /// get the isAutoValue flag
    pub fn is_auto_value(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::IsAutoValue)?
            .character_data()?
            .parse_bool()
    }
}

//#########################################################

/// The `EcucAnyReferenceValue` is an enum that can hold either of the reference value types
/// It is used as a return type for the iterator of reference values
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EcucAnyReferenceValue {
    /// An instance reference value
    Instance(EcucInstanceReferenceValue),
    /// A normal reference value
    Reference(EcucReferenceValue),
}

impl TryFrom<Element> for EcucAnyReferenceValue {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::EcucInstanceReferenceValue => {
                Ok(EcucAnyReferenceValue::Instance(EcucInstanceReferenceValue(element)))
            }
            ElementName::EcucReferenceValue => Ok(EcucAnyReferenceValue::Reference(EcucReferenceValue(element))),
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "EcucAnyReferenceValue".to_string(),
            }),
        }
    }
}

impl AbstractionElement for EcucAnyReferenceValue {
    fn element(&self) -> &Element {
        match self {
            EcucAnyReferenceValue::Instance(instance) => instance.element(),
            EcucAnyReferenceValue::Reference(reference) => reference.element(),
        }
    }
}

impl IdentifiableAbstractionElement for EcucAnyReferenceValue {}

//#########################################################

#[cfg(test)]
mod test {
    use crate::{
        AbstractionElement, AutosarModelAbstraction, ecu_configuration::EcucAnyReferenceValue,
        software_component::AbstractSwComponentType,
    };
    use autosar_data::AutosarVersion;

    #[test]
    fn test_ecu_configuration_values() {
        let definition_model = AutosarModelAbstraction::create("definition.arxml", AutosarVersion::LATEST);
        let def_package = definition_model.get_or_create_package("/def_package").unwrap();

        let values_model = AutosarModelAbstraction::create("values.arxml", AutosarVersion::LATEST);
        let val_package = values_model.get_or_create_package("/val_package").unwrap();

        // create a definition for the ECU configuration
        let module_def = def_package.create_ecuc_module_def("ModuleDef").unwrap();
        let container_def = module_def.create_param_conf_container_def("ContainerDef").unwrap();
        let instance_ref_def = container_def
            .create_instance_reference_def("InstanceRefDef", "origin")
            .unwrap();
        let foreign_reference_def = container_def
            .create_foreign_reference_def("ForeignRefDef", "origin")
            .unwrap();
        let choice_reference_def = container_def
            .create_choice_reference_def("ChoiceRefDef", "origin")
            .unwrap();
        let reference_def = container_def.create_reference_def("RefDef", "origin").unwrap();
        let uri_reference_def = container_def.create_uri_reference_def("UriRefDef", "origin").unwrap();

        // create an ecu configuration based on the definition model
        let ecuc_config_values = val_package
            .create_ecuc_module_configuration_values("Module", &module_def)
            .unwrap();
        let container_values = ecuc_config_values
            .create_container_value("Container", &container_def)
            .unwrap();

        // create an instance reference value
        // an InstanceReferenceValue can only refer to certain elements; in oder to be able to set up a valid
        // instance reference we need to create the "infrastructure" in the model
        let comp = val_package.create_composition_sw_component_type("comp").unwrap();
        let port_interface = val_package.create_sender_receiver_interface("sr_interface").unwrap();
        let r_port_prototype = comp.create_r_port("sr_r_port", &port_interface).unwrap();
        // create the instance reference value
        let instance_ref = container_values
            .create_instance_reference(
                &instance_ref_def,
                &[r_port_prototype.element()],
                r_port_prototype.element(),
            )
            .unwrap();
        // the definition is in a different model, so it can't be resolved
        assert!(instance_ref.definition().is_none());
        assert_eq!(instance_ref.definition_ref(), instance_ref_def.element().path().ok());
        let (target_context, target) = instance_ref.target().unwrap();
        assert_eq!(&target, r_port_prototype.element());
        assert_eq!(target_context, &[r_port_prototype.element().clone()]);
        assert_eq!(instance_ref.index(), None);
        instance_ref.set_index(Some(42)).unwrap();
        assert_eq!(instance_ref.index(), Some(42));
        assert_eq!(instance_ref.is_auto_value(), None);
        instance_ref.set_is_auto_value(Some(true)).unwrap();
        assert_eq!(instance_ref.is_auto_value(), Some(true));

        let foreign_ref = container_values
            .create_reference_value(&foreign_reference_def, val_package.element())
            .unwrap();
        assert!(foreign_ref.definition().is_none());
        assert_eq!(
            foreign_ref.definition_ref(),
            foreign_reference_def.element().path().ok()
        );
        assert_eq!(&foreign_ref.target().unwrap(), val_package.element());
        assert_eq!(foreign_ref.index(), None);
        foreign_ref.set_index(Some(42)).unwrap();
        assert_eq!(foreign_ref.index(), Some(42));
        assert_eq!(foreign_ref.is_auto_value(), None);
        foreign_ref.set_is_auto_value(Some(true)).unwrap();
        assert_eq!(foreign_ref.is_auto_value(), Some(true));

        let choice_ref = container_values
            .create_reference_value(&choice_reference_def, val_package.element())
            .unwrap();
        assert!(choice_ref.definition().is_none());
        assert_eq!(choice_ref.definition_ref(), choice_reference_def.element().path().ok());
        assert_eq!(&choice_ref.target().unwrap(), val_package.element());
        assert_eq!(choice_ref.index(), None);
        choice_ref.set_index(Some(42)).unwrap();
        assert_eq!(choice_ref.index(), Some(42));
        assert_eq!(choice_ref.is_auto_value(), None);
        choice_ref.set_is_auto_value(Some(true)).unwrap();
        assert_eq!(choice_ref.is_auto_value(), Some(true));

        let ref_ref = container_values
            .create_reference_value(&reference_def, val_package.element())
            .unwrap();
        assert!(ref_ref.definition().is_none());
        assert_eq!(ref_ref.definition_ref(), reference_def.element().path().ok());
        assert_eq!(&ref_ref.target().unwrap(), val_package.element());
        assert_eq!(ref_ref.index(), None);
        ref_ref.set_index(Some(42)).unwrap();
        assert_eq!(ref_ref.index(), Some(42));
        assert_eq!(ref_ref.is_auto_value(), None);
        ref_ref.set_is_auto_value(Some(true)).unwrap();
        assert_eq!(ref_ref.is_auto_value(), Some(true));

        let uri_ref = container_values
            .create_reference_value(&uri_reference_def, val_package.element())
            .unwrap();
        assert!(uri_ref.definition().is_none());
        assert_eq!(uri_ref.definition_ref(), uri_reference_def.element().path().ok());
        assert_eq!(&uri_ref.target().unwrap(), val_package.element());
        assert_eq!(uri_ref.index(), None);
        uri_ref.set_index(Some(42)).unwrap();
        assert_eq!(uri_ref.index(), Some(42));
        assert_eq!(uri_ref.is_auto_value(), None);
        uri_ref.set_is_auto_value(Some(true)).unwrap();
        assert_eq!(uri_ref.is_auto_value(), Some(true));

        assert_eq!(container_values.reference_values().count(), 5);

        let any_ref = EcucAnyReferenceValue::try_from(instance_ref.element().clone()).unwrap();
        assert!(matches!(any_ref, EcucAnyReferenceValue::Instance(_)));
        assert_eq!(any_ref.element(), instance_ref.element());
        let any_ref = EcucAnyReferenceValue::try_from(foreign_ref.element().clone()).unwrap();
        assert!(matches!(any_ref, EcucAnyReferenceValue::Reference(_)));
        assert_eq!(any_ref.element(), foreign_ref.element());
        let err = EcucAnyReferenceValue::try_from(val_package.element().clone());
        assert!(err.is_err());
    }
}
