//! # Autosar Data Types
//!
//! This module contains the implementation of the AUTOSAR data types, as well as supporting elements like compu methods and data constraints.

use crate::{
    abstraction_element, AbstractionElement, ArPackage, AutosarAbstractionError, IdentifiableAbstractionElement,
};
use autosar_data::{Element, ElementName};

mod applicationtype;
mod basetype;
mod compu_method;
mod implementationtype;
mod mapping;

pub use applicationtype::*;
pub use basetype::*;
pub use compu_method::*;
pub use implementationtype::*;
pub use mapping::*;

//#########################################################

/// `AbstractAutosarDataType` is a marker trait for all data types
pub trait AbstractAutosarDataType: AbstractionElement {}

//#########################################################

/// `AutosarDataType` is the abstract base class for all data types in the AUTOSAR metamodel.
///
/// It encapsulates both application data types and implementation data types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AutosarDataType {
    /// An application primitive data type
    ApplicationPrimitiveDataType(ApplicationPrimitiveDataType),
    /// An application array data type
    ApplicationArrayDataType(ApplicationArrayDataType),
    /// An application record data type
    ApplicationRecordDataType(ApplicationRecordDataType),
    /// An implementation data type
    ImplementationDataType(ImplementationDataType),
}

impl AbstractionElement for AutosarDataType {
    fn element(&self) -> &Element {
        match self {
            AutosarDataType::ApplicationPrimitiveDataType(data_type) => data_type.element(),
            AutosarDataType::ApplicationArrayDataType(data_type) => data_type.element(),
            AutosarDataType::ApplicationRecordDataType(data_type) => data_type.element(),
            AutosarDataType::ImplementationDataType(data_type) => data_type.element(),
        }
    }
}

impl TryFrom<Element> for AutosarDataType {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::ApplicationPrimitiveDataType => Ok(Self::ApplicationPrimitiveDataType(
                ApplicationPrimitiveDataType::try_from(element)?,
            )),
            ElementName::ApplicationArrayDataType => Ok(Self::ApplicationArrayDataType(
                ApplicationArrayDataType::try_from(element)?,
            )),
            ElementName::ApplicationRecordDataType => Ok(Self::ApplicationRecordDataType(
                ApplicationRecordDataType::try_from(element)?,
            )),
            ElementName::ImplementationDataType => {
                Ok(Self::ImplementationDataType(ImplementationDataType::try_from(element)?))
            }
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "AutosarDataType".to_string(),
            }),
        }
    }
}

impl IdentifiableAbstractionElement for AutosarDataType {}

//#########################################################

/// `Unit` represents a unit of measurement.
///
/// Use [`ArPackage::create_unit`] to create a new unit.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Unit(Element);
abstraction_element!(Unit, Unit);
impl IdentifiableAbstractionElement for Unit {}

impl Unit {
    /// Create a new unit
    pub(crate) fn new(
        name: &str,
        package: &ArPackage,
        display_name: Option<&str>,
    ) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let unit = elements.create_named_sub_element(ElementName::Unit, name)?;

        if let Some(display_name) = display_name {
            unit.create_sub_element(ElementName::DisplayName)?
                .set_character_data(display_name)?;
        }

        Ok(Self(unit))
    }
}

//#########################################################

/// `DataConstr` represents a data constraint.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataConstr(Element);
abstraction_element!(DataConstr, DataConstr);
impl IdentifiableAbstractionElement for DataConstr {}

impl DataConstr {
    /// Create a new data constraint
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let data_constr = elements.create_named_sub_element(ElementName::DataConstr, name)?;

        Ok(Self(data_constr))
    }

    /// Create a data constraint rule
    pub fn create_data_constr_rule(
        &self,
        rule_type: DataConstrType,
        lower_limit: Option<f64>,
        upper_limit: Option<f64>,
    ) -> Result<DataConstrRule, AutosarAbstractionError> {
        let data_constr_rules = self.element().get_or_create_sub_element(ElementName::DataConstrRules)?;
        let rule = DataConstrRule::new(&data_constr_rules, rule_type, lower_limit, upper_limit)?;
        Ok(rule)
    }

    /// Get all data constraint rules
    pub fn data_constr_rules(&self) -> impl Iterator<Item = DataConstrRule> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::DataConstrRules)
            .into_iter()
            .flat_map(|rules| rules.sub_elements())
            .filter_map(|elem| DataConstrRule::try_from(elem).ok())
    }
}

//#########################################################

/// `DataConstrRule` represents a data constraint rule.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataConstrRule(Element);
abstraction_element!(DataConstrRule, DataConstrRule);

impl DataConstrRule {
    pub(crate) fn new(
        parent: &Element,
        rule_type: DataConstrType,
        lower_limit: Option<f64>,
        upper_limit: Option<f64>,
    ) -> Result<Self, AutosarAbstractionError> {
        let rule = parent.create_sub_element(ElementName::DataConstrRule)?;
        let constrs = match rule_type {
            DataConstrType::Internal => rule.create_sub_element(ElementName::InternalConstrs)?,
            DataConstrType::Physical => rule.create_sub_element(ElementName::PhysConstrs)?,
        };

        if let Some(lower_limit) = lower_limit {
            constrs
                .create_sub_element(ElementName::LowerLimit)?
                .set_character_data(lower_limit)?;
        }

        if let Some(upper_limit) = upper_limit {
            constrs
                .create_sub_element(ElementName::UpperLimit)?
                .set_character_data(upper_limit)?;
        }

        Ok(Self(rule))
    }

    /// get the constraint type
    #[must_use]
    pub fn rule_type(&self) -> DataConstrType {
        if self.element().get_sub_element(ElementName::InternalConstrs).is_some() {
            DataConstrType::Internal
        } else {
            DataConstrType::Physical
        }
    }

    /// get the lower limit
    #[must_use]
    pub fn lower_limit(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::InternalConstrs)
            .or(self.element().get_sub_element(ElementName::PhysConstrs))?
            .get_sub_element(ElementName::LowerLimit)?
            .character_data()?
            .parse_float()
    }

    /// get the upper limit
    #[must_use]
    pub fn upper_limit(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::InternalConstrs)
            .or(self.element().get_sub_element(ElementName::PhysConstrs))?
            .get_sub_element(ElementName::UpperLimit)?
            .character_data()?
            .parse_float()
    }
}

//#########################################################

/// The type of a data constraint rule
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataConstrType {
    /// Internal value data constraint
    Internal,
    /// Physical value data constraint
    Physical,
}

//#########################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::AutosarModelAbstraction;
    use autosar_data::AutosarVersion;

    #[test]
    fn data_constr() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/DataConstraints").unwrap();

        let data_constr = DataConstr::new("DataConstr", &package).unwrap();

        let rule1 = data_constr
            .create_data_constr_rule(DataConstrType::Internal, Some(1.0), Some(100.0))
            .unwrap();
        assert_eq!(rule1.rule_type(), DataConstrType::Internal);
        assert_eq!(rule1.lower_limit(), Some(1.0));
        assert_eq!(rule1.upper_limit(), Some(100.0));

        let rule2 = data_constr
            .create_data_constr_rule(DataConstrType::Physical, Some(2.0), Some(200.0))
            .unwrap();
        assert_eq!(rule2.rule_type(), DataConstrType::Physical);
        assert_eq!(rule2.lower_limit(), Some(2.0));
        assert_eq!(rule2.upper_limit(), Some(200.0));

        let rules = data_constr.data_constr_rules().collect::<Vec<_>>();
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn autosar_data_type() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/DataTypes").unwrap();

        let app_primitive = ApplicationPrimitiveDataType::new(
            "Primitive",
            &package,
            ApplicationPrimitiveCategory::Value,
            None,
            None,
            None,
        )
        .unwrap();
        let app_array = ApplicationArrayDataType::new("Array", &package, &app_primitive, 1).unwrap();
        let app_record = ApplicationRecordDataType::new("Record", &package).unwrap();
        let base_type =
            SwBaseType::new("uint8", &package, 8, BaseTypeEncoding::None, None, None, Some("uint8")).unwrap();
        let impl_settings = ImplementationDataTypeSettings::Value {
            name: "ImplValue".to_string(),
            base_type: base_type.clone(),
            compu_method: None,
            data_constraint: None,
        };
        let impl_type = ImplementationDataType::new(&package, impl_settings).unwrap();

        let app_primitive2 = AutosarDataType::try_from(app_primitive.element().clone()).unwrap();
        assert!(matches!(
            app_primitive2,
            AutosarDataType::ApplicationPrimitiveDataType(_)
        ));
        assert_eq!(app_primitive2.element(), app_primitive.element());

        let app_array2 = AutosarDataType::try_from(app_array.element().clone()).unwrap();
        assert!(matches!(app_array2, AutosarDataType::ApplicationArrayDataType(_)));
        assert_eq!(app_array2.element(), app_array.element());

        let app_record2 = AutosarDataType::try_from(app_record.element().clone()).unwrap();
        assert!(matches!(app_record2, AutosarDataType::ApplicationRecordDataType(_)));
        assert_eq!(app_record2.element(), app_record.element());

        let impl_type2 = AutosarDataType::try_from(impl_type.element().clone()).unwrap();
        assert!(matches!(impl_type2, AutosarDataType::ImplementationDataType(_)));
        assert_eq!(impl_type2.element(), impl_type.element());
    }
}
