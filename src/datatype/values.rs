use std::str::FromStr;

use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, Element, IdentifiableAbstractionElement,
    abstraction_element,
    datatype::{ApplicationArrayElement, ApplicationPrimitiveCategory, ApplicationRecordElement, Unit},
    software_component::{ArgumentDataPrototype, ParameterDataPrototype, VariableDataPrototype},
};
use autosar_data::{ElementName, EnumItem};

//#########################################################

/// Specification of a constant that can be part of a package, i.e. it can be defined stand-alone.
/// These constant values can be referenced from value specifications.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantSpecification(Element);
abstraction_element!(ConstantSpecification, ConstantSpecification);
impl IdentifiableAbstractionElement for ConstantSpecification {}

impl ConstantSpecification {
    pub(crate) fn new(
        name: &str,
        package: &ArPackage,
        value: ValueSpecification,
    ) -> Result<ConstantSpecification, AutosarAbstractionError> {
        let value: ValueSpecification = value;
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;

        let const_spec_elem = elements
            .create_named_sub_element(ElementName::ConstantSpecification, name)
            .unwrap();
        let const_spec = Self(const_spec_elem);
        // let const_spec = Self(elements.create_named_sub_element(ElementName::ConstantSpecification, name)?);
        const_spec.set_value_specification(value)?;

        Ok(const_spec)
    }

    /// set the value specification of the constant
    pub fn set_value_specification(&self, value: ValueSpecification) -> Result<(), AutosarAbstractionError> {
        // remove the existing value
        let _ = self.element().remove_sub_element_kind(ElementName::ValueSpec);
        let value_spec_elem = self.element().create_sub_element(ElementName::ValueSpec)?;
        value.store(&value_spec_elem)?;
        Ok(())
    }

    /// get the value specification of the constant
    #[must_use]
    pub fn value_specification(&self) -> Option<ValueSpecification> {
        let spec_elem = self
            .element()
            .get_sub_element(ElementName::ValueSpec)
            .and_then(|vs_elem| vs_elem.get_sub_element_at(0))?;
        ValueSpecification::load(&spec_elem)
    }
}

//#########################################################

/// array of values
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayValueSpecification {
    /// SHORT-LABEL: used to identify the array in a human readable way. This is used when the array is part of a record.
    pub label: Option<String>,
    /// the values of the array
    pub values: Vec<ValueSpecification>,
}

impl ArrayValueSpecification {
    fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        let array_elem = parent.create_sub_element(ElementName::ArrayValueSpecification)?;
        store_label(&array_elem, &self.label)?;
        let elements_elem = array_elem.create_sub_element(ElementName::Elements)?;
        for value in &self.values {
            value.store(&elements_elem)?;
        }
        Ok(())
    }

    fn load(element: &Element) -> Option<Self> {
        let label = load_label(element);
        let elements_elem = element.get_sub_element(ElementName::Elements)?;
        let values = elements_elem
            .sub_elements()
            .filter_map(|elem| ValueSpecification::load(&elem))
            .collect::<Vec<_>>();

        Some(Self { label, values })
    }
}

impl From<ArrayValueSpecification> for ValueSpecification {
    fn from(value_spec: ArrayValueSpecification) -> Self {
        ValueSpecification::Array(value_spec)
    }
}

//#########################################################

/// record of values. The values may be named using short-labels, but these are not mandatory.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordValueSpecification {
    /// SHORT-LABEL: used to identify the record in a human readable way. This is used when the record is part of a record.
    pub label: Option<String>,
    /// the values of the record
    /// The values may be named using short-labels, but these are not mandatory.
    pub values: Vec<ValueSpecification>,
}

impl RecordValueSpecification {
    fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        let record_elem = parent.create_sub_element(ElementName::RecordValueSpecification)?;
        store_label(&record_elem, &self.label)?;
        let fields_elem = record_elem.create_sub_element(ElementName::Fields)?;
        for value in &self.values {
            value.store(&fields_elem)?;
        }
        Ok(())
    }

    fn load(element: &Element) -> Option<Self> {
        let label = load_label(element);
        let fields_elem = element.get_sub_element(ElementName::Fields)?;
        let values = fields_elem
            .sub_elements()
            .filter_map(|elem| ValueSpecification::load(&elem))
            .collect::<Vec<_>>();

        Some(Self { label, values })
    }
}

impl From<RecordValueSpecification> for ValueSpecification {
    fn from(value_spec: RecordValueSpecification) -> Self {
        ValueSpecification::Record(value_spec)
    }
}

//#########################################################

/// textual value
#[derive(Debug, Clone, PartialEq)]
pub struct TextValueSpecification {
    /// SHORT-LABEL: used to identify the text in a human readable way. This is used when the text is part of a record.
    pub label: Option<String>,
    /// the text value
    pub value: String,
}

impl TextValueSpecification {
    fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        let text_elem = parent.create_sub_element(ElementName::TextValueSpecification)?;
        store_label(&text_elem, &self.label)?;
        let value_elem = text_elem.create_sub_element(ElementName::Value)?;
        value_elem.set_character_data(self.value.clone())?;
        Ok(())
    }

    fn load(element: &Element) -> Option<Self> {
        let label = load_label(element);
        let value = element
            .get_sub_element(ElementName::Value)?
            .character_data()?
            .string_value()?;

        Some(Self { label, value })
    }
}

impl From<TextValueSpecification> for ValueSpecification {
    fn from(value_spec: TextValueSpecification) -> Self {
        ValueSpecification::Text(value_spec)
    }
}

//#########################################################

/// numerical value
#[derive(Debug, Clone, PartialEq)]
pub struct NumericalValueSpecification {
    /// SHORT-LABEL: used to identify the number in a human readable way. This is used when the number is part of a record.
    pub label: Option<String>,
    /// the number value
    pub value: f64,
}

impl NumericalValueSpecification {
    fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        let num_elem = parent.create_sub_element(ElementName::NumericalValueSpecification)?;
        store_label(&num_elem, &self.label)?;
        let value_elem = num_elem.create_sub_element(ElementName::Value)?;
        value_elem.set_character_data(self.value)?;
        Ok(())
    }

    fn load(element: &Element) -> Option<Self> {
        let label = load_label(element);
        let value = element
            .get_sub_element(ElementName::Value)?
            .character_data()?
            .parse_float()?;

        Some(Self { label, value })
    }
}

impl From<NumericalValueSpecification> for ValueSpecification {
    fn from(value_spec: NumericalValueSpecification) -> Self {
        ValueSpecification::Numerical(value_spec)
    }
}

//#########################################################

/// reference to a `ConstantValue`
#[derive(Debug, Clone, PartialEq)]
pub struct ConstantReference {
    /// SHORT-LABEL: used to identify the constant in a human readable way. This is used when the constant is part of a record.
    pub label: Option<String>,
    /// Reference to the constant specification
    pub constant: ConstantSpecification,
}

impl ConstantReference {
    fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        let const_ref_elem = parent.create_sub_element(ElementName::ConstantReference)?;
        store_label(&const_ref_elem, &self.label)?;
        const_ref_elem
            .create_sub_element(ElementName::ConstantRef)
            .and_then(|cr_elem| cr_elem.set_reference_target(self.constant.element()))?;
        Ok(())
    }

    fn load(element: &Element) -> Option<Self> {
        let label = load_label(element);
        let constant_elem = element
            .get_sub_element(ElementName::ConstantRef)?
            .get_reference_target()
            .ok()?;
        let constant = ConstantSpecification::try_from(constant_elem).ok()?;

        Some(Self { label, constant })
    }
}

impl From<ConstantReference> for ValueSpecification {
    fn from(value_spec: ConstantReference) -> Self {
        ValueSpecification::ConstantReference(value_spec)
    }
}

//#########################################################

/// Application value
#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationValueSpecification {
    /// SHORT-LABEL: used to identify the application value in a human readable way. This is used when the application value is part of a record.
    pub label: Option<String>,
    /// category of the application value
    pub category: ApplicationPrimitiveCategory,
    /// axis values of a compound primitive data type. Required for categories `ResAxis`, Cure, Map, Cuboid, Cube4, Cube5
    pub sw_axis_conts: Vec<SwAxisCont>,
    /// values of a compound primitive data type
    pub sw_value_cont: SwValueCont,
}

impl ApplicationValueSpecification {
    fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        let app_elem = parent.create_sub_element(ElementName::ApplicationValueSpecification)?;
        store_label(&app_elem, &self.label)?;
        let category_elem = app_elem.create_sub_element(ElementName::Category)?;
        category_elem.set_character_data(self.category.to_string())?;
        let sw_axis_conts_elem = app_elem.create_sub_element(ElementName::SwAxisConts)?;
        for sw_axis in &self.sw_axis_conts {
            sw_axis.store(&sw_axis_conts_elem)?;
        }
        let sw_value_cont_elem = app_elem.create_sub_element(ElementName::SwValueCont)?;
        self.sw_value_cont.store(&sw_value_cont_elem)?;

        Ok(())
    }

    fn load(element: &Element) -> Option<Self> {
        let label = load_label(element);
        let category_string = element
            .get_sub_element(ElementName::Category)?
            .character_data()?
            .string_value()?;
        let category = ApplicationPrimitiveCategory::from_str(&category_string).ok()?;
        let sw_axis_conts_elem = element.get_sub_element(ElementName::SwAxisConts)?;
        let sw_axis_conts = sw_axis_conts_elem
            .sub_elements()
            .filter_map(|elem| SwAxisCont::load(&elem))
            .collect::<Vec<_>>();
        let sw_value_cont_elem = element.get_sub_element(ElementName::SwValueCont)?;
        let sw_value_cont = SwValueCont::load(&sw_value_cont_elem)?;

        Some(Self {
            label,
            category,
            sw_axis_conts,
            sw_value_cont,
        })
    }
}

impl From<ApplicationValueSpecification> for ValueSpecification {
    fn from(value_spec: ApplicationValueSpecification) -> Self {
        ValueSpecification::Application(value_spec)
    }
}

//#########################################################

/// Default init pattern, which is used when an optional `ApplicationRecordElement` in not available
#[derive(Debug, Clone, PartialEq)]
pub struct NotAvailableValueSpecification {
    /// SHORT-LABEL: used to identify the default pattern in a human readable way. This is used when the default pattern is part of a record.
    pub label: Option<String>,
    /// initialization pattern for memory occupied by unavailable application record elements; available in `AUTOSAR_00049` and newer
    pub default_pattern: Option<u64>, // presumably this could be u8 to initialize bytes in memory. But the spec only says it's a positive integer
}

impl NotAvailableValueSpecification {
    fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        let not_available_elem = parent.create_sub_element(ElementName::NotAvailableValueSpecification)?;
        store_label(&not_available_elem, &self.label)?;
        if let Some(pattern) = &self.default_pattern {
            // try to create the pattern element; it is not available in older versions of AUTOSAR
            if let Ok(pattern_elem) = not_available_elem.create_sub_element(ElementName::DefaultPattern) {
                pattern_elem.set_character_data(pattern.to_string())?;
            }
        }
        Ok(())
    }

    fn load(element: &Element) -> Option<Self> {
        let label = load_label(element);
        let default_pattern = element
            .get_sub_element(ElementName::DefaultPattern)
            .and_then(|dp_elem| dp_elem.character_data())
            .and_then(|cdata| cdata.parse_integer());
        Some(Self { label, default_pattern })
    }
}

impl From<NotAvailableValueSpecification> for ValueSpecification {
    fn from(value_spec: NotAvailableValueSpecification) -> Self {
        ValueSpecification::NotAvailable(value_spec)
    }
}

//#########################################################

/// reference to a `DataPrototype`, to be used as a pointer in the software
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceValueSpecification {
    /// SHORT-LABEL: used to identify the reference in a human readable way. This is used when the reference is part of a record.
    pub label: Option<String>,
    /// data prototype that will be referenced as a pointer in the software
    pub reference_value: DataPrototype,
}

impl ReferenceValueSpecification {
    fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        let ref_value_elem = parent.create_sub_element(ElementName::ReferenceValueSpecification)?;
        store_label(&ref_value_elem, &self.label)?;
        ref_value_elem
            .create_sub_element(ElementName::ReferenceValueRef)
            .and_then(|rvr_elem| rvr_elem.set_reference_target(self.reference_value.element()))?;
        Ok(())
    }

    fn load(element: &Element) -> Option<Self> {
        let label = load_label(element);
        let reference_value_elem = element
            .get_sub_element(ElementName::ReferenceValueRef)?
            .get_reference_target()
            .ok()?;
        let reference_value = DataPrototype::try_from(reference_value_elem).ok()?;

        Some(Self { label, reference_value })
    }
}

impl From<ReferenceValueSpecification> for ValueSpecification {
    fn from(value_spec: ReferenceValueSpecification) -> Self {
        ValueSpecification::Reference(value_spec)
    }
}

//#########################################################

/// A rule to generate application values for an array value specification
#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationRuleBasedValueSpecification {
    /// SHORT-LABEL: used to identify the application value in a human readable way. This is used when the application value is part of a record.
    pub label: Option<String>,
    /// category of the application value
    pub category: ApplicationPrimitiveCategory,
    /// rule-based axis values of a compound primitive data type. Required for categories `ResAxis`, Cure, Map, Cuboid, Cube4, Cube5
    pub sw_axis_cont: Vec<RuleBasedAxisCont>,
    /// rule-based values of a compound primitive data type
    pub sw_value_cont: RuleBasedValueCont,
}

impl ApplicationRuleBasedValueSpecification {
    fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        let app_rule_elem = parent.create_sub_element(ElementName::ApplicationRuleBasedValueSpecification)?;
        store_label(&app_rule_elem, &self.label)?;
        let category_elem = app_rule_elem.create_sub_element(ElementName::Category)?;
        category_elem.set_character_data(self.category.to_string())?;
        let sw_axis_cont_elem = app_rule_elem.create_sub_element(ElementName::SwAxisConts)?;
        for sw_axis in &self.sw_axis_cont {
            sw_axis.store(&sw_axis_cont_elem)?;
        }
        self.sw_value_cont.store(&app_rule_elem)?;

        Ok(())
    }

    fn load(element: &Element) -> Option<Self> {
        let label = load_label(element);

        let category_string = element
            .get_sub_element(ElementName::Category)?
            .character_data()?
            .string_value()?;
        let category = ApplicationPrimitiveCategory::from_str(&category_string).ok()?;

        let sw_axis_cont_elem = element.get_sub_element(ElementName::SwAxisConts)?;
        let sw_axis_cont = sw_axis_cont_elem
            .sub_elements()
            .filter_map(|elem| RuleBasedAxisCont::load(&elem))
            .collect::<Vec<_>>();

        let sw_value_cont_elem = element.get_sub_element(ElementName::SwValueCont)?;
        let sw_value_cont = RuleBasedValueCont::load(&sw_value_cont_elem)?;

        Some(Self {
            label,
            category,
            sw_axis_cont,
            sw_value_cont,
        })
    }
}

impl From<ApplicationRuleBasedValueSpecification> for ValueSpecification {
    fn from(value_spec: ApplicationRuleBasedValueSpecification) -> Self {
        ValueSpecification::ApplicationRuleBased(value_spec)
    }
}

//#########################################################

/// A rule to generate composite values for an array value specification
#[derive(Debug, Clone, PartialEq)]
pub struct CompositeRuleBasedValueSpecification {
    /// SHORT-LABEL: used to identify the composite value in a human readable way. This is used when the composite value is part of a record.
    pub label: Option<String>,
    /// collection of specified compound values. The last value is used by the filling rule to fill the array
    pub argument: Vec<CompositeValueSpecification>,
    /// collection of specified primitive values. The last value is used by the filling rule to fill the array
    pub compound_primitive_argument: Vec<CompositeRuleBasedValueArgument>,
    /// maximum size of the array to fill. It is used if the filling rule is set to `FILL_UNTIL_MAX_SIZE`
    pub max_size_to_fill: Option<u64>,
    /// rule to fill the array
    pub rule: RuleBasedFillUntil,
}

impl CompositeRuleBasedValueSpecification {
    fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        let comp_rule_elem = parent.create_sub_element(ElementName::CompositeRuleBasedValueSpecification)?;
        store_label(&comp_rule_elem, &self.label)?;
        let arguments_elem = comp_rule_elem.create_sub_element(ElementName::Arguments)?;
        for arg in &self.argument {
            arg.store(&arguments_elem)?;
        }
        let compound_primitive_arguments_elem =
            comp_rule_elem.create_sub_element(ElementName::CompoundPrimitiveArguments)?;
        for arg in &self.compound_primitive_argument {
            arg.store(&compound_primitive_arguments_elem)?;
        }
        if let Some(max_size) = self.max_size_to_fill {
            let max_size_elem = comp_rule_elem.create_sub_element(ElementName::MaxSizeToFill)?;
            max_size_elem.set_character_data(max_size.to_string())?;
        }
        let rule_elem = comp_rule_elem.create_sub_element(ElementName::Rule)?;
        rule_elem.set_character_data(self.rule.to_string())?;

        Ok(())
    }

    fn load(element: &Element) -> Option<Self> {
        let label = load_label(element);
        let arguments_elem = element.get_sub_element(ElementName::Arguments)?;
        let argument = arguments_elem
            .sub_elements()
            .filter_map(|elem| CompositeValueSpecification::load(&elem))
            .collect::<Vec<_>>();
        let compound_primitive_arguments_elem = element.get_sub_element(ElementName::CompoundPrimitiveArguments)?;
        let compound_primitive_argument = compound_primitive_arguments_elem
            .sub_elements()
            .filter_map(|elem| CompositeRuleBasedValueArgument::load(&elem))
            .collect::<Vec<_>>();
        let max_size_to_fill = element
            .get_sub_element(ElementName::MaxSizeToFill)
            .and_then(|ms_elem| ms_elem.character_data())
            .and_then(|cdata| cdata.parse_integer());
        let rule_string = element
            .get_sub_element(ElementName::Rule)?
            .character_data()?
            .string_value()?;
        let rule = RuleBasedFillUntil::from_str(&rule_string).ok()?;

        Some(Self {
            label,
            argument,
            compound_primitive_argument,
            max_size_to_fill,
            rule,
        })
    }
}

impl From<CompositeRuleBasedValueSpecification> for ValueSpecification {
    fn from(value_spec: CompositeRuleBasedValueSpecification) -> Self {
        ValueSpecification::CompositeRuleBased(value_spec)
    }
}

//#########################################################

/// A rule to generate numerical values for an array value specification
#[derive(Debug, Clone, PartialEq)]
pub struct NumericalRuleBasedValueSpecification {
    /// SHORT-LABEL: used to identify the numerical value in a human readable way. This is used when the numerical value is part of a record.
    pub label: Option<String>,
    /// rule-based values for the array
    pub rule_based_values: RuleBasedValueSpecification,
}

impl NumericalRuleBasedValueSpecification {
    fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        let num_rule_elem = parent.create_sub_element(ElementName::NumericalRuleBasedValueSpecification)?;
        store_label(&num_rule_elem, &self.label)?;
        self.rule_based_values.store(&num_rule_elem)?;

        Ok(())
    }

    fn load(element: &Element) -> Option<Self> {
        let label = load_label(element);
        let rule_based_values_elem = element.get_sub_element(ElementName::RuleBasedValues)?;
        let rule_based_values = RuleBasedValueSpecification::load(&rule_based_values_elem)?;

        Some(Self {
            label,
            rule_based_values,
        })
    }
}

impl From<NumericalRuleBasedValueSpecification> for ValueSpecification {
    fn from(value_spec: NumericalRuleBasedValueSpecification) -> Self {
        ValueSpecification::NumericalRuleBased(value_spec)
    }
}

//#########################################################

#[derive(Debug, Clone, PartialEq)]
/// Specification of a value. It is used for constants, signal init values and port init values.
pub enum ValueSpecification {
    /// array of values
    Array(ArrayValueSpecification),
    /// record of values. The values may be named using short-labels, but these are not mandatory.
    Record(RecordValueSpecification),
    /// textual value
    Text(TextValueSpecification),
    /// numerical value
    Numerical(NumericalValueSpecification),
    /// reference to a `ConstantValue`
    ConstantReference(ConstantReference),
    /// Application value
    Application(ApplicationValueSpecification),
    /// Default init pattern, which is used when an optional `ApplicationRecordElement` in not available
    NotAvailable(NotAvailableValueSpecification),
    /// reference to a `DataPrototype`, to be used as a pointer in the software
    Reference(ReferenceValueSpecification),
    /// A rule to generate application values for an array value specification
    ApplicationRuleBased(ApplicationRuleBasedValueSpecification),
    /// A rule to generate composite values for an array value specification
    CompositeRuleBased(CompositeRuleBasedValueSpecification),
    /// A rule to generate numerical values for an array value specification
    NumericalRuleBased(NumericalRuleBasedValueSpecification),
}

impl ValueSpecification {
    pub(crate) fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        match self {
            Self::Array(array_spec) => array_spec.store(parent),
            Self::Record(record_spec) => record_spec.store(parent),
            Self::Text(text_spec) => text_spec.store(parent),
            Self::Numerical(num_spec) => num_spec.store(parent),
            Self::ConstantReference(constant_ref) => constant_ref.store(parent),
            Self::Application(app_spec) => app_spec.store(parent),
            Self::NotAvailable(not_available_spec) => not_available_spec.store(parent),
            Self::Reference(ref_value_spec) => ref_value_spec.store(parent),
            Self::ApplicationRuleBased(app_rule_spec) => app_rule_spec.store(parent),
            Self::CompositeRuleBased(comp_rule_spec) => comp_rule_spec.store(parent),
            Self::NumericalRuleBased(num_rule_spec) => num_rule_spec.store(parent),
        }
    }

    pub(crate) fn load(elem: &Element) -> Option<ValueSpecification> {
        match elem.element_name() {
            ElementName::ArrayValueSpecification => ArrayValueSpecification::load(elem).map(Self::Array),
            ElementName::RecordValueSpecification => RecordValueSpecification::load(elem).map(Self::Record),
            ElementName::TextValueSpecification => TextValueSpecification::load(elem).map(Self::Text),
            ElementName::NumericalValueSpecification => NumericalValueSpecification::load(elem).map(Self::Numerical),
            ElementName::ConstantReference => ConstantReference::load(elem).map(Self::ConstantReference),
            ElementName::ApplicationValueSpecification => {
                ApplicationValueSpecification::load(elem).map(Self::Application)
            }
            ElementName::NotAvailableValueSpecification => {
                NotAvailableValueSpecification::load(elem).map(Self::NotAvailable)
            }
            ElementName::ReferenceValueSpecification => ReferenceValueSpecification::load(elem).map(Self::Reference),
            ElementName::ApplicationRuleBasedValueSpecification => {
                ApplicationRuleBasedValueSpecification::load(elem).map(Self::ApplicationRuleBased)
            }
            ElementName::CompositeRuleBasedValueSpecification => {
                CompositeRuleBasedValueSpecification::load(elem).map(Self::CompositeRuleBased)
            }
            ElementName::NumericalRuleBasedValueSpecification => {
                NumericalRuleBasedValueSpecification::load(elem).map(Self::NumericalRuleBased)
            }
            _ => None,
        }
    }
}

fn store_label(parent: &Element, label: &Option<String>) -> Result<(), AutosarAbstractionError> {
    if let Some(label) = label {
        let label_elem = parent.create_sub_element(ElementName::ShortLabel)?;
        label_elem.set_character_data(label.clone())?;
    }
    Ok(())
}

fn load_label(element: &Element) -> Option<String> {
    let label_elem = element.get_sub_element(ElementName::ShortLabel)?;
    label_elem.character_data()?.string_value()
}

//#########################################################

/// standard fill rules for rule based value specifications
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleBasedFillUntil {
    /// `FILL_UNTIL_END`: fills the value of the last RuleBasedValueSpecification.arguments
    /// until the last element of the array has been filled
    End,
    /// `FILL_UNTIL_MAX_SIZE`: fills the value of the last RuleBasedValueSpecification.arguments
    /// until maxSizeToFill elements of the array have been filled
    MaxSize,
}

impl FromStr for RuleBasedFillUntil {
    type Err = AutosarAbstractionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "FILL_UNTIL_END" => Ok(RuleBasedFillUntil::End),
            "FILL_UNTIL_MAX_SIZE" => Ok(RuleBasedFillUntil::MaxSize),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "RuleBasedFillUntil".to_string(),
            }),
        }
    }
}

impl std::fmt::Display for RuleBasedFillUntil {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleBasedFillUntil::End => f.write_str("FILL_UNTIL_END"),
            RuleBasedFillUntil::MaxSize => f.write_str("FILL_UNTIL_MAX_SIZE"),
        }
    }
}

//#########################################################

/// specification of the axis values of a compound primitive data type (curve, map)
#[derive(Debug, Clone, PartialEq)]
pub struct SwAxisCont {
    /// category of the axis; one of `STD_AXIS`, `COM_AXIS`, `COM_AXIS`, `RES_AXIS`
    pub category: SwAxisContCategory,
    /// dimensions of the axis, used if the category is `RES_AXIS`, otherwise it should be empty
    pub sw_array_size: Vec<u64>,
    /// index of the axis. Here 1 is the x axis, 2 is the y axis, ...
    pub sw_axis_index: u64,
    /// axis values in the physical domain
    pub sw_values_phys: Vec<SwValue>,
    /// pyhsical unit of the axis values
    pub unit: Option<Unit>,
    /// display name of the unit
    pub unit_display_name: Option<String>,
}

impl SwAxisCont {
    fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        let sw_axis_cont_elem = parent.create_sub_element(ElementName::SwAxisCont)?;
        let category_elem = sw_axis_cont_elem.create_sub_element(ElementName::Category)?;
        category_elem.set_character_data::<EnumItem>(self.category.into())?;
        let sw_array_size_elem = sw_axis_cont_elem.create_sub_element(ElementName::SwArraysize)?;
        for size in &self.sw_array_size {
            let size_elem = sw_array_size_elem.create_sub_element(ElementName::Vf)?;
            size_elem.set_character_data(*size)?;
        }
        let sw_axis_index_elem = sw_axis_cont_elem.create_sub_element(ElementName::SwAxisIndex)?;
        sw_axis_index_elem.set_character_data(self.sw_axis_index.to_string())?;
        let sw_values_phys_elem = sw_axis_cont_elem.create_sub_element(ElementName::SwValuesPhys)?;
        for value in &self.sw_values_phys {
            value.store(&sw_values_phys_elem)?;
        }

        if let Some(unit) = &self.unit {
            sw_axis_cont_elem
                .create_sub_element(ElementName::UnitRef)
                .and_then(|unit_elem| unit_elem.set_reference_target(unit.element()))?;
        }
        if let Some(unit_display_name) = &self.unit_display_name {
            // try to create the UnitDisplayName element; it is not available in older versions of AUTOSAR, so errors are ignored
            let _ = sw_axis_cont_elem
                .create_sub_element(ElementName::UnitDisplayName)
                .and_then(|udn_elem| udn_elem.set_character_data(unit_display_name.clone()));
        }

        Ok(())
    }

    fn load(element: &Element) -> Option<Self> {
        let category_elem = element.get_sub_element(ElementName::Category)?;
        let category = SwAxisContCategory::try_from(category_elem.character_data()?.enum_value()?).ok()?;
        let sw_array_size_elem = element.get_sub_element(ElementName::SwArraysize)?;
        // The SW-ARRAY-SIZE element can contain either one V or many VF elements
        // The iterator doesn't care, and parse_integer works regardless
        let sw_array_size = sw_array_size_elem
            .sub_elements()
            .filter_map(|elem| elem.character_data()?.parse_integer())
            .collect::<Vec<_>>();

        let sw_axis_index_elem = element.get_sub_element(ElementName::SwAxisIndex)?;
        let sw_axis_index = sw_axis_index_elem.character_data()?.parse_integer()?;

        let sw_values_phys_elem = element.get_sub_element(ElementName::SwValuesPhys)?;
        let sw_values_phys = sw_values_phys_elem
            .sub_elements()
            .filter_map(|elem| SwValue::load(&elem))
            .collect::<Vec<_>>();

        let unit = element
            .get_sub_element(ElementName::UnitRef)
            .and_then(|unit_elem| unit_elem.get_reference_target().ok())
            .and_then(|unit_elem| Unit::try_from(unit_elem).ok());
        let unit_display_name = element
            .get_sub_element(ElementName::UnitDisplayName)
            .and_then(|udn_elem| udn_elem.character_data())
            .and_then(|cdata| cdata.string_value());

        Some(Self {
            category,
            sw_array_size,
            sw_axis_index,
            sw_values_phys,
            unit,
            unit_display_name,
        })
    }
}

//#########################################################

/// enumeration of the axis categories.
/// This is a restricted version of the `CalprmAxisCategoryEnum`: `FixAxis` is not permitted in `SwAxisCont`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwAxisContCategory {
    /// standard axis
    StdAxis,
    /// commmon axis
    ComAxis,
    /// rescale axis
    ResAxis,
}

impl TryFrom<EnumItem> for SwAxisContCategory {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<Self, Self::Error> {
        match value {
            EnumItem::StdAxis => Ok(SwAxisContCategory::StdAxis), // old form: STD-AXIS
            EnumItem::Stdaxis => Ok(SwAxisContCategory::StdAxis), // new form: STD_AXIS
            EnumItem::ComAxis => Ok(SwAxisContCategory::ComAxis), // old form: COM-AXIS
            EnumItem::Comaxis => Ok(SwAxisContCategory::ComAxis), // new form: COM_AXIS
            EnumItem::ResAxis => Ok(SwAxisContCategory::ResAxis), // old form: RES-AXIS
            EnumItem::Resaxis => Ok(SwAxisContCategory::ResAxis), // new form: RES_AXIS
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "SwAxisContCategory".to_string(),
            }),
        }
    }
}

impl From<SwAxisContCategory> for EnumItem {
    fn from(value: SwAxisContCategory) -> Self {
        match value {
            SwAxisContCategory::StdAxis => EnumItem::Stdaxis,
            SwAxisContCategory::ComAxis => EnumItem::Comaxis,
            SwAxisContCategory::ResAxis => EnumItem::Resaxis,
        }
    }
}

//#########################################################

/// specification of the values of a compound primitive data type (curve, map)
#[derive(Debug, Clone, PartialEq)]
pub struct SwValueCont {
    /// dimensions of the array
    pub sw_array_size: Vec<u64>,
    /// values in the physical domain
    pub sw_values_phys: Vec<SwValue>,
}

impl SwValueCont {
    fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        let sw_array_size_elem = parent.create_sub_element(ElementName::SwArraysize)?;
        for size in &self.sw_array_size {
            let size_elem = sw_array_size_elem.create_sub_element(ElementName::Vf)?;
            size_elem.set_character_data(*size)?;
        }

        let sw_values_phys_elem = parent.create_sub_element(ElementName::SwValuesPhys)?;
        for value in &self.sw_values_phys {
            value.store(&sw_values_phys_elem)?;
        }
        Ok(())
    }

    fn load(element: &Element) -> Option<Self> {
        let sw_array_size_elem = element.get_sub_element(ElementName::SwArraysize)?;
        let sw_array_size = sw_array_size_elem
            .sub_elements()
            .filter_map(|elem| elem.character_data()?.parse_integer())
            .collect::<Vec<_>>();

        let sw_values_phys_elem = element.get_sub_element(ElementName::SwValuesPhys)?;
        let sw_values_phys = sw_values_phys_elem
            .sub_elements()
            .filter_map(|elem| SwValue::load(&elem))
            .collect::<Vec<_>>();

        Some(Self {
            sw_array_size,
            sw_values_phys,
        })
    }
}

//#########################################################

/// a single value of a compound primitive data type (curve, map)
#[derive(Debug, Clone, PartialEq)]
pub enum SwValue {
    /// numerical value
    V(f64),
    /// numerical value
    Vf(f64),
    /// value group
    Vg {
        /// label of the value group
        label: Option<String>,
        /// content of the value group
        vg_content: Vec<SwValue>,
    },
    /// textual value
    Vt(String),
    /// Vtf element with numerical value
    VtfNumber(f64),
    /// Vtf element with textual value
    VtfText(String),
}

impl SwValue {
    fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        match self {
            SwValue::V(value) => {
                let value_elem = parent.create_sub_element(ElementName::V)?;
                value_elem.set_character_data(value.to_string())?;
            }
            SwValue::Vf(value) => {
                let value_elem = parent.create_sub_element(ElementName::Vf)?;
                value_elem.set_character_data(value.to_string())?;
            }
            SwValue::Vg { label, vg_content } => {
                let value_group_elem = parent.create_sub_element(ElementName::Vg)?;
                if let Some(label) = label {
                    let label_elem = value_group_elem
                        .create_sub_element(ElementName::Label)?
                        .create_sub_element(ElementName::L4)?;
                    label_elem.set_character_data(label.clone())?;
                }
                for value in vg_content {
                    value.store(&value_group_elem)?;
                }
            }
            SwValue::Vt(value) => {
                let value_elem = parent.create_sub_element(ElementName::Vt)?;
                value_elem.set_character_data(value.clone())?;
            }
            SwValue::VtfNumber(value) => {
                let value_elem = parent
                    .create_sub_element(ElementName::Vtf)?
                    .create_sub_element(ElementName::Vf)?;
                value_elem.set_character_data(*value)?;
            }
            SwValue::VtfText(value) => {
                let value_elem = parent
                    .create_sub_element(ElementName::Vtf)?
                    .create_sub_element(ElementName::Vt)?;
                value_elem.set_character_data(value.clone())?;
            }
        }
        Ok(())
    }

    fn load(element: &Element) -> Option<Self> {
        let value = match element.element_name() {
            ElementName::V => {
                let value = element.character_data()?.parse_float()?;
                SwValue::V(value)
            }
            ElementName::Vf => {
                let value = element.character_data()?.parse_float()?;
                SwValue::Vf(value)
            }
            ElementName::Vg => {
                let label = element
                    .get_sub_element(ElementName::Label)
                    .and_then(|label_elem| label_elem.get_sub_element(ElementName::L4))
                    .and_then(|sl_elem| sl_elem.character_data())
                    .and_then(|cdata| cdata.string_value());
                let vg_content = element
                    .sub_elements()
                    .filter_map(|elem| SwValue::load(&elem))
                    .collect::<Vec<_>>();
                SwValue::Vg { label, vg_content }
            }
            ElementName::Vt => {
                let value = element.character_data()?.string_value()?;
                SwValue::Vt(value)
            }
            ElementName::Vtf => {
                // The VTF element can contain either a Vf or a Vt element
                if let Some(vf) = element.get_sub_element(ElementName::Vf) {
                    SwValue::VtfNumber(vf.character_data()?.parse_float()?)
                } else if let Some(vt) = element.get_sub_element(ElementName::Vt) {
                    SwValue::VtfText(vt.character_data()?.string_value()?)
                } else {
                    return None;
                }
            }
            _ => return None,
        };
        Some(value)
    }
}

//#########################################################

/// specification of the axis values of a compound primitive data type (curve, map) in a rule-based definition
#[derive(Debug, Clone, PartialEq)]
pub struct RuleBasedAxisCont {
    /// category of the axis; one of `STD_AXIS`, `COM_AXIS`, `COM_AXIS`, `RES_AXIS`
    pub category: SwAxisContCategory,
    /// dimensions of the axis, used if the category is `RES_AXIS`, otherwise it should be empty
    pub sw_array_size: Vec<u64>,
    /// index of the axis. Here 1 is the x axis, 2 is the y axis, ...
    pub sw_axis_index: u64,
    /// axis values in the physical domain
    pub rule_based_values: RuleBasedValueSpecification,
    /// pyhsical unit of the axis values
    pub unit: Option<Unit>,
}

impl RuleBasedAxisCont {
    fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        let axis_cont_elem = parent.create_sub_element(ElementName::RuleBasedAxisCont)?;
        let category_elem = axis_cont_elem.create_sub_element(ElementName::Category)?;
        category_elem.set_character_data::<EnumItem>(self.category.into())?;
        let sw_array_size_elem = axis_cont_elem.create_sub_element(ElementName::SwArraysize)?;
        for size in &self.sw_array_size {
            let size_elem = sw_array_size_elem.create_sub_element(ElementName::Vf)?;
            size_elem.set_character_data(*size)?;
        }
        let sw_axis_index_elem = axis_cont_elem.create_sub_element(ElementName::SwAxisIndex)?;
        sw_axis_index_elem.set_character_data(self.sw_axis_index.to_string())?;
        self.rule_based_values.store(&axis_cont_elem)?;

        if let Some(unit) = &self.unit {
            axis_cont_elem
                .create_sub_element(ElementName::UnitRef)
                .and_then(|unit_elem| unit_elem.set_reference_target(unit.element()))?;
        }

        Ok(())
    }

    fn load(element: &Element) -> Option<Self> {
        let category_elem = element.get_sub_element(ElementName::Category)?;
        let category = SwAxisContCategory::try_from(category_elem.character_data()?.enum_value()?).ok()?;
        let sw_array_size_elem = element.get_sub_element(ElementName::SwArraysize)?;
        // The SW-ARRAY-SIZE element can contain either one V or many VF elements
        // The iterator doesn't care, and parse_integer works regardless
        let sw_array_size = sw_array_size_elem
            .sub_elements()
            .filter_map(|elem| elem.character_data()?.parse_integer())
            .collect::<Vec<_>>();

        let sw_axis_index_elem = element.get_sub_element(ElementName::SwAxisIndex)?;
        let sw_axis_index = sw_axis_index_elem.character_data()?.parse_integer()?;

        let rule_based_values_elem = element.get_sub_element(ElementName::RuleBasedValues)?;
        let rule_based_values = RuleBasedValueSpecification::load(&rule_based_values_elem)?;

        let unit = element
            .get_sub_element(ElementName::UnitRef)
            .and_then(|unit_elem| unit_elem.get_reference_target().ok())
            .and_then(|unit_elem| Unit::try_from(unit_elem).ok());

        Some(Self {
            category,
            sw_array_size,
            sw_axis_index,
            rule_based_values,
            unit,
        })
    }
}

//#########################################################

/// specification of the values of a compound primitive data type (curve, map) in a rule-based definition
#[derive(Debug, Clone, PartialEq)]
pub struct RuleBasedValueCont {
    /// values
    pub rule_based_values: RuleBasedValueSpecification,
    /// dimensions of the array
    pub sw_array_size: Vec<u64>,
    /// physical unit of the values
    pub unit: Option<Unit>,
}

impl RuleBasedValueCont {
    fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        let sw_value_cont_elem = parent.create_sub_element(ElementName::SwValueCont)?;
        self.rule_based_values.store(&sw_value_cont_elem)?;
        let sw_array_size_elem = sw_value_cont_elem.create_sub_element(ElementName::SwArraysize)?;
        for size in &self.sw_array_size {
            let size_elem = sw_array_size_elem.create_sub_element(ElementName::Vf)?;
            size_elem.set_character_data(*size)?;
        }
        if let Some(unit) = &self.unit {
            sw_value_cont_elem
                .create_sub_element(ElementName::UnitRef)
                .and_then(|unit_elem| unit_elem.set_reference_target(unit.element()))?;
        }
        Ok(())
    }

    fn load(element: &Element) -> Option<Self> {
        let rule_based_values_elem = element.get_sub_element(ElementName::RuleBasedValues)?;
        let rule_based_values = RuleBasedValueSpecification::load(&rule_based_values_elem)?;
        let sw_array_size_elem = element.get_sub_element(ElementName::SwArraysize)?;
        let sw_array_size = sw_array_size_elem
            .sub_elements()
            .filter_map(|elem| elem.character_data()?.parse_integer())
            .collect::<Vec<_>>();
        let unit = element
            .get_sub_element(ElementName::UnitRef)
            .and_then(|unit_elem| unit_elem.get_reference_target().ok())
            .and_then(|unit_elem| Unit::try_from(unit_elem).ok());

        Some(Self {
            rule_based_values,
            sw_array_size,
            unit,
        })
    }
}

//#########################################################

/// rule based value specification
#[derive(Debug, Clone, PartialEq)]
pub struct RuleBasedValueSpecification {
    /// arguments of the rule-based value specification; they are filled in-order, andf the last one is repeated as required
    pub arguments: Vec<RuleArgument>,
    /// maximum size of the array to fill. It is used if the filling rule is set to `FILL_UNTIL_MAX_SIZE`
    pub max_size_to_fill: Option<u64>,
    /// rule to fill the array
    pub rule: RuleBasedFillUntil,
}

impl RuleBasedValueSpecification {
    fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        let rule_based_value_elem = parent.create_sub_element(ElementName::RuleBasedValues)?;
        // store the arguments
        let arguments_elem = rule_based_value_elem
            .create_sub_element(ElementName::Argumentss)?
            .create_sub_element(ElementName::RuleArguments)?;
        for argument in &self.arguments {
            argument.store(&arguments_elem)?;
        }
        if let Some(max_size) = self.max_size_to_fill {
            let max_size_elem = rule_based_value_elem.create_sub_element(ElementName::MaxSizeToFill)?;
            max_size_elem.set_character_data(max_size.to_string())?;
        }
        let rule_elem = rule_based_value_elem.create_sub_element(ElementName::Rule)?;
        rule_elem.set_character_data(self.rule.to_string())?;
        Ok(())
    }

    fn load(element: &Element) -> Option<Self> {
        let arguments = element
            .get_sub_element(ElementName::Argumentss)?
            .get_sub_element(ElementName::RuleArguments)?
            .sub_elements()
            .filter_map(|elem| RuleArgument::load(&elem))
            .collect::<Vec<_>>();

        let max_size_to_fill = element
            .get_sub_element(ElementName::MaxSizeToFill)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_integer());

        let rule_text = element
            .get_sub_element(ElementName::Rule)?
            .character_data()?
            .string_value()?;
        let rule = RuleBasedFillUntil::from_str(&rule_text).ok()?;

        Some(Self {
            arguments,
            max_size_to_fill,
            rule,
        })
    }
}

//#########################################################

/// specification of a composite value
#[derive(Debug, Clone, PartialEq)]
pub enum CompositeValueSpecification {
    /// array of values
    Array(ArrayValueSpecification),
    /// record of values
    Record(RecordValueSpecification),
}

impl CompositeValueSpecification {
    fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        match self {
            Self::Array(array) => array.store(parent),
            Self::Record(record) => record.store(parent),
        }
    }

    fn load(element: &Element) -> Option<Self> {
        match element.element_name() {
            ElementName::ArrayValueSpecification => ArrayValueSpecification::load(element).map(Self::Array),
            ElementName::RecordValueSpecification => RecordValueSpecification::load(element).map(Self::Record),
            _ => None,
        }
    }
}

//#########################################################

/// specification of a composite value argument
#[derive(Debug, Clone, PartialEq)]
pub enum CompositeRuleBasedValueArgument {
    /// argument is an application value
    Application(ApplicationValueSpecification),
    /// argument is a rule-based application value
    ApplicationRuleBased(ApplicationRuleBasedValueSpecification),
}

impl CompositeRuleBasedValueArgument {
    fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        match self {
            Self::Application(app) => app.store(parent),
            Self::ApplicationRuleBased(app_rule) => app_rule.store(parent),
        }
    }

    fn load(element: &Element) -> Option<Self> {
        match element.element_name() {
            ElementName::ApplicationValueSpecification => {
                ApplicationValueSpecification::load(element).map(Self::Application)
            }
            ElementName::ApplicationRuleBasedValueSpecification => {
                ApplicationRuleBasedValueSpecification::load(element).map(Self::ApplicationRuleBased)
            }
            _ => None,
        }
    }
}

//#########################################################

#[derive(Debug, Clone, PartialEq)]
/// argument of a rule-based value specification
pub enum RuleArgument {
    /// V: argument is a numerical value
    V(f64),
    /// VF: argument is a numerical value
    Vf(f64),
    /// VT: argument is a text value
    Vt(String),
    /// VTF: argument is a numerical value
    VtfNumber(f64),
    /// VTF: argument is a text value
    VtfText(String),
}

impl RuleArgument {
    fn store(&self, parent: &Element) -> Result<(), AutosarAbstractionError> {
        match self {
            RuleArgument::V(value) => {
                let value_elem = parent.create_sub_element(ElementName::V)?;
                value_elem.set_character_data(value.to_string())?;
            }
            RuleArgument::Vf(value) => {
                let value_elem = parent.create_sub_element(ElementName::Vf)?;
                value_elem.set_character_data(value.to_string())?;
            }
            RuleArgument::Vt(value) => {
                let value_elem = parent.create_sub_element(ElementName::Vt)?;
                value_elem.set_character_data(value.clone())?;
            }
            RuleArgument::VtfNumber(value) => {
                let value_elem = parent.create_sub_element(ElementName::Vtf)?;
                value_elem.set_character_data(value.to_string())?;
            }
            RuleArgument::VtfText(value) => {
                let value_elem = parent.create_sub_element(ElementName::Vtf)?;
                value_elem.set_character_data(value.clone())?;
            }
        }
        Ok(())
    }

    fn load(element: &Element) -> Option<Self> {
        let value = match element.element_name() {
            ElementName::V => {
                let value = element.character_data()?.parse_float()?;
                RuleArgument::V(value)
            }
            ElementName::Vf => {
                let value = element.character_data()?.parse_float()?;
                RuleArgument::Vf(value)
            }
            ElementName::Vt => {
                let value = element.character_data()?.string_value()?;
                RuleArgument::Vt(value)
            }
            ElementName::Vtf => {
                if let Some(value) = element.character_data()?.parse_float() {
                    RuleArgument::VtfNumber(value)
                } else {
                    let value = element.character_data()?.string_value()?;
                    RuleArgument::VtfText(value)
                }
            }
            _ => return None,
        };
        Some(value)
    }
}

//#########################################################

/// enum of all data prototypes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataPrototype {
    /// argument data prototype
    ArgumentDataPrototype(ArgumentDataPrototype),
    /// parameter data prototype
    ParameterDataPrototype(ParameterDataPrototype),
    /// variable data prototype
    VariableDataPrototype(VariableDataPrototype),
    /// application array element
    ApplicationArrayElement(ApplicationArrayElement),
    /// application record element
    ApplicationRecordElement(ApplicationRecordElement),
}

impl TryFrom<Element> for DataPrototype {
    type Error = AutosarAbstractionError;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        match value.element_name() {
            ElementName::ArgumentDataPrototype => {
                ArgumentDataPrototype::try_from(value).map(Self::ArgumentDataPrototype)
            }
            ElementName::ParameterDataPrototype => {
                ParameterDataPrototype::try_from(value).map(Self::ParameterDataPrototype)
            }
            ElementName::VariableDataPrototype => {
                VariableDataPrototype::try_from(value).map(Self::VariableDataPrototype)
            }
            ElementName::Element => ApplicationArrayElement::try_from(value).map(Self::ApplicationArrayElement),
            ElementName::ApplicationRecordElement => {
                ApplicationRecordElement::try_from(value).map(Self::ApplicationRecordElement)
            }
            _ => Err(AutosarAbstractionError::ConversionError {
                element: value,
                dest: "DataPrototype".to_string(),
            }),
        }
    }
}

impl AbstractionElement for DataPrototype {
    fn element(&self) -> &Element {
        match self {
            DataPrototype::ArgumentDataPrototype(arg) => arg.element(),
            DataPrototype::ParameterDataPrototype(param) => param.element(),
            DataPrototype::VariableDataPrototype(var) => var.element(),
            DataPrototype::ApplicationArrayElement(arr) => arr.element(),
            DataPrototype::ApplicationRecordElement(rec) => rec.element(),
        }
    }
}

//#########################################################

#[cfg(test)]
mod test {
    use std::vec;

    use super::*;
    use crate::{
        AutosarModelAbstraction,
        datatype::{BaseTypeEncoding, ImplementationDataTypeSettings},
        software_component::{ArgumentDirection, ClientServerInterface},
    };
    use autosar_data::AutosarVersion;

    #[test]
    fn numerical_value_specification() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/Pkg").unwrap();

        let spec = NumericalValueSpecification {
            label: Some("NumericalValue".to_string()),
            value: 33.3,
        };
        let constant = package
            .create_constant_specification("ConstantSpec", spec.clone())
            .unwrap();
        let spec_read = constant.value_specification().unwrap();
        assert_eq!(spec_read, spec.into());
    }

    #[test]
    fn array_value_specification() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/Pkg").unwrap();

        let spec = ArrayValueSpecification {
            label: Some("ArrayValue".to_string()),
            values: vec![
                NumericalValueSpecification {
                    label: None,
                    value: 11.0,
                }
                .into(),
                NumericalValueSpecification {
                    label: None,
                    value: 12.3,
                }
                .into(),
            ],
        };
        let constant = package
            .create_constant_specification("ConstantSpec", spec.clone())
            .unwrap();
        let spec_read = constant.value_specification().unwrap();
        assert_eq!(spec_read, spec.into());
    }

    #[test]
    fn record_value_specification() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/Pkg").unwrap();

        let spec = RecordValueSpecification {
            label: Some("RecordValue".to_string()),
            values: vec![
                NumericalValueSpecification {
                    label: None,
                    value: 1.0,
                }
                .into(),
                NumericalValueSpecification {
                    label: None,
                    value: 3.1,
                }
                .into(),
            ],
        };
        let constant = package
            .create_constant_specification("ConstantSpec", spec.clone())
            .unwrap();
        let spec_read = constant.value_specification().unwrap();
        assert_eq!(spec_read, spec.into());
    }

    #[test]
    fn text_value_specification() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/Pkg").unwrap();

        let spec = TextValueSpecification {
            label: Some("TextValue".to_string()),
            value: "Hello World".to_string(),
        };
        let constant = package
            .create_constant_specification("ConstantSpec", spec.clone())
            .unwrap();
        let spec_read = constant.value_specification().unwrap();
        assert_eq!(spec_read, spec.into());
    }

    #[test]
    fn reference_value_specification() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/Pkg").unwrap();

        let base_type = package
            .create_sw_base_type("base", 32, BaseTypeEncoding::None, None, None, None)
            .unwrap();
        let impl_settings = ImplementationDataTypeSettings::Value {
            name: "ImplementationValue".to_string(),
            base_type,
            compu_method: None,
            data_constraint: None,
        };
        let datatype = package.create_implementation_data_type(&impl_settings).unwrap();
        let app_data_type = package
            .create_application_primitive_data_type(
                "AppDataType",
                ApplicationPrimitiveCategory::Value,
                None,
                None,
                None,
            )
            .unwrap();

        // create a constant using each different kind of DataPrototype
        // ArgumentDataPrototype of a ClientServerInterface
        let client_server_interface = ClientServerInterface::new("CS_Interface", &package).unwrap();
        let cs_operation = client_server_interface.create_operation("Operation").unwrap();
        let argument_data_prototype = cs_operation
            .create_argument("adp", &datatype, ArgumentDirection::In)
            .unwrap();

        let spec = ReferenceValueSpecification {
            label: Some("ReferenceValue".to_string()),
            reference_value: DataPrototype::ArgumentDataPrototype(argument_data_prototype),
        };
        let constant = package
            .create_constant_specification("ConstantSpec1", spec.clone())
            .unwrap();
        let spec_read = constant.value_specification().unwrap();
        assert_eq!(spec_read, spec.into());

        // ParameterDataPrototype of a ParameterInterface
        let parameter_interface = package.create_parameter_interface("P_Interface").unwrap();
        let parameter_data_prototype = parameter_interface.create_parameter("pdp", &datatype).unwrap();

        let spec = ReferenceValueSpecification {
            label: Some("ReferenceValue".to_string()),
            reference_value: DataPrototype::ParameterDataPrototype(parameter_data_prototype),
        };
        let constant = package
            .create_constant_specification("ConstantSpec2", spec.clone())
            .unwrap();
        let spec_read = constant.value_specification().unwrap();
        assert_eq!(spec_read, spec.into());

        // VariableDataPrototype of a SenderReceiverInterface
        let sender_receiver_interface = package.create_sender_receiver_interface("SR_Interface").unwrap();
        let variable_data_prototype = sender_receiver_interface.create_data_element("vdp", &datatype).unwrap();

        let spec = ReferenceValueSpecification {
            label: Some("ReferenceValue".to_string()),
            reference_value: DataPrototype::VariableDataPrototype(variable_data_prototype),
        };
        let constant = package
            .create_constant_specification("ConstantSpec3", spec.clone())
            .unwrap();
        let spec_read = constant.value_specification().unwrap();
        assert_eq!(spec_read, spec.into());

        // ApplicationArrayElement of an ApplicationArrayDataType
        let application_array_data_type = package
            .create_application_array_data_type(
                "ArrayDataType",
                &app_data_type,
                crate::datatype::ApplicationArraySize::Fixed(1),
            )
            .unwrap();
        let application_array_element = application_array_data_type.array_element().unwrap();

        let spec = ReferenceValueSpecification {
            label: Some("ReferenceValue".to_string()),
            reference_value: DataPrototype::ApplicationArrayElement(application_array_element),
        };
        let constant = package
            .create_constant_specification("ConstantSpec4", spec.clone())
            .unwrap();
        let spec_read = constant.value_specification().unwrap();
        assert_eq!(spec_read, spec.into());

        // ApplicationRecordElement of an ApplicationRecordDataType
        let application_record_data_type = package.create_application_record_data_type("RecordDataType").unwrap();
        let application_record_element = application_record_data_type
            .create_record_element("Element", &app_data_type)
            .unwrap();

        let spec = ReferenceValueSpecification {
            label: Some("ReferenceValue".to_string()),
            reference_value: DataPrototype::ApplicationRecordElement(application_record_element),
        };
        let constant = package
            .create_constant_specification("ConstantSpec5", spec.clone())
            .unwrap();
        let spec_read = constant.value_specification().unwrap();
        assert_eq!(spec_read, spec.into());
    }

    #[test]
    fn constant_reference_specification() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/Pkg").unwrap();

        let spec = NumericalValueSpecification {
            label: None,
            value: 1.0,
        };
        let target_constant = package.create_constant_specification("Target", spec).unwrap();

        let spec = ConstantReference {
            label: Some("ConstantReference".to_string()),
            constant: target_constant,
        };
        let constant = package
            .create_constant_specification("ConstantSpec", spec.clone())
            .unwrap();
        let spec_read = constant.value_specification().unwrap();
        assert_eq!(spec_read, spec.into());
    }

    #[test]
    fn application_value_specification() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/Pkg").unwrap();
        let unit = package.create_unit("abc", Some("display_name")).unwrap();

        let spec = ApplicationValueSpecification {
            label: Some("ApplicationValue".to_string()),
            category: ApplicationPrimitiveCategory::ResAxis,
            sw_axis_conts: vec![
                SwAxisCont {
                    category: SwAxisContCategory::StdAxis,
                    sw_array_size: vec![1, 2],
                    sw_axis_index: 1,
                    sw_values_phys: vec![SwValue::V(0.0), SwValue::Vf(1.0), SwValue::Vt("text".to_string())],
                    unit: Some(unit),
                    unit_display_name: Some("display_name".to_string()),
                },
                SwAxisCont {
                    category: SwAxisContCategory::ComAxis,
                    sw_array_size: vec![3, 4],
                    sw_axis_index: 2,
                    sw_values_phys: vec![
                        SwValue::Vg {
                            label: Some("label".to_string()),
                            vg_content: vec![SwValue::VtfNumber(42.0)],
                        },
                        SwValue::VtfText("text".to_string()),
                    ],
                    unit: None,
                    unit_display_name: None,
                },
            ],
            sw_value_cont: SwValueCont {
                sw_array_size: vec![1, 2],
                sw_values_phys: vec![SwValue::Vf(0.0), SwValue::Vf(1.0)],
            },
        };
        let constant = package
            .create_constant_specification("ConstantSpec", spec.clone())
            .unwrap();
        let spec_read = constant.value_specification().unwrap();
        assert_eq!(spec_read, spec.into());
    }

    #[test]
    fn not_available_value_specification() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/Pkg").unwrap();

        let spec = NotAvailableValueSpecification {
            label: Some("NotAvailableValue".to_string()),
            default_pattern: Some(0x11),
        };
        let constant = package
            .create_constant_specification("ConstantSpec", spec.clone())
            .unwrap();
        let spec_read = constant.value_specification().unwrap();
        assert_eq!(spec_read, spec.into());
    }

    #[test]
    fn aplication_rule_based_value_specification() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/Pkg").unwrap();
        let unit = package.create_unit("abc", Some("display_name")).unwrap();

        let spec = ApplicationRuleBasedValueSpecification {
            label: Some("ApplicationRuleBasedValue".to_string()),
            category: ApplicationPrimitiveCategory::ResAxis,
            sw_axis_cont: vec![
                RuleBasedAxisCont {
                    category: SwAxisContCategory::StdAxis,
                    sw_array_size: vec![1, 2],
                    sw_axis_index: 1,
                    rule_based_values: RuleBasedValueSpecification {
                        arguments: vec![RuleArgument::Vf(0.0), RuleArgument::Vf(1.0)],
                        max_size_to_fill: Some(10),
                        rule: RuleBasedFillUntil::MaxSize,
                    },
                    unit: Some(unit.clone()),
                },
                RuleBasedAxisCont {
                    category: SwAxisContCategory::ComAxis,
                    sw_array_size: vec![3, 4],
                    sw_axis_index: 2,
                    rule_based_values: RuleBasedValueSpecification {
                        arguments: vec![RuleArgument::Vf(0.0), RuleArgument::Vf(1.0)],
                        max_size_to_fill: None,
                        rule: RuleBasedFillUntil::End,
                    },
                    unit: None,
                },
            ],
            sw_value_cont: RuleBasedValueCont {
                rule_based_values: RuleBasedValueSpecification {
                    arguments: vec![RuleArgument::Vf(0.0), RuleArgument::Vf(1.0)],
                    max_size_to_fill: None,
                    rule: RuleBasedFillUntil::End,
                },
                sw_array_size: vec![1, 2],
                unit: Some(unit),
            },
        };
        let constant = package
            .create_constant_specification("ConstantSpec", spec.clone())
            .unwrap();
        let spec_read = constant.value_specification().unwrap();
        assert_eq!(spec_read, spec.into());
    }

    #[test]
    fn composite_rule_based_value_specification() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/Pkg").unwrap();

        let spec = CompositeRuleBasedValueSpecification {
            label: Some("CompositeRuleBasedValue".to_string()),
            argument: vec![CompositeValueSpecification::Array(ArrayValueSpecification {
                label: Some("ArrayValue".to_string()),
                values: vec![
                    NumericalValueSpecification {
                        label: None,
                        value: 123.4,
                    }
                    .into(),
                    NumericalValueSpecification {
                        label: None,
                        value: 0.12345,
                    }
                    .into(),
                ],
            })],
            compound_primitive_argument: vec![
                CompositeRuleBasedValueArgument::Application(ApplicationValueSpecification {
                    label: Some("ApplicationValue".to_string()),
                    category: ApplicationPrimitiveCategory::ResAxis,
                    sw_axis_conts: vec![],
                    sw_value_cont: SwValueCont {
                        sw_array_size: vec![1, 2],
                        sw_values_phys: vec![SwValue::Vf(0.0), SwValue::Vf(1.0)],
                    },
                }),
                CompositeRuleBasedValueArgument::ApplicationRuleBased(ApplicationRuleBasedValueSpecification {
                    label: Some("ApplicationRuleBasedValue".to_string()),
                    category: ApplicationPrimitiveCategory::ResAxis,
                    sw_axis_cont: vec![],
                    sw_value_cont: RuleBasedValueCont {
                        rule_based_values: RuleBasedValueSpecification {
                            arguments: vec![RuleArgument::Vf(0.0), RuleArgument::Vf(1.0)],
                            max_size_to_fill: None,
                            rule: RuleBasedFillUntil::End,
                        },
                        sw_array_size: vec![1, 2],
                        unit: None,
                    },
                }),
            ],
            max_size_to_fill: Some(10),
            rule: RuleBasedFillUntil::MaxSize,
        };
        let constant = package
            .create_constant_specification("ConstantSpec", spec.clone())
            .unwrap();
        let spec_read = constant.value_specification().unwrap();
        assert_eq!(spec_read, spec.into());
    }

    #[test]
    fn numerical_rule_based_value_specification() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/Pkg").unwrap();

        let spec = NumericalRuleBasedValueSpecification {
            label: Some("NumericalRuleBasedValue".to_string()),
            rule_based_values: RuleBasedValueSpecification {
                arguments: vec![RuleArgument::Vf(0.0), RuleArgument::Vf(1.0)],
                max_size_to_fill: Some(10),
                rule: RuleBasedFillUntil::MaxSize,
            },
        };
        let constant = package
            .create_constant_specification("ConstantSpec", spec.clone())
            .unwrap();
        let spec_read = constant.value_specification().unwrap();
        assert_eq!(spec_read, spec.into());
    }

    #[test]
    fn conversions() {
        // RuleBasedFillUntil
        let value = RuleBasedFillUntil::End;
        let value_str = value.to_string();
        assert_eq!(value_str, "FILL_UNTIL_END");
        assert_eq!(RuleBasedFillUntil::from_str(&value_str).unwrap(), value);

        let value = RuleBasedFillUntil::MaxSize;
        let value_str = value.to_string();
        assert_eq!(value_str, "FILL_UNTIL_MAX_SIZE");
        assert_eq!(RuleBasedFillUntil::from_str(&value_str).unwrap(), value);

        // SwAxisContCategory
        let value = SwAxisContCategory::StdAxis;
        let enum_val: EnumItem = value.into();
        assert_eq!(enum_val, EnumItem::Stdaxis);
        assert_eq!(SwAxisContCategory::try_from(enum_val).unwrap(), value);
        // alternative form: EnumItem::StdAxis
        let enum_val = EnumItem::StdAxis;
        assert_eq!(SwAxisContCategory::try_from(enum_val).unwrap(), value);

        let value = SwAxisContCategory::ComAxis;
        let enum_val: EnumItem = value.into();
        assert_eq!(enum_val, EnumItem::Comaxis);
        assert_eq!(SwAxisContCategory::try_from(enum_val).unwrap(), value);
        // alternative form: EnumItem::ComAxis
        let enum_val = EnumItem::ComAxis;
        assert_eq!(SwAxisContCategory::try_from(enum_val).unwrap(), value);

        let value = SwAxisContCategory::ResAxis;
        let enum_val: EnumItem = value.into();
        assert_eq!(enum_val, EnumItem::Resaxis);
        assert_eq!(SwAxisContCategory::try_from(enum_val).unwrap(), value);
        // alternative form: EnumItem::ResAxis
        let enum_val = EnumItem::ResAxis;
        assert_eq!(SwAxisContCategory::try_from(enum_val).unwrap(), value);

        // invalid conversion
        assert!(SwAxisContCategory::try_from(EnumItem::Aa).is_err());
    }
}
