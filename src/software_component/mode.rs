use std::str::FromStr;

use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, Element, IdentifiableAbstractionElement,
    abstraction_element,
};
use autosar_data::ElementName;

//##################################################################

/// A `ModeDeclarationGroup` is a collection of mode declarations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModeDeclarationGroup(Element);
abstraction_element!(ModeDeclarationGroup, ModeDeclarationGroup);
impl IdentifiableAbstractionElement for ModeDeclarationGroup {}

impl ModeDeclarationGroup {
    /// Create a new `ModeDeclarationGroup`
    pub(crate) fn new(
        name: &str,
        package: &ArPackage,
        category: Option<ModeDeclarationGroupCategory>,
    ) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let mode_declaration_group_elem = elements.create_named_sub_element(ElementName::ModeDeclarationGroup, name)?;
        let mode_declaration_group = Self(mode_declaration_group_elem);

        mode_declaration_group.set_category(category)?;

        Ok(mode_declaration_group)
    }

    /// Set the category of the mode declaration group
    pub fn set_category(&self, category: Option<ModeDeclarationGroupCategory>) -> Result<(), AutosarAbstractionError> {
        if let Some(category) = category {
            self.element()
                .get_or_create_sub_element(ElementName::Category)?
                .set_character_data(category.to_string())?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::Category);
        }
        Ok(())
    }

    /// Get the category of the mode declaration group
    #[must_use]
    pub fn category(&self) -> Option<ModeDeclarationGroupCategory> {
        let category = self
            .element()
            .get_sub_element(ElementName::Category)?
            .character_data()?
            .string_value()?;
        ModeDeclarationGroupCategory::from_str(&category).ok()
    }

    /// Create a new mode declaration in the mode declaration group
    pub fn create_mode_declaration(&self, name: &str) -> Result<ModeDeclaration, AutosarAbstractionError> {
        let mode_declarations = self
            .element()
            .get_or_create_sub_element(ElementName::ModeDeclarations)?;
        ModeDeclaration::new(name, &mode_declarations)
    }

    /// Iterate over all mode declarations in the mode declaration group
    pub fn mode_declarations(&self) -> impl Iterator<Item = ModeDeclaration> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::ModeDeclarations)
            .into_iter()
            .flat_map(|mode_declarations| mode_declarations.sub_elements())
            .filter_map(|elem| ModeDeclaration::try_from(elem).ok())
    }

    /// Set the initial mode of the mode declaration group
    ///
    /// The initial mode is active before any mode is set.
    /// This setting is required to be present and the referenced mode must be part of the mode declaration group.
    pub fn set_initial_mode(&self, mode_declaration: &ModeDeclaration) -> Result<(), AutosarAbstractionError> {
        if mode_declaration.element().named_parent()?.as_ref() != Some(self.element()) {
            return Err(AutosarAbstractionError::InvalidParameter(
                "Mode declaration is not part of the mode declaration group".to_string(),
            ));
        }
        self.element()
            .get_or_create_sub_element(ElementName::InitialModeRef)?
            .set_reference_target(mode_declaration.element())?;
        Ok(())
    }

    /// Get the initial mode of the mode declaration group
    #[must_use]
    pub fn initial_mode(&self) -> Option<ModeDeclaration> {
        self.element()
            .get_sub_element(ElementName::InitialModeRef)
            .and_then(|elem| elem.get_reference_target().ok())
            .and_then(|target| ModeDeclaration::try_from(target).ok())
    }

    /// set the onTransitionValue attribute of the mode declaration group
    pub fn set_on_transition_value(&self, value: Option<u64>) -> Result<(), AutosarAbstractionError> {
        if let Some(value) = value {
            self.element()
                .get_or_create_sub_element(ElementName::OnTransitionValue)?
                .set_character_data(value)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::OnTransitionValue);
        }
        Ok(())
    }

    /// Get the onTransitionValue attribute of the mode declaration group
    #[must_use]
    pub fn on_transition_value(&self) -> Option<u64> {
        self.element()
            .get_sub_element(ElementName::OnTransitionValue)?
            .character_data()?
            .parse_integer()
    }
}

//##################################################################

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Category of mode declaration groupy, which defines the ordering of the modes in the group
pub enum ModeDeclarationGroupCategory {
    /// Ordering of the modes in the mode declaration group is alphabetic, and the modes may not set a value
    AlphabeticOrder,
    /// Ordering of modes in the mode declaration group is made explixit by the value, which must be set for each mode.
    /// Additonally, the on_transition_value attribute must be set in this case.
    ExplicitOrder,
}

impl std::fmt::Display for ModeDeclarationGroupCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModeDeclarationGroupCategory::AlphabeticOrder => f.write_str("ALPHABETIC_ORDER"),
            ModeDeclarationGroupCategory::ExplicitOrder => f.write_str("EXPLICIT_ORDER"),
        }
    }
}

impl std::str::FromStr for ModeDeclarationGroupCategory {
    type Err = AutosarAbstractionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ALPHABETIC_ORDER" => Ok(ModeDeclarationGroupCategory::AlphabeticOrder),
            "EXPLICIT_ORDER" => Ok(ModeDeclarationGroupCategory::ExplicitOrder),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: s.to_string(),
                dest: "ModeDeclarationGroupCategory".to_string(),
            }),
        }
    }
}

//##################################################################

/// A `ModeDeclaration` represents a mode declaration in a `ModeDeclarationGroup`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModeDeclaration(Element);
abstraction_element!(ModeDeclaration, ModeDeclaration);
impl IdentifiableAbstractionElement for ModeDeclaration {}

impl ModeDeclaration {
    /// Create a new `ModeDeclaration`
    fn new(name: &str, parent_element: &Element) -> Result<Self, AutosarAbstractionError> {
        let mode_declaration = parent_element.create_named_sub_element(ElementName::ModeDeclaration, name)?;
        Ok(Self(mode_declaration))
    }

    /// Set the value that should be used to represent the mode in the RTE
    pub fn set_value(&self, value: Option<u64>) -> Result<(), AutosarAbstractionError> {
        if let Some(value) = value {
            self.element()
                .get_or_create_sub_element(ElementName::Value)?
                .set_character_data(value)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::Value);
        }
        Ok(())
    }

    /// Get the value that should be used to represent the mode in the RTE
    #[must_use]
    pub fn value(&self) -> Option<u64> {
        self.element()
            .get_sub_element(ElementName::Value)?
            .character_data()?
            .parse_integer()
    }

    /// Get the mode declaration group that this mode declaration belongs to
    pub fn mode_declaration_group(&self) -> Result<ModeDeclarationGroup, AutosarAbstractionError> {
        let Some(parent) = self.element().named_parent()? else {
            return Err(AutosarAbstractionError::InvalidParameter(
                "Mode declaration has no parent".to_string(),
            ));
        };
        ModeDeclarationGroup::try_from(parent)
    }
}

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::AutosarModelAbstraction;
    use autosar_data::AutosarVersion;

    #[test]
    fn mode_declaration_group() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/Pkg").unwrap();
        let mode_declaration_group = package.create_mode_declaration_group("test_group", None).unwrap();

        assert_eq!(mode_declaration_group.category(), None);
        assert_eq!(mode_declaration_group.name().unwrap(), "test_group");

        mode_declaration_group
            .set_category(Some(ModeDeclarationGroupCategory::ExplicitOrder))
            .unwrap();
        assert_eq!(
            mode_declaration_group.category(),
            Some(ModeDeclarationGroupCategory::ExplicitOrder)
        );
        mode_declaration_group
            .set_category(Some(ModeDeclarationGroupCategory::AlphabeticOrder))
            .unwrap();
        assert_eq!(
            mode_declaration_group.category(),
            Some(ModeDeclarationGroupCategory::AlphabeticOrder)
        );

        assert_eq!(mode_declaration_group.on_transition_value(), None);
        mode_declaration_group.set_on_transition_value(Some(42)).unwrap();
        assert_eq!(mode_declaration_group.on_transition_value(), Some(42));
        mode_declaration_group.set_on_transition_value(None).unwrap();
        assert_eq!(mode_declaration_group.on_transition_value(), None);

        let mode_declaration = mode_declaration_group.create_mode_declaration("test_mode").unwrap();
        mode_declaration.set_value(Some(1)).unwrap();
        assert_eq!(mode_declaration.value(), Some(1));

        assert_eq!(mode_declaration_group.mode_declarations().count(), 1);

        mode_declaration_group.set_initial_mode(&mode_declaration).unwrap();
        assert_eq!(mode_declaration_group.initial_mode().unwrap(), mode_declaration);

        let mode_declaration_group_2 = package.create_mode_declaration_group("test_group_2", None).unwrap();
        mode_declaration_group_2
            .set_initial_mode(&mode_declaration)
            .unwrap_err(); // should fail, because mode_declaration is not part of the group
    }
}
