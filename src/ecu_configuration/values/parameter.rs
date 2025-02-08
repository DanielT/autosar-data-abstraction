use crate::{
    abstraction_element,
    ecu_configuration::{EcucNumericalParamDef, EcucParameterDef, EcucTextualParamDef},
    AbstractionElement, AutosarAbstractionError,
};
use autosar_data::{Element, ElementName};

//#########################################################

/// The `EcucAddInfoParamValue` holds descriptive text and takes the role of a parameter in the ECU configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucAddInfoParamValue(Element);
abstraction_element!(EcucAddInfoParamValue, EcucAddInfoParamValue);

// Stub - does anyone actually use this?

//#########################################################

/// The `EcucNumericalParamValue` holds a numerical value and can represent boolean, float or int parameter definitions.
///
/// Internally this value is stored as a string; in additon to the value() function, there are also
/// value_bool(), value_int() and value_float() functions, which parse the string and should be used as appropriate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucNumericalParamValue(Element);
abstraction_element!(EcucNumericalParamValue, EcucNumericalParamValue);

impl EcucNumericalParamValue {
    pub(crate) fn new<T: EcucNumericalParamDef>(
        parent: &Element,
        definition: &T,
        value: &str,
    ) -> Result<Self, AutosarAbstractionError> {
        let numerical_param_elem = parent.create_sub_element(ElementName::EcucNumericalParamValue)?;

        let numerical_param = Self(numerical_param_elem);
        numerical_param.set_definition(definition)?;
        numerical_param.set_value(value)?;

        Ok(numerical_param)
    }

    /// set the parameter definition reference
    pub fn set_definition<T: EcucNumericalParamDef>(&self, definition: &T) -> Result<(), AutosarAbstractionError> {
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
    pub fn definition(&self) -> Option<EcucParameterDef> {
        let definition_elem = self
            .element()
            .get_sub_element(ElementName::DefinitionRef)?
            .get_reference_target()
            .ok()?;
        EcucParameterDef::try_from(definition_elem).ok()
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

    /// set the numerical value as a string
    pub fn set_value(&self, value: &str) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::Value)?
            .set_character_data(value)?;

        Ok(())
    }

    /// get the numerical value as a string
    pub fn value(&self) -> Option<String> {
        self.element()
            .get_sub_element(ElementName::Value)?
            .character_data()?
            .string_value()
    }

    /// get the numerical value as a boolean
    pub fn value_bool(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::Value)?
            .character_data()?
            .parse_bool()
    }

    /// get the numerical value as an integer
    pub fn value_int(&self) -> Option<i64> {
        self.element()
            .get_sub_element(ElementName::Value)?
            .character_data()?
            .parse_integer()
    }

    /// get the numerical value as a float
    pub fn value_float(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::Value)?
            .character_data()?
            .parse_float()
    }

    /// set the index of the parameter
    ///
    /// If the parameter definition has `requiresIndex` set to `true`, then the parameter
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

    /// get the index of the parameter
    ///
    /// If the parameter definition has `requiresIndex` set to `true`, then the parameter
    /// must have an index. Otherwise the index is meaningless.
    pub fn index(&self) -> Option<u64> {
        self.element()
            .get_sub_element(ElementName::Index)?
            .character_data()?
            .parse_integer()
    }

    /// set the isAutoValue flag
    ///
    /// If the parameter definition has `withAuto` set to `true`, then the parameter is allowed to have an auto value.
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

/// The `EcucTextualParamValue` holds a string value and can represent a enumeration,
///  string, multi-line string, function name or linker symbol parameter definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucTextualParamValue(Element);
abstraction_element!(EcucTextualParamValue, EcucTextualParamValue);

impl EcucTextualParamValue {
    pub(crate) fn new<T: EcucTextualParamDef>(
        parent: &Element,
        definition: &T,
        value: &str,
    ) -> Result<Self, AutosarAbstractionError> {
        let textual_param_elem = parent.create_sub_element(ElementName::EcucTextualParamValue)?;

        let textual_param = Self(textual_param_elem);
        textual_param.set_definition(definition)?;
        textual_param.set_value(value)?;

        Ok(textual_param)
    }

    /// set the parameter definition reference
    pub fn set_definition<T: EcucTextualParamDef>(&self, definition: &T) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::DefinitionRef)?
            .set_reference_target(definition.element())?;

        Ok(())
    }

    /// get the parameter definition
    ///
    /// This function returns the definition as an `EcucParameterDef` enum, which
    /// could contain either an `EcucStringParamDef`, `EcucMultiStringParamDef`,
    /// `EcucFunctionNameDef` or `EcucLinkerSymbolDef`.
    /// If the definition is not loaded, use `definition_ref()` instead.
    pub fn definition(&self) -> Option<EcucParameterDef> {
        let definition_elem = self
            .element()
            .get_sub_element(ElementName::DefinitionRef)?
            .get_reference_target()
            .ok()?;
        EcucParameterDef::try_from(definition_elem).ok()
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

    /// set the textual value
    pub fn set_value(&self, value: &str) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::Value)?
            .set_character_data(value)?;

        Ok(())
    }

    /// get the textual value
    pub fn value(&self) -> Option<String> {
        self.element()
            .get_sub_element(ElementName::Value)?
            .character_data()?
            .string_value()
    }

    /// set the index of the parameter
    ///
    /// If the parameter definition has `requiresIndex` set to `true`, then the parameter
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

    /// get the index of the parameter
    ///
    /// If the parameter definition has `requiresIndex` set to `true`, then the parameter
    /// must have an index. Otherwise the index is meaningless.
    pub fn index(&self) -> Option<u64> {
        self.element()
            .get_sub_element(ElementName::Index)?
            .character_data()?
            .parse_integer()
    }

    /// set the isAutoValue flag
    ///
    /// If the parameter definition has `withAuto` set to `true`, then the parameter is allowed to have an auto value.
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
    ///
    /// If the parameter definition has `withAuto` set to `true`, then the parameter is allowed to have an auto value.
    pub fn is_auto_value(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::IsAutoValue)?
            .character_data()?
            .parse_bool()
    }
}

//#########################################################

/// The `EcucParameterValue` is an enum that can hold an `EcucAddInfoParamValue`,
/// an `EcucNumericalParamValue` or an `EcucTextualParamValue`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EcucParameterValue {
    /// AddInfo parameter value
    AddInfo(EcucAddInfoParamValue),
    /// Numerical parameter value
    Numerical(EcucNumericalParamValue),
    /// Textual parameter value
    Textual(EcucTextualParamValue),
}

impl AbstractionElement for EcucParameterValue {
    fn element(&self) -> &Element {
        match self {
            EcucParameterValue::AddInfo(elem) => elem.element(),
            EcucParameterValue::Numerical(elem) => elem.element(),
            EcucParameterValue::Textual(elem) => elem.element(),
        }
    }
}

impl TryFrom<Element> for EcucParameterValue {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::EcucAddInfoParamValue => Ok(EcucParameterValue::AddInfo(EcucAddInfoParamValue(element))),
            ElementName::EcucNumericalParamValue => Ok(EcucParameterValue::Numerical(EcucNumericalParamValue(element))),
            ElementName::EcucTextualParamValue => Ok(EcucParameterValue::Textual(EcucTextualParamValue(element))),
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "EcucParameterValue".to_string(),
            }),
        }
    }
}

//#########################################################

#[cfg(test)]
mod test {
    use crate::{
        ecu_configuration::{EcucParameterDef, EcucParameterValue},
        AbstractionElement, AutosarModelAbstraction,
    };
    use autosar_data::{AutosarVersion, ElementName};

    #[test]
    fn test_parameter_values() {
        let definition_model = AutosarModelAbstraction::create("definition.arxml", AutosarVersion::LATEST);
        let def_package = definition_model.get_or_create_package("/def_package").unwrap();

        let values_model = AutosarModelAbstraction::create("values.arxml", AutosarVersion::LATEST);
        let val_package = values_model.get_or_create_package("/val_package").unwrap();

        // create a definition for the ECU configuration
        let module_def = def_package.create_ecuc_module_def("ModuleDef").unwrap();
        let container_def = module_def.create_param_conf_container_def("ContainerDef").unwrap();
        let int_param_def = container_def
            .create_integer_param_def("IntParam", "AUTOSAR_ECUC")
            .unwrap();
        let float_param_def = container_def
            .create_float_param_def("FloatParam", "AUTOSAR_ECUC")
            .unwrap();
        let bool_param_def = container_def
            .create_boolean_param_def("BoolParam", "AUTOSAR_ECUC")
            .unwrap();
        let string_param_def = container_def
            .create_string_param_def("StringParam", "AUTOSAR_ECUC")
            .unwrap();
        let fnc_param_def = container_def
            .create_function_name_def("FncParam", "AUTOSAR_ECUC")
            .unwrap();
        let link_param_def = container_def
            .create_linker_symbol_def("LinkParam", "AUTOSAR_ECUC")
            .unwrap();
        let enum_param_def = container_def
            .create_enumeration_param_def("EnumParam", "AUTOSAR_ECUC")
            .unwrap();
        enum_param_def.create_enumeration_literal("EnumLiteral_1").unwrap();
        enum_param_def.create_enumeration_literal("EnumLiteral_2").unwrap();

        // create an ecu configuration based on the definition model
        let ecuc_value_collection = val_package.create_ecuc_value_collection("EcucValues").unwrap();
        let ecuc_config_values = val_package
            .create_ecuc_module_configuration_values("Module", &module_def)
            .unwrap();
        ecuc_value_collection
            .add_module_configuration(&ecuc_config_values)
            .unwrap();
        let container_values = ecuc_config_values
            .create_container_value("Container", &container_def)
            .unwrap();

        let int_param_value = container_values
            .create_numerical_param_value(&int_param_def, "42")
            .unwrap();
        assert_eq!(int_param_value.value_int(), Some(42));
        int_param_value.set_value("43").unwrap();
        assert_eq!(int_param_value.value_int(), Some(43));
        int_param_value.set_index(Some(1)).unwrap();
        assert_eq!(int_param_value.index(), Some(1));
        int_param_value.set_is_auto_value(Some(true)).unwrap();
        assert_eq!(int_param_value.is_auto_value(), Some(true));
        assert_eq!(int_param_value.definition_ref(), int_param_def.element().path().ok());
        // the definition is not loaded in the same model, so we can't get it
        assert!(int_param_value.definition().is_none());

        let float_param_value = container_values
            .create_numerical_param_value(&float_param_def, "3.14")
            .unwrap();
        assert_eq!(float_param_value.value().as_deref(), Some("3.14"));
        assert_eq!(float_param_value.value_float(), Some(3.14));
        float_param_value.set_value("2.71").unwrap();
        assert_eq!(float_param_value.value_float(), Some(2.71));
        float_param_value.set_index(Some(2)).unwrap();
        assert_eq!(float_param_value.index(), Some(2));
        float_param_value.set_is_auto_value(Some(false)).unwrap();
        assert_eq!(float_param_value.is_auto_value(), Some(false));
        assert_eq!(
            float_param_value.definition_ref(),
            float_param_def.element().path().ok()
        );

        let bool_param_value = container_values
            .create_numerical_param_value(&bool_param_def, "true")
            .unwrap();
        assert_eq!(bool_param_value.value().as_deref(), Some("true"));
        assert_eq!(bool_param_value.value_bool(), Some(true));
        bool_param_value.set_value("false").unwrap();
        assert_eq!(bool_param_value.value_bool(), Some(false));
        bool_param_value.set_index(Some(3)).unwrap();
        assert_eq!(bool_param_value.index(), Some(3));
        bool_param_value.set_is_auto_value(None).unwrap();
        assert_eq!(bool_param_value.is_auto_value(), None);
        assert_eq!(bool_param_value.definition_ref(), bool_param_def.element().path().ok());

        let string_param_value = container_values
            .create_textual_param_value(&string_param_def, "Hello, World!")
            .unwrap();
        assert_eq!(string_param_value.value().as_deref(), Some("Hello, World!"));
        string_param_value.set_value("Goodbye, World!").unwrap();
        assert_eq!(string_param_value.value().as_deref(), Some("Goodbye, World!"));
        string_param_value.set_index(Some(4)).unwrap();
        assert_eq!(string_param_value.index(), Some(4));
        string_param_value.set_is_auto_value(Some(true)).unwrap();
        assert_eq!(string_param_value.is_auto_value(), Some(true));
        assert_eq!(
            string_param_value.definition_ref(),
            string_param_def.element().path().ok()
        );

        let fnc_param_value = container_values
            .create_textual_param_value(&fnc_param_def, "function_name")
            .unwrap();
        let link_param_value = container_values
            .create_textual_param_value(&link_param_def, "linker_symbol")
            .unwrap();

        let enum_param_value = container_values
            .create_textual_param_value(&enum_param_def, "EnumLiteral_1")
            .unwrap();
        assert_eq!(enum_param_value.value().as_deref(), Some("EnumLiteral_1"));
        enum_param_value.set_value("EnumLiteral_2").unwrap();
        assert_eq!(enum_param_value.value().as_deref(), Some("EnumLiteral_2"));
        enum_param_value.set_index(Some(5)).unwrap();
        assert_eq!(enum_param_value.index(), Some(5));
        enum_param_value.set_is_auto_value(Some(false)).unwrap();
        assert_eq!(enum_param_value.is_auto_value(), Some(false));
        assert_eq!(enum_param_value.definition_ref(), enum_param_def.element().path().ok());

        let mut parameters_iter = container_values.parameter_values();
        assert_eq!(
            parameters_iter.next().unwrap(),
            EcucParameterValue::Numerical(int_param_value.clone())
        );
        assert_eq!(
            parameters_iter.next().unwrap(),
            EcucParameterValue::Numerical(float_param_value.clone())
        );
        assert_eq!(
            parameters_iter.next().unwrap(),
            EcucParameterValue::Numerical(bool_param_value.clone())
        );
        assert_eq!(
            parameters_iter.next().unwrap(),
            EcucParameterValue::Textual(string_param_value.clone())
        );
        assert_eq!(
            parameters_iter.next().unwrap(),
            EcucParameterValue::Textual(fnc_param_value.clone())
        );
        assert_eq!(
            parameters_iter.next().unwrap(),
            EcucParameterValue::Textual(link_param_value.clone())
        );
        assert_eq!(
            parameters_iter.next().unwrap(),
            EcucParameterValue::Textual(enum_param_value.clone())
        );
        assert_eq!(container_values.parameter_values().count(), 7);
        let ecuc_param = container_values.parameter_values().next().unwrap();
        assert_eq!(ecuc_param.element(), int_param_value.element());

        // copy the definition into the value model
        // once the definition and values are in the same model, we can get the definition directly
        values_model
            .root_element()
            .get_sub_element(ElementName::ArPackages)
            .unwrap()
            .create_copied_sub_element(def_package.element())
            .unwrap();
        // get the definitions from the value model
        let int_param_def = EcucParameterDef::try_from(
            values_model
                .get_element_by_path(&int_param_def.element().path().unwrap())
                .unwrap(),
        )
        .unwrap();
        let float_param_def = EcucParameterDef::try_from(
            values_model
                .get_element_by_path(&float_param_def.element().path().unwrap())
                .unwrap(),
        )
        .unwrap();
        let bool_param_def = EcucParameterDef::try_from(
            values_model
                .get_element_by_path(&bool_param_def.element().path().unwrap())
                .unwrap(),
        )
        .unwrap();
        let string_param_def = EcucParameterDef::try_from(
            values_model
                .get_element_by_path(&string_param_def.element().path().unwrap())
                .unwrap(),
        )
        .unwrap();
        let enum_param_def = EcucParameterDef::try_from(
            values_model
                .get_element_by_path(&enum_param_def.element().path().unwrap())
                .unwrap(),
        )
        .unwrap();

        // get the definition from the value model
        assert_eq!(int_param_value.definition().unwrap(), int_param_def);
        assert_eq!(float_param_value.definition().unwrap(), float_param_def);
        assert_eq!(bool_param_value.definition().unwrap(), bool_param_def);
        assert_eq!(string_param_value.definition().unwrap(), string_param_def);
        assert_eq!(enum_param_value.definition().unwrap(), enum_param_def);

        assert_eq!(int_param_value.definition().unwrap().element(), int_param_def.element());
    }
}
