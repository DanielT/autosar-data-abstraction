use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, IdentifiableAbstractionElement, abstraction_element,
};
use autosar_data::{AttributeName, Element, ElementName};

//#########################################################

/// A `CompuMethod` describes the conversion between physical and internal values
///
/// Use [`ArPackage::create_compu_method`] to create a new `CompuMethod`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompuMethod(Element);
abstraction_element!(CompuMethod, CompuMethod);
impl IdentifiableAbstractionElement for CompuMethod {}

impl CompuMethod {
    /// Create a new `CompuMethod`
    pub(crate) fn new(
        name: &str,
        package: &ArPackage,
        content: CompuMethodContent,
    ) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let compu_method = elements.create_named_sub_element(ElementName::CompuMethod, name)?;

        let compu_method = Self(compu_method);
        compu_method.set_content(content)?;

        Ok(compu_method)
    }

    /// Get the category of the `CompuMethod`
    #[must_use]
    pub fn category(&self) -> Option<CompuMethodCategory> {
        let category = self
            .element()
            .get_sub_element(ElementName::Category)?
            .character_data()?
            .string_value()?;

        CompuMethodCategory::try_from(category.as_str()).ok()
    }

    /// Apply `CompumethodContent` to the `CompuMethod`
    ///
    /// This will remove any existing content
    pub fn set_content(&self, content: CompuMethodContent) -> Result<(), AutosarAbstractionError> {
        let compu_method = self.element();

        // remove existing content
        let _ = compu_method.remove_sub_element_kind(ElementName::CompuInternalToPhys);
        let _ = compu_method.remove_sub_element_kind(ElementName::CompuPhysToInternal);

        match content {
            CompuMethodContent::Identical => {
                compu_method
                    .get_or_create_sub_element(ElementName::Category)?
                    .set_character_data("IDENTICAL")?;
            }
            CompuMethodContent::Linear(linear_content) => {
                compu_method
                    .get_or_create_sub_element(ElementName::Category)?
                    .set_character_data("LINEAR")?;

                let compu_scale = self.create_compu_scale(
                    linear_content.direction,
                    linear_content.lower_limit,
                    linear_content.upper_limit,
                )?;
                compu_scale.set_content(CompuScaleContent::RationalCoeffs {
                    numerator: vec![linear_content.offset, linear_content.factor],
                    denominator: vec![linear_content.divisor],
                })?;
            }
            CompuMethodContent::ScaleLinear(scale_linear_content) => {
                compu_method
                    .get_or_create_sub_element(ElementName::Category)?
                    .set_character_data("SCALE_LINEAR")?;

                for scale_linear in scale_linear_content {
                    let compu_scale = self.create_compu_scale(
                        scale_linear.direction,
                        Some(scale_linear.lower_limit),
                        Some(scale_linear.upper_limit),
                    )?;
                    compu_scale.set_content(CompuScaleContent::RationalCoeffs {
                        numerator: vec![scale_linear.offset, scale_linear.factor],
                        denominator: vec![scale_linear.divisor],
                    })?;
                }
            }
            CompuMethodContent::Rational(rational_content) => {
                compu_method
                    .get_or_create_sub_element(ElementName::Category)?
                    .set_character_data("RAT_FUNC")?;

                let compu_scale = self.create_compu_scale(
                    rational_content.direction,
                    Some(rational_content.lower_limit),
                    Some(rational_content.upper_limit),
                )?;

                compu_scale.set_content(CompuScaleContent::RationalCoeffs {
                    numerator: rational_content.numerator,
                    denominator: rational_content.denominator,
                })?;
            }
            CompuMethodContent::ScaleRational(scale_rational_content) => {
                compu_method
                    .get_or_create_sub_element(ElementName::Category)?
                    .set_character_data("SCALE_RAT_FUNC")?;

                for scale_rational in scale_rational_content {
                    let compu_scale = self.create_compu_scale(
                        scale_rational.direction,
                        Some(scale_rational.lower_limit),
                        Some(scale_rational.upper_limit),
                    )?;

                    compu_scale.set_content(CompuScaleContent::RationalCoeffs {
                        numerator: scale_rational.numerator,
                        denominator: scale_rational.denominator,
                    })?;
                }
            }
            CompuMethodContent::TextTable(text_table_content) => {
                compu_method
                    .get_or_create_sub_element(ElementName::Category)?
                    .set_character_data("TEXTTABLE")?;

                for text_table_item in text_table_content {
                    let compu_scale = self.create_compu_scale(
                        CompuScaleDirection::IntToPhys,
                        Some(text_table_item.value),
                        Some(text_table_item.value),
                    )?;
                    compu_scale.set_content(CompuScaleContent::TextConstant(text_table_item.text))?;
                }
            }
            CompuMethodContent::ScaleLinearAndTextTable(scale_linear_content, text_table_content) => {
                compu_method
                    .get_or_create_sub_element(ElementName::Category)?
                    .set_character_data("SCALE_LINEAR_AND_TEXTTABLE")?;

                for scale_linear in scale_linear_content {
                    let compu_scale = self.create_compu_scale(
                        scale_linear.direction,
                        Some(scale_linear.lower_limit),
                        Some(scale_linear.upper_limit),
                    )?;

                    compu_scale.set_content(CompuScaleContent::RationalCoeffs {
                        numerator: vec![scale_linear.offset, scale_linear.factor],
                        denominator: vec![scale_linear.divisor],
                    })?;
                }

                for text_table_item in text_table_content {
                    let compu_scale = self.create_compu_scale(
                        CompuScaleDirection::IntToPhys,
                        Some(text_table_item.value),
                        Some(text_table_item.value),
                    )?;

                    compu_scale.set_content(CompuScaleContent::TextConstant(text_table_item.text))?;
                }
            }
            CompuMethodContent::ScaleRationalAndTextTable(scale_rational_content, text_table_content) => {
                compu_method
                    .get_or_create_sub_element(ElementName::Category)?
                    .set_character_data("SCALE_RATIONAL_AND_TEXTTABLE")?;

                for scale_rational in scale_rational_content {
                    let compu_scale = self.create_compu_scale(
                        scale_rational.direction,
                        Some(scale_rational.lower_limit),
                        Some(scale_rational.upper_limit),
                    )?;

                    compu_scale.set_content(CompuScaleContent::RationalCoeffs {
                        numerator: scale_rational.numerator,
                        denominator: scale_rational.denominator,
                    })?;
                }

                for text_table_item in text_table_content {
                    let compu_scale = self.create_compu_scale(
                        CompuScaleDirection::IntToPhys,
                        Some(text_table_item.value),
                        Some(text_table_item.value),
                    )?;

                    compu_scale.set_content(CompuScaleContent::TextConstant(text_table_item.text))?;
                }
            }
            CompuMethodContent::BitfieldTextTable(bitfield_text_table_content) => {
                compu_method
                    .get_or_create_sub_element(ElementName::Category)?
                    .set_character_data("BITFIELD_TEXTTABLE")?;

                for bitfield_text_table_item in bitfield_text_table_content {
                    let compu_scale = self.create_compu_scale(
                        CompuScaleDirection::IntToPhys,
                        Some(bitfield_text_table_item.value),
                        Some(bitfield_text_table_item.value),
                    )?;

                    compu_scale.set_content(CompuScaleContent::TextConstant(bitfield_text_table_item.text))?;
                    compu_scale.set_mask(bitfield_text_table_item.mask)?;
                }
            }
            CompuMethodContent::TabNoInterpretation(tab_content) => {
                compu_method
                    .get_or_create_sub_element(ElementName::Category)?
                    .set_character_data("TAB_NOINTP")?;

                for tab_item in tab_content {
                    let compu_scale = self.create_compu_scale(
                        CompuScaleDirection::IntToPhys,
                        Some(tab_item.value_in),
                        Some(tab_item.value_in),
                    )?;

                    compu_scale.set_content(CompuScaleContent::NumericConstant(tab_item.value_out))?;
                }
            }
        }

        Ok(())
    }

    /// get the content of the `CompuMethod`
    #[must_use]
    pub fn content(&self) -> Option<CompuMethodContent> {
        let compu_method = self.element();

        let category = compu_method
            .get_sub_element(ElementName::Category)?
            .character_data()?
            .string_value()?;
        let category = CompuMethodCategory::try_from(category.as_str()).ok()?;

        match category {
            CompuMethodCategory::Identical => Some(CompuMethodContent::Identical),
            CompuMethodCategory::Linear => {
                let int_to_phys = compu_method.get_sub_element(ElementName::CompuInternalToPhys);
                let phys_to_int = compu_method.get_sub_element(ElementName::CompuPhysToInternal);

                let (direction, dir_elem) = match (int_to_phys, phys_to_int) {
                    (Some(int_to_phys), None) => (CompuScaleDirection::IntToPhys, int_to_phys),
                    (None, Some(phys_to_int)) => (CompuScaleDirection::PhysToInt, phys_to_int),
                    _ => return None,
                };
                let compu_scale_elem = dir_elem
                    .get_sub_element(ElementName::CompuScales)?
                    .get_sub_element(ElementName::CompuScale)?;
                let compu_scale = CompuScale::try_from(compu_scale_elem).ok()?;
                let content = compu_scale.content()?;
                if let CompuScaleContent::RationalCoeffs { numerator, denominator } = content {
                    let offset = numerator[0];
                    let factor = numerator[1];
                    let divisor = denominator[0];
                    let lower_limit = compu_scale.lower_limit();
                    let upper_limit = compu_scale.upper_limit();

                    Some(CompuMethodContent::Linear(CompuMethodLinearContent {
                        direction,
                        offset,
                        factor,
                        divisor,
                        lower_limit,
                        upper_limit,
                    }))
                } else {
                    None
                }
            }
            CompuMethodCategory::ScaleLinear => {
                let mut scale_linear_content = Vec::new();
                let iter_int_to_phys = self
                    .int_to_phys_compu_scales()
                    .map(|cs| (cs, CompuScaleDirection::IntToPhys));
                let iter_phys_to_int = self
                    .phys_to_int_compu_scales()
                    .map(|cs| (cs, CompuScaleDirection::PhysToInt));
                let iter = iter_int_to_phys.chain(iter_phys_to_int);

                for (compu_scale, direction) in iter {
                    let lower_limit = compu_scale.lower_limit()?;
                    let upper_limit = compu_scale.upper_limit()?;
                    let content = compu_scale.content()?;
                    if let CompuScaleContent::RationalCoeffs { numerator, denominator } = content {
                        let offset = numerator[0];
                        let factor = numerator[1];
                        let divisor = denominator[0];

                        scale_linear_content.push(CompuMethodScaleLinearContent {
                            direction,
                            offset,
                            factor,
                            divisor,
                            lower_limit,
                            upper_limit,
                        });
                    }
                }

                Some(CompuMethodContent::ScaleLinear(scale_linear_content))
            }
            CompuMethodCategory::Rational => {
                let int_to_phys = compu_method.get_sub_element(ElementName::CompuInternalToPhys);
                let phys_to_int = compu_method.get_sub_element(ElementName::CompuPhysToInternal);

                let (direction, dir_elem) = match (int_to_phys, phys_to_int) {
                    (Some(int_to_phys), None) => (CompuScaleDirection::IntToPhys, int_to_phys),
                    (None, Some(phys_to_int)) => (CompuScaleDirection::PhysToInt, phys_to_int),
                    _ => return None,
                };
                let compu_scale_elem = dir_elem
                    .get_sub_element(ElementName::CompuScales)?
                    .get_sub_element(ElementName::CompuScale)?;
                let compu_scale = CompuScale::try_from(compu_scale_elem).ok()?;
                let lower_limit = compu_scale.lower_limit()?;
                let upper_limit = compu_scale.upper_limit()?;
                let content = compu_scale.content()?;
                if let CompuScaleContent::RationalCoeffs { numerator, denominator } = content {
                    Some(CompuMethodContent::Rational(CompuMethodRationalContent {
                        direction,
                        denominator,
                        numerator,
                        lower_limit,
                        upper_limit,
                    }))
                } else {
                    None
                }
            }
            CompuMethodCategory::ScaleRational => {
                let mut scale_rational_content = Vec::new();
                let iter_int_to_phys = self
                    .int_to_phys_compu_scales()
                    .map(|cs| (cs, CompuScaleDirection::IntToPhys));
                let iter_phys_to_int = self
                    .phys_to_int_compu_scales()
                    .map(|cs| (cs, CompuScaleDirection::PhysToInt));
                let iter = iter_int_to_phys.chain(iter_phys_to_int);

                for (compu_scale, direction) in iter {
                    let lower_limit = compu_scale.lower_limit()?;
                    let upper_limit = compu_scale.upper_limit()?;
                    let content = compu_scale.content()?;
                    if let CompuScaleContent::RationalCoeffs { numerator, denominator } = content {
                        scale_rational_content.push(CompuMethodRationalContent {
                            direction,
                            denominator,
                            numerator,
                            lower_limit,
                            upper_limit,
                        });
                    }
                }

                Some(CompuMethodContent::ScaleRational(scale_rational_content))
            }
            CompuMethodCategory::TextTable => {
                let mut text_table_content = Vec::new();
                let iter = self.int_to_phys_compu_scales();

                for compu_scale in iter {
                    let value = compu_scale.lower_limit()?;
                    let content = compu_scale.content()?;
                    if let CompuScaleContent::TextConstant(text) = content {
                        text_table_content.push(CompuMethodTextTableContent { text, value });
                    }
                }

                Some(CompuMethodContent::TextTable(text_table_content))
            }
            CompuMethodCategory::ScaleLinearAndTextTable => {
                let mut scale_linear_content = Vec::new();
                let mut text_table_content = Vec::new();
                let iter = self
                    .int_to_phys_compu_scales()
                    .map(|cs| (cs, CompuScaleDirection::IntToPhys));

                for (compu_scale, direction) in iter {
                    let lower_limit = compu_scale.lower_limit()?;
                    let upper_limit = compu_scale.upper_limit()?;
                    let content = compu_scale.content()?;
                    if let CompuScaleContent::RationalCoeffs { numerator, denominator } = content {
                        let offset = numerator[0];
                        let factor = numerator[1];
                        let divisor = denominator[0];

                        scale_linear_content.push(CompuMethodScaleLinearContent {
                            direction,
                            offset,
                            factor,
                            divisor,
                            lower_limit,
                            upper_limit,
                        });
                    } else if let CompuScaleContent::TextConstant(text) = content {
                        text_table_content.push(CompuMethodTextTableContent {
                            text,
                            value: lower_limit,
                        });
                    }
                }

                Some(CompuMethodContent::ScaleLinearAndTextTable(
                    scale_linear_content,
                    text_table_content,
                ))
            }
            CompuMethodCategory::ScaleRationalAndTextTable => {
                let mut scale_rational_content = Vec::new();
                let mut text_table_content = Vec::new();
                let iter_int_to_phys = self
                    .int_to_phys_compu_scales()
                    .map(|cs| (cs, CompuScaleDirection::IntToPhys));
                let iter_phys_to_int = self
                    .phys_to_int_compu_scales()
                    .map(|cs| (cs, CompuScaleDirection::PhysToInt));
                let iter = iter_int_to_phys.chain(iter_phys_to_int);

                for (compu_scale, direction) in iter {
                    let lower_limit = compu_scale.lower_limit()?;
                    let upper_limit = compu_scale.upper_limit()?;
                    let content = compu_scale.content()?;
                    if let CompuScaleContent::RationalCoeffs { numerator, denominator } = content {
                        scale_rational_content.push(CompuMethodRationalContent {
                            direction,
                            denominator,
                            numerator,
                            lower_limit,
                            upper_limit,
                        });
                    } else if let CompuScaleContent::TextConstant(text) = content {
                        text_table_content.push(CompuMethodTextTableContent {
                            text,
                            value: lower_limit,
                        });
                    }
                }

                Some(CompuMethodContent::ScaleRationalAndTextTable(
                    scale_rational_content,
                    text_table_content,
                ))
            }
            CompuMethodCategory::BitfieldTextTable => {
                let mut bitfield_text_table_content = Vec::new();
                let iter = self.int_to_phys_compu_scales();

                for compu_scale in iter {
                    let value = compu_scale.lower_limit()?;
                    let mask = compu_scale.mask()?;
                    let content = compu_scale.content()?;
                    if let CompuScaleContent::TextConstant(text) = content {
                        bitfield_text_table_content.push(CompuMethodBitfieldTextTableContent { text, value, mask });
                    }
                }

                Some(CompuMethodContent::BitfieldTextTable(bitfield_text_table_content))
            }
            CompuMethodCategory::TabNoInterpretation => {
                let mut tab_content = Vec::new();
                let iter = self.int_to_phys_compu_scales();

                for compu_scale in iter {
                    let value_in = compu_scale.lower_limit()?;
                    if let CompuScaleContent::NumericConstant(value_out) = compu_scale.content()? {
                        tab_content.push(CompuMethodTabNoIntpContent { value_in, value_out });
                    }
                }

                Some(CompuMethodContent::TabNoInterpretation(tab_content))
            }
        }
    }

    /// create a `CompuScale` in the `CompuMethod`
    pub fn create_compu_scale(
        &self,
        direction: CompuScaleDirection,
        lower_limit: Option<f64>,
        upper_limit: Option<f64>,
    ) -> Result<CompuScale, AutosarAbstractionError> {
        let category = self.category().ok_or(AutosarAbstractionError::InvalidParameter(
            "CompuMethod category not set".to_string(),
        ))?;

        let c1 = self.int_to_phys_compu_scales().count();
        let c2 = self.phys_to_int_compu_scales().count();
        match category {
            CompuMethodCategory::Identical => {
                return Err(AutosarAbstractionError::InvalidParameter(
                    "CompuMethod category IDENTICAL does not use CompuScales".to_string(),
                ));
            }
            CompuMethodCategory::Linear => {
                if c1 > 0 || c2 > 0 {
                    return Err(AutosarAbstractionError::InvalidParameter(
                        "CompuMethod category LINEAR may only use one CompuScale, but it already exists".to_string(),
                    ));
                }
            }
            CompuMethodCategory::ScaleLinear => {
                if (direction == CompuScaleDirection::IntToPhys && c2 > 0)
                    || (direction == CompuScaleDirection::PhysToInt && c1 > 0)
                {
                    return Err(AutosarAbstractionError::InvalidParameter(
                        "CompuMethod category SCALE_LINEAR may not use IntToPhys and PhysToInt CompuScales at the same time".to_string(),
                    ));
                }
            }
            CompuMethodCategory::Rational => {
                if (direction == CompuScaleDirection::IntToPhys && c1 > 0)
                    || (direction == CompuScaleDirection::PhysToInt && c2 > 0)
                {
                    return Err(AutosarAbstractionError::InvalidParameter(
                        "CompuMethod category RAT_FUNC can only have one CompuScale for each direction".to_string(),
                    ));
                }
            }
            CompuMethodCategory::ScaleRational => {
                // no restriction, any number of CompuScales can be added
            }
            CompuMethodCategory::TextTable => {
                if direction == CompuScaleDirection::PhysToInt {
                    return Err(AutosarAbstractionError::InvalidParameter(
                        "CompuMethod category TEXTTABLE may not use PhysToInt CompuScales".to_string(),
                    ));
                }
            }
            CompuMethodCategory::BitfieldTextTable => {
                if direction == CompuScaleDirection::PhysToInt {
                    return Err(AutosarAbstractionError::InvalidParameter(
                        "CompuMethod category BITFIELD_TEXTTABLE may not use PhysToInt CompuScales".to_string(),
                    ));
                }
            }
            CompuMethodCategory::ScaleLinearAndTextTable => {
                if direction == CompuScaleDirection::PhysToInt {
                    return Err(AutosarAbstractionError::InvalidParameter(
                        "CompuMethod category SCALE_LINEAR_AND_TEXTTABLE may not use PhysToInt CompuScales".to_string(),
                    ));
                }
            }
            CompuMethodCategory::ScaleRationalAndTextTable => {
                // no restriction, any number of CompuScales can be added
            }
            CompuMethodCategory::TabNoInterpretation => {
                if direction == CompuScaleDirection::PhysToInt {
                    return Err(AutosarAbstractionError::InvalidParameter(
                        "CompuMethod category TAB_NOINTP may not use PhysToInt CompuScales".to_string(),
                    ));
                }
            }
        }

        let compu_scales = if direction == CompuScaleDirection::IntToPhys {
            self.element()
                .get_or_create_sub_element(ElementName::CompuInternalToPhys)?
                .get_or_create_sub_element(ElementName::CompuScales)?
        } else {
            self.element()
                .get_or_create_sub_element(ElementName::CompuPhysToInternal)?
                .get_or_create_sub_element(ElementName::CompuScales)?
        };
        CompuScale::new(&compu_scales, lower_limit, upper_limit)
    }

    /// Create an iterator over the internal-to-physical `CompuScales`
    pub fn int_to_phys_compu_scales(&self) -> impl Iterator<Item = CompuScale> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::CompuInternalToPhys)
            .and_then(|citp| citp.get_sub_element(ElementName::CompuScales))
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|cs| CompuScale::try_from(cs).ok())
    }

    /// Create an iterator over the physical-to-internal `CompuScales`
    pub fn phys_to_int_compu_scales(&self) -> impl Iterator<Item = CompuScale> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::CompuPhysToInternal)
            .and_then(|citp| citp.get_sub_element(ElementName::CompuScales))
            .into_iter()
            .flat_map(|elem: Element| elem.sub_elements())
            .filter_map(|cs| CompuScale::try_from(cs).ok())
    }
}

//#########################################################

/// Category of a `CompuMethod`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompuMethodCategory {
    /// Identical conversion: internal and physical values are the same
    Identical,
    /// Linear conversion: `y = offset + factor * x / divisor`
    Linear,
    /// Linear conversion with multiple scales, each with its own limits
    ScaleLinear,
    /// Rational function conversion: `y = (n0 + n1 * x + n2 * x^2 + ...) / (d0 + d1 * x + d2 * x^2 + ...)`
    Rational,
    /// Rational function conversion with multiple scales, each with its own limits
    ScaleRational,
    /// Text table conversion
    TextTable,
    /// Bitfield text table conversion
    BitfieldTextTable,
    /// Linear conversion with multiple scales and a text table
    ScaleLinearAndTextTable,
    /// Rational function conversion with multiple scales and a text table
    ScaleRationalAndTextTable,
    /// Value table with no interpretation
    TabNoInterpretation,
}

impl std::fmt::Display for CompuMethodCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CompuMethodCategory::Identical => write!(f, "IDENTICAL"),
            CompuMethodCategory::Linear => write!(f, "LINEAR"),
            CompuMethodCategory::ScaleLinear => write!(f, "SCALE_LINEAR"),
            CompuMethodCategory::Rational => write!(f, "RAT_FUNC"),
            CompuMethodCategory::ScaleRational => write!(f, "SCALE_RAT_FUNC"),
            CompuMethodCategory::TextTable => write!(f, "TEXTTABLE"),
            CompuMethodCategory::BitfieldTextTable => write!(f, "BITFIELD_TEXTTABLE"),
            CompuMethodCategory::ScaleLinearAndTextTable => write!(f, "SCALE_LINEAR_AND_TEXTTABLE"),
            CompuMethodCategory::ScaleRationalAndTextTable => write!(f, "SCALE_RATIONAL_AND_TEXTTABLE"),
            CompuMethodCategory::TabNoInterpretation => write!(f, "TAB_NOINTP"),
        }
    }
}

impl TryFrom<&str> for CompuMethodCategory {
    type Error = AutosarAbstractionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "IDENTICAL" => Ok(CompuMethodCategory::Identical),
            "LINEAR" => Ok(CompuMethodCategory::Linear),
            "SCALE_LINEAR" => Ok(CompuMethodCategory::ScaleLinear),
            "RAT_FUNC" => Ok(CompuMethodCategory::Rational),
            "SCALE_RAT_FUNC" => Ok(CompuMethodCategory::ScaleRational),
            "TEXTTABLE" => Ok(CompuMethodCategory::TextTable),
            "BITFIELD_TEXTTABLE" => Ok(CompuMethodCategory::BitfieldTextTable),
            "SCALE_LINEAR_AND_TEXTTABLE" => Ok(CompuMethodCategory::ScaleLinearAndTextTable),
            "SCALE_RATIONAL_AND_TEXTTABLE" => Ok(CompuMethodCategory::ScaleRationalAndTextTable),
            "TAB_NOINTP" => Ok(CompuMethodCategory::TabNoInterpretation),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "CompuMethodCategory".to_string(),
            }),
        }
    }
}

//#########################################################

/// A `CompuScale` describes the conversion between physical and internal values, as well as the limits of the scale
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompuScale(Element);
abstraction_element!(CompuScale, CompuScale);

impl CompuScale {
    fn new(
        parent: &Element,
        lower_limit: Option<f64>,
        upper_limit: Option<f64>,
    ) -> Result<Self, AutosarAbstractionError> {
        let compu_scale = parent.create_sub_element(ElementName::CompuScale)?;

        if let Some(lower_limit) = lower_limit {
            let cs_lower = compu_scale.create_sub_element(ElementName::LowerLimit)?;
            cs_lower.set_character_data(lower_limit)?;
            cs_lower.set_attribute_string(AttributeName::IntervalType, "CLOSED")?;
        }
        if let Some(upper_limit) = upper_limit {
            let cs_upper = compu_scale.create_sub_element(ElementName::UpperLimit)?;
            cs_upper.set_character_data(upper_limit)?;
            cs_upper.set_attribute_string(AttributeName::IntervalType, "CLOSED")?;
        }

        Ok(Self(compu_scale))
    }

    /// get the lower limit of the `CompuScale`
    #[must_use]
    pub fn lower_limit(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::LowerLimit)?
            .character_data()?
            .parse_float()
    }

    /// get the upper limit of the `CompuScale`
    #[must_use]
    pub fn upper_limit(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::UpperLimit)?
            .character_data()?
            .parse_float()
    }

    /// Set the mask of the `CompuScale`, applicable for `BitfieldTextTable`
    pub fn set_mask(&self, mask: u64) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::Mask)?
            .set_character_data(mask)?;
        Ok(())
    }

    /// Get the mask of the `CompuScale`, applicable for `BitfieldTextTable`
    #[must_use]
    pub fn mask(&self) -> Option<u64> {
        self.element()
            .get_sub_element(ElementName::Mask)?
            .character_data()?
            .parse_integer()
    }

    /// Set the content of the `CompuScale`
    pub fn set_content(&self, content: CompuScaleContent) -> Result<(), AutosarAbstractionError> {
        // remove existing content which may conflict with the new content
        let _ = self.element().remove_sub_element_kind(ElementName::CompuConst);
        let _ = self.element().remove_sub_element_kind(ElementName::CompuRationalCoeffs);

        match content {
            CompuScaleContent::TextConstant(value) => {
                self.element()
                    .create_sub_element(ElementName::CompuConst)?
                    .create_sub_element(ElementName::Vt)?
                    .set_character_data(value)?;
            }
            CompuScaleContent::NumericConstant(value) => {
                self.element()
                    .create_sub_element(ElementName::CompuConst)?
                    .create_sub_element(ElementName::V)?
                    .set_character_data(value)?;
            }
            CompuScaleContent::RationalCoeffs { numerator, denominator } => {
                let rational = self.element().create_sub_element(ElementName::CompuRationalCoeffs)?;
                let num = rational.create_sub_element(ElementName::CompuNumerator)?;
                for n in numerator {
                    num.create_sub_element(ElementName::V)?.set_character_data(n)?;
                }
                let den = rational.create_sub_element(ElementName::CompuDenominator)?;
                for d in denominator {
                    den.create_sub_element(ElementName::V)?.set_character_data(d)?;
                }
            }
        }

        Ok(())
    }

    /// Get the content of the `CompuScale`
    #[must_use]
    pub fn content(&self) -> Option<CompuScaleContent> {
        let compu_scale = self.element();

        if let Some(compu_const) = compu_scale.get_sub_element(ElementName::CompuConst) {
            if let Some(value) = compu_const
                .get_sub_element(ElementName::Vt)
                .and_then(|vt| vt.character_data())
                .and_then(|cdata| cdata.string_value())
            {
                return Some(CompuScaleContent::TextConstant(value));
            } else if let Some(value) = compu_const
                .get_sub_element(ElementName::V)
                .and_then(|v| v.character_data())
                .and_then(|cdata| cdata.parse_float())
            {
                return Some(CompuScaleContent::NumericConstant(value));
            }
        } else if let Some(compu_rational_coeffs) = compu_scale.get_sub_element(ElementName::CompuRationalCoeffs) {
            let mut numerator = vec![];
            let compu_numerator = compu_rational_coeffs.get_sub_element(ElementName::CompuNumerator)?;
            for v in compu_numerator.sub_elements() {
                if let Some(value) = v.character_data().and_then(|cdata| cdata.parse_float()) {
                    numerator.push(value);
                }
            }
            let mut denominator = vec![];
            let compu_denominator = compu_rational_coeffs.get_sub_element(ElementName::CompuDenominator)?;
            for v in compu_denominator.sub_elements() {
                if let Some(value) = v.character_data().and_then(|cdata| cdata.parse_float()) {
                    denominator.push(value);
                }
            }
            return Some(CompuScaleContent::RationalCoeffs { numerator, denominator });
        }

        None
    }
}

//#########################################################

/// Direction of a `CompuScale`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompuScaleDirection {
    /// Internal to physical conversion
    IntToPhys,
    /// Physical to internal conversion
    PhysToInt,
}

//#########################################################

/// Content of a `CompuScale`
#[derive(Debug, Clone, PartialEq)]
pub enum CompuScaleContent {
    /// Text constant
    TextConstant(String),
    /// Numeric constant
    NumericConstant(f64),
    /// Coefficients of a rational function
    RationalCoeffs {
        /// list of numerator coefficients
        numerator: Vec<f64>,
        /// list of denominator coefficients
        denominator: Vec<f64>,
    },
}

//#########################################################

/// Content of a `CompuMethod`
#[derive(Debug, Clone, PartialEq)]
pub enum CompuMethodContent {
    /// Identical conversion: internal and physical values are the same
    Identical,
    /// Linear conversion: `y = offset + factor * x / divisor`
    Linear(CompuMethodLinearContent),
    /// Linear conversion with multiple scales, each with its own limits
    ScaleLinear(Vec<CompuMethodScaleLinearContent>),
    /// Rational function conversion: `y = (n0 + n1 * x + n2 * x^2 + ...) / (d0 + d1 * x + d2 * x^2 + ...)`
    Rational(CompuMethodRationalContent),
    /// Rational function conversion with multiple scales, each with its own limits
    ScaleRational(Vec<CompuMethodRationalContent>),
    /// Text table conversion
    TextTable(Vec<CompuMethodTextTableContent>),
    /// Bitfield text table conversion
    BitfieldTextTable(Vec<CompuMethodBitfieldTextTableContent>),
    /// Linear conversion with multiple scales and a text table
    ScaleLinearAndTextTable(Vec<CompuMethodScaleLinearContent>, Vec<CompuMethodTextTableContent>),
    /// Rational function conversion with multiple scales and a text table
    ScaleRationalAndTextTable(Vec<CompuMethodRationalContent>, Vec<CompuMethodTextTableContent>),
    /// Value table with no interpretation
    TabNoInterpretation(Vec<CompuMethodTabNoIntpContent>),
}

/// Parameters of a linear conversion
#[derive(Debug, Clone, PartialEq)]
pub struct CompuMethodLinearContent {
    /// direction of the conversion
    pub direction: CompuScaleDirection,
    /// offset
    pub offset: f64,
    /// factor
    pub factor: f64,
    /// divisor
    pub divisor: f64,
    /// optional: lower limit of the scale
    pub lower_limit: Option<f64>,
    /// optional: upper limit of the scale
    pub upper_limit: Option<f64>,
}

/// Parameters of a single linear conversion scale for a scale linear conversion
#[derive(Debug, Clone, PartialEq)]
pub struct CompuMethodScaleLinearContent {
    /// direction of the conversion
    pub direction: CompuScaleDirection,
    /// offset
    pub offset: f64,
    /// factor
    pub factor: f64,
    /// divisor
    pub divisor: f64,
    /// lower limit of the scale
    pub lower_limit: f64,
    /// upper limit of the scale
    pub upper_limit: f64,
}

/// Parameters of a rational function conversion
#[derive(Debug, Clone, PartialEq)]
pub struct CompuMethodRationalContent {
    /// direction of the conversion
    pub direction: CompuScaleDirection,
    /// list of numerator coefficients
    pub denominator: Vec<f64>,
    /// list of denominator coefficients
    pub numerator: Vec<f64>,
    /// lower limit of the scale
    pub lower_limit: f64,
    /// upper limit of the scale
    pub upper_limit: f64,
}

/// A single entry of a text table conversion
#[derive(Debug, Clone, PartialEq)]
pub struct CompuMethodTextTableContent {
    /// text of this entry
    pub text: String,
    /// numeric value of this entry
    pub value: f64,
}

/// A single entry of a bitfield text table conversion
#[derive(Debug, Clone, PartialEq)]
pub struct CompuMethodBitfieldTextTableContent {
    /// text of this entry
    pub text: String,
    /// numeric value of this entry
    pub value: f64,
    /// bit mask of this entry
    pub mask: u64,
}

/// A single entry of a value table with no interpretation
#[derive(Debug, Clone, PartialEq)]
pub struct CompuMethodTabNoIntpContent {
    /// input value
    pub value_in: f64,
    /// output value
    pub value_out: f64,
}

//#########################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::AutosarModelAbstraction;
    use autosar_data::AutosarVersion;

    #[test]
    fn compu_method() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/Package").unwrap();

        let compu_method1 = CompuMethod::new("compu_method1", &package, CompuMethodContent::Identical).unwrap();
        assert_eq!(compu_method1.category(), Some(CompuMethodCategory::Identical));
        assert_eq!(compu_method1.content(), Some(CompuMethodContent::Identical));

        let content2 = CompuMethodContent::Linear(CompuMethodLinearContent {
            direction: CompuScaleDirection::IntToPhys,
            offset: 0.01,
            factor: 1.01,
            divisor: 1.02,
            lower_limit: Some(0.0),
            upper_limit: Some(100.0),
        });
        let compu_method2 = CompuMethod::new("compu_method2", &package, content2.clone()).unwrap();
        assert_eq!(compu_method2.category(), Some(CompuMethodCategory::Linear));
        assert_eq!(compu_method2.content().unwrap(), content2);
        assert_eq!(compu_method2.int_to_phys_compu_scales().count(), 1);
        assert_eq!(compu_method2.phys_to_int_compu_scales().count(), 0);

        let content3 = CompuMethodContent::Linear(CompuMethodLinearContent {
            direction: CompuScaleDirection::PhysToInt,
            offset: 0.01,
            factor: 1.01,
            divisor: 1.02,
            lower_limit: Some(0.0),
            upper_limit: Some(100.0),
        });
        let compu_method3 = CompuMethod::new("compu_method3", &package, content3.clone()).unwrap();
        assert_eq!(compu_method3.category(), Some(CompuMethodCategory::Linear));
        assert_eq!(compu_method3.content().unwrap(), content3);
        assert_eq!(compu_method3.int_to_phys_compu_scales().count(), 0);
        assert_eq!(compu_method3.phys_to_int_compu_scales().count(), 1);

        let content4 = CompuMethodContent::ScaleLinear(vec![
            CompuMethodScaleLinearContent {
                direction: CompuScaleDirection::IntToPhys,
                offset: 0.0,
                factor: 2.0,
                divisor: 1.5,
                lower_limit: 0.0,
                upper_limit: 100.0,
            },
            CompuMethodScaleLinearContent {
                direction: CompuScaleDirection::IntToPhys,
                offset: 0.0,
                factor: 1.0,
                divisor: 1.0,
                lower_limit: 200.0,
                upper_limit: 400.0,
            },
        ]);
        let compu_method4 = CompuMethod::new("compu_method4", &package, content4.clone()).unwrap();
        assert_eq!(compu_method4.category(), Some(CompuMethodCategory::ScaleLinear));
        assert_eq!(compu_method4.content().unwrap(), content4);
        assert_eq!(compu_method4.int_to_phys_compu_scales().count(), 2);
        assert_eq!(compu_method4.phys_to_int_compu_scales().count(), 0);

        let content5 = CompuMethodContent::ScaleLinear(vec![CompuMethodScaleLinearContent {
            direction: CompuScaleDirection::PhysToInt,
            offset: 0.0,
            factor: 2.0,
            divisor: 1.5,
            lower_limit: 0.0,
            upper_limit: 100.0,
        }]);
        let compu_method5 = CompuMethod::new("compu_method5", &package, content5.clone()).unwrap();
        assert_eq!(compu_method5.category(), Some(CompuMethodCategory::ScaleLinear));
        assert_eq!(compu_method5.content().unwrap(), content5);

        let content6 = CompuMethodContent::Rational(CompuMethodRationalContent {
            direction: CompuScaleDirection::IntToPhys,
            numerator: vec![1.1, 2.2, 3.3, 4.4],
            denominator: vec![0.1, 0.2, 0.3],
            lower_limit: 0.0,
            upper_limit: 100.0,
        });
        let compu_method6 = CompuMethod::new("compu_method6", &package, content6.clone()).unwrap();
        assert_eq!(compu_method6.category(), Some(CompuMethodCategory::Rational));
        assert_eq!(compu_method6.content().unwrap(), content6);

        let content7 = CompuMethodContent::Rational(CompuMethodRationalContent {
            direction: CompuScaleDirection::PhysToInt,
            numerator: vec![1.1, 2.2, 3.3, 4.4],
            denominator: vec![0.1, 0.2, 0.3],
            lower_limit: 0.0,
            upper_limit: 100.0,
        });
        let compu_method7 = CompuMethod::new("compu_method7", &package, content7.clone()).unwrap();
        assert_eq!(compu_method7.category(), Some(CompuMethodCategory::Rational));
        assert_eq!(compu_method7.content().unwrap(), content7);

        let content8 = CompuMethodContent::ScaleRational(vec![
            CompuMethodRationalContent {
                direction: CompuScaleDirection::IntToPhys,
                numerator: vec![1.1, 2.2, 3.3, 4.4],
                denominator: vec![0.1, 0.2, 0.3],
                lower_limit: 0.0,
                upper_limit: 100.0,
            },
            CompuMethodRationalContent {
                direction: CompuScaleDirection::IntToPhys,
                numerator: vec![1.1, 2.2, 3.3, 4.4],
                denominator: vec![0.1, 0.2, 0.3],
                lower_limit: 200.0,
                upper_limit: 400.0,
            },
        ]);
        let compu_method8 = CompuMethod::new("compu_method8", &package, content8.clone()).unwrap();
        assert_eq!(compu_method8.category(), Some(CompuMethodCategory::ScaleRational));
        assert_eq!(compu_method8.content().unwrap(), content8);

        let content9 = CompuMethodContent::ScaleRational(vec![CompuMethodRationalContent {
            direction: CompuScaleDirection::PhysToInt,
            numerator: vec![1.1, 2.2, 3.3, 4.4],
            denominator: vec![0.1, 0.2, 0.3],
            lower_limit: 0.0,
            upper_limit: 100.0,
        }]);
        let compu_method9 = CompuMethod::new("compu_method9", &package, content9.clone()).unwrap();
        assert_eq!(compu_method9.category(), Some(CompuMethodCategory::ScaleRational));
        assert_eq!(compu_method9.content().unwrap(), content9);

        let content10 = CompuMethodContent::TextTable(vec![
            CompuMethodTextTableContent {
                text: "text1".to_string(),
                value: 1.0,
            },
            CompuMethodTextTableContent {
                text: "text2".to_string(),
                value: 2.0,
            },
        ]);
        let compu_method10 = CompuMethod::new("compu_method10", &package, content10.clone()).unwrap();
        assert_eq!(compu_method10.category(), Some(CompuMethodCategory::TextTable));
        assert_eq!(compu_method10.content().unwrap(), content10);

        let content11 = CompuMethodContent::BitfieldTextTable(vec![
            CompuMethodBitfieldTextTableContent {
                text: "text1".to_string(),
                value: 1.0,
                mask: 0b0000_0001,
            },
            CompuMethodBitfieldTextTableContent {
                text: "text2".to_string(),
                value: 2.0,
                mask: 0b0000_0010,
            },
        ]);
        let compu_method11 = CompuMethod::new("compu_method11", &package, content11.clone()).unwrap();
        assert_eq!(compu_method11.category(), Some(CompuMethodCategory::BitfieldTextTable));
        assert_eq!(compu_method11.content().unwrap(), content11);

        let content12 = CompuMethodContent::ScaleLinearAndTextTable(
            vec![
                CompuMethodScaleLinearContent {
                    direction: CompuScaleDirection::IntToPhys,
                    offset: 0.0,
                    factor: 2.0,
                    divisor: 1.5,
                    lower_limit: 0.0,
                    upper_limit: 100.0,
                },
                CompuMethodScaleLinearContent {
                    direction: CompuScaleDirection::IntToPhys,
                    offset: 0.0,
                    factor: 1.0,
                    divisor: 1.0,
                    lower_limit: 200.0,
                    upper_limit: 400.0,
                },
            ],
            vec![
                CompuMethodTextTableContent {
                    text: "text1".to_string(),
                    value: 1.0,
                },
                CompuMethodTextTableContent {
                    text: "text2".to_string(),
                    value: 2.0,
                },
            ],
        );
        let compu_method12 = CompuMethod::new("compu_method12", &package, content12.clone()).unwrap();
        assert_eq!(
            compu_method12.category(),
            Some(CompuMethodCategory::ScaleLinearAndTextTable)
        );
        assert_eq!(compu_method12.content().unwrap(), content12);

        let content13 = CompuMethodContent::ScaleRationalAndTextTable(
            vec![
                CompuMethodRationalContent {
                    direction: CompuScaleDirection::IntToPhys,
                    numerator: vec![1.1, 2.2, 3.3, 4.4],
                    denominator: vec![0.1, 0.2, 0.3],
                    lower_limit: 0.0,
                    upper_limit: 100.0,
                },
                CompuMethodRationalContent {
                    direction: CompuScaleDirection::IntToPhys,
                    numerator: vec![1.1, 2.2, 3.3, 4.4],
                    denominator: vec![0.1, 0.2, 0.3],
                    lower_limit: 200.0,
                    upper_limit: 400.0,
                },
            ],
            vec![
                CompuMethodTextTableContent {
                    text: "text1".to_string(),
                    value: 1.0,
                },
                CompuMethodTextTableContent {
                    text: "text2".to_string(),
                    value: 2.0,
                },
            ],
        );
        let compu_method13 = CompuMethod::new("compu_method13", &package, content13.clone()).unwrap();
        assert_eq!(
            compu_method13.category(),
            Some(CompuMethodCategory::ScaleRationalAndTextTable)
        );
        assert_eq!(compu_method13.content().unwrap(), content13);
        assert_eq!(compu_method13.int_to_phys_compu_scales().count(), 4);
        assert_eq!(compu_method13.phys_to_int_compu_scales().count(), 0);

        let content14 = CompuMethodContent::ScaleRationalAndTextTable(
            vec![CompuMethodRationalContent {
                direction: CompuScaleDirection::PhysToInt,
                numerator: vec![1.1, 2.2, 3.3, 4.4],
                denominator: vec![0.1, 0.2, 0.3],
                lower_limit: 0.0,
                upper_limit: 100.0,
            }],
            vec![CompuMethodTextTableContent {
                text: "text1".to_string(),
                value: 1.0,
            }],
        );
        let compu_method14 = CompuMethod::new("compu_method14", &package, content14.clone()).unwrap();
        assert_eq!(
            compu_method14.category(),
            Some(CompuMethodCategory::ScaleRationalAndTextTable)
        );
        assert_eq!(compu_method14.content().unwrap(), content14);
        assert_eq!(compu_method14.int_to_phys_compu_scales().count(), 1);
        assert_eq!(compu_method14.phys_to_int_compu_scales().count(), 1);

        let content15 = CompuMethodContent::TabNoInterpretation(vec![
            CompuMethodTabNoIntpContent {
                value_in: 1.0,
                value_out: 2.0,
            },
            CompuMethodTabNoIntpContent {
                value_in: 3.0,
                value_out: 4.0,
            },
        ]);
        let compu_method15 = CompuMethod::new("compu_method15", &package, content15.clone()).unwrap();
        assert_eq!(
            compu_method15.category(),
            Some(CompuMethodCategory::TabNoInterpretation)
        );
        assert_eq!(compu_method15.content().unwrap(), content15);
    }

    #[test]
    fn compu_method_category() {
        assert_eq!(
            CompuMethodCategory::try_from("IDENTICAL").unwrap(),
            CompuMethodCategory::Identical
        );
        assert_eq!(
            CompuMethodCategory::try_from("LINEAR").unwrap(),
            CompuMethodCategory::Linear
        );
        assert_eq!(
            CompuMethodCategory::try_from("SCALE_LINEAR").unwrap(),
            CompuMethodCategory::ScaleLinear
        );
        assert_eq!(
            CompuMethodCategory::try_from("RAT_FUNC").unwrap(),
            CompuMethodCategory::Rational
        );
        assert_eq!(
            CompuMethodCategory::try_from("SCALE_RAT_FUNC").unwrap(),
            CompuMethodCategory::ScaleRational
        );
        assert_eq!(
            CompuMethodCategory::try_from("TEXTTABLE").unwrap(),
            CompuMethodCategory::TextTable
        );
        assert_eq!(
            CompuMethodCategory::try_from("BITFIELD_TEXTTABLE").unwrap(),
            CompuMethodCategory::BitfieldTextTable
        );
        assert_eq!(
            CompuMethodCategory::try_from("SCALE_LINEAR_AND_TEXTTABLE").unwrap(),
            CompuMethodCategory::ScaleLinearAndTextTable
        );
        assert_eq!(
            CompuMethodCategory::try_from("SCALE_RATIONAL_AND_TEXTTABLE").unwrap(),
            CompuMethodCategory::ScaleRationalAndTextTable
        );
        assert_eq!(
            CompuMethodCategory::try_from("TAB_NOINTP").unwrap(),
            CompuMethodCategory::TabNoInterpretation
        );
        assert!(CompuMethodCategory::try_from("INVALID").is_err());

        assert_eq!(CompuMethodCategory::Identical.to_string(), "IDENTICAL");
        assert_eq!(CompuMethodCategory::Linear.to_string(), "LINEAR");
        assert_eq!(CompuMethodCategory::ScaleLinear.to_string(), "SCALE_LINEAR");
        assert_eq!(CompuMethodCategory::Rational.to_string(), "RAT_FUNC");
        assert_eq!(CompuMethodCategory::ScaleRational.to_string(), "SCALE_RAT_FUNC");
        assert_eq!(CompuMethodCategory::TextTable.to_string(), "TEXTTABLE");
        assert_eq!(CompuMethodCategory::BitfieldTextTable.to_string(), "BITFIELD_TEXTTABLE");
        assert_eq!(
            CompuMethodCategory::ScaleLinearAndTextTable.to_string(),
            "SCALE_LINEAR_AND_TEXTTABLE"
        );
        assert_eq!(
            CompuMethodCategory::ScaleRationalAndTextTable.to_string(),
            "SCALE_RATIONAL_AND_TEXTTABLE"
        );
        assert_eq!(CompuMethodCategory::TabNoInterpretation.to_string(), "TAB_NOINTP");
    }
}
