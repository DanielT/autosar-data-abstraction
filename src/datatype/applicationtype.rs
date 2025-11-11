use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, Element, IdentifiableAbstractionElement,
    abstraction_element,
    datatype::{self, DataTypeMap},
    get_reference_parents, is_used,
    software_component::{ArgumentDataPrototype, ParameterDataPrototype, VariableDataPrototype},
};
use autosar_data::{ElementName, EnumItem};
use datatype::{AbstractAutosarDataType, CompuMethod, DataConstr, Unit};

//#########################################################

/// An application array data type
///
/// Use [`ArPackage::create_application_array_data_type`] to create a new application array data type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApplicationArrayDataType(Element);
abstraction_element!(ApplicationArrayDataType, ApplicationArrayDataType);
impl IdentifiableAbstractionElement for ApplicationArrayDataType {}
impl AbstractAutosarDataType for ApplicationArrayDataType {}

impl ApplicationArrayDataType {
    /// create a new application array data type in the given package
    pub(crate) fn new<T: Into<ApplicationDataType> + AbstractionElement>(
        name: &str,
        package: &ArPackage,
        element_type: &T,
        size: ApplicationArraySize,
    ) -> Result<Self, AutosarAbstractionError> {
        let element_type = element_type.clone().into();
        Self::new_internal(name, package, &element_type, size)
    }

    /// remove the application array data type from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        if let Some(array_element) = self.array_element() {
            array_element.remove(deep)?;
        }
        let ref_parents = get_reference_parents(self.element())?;

        AbstractionElement::remove(self, deep)?;

        remove_helper(ref_parents, deep)
    }

    fn new_internal(
        name: &str,
        package: &ArPackage,
        element_type: &ApplicationDataType,
        size: ApplicationArraySize,
    ) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let application_array_data_type =
            elements.create_named_sub_element(ElementName::ApplicationArrayDataType, name)?;

        application_array_data_type
            .create_sub_element(ElementName::Category)?
            .set_character_data("ARRAY")?;

        let application_array_data_type = Self(application_array_data_type);
        ApplicationArrayElement::new("Element", &application_array_data_type, element_type)?;

        // set the size of the array after creating the array element, since some settings must be set in the array element
        application_array_data_type.set_size(size)?;

        Ok(application_array_data_type)
    }

    /// get the array element of the array data type
    #[must_use]
    pub fn array_element(&self) -> Option<ApplicationArrayElement> {
        self.element().get_sub_element(ElementName::Element)?.try_into().ok()
    }

    /// set the size of the array
    pub fn set_size(&self, size: ApplicationArraySize) -> Result<(), AutosarAbstractionError> {
        let array_element = self.array_element().ok_or(AutosarAbstractionError::InvalidParameter(
            "Array data type has no array element".to_string(),
        ))?;
        if let Some(datatype) = array_element.data_type() {
            let is_array_datatype = matches!(datatype, ApplicationDataType::Array(_));
            if is_array_datatype && matches!(size, ApplicationArraySize::VariableLinear(_)) {
                return Err(AutosarAbstractionError::InvalidParameter(
                    "When the size type is VariableLinear, the element type may not be an array".to_string(),
                ));
            } else if !is_array_datatype
                && matches!(
                    size,
                    ApplicationArraySize::VariableSquare
                        | ApplicationArraySize::VariableRectangular(_)
                        | ApplicationArraySize::VariableFullyFlexible(_)
                )
            {
                return Err(AutosarAbstractionError::InvalidParameter(
                    "When the size type is VariableSquare, VariableRectangular or VariableFullyFlexible, the element type must be an array".to_string(),
                ));
            }
        }
        let array_element_elem = array_element.element();
        match size {
            ApplicationArraySize::Fixed(size) => {
                let _ = self
                    .element()
                    .remove_sub_element_kind(ElementName::DynamicArraySizeProfile);
                array_element_elem
                    .get_or_create_sub_element(ElementName::MaxNumberOfElements)?
                    .set_character_data(size)?;
                array_element_elem
                    .get_or_create_sub_element(ElementName::ArraySizeSemantics)?
                    .set_character_data(EnumItem::FixedSize)?;
                let _ = array_element_elem.remove_sub_element_kind(ElementName::ArraySizeHandling);
            }
            ApplicationArraySize::VariableLinear(max_size) => {
                self.element()
                    .get_or_create_sub_element(ElementName::DynamicArraySizeProfile)?
                    .set_character_data("VSA_LINEAR")?;
                array_element_elem
                    .get_or_create_sub_element(ElementName::MaxNumberOfElements)?
                    .set_character_data(max_size)?;
                array_element_elem
                    .get_or_create_sub_element(ElementName::ArraySizeSemantics)?
                    .set_character_data(EnumItem::VariableSize)?;
                array_element_elem
                    .get_or_create_sub_element(ElementName::ArraySizeHandling)?
                    .set_character_data(EnumItem::AllIndicesSameArraySize)?;
            }
            ApplicationArraySize::VariableSquare => {
                self.element()
                    .get_or_create_sub_element(ElementName::DynamicArraySizeProfile)?
                    .set_character_data("VSA_SQUARE")?;
                let _ = array_element_elem.remove_sub_element_kind(ElementName::MaxNumberOfElements);
                array_element_elem
                    .get_or_create_sub_element(ElementName::ArraySizeSemantics)?
                    .set_character_data(EnumItem::VariableSize)?;
                array_element_elem
                    .get_or_create_sub_element(ElementName::ArraySizeHandling)?
                    .set_character_data(EnumItem::InheritedFromArrayElementTypeSize)?;
            }
            ApplicationArraySize::VariableRectangular(max_size) => {
                self.element()
                    .get_or_create_sub_element(ElementName::DynamicArraySizeProfile)?
                    .set_character_data("VSA_RECTANGULAR")?;
                array_element_elem
                    .get_or_create_sub_element(ElementName::MaxNumberOfElements)?
                    .set_character_data(max_size)?;
                array_element_elem
                    .get_or_create_sub_element(ElementName::ArraySizeSemantics)?
                    .set_character_data(EnumItem::VariableSize)?;
                array_element_elem
                    .get_or_create_sub_element(ElementName::ArraySizeHandling)?
                    .set_character_data(EnumItem::AllIndicesSameArraySize)?;
            }
            ApplicationArraySize::VariableFullyFlexible(max_size) => {
                self.element()
                    .get_or_create_sub_element(ElementName::DynamicArraySizeProfile)?
                    .set_character_data("VSA_FULLY_FLEXIBLE")?;
                array_element_elem
                    .get_or_create_sub_element(ElementName::MaxNumberOfElements)?
                    .set_character_data(max_size)?;
                array_element_elem
                    .get_or_create_sub_element(ElementName::ArraySizeSemantics)?
                    .set_character_data(EnumItem::VariableSize)?;
                array_element_elem
                    .get_or_create_sub_element(ElementName::ArraySizeHandling)?
                    .set_character_data(EnumItem::AllIndicesDifferentArraySize)?;
            }
        }

        Ok(())
    }

    /// get the size of the array
    #[must_use]
    pub fn size(&self) -> Option<ApplicationArraySize> {
        let max_number_of_elements = self
            .array_element()?
            .element()
            .get_sub_element(ElementName::MaxNumberOfElements);

        if let Some(size_profile) = self
            .element()
            .get_sub_element(ElementName::DynamicArraySizeProfile)
            .and_then(|elem| elem.character_data().and_then(|cdata| cdata.string_value()))
        {
            match size_profile.as_str() {
                "VSA_LINEAR" => {
                    let max_size = max_number_of_elements?.character_data()?.parse_integer()?;
                    Some(ApplicationArraySize::VariableLinear(max_size))
                }
                "VSA_SQUARE" => Some(ApplicationArraySize::VariableSquare),
                "VSA_RECTANGULAR" => {
                    let max_size = max_number_of_elements?.character_data()?.parse_integer()?;
                    Some(ApplicationArraySize::VariableRectangular(max_size))
                }
                "VSA_FULLY_FLEXIBLE" => {
                    let max_size = max_number_of_elements?.character_data()?.parse_integer()?;
                    Some(ApplicationArraySize::VariableFullyFlexible(max_size))
                }
                _ => None,
            }
        } else {
            let size = max_number_of_elements?.character_data()?.parse_integer()?;
            Some(ApplicationArraySize::Fixed(size))
        }
    }
}

//#########################################################

/// definition of the size type of an application array data type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ApplicationArraySize {
    /// Fixed size array, with the given size
    Fixed(u64),
    /// Variable size array, with a single dimension. The maximum size is given
    VariableLinear(u64),
    /// Variable size "square" array, with two or more dimensions. All dimensions have the same maximum size
    /// This maximum size is set in the innermost dimension; it is not set here.
    /// When the size is set to `VariableSquare`, the array element data type must also be an `ApplicationArrayDataType`
    VariableSquare,
    /// Variable size "rectangular" array, with two or more dimensions. Each dimension has its own maximum size.
    /// The array element data type must also be an `ApplicationArrayDataType`.
    VariableRectangular(u64),
    /// Fully flexible variable size array of arrays. The maximum number of elements of each contained array is not necessarily identical
    /// The array element data type must also be an `ApplicationArrayDataType`.
    VariableFullyFlexible(u64),
}

//#########################################################

/// An element in an application array data type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApplicationArrayElement(Element);
abstraction_element!(ApplicationArrayElement, Element);
impl IdentifiableAbstractionElement for ApplicationArrayElement {}

impl ApplicationArrayElement {
    fn new(
        name: &str,
        parent: &ApplicationArrayDataType,
        data_type: &ApplicationDataType,
    ) -> Result<Self, AutosarAbstractionError> {
        let application_array_element = parent.element().create_named_sub_element(ElementName::Element, name)?;
        let application_array_element = Self(application_array_element);

        application_array_element.set_data_type(data_type)?;

        Ok(application_array_element)
    }

    /// remove the application array element from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        let opt_data_type = self.data_type();

        AbstractionElement::remove(self, deep)?;

        if deep
            && let Some(data_type) = opt_data_type
            && !is_used(data_type.element())
        {
            data_type.remove(deep)?;
        }

        Ok(())
    }

    /// set the data type of the array element
    pub fn set_data_type<T: Into<ApplicationDataType> + AbstractionElement>(
        &self,
        data_type: &T,
    ) -> Result<(), AutosarAbstractionError> {
        let data_type: ApplicationDataType = data_type.clone().into();
        self.element()
            .get_or_create_sub_element(ElementName::TypeTref)?
            .set_reference_target(data_type.element())?;
        // keep the category synced with the data type, as required by [constr_1152]
        if let Some(category) = data_type.category() {
            self.element()
                .get_or_create_sub_element(ElementName::Category)?
                .set_character_data(category)?;
        } else {
            // remove the category if the data type has no category
            let _ = self.element().remove_sub_element_kind(ElementName::Category);
        }

        Ok(())
    }

    /// get the data type of the array element
    #[must_use]
    pub fn data_type(&self) -> Option<ApplicationDataType> {
        self.element()
            .get_sub_element(ElementName::TypeTref)?
            .get_reference_target()
            .ok()?
            .try_into()
            .ok()
    }
}

//#########################################################

/// An application record data type
///
/// Use [`ArPackage::create_application_record_data_type`] to create a new application record data type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApplicationRecordDataType(Element);
abstraction_element!(ApplicationRecordDataType, ApplicationRecordDataType);
impl IdentifiableAbstractionElement for ApplicationRecordDataType {}
impl AbstractAutosarDataType for ApplicationRecordDataType {}

impl ApplicationRecordDataType {
    /// create a new application record data type in the given package
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let application_record_data_type =
            elements.create_named_sub_element(ElementName::ApplicationRecordDataType, name)?;

        application_record_data_type
            .create_sub_element(ElementName::Category)?
            .set_character_data("STRUCTURE")?;

        Ok(Self(application_record_data_type))
    }

    /// remove the application record data type from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        for record_element in self.record_elements() {
            record_element.remove(deep)?;
        }
        let ref_parents = get_reference_parents(self.element())?;

        AbstractionElement::remove(self, deep)?;

        remove_helper(ref_parents, deep)
    }

    /// create a new element in the record data type
    pub fn create_record_element<T: Into<ApplicationDataType> + Clone>(
        &self,
        name: &str,
        data_type: &T,
    ) -> Result<ApplicationRecordElement, AutosarAbstractionError> {
        ApplicationRecordElement::new(name, self, &data_type.clone().into())
    }

    /// iterate over the record elements of the record data type
    pub fn record_elements(&self) -> impl Iterator<Item = ApplicationRecordElement> + Send + use<> {
        self.element()
            .get_sub_element(ElementName::Elements)
            .into_iter()
            .flat_map(|elements| elements.sub_elements())
            .filter_map(|element| ApplicationRecordElement::try_from(element).ok())
    }
}

//#########################################################

/// An element in an application record data type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApplicationRecordElement(Element);
abstraction_element!(ApplicationRecordElement, ApplicationRecordElement);
impl IdentifiableAbstractionElement for ApplicationRecordElement {}

impl ApplicationRecordElement {
    fn new(
        name: &str,
        parent: &ApplicationRecordDataType,
        data_type: &ApplicationDataType,
    ) -> Result<Self, AutosarAbstractionError> {
        let application_record_element = parent
            .element()
            .get_or_create_sub_element(ElementName::Elements)?
            .create_named_sub_element(ElementName::ApplicationRecordElement, name)?;

        let application_record_element = Self(application_record_element);
        application_record_element.set_data_type(data_type)?;

        Ok(application_record_element)
    }

    /// remove the application record element from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        let opt_data_type = self.data_type();

        AbstractionElement::remove(self, deep)?;

        if deep
            && let Some(data_type) = opt_data_type
            && !is_used(data_type.element())
        {
            data_type.remove(deep)?;
        }

        Ok(())
    }

    /// set the data type of the record element
    pub fn set_data_type<T: Into<ApplicationDataType> + AbstractionElement>(
        &self,
        data_type: &T,
    ) -> Result<(), AutosarAbstractionError> {
        let data_type: ApplicationDataType = data_type.clone().into();
        self.element()
            .get_or_create_sub_element(ElementName::TypeTref)?
            .set_reference_target(data_type.element())?;
        if let Some(category) = data_type.category() {
            self.element()
                .get_or_create_sub_element(ElementName::Category)?
                .set_character_data(category)?;
        } else {
            // remove the category if the data type has no category
            let _ = self.element().remove_sub_element_kind(ElementName::Category);
        }

        Ok(())
    }

    /// get the data type of the record element
    #[must_use]
    pub fn data_type(&self) -> Option<ApplicationDataType> {
        self.element()
            .get_sub_element(ElementName::TypeTref)?
            .get_reference_target()
            .ok()?
            .try_into()
            .ok()
    }
}

//#########################################################

/// An application primitive data type
///
/// Use [`ArPackage::create_application_primitive_data_type`] to create a new application primitive data type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApplicationPrimitiveDataType(Element);
abstraction_element!(ApplicationPrimitiveDataType, ApplicationPrimitiveDataType);
impl IdentifiableAbstractionElement for ApplicationPrimitiveDataType {}
impl AbstractAutosarDataType for ApplicationPrimitiveDataType {}

impl ApplicationPrimitiveDataType {
    /// create a new application primitive data type in the given package
    pub(crate) fn new(
        name: &str,
        package: &ArPackage,
        category: ApplicationPrimitiveCategory,
        compu_method: Option<&CompuMethod>,
        unit: Option<&Unit>,
        data_constraint: Option<&DataConstr>,
    ) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let application_primitive_data_type =
            elements.create_named_sub_element(ElementName::ApplicationPrimitiveDataType, name)?;

        let application_primitive_data_type = Self(application_primitive_data_type);

        application_primitive_data_type.set_category(category)?;
        application_primitive_data_type.set_compu_method(compu_method)?;
        application_primitive_data_type.set_unit(unit)?;
        application_primitive_data_type.set_data_constraint(data_constraint)?;

        Ok(application_primitive_data_type)
    }

    /// remove the application primitive data type from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        let opt_compu_method = self.compu_method();
        let opt_unit = self.unit();
        let opt_data_constraint = self.data_constraint();
        let ref_parents = get_reference_parents(self.element())?;

        AbstractionElement::remove(self, deep)?;

        if deep {
            if let Some(compu_method) = opt_compu_method
                && !is_used(compu_method.element())
            {
                compu_method.remove(deep)?;
            }
            if let Some(unit) = opt_unit
                && !is_used(unit.element())
            {
                unit.remove(deep)?;
            }
            if let Some(data_constraint) = opt_data_constraint
                && !is_used(data_constraint.element())
            {
                data_constraint.remove(deep)?;
            }
        }

        remove_helper(ref_parents, deep)
    }

    /// set the category of the primitive data type
    pub fn set_category(&self, category: ApplicationPrimitiveCategory) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::Category)?
            .set_character_data(category.to_string())?;

        Ok(())
    }

    /// get the category of the primitive data type
    #[must_use]
    pub fn category(&self) -> Option<ApplicationPrimitiveCategory> {
        self.element()
            .get_sub_element(ElementName::Category)?
            .character_data()?
            .string_value()?
            .parse()
            .ok()
    }

    /// set the compu method of the primitive data type
    pub fn set_compu_method(&self, compu_method: Option<&CompuMethod>) -> Result<(), AutosarAbstractionError> {
        if let Some(compu_method) = compu_method {
            self.element()
                .get_or_create_sub_element(ElementName::SwDataDefProps)?
                .get_or_create_sub_element(ElementName::SwDataDefPropsVariants)?
                .get_or_create_sub_element(ElementName::SwDataDefPropsConditional)?
                .get_or_create_sub_element(ElementName::CompuMethodRef)?
                .set_reference_target(compu_method.element())?;
        } else {
            let _ = self
                .element()
                .get_sub_element(ElementName::SwDataDefProps)
                .and_then(|sddp| sddp.get_sub_element(ElementName::SwDataDefPropsVariants))
                .and_then(|sddpv| sddpv.get_sub_element(ElementName::SwDataDefPropsConditional))
                .and_then(|sddpc| sddpc.remove_sub_element_kind(ElementName::CompuMethodRef).ok());
        }

        Ok(())
    }

    /// get the compu method of the primitive data type
    #[must_use]
    pub fn compu_method(&self) -> Option<CompuMethod> {
        self.element()
            .get_sub_element(ElementName::SwDataDefProps)?
            .get_sub_element(ElementName::SwDataDefPropsVariants)?
            .get_sub_element(ElementName::SwDataDefPropsConditional)?
            .get_sub_element(ElementName::CompuMethodRef)?
            .get_reference_target()
            .ok()?
            .try_into()
            .ok()
    }

    /// set the unit of the primitive data type
    pub fn set_unit(&self, unit: Option<&Unit>) -> Result<(), AutosarAbstractionError> {
        if let Some(unit) = unit {
            self.element()
                .get_or_create_sub_element(ElementName::SwDataDefProps)?
                .get_or_create_sub_element(ElementName::SwDataDefPropsVariants)?
                .get_or_create_sub_element(ElementName::SwDataDefPropsConditional)?
                .get_or_create_sub_element(ElementName::UnitRef)?
                .set_reference_target(unit.element())?;
        } else {
            let _ = self
                .element()
                .get_sub_element(ElementName::SwDataDefProps)
                .and_then(|sddp| sddp.get_sub_element(ElementName::SwDataDefPropsVariants))
                .and_then(|sddpv| sddpv.get_sub_element(ElementName::SwDataDefPropsConditional))
                .and_then(|sddpc| sddpc.remove_sub_element_kind(ElementName::UnitRef).ok());
        }

        Ok(())
    }

    /// get the unit of the primitive data type
    #[must_use]
    pub fn unit(&self) -> Option<Unit> {
        self.element()
            .get_sub_element(ElementName::SwDataDefProps)?
            .get_sub_element(ElementName::SwDataDefPropsVariants)?
            .get_sub_element(ElementName::SwDataDefPropsConditional)?
            .get_sub_element(ElementName::UnitRef)?
            .get_reference_target()
            .ok()?
            .try_into()
            .ok()
    }

    /// set the data constraint of the primitive data type
    pub fn set_data_constraint(&self, data_constraint: Option<&DataConstr>) -> Result<(), AutosarAbstractionError> {
        if let Some(data_constraint) = data_constraint {
            self.element()
                .get_or_create_sub_element(ElementName::SwDataDefProps)?
                .get_or_create_sub_element(ElementName::SwDataDefPropsVariants)?
                .get_or_create_sub_element(ElementName::SwDataDefPropsConditional)?
                .get_or_create_sub_element(ElementName::DataConstrRef)?
                .set_reference_target(data_constraint.element())?;
        } else {
            let _ = self
                .element()
                .get_sub_element(ElementName::SwDataDefProps)
                .and_then(|sddp| sddp.get_sub_element(ElementName::SwDataDefPropsVariants))
                .and_then(|sddpv| sddpv.get_sub_element(ElementName::SwDataDefPropsConditional))
                .and_then(|sddpc| sddpc.remove_sub_element_kind(ElementName::DataConstrRef).ok());
        }

        Ok(())
    }

    /// get the data constraint of the primitive data type
    #[must_use]
    pub fn data_constraint(&self) -> Option<DataConstr> {
        self.element()
            .get_sub_element(ElementName::SwDataDefProps)?
            .get_sub_element(ElementName::SwDataDefPropsVariants)?
            .get_sub_element(ElementName::SwDataDefPropsConditional)?
            .get_sub_element(ElementName::DataConstrRef)?
            .get_reference_target()
            .ok()?
            .try_into()
            .ok()
    }
}

//#########################################################

/// The category of an application primitive data type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApplicationPrimitiveCategory {
    /// Value
    Value,
    /// Value block
    ValBlk,
    /// String
    String,
    /// Boolean
    Boolean,
    /// Common axis
    ComAxis,
    /// Rescale axis
    ResAxis,
    /// Curve - 1D array with an axis
    Curve,
    /// Map - 2D array with two axes
    Map,
    /// Cuboid - 3D array with three axes
    Cuboid,
    /// Cube4 - 4D array with four axes
    Cube4,
    /// Cube5 - 5D array with five axes
    Cube5,
}

impl std::fmt::Display for ApplicationPrimitiveCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ApplicationPrimitiveCategory::Value => f.write_str("VALUE"),
            ApplicationPrimitiveCategory::ValBlk => f.write_str("VAL_BLK"),
            ApplicationPrimitiveCategory::String => f.write_str("STRING"),
            ApplicationPrimitiveCategory::Boolean => f.write_str("BOOLEAN"),
            ApplicationPrimitiveCategory::ComAxis => f.write_str("COM_AXIS"),
            ApplicationPrimitiveCategory::ResAxis => f.write_str("RES_AXIS"),
            ApplicationPrimitiveCategory::Curve => f.write_str("CURVE"),
            ApplicationPrimitiveCategory::Map => f.write_str("MAP"),
            ApplicationPrimitiveCategory::Cuboid => f.write_str("CUBOID"),
            ApplicationPrimitiveCategory::Cube4 => f.write_str("CUBE_4"),
            ApplicationPrimitiveCategory::Cube5 => f.write_str("CUBE_5"),
        }
    }
}

impl std::str::FromStr for ApplicationPrimitiveCategory {
    type Err = AutosarAbstractionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "VALUE" => Ok(ApplicationPrimitiveCategory::Value),
            "VAL_BLK" => Ok(ApplicationPrimitiveCategory::ValBlk),
            "STRING" => Ok(ApplicationPrimitiveCategory::String),
            "BOOLEAN" => Ok(ApplicationPrimitiveCategory::Boolean),
            "COM_AXIS" => Ok(ApplicationPrimitiveCategory::ComAxis),
            "RES_AXIS" => Ok(ApplicationPrimitiveCategory::ResAxis),
            "CURVE" => Ok(ApplicationPrimitiveCategory::Curve),
            "MAP" => Ok(ApplicationPrimitiveCategory::Map),
            "CUBOID" => Ok(ApplicationPrimitiveCategory::Cuboid),
            "CUBE_4" => Ok(ApplicationPrimitiveCategory::Cube4),
            "CUBE_5" => Ok(ApplicationPrimitiveCategory::Cube5),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: s.to_string(),
                dest: "ApplicationPrimitiveCategory".to_string(),
            }),
        }
    }
}

//#########################################################

/// A wrapper for all application data types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ApplicationDataType {
    /// An array data type
    Array(ApplicationArrayDataType),
    /// A record data type
    Record(ApplicationRecordDataType),
    /// A primitive data type
    Primitive(ApplicationPrimitiveDataType),
}

impl AbstractionElement for ApplicationDataType {
    fn element(&self) -> &Element {
        match self {
            ApplicationDataType::Array(e) => e.element(),
            ApplicationDataType::Record(e) => e.element(),
            ApplicationDataType::Primitive(e) => e.element(),
        }
    }
}

impl TryFrom<Element> for ApplicationDataType {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::ApplicationArrayDataType => Ok(ApplicationDataType::Array(element.try_into()?)),
            ElementName::ApplicationRecordDataType => Ok(ApplicationDataType::Record(element.try_into()?)),
            ElementName::ApplicationPrimitiveDataType => Ok(ApplicationDataType::Primitive(element.try_into()?)),
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "ApplicationDataType".to_string(),
            }),
        }
    }
}

impl IdentifiableAbstractionElement for ApplicationDataType {}

impl From<ApplicationPrimitiveDataType> for ApplicationDataType {
    fn from(val: ApplicationPrimitiveDataType) -> Self {
        ApplicationDataType::Primitive(val)
    }
}

impl From<ApplicationRecordDataType> for ApplicationDataType {
    fn from(val: ApplicationRecordDataType) -> Self {
        ApplicationDataType::Record(val)
    }
}

impl From<ApplicationArrayDataType> for ApplicationDataType {
    fn from(val: ApplicationArrayDataType) -> Self {
        ApplicationDataType::Array(val)
    }
}

impl ApplicationDataType {
    /// get the category of the data type
    #[must_use]
    pub fn category(&self) -> Option<String> {
        self.element()
            .get_sub_element(ElementName::Category)?
            .character_data()?
            .string_value()
    }

    /// remove the application record data type from the model
    pub fn remove(self, deep: bool) -> Result<(), AutosarAbstractionError> {
        match self {
            ApplicationDataType::Array(e) => e.remove(deep),
            ApplicationDataType::Record(e) => e.remove(deep),
            ApplicationDataType::Primitive(e) => e.remove(deep),
        }
    }
}

//#########################################################

fn remove_helper(ref_parents: Vec<(Element, Element)>, deep: bool) -> Result<(), AutosarAbstractionError> {
    for (named_parent, parent) in ref_parents {
        match named_parent.element_name() {
            ElementName::Element => {
                if let Ok(app_data_type_ref) = ApplicationArrayElement::try_from(named_parent) {
                    app_data_type_ref.remove(deep)?;
                }
            }
            ElementName::ApplicationRecordElement => {
                if let Ok(app_data_type_ref) = ApplicationRecordElement::try_from(named_parent) {
                    app_data_type_ref.remove(deep)?;
                }
            }
            ElementName::DataTypeMappingSet => {
                // don't remove the whole mapping set, only the mapping
                if let Ok(data_type_map) = DataTypeMap::try_from(parent) {
                    data_type_map.remove(deep)?;
                }
            }
            ElementName::ParameterDataPrototype => {
                if let Ok(param_prototype) = ParameterDataPrototype::try_from(named_parent) {
                    param_prototype.remove(deep)?;
                }
            }
            ElementName::VariableDataPrototype => {
                if let Ok(var_data_prototype) = VariableDataPrototype::try_from(parent) {
                    var_data_prototype.remove(deep)?;
                }
            }
            ElementName::ArgumentDataPrototype => {
                if let Ok(arg_data_prototype) = ArgumentDataPrototype::try_from(parent) {
                    arg_data_prototype.remove(deep)?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

//#########################################################

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AutosarModelAbstraction,
        datatype::{BaseTypeEncoding, ImplementationDataTypeSettings},
        software_component::ArgumentDirection,
    };
    use autosar_data::AutosarVersion;
    use datatype::{CompuMethodContent, CompuMethodLinearContent};

    #[test]
    fn test_application_array_data_type() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/DataTypes").unwrap();
        let element_type = ApplicationPrimitiveDataType::new(
            "Element",
            &package,
            ApplicationPrimitiveCategory::Value,
            None,
            None,
            None,
        )
        .unwrap();
        let array_data_type =
            ApplicationArrayDataType::new("Array", &package, &element_type, ApplicationArraySize::Fixed(10)).unwrap();
        let array_type_2 =
            ApplicationArrayDataType::new("Array2", &package, &element_type, ApplicationArraySize::Fixed(100)).unwrap();

        assert_eq!(
            array_data_type.array_element().unwrap().data_type().unwrap(),
            ApplicationDataType::Primitive(element_type)
        );
        assert_eq!(array_data_type.size().unwrap(), ApplicationArraySize::Fixed(10));

        array_data_type
            .set_size(ApplicationArraySize::VariableLinear(100))
            .unwrap();
        assert_eq!(
            array_data_type.size().unwrap(),
            ApplicationArraySize::VariableLinear(100)
        );

        // the inner type must be an array type for the following size settings
        let result = array_data_type.set_size(ApplicationArraySize::VariableSquare);
        assert!(result.is_err());
        let result = array_data_type.set_size(ApplicationArraySize::VariableRectangular(100));
        assert!(result.is_err());
        let result = array_data_type.set_size(ApplicationArraySize::VariableFullyFlexible(100));
        assert!(result.is_err());

        // reassign the array element type to an array type
        array_data_type
            .array_element()
            .unwrap()
            .set_data_type(&array_type_2)
            .unwrap();
        array_data_type.set_size(ApplicationArraySize::VariableSquare).unwrap();
        assert_eq!(array_data_type.size().unwrap(), ApplicationArraySize::VariableSquare);
        array_data_type
            .set_size(ApplicationArraySize::VariableRectangular(100))
            .unwrap();
        assert_eq!(
            array_data_type.size().unwrap(),
            ApplicationArraySize::VariableRectangular(100)
        );
        array_data_type
            .set_size(ApplicationArraySize::VariableFullyFlexible(100))
            .unwrap();
        assert_eq!(
            array_data_type.size().unwrap(),
            ApplicationArraySize::VariableFullyFlexible(100)
        );

        // reassign the array element type
        let element_type_2 = ApplicationPrimitiveDataType::new(
            "Element2",
            &package,
            ApplicationPrimitiveCategory::Value,
            None,
            None,
            None,
        )
        .unwrap();
        let array_element = array_data_type.array_element().unwrap();
        array_element.set_data_type(&element_type_2).unwrap();
        assert_eq!(
            array_element.data_type().unwrap(),
            ApplicationDataType::Primitive(element_type_2)
        );
    }

    #[test]
    fn test_application_record_data_type() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/DataTypes").unwrap();
        let record_data_type = ApplicationRecordDataType::new("Record", &package).unwrap();
        let element_type = ApplicationPrimitiveDataType::new(
            "Element",
            &package,
            ApplicationPrimitiveCategory::Value,
            None,
            None,
            None,
        )
        .unwrap();
        let record_element = record_data_type
            .create_record_element("Element", &element_type)
            .unwrap();

        assert_eq!(
            record_element.data_type().unwrap(),
            ApplicationDataType::Primitive(element_type)
        );
        assert_eq!(record_data_type.record_elements().next().unwrap(), record_element);
    }

    #[test]
    fn test_application_primitive_data_type() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/DataTypes").unwrap();
        let compu_method = CompuMethod::new(
            "CompuMethod",
            &package,
            CompuMethodContent::Linear(CompuMethodLinearContent {
                direction: datatype::CompuScaleDirection::IntToPhys,
                offset: 0.0,
                factor: 100.0,
                divisor: 1.0,
                lower_limit: None,
                upper_limit: None,
            }),
        )
        .unwrap();
        let unit = Unit::new("Unit", &package, Some("Unit name")).unwrap();
        let data_constraint = DataConstr::new("DataConstraint", &package).unwrap();
        let primitive_data_type = ApplicationPrimitiveDataType::new(
            "Primitive",
            &package,
            ApplicationPrimitiveCategory::Value,
            Some(&compu_method),
            Some(&unit),
            Some(&data_constraint),
        )
        .unwrap();

        assert_eq!(
            primitive_data_type.category().unwrap(),
            ApplicationPrimitiveCategory::Value
        );
        assert_eq!(primitive_data_type.compu_method().unwrap(), compu_method);
        assert_eq!(primitive_data_type.unit().unwrap(), unit);
        assert_eq!(primitive_data_type.data_constraint().unwrap(), data_constraint);

        primitive_data_type
            .set_category(ApplicationPrimitiveCategory::Boolean)
            .unwrap();
        assert_eq!(
            primitive_data_type.category().unwrap(),
            ApplicationPrimitiveCategory::Boolean
        );
        primitive_data_type.set_compu_method(None).unwrap();
        assert!(primitive_data_type.compu_method().is_none());
        primitive_data_type.set_unit(None).unwrap();
        assert!(primitive_data_type.unit().is_none());
        primitive_data_type.set_data_constraint(None).unwrap();
        assert!(primitive_data_type.data_constraint().is_none());
    }

    #[test]
    fn test_application_data_type() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/DataTypes").unwrap();
        let element_type = ApplicationPrimitiveDataType::new(
            "Element",
            &package,
            ApplicationPrimitiveCategory::Value,
            None,
            None,
            None,
        )
        .unwrap();
        let array_data_type =
            ApplicationArrayDataType::new("Array", &package, &element_type, ApplicationArraySize::Fixed(10)).unwrap();
        let record_data_type = ApplicationRecordDataType::new("Record", &package).unwrap();
        let primitive_data_type = ApplicationPrimitiveDataType::new(
            "Primitive",
            &package,
            ApplicationPrimitiveCategory::Value,
            None,
            None,
            None,
        )
        .unwrap();

        let data_type: ApplicationDataType = array_data_type.clone().into();
        assert_eq!(data_type, ApplicationDataType::Array(array_data_type.clone()));
        assert_eq!(data_type.category().unwrap(), "ARRAY");

        let data_type: ApplicationDataType = record_data_type.clone().into();
        assert_eq!(data_type, ApplicationDataType::Record(record_data_type.clone()));
        assert_eq!(data_type.category().unwrap(), "STRUCTURE");

        let data_type: ApplicationDataType = primitive_data_type.clone().into();
        assert_eq!(data_type, ApplicationDataType::Primitive(primitive_data_type.clone()));
        assert_eq!(data_type.category().unwrap(), "VALUE");

        let result = ApplicationDataType::try_from(package.element().clone());
        assert!(result.is_err());
    }

    #[test]
    fn remove() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/DataTypes").unwrap();
        let element_type = package
            .create_application_primitive_data_type(
                "AppPrimitive",
                ApplicationPrimitiveCategory::Value,
                None,
                None,
                None,
            )
            .unwrap();
        let array_data_type = package
            .create_application_array_data_type("AppArray", &element_type, ApplicationArraySize::Fixed(10))
            .unwrap();

        // create a matching implementation data type
        let base_type = package
            .create_sw_base_type("uint8", 8, BaseTypeEncoding::TwosComplement, None, None, Some("uint8"))
            .unwrap();
        let impl_array = package
            .create_implementation_data_type(&ImplementationDataTypeSettings::Array {
                name: "ImplArray".to_string(),
                length: 10,
                element_type: Box::new(ImplementationDataTypeSettings::Value {
                    name: "ImplPrimitive".to_string(),
                    base_type,
                    compu_method: None,
                    data_constraint: None,
                }),
            })
            .unwrap();

        // create a data type mapping that maps the implementation array to the application array
        let data_type_mapping_set = package.create_data_type_mapping_set("DataTypeMappingSet").unwrap();
        data_type_mapping_set
            .create_data_type_map(&impl_array, &array_data_type)
            .unwrap();

        // create a SenderReceiverInterface that uses the application array data type
        let sr_interface = package.create_sender_receiver_interface("SRInterface").unwrap();
        let _vdp = sr_interface.create_data_element("VDP", &array_data_type).unwrap();
        // create a client-server interface that uses the application array data type
        let cs_interface = package.create_client_server_interface("CSInterface").unwrap();
        let cso = cs_interface.create_operation("ADP").unwrap();
        let _adp = cso
            .create_argument("ADP", &array_data_type, ArgumentDirection::In)
            .unwrap();

        // create an application record data type that uses the application array data type
        let record_data_type = package.create_application_record_data_type("AppRecord").unwrap();
        let _record_element = record_data_type
            .create_record_element("RecordElement", &array_data_type)
            .unwrap();

        // remove the application array data type deeply
        array_data_type.remove(true).unwrap();

        // check that all related elements have been removed
        assert_eq!(data_type_mapping_set.data_type_maps().count(), 0);
        assert_eq!(sr_interface.data_elements().count(), 0);
        assert_eq!(cso.arguments().count(), 0);
        assert_eq!(record_data_type.record_elements().count(), 0);
    }
}
