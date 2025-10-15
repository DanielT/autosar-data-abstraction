use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element,
};
use autosar_data::{Element, ElementName, EnumItem};

mod container;
mod parameter;
mod reference;

pub use container::*;
pub use parameter::*;
pub use reference::*;

//#########################################################

/// `EcucCommonAttributes` provides methods to modify attributes that are shared by all parameters and references
pub trait EcucCommonAttributes: EcucDefinitionElement {
    /// set the multiplicity config classes of the parameter definition.
    /// If an empty list is provided, the multiplicity config classes are removed.
    ///
    /// This setting is required if the containing `EcucModuleDef` has the category `VENDOR_SPECIFIC_MODULE_DEFINITION`.
    fn set_multiplicity_config_classes(
        &self,
        config: &[(EcucConfigurationClass, EcucConfigurationVariant)],
    ) -> Result<(), AutosarAbstractionError> {
        set_config_classes(
            self.element(),
            ElementName::MultiplicityConfigClasses,
            ElementName::EcucMultiplicityConfigurationClass,
            config,
        )?;
        Ok(())
    }

    /// get the multiplicity config classes of the parameter definition
    #[must_use]
    fn multiplicity_config_classes(&self) -> Vec<(EcucConfigurationClass, EcucConfigurationVariant)> {
        get_config_classes(self.element(), ElementName::MultiplicityConfigClasses)
    }

    /// set the origin of the parameter definition
    ///
    /// The origin is a string that describes if the parameter was defined in the AUTOSAR standard or by a vendor.
    /// Standardized parameters use the origin "AUTOSAR_ECUC", while vendors are supposed to use string like "VendorXyz_v1.3"
    fn set_origin(&self, origin: &str) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::Origin)?
            .set_character_data(origin)?;
        Ok(())
    }

    /// get the origin of the parameter definition
    ///
    /// The origin is a string that describes if the parameter was defined in the AUTOSAR standard or by a vendor.
    /// Standardized parameters use the origin "AUTOSAR_ECUC", while vendors are supposed to use string like "VendorXyz_v1.3"
    #[must_use]
    fn origin(&self) -> Option<String> {
        self.element()
            .get_sub_element(ElementName::Origin)?
            .character_data()?
            .string_value()
    }

    /// set or remove the postBuildVariantMultiplicity attribute
    ///
    /// If postBuildVariantMultiplicity is true, then the parameter or reference
    /// may have a different number of instances in different post-build variants.
    fn set_post_build_variant_multiplicity(
        &self,
        post_build_variant_multiplicity: Option<bool>,
    ) -> Result<(), AutosarAbstractionError> {
        if let Some(post_build_variant_multiplicity) = post_build_variant_multiplicity {
            self.element()
                .get_or_create_sub_element(ElementName::PostBuildVariantMultiplicity)?
                .set_character_data(post_build_variant_multiplicity)?;
        } else {
            let _ = self
                .element()
                .remove_sub_element_kind(ElementName::PostBuildVariantMultiplicity);
        }

        Ok(())
    }

    /// get the postBuildVariantMultiplicity attribute
    ///
    /// If postBuildVariantMultiplicity is true, then the parameter or reference
    /// may have a different number of instances in different post-build variants.
    #[must_use]
    fn post_build_variant_multiplicity(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::PostBuildVariantMultiplicity)?
            .character_data()?
            .parse_bool()
    }

    /// set or remove the postBuildVariantValue attribute
    ///
    /// If postBuildVariantValue is true, then the parameter or reference
    /// may have different values in different post-build variants.
    fn set_post_build_variant_value(
        &self,
        post_build_variant_value: Option<bool>,
    ) -> Result<(), AutosarAbstractionError> {
        if let Some(post_build_variant_value) = post_build_variant_value {
            self.element()
                .get_or_create_sub_element(ElementName::PostBuildVariantValue)?
                .set_character_data(post_build_variant_value)?;
        } else {
            let _ = self
                .element()
                .remove_sub_element_kind(ElementName::PostBuildVariantValue);
        }

        Ok(())
    }

    /// get the postBuildVariantValue attribute
    ///
    /// If postBuildVariantValue is true, then the parameter or reference
    /// may have different values in different post-build variants.
    #[must_use]
    fn post_build_variant_value(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::PostBuildVariantValue)?
            .character_data()?
            .parse_bool()
    }

    /// set or remove the requiresIndex attribute
    fn set_requires_index(&self, requires_index: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(requires_index) = requires_index {
            self.element()
                .get_or_create_sub_element(ElementName::RequiresIndex)?
                .set_character_data(requires_index)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::RequiresIndex);
        }

        Ok(())
    }

    /// get the requiresIndex attribute
    #[must_use]
    fn requires_index(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::RequiresIndex)?
            .character_data()?
            .parse_bool()
    }

    /// set the value config classes of the parameter definition.
    ///
    /// If an empty list is provided, the value config classes are removed.
    /// According to the specification setting is required if the containing `EcucModuleDef`
    /// has the category "VENDOR_SPECIFIC_MODULE_DEFINITION", but in practice it is rarely used.
    fn set_value_config_classes(
        &self,
        config: &[(EcucConfigurationClass, EcucConfigurationVariant)],
    ) -> Result<(), AutosarAbstractionError> {
        set_config_classes(
            self.element(),
            ElementName::ValueConfigClasses,
            ElementName::EcucValueConfigurationClass,
            config,
        )?;
        Ok(())
    }

    /// get the value config classes of the parameter definition
    ///
    /// According to the specification setting is required if the containing `EcucModuleDef`
    /// has the category "VENDOR_SPECIFIC_MODULE_DEFINITION", but in practice it is rarely used.
    #[must_use]
    fn value_config_classes(&self) -> Vec<(EcucConfigurationClass, EcucConfigurationVariant)> {
        get_config_classes(self.element(), ElementName::ValueConfigClasses)
    }

    /// set or remove the withAuto attribute
    ///
    /// If withAuto is true, then the parameter or reference is allowed to set its isAutoValue attribute to true.
    fn set_with_auto(&self, with_auto: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(with_auto) = with_auto {
            self.element()
                .get_or_create_sub_element(ElementName::WithAuto)?
                .set_character_data(with_auto)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::WithAuto);
        }

        Ok(())
    }

    /// get the withAuto attribute
    fn with_auto(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::WithAuto)?
            .character_data()?
            .parse_bool()
    }
}

/// `EcucDefinitionElement` provides methods to modify attributes that are shared by all elements of the ecuc definition
pub trait EcucDefinitionElement: AbstractionElement {
    /// set or remove the lower multiplicity attribute
    fn set_lower_multiplicity(&self, lower_multiplicity: Option<u32>) -> Result<(), AutosarAbstractionError> {
        if let Some(lower_multiplicity) = lower_multiplicity {
            self.element()
                .get_or_create_sub_element(ElementName::LowerMultiplicity)?
                .set_character_data(lower_multiplicity as u64)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::LowerMultiplicity);
        }

        Ok(())
    }

    /// get the lower multiplicity attribute
    #[must_use]
    fn lower_multiplicity(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::LowerMultiplicity)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set or remove the upper multiplicity attribute
    fn set_upper_multiplicity(&self, upper_multiplicity: Option<u32>) -> Result<(), AutosarAbstractionError> {
        if let Some(upper_multiplicity) = upper_multiplicity {
            self.element()
                .get_or_create_sub_element(ElementName::UpperMultiplicity)?
                .set_character_data(upper_multiplicity as u64)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::UpperMultiplicity);
        }

        Ok(())
    }

    /// get the upper multiplicity attribute
    #[must_use]
    fn upper_multiplicity(&self) -> Option<u32> {
        self.element()
            .get_sub_element(ElementName::UpperMultiplicity)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set or remove the upper multiplicity infinite attribute
    ///
    /// if this attribute is set to true, the upper multiplicity is infinite
    /// (i.e. the module definition can be used an arbitrary number of times)
    /// When this attribute is true, the upper multiplicity attribute automatically removed.
    fn set_upper_multiplicity_infinite(&self, infinite: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(infinite) = infinite {
            self.element()
                .get_or_create_sub_element(ElementName::UpperMultiplicityInfinite)?
                .set_character_data(infinite)?;
            if infinite {
                let _ = self.element().remove_sub_element_kind(ElementName::UpperMultiplicity);
            }
        } else {
            let _ = self
                .element()
                .remove_sub_element_kind(ElementName::UpperMultiplicityInfinite);
        }

        Ok(())
    }

    /// get the upper multiplicity infinite attribute
    #[must_use]
    fn upper_multiplicity_infinite(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::UpperMultiplicityInfinite)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_bool())
    }
}

//#########################################################

/// The `EcucDefinitionCollection` is a container for all module definitions in the ECU configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucDefinitionCollection(Element);
abstraction_element!(EcucDefinitionCollection, EcucDefinitionCollection);
impl IdentifiableAbstractionElement for EcucDefinitionCollection {}

impl EcucDefinitionCollection {
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let ecuc_definition_collection_elem =
            elements.create_named_sub_element(ElementName::EcucDefinitionCollection, name)?;

        Ok(Self(ecuc_definition_collection_elem))
    }

    /// add a reference to a module definition to the collection
    pub fn add_module_def(&self, module_def: &EcucModuleDef) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::ModuleRefs)?
            .create_sub_element(ElementName::ModuleRef)?
            .set_reference_target(module_def.element())?;
        Ok(())
    }

    /// iterate over all module definitions in the collection
    pub fn module_defs(&self) -> impl Iterator<Item = EcucModuleDef> + Send + use<> {
        self.element()
            .get_sub_element(ElementName::ModuleRefs)
            .into_iter()
            .flat_map(|module_refs_elem| module_refs_elem.sub_elements())
            .filter_map(|module_ref_elem| module_ref_elem.get_reference_target().ok())
            .filter_map(|module_def_elem| EcucModuleDef::try_from(module_def_elem).ok())
    }
}

//#########################################################

/// The `EcucModuleDef` is a container for the definition of a single base software module
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucModuleDef(Element);
abstraction_element!(EcucModuleDef, EcucModuleDef);
impl IdentifiableAbstractionElement for EcucModuleDef {}
impl EcucDefinitionElement for EcucModuleDef {}

impl EcucModuleDef {
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let ecuc_module_def_elem = elements.create_named_sub_element(ElementName::EcucModuleDef, name)?;

        Ok(Self(ecuc_module_def_elem))
    }

    /// create a new `EcucChoiceContainerDef` in the module
    pub fn create_choice_container_def(&self, name: &str) -> Result<EcucChoiceContainerDef, AutosarAbstractionError> {
        let containers_elem = self.element().get_or_create_sub_element(ElementName::Containers)?;
        EcucChoiceContainerDef::new(name, &containers_elem)
    }

    /// create a new `EcucParamConfContainerDef` in the module
    pub fn create_param_conf_container_def(
        &self,
        name: &str,
    ) -> Result<EcucParamConfContainerDef, AutosarAbstractionError> {
        let containers_elem = self.element().get_or_create_sub_element(ElementName::Containers)?;
        EcucParamConfContainerDef::new(name, &containers_elem)
    }

    /// iterate over all containers in the module
    pub fn containers(&self) -> impl Iterator<Item = EcucContainerDef> + Send + use<> {
        self.element()
            .get_sub_element(ElementName::Containers)
            .into_iter()
            .flat_map(|containers_elem| containers_elem.sub_elements())
            .filter_map(|container_elem| EcucContainerDef::try_from(container_elem).ok())
    }

    /// set or remove the apiServicePrefix for the module
    ///
    /// for CDD modules the short name of the module is always "CDD", so
    /// this attribute is needed to define the prefix for the API services
    pub fn set_api_service_prefix(&self, prefix: Option<&str>) -> Result<(), AutosarAbstractionError> {
        if let Some(prefix) = prefix {
            self.element()
                .get_or_create_sub_element(ElementName::ApiServicePrefix)?
                .set_character_data(prefix)?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::ApiServicePrefix);
        }

        Ok(())
    }

    /// get the apiServicePrefix for the module
    ///
    /// for CDD modules the short name of the module is always "CDD", so
    /// this attribute is needed to define the prefix for the API services
    #[must_use]
    pub fn api_service_prefix(&self) -> Option<String> {
        self.element()
            .get_sub_element(ElementName::ApiServicePrefix)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.string_value())
    }

    /// set the supported configuration variants for the module
    pub fn set_supported_config_variants(
        &self,
        variants: &[EcucConfigurationVariant],
    ) -> Result<(), AutosarAbstractionError> {
        // remove the old supported configuration variants list
        let _ = self
            .element()
            .remove_sub_element_kind(ElementName::SupportedConfigVariants);

        // create the new supported configuration variants list
        let supported_config_variants_elem = self
            .element()
            .create_sub_element(ElementName::SupportedConfigVariants)?;
        for variant in variants {
            let variant_elem =
                supported_config_variants_elem.create_sub_element(ElementName::SupportedConfigVariant)?;
            variant_elem.set_character_data::<EnumItem>((*variant).into())?;
        }

        Ok(())
    }

    /// get the supported configuration variants for the module
    #[must_use]
    pub fn supported_config_variants(&self) -> Vec<EcucConfigurationVariant> {
        self.element()
            .get_sub_element(ElementName::SupportedConfigVariants)
            .map(|elem| {
                elem.sub_elements()
                    .filter_map(|variant_elem| {
                        variant_elem
                            .character_data()
                            .and_then(|cdata| cdata.enum_value())
                            .and_then(|enum_item| EcucConfigurationVariant::try_from(enum_item).ok())
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// set or remove the post build variant support attribute
    pub fn set_post_build_variant_support(&self, support: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(support) = support {
            self.element()
                .get_or_create_sub_element(ElementName::PostBuildVariantSupport)?
                .set_character_data(support)?;
        } else {
            let _ = self
                .element()
                .remove_sub_element_kind(ElementName::PostBuildVariantSupport);
        }

        Ok(())
    }

    /// get the post build variant support attribute
    #[must_use]
    pub fn post_build_variant_support(&self) -> Option<bool> {
        self.element()
            .get_sub_element(ElementName::PostBuildVariantSupport)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_bool())
    }

    /// set or remove the category of the module definition
    pub fn set_category(&self, category: Option<EcucModuleDefCategory>) -> Result<(), AutosarAbstractionError> {
        if let Some(category) = category {
            self.element()
                .get_or_create_sub_element(ElementName::Category)?
                .set_character_data(category.to_string())?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::Category);
        }

        Ok(())
    }

    /// get the category of the module definition
    #[must_use]
    pub fn category(&self) -> Option<EcucModuleDefCategory> {
        self.element()
            .get_sub_element(ElementName::Category)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.string_value())
            .and_then(|value| EcucModuleDefCategory::try_from(value.as_str()).ok())
    }

    /// set or remove the reference to a refined standard module
    ///
    /// This reference is only used if the category is `VendorSpecificModuleDefinition`
    pub fn set_refined_module_def(
        &self,
        refined_module_def: Option<&EcucModuleDef>,
    ) -> Result<(), AutosarAbstractionError> {
        if let Some(refined_module_def) = refined_module_def {
            self.element()
                .get_or_create_sub_element(ElementName::RefinedModuleDefRef)?
                .set_reference_target(refined_module_def.element())?;
        } else {
            let _ = self.element().remove_sub_element_kind(ElementName::RefinedModuleDefRef);
        }

        Ok(())
    }

    /// get the reference to a refined standard module
    ///
    /// This reference is only used if the category is `VendorSpecificModuleDefinition`
    #[must_use]
    pub fn refined_module_def(&self) -> Option<EcucModuleDef> {
        self.element()
            .get_sub_element(ElementName::RefinedModuleDefRef)
            .and_then(|elem| elem.get_reference_target().ok())
            .and_then(|target| EcucModuleDef::try_from(target).ok())
    }
}

//#########################################################

/// `EcucConfigurationVariant` provides the different configuration variants that
/// can be used by the module definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EcucConfigurationVariant {
    /// Preconfigured (i.e. fixed) configuration which cannot be changed.
    PreconfiguredConfiguration,
    /// Recommended configuration
    RecommendedConfiguration,
    /// the BSW Module implementation may use `PreCompileTime` and `LinkTime` configuration parameters
    VariantLinkTime,
    /// the BSW Module implementation may use `PreCompileTime`, `LinkTime` and `PostBuild` configuration parameters
    VariantPostBuild,
    /// the BSW Module implementation may use `PreCompileTime` configuration parameters
    VariantPreCompile,
    /// deprecated in Autosar 4.2.1 - the BSW Module implementation may use `PreCompileTime`, `LinkTime` and `PostBuild` loadable configuration parameters
    VariantPostBuildLoadable,
    /// deprecated in Autosar 4.2.1 - the BSW Module implementation may use `PreCompileTime`, `LinkTime` and `PostBuild` selectable configuration parameters
    VariantPostBuildSelectable,
}

impl From<EcucConfigurationVariant> for EnumItem {
    fn from(value: EcucConfigurationVariant) -> Self {
        match value {
            EcucConfigurationVariant::PreconfiguredConfiguration => Self::PreconfiguredConfiguration,
            EcucConfigurationVariant::RecommendedConfiguration => Self::RecommendedConfiguration,
            EcucConfigurationVariant::VariantLinkTime => Self::VariantLinkTime,
            EcucConfigurationVariant::VariantPostBuild => Self::VariantPostBuild,
            EcucConfigurationVariant::VariantPreCompile => Self::VariantPreCompile,
            EcucConfigurationVariant::VariantPostBuildLoadable => Self::VariantPostBuildLoadable,
            EcucConfigurationVariant::VariantPostBuildSelectable => Self::VariantPostBuildSelectable,
        }
    }
}

impl TryFrom<EnumItem> for EcucConfigurationVariant {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<Self, Self::Error> {
        match value {
            EnumItem::PreconfiguredConfiguration => Ok(Self::PreconfiguredConfiguration),
            EnumItem::RecommendedConfiguration => Ok(Self::RecommendedConfiguration),
            EnumItem::VariantLinkTime => Ok(Self::VariantLinkTime),
            EnumItem::VariantPostBuild => Ok(Self::VariantPostBuild),
            EnumItem::VariantPreCompile => Ok(Self::VariantPreCompile),
            EnumItem::VariantPostBuildLoadable => Ok(Self::VariantPostBuildLoadable),
            EnumItem::VariantPostBuildSelectable => Ok(Self::VariantPostBuildSelectable),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "EcucConfigurationVariant".to_string(),
            }),
        }
    }
}

//#########################################################

/// `EcucConfigurationClassEnum` provides the different configuration classes for Autosar configuration parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EcucConfigurationClass {
    /// Link Time: parts of configuration are delivered from another object code file
    Link,
    /// `PostBuild`: a configuration parameter can be changed after compilation
    PostBuild,
    /// `PreCompile`: a configuration parameter can not be changed after compilation
    PreCompile,
    /// `PublishedInformation` is used to specify the fact that certain information is fixed even before the pre-compile stage.
    PublishedInformation,
}

impl From<EcucConfigurationClass> for EnumItem {
    fn from(value: EcucConfigurationClass) -> Self {
        match value {
            EcucConfigurationClass::Link => Self::Link,
            EcucConfigurationClass::PostBuild => Self::PostBuild,
            EcucConfigurationClass::PreCompile => Self::PreCompile,
            EcucConfigurationClass::PublishedInformation => Self::PublishedInformation,
        }
    }
}

impl TryFrom<EnumItem> for EcucConfigurationClass {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<Self, Self::Error> {
        match value {
            EnumItem::Link => Ok(Self::Link),
            EnumItem::PostBuild => Ok(Self::PostBuild),
            EnumItem::PreCompile => Ok(Self::PreCompile),
            EnumItem::PublishedInformation => Ok(Self::PublishedInformation),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "EcucConfigurationClass".to_string(),
            }),
        }
    }
}

//#########################################################

/// The `EcucModuleDefCategory` represents the possible category values for a module definition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EcucModuleDefCategory {
    /// The module definition is a standardized module (StMD)
    StandardizedModuleDefinition,
    /// The module definition is a vendor specific module (VSMD)
    VendorSpecificModuleDefinition,
}

impl std::fmt::Display for EcucModuleDefCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EcucModuleDefCategory::StandardizedModuleDefinition => write!(f, "STANDARDIZED_MODULE_DEFINITION"),
            EcucModuleDefCategory::VendorSpecificModuleDefinition => write!(f, "VENDOR_SPECIFIC_MODULE_DEFINITION"),
        }
    }
}

impl TryFrom<&str> for EcucModuleDefCategory {
    type Error = AutosarAbstractionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "STANDARDIZED_MODULE_DEFINITION" => Ok(Self::StandardizedModuleDefinition),
            "VENDOR_SPECIFIC_MODULE_DEFINITION" => Ok(Self::VendorSpecificModuleDefinition),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "EcucModuleDefCategory".to_string(),
            }),
        }
    }
}

//#########################################################

// helper for setting the multiplicity config classes and the value config classes
fn set_config_classes(
    base: &Element,
    element_name_l1: ElementName,
    element_name_l2: ElementName,
    config: &[(EcucConfigurationClass, EcucConfigurationVariant)],
) -> Result<(), AutosarAbstractionError> {
    // remove the existing multiplicity config classes, since we configure
    // the entire list instead of updating the existing one
    let _ = base.remove_sub_element_kind(element_name_l1);

    if !config.is_empty() {
        // create the new multiplicity config classes
        let config_classes = base.create_sub_element(element_name_l1)?;
        for (config_class, variant) in config {
            let ecuc_config_class_elem = config_classes.create_sub_element(element_name_l2)?;
            ecuc_config_class_elem
                .create_sub_element(ElementName::ConfigClass)?
                .set_character_data::<EnumItem>((*config_class).into())?;
            ecuc_config_class_elem
                .create_sub_element(ElementName::ConfigVariant)?
                .set_character_data::<EnumItem>((*variant).into())?;
        }
    }

    Ok(())
}

// helper for getting the multiplicity config classes and the value config classes
fn get_config_classes(
    base: &Element,
    element_name_l1: ElementName,
) -> Vec<(EcucConfigurationClass, EcucConfigurationVariant)> {
    base.get_sub_element(element_name_l1)
        .into_iter()
        .flat_map(|config_classes| config_classes.sub_elements())
        .filter_map(|config_class| {
            let class = config_class
                .get_sub_element(ElementName::ConfigClass)?
                .character_data()?
                .enum_value()?;
            let variant = config_class
                .get_sub_element(ElementName::ConfigVariant)?
                .character_data()?
                .enum_value()?;
            Some((
                EcucConfigurationClass::try_from(class).ok()?,
                EcucConfigurationVariant::try_from(variant).ok()?,
            ))
        })
        .collect()
}

//#########################################################

/// A `EcucDestinationUriDefSet` contains a list of `EcucDestinationUriDef`s
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucDestinationUriDefSet(Element);
abstraction_element!(EcucDestinationUriDefSet, EcucDestinationUriDefSet);
impl IdentifiableAbstractionElement for EcucDestinationUriDefSet {}

impl EcucDestinationUriDefSet {
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let ecuc_destination_uri_def_set_elem =
            elements.create_named_sub_element(ElementName::EcucDestinationUriDefSet, name)?;

        Ok(Self(ecuc_destination_uri_def_set_elem))
    }

    /// create a new `EcucDestinationUriDef`
    pub fn create_destination_uri_def(
        &self,
        name: &str,
        contract: EcucDestinationUriNestingContract,
    ) -> Result<EcucDestinationUriDef, AutosarAbstractionError> {
        let defs = self
            .element()
            .get_or_create_sub_element(ElementName::DestinationUriDefs)?;
        EcucDestinationUriDef::new(name, &defs, contract)
    }

    /// iterate over all destination uri definitions in the set
    pub fn destination_uri_defs(&self) -> impl Iterator<Item = EcucDestinationUriDef> + Send + use<> {
        self.element()
            .get_sub_element(ElementName::DestinationUriDefs)
            .into_iter()
            .flat_map(|defs_elem| defs_elem.sub_elements())
            .filter_map(|def_elem| EcucDestinationUriDef::try_from(def_elem).ok())
    }
}

//#########################################################

/// A `EcucDestinationUriDef` defines a target for an `EcucUriReferenceDef`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EcucDestinationUriDef(Element);
abstraction_element!(EcucDestinationUriDef, EcucDestinationUriDef);
impl IdentifiableAbstractionElement for EcucDestinationUriDef {}

impl EcucDestinationUriDef {
    /// create a new `EcucDestinationUriDef`
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        contract: EcucDestinationUriNestingContract,
    ) -> Result<Self, AutosarAbstractionError> {
        let ecuc_destination_uri_def_elem =
            parent.create_named_sub_element(ElementName::EcucDestinationUriDef, name)?;

        let ecuc_destination_uri_def = Self(ecuc_destination_uri_def_elem);
        ecuc_destination_uri_def.set_nesting_contract(contract)?;

        Ok(ecuc_destination_uri_def)
    }

    /// set the nesting contract for the destination uri
    pub fn set_nesting_contract(
        &self,
        contract: EcucDestinationUriNestingContract,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::DestinationUriPolicy)?
            .get_or_create_sub_element(ElementName::DestinationUriNestingContract)?
            .set_character_data::<EnumItem>(contract.into())?;
        Ok(())
    }

    /// get the nesting contract for the destination uri
    #[must_use]
    pub fn nesting_contract(&self) -> Option<EcucDestinationUriNestingContract> {
        self.element()
            .get_sub_element(ElementName::DestinationUriPolicy)?
            .get_sub_element(ElementName::DestinationUriNestingContract)?
            .character_data()?
            .enum_value()
            .and_then(|enum_item| EcucDestinationUriNestingContract::try_from(enum_item).ok())
    }

    /// create an `EcucParamConfContainerDef` in the destination uri policy
    pub fn create_param_conf_container_def(
        &self,
        name: &str,
    ) -> Result<EcucParamConfContainerDef, AutosarAbstractionError> {
        let containers = self
            .element()
            .get_or_create_sub_element(ElementName::DestinationUriPolicy)?
            .get_or_create_sub_element(ElementName::Containers)?;
        EcucParamConfContainerDef::new(name, &containers)
    }

    /// create an `EcucChoiceContainerDef` in the destination uri policy
    pub fn create_choice_container_def(&self, name: &str) -> Result<EcucChoiceContainerDef, AutosarAbstractionError> {
        let containers = self
            .element()
            .get_or_create_sub_element(ElementName::DestinationUriPolicy)?
            .get_or_create_sub_element(ElementName::Containers)?;
        EcucChoiceContainerDef::new(name, &containers)
    }

    /// iterate over all containers in the destination uri policy
    pub fn containers(&self) -> impl Iterator<Item = EcucContainerDef> + Send + use<> {
        self.element()
            .get_sub_element(ElementName::DestinationUriPolicy)
            .and_then(|dup_elem| dup_elem.get_sub_element(ElementName::Containers))
            .into_iter()
            .flat_map(|policy_elem| policy_elem.sub_elements())
            .filter_map(|container_elem| EcucContainerDef::try_from(container_elem).ok())
    }

    // theoretically, the destination uri def could also contain parameters or references
    // it looks like nobody uses these, so we don't implement them here
}

//#########################################################

/// `EcucDestinationUriNestingContract` provides the different nesting contracts for destination URIs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EcucDestinationUriNestingContract {
    /// `EcucDestinationUriPolicy` describes elements (subContainers, Parameters, References) that are directly owned by the target container.
    LeafOfTargetContainer,
    /// `EcucDestinationUriPolicy` describes the target container of `EcucUriReferenceDef`.
    TargetContainer,
    /// `EcucDestinationUriPolicy` describes elements (subContainers, Parameters, References) that are owned by the target container or its subContainers.
    VertexOfTargetContainer,
}

impl From<EcucDestinationUriNestingContract> for EnumItem {
    fn from(value: EcucDestinationUriNestingContract) -> Self {
        match value {
            EcucDestinationUriNestingContract::LeafOfTargetContainer => Self::LeafOfTargetContainer,
            EcucDestinationUriNestingContract::TargetContainer => Self::TargetContainer,
            EcucDestinationUriNestingContract::VertexOfTargetContainer => Self::VertexOfTargetContainer,
        }
    }
}

impl TryFrom<EnumItem> for EcucDestinationUriNestingContract {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<Self, Self::Error> {
        match value {
            EnumItem::LeafOfTargetContainer => Ok(Self::LeafOfTargetContainer),
            EnumItem::TargetContainer => Ok(Self::TargetContainer),
            EnumItem::VertexOfTargetContainer => Ok(Self::VertexOfTargetContainer),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "EcucDestinationUriNestingContract".to_string(),
            }),
        }
    }
}

//#########################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::AutosarModelAbstraction;
    use autosar_data::AutosarVersion;

    #[test]
    fn ecuc_module_def() {
        let model = AutosarModelAbstraction::create("file.arxml", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        let ecuc_definition_collection = package.create_ecuc_definition_collection("collection").unwrap();
        assert_eq!(ecuc_definition_collection.module_defs().count(), 0);

        let ecuc_module_def = package.create_ecuc_module_def("module_def").unwrap();
        ecuc_definition_collection.add_module_def(&ecuc_module_def).unwrap();
        assert_eq!(ecuc_definition_collection.module_defs().count(), 1);

        assert_eq!(ecuc_module_def.api_service_prefix(), None);
        assert_eq!(
            ecuc_module_def.supported_config_variants(),
            Vec::<EcucConfigurationVariant>::new()
        );
        assert_eq!(ecuc_module_def.post_build_variant_support(), None);
        assert_eq!(ecuc_module_def.category(), None);
        assert_eq!(ecuc_module_def.refined_module_def(), None);
        assert_eq!(ecuc_module_def.lower_multiplicity(), None);
        assert_eq!(ecuc_module_def.upper_multiplicity(), None);
        assert_eq!(ecuc_module_def.upper_multiplicity_infinite(), None);

        let base_ecuc_module_def = package.create_ecuc_module_def("base_module_def").unwrap();
        ecuc_module_def
            .set_refined_module_def(Some(&base_ecuc_module_def))
            .unwrap();
        assert_eq!(ecuc_module_def.refined_module_def(), Some(base_ecuc_module_def.clone()));
        ecuc_module_def.set_api_service_prefix(Some("prefix")).unwrap();
        assert_eq!(ecuc_module_def.api_service_prefix(), Some("prefix".to_string()));
        ecuc_module_def
            .set_supported_config_variants(&[
                EcucConfigurationVariant::PreconfiguredConfiguration,
                EcucConfigurationVariant::VariantLinkTime,
            ])
            .unwrap();
        assert_eq!(
            ecuc_module_def.supported_config_variants(),
            vec![
                EcucConfigurationVariant::PreconfiguredConfiguration,
                EcucConfigurationVariant::VariantLinkTime
            ]
        );
        ecuc_module_def.set_post_build_variant_support(Some(true)).unwrap();
        assert_eq!(ecuc_module_def.post_build_variant_support(), Some(true));
        ecuc_module_def
            .set_category(Some(EcucModuleDefCategory::VendorSpecificModuleDefinition))
            .unwrap();
        assert_eq!(
            ecuc_module_def.category(),
            Some(EcucModuleDefCategory::VendorSpecificModuleDefinition)
        );
        ecuc_module_def.set_lower_multiplicity(Some(1)).unwrap();
        assert_eq!(ecuc_module_def.lower_multiplicity(), Some(1));
        ecuc_module_def.set_upper_multiplicity(Some(2)).unwrap();
        assert_eq!(ecuc_module_def.upper_multiplicity(), Some(2));
        ecuc_module_def.set_upper_multiplicity_infinite(Some(true)).unwrap();
        assert_eq!(ecuc_module_def.upper_multiplicity_infinite(), Some(true));
    }

    #[test]
    fn ecuc_configuration_variant_enum_conversion() {
        let variants = [
            EcucConfigurationVariant::PreconfiguredConfiguration,
            EcucConfigurationVariant::RecommendedConfiguration,
            EcucConfigurationVariant::VariantLinkTime,
            EcucConfigurationVariant::VariantPostBuild,
            EcucConfigurationVariant::VariantPreCompile,
            EcucConfigurationVariant::VariantPostBuildLoadable,
            EcucConfigurationVariant::VariantPostBuildSelectable,
        ];

        for variant in &variants {
            let enum_item: EnumItem = (*variant).into();
            let converted_variant = EcucConfigurationVariant::try_from(enum_item).unwrap();
            assert_eq!(*variant, converted_variant);
        }
    }

    #[test]
    fn destionation_uri_defs() {
        let model = AutosarModelAbstraction::create("file.arxml", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        let ecuc_destination_uri_def_set = package
            .create_ecuc_destination_uri_def_set("destination_uri_def_set")
            .unwrap();
        assert_eq!(ecuc_destination_uri_def_set.destination_uri_defs().count(), 0);

        let ecuc_destination_uri_def = ecuc_destination_uri_def_set
            .create_destination_uri_def(
                "destination_uri_def",
                EcucDestinationUriNestingContract::LeafOfTargetContainer,
            )
            .unwrap();
        assert_eq!(ecuc_destination_uri_def_set.destination_uri_defs().count(), 1);

        assert_eq!(
            ecuc_destination_uri_def.nesting_contract(),
            Some(EcucDestinationUriNestingContract::LeafOfTargetContainer)
        );
        assert_eq!(ecuc_destination_uri_def.containers().count(), 0);

        let _param_conf_container_def = ecuc_destination_uri_def
            .create_param_conf_container_def("param_conf_container")
            .unwrap();
        assert_eq!(ecuc_destination_uri_def.containers().count(), 1);

        let _choice_container_def = ecuc_destination_uri_def
            .create_choice_container_def("choice_container")
            .unwrap();
        assert_eq!(ecuc_destination_uri_def.containers().count(), 2);
    }
}
