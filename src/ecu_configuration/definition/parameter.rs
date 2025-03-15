use crate::ecu_configuration::{EcucCommonAttributes, EcucDefinitionElement};
use crate::{AbstractionElement, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element};
use autosar_data::{Element, ElementName};

//#########################################################

/// This trait provides the common operation shared by all string parameter definitions
pub trait EcucAbstractStringParamDef: EcucCommonAttributes {
    /// set or remove the max length attribute
    fn set_max_length(&self, max_length: Option<u32>) -> Result<(), AutosarAbstractionError>;

    /// get the max length attribute
    fn max_length(&self) -> Option<u32>;

    /// set or remove the min length attribute
    fn set_min_length(&self, min_length: Option<u32>) -> Result<(), AutosarAbstractionError>;

    /// get the min length attribute
    fn min_length(&self) -> Option<u32>;

    /// set or remove the regular expression attribute
    /// The regular expression is a string that is used to validate the string parameter
    fn set_regular_expression(&self, regular_expression: Option<&str>) -> Result<(), AutosarAbstractionError>;

    /// get the regular expression attribute
    /// The regular expression is a string that is used to validate the string parameter
    fn regular_expression(&self) -> Option<String>;

    /// set or remove the default value attribute
    fn set_default_value(&self, default_value: Option<&str>) -> Result<(), AutosarAbstractionError>;

    /// get the default value attribute
    fn default_value(&self) -> Option<String>;
}

macro_rules! string_param {
    ($name: ident, $elemname_variants: ident, $elemname_conditional: ident) => {
        impl EcucAbstractStringParamDef for $name {
            fn set_max_length(&self, max_length: Option<u32>) -> Result<(), AutosarAbstractionError> {
                if let Some(max_length) = max_length {
                    self.element()
                        .get_or_create_sub_element(ElementName::$elemname_variants)?
                        .get_or_create_sub_element(ElementName::$elemname_conditional)?
                        .get_or_create_sub_element(ElementName::MaxLength)?
                        .set_character_data(max_length as u64)?;
                } else {
                    if let Some(espdc) = self
                        .element()
                        .get_sub_element(ElementName::$elemname_variants)
                        .and_then(|espdv| espdv.get_sub_element(ElementName::$elemname_conditional))
                    {
                        let _ = espdc.remove_sub_element_kind(ElementName::MaxLength);
                    }
                }

                Ok(())
            }

            /// get the max length attribute
            fn max_length(&self) -> Option<u32> {
                self.element()
                    .get_sub_element(ElementName::$elemname_variants)?
                    .get_sub_element(ElementName::$elemname_conditional)?
                    .get_sub_element(ElementName::MaxLength)?
                    .character_data()?
                    .parse_integer()
            }

            /// set or remove the min length attribute
            fn set_min_length(&self, min_length: Option<u32>) -> Result<(), AutosarAbstractionError> {
                if let Some(min_length) = min_length {
                    self.element()
                        .get_or_create_sub_element(ElementName::$elemname_variants)?
                        .get_or_create_sub_element(ElementName::$elemname_conditional)?
                        .get_or_create_sub_element(ElementName::MinLength)?
                        .set_character_data(min_length as u64)?;
                } else {
                    if let Some(espdc) = self
                        .element()
                        .get_sub_element(ElementName::$elemname_variants)
                        .and_then(|espdv| espdv.get_sub_element(ElementName::$elemname_conditional))
                    {
                        let _ = espdc.remove_sub_element_kind(ElementName::MinLength);
                    }
                }

                Ok(())
            }

            /// get the min length attribute
            fn min_length(&self) -> Option<u32> {
                self.element()
                    .get_sub_element(ElementName::$elemname_variants)?
                    .get_sub_element(ElementName::$elemname_conditional)?
                    .get_sub_element(ElementName::MinLength)?
                    .character_data()?
                    .parse_integer()
            }

            /// set or remove the regular expression attribute
            /// The regular expression is a string that is used to validate the string parameter
            fn set_regular_expression(&self, regular_expression: Option<&str>) -> Result<(), AutosarAbstractionError> {
                if let Some(regular_expression) = regular_expression {
                    self.element()
                        .get_or_create_sub_element(ElementName::$elemname_variants)?
                        .get_or_create_sub_element(ElementName::$elemname_conditional)?
                        .get_or_create_sub_element(ElementName::RegularExpression)?
                        .set_character_data(regular_expression)?;
                } else {
                    if let Some(espdc) = self
                        .element()
                        .get_sub_element(ElementName::$elemname_variants)
                        .and_then(|espdv| espdv.get_sub_element(ElementName::$elemname_conditional))
                    {
                        let _ = espdc.remove_sub_element_kind(ElementName::RegularExpression);
                    }
                }

                Ok(())
            }

            /// get the regular expression attribute
            /// The regular expression is a string that is used to validate the string parameter
            fn regular_expression(&self) -> Option<String> {
                self.element()
                    .get_sub_element(ElementName::$elemname_variants)?
                    .get_sub_element(ElementName::$elemname_conditional)?
                    .get_sub_element(ElementName::RegularExpression)?
                    .character_data()?
                    .string_value()
            }

            /// set or remove the default value attribute
            fn set_default_value(&self, default_value: Option<&str>) -> Result<(), AutosarAbstractionError> {
                if let Some(default_value) = default_value {
                    self.element()
                        .get_or_create_sub_element(ElementName::$elemname_variants)?
                        .get_or_create_sub_element(ElementName::$elemname_conditional)?
                        .get_or_create_sub_element(ElementName::DefaultValue)?
                        .set_character_data(default_value)?;
                } else {
                    if let Some(espdc) = self
                        .element()
                        .get_sub_element(ElementName::$elemname_variants)
                        .and_then(|espdv| espdv.get_sub_element(ElementName::$elemname_conditional))
                    {
                        let _ = espdc.remove_sub_element_kind(ElementName::DefaultValue);
                    }
                }

                Ok(())
            }

            /// get the default value attribute
            fn default_value(&self) -> Option<String> {
                self.element()
                    .get_sub_element(ElementName::$elemname_variants)?
                    .get_sub_element(ElementName::$elemname_conditional)?
                    .get_sub_element(ElementName::DefaultValue)?
                    .character_data()?
                    .string_value()
            }
        }
        impl EcucTextualParamDef for $name {}
    };
}

//#########################################################

/// marker trait for all parameter defintions
pub trait EcucParamDef: EcucCommonAttributes {}

//#########################################################

/// marker trait for numerical parameter defintions: EcucFloatParamDef, EcucIntegerParamDef, EcucBooleanParamDef
pub trait EcucNumericalParamDef: EcucParamDef {}

//#########################################################

/// marker trait for textual parameter defintions: EcucEnumerationParamDef,
/// EcucFunctionNameDef, EcucLinkerSymbolDef, EcucMultilineStringParamDef, EcucStringParamDef
///
/// This grouping is determined by the usage in the value definition: `EcucTextualParamValue` can refer to any of these
pub trait EcucTextualParamDef: EcucParamDef {}

//#########################################################

/// `EcucAddInfoParamDef` is used to specify the need for formated text in the ECU configuration value description
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucAddInfoParamDef(Element);
abstraction_element!(EcucAddInfoParamDef, EcucAddInfoParamDef);
impl IdentifiableAbstractionElement for EcucAddInfoParamDef {}
impl EcucCommonAttributes for EcucAddInfoParamDef {}
impl EcucDefinitionElement for EcucAddInfoParamDef {}

impl EcucAddInfoParamDef {
    pub(crate) fn new(name: &str, parameters_elem: &Element, origin: &str) -> Result<Self, AutosarAbstractionError> {
        let addinfo_def_elem = parameters_elem.create_named_sub_element(ElementName::EcucAddInfoParamDef, name)?;

        let addinfo_def = Self(addinfo_def_elem);
        addinfo_def.set_origin(origin)?;

        Ok(addinfo_def)
    }
}

//#########################################################

/// `EcucBooleanParamDef` is used to specify a boolean parameter in the ECU configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucBooleanParamDef(Element);
abstraction_element!(EcucBooleanParamDef, EcucBooleanParamDef);
impl IdentifiableAbstractionElement for EcucBooleanParamDef {}
impl EcucCommonAttributes for EcucBooleanParamDef {}
impl EcucDefinitionElement for EcucBooleanParamDef {}
impl EcucParamDef for EcucBooleanParamDef {}
impl EcucNumericalParamDef for EcucBooleanParamDef {}

impl EcucBooleanParamDef {
    pub(crate) fn new(name: &str, parameters_elem: &Element, origin: &str) -> Result<Self, AutosarAbstractionError> {
        let boolean_def_elem = parameters_elem.create_named_sub_element(ElementName::EcucBooleanParamDef, name)?;

        let boolean_def = Self(boolean_def_elem);
        boolean_def.set_origin(origin)?;

        Ok(boolean_def)
    }

    /// set the default value of the boolean parameter
    pub fn set_default_value(&self, default_value: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(default_value) = default_value {
            self.element()
                .get_or_create_sub_element(ElementName::DefaultValue)?
                .set_character_data(default_value)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::DefaultValue);
        }

        Ok(())
    }

    /// get the default value of the boolean parameter
    pub fn default_value(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::DefaultValue)?
            .character_data()?
            .parse_bool()
    }
}

//#########################################################

/// `EcucEnumerationParamDef` is used to specify an enumeration parameter in the ECU configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucEnumerationParamDef(Element);
abstraction_element!(EcucEnumerationParamDef, EcucEnumerationParamDef);
impl IdentifiableAbstractionElement for EcucEnumerationParamDef {}
impl EcucCommonAttributes for EcucEnumerationParamDef {}
impl EcucDefinitionElement for EcucEnumerationParamDef {}
impl EcucParamDef for EcucEnumerationParamDef {}
impl EcucTextualParamDef for EcucEnumerationParamDef {}

impl EcucEnumerationParamDef {
    pub(crate) fn new(name: &str, parameters_elem: &Element, origin: &str) -> Result<Self, AutosarAbstractionError> {
        let enumeration_def_elem =
            parameters_elem.create_named_sub_element(ElementName::EcucEnumerationParamDef, name)?;

        let enumeration_def = Self(enumeration_def_elem);
        enumeration_def.set_origin(origin)?;

        Ok(enumeration_def)
    }

    /// create a new enumeration literal
    pub fn create_enumeration_literal(&self, name: &str) -> Result<EcucEnumerationLiteralDef, AutosarAbstractionError> {
        let literals_elem = self.element().get_or_create_sub_element(ElementName::Literals)?;

        EcucEnumerationLiteralDef::new(name, &literals_elem)
    }

    /// iterate over all enumeration literals
    pub fn enumeration_literals(&self) -> impl Iterator<Item = EcucEnumerationLiteralDef> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::Literals)
            .into_iter()
            .flat_map(|literals_elem| literals_elem.sub_elements())
            .filter_map(|elem| elem.try_into().ok())
    }

    /// set the default value of the enumeration parameter
    ///
    /// Note: enumeration literals must be created first, since the default value must match one of the literals
    pub fn set_default_value(&self, default_value: Option<&str>) -> Result<(), AutosarAbstractionError> {
        if let Some(default_value) = default_value {
            if !self
                .enumeration_literals()
                .any(|literal| literal.name().as_deref() == Some(default_value))
            {
                return Err(AutosarAbstractionError::InvalidParameter(format!(
                    "Default value {default_value} not found in enumeration literals"
                )));
            }
            self.element()
                .get_or_create_sub_element(ElementName::DefaultValue)?
                .set_character_data(default_value)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::DefaultValue);
        }

        Ok(())
    }

    /// get the default value of the enumeration parameter
    pub fn default_value(&self) -> Option<String> {
        self.element()
            .get_sub_element(ElementName::DefaultValue)?
            .character_data()?
            .string_value()
    }
}

//#########################################################

/// `EcucEnumerationLiteralDef` is used to specify an enumeration literal in the ECU configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucEnumerationLiteralDef(Element);
abstraction_element!(EcucEnumerationLiteralDef, EcucEnumerationLiteralDef);
impl IdentifiableAbstractionElement for EcucEnumerationLiteralDef {}

impl EcucEnumerationLiteralDef {
    pub(crate) fn new(name: &str, literals_elem: &Element) -> Result<Self, AutosarAbstractionError> {
        let enumeration_literal_def_elem =
            literals_elem.create_named_sub_element(ElementName::EcucEnumerationLiteralDef, name)?;

        Ok(Self(enumeration_literal_def_elem))
    }
}

//#########################################################

/// `EcucFloatParamDef` is used to specify a float parameter in the ECU configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucFloatParamDef(Element);
abstraction_element!(EcucFloatParamDef, EcucFloatParamDef);
impl IdentifiableAbstractionElement for EcucFloatParamDef {}
impl EcucCommonAttributes for EcucFloatParamDef {}
impl EcucDefinitionElement for EcucFloatParamDef {}
impl EcucParamDef for EcucFloatParamDef {}
impl EcucNumericalParamDef for EcucFloatParamDef {}

impl EcucFloatParamDef {
    pub(crate) fn new(name: &str, parameters_elem: &Element, origin: &str) -> Result<Self, AutosarAbstractionError> {
        let float_def_elem = parameters_elem.create_named_sub_element(ElementName::EcucFloatParamDef, name)?;

        let float_def = Self(float_def_elem);
        float_def.set_origin(origin)?;

        Ok(float_def)
    }

    /// set the default value of the float parameter
    pub fn set_default_value(&self, default_value: Option<f64>) -> Result<(), AutosarAbstractionError> {
        if let Some(default_value) = default_value {
            self.element()
                .get_or_create_sub_element(ElementName::DefaultValue)?
                .set_character_data(default_value)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::DefaultValue);
        }

        Ok(())
    }

    /// get the default value of the float parameter
    pub fn default_value(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::DefaultValue)?
            .character_data()?
            .parse_float()
    }

    /// set the min value of the float parameter
    pub fn set_min(&self, min: Option<f64>) -> Result<(), AutosarAbstractionError> {
        if let Some(min) = min {
            self.element()
                .get_or_create_sub_element(ElementName::Min)?
                .set_character_data(min)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::Min);
        }

        Ok(())
    }

    /// get the min value of the float parameter
    pub fn min(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::Min)?
            .character_data()?
            .parse_float()
    }

    /// set the max value of the float parameter
    pub fn set_max(&self, max: Option<f64>) -> Result<(), AutosarAbstractionError> {
        if let Some(max) = max {
            self.element()
                .get_or_create_sub_element(ElementName::Max)?
                .set_character_data(max)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::Max);
        }

        Ok(())
    }

    /// get the max value of the float parameter
    pub fn max(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::Max)?
            .character_data()?
            .parse_float()
    }
}

//#########################################################

/// `EcucIntegerParamDef` is used to specify an integer parameter in the ECU configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucIntegerParamDef(Element);
abstraction_element!(EcucIntegerParamDef, EcucIntegerParamDef);
impl IdentifiableAbstractionElement for EcucIntegerParamDef {}
impl EcucCommonAttributes for EcucIntegerParamDef {}
impl EcucDefinitionElement for EcucIntegerParamDef {}
impl EcucParamDef for EcucIntegerParamDef {}
impl EcucNumericalParamDef for EcucIntegerParamDef {}

impl EcucIntegerParamDef {
    pub(crate) fn new(name: &str, parameters_elem: &Element, origin: &str) -> Result<Self, AutosarAbstractionError> {
        let integer_def_elem = parameters_elem.create_named_sub_element(ElementName::EcucIntegerParamDef, name)?;

        let integer_def = Self(integer_def_elem);
        integer_def.set_origin(origin)?;

        Ok(integer_def)
    }

    /// set the default value of the integer parameter
    pub fn set_default_value(&self, default_value: Option<i64>) -> Result<(), AutosarAbstractionError> {
        if let Some(default_value) = default_value {
            self.element()
                .get_or_create_sub_element(ElementName::DefaultValue)?
                .set_character_data(default_value.to_string())?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::DefaultValue);
        }

        Ok(())
    }

    /// get the default value of the integer parameter
    pub fn default_value(&self) -> Option<i64> {
        self.element()
            .get_sub_element(ElementName::DefaultValue)?
            .character_data()?
            .parse_integer()
    }

    /// set the min value of the integer parameter
    pub fn set_min(&self, min: Option<i64>) -> Result<(), AutosarAbstractionError> {
        if let Some(min) = min {
            self.element()
                .get_or_create_sub_element(ElementName::Min)?
                .set_character_data(min.to_string())?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::Min);
        }

        Ok(())
    }

    /// get the min value of the integer parameter
    pub fn min(&self) -> Option<i64> {
        self.element()
            .get_sub_element(ElementName::Min)?
            .character_data()?
            .parse_integer()
    }

    /// set the max value of the integer parameter
    pub fn set_max(&self, max: Option<i64>) -> Result<(), AutosarAbstractionError> {
        if let Some(max) = max {
            self.element()
                .get_or_create_sub_element(ElementName::Max)?
                .set_character_data(max.to_string())?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::Max);
        }

        Ok(())
    }

    /// get the max value of the integer parameter
    pub fn max(&self) -> Option<i64> {
        self.element()
            .get_sub_element(ElementName::Max)?
            .character_data()?
            .parse_integer()
    }
}

//#########################################################

/// `EcucFunctionNameDef` is used to specify a function name parameter in the ECU configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucFunctionNameDef(Element);
abstraction_element!(EcucFunctionNameDef, EcucFunctionNameDef);
impl IdentifiableAbstractionElement for EcucFunctionNameDef {}
impl EcucCommonAttributes for EcucFunctionNameDef {}
impl EcucDefinitionElement for EcucFunctionNameDef {}
impl EcucParamDef for EcucFunctionNameDef {}

string_param!(
    EcucFunctionNameDef,
    EcucFunctionNameDefVariants,
    EcucFunctionNameDefConditional
);

impl EcucFunctionNameDef {
    pub(crate) fn new(name: &str, parameters_elem: &Element, origin: &str) -> Result<Self, AutosarAbstractionError> {
        let function_name_def_elem =
            parameters_elem.create_named_sub_element(ElementName::EcucFunctionNameDef, name)?;

        let function_name_def = Self(function_name_def_elem);
        function_name_def.set_origin(origin)?;

        Ok(function_name_def)
    }
}

//#########################################################

/// `EcucLinkerSymbolDef` is used to specify a linker symbol parameter in the ECU configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucLinkerSymbolDef(Element);
abstraction_element!(EcucLinkerSymbolDef, EcucLinkerSymbolDef);
impl IdentifiableAbstractionElement for EcucLinkerSymbolDef {}
impl EcucCommonAttributes for EcucLinkerSymbolDef {}
impl EcucDefinitionElement for EcucLinkerSymbolDef {}
impl EcucParamDef for EcucLinkerSymbolDef {}

string_param!(
    EcucLinkerSymbolDef,
    EcucLinkerSymbolDefVariants,
    EcucLinkerSymbolDefConditional
);

impl EcucLinkerSymbolDef {
    pub(crate) fn new(name: &str, parameters_elem: &Element, origin: &str) -> Result<Self, AutosarAbstractionError> {
        let linker_symbol_def_elem =
            parameters_elem.create_named_sub_element(ElementName::EcucLinkerSymbolDef, name)?;

        let linker_symbol_def = Self(linker_symbol_def_elem);
        linker_symbol_def.set_origin(origin)?;

        Ok(linker_symbol_def)
    }
}

//#########################################################

/// `EcucMultilineStringParamDef` is used to specify a multiline string parameter in the ECU configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucMultilineStringParamDef(Element);
abstraction_element!(EcucMultilineStringParamDef, EcucMultilineStringParamDef);
impl IdentifiableAbstractionElement for EcucMultilineStringParamDef {}
impl EcucCommonAttributes for EcucMultilineStringParamDef {}
impl EcucDefinitionElement for EcucMultilineStringParamDef {}
impl EcucParamDef for EcucMultilineStringParamDef {}

string_param!(
    EcucMultilineStringParamDef,
    EcucMultilineStringParamDefVariants,
    EcucMultilineStringParamDefConditional
);

impl EcucMultilineStringParamDef {
    pub(crate) fn new(name: &str, parameters_elem: &Element, origin: &str) -> Result<Self, AutosarAbstractionError> {
        let multiline_string_def_elem =
            parameters_elem.create_named_sub_element(ElementName::EcucMultilineStringParamDef, name)?;

        let multiline_string_def = Self(multiline_string_def_elem);
        multiline_string_def.set_origin(origin)?;

        Ok(multiline_string_def)
    }
}

//#########################################################

/// `EcucStringParamDef` is used to specify a string parameter in the ECU configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucStringParamDef(Element);
abstraction_element!(EcucStringParamDef, EcucStringParamDef);
impl IdentifiableAbstractionElement for EcucStringParamDef {}
impl EcucCommonAttributes for EcucStringParamDef {}
impl EcucDefinitionElement for EcucStringParamDef {}
impl EcucParamDef for EcucStringParamDef {}

string_param!(
    EcucStringParamDef,
    EcucStringParamDefVariants,
    EcucStringParamDefConditional
);

impl EcucStringParamDef {
    pub(crate) fn new(name: &str, parameters_elem: &Element, origin: &str) -> Result<Self, AutosarAbstractionError> {
        let string_def_elem = parameters_elem.create_named_sub_element(ElementName::EcucStringParamDef, name)?;

        let string_def = Self(string_def_elem);
        string_def.set_origin(origin)?;

        Ok(string_def)
    }
}

//#########################################################

/// `EcucParameterDef` encapsulates all possible parameter types in the ECU configuration, and is used as a return type for the iterator
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EcucParameterDef {
    /// AddInfo parameter
    AddInfo(EcucAddInfoParamDef),
    /// Boolean parameter
    Boolean(EcucBooleanParamDef),
    /// Enumeration parameter
    Enumeration(EcucEnumerationParamDef),
    /// Float parameter
    Float(EcucFloatParamDef),
    /// Integer parameter
    Integer(EcucIntegerParamDef),
    /// FunctionName parameter
    FunctionName(EcucFunctionNameDef),
    /// LinkerSymbol parameter
    LinkerSymbol(EcucLinkerSymbolDef),
    /// MultilineString parameter
    MultilineString(EcucMultilineStringParamDef),
    /// String parameter
    String(EcucStringParamDef),
}

impl AbstractionElement for EcucParameterDef {
    fn element(&self) -> &Element {
        match self {
            EcucParameterDef::AddInfo(elem) => elem.element(),
            EcucParameterDef::Boolean(elem) => elem.element(),
            EcucParameterDef::Enumeration(elem) => elem.element(),
            EcucParameterDef::Float(elem) => elem.element(),
            EcucParameterDef::Integer(elem) => elem.element(),
            EcucParameterDef::FunctionName(elem) => elem.element(),
            EcucParameterDef::LinkerSymbol(elem) => elem.element(),
            EcucParameterDef::MultilineString(elem) => elem.element(),
            EcucParameterDef::String(elem) => elem.element(),
        }
    }
}

impl TryFrom<Element> for EcucParameterDef {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::EcucAddInfoParamDef => Ok(EcucParameterDef::AddInfo(element.try_into()?)),
            ElementName::EcucBooleanParamDef => Ok(EcucParameterDef::Boolean(element.try_into()?)),
            ElementName::EcucEnumerationParamDef => Ok(EcucParameterDef::Enumeration(element.try_into()?)),
            ElementName::EcucFloatParamDef => Ok(EcucParameterDef::Float(element.try_into()?)),
            ElementName::EcucIntegerParamDef => Ok(EcucParameterDef::Integer(element.try_into()?)),
            ElementName::EcucFunctionNameDef => Ok(EcucParameterDef::FunctionName(element.try_into()?)),
            ElementName::EcucLinkerSymbolDef => Ok(EcucParameterDef::LinkerSymbol(element.try_into()?)),
            ElementName::EcucMultilineStringParamDef => Ok(EcucParameterDef::MultilineString(element.try_into()?)),
            ElementName::EcucStringParamDef => Ok(EcucParameterDef::String(EcucStringParamDef(element))),
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "EcucParameterDef".to_string(),
            }),
        }
    }
}

impl IdentifiableAbstractionElement for EcucParameterDef {}
impl EcucDefinitionElement for EcucParameterDef {}
impl EcucCommonAttributes for EcucParameterDef {}
impl EcucParamDef for EcucParameterDef {}

//#########################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        AutosarModelAbstraction,
        ecu_configuration::{EcucConfigurationClass, EcucConfigurationVariant},
    };
    use autosar_data::AutosarVersion;

    #[test]
    fn parameter() {
        let model = AutosarModelAbstraction::create("test.arxml", AutosarVersion::Autosar_00048);
        let pkg = model.get_or_create_package("/test").unwrap();
        let ecuc_module = pkg.create_ecuc_module_def("EcucModule").unwrap();
        let container = ecuc_module.create_param_conf_container_def("Container").unwrap();

        let addinfo = container.create_add_info_param_def("AddInfo", "origin").unwrap();
        let boolean = container.create_boolean_param_def("Boolean", "origin").unwrap();
        let enumeration = container.create_enumeration_param_def("Enumeration", "origin").unwrap();
        let float = container.create_float_param_def("Float", "origin").unwrap();
        let integer = container.create_integer_param_def("Integer", "origin").unwrap();
        let function_name = container
            .create_function_name_param_def("FunctionName", "origin")
            .unwrap();
        let linker_symbol = container
            .create_linker_symbol_param_def("LinkerSymbol", "origin")
            .unwrap();
        let multiline_string = container
            .create_multiline_string_param_def("MultilineString", "origin")
            .unwrap();
        let string = container.create_string_param_def("String", "origin").unwrap();

        let addinfo2 = EcucParameterDef::AddInfo(addinfo);
        let boolean2 = EcucParameterDef::Boolean(boolean);
        let enumeration2 = EcucParameterDef::Enumeration(enumeration);
        let float2 = EcucParameterDef::Float(float);
        let integer2 = EcucParameterDef::Integer(integer);
        let function_name2 = EcucParameterDef::FunctionName(function_name);
        let linker_symbol2 = EcucParameterDef::LinkerSymbol(linker_symbol);
        let multiline_string2 = EcucParameterDef::MultilineString(multiline_string);
        let string2 = EcucParameterDef::String(string);

        assert_eq!(addinfo2.name(), Some("AddInfo".to_string()));
        assert_eq!(boolean2.name(), Some("Boolean".to_string()));
        assert_eq!(enumeration2.name(), Some("Enumeration".to_string()));
        assert_eq!(float2.name(), Some("Float".to_string()));
        assert_eq!(integer2.name(), Some("Integer".to_string()));
        assert_eq!(function_name2.name(), Some("FunctionName".to_string()));
        assert_eq!(linker_symbol2.name(), Some("LinkerSymbol".to_string()));
        assert_eq!(multiline_string2.name(), Some("MultilineString".to_string()));
        assert_eq!(string2.name(), Some("String".to_string()));
    }

    #[test]
    fn string_parameters() {
        let model = AutosarModelAbstraction::create("test.arxml", AutosarVersion::Autosar_00048);
        let pkg = model.get_or_create_package("/test").unwrap();
        let ecuc_module = pkg.create_ecuc_module_def("EcucModule").unwrap();
        let container = ecuc_module.create_param_conf_container_def("Container").unwrap();

        let function_name = container
            .create_function_name_param_def("FunctionName", "origin")
            .unwrap();
        function_name.set_max_length(Some(10)).unwrap();
        assert_eq!(function_name.max_length(), Some(10));
        function_name.set_max_length(None).unwrap();
        assert_eq!(function_name.max_length(), None);
        function_name.set_min_length(Some(5)).unwrap();
        assert_eq!(function_name.min_length(), Some(5));
        function_name.set_min_length(None).unwrap();
        assert_eq!(function_name.min_length(), None);
        function_name.set_regular_expression(Some(r"^\d{5}$")).unwrap();
        assert_eq!(function_name.regular_expression(), Some(r"^\d{5}$".to_string()));
        function_name.set_regular_expression(None).unwrap();
        assert_eq!(function_name.regular_expression(), None);
        function_name.set_default_value(Some("12345")).unwrap();
        assert_eq!(function_name.default_value(), Some("12345".to_string()));
        function_name.set_default_value(None).unwrap();
        assert_eq!(function_name.default_value(), None);

        let linker_symbol = container
            .create_linker_symbol_param_def("LinkerSymbol", "origin")
            .unwrap();
        linker_symbol.set_max_length(Some(10)).unwrap();
        assert_eq!(linker_symbol.max_length(), Some(10));
        linker_symbol.set_max_length(None).unwrap();
        assert_eq!(linker_symbol.max_length(), None);
        linker_symbol.set_min_length(Some(5)).unwrap();
        assert_eq!(linker_symbol.min_length(), Some(5));
        linker_symbol.set_min_length(None).unwrap();
        assert_eq!(linker_symbol.min_length(), None);
        linker_symbol.set_regular_expression(Some(r"^\d{5}$")).unwrap();
        assert_eq!(linker_symbol.regular_expression(), Some(r"^\d{5}$".to_string()));
        linker_symbol.set_regular_expression(None).unwrap();
        assert_eq!(linker_symbol.regular_expression(), None);
        linker_symbol.set_default_value(Some("12345")).unwrap();
        assert_eq!(linker_symbol.default_value(), Some("12345".to_string()));
        linker_symbol.set_default_value(None).unwrap();
        assert_eq!(linker_symbol.default_value(), None);

        let multiline_string = container
            .create_multiline_string_param_def("MultilineString", "origin")
            .unwrap();
        multiline_string.set_max_length(Some(10)).unwrap();
        assert_eq!(multiline_string.max_length(), Some(10));
        multiline_string.set_max_length(None).unwrap();
        assert_eq!(multiline_string.max_length(), None);
        multiline_string.set_min_length(Some(5)).unwrap();
        assert_eq!(multiline_string.min_length(), Some(5));
        multiline_string.set_min_length(None).unwrap();
        assert_eq!(multiline_string.min_length(), None);
        multiline_string.set_regular_expression(Some(r"^\d{5}$")).unwrap();
        assert_eq!(multiline_string.regular_expression(), Some(r"^\d{5}$".to_string()));
        multiline_string.set_regular_expression(None).unwrap();
        assert_eq!(multiline_string.regular_expression(), None);
        multiline_string.set_default_value(Some("12345")).unwrap();
        assert_eq!(multiline_string.default_value(), Some("12345".to_string()));
        multiline_string.set_default_value(None).unwrap();
        assert_eq!(multiline_string.default_value(), None);

        let string = container.create_string_param_def("String", "origin").unwrap();
        string.set_max_length(Some(10)).unwrap();
        assert_eq!(string.max_length(), Some(10));
        string.set_max_length(None).unwrap();
        assert_eq!(string.max_length(), None);
        string.set_min_length(Some(5)).unwrap();
        assert_eq!(string.min_length(), Some(5));
        string.set_min_length(None).unwrap();
        assert_eq!(string.min_length(), None);
        string.set_regular_expression(Some(r"^\d{5}$")).unwrap();
        assert_eq!(string.regular_expression(), Some(r"^\d{5}$".to_string()));
        string.set_regular_expression(None).unwrap();
        assert_eq!(string.regular_expression(), None);
        string.set_default_value(Some("12345")).unwrap();
        assert_eq!(string.default_value(), Some("12345".to_string()));
        string.set_default_value(None).unwrap();
        assert_eq!(string.default_value(), None);

        // trait functions
        let mcc = [(
            EcucConfigurationClass::PreCompile,
            EcucConfigurationVariant::VariantPreCompile,
        )];
        string.set_multiplicity_config_classes(&mcc).unwrap();
        assert_eq!(string.multiplicity_config_classes(), mcc);
        string.set_origin("AUTOSAR_ECUC").unwrap();
        assert_eq!(string.origin().as_deref(), Some("AUTOSAR_ECUC"));
        string.set_post_build_variant_multiplicity(Some(true)).unwrap();
        assert_eq!(string.post_build_variant_multiplicity(), Some(true));
        string.set_post_build_variant_value(Some(true)).unwrap();
        assert_eq!(string.post_build_variant_value(), Some(true));
        string.set_requires_index(Some(false)).unwrap();
        assert_eq!(string.requires_index(), Some(false));
        let vcc = [(
            EcucConfigurationClass::PostBuild,
            EcucConfigurationVariant::VariantPostBuild,
        )];
        string.set_value_config_classes(&vcc).unwrap();
        assert_eq!(string.value_config_classes(), vcc);
        string.set_with_auto(Some(true)).unwrap();
        assert_eq!(string.with_auto(), Some(true));
    }

    #[test]
    fn boolean_parameters() {
        let model = AutosarModelAbstraction::create("test.arxml", AutosarVersion::Autosar_00048);
        let pkg = model.get_or_create_package("/test").unwrap();
        let ecuc_module = pkg.create_ecuc_module_def("EcucModule").unwrap();
        let container = ecuc_module.create_param_conf_container_def("Container").unwrap();

        let boolean = container.create_boolean_param_def("Boolean", "origin").unwrap();
        boolean.set_default_value(Some(true)).unwrap();
        assert_eq!(boolean.default_value(), Some(true));
        boolean.set_default_value(None).unwrap();
        assert_eq!(boolean.default_value(), None);
    }

    #[test]
    fn enumeration_parameters() {
        let model = AutosarModelAbstraction::create("test.arxml", AutosarVersion::Autosar_00048);
        let pkg = model.get_or_create_package("/test").unwrap();
        let ecuc_module = pkg.create_ecuc_module_def("EcucModule").unwrap();
        let container = ecuc_module.create_param_conf_container_def("Container").unwrap();

        let enumeration = container.create_enumeration_param_def("Enumeration", "origin").unwrap();
        // setting a default value before creating literals should fail
        let result = enumeration.set_default_value(Some("Literal1"));
        assert!(result.is_err());
        // create literals
        let _literal1 = enumeration.create_enumeration_literal("Literal1").unwrap();
        let _literal2 = enumeration.create_enumeration_literal("Literal2").unwrap();

        enumeration.set_default_value(Some("Literal1")).unwrap();
        assert_eq!(enumeration.default_value(), Some("Literal1".to_string()));
    }

    #[test]
    fn float_parameters() {
        let model = AutosarModelAbstraction::create("test.arxml", AutosarVersion::Autosar_00048);
        let pkg = model.get_or_create_package("/test").unwrap();
        let ecuc_module = pkg.create_ecuc_module_def("EcucModule").unwrap();
        let container = ecuc_module.create_param_conf_container_def("Container").unwrap();

        let float = container.create_float_param_def("Float", "origin").unwrap();
        float.set_default_value(Some(1.23)).unwrap();
        assert_eq!(float.default_value(), Some(1.23));
        float.set_min(Some(0.0)).unwrap();
        assert_eq!(float.min(), Some(0.0));
        float.set_max(Some(2.0)).unwrap();
        assert_eq!(float.max(), Some(2.0));
    }

    #[test]
    fn integer_parameters() {
        let model = AutosarModelAbstraction::create("test.arxml", AutosarVersion::Autosar_00048);
        let pkg = model.get_or_create_package("/test").unwrap();
        let ecuc_module = pkg.create_ecuc_module_def("EcucModule").unwrap();
        let container = ecuc_module.create_param_conf_container_def("Container").unwrap();

        let integer = container.create_integer_param_def("Integer", "origin").unwrap();
        integer.set_default_value(Some(123)).unwrap();
        assert_eq!(integer.default_value(), Some(123));
        integer.set_min(Some(0)).unwrap();
        assert_eq!(integer.min(), Some(0));
        integer.set_max(Some(200)).unwrap();
        assert_eq!(integer.max(), Some(200));
    }
}
