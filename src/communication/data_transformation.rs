use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, ByteOrder, IdentifiableAbstractionElement,
    abstraction_element,
};
use autosar_data::{AutosarVersion, Element, ElementName, EnumItem};

/// A [`DataTransformationSet`] contains `DataTransformation`s and `TransformationTechnology`s used in communication
///
/// Use [`ArPackage::create_data_transformation_set`] to create a new `DataTransformationSet`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataTransformationSet(Element);
abstraction_element!(DataTransformationSet, DataTransformationSet);
impl IdentifiableAbstractionElement for DataTransformationSet {}

impl DataTransformationSet {
    /// Create a new `DataTransformationSet`
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let transformation_set = elements.create_named_sub_element(ElementName::DataTransformationSet, name)?;

        Ok(Self(transformation_set))
    }

    /// Create a new `DataTransformation` in the `DataTransformationSet`
    pub fn create_data_transformation(
        &self,
        name: &str,
        transformations: &[&TransformationTechnology],
        execute_despite_data_unavailability: bool,
    ) -> Result<DataTransformation, AutosarAbstractionError> {
        let data_transformations = self
            .element()
            .get_or_create_sub_element(ElementName::DataTransformations)?;
        let transformation = DataTransformation::new(
            &data_transformations,
            name,
            transformations,
            execute_despite_data_unavailability,
        )?;
        Ok(transformation)
    }

    /// Iterate over all `DataTransformation`s in the `DataTransformationSet`
    pub fn data_transformations(&self) -> impl Iterator<Item = DataTransformation> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::DataTransformations)
            .into_iter()
            .flat_map(|container| container.sub_elements())
            .filter_map(|elem| elem.try_into().ok())
    }

    /// Create a new `TransformationTechnology` in the `DataTransformationSet`
    pub fn create_transformation_technology(
        &self,
        name: &str,
        config: &TransformationTechnologyConfig,
    ) -> Result<TransformationTechnology, AutosarAbstractionError> {
        let transtechs = self
            .element()
            .get_or_create_sub_element(ElementName::TransformationTechnologys)?;
        TransformationTechnology::new(&transtechs, name, config)
    }

    /// Iterate over all `TransformationTechnology`s in the `DataTransformationSet`
    pub fn transformation_technologies(&self) -> impl Iterator<Item = TransformationTechnology> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::TransformationTechnologys)
            .into_iter()
            .flat_map(|container| container.sub_elements())
            .filter_map(|elem| elem.try_into().ok())
    }
}

//#########################################################

/// A `DataTransformation` is a chain of `TransformationTechnology`s that are used to transform data
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataTransformation(Element);
abstraction_element!(DataTransformation, DataTransformation);
impl IdentifiableAbstractionElement for DataTransformation {}

impl DataTransformation {
    /// Create a new `DataTransformation`
    fn new(
        parent: &Element,
        name: &str,
        transformations: &[&TransformationTechnology],
        execute_despite_data_unavailability: bool,
    ) -> Result<Self, AutosarAbstractionError> {
        // an empty chain is not allowed
        if transformations.is_empty() {
            return Err(AutosarAbstractionError::InvalidParameter(
                "A DataTransformation must contain at least one TransformationTechnology".to_string(),
            ));
        }

        // only the first transformation in a chain may have TransformerClass 'Serializer'
        for transformation in &transformations[1..] {
            if transformation.transformer_class() == Some(EnumItem::Serializer) {
                return Err(AutosarAbstractionError::InvalidParameter(
                    "A DataTransformation may only contain a TransformationTechnology with TransformerClass 'Serializer' at the start of the chain".to_string(),
                ));
            }
        }

        // every transformation in the chain must be part of the same DataTransformationSet as the DataTransformation
        let dts = parent
            .named_parent()?
            .and_then(|dts| DataTransformationSet::try_from(dts).ok());
        if !transformations
            .iter()
            .all(|ttech| ttech.data_transformation_set() == dts)
        {
            return Err(AutosarAbstractionError::InvalidParameter(
                "All TransformationTechnologies in a DataTransformation must be part of the same DataTransformationSet"
                    .to_string(),
            ));
        }

        // if any of the transformations is an E2E transformation, then executeDespiteDataUnavailability must be true
        if transformations
            .iter()
            .any(|ttech| ttech.protocol().as_deref() == Some("E2E"))
            && !execute_despite_data_unavailability
        {
            return Err(AutosarAbstractionError::InvalidParameter(
                "If a DataTransformation contains an E2E transformation, executeDespiteDataUnavailability must be true"
                    .to_string(),
            ));
        }

        let transformation = parent.create_named_sub_element(ElementName::DataTransformation, name)?;
        transformation
            .create_sub_element(ElementName::ExecuteDespiteDataUnavailability)?
            .set_character_data(execute_despite_data_unavailability)?;
        let chain_refs = transformation.create_sub_element(ElementName::TransformerChainRefs)?;
        for transformation in transformations {
            chain_refs
                .create_sub_element(ElementName::TransformerChainRef)?
                .set_reference_target(transformation.element())?;
        }

        Ok(Self(transformation))
    }

    /// get the `DataTransformationSet` that contains this `DataTransformation`
    #[must_use]
    pub fn data_transformation_set(&self) -> Option<DataTransformationSet> {
        self.element()
            .named_parent()
            .ok()?
            .and_then(|dts| DataTransformationSet::try_from(dts).ok())
    }

    /// Create an iterator over the `TransformationTechnologies` in the `DataTransformation`
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::{*, communication::*};
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// # let package = model.get_or_create_package("/pkg")?;
    /// let dts = package.create_data_transformation_set("dts")?;
    /// let config = TransformationTechnologyConfig::Com(ComTransformationTechnologyConfig { isignal_ipdu_length: 8 });
    /// let ttech = dts.create_transformation_technology("ttech1", &config)?;
    /// let dt = dts.create_data_transformation("dt", &[&ttech], true)?;
    /// let mut ttech_iter = dt.transformation_technologies();
    /// assert_eq!(ttech_iter.next(), Some(ttech));
    /// # Ok(())}
    /// ```
    pub fn transformation_technologies(&self) -> impl Iterator<Item = TransformationTechnology> + Send + 'static {
        self.0
            .get_sub_element(ElementName::TransformerChainRefs)
            .into_iter()
            .flat_map(|container| container.sub_elements())
            .filter_map(|elem| {
                elem.get_reference_target()
                    .ok()
                    .and_then(|ttech| TransformationTechnology::try_from(ttech).ok())
            })
    }
}

//#########################################################

/// A `TransformationTechnology` describes how to transform signal or PDU data
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransformationTechnology(Element);
abstraction_element!(TransformationTechnology, TransformationTechnology);
impl IdentifiableAbstractionElement for TransformationTechnology {}

impl TransformationTechnology {
    /// Create a new `TransformationTechnology`
    fn new(
        parent: &Element,
        name: &str,
        config: &TransformationTechnologyConfig,
    ) -> Result<Self, AutosarAbstractionError> {
        let ttech_elem = parent.create_named_sub_element(ElementName::TransformationTechnology, name)?;
        let ttech = Self(ttech_elem);
        ttech.set_config(config)?;

        Ok(ttech)
    }

    /// Set the configuration of the `TransformationTechnology`
    pub fn set_config(&self, config: &TransformationTechnologyConfig) -> Result<(), AutosarAbstractionError> {
        let ttech = self.element();
        let version = ttech.min_version()?;
        let buffer_props = ttech.get_or_create_sub_element(ElementName::BufferProperties)?;

        match config {
            TransformationTechnologyConfig::Generic(generic_config) => {
                ttech
                    .get_or_create_sub_element(ElementName::Protocol)?
                    .set_character_data(generic_config.protocol_name.as_str())?;
                ttech
                    .get_or_create_sub_element(ElementName::Version)?
                    .set_character_data(generic_config.protocol_version.as_str())?;
                ttech
                    .get_or_create_sub_element(ElementName::TransformerClass)?
                    .set_character_data(EnumItem::Custom)?;
                buffer_props
                    .get_or_create_sub_element(ElementName::HeaderLength)?
                    .set_character_data(u64::from(generic_config.header_length))?;
                buffer_props
                    .get_or_create_sub_element(ElementName::InPlace)?
                    .set_character_data(generic_config.in_place)?;

                // remove the transformation descriptions, which are only used by E2E and SOMEIP
                let _ = ttech.remove_sub_element_kind(ElementName::TransformationDescriptions);
                // remove the buffer computation, which is only used by COM
                let _ = buffer_props.remove_sub_element_kind(ElementName::BufferComputation);
                // remove the NeedsOriginalData setting, which is only used by E2E
                let _ = ttech.remove_sub_element_kind(ElementName::NeedsOriginalData);
            }
            TransformationTechnologyConfig::Com(com_config) => {
                ttech
                    .get_or_create_sub_element(ElementName::Protocol)?
                    .set_character_data("COMBased")?;
                ttech
                    .get_or_create_sub_element(ElementName::Version)?
                    .set_character_data("1")?;
                ttech
                    .get_or_create_sub_element(ElementName::TransformerClass)?
                    .set_character_data(EnumItem::Serializer)?;

                // comxf does not have a header
                buffer_props
                    .get_or_create_sub_element(ElementName::HeaderLength)?
                    .set_character_data(0)?;
                // comxf is always the first transformer in a chain, and the first transformer is not allowed to be in place
                buffer_props
                    .get_or_create_sub_element(ElementName::InPlace)?
                    .set_character_data("false")?;

                if version <= AutosarVersion::Autosar_00049 {
                    let _ = buffer_props.remove_sub_element_kind(ElementName::BufferComputation);
                    // only in versions up to AUTOSAR R20-11 (AUTOSAR_00049): a COM transformer must have a BUFFER-COMPUTATION
                    let bufcomp_compu = buffer_props
                        .create_sub_element(ElementName::BufferComputation)?
                        .create_sub_element(ElementName::CompuRationalCoeffs)?;
                    let numerator = bufcomp_compu.create_sub_element(ElementName::CompuNumerator)?;
                    numerator
                        .create_sub_element(ElementName::V)?
                        .set_character_data(u64::from(com_config.isignal_ipdu_length))?;
                    numerator.create_sub_element(ElementName::V)?.set_character_data(1)?;
                    bufcomp_compu
                        .create_sub_element(ElementName::CompuDenominator)?
                        .create_sub_element(ElementName::V)?
                        .set_character_data(1)?;
                }

                // remove the transformation descriptions, which are only used by E2E and SOMEIP
                let _ = ttech.remove_sub_element_kind(ElementName::TransformationDescriptions);
                // remove the NeedsOriginalData setting, which is only used by E2E
                let _ = ttech.remove_sub_element_kind(ElementName::NeedsOriginalData);
            }
            TransformationTechnologyConfig::E2E(e2e_config) => {
                ttech
                    .get_or_create_sub_element(ElementName::Protocol)?
                    .set_character_data("E2E")?;
                ttech
                    .get_or_create_sub_element(ElementName::Version)?
                    .set_character_data("1.0.0")?;
                ttech
                    .get_or_create_sub_element(ElementName::TransformerClass)?
                    .set_character_data(EnumItem::Safety)?;
                if version >= AutosarVersion::Autosar_4_3_0 {
                    ttech
                        .get_or_create_sub_element(ElementName::HasInternalState)?
                        .set_character_data("true")?;
                }
                ttech
                    .get_or_create_sub_element(ElementName::NeedsOriginalData)?
                    .set_character_data("false")?;

                // select the profile name and header length based on the chosen E2E profile
                let (profile_name, header_length) = match e2e_config.profile {
                    E2EProfile::P01 => ("PROFILE_01", 16),
                    E2EProfile::P02 => ("PROFILE_02", 16),
                    E2EProfile::P04 => ("PROFILE_04", 96),
                    E2EProfile::P04m => ("PROFILE_04m", 128),
                    E2EProfile::P05 => ("PROFILE_05", 24),
                    E2EProfile::P06 => ("PROFILE_06", 40),
                    E2EProfile::P07 => ("PROFILE_07", 160),
                    E2EProfile::P07m => ("PROFILE_07m", 192),
                    E2EProfile::P08 => ("PROFILE_08", 128),
                    E2EProfile::P08m => ("PROFILE_08m", 160),
                    E2EProfile::P11 => ("PROFILE_11", 16),
                    E2EProfile::P22 => ("PROFILE_22", 16),
                    E2EProfile::P44 => ("PROFILE_44", 96),
                    E2EProfile::P44m => ("PROFILE_44m", 128),
                };

                // when E2E is used in a transformer chain after COM, the header length must be zero
                // since we don#t know how this transformer will be used, the user must set zero_header_length to true if the header length should be zero
                let real_header_length = if e2e_config.zero_header_length {
                    0u32
                } else {
                    header_length
                };

                buffer_props
                    .get_or_create_sub_element(ElementName::HeaderLength)?
                    .set_character_data(u64::from(real_header_length))?;
                buffer_props
                    .get_or_create_sub_element(ElementName::InPlace)?
                    .set_character_data(e2e_config.transform_in_place)?;

                let trans_desc = ttech.get_or_create_sub_element(ElementName::TransformationDescriptions)?;
                let _ = trans_desc.remove_sub_element_kind(ElementName::SomeipTransformationDescription);
                let e2e_desc = trans_desc.get_or_create_sub_element(ElementName::EndToEndTransformationDescription)?;

                // create the E2E profile description, with the mandatory fields
                e2e_desc
                    .get_or_create_sub_element(ElementName::ProfileName)?
                    .set_character_data(profile_name)?;
                e2e_desc
                    .get_or_create_sub_element(ElementName::UpperHeaderBitsToShift)?
                    .set_character_data(u64::from(e2e_config.offset))?;
                e2e_desc
                    .get_or_create_sub_element(ElementName::MaxDeltaCounter)?
                    .set_character_data(u64::from(e2e_config.max_delta_counter))?;
                e2e_desc
                    .get_or_create_sub_element(ElementName::MaxErrorStateInit)?
                    .set_character_data(u64::from(e2e_config.max_error_state_init))?;
                e2e_desc
                    .get_or_create_sub_element(ElementName::MaxErrorStateInvalid)?
                    .set_character_data(u64::from(e2e_config.max_error_state_invalid))?;
                e2e_desc
                    .get_or_create_sub_element(ElementName::MaxErrorStateValid)?
                    .set_character_data(u64::from(e2e_config.max_error_state_valid))?;
                e2e_desc
                    .get_or_create_sub_element(ElementName::MaxNoNewOrRepeatedData)?
                    .set_character_data(u64::from(e2e_config.max_no_new_or_repeated_data))?;
                e2e_desc
                    .get_or_create_sub_element(ElementName::MinOkStateInit)?
                    .set_character_data(u64::from(e2e_config.min_ok_state_init))?;
                e2e_desc
                    .get_or_create_sub_element(ElementName::MinOkStateInvalid)?
                    .set_character_data(u64::from(e2e_config.min_ok_state_invalid))?;
                e2e_desc
                    .get_or_create_sub_element(ElementName::MinOkStateValid)?
                    .set_character_data(u64::from(e2e_config.min_ok_state_valid))?;

                // window size is one value in AUTOSAR 4.4.0 (AUTOSAR_00047) and older, and three values in AUTOSAR 4.5.0 (AUTOSAR_00048) and newer
                if version <= AutosarVersion::Autosar_00047 {
                    // window size is only valid in AUTOSAR 4.4.0 (AUTOSAR_00047) and older
                    e2e_desc
                        .get_or_create_sub_element(ElementName::WindowSize)?
                        .set_character_data(u64::from(e2e_config.window_size))?;
                } else {
                    // new (Autosar 4.5.0+): window size can be set for each state
                    e2e_desc
                        .get_or_create_sub_element(ElementName::WindowSizeInit)?
                        .set_character_data(u64::from(e2e_config.window_size_init.unwrap_or(e2e_config.window_size)))?;
                    e2e_desc
                        .get_or_create_sub_element(ElementName::WindowSizeInvalid)?
                        .set_character_data(u64::from(
                            e2e_config.window_size_invalid.unwrap_or(e2e_config.window_size),
                        ))?;
                    e2e_desc
                        .get_or_create_sub_element(ElementName::WindowSizeValid)?
                        .set_character_data(u64::from(
                            e2e_config.window_size_valid.unwrap_or(e2e_config.window_size),
                        ))?;
                }

                // special handling for E2E profiles 01 and 11
                if matches!(e2e_config.profile, E2EProfile::P01 | E2EProfile::P11) {
                    // data id mode
                    let Some(data_id_mode) = e2e_config.data_id_mode else {
                        return Err(AutosarAbstractionError::InvalidParameter(
                            "Data ID mode is required for E2E profiles 01 and 11".to_string(),
                        ));
                    };
                    e2e_desc
                        .get_or_create_sub_element(ElementName::DataIdMode)?
                        .set_character_data::<EnumItem>(data_id_mode.into())?;

                    // counter offset
                    let Some(counter_offset) = e2e_config.counter_offset else {
                        return Err(AutosarAbstractionError::InvalidParameter(
                            "Counter offset is required for E2E profiles 01 and 11".to_string(),
                        ));
                    };
                    e2e_desc
                        .get_or_create_sub_element(ElementName::CounterOffset)?
                        .set_character_data(u64::from(counter_offset))?;

                    // crc offset
                    let Some(crc_offset) = e2e_config.crc_offset else {
                        return Err(AutosarAbstractionError::InvalidParameter(
                            "CRC offset is required for E2E profiles 01 and 11".to_string(),
                        ));
                    };
                    e2e_desc
                        .get_or_create_sub_element(ElementName::CrcOffset)?
                        .set_character_data(u64::from(crc_offset))?;

                    // data id nibble offset
                    if data_id_mode == DataIdMode::Lower12Bit {
                        let Some(data_id_nibble_offset) = e2e_config.data_id_nibble_offset else {
                            return Err(AutosarAbstractionError::InvalidParameter(
                                "Data ID nibble offset is required for E2E profiles 01 and 11 with DataIdMode::Lower12Bit".to_string(),
                            ));
                        };
                        e2e_desc
                            .get_or_create_sub_element(ElementName::DataIdNibbleOffset)?
                            .set_character_data(u64::from(data_id_nibble_offset))?;
                    }

                    // offset may only be set if the profile is not 01 or 11
                    let _ = e2e_desc.remove_sub_element_kind(ElementName::Offset);
                } else {
                    // offset may only be set if the profile is not 01 or 11
                    e2e_desc
                        .get_or_create_sub_element(ElementName::Offset)?
                        .set_character_data(u64::from(e2e_config.offset))?;

                    // remove the data id mode, counter offset, crc offset, and data id nibble offset, which are only used by profiles 01 and 11
                    let _ = e2e_desc.remove_sub_element_kind(ElementName::DataIdMode);
                    let _ = e2e_desc.remove_sub_element_kind(ElementName::CounterOffset);
                    let _ = e2e_desc.remove_sub_element_kind(ElementName::CrcOffset);
                    let _ = e2e_desc.remove_sub_element_kind(ElementName::DataIdNibbleOffset);
                }

                // optional fields
                // profile behavior
                if let Some(profile_behavior) = e2e_config.profile_behavior {
                    e2e_desc
                        .get_or_create_sub_element(ElementName::ProfileBehavior)?
                        .set_character_data::<EnumItem>(profile_behavior.into())?;
                }

                // sync counter init
                if let Some(sync_counter_init) = e2e_config.sync_counter_init {
                    e2e_desc
                        .get_or_create_sub_element(ElementName::SyncCounterInit)?
                        .set_character_data(u64::from(sync_counter_init))?;
                }

                // remove the buffer computation, which is only used by COM
                let _ = buffer_props.remove_sub_element_kind(ElementName::BufferComputation);
            }
            TransformationTechnologyConfig::SomeIp(someip_config) => {
                ttech
                    .get_or_create_sub_element(ElementName::Protocol)?
                    .set_character_data("SOMEIP")?;
                ttech
                    .get_or_create_sub_element(ElementName::TransformerClass)?
                    .set_character_data(EnumItem::Serializer)?;

                // someip header length is always 64
                buffer_props
                    .get_or_create_sub_element(ElementName::HeaderLength)?
                    .set_character_data(64)?;
                // someip is always the first transformer in a chain, and the first transformer is not allowed to be in place
                buffer_props
                    .get_or_create_sub_element(ElementName::InPlace)?
                    .set_character_data("false")?;

                let trans_desc = ttech.get_or_create_sub_element(ElementName::TransformationDescriptions)?;
                let _ = trans_desc.remove_sub_element_kind(ElementName::EndToEndTransformationDescription);
                let someip_desc = trans_desc.get_or_create_sub_element(ElementName::SomeipTransformationDescription)?;
                someip_desc
                    .get_or_create_sub_element(ElementName::Alignment)?
                    .set_character_data(u64::from(someip_config.alignment))?;
                someip_desc
                    .get_or_create_sub_element(ElementName::ByteOrder)?
                    .set_character_data::<EnumItem>(someip_config.byte_order.into())?;
                someip_desc
                    .get_or_create_sub_element(ElementName::InterfaceVersion)?
                    .set_character_data(u64::from(someip_config.interface_version))?;

                // the someip transformer must currently always use version "1.0.0"
                ttech
                    .get_or_create_sub_element(ElementName::Version)?
                    .set_character_data("1.0.0")?;

                // remove the buffer computation, which is only used by COM
                let _ = buffer_props.remove_sub_element_kind(ElementName::BufferComputation);
                // remove the needs original data setting, which is only used by E2E
                let _ = ttech.remove_sub_element_kind(ElementName::NeedsOriginalData);
            }
        }

        Ok(())
    }

    /// Get the protocol of the `TransformationTechnology`
    #[must_use]
    pub fn protocol(&self) -> Option<String> {
        self.element()
            .get_sub_element(ElementName::Protocol)?
            .character_data()?
            .string_value()
    }

    /// Get the transformer class of the `TransformationTechnology`
    #[must_use]
    pub fn transformer_class(&self) -> Option<EnumItem> {
        self.element()
            .get_sub_element(ElementName::TransformerClass)?
            .character_data()?
            .enum_value()
    }

    /// get the `DataTransformationSet` that contains this `TransformationTechnology`
    #[must_use]
    pub fn data_transformation_set(&self) -> Option<DataTransformationSet> {
        self.element()
            .named_parent()
            .ok()?
            .and_then(|dts| DataTransformationSet::try_from(dts).ok())
    }

    /// Get the configuration of the `TransformationTechnology`
    #[must_use]
    pub fn config(&self) -> Option<TransformationTechnologyConfig> {
        let protocol = self
            .element()
            .get_sub_element(ElementName::Protocol)?
            .character_data()?
            .string_value()?;

        let opt_e2e_desc = self
            .element()
            .get_sub_element(ElementName::TransformationDescriptions)
            .and_then(|tdesc| tdesc.get_sub_element(ElementName::EndToEndTransformationDescription));
        let opt_someip_desc = self
            .element()
            .get_sub_element(ElementName::TransformationDescriptions)
            .and_then(|tdesc| tdesc.get_sub_element(ElementName::SomeipTransformationDescription));

        if let Some(e2e_desc) = opt_e2e_desc {
            // E2E transformation
            let profile_name = e2e_desc
                .get_sub_element(ElementName::ProfileName)?
                .character_data()?
                .string_value()?;
            let profile = match profile_name.as_str() {
                "PROFILE_01" => E2EProfile::P01,
                "PROFILE_02" => E2EProfile::P02,
                "PROFILE_04" => E2EProfile::P04,
                "PROFILE_04m" => E2EProfile::P04m,
                "PROFILE_05" => E2EProfile::P05,
                "PROFILE_06" => E2EProfile::P06,
                "PROFILE_07" => E2EProfile::P07,
                "PROFILE_07m" => E2EProfile::P07m,
                "PROFILE_08" => E2EProfile::P08,
                "PROFILE_08m" => E2EProfile::P08m,
                "PROFILE_11" => E2EProfile::P11,
                "PROFILE_22" => E2EProfile::P22,
                "PROFILE_44" => E2EProfile::P44,
                "PROFILE_44m" => E2EProfile::P44m,
                _ => return None,
            };

            let buffer_props = self.element().get_sub_element(ElementName::BufferProperties)?;
            let in_place = buffer_props
                .get_sub_element(ElementName::InPlace)?
                .character_data()?
                .parse_bool()?;
            let buffer_header_length = buffer_props
                .get_sub_element(ElementName::HeaderLength)?
                .character_data()?
                .parse_integer::<u32>()?;

            let opt_window_size_init = e2e_desc
                .get_sub_element(ElementName::WindowSizeInit)
                .and_then(|elem| elem.character_data())
                .and_then(|cdata| cdata.parse_integer::<u32>());
            let opt_window_size_invalid = e2e_desc
                .get_sub_element(ElementName::WindowSizeInvalid)
                .and_then(|elem| elem.character_data())
                .and_then(|cdata| cdata.parse_integer::<u32>());
            let opt_window_size_valid = e2e_desc
                .get_sub_element(ElementName::WindowSizeValid)
                .and_then(|elem| elem.character_data())
                .and_then(|cdata| cdata.parse_integer::<u32>());
            // window size is one value in AUTOSAR 4.4.0 (AUTOSAR_00047) and older, and three values in AUTOSAR 4.5.0 (AUTOSAR_00048) and newer
            // when getting the config, first try to use the WINDOW-SIZE, then fall back to the three separate values
            let opt_window_size = e2e_desc
                .get_sub_element(ElementName::WindowSize)
                .and_then(|elem| elem.character_data())
                .and_then(|cdata| cdata.parse_integer())
                .or(opt_window_size_init)
                .or(opt_window_size_invalid)
                .or(opt_window_size_valid);

            let config = E2ETransformationTechnologyConfig {
                profile,
                zero_header_length: buffer_header_length == 0,
                transform_in_place: in_place,
                offset: 0,
                max_delta_counter: e2e_desc
                    .get_sub_element(ElementName::MaxDeltaCounter)?
                    .character_data()?
                    .parse_integer()?,
                max_error_state_init: e2e_desc
                    .get_sub_element(ElementName::MaxErrorStateInit)?
                    .character_data()?
                    .parse_integer()?,
                max_error_state_invalid: e2e_desc
                    .get_sub_element(ElementName::MaxErrorStateInvalid)?
                    .character_data()?
                    .parse_integer()?,
                max_error_state_valid: e2e_desc
                    .get_sub_element(ElementName::MaxErrorStateValid)?
                    .character_data()?
                    .parse_integer()?,
                max_no_new_or_repeated_data: e2e_desc
                    .get_sub_element(ElementName::MaxNoNewOrRepeatedData)?
                    .character_data()?
                    .parse_integer()?,
                min_ok_state_init: e2e_desc
                    .get_sub_element(ElementName::MinOkStateInit)?
                    .character_data()?
                    .parse_integer()?,
                min_ok_state_invalid: e2e_desc
                    .get_sub_element(ElementName::MinOkStateInvalid)?
                    .character_data()?
                    .parse_integer()?,
                min_ok_state_valid: e2e_desc
                    .get_sub_element(ElementName::MinOkStateValid)?
                    .character_data()?
                    .parse_integer()?,
                window_size: opt_window_size?,
                window_size_init: e2e_desc
                    .get_sub_element(ElementName::WindowSizeInit)
                    .and_then(|elem| elem.character_data())
                    .and_then(|cd| cd.parse_integer()),
                window_size_invalid: e2e_desc
                    .get_sub_element(ElementName::WindowSizeInvalid)
                    .and_then(|elem| elem.character_data())
                    .and_then(|cd| cd.parse_integer()),
                window_size_valid: e2e_desc
                    .get_sub_element(ElementName::WindowSizeValid)
                    .and_then(|elem| elem.character_data())
                    .and_then(|cd| cd.parse_integer()),
                profile_behavior: e2e_desc
                    .get_sub_element(ElementName::ProfileBehavior)
                    .and_then(|elem| elem.character_data())
                    .and_then(|cd| cd.enum_value())
                    .and_then(|enumitem| enumitem.try_into().ok()),
                sync_counter_init: e2e_desc
                    .get_sub_element(ElementName::SyncCounterInit)
                    .and_then(|elem| elem.character_data())
                    .and_then(|cd| cd.parse_integer()),
                data_id_mode: e2e_desc
                    .get_sub_element(ElementName::DataIdMode)
                    .and_then(|elem| elem.character_data())
                    .and_then(|cd| cd.enum_value())
                    .and_then(|enumitem| enumitem.try_into().ok()),
                data_id_nibble_offset: e2e_desc
                    .get_sub_element(ElementName::DataIdNibbleOffset)
                    .and_then(|elem| elem.character_data())
                    .and_then(|cd| cd.parse_integer()),
                crc_offset: e2e_desc
                    .get_sub_element(ElementName::CrcOffset)
                    .and_then(|elem| elem.character_data())
                    .and_then(|cd| cd.parse_integer()),
                counter_offset: e2e_desc
                    .get_sub_element(ElementName::CounterOffset)
                    .and_then(|elem| elem.character_data())
                    .and_then(|cd| cd.parse_integer()),
            };

            Some(TransformationTechnologyConfig::E2E(config))
        } else if let Some(someip_desc) = opt_someip_desc {
            // SOMEIP transformation
            let someip_config = SomeIpTransformationTechnologyConfig {
                alignment: someip_desc
                    .get_sub_element(ElementName::Alignment)?
                    .character_data()?
                    .parse_integer()?,
                byte_order: someip_desc
                    .get_sub_element(ElementName::ByteOrder)?
                    .character_data()?
                    .enum_value()
                    .and_then(|enumitem| enumitem.try_into().ok())?,
                interface_version: someip_desc
                    .get_sub_element(ElementName::InterfaceVersion)?
                    .character_data()?
                    .parse_integer()?,
            };
            Some(TransformationTechnologyConfig::SomeIp(someip_config))
        } else if protocol == "COMBased" || protocol == "ComBased" {
            // COM transformation
            let com_config = ComTransformationTechnologyConfig {
                isignal_ipdu_length: self
                    .element()
                    .get_sub_element(ElementName::BufferProperties)?
                    .get_sub_element(ElementName::BufferComputation)?
                    .get_sub_element(ElementName::CompuRationalCoeffs)?
                    .get_sub_element(ElementName::CompuNumerator)?
                    .get_sub_element(ElementName::V)?
                    .character_data()?
                    .parse_integer()?,
            };
            Some(TransformationTechnologyConfig::Com(com_config))
        } else {
            // generic transformation
            let buffer_props = self.element().get_sub_element(ElementName::BufferProperties)?;
            let in_place = buffer_props
                .get_sub_element(ElementName::InPlace)?
                .character_data()?
                .parse_bool()?;

            let generic_config = GenericTransformationTechnologyConfig {
                protocol_name: self
                    .element()
                    .get_sub_element(ElementName::Protocol)?
                    .character_data()?
                    .string_value()?,
                protocol_version: self
                    .element()
                    .get_sub_element(ElementName::Version)?
                    .character_data()?
                    .string_value()?,
                header_length: buffer_props
                    .get_sub_element(ElementName::HeaderLength)?
                    .character_data()?
                    .parse_integer()?,
                in_place,
            };
            Some(TransformationTechnologyConfig::Generic(generic_config))
        }
    }
}

//#########################################################

/// The configuration of any kind of `TransformationTechnology`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransformationTechnologyConfig {
    /// Configuration for a generic transformation technology
    Generic(GenericTransformationTechnologyConfig),
    /// Configuration for a COM transformation technology
    Com(ComTransformationTechnologyConfig),
    /// Configuration for an E2E transformation technology
    E2E(E2ETransformationTechnologyConfig),
    /// Configuration for a SOMEIP transformation technology
    SomeIp(SomeIpTransformationTechnologyConfig),
}

//#########################################################

/// Configuration for a generic transformation technology
/// For a generic trasformation, the mandatory values must be chosen by the user
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericTransformationTechnologyConfig {
    /// The name of the custom protocol
    pub protocol_name: String,
    /// The version of the custom protocol
    pub protocol_version: String,
    /// The length of the header in bits
    pub header_length: u32,
    /// Should the transformation take place in the existing buffer or in a separate buffer?
    pub in_place: bool,
}

//#########################################################

/// Configuration for a COM transformation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComTransformationTechnologyConfig {
    /// The length of the `ISignalIpdu` tha will be transformed by this Com transformer.
    /// The value is only used up to AUTOSAR R20-11 (`AUTOSAR_00049`), where it is needed to calculate the buffer size.
    pub isignal_ipdu_length: u32,
}

//#########################################################

/// Configuration for an E2E transformation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct E2ETransformationTechnologyConfig {
    /// E2E profile to use
    pub profile: E2EProfile,
    /// When E2E is used in a transformer chain after COM, the header length must be zero.
    /// In this configuration you are expected to provide space for the E2E data inside the signal group layout, and `zero_header_length` should be set to true.
    /// If `zero_header_length` is set to false, the appropriate header length for the chosen E2E profile will be used (e.g. 24 bits for `PROFILE_05`)
    pub zero_header_length: bool,
    /// Should the E2E transformation take place in the existing buffer or in a separate buffer?
    pub transform_in_place: bool,
    /// The offset in bits from the start of the buffer where the E2E data should be placed
    /// If E2E is used after COM, the offset should be 0; if E2E is used after SOMEIP, the offset should be 64
    pub offset: u32,
    /// Maximum jump in the counter value between two consecutive messages
    pub max_delta_counter: u32,
    /// The maximum allowed number of consecutive failed counter checks in the init state
    pub max_error_state_init: u32,
    /// The maximum allowed number of consecutive failed counter checks in the invalid state
    pub max_error_state_invalid: u32,
    /// The maximum allowed number of consecutive failed counter checks in the valid state
    pub max_error_state_valid: u32,
    /// The maximum allowed number of consecutive failed counter checks
    pub max_no_new_or_repeated_data: u32,
    /// The minimum allowed number of consecutive successful counter checks in the init state
    pub min_ok_state_init: u32,
    /// The minimum allowed number of consecutive successful counter checks in the invalid state
    pub min_ok_state_invalid: u32,
    /// The minimum allowed number of consecutive successful counter checks in the valid state
    pub min_ok_state_valid: u32,
    /// window size: Size of the monitoring window for the E2E state machine.
    /// This can be directly set up to AUTOSAR 4.4.0 (`AUTOSAR_00047`).
    /// For newer files this only provides the default if `window_size_init`, `window_size_invalid` and `window_size_valid` are not set
    pub window_size: u32,
    /// window size in the init state - only valid in AUTOSAR 4.5.0 (`AUTOSAR_00048`) and newer. if it is not set, this will default to `window_size`
    pub window_size_init: Option<u32>,
    /// window size in the invalid state - only valid in AUTOSAR 4.5.0 (`AUTOSAR_00048`) and newer. if it is not set, this will default to `window_size`
    pub window_size_invalid: Option<u32>,
    /// window size in the valid state - only valid in AUTOSAR 4.5.0 (`AUTOSAR_00048`) and newer. if it is not set, this will default to `window_size`
    pub window_size_valid: Option<u32>,
    /// Behavior of the check functionality
    pub profile_behavior: Option<E2EProfileBehavior>,
    /// Number of successful checks required for validating the consistency of the counter
    pub sync_counter_init: Option<u32>,
    /// The data ID mode to use; required for E2E profiles 01 and 11, unused otherwise
    pub data_id_mode: Option<DataIdMode>,
    /// Offset of the data ID in the Data[] array in bits. Required for E2E profiles 01 and 11 when `data_id_mode` is `Lower12Bit`, unused otherwise
    pub data_id_nibble_offset: Option<u32>,
    /// Offset of the crc in the Data[] array in bits. Required for E2E profiles 01 and 11, unused otherwise
    pub crc_offset: Option<u32>,
    /// Offset of the counter in the Data[] array in bits. Required for E2E profiles 01 and 11, unused otherwise
    pub counter_offset: Option<u32>,
}

//#########################################################

/// Configuration for a SOMEIP transformation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SomeIpTransformationTechnologyConfig {
    /// The alignment of the data in bits
    pub alignment: u32,
    /// The byte order of the data
    pub byte_order: ByteOrder,
    /// The interface version the SOME/IP transformer shall use.
    pub interface_version: u32,
}

//#########################################################

/// enumeration of the possible E2E profiles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum E2EProfile {
    /// E2E Profile 01: Legacy profile, uses a 4-bit counter, 16-bit data id and an 8-bit CRC. New projects should use P11 instead.
    P01,
    /// E2E Profile 02: Legacy profile, uses a 8-bit counter, 8-bit data id and a 8-bit CRC. New projects should use P22 instead.
    P02,
    /// E2E Profile 04: Uses an 16-bit length, 16-bit counter, 32-bit data id and a 32-bit CRC
    P04,
    /// E2E Profile 04m: Uses an 16-bit length, 16-bit counter, 32-bit data id and a 32-bit CRC, as well as source ID, message type and message result
    P04m,
    /// E2E Profile 05: Uses an 8-bit counter, 16-bit data id and a 16-bit CRC
    P05,
    /// E2E Profile 06: Uses a 16-bit length, 8-bit counter, 16-bit data id and a 16-bit CRC
    P06,
    /// E2E Profile 07: Uses an 32-bit length, 32-bit counter, 32-bit data id and a 64-bit CRC
    P07,
    /// E2E Profile 07: Uses an 32-bit length, 32-bit counter, 32-bit data id and a 64-bit CRC, as well as source ID, message type and message result
    P07m,
    /// E2E Profile 08: Uses an 32-bit length, 32-bit counter, 32-bit data id and a 32-bit CRC
    P08,
    /// E2E Profile 08m: Uses an 32-bit length, 32-bit counter, 32-bit data id and a 32-bit CRC, as well as source ID, message type and message result
    P08m,
    /// E2E Profile 11: Uses an 4-bit counter, 16-bit or 12-bit data id and a 8-bit CRC
    P11,
    /// E2E Profile 22: Uses a 4-bit counter, 8-bit data id and a 8-bit CRC
    P22,
    /// E2E Profile 44: Uses a 16-bit length, 16-bit counter, 32-bit data id and a 32-bit CRC
    P44,
    /// E2E Profile 44m: Uses a 16-bit length, 16-bit counter, 32-bit data id and a 32-bit CRC, as well as source ID, message type and message result
    P44m,
}

//#########################################################

/// there are two standardized behaviors for E2E profiles, which can be selected for each E2E transformation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum E2EProfileBehavior {
    /// Pre Autosar-R4.2 behavior
    PreR4_2,
    /// behavior according to Autosar-R4.2 and newer
    R4_2,
}

impl From<E2EProfileBehavior> for EnumItem {
    fn from(e2e_profile_behavior: E2EProfileBehavior) -> EnumItem {
        match e2e_profile_behavior {
            E2EProfileBehavior::PreR4_2 => EnumItem::PreR4_2,
            E2EProfileBehavior::R4_2 => EnumItem::R4_2,
        }
    }
}

impl TryFrom<EnumItem> for E2EProfileBehavior {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<E2EProfileBehavior, AutosarAbstractionError> {
        match value {
            EnumItem::PreR4_2 => Ok(E2EProfileBehavior::PreR4_2),
            EnumItem::R4_2 => Ok(E2EProfileBehavior::R4_2),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "E2EProfileBehavior".to_string(),
            }),
        }
    }
}

//#########################################################

/// data ID modes for E2E profiles 01 and 11
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataIdMode {
    /// Two bytes of the data id are included in the CRC (double ID configuration).
    All16Bit,
    /// The data id is split into two 8-bit parts, which are included in the CRC in an alternating manner.
    Alternating8Bit,
    /// The low byte is included in the implicit CRC calculation, the low nibble of the high byte is transmitted along with the data
    Lower12Bit,
    /// Only the low byte is included, the high byte is never used
    Lower8Bit,
}

impl From<DataIdMode> for EnumItem {
    fn from(data_id_mode: DataIdMode) -> EnumItem {
        match data_id_mode {
            DataIdMode::All16Bit => EnumItem::All16Bit,
            DataIdMode::Alternating8Bit => EnumItem::Alternating8Bit,
            DataIdMode::Lower12Bit => EnumItem::Lower12Bit,
            DataIdMode::Lower8Bit => EnumItem::Lower8Bit,
        }
    }
}

impl TryFrom<EnumItem> for DataIdMode {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<DataIdMode, AutosarAbstractionError> {
        match value {
            EnumItem::All16Bit => Ok(DataIdMode::All16Bit),
            EnumItem::Alternating8Bit => Ok(DataIdMode::Alternating8Bit),
            EnumItem::Lower12Bit => Ok(DataIdMode::Lower12Bit),
            EnumItem::Lower8Bit => Ok(DataIdMode::Lower8Bit),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "DataIdMode".to_string(),
            }),
        }
    }
}

//#########################################################

/// message types that can be used in a SOME/IP message header, depending on the type of communication
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SomeIpMessageType {
    /// Notification message
    Notification,
    /// Request message, which expects a response
    Request,
    /// Request without return message - a fire and forget message
    RequestNoReturn,
    /// Response message
    Response,
}

impl From<SomeIpMessageType> for EnumItem {
    fn from(someip_msg_type: SomeIpMessageType) -> EnumItem {
        match someip_msg_type {
            SomeIpMessageType::Notification => EnumItem::Notification,
            SomeIpMessageType::Request => EnumItem::Request,
            SomeIpMessageType::RequestNoReturn => EnumItem::RequestNoReturn,
            SomeIpMessageType::Response => EnumItem::Response,
        }
    }
}

impl TryFrom<EnumItem> for SomeIpMessageType {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<SomeIpMessageType, AutosarAbstractionError> {
        match value {
            EnumItem::Notification => Ok(SomeIpMessageType::Notification),
            EnumItem::Request => Ok(SomeIpMessageType::Request),
            EnumItem::RequestNoReturn => Ok(SomeIpMessageType::RequestNoReturn),
            EnumItem::Response => Ok(SomeIpMessageType::Response),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "SomeIpMessageType".to_string(),
            }),
        }
    }
}

//#########################################################

/// Properties for the End to End transformation of an ISignal(Group)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndToEndTransformationISignalProps(Element);
abstraction_element!(EndToEndTransformationISignalProps, EndToEndTransformationISignalProps);

impl EndToEndTransformationISignalProps {
    pub(crate) fn new(
        parent_element: Element,
        transformer: &TransformationTechnology,
    ) -> Result<Self, AutosarAbstractionError> {
        if transformer.protocol().as_deref() != Some("E2E") {
            return Err(AutosarAbstractionError::InvalidParameter(
                "EndToEndTransformationISignalProps must reference a E2E transformer".to_string(),
            ));
        }
        let e2e_props_elem = parent_element.create_sub_element(ElementName::EndToEndTransformationISignalProps)?;

        let e2e_props = Self(e2e_props_elem);
        e2e_props.set_transformer(transformer)?;

        Ok(e2e_props)
    }

    fn inner_element(&self) -> Option<Element> {
        self.0
            .get_sub_element(ElementName::EndToEndTransformationISignalPropsVariants)?
            .get_sub_element(ElementName::EndToEndTransformationISignalPropsConditional)
    }

    fn create_inner_element(&self) -> Result<Element, AutosarAbstractionError> {
        let e2e_props_elem = self
            .element()
            .get_or_create_sub_element(ElementName::EndToEndTransformationISignalPropsVariants)?
            .get_or_create_sub_element(ElementName::EndToEndTransformationISignalPropsConditional)?;
        Ok(e2e_props_elem)
    }

    /// set the transformer reference of the E2E transformation properties
    pub fn set_transformer(&self, transformer: &TransformationTechnology) -> Result<(), AutosarAbstractionError> {
        if transformer.protocol().as_deref() != Some("E2E") {
            return Err(AutosarAbstractionError::InvalidParameter(
                "EndToEndTransformationISignalProps must reference a E2E transformer".to_string(),
            ));
        }
        self.create_inner_element()?
            .get_or_create_sub_element(ElementName::TransformerRef)?
            .set_reference_target(transformer.element())?;
        Ok(())
    }

    /// get the transformer reference of the E2E transformation properties
    #[must_use]
    pub fn transformer(&self) -> Option<TransformationTechnology> {
        let t_elem = self
            .inner_element()?
            .get_sub_element(ElementName::TransformerRef)?
            .get_reference_target()
            .ok()?;
        TransformationTechnology::try_from(t_elem).ok()
    }

    /// set the data IDs that are used for the E2E transformation
    pub fn set_data_ids(&self, data_ids: &[u32]) -> Result<(), AutosarAbstractionError> {
        if data_ids.is_empty() {
            let _ = self
                .inner_element()
                .and_then(|inner| inner.remove_sub_element_kind(ElementName::DataIds).ok());
        } else {
            let data_ids_elem = self.create_inner_element()?.create_sub_element(ElementName::DataIds)?;
            for data_id in data_ids {
                data_ids_elem
                    .create_sub_element(ElementName::DataId)?
                    .set_character_data(u64::from(*data_id))?;
            }
        }
        Ok(())
    }

    /// get the data IDs that are used for the E2E transformation
    #[must_use]
    pub fn data_ids(&self) -> Vec<u32> {
        self.inner_element()
            .and_then(|inner_elem| inner_elem.get_sub_element(ElementName::DataIds))
            .map(|elem| {
                elem.sub_elements()
                    .filter_map(|elem| elem.character_data().and_then(|cdata| cdata.parse_integer()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// set the length of payload and E2E header in bits
    pub fn set_data_length(&self, data_length: Option<u32>) -> Result<(), AutosarAbstractionError> {
        if let Some(data_length) = data_length {
            self.create_inner_element()?
                .get_or_create_sub_element(ElementName::DataLength)?
                .set_character_data(data_length.to_string())?;
        } else {
            let _ = self
                .inner_element()
                .and_then(|inner| inner.remove_sub_element_kind(ElementName::DataLength).ok());
        }
        Ok(())
    }

    /// get the length of payload and E2E header in bits
    #[must_use]
    pub fn data_length(&self) -> Option<u32> {
        self.inner_element()?
            .get_sub_element(ElementName::DataLength)?
            .character_data()?
            .parse_integer()
    }

    /// set the maximum data length
    pub fn set_max_data_length(&self, max_data_length: Option<u32>) -> Result<(), AutosarAbstractionError> {
        if let Some(max_data_length) = max_data_length {
            self.create_inner_element()?
                .get_or_create_sub_element(ElementName::MaxDataLength)?
                .set_character_data(max_data_length.to_string())?;
        } else {
            let _ = self
                .inner_element()
                .and_then(|inner| inner.remove_sub_element_kind(ElementName::MaxDataLength).ok());
        }
        Ok(())
    }

    /// get the maximum data length
    #[must_use]
    pub fn max_data_length(&self) -> Option<u32> {
        self.inner_element()?
            .get_sub_element(ElementName::MaxDataLength)
            .and_then(|elem| elem.character_data())
            .and_then(|cdata| cdata.parse_integer())
    }

    /// set the minimum data length
    pub fn set_min_data_length(&self, min_data_length: Option<u32>) -> Result<(), AutosarAbstractionError> {
        if let Some(min_data_length) = min_data_length {
            self.create_inner_element()?
                .get_or_create_sub_element(ElementName::MinDataLength)?
                .set_character_data(min_data_length.to_string())?;
        } else {
            let _ = self
                .inner_element()
                .and_then(|inner| inner.remove_sub_element_kind(ElementName::MinDataLength).ok());
        }
        Ok(())
    }

    /// get the minimum data length
    #[must_use]
    pub fn min_data_length(&self) -> Option<u32> {
        self.inner_element()?
            .get_sub_element(ElementName::MinDataLength)?
            .character_data()?
            .parse_integer()
    }

    /// set the source ID
    pub fn set_source_id(&self, source_id: Option<u32>) -> Result<(), AutosarAbstractionError> {
        if let Some(source_id) = source_id {
            self.create_inner_element()?
                .get_or_create_sub_element(ElementName::SourceId)?
                .set_character_data(source_id.to_string())?;
        } else {
            let _ = self
                .inner_element()
                .and_then(|inner| inner.remove_sub_element_kind(ElementName::SourceId).ok());
        }
        Ok(())
    }

    /// get the source ID
    #[must_use]
    pub fn source_id(&self) -> Option<u32> {
        self.inner_element()?
            .get_sub_element(ElementName::SourceId)?
            .character_data()?
            .parse_integer()
    }
}

//#########################################################

/// Properties for the SOMEIP transformation of an ISignal(Group)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SomeIpTransformationISignalProps(Element);
abstraction_element!(SomeIpTransformationISignalProps, SomeipTransformationISignalProps);

impl SomeIpTransformationISignalProps {
    pub(crate) fn new(
        parent_element: Element,
        transformer: &TransformationTechnology,
    ) -> Result<Self, AutosarAbstractionError> {
        if transformer.protocol().as_deref() != Some("SOMEIP") {
            return Err(AutosarAbstractionError::InvalidParameter(
                "SOMEIPTransformationISignalProps must reference a SOMEIP transformer".to_string(),
            ));
        }
        let someip_props_elem = parent_element.create_sub_element(ElementName::SomeipTransformationISignalProps)?;
        let someip_props = Self(someip_props_elem);
        someip_props.set_transformer(transformer)?;

        Ok(someip_props)
    }

    fn inner_element(&self) -> Option<Element> {
        self.0
            .get_sub_element(ElementName::SomeipTransformationISignalPropsVariants)?
            .get_sub_element(ElementName::SomeipTransformationISignalPropsConditional)
    }

    fn create_inner_element(&self) -> Result<Element, AutosarAbstractionError> {
        let e2e_props_elem = self
            .element()
            .get_or_create_sub_element(ElementName::SomeipTransformationISignalPropsVariants)?
            .get_or_create_sub_element(ElementName::SomeipTransformationISignalPropsConditional)?;
        Ok(e2e_props_elem)
    }

    /// set the transformer reference of the SOMEIP transformation properties
    pub fn set_transformer(&self, transformer: &TransformationTechnology) -> Result<(), AutosarAbstractionError> {
        if transformer.protocol().as_deref() != Some("SOMEIP") {
            return Err(AutosarAbstractionError::InvalidParameter(
                "SOMEIPTransformationISignalProps must reference a SOMEIP transformer".to_string(),
            ));
        }
        self.create_inner_element()?
            .get_or_create_sub_element(ElementName::TransformerRef)?
            .set_reference_target(transformer.element())?;
        Ok(())
    }

    /// get the transformer reference of the SOMEIP transformation properties
    #[must_use]
    pub fn transformer(&self) -> Option<TransformationTechnology> {
        let t_elem = self
            .inner_element()?
            .get_sub_element(ElementName::TransformerRef)?
            .get_reference_target()
            .ok()?;
        TransformationTechnology::try_from(t_elem).ok()
    }

    /// set the legacy strings property
    pub fn set_legacy_strings(&self, legacy_strings: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(legacy_strings) = legacy_strings {
            self.create_inner_element()?
                .get_or_create_sub_element(ElementName::ImplementsLegacyStringSerialization)?
                .set_character_data(legacy_strings.to_string())?;
        } else {
            let _ = self.inner_element().and_then(|inner| {
                inner
                    .remove_sub_element_kind(ElementName::ImplementsLegacyStringSerialization)
                    .ok()
            });
        }
        Ok(())
    }

    /// get the legacy strings property
    #[must_use]
    pub fn legacy_strings(&self) -> Option<bool> {
        self.inner_element()?
            .get_sub_element(ElementName::ImplementsLegacyStringSerialization)?
            .character_data()?
            .parse_bool()
    }

    /// set the interface version property
    pub fn set_interface_version(&self, interface_version: Option<u32>) -> Result<(), AutosarAbstractionError> {
        if let Some(interface_version) = interface_version {
            self.create_inner_element()?
                .get_or_create_sub_element(ElementName::InterfaceVersion)?
                .set_character_data(interface_version.to_string())?;
        } else {
            let _ = self
                .inner_element()
                .and_then(|inner| inner.remove_sub_element_kind(ElementName::InterfaceVersion).ok());
        }
        Ok(())
    }

    /// get the interface version property
    #[must_use]
    pub fn interface_version(&self) -> Option<u32> {
        self.inner_element()?
            .get_sub_element(ElementName::InterfaceVersion)?
            .character_data()?
            .parse_integer()
    }

    /// set the dynamic length property
    pub fn set_dynamic_length(&self, dynamic_length: Option<bool>) -> Result<(), AutosarAbstractionError> {
        if let Some(dynamic_length) = dynamic_length {
            self.create_inner_element()?
                .get_or_create_sub_element(ElementName::IsDynamicLengthFieldSize)?
                .set_character_data(dynamic_length.to_string())?;
        } else {
            let _ = self.inner_element().and_then(|inner| {
                inner
                    .remove_sub_element_kind(ElementName::IsDynamicLengthFieldSize)
                    .ok()
            });
        }
        Ok(())
    }

    /// get the dynamic length property
    #[must_use]
    pub fn dynamic_length(&self) -> Option<bool> {
        self.inner_element()?
            .get_sub_element(ElementName::IsDynamicLengthFieldSize)?
            .character_data()?
            .parse_bool()
    }

    /// set the message type property
    pub fn set_message_type(&self, message_type: Option<SomeIpMessageType>) -> Result<(), AutosarAbstractionError> {
        if let Some(message_type) = message_type {
            self.create_inner_element()?
                .get_or_create_sub_element(ElementName::MessageType)?
                .set_character_data::<EnumItem>(message_type.into())?;
        } else {
            let _ = self
                .inner_element()
                .and_then(|inner| inner.remove_sub_element_kind(ElementName::MessageType).ok());
        }
        Ok(())
    }

    /// get the message type property
    #[must_use]
    pub fn message_type(&self) -> Option<SomeIpMessageType> {
        self.inner_element()?
            .get_sub_element(ElementName::MessageType)?
            .character_data()?
            .enum_value()?
            .try_into()
            .ok()
    }

    /// set the size of array length property
    pub fn set_size_of_array_length(&self, size_of_array_length: Option<u32>) -> Result<(), AutosarAbstractionError> {
        if let Some(size_of_array_length) = size_of_array_length {
            self.create_inner_element()?
                .get_or_create_sub_element(ElementName::SizeOfArrayLengthFields)?
                .set_character_data(size_of_array_length.to_string())?;
        } else {
            let _ = self
                .inner_element()
                .and_then(|inner| inner.remove_sub_element_kind(ElementName::SizeOfArrayLengthFields).ok());
        }
        Ok(())
    }

    /// get the size of array length property
    #[must_use]
    pub fn size_of_array_length(&self) -> Option<u32> {
        self.inner_element()?
            .get_sub_element(ElementName::SizeOfArrayLengthFields)?
            .character_data()?
            .parse_integer()
    }

    /// set the size of string length property
    pub fn set_size_of_string_length(&self, size_of_string_length: Option<u32>) -> Result<(), AutosarAbstractionError> {
        if let Some(size_of_string_length) = size_of_string_length {
            self.create_inner_element()?
                .get_or_create_sub_element(ElementName::SizeOfStringLengthFields)?
                .set_character_data(size_of_string_length.to_string())?;
        } else {
            let _ = self.inner_element().and_then(|inner| {
                inner
                    .remove_sub_element_kind(ElementName::SizeOfStringLengthFields)
                    .ok()
            });
        }
        Ok(())
    }

    /// get the size of string length property
    #[must_use]
    pub fn size_of_string_length(&self) -> Option<u32> {
        self.inner_element()?
            .get_sub_element(ElementName::SizeOfStringLengthFields)?
            .character_data()?
            .parse_integer()
    }

    /// set the size of struct length property
    pub fn set_size_of_struct_length(&self, size_of_struct_length: Option<u32>) -> Result<(), AutosarAbstractionError> {
        if let Some(size_of_struct_length) = size_of_struct_length {
            self.create_inner_element()?
                .get_or_create_sub_element(ElementName::SizeOfStructLengthFields)?
                .set_character_data(size_of_struct_length.to_string())?;
        } else {
            let _ = self.inner_element().and_then(|inner| {
                inner
                    .remove_sub_element_kind(ElementName::SizeOfStructLengthFields)
                    .ok()
            });
        }
        Ok(())
    }

    /// get the size of struct length property
    #[must_use]
    pub fn size_of_struct_length(&self) -> Option<u32> {
        self.inner_element()?
            .get_sub_element(ElementName::SizeOfStructLengthFields)?
            .character_data()?
            .parse_integer()
    }

    /// set the size of union length property
    pub fn set_size_of_union_length(&self, size_of_union_length: Option<u32>) -> Result<(), AutosarAbstractionError> {
        if let Some(size_of_union_length) = size_of_union_length {
            self.create_inner_element()?
                .get_or_create_sub_element(ElementName::SizeOfUnionLengthFields)?
                .set_character_data(size_of_union_length.to_string())?;
        } else {
            let _ = self
                .inner_element()
                .and_then(|inner| inner.remove_sub_element_kind(ElementName::SizeOfUnionLengthFields).ok());
        }
        Ok(())
    }

    /// get the size of union length property
    #[must_use]
    pub fn size_of_union_length(&self) -> Option<u32> {
        self.inner_element()?
            .get_sub_element(ElementName::SizeOfUnionLengthFields)?
            .character_data()?
            .parse_integer()
    }
}

//#########################################################

/// Wrapper enum for the properties for the transformation of an ISignal(Group)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransformationISignalProps {
    /// Properties for the End to End transformation of an ISignal(Group)
    E2E(EndToEndTransformationISignalProps),
    /// Properties for the SOMEIP transformation of an ISignal(Group)
    SomeIp(SomeIpTransformationISignalProps),
}

impl AbstractionElement for TransformationISignalProps {
    fn element(&self) -> &Element {
        match self {
            TransformationISignalProps::E2E(e2e_props) => e2e_props.element(),
            TransformationISignalProps::SomeIp(someip_props) => someip_props.element(),
        }
    }
}

impl TryFrom<Element> for TransformationISignalProps {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::EndToEndTransformationISignalProps => {
                EndToEndTransformationISignalProps::try_from(element).map(TransformationISignalProps::E2E)
            }
            ElementName::SomeipTransformationISignalProps => {
                SomeIpTransformationISignalProps::try_from(element).map(TransformationISignalProps::SomeIp)
            }
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "TransformationISignalProps".to_string(),
            }),
        }
    }
}

//#########################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        AutosarModelAbstraction,
        communication::{ISignal, SystemSignal},
        datatype::{BaseTypeEncoding, SwBaseType},
    };

    #[test]
    fn transformation_technologies() {
        // Data Transformation Sets and transformation technologies were introduced in AUTOSAR 4.2.1
        // There have been several changes over time, so it makes sense to test with multiple versions
        let versions = vec![
            AutosarVersion::Autosar_4_2_1,
            AutosarVersion::Autosar_4_3_0,
            AutosarVersion::Autosar_00044,
            AutosarVersion::Autosar_00046,
            AutosarVersion::Autosar_00047,
            AutosarVersion::Autosar_00048,
            AutosarVersion::Autosar_00049,
            AutosarVersion::Autosar_00050,
            AutosarVersion::Autosar_00051,
            AutosarVersion::Autosar_00052,
        ];
        for version in versions {
            create_transformation_technologies(version);
        }
    }

    fn create_transformation_technologies(file_version: AutosarVersion) {
        let model = AutosarModelAbstraction::create("test", file_version);
        let package = model.get_or_create_package("/package").unwrap();
        let dts = DataTransformationSet::new("test", &package).unwrap();

        // create a generic transformation technology
        let config = TransformationTechnologyConfig::Generic(GenericTransformationTechnologyConfig {
            protocol_name: "test".to_string(),
            protocol_version: "1.0.0".to_string(),
            header_length: 16,
            in_place: true,
        });
        let ttech = dts.create_transformation_technology("generic", &config).unwrap();
        let config2 = ttech.config().unwrap();
        assert_eq!(config, config2);

        // create a COM transformation technology
        let config = TransformationTechnologyConfig::Com(ComTransformationTechnologyConfig { isignal_ipdu_length: 8 });
        dts.create_transformation_technology("com", &config).unwrap();

        // create an E2E transformation technology for each profile
        for profile in &[
            E2EProfile::P01,
            E2EProfile::P02,
            E2EProfile::P04,
            E2EProfile::P04m,
            E2EProfile::P05,
            E2EProfile::P06,
            E2EProfile::P07,
            E2EProfile::P07m,
            E2EProfile::P08,
            E2EProfile::P08m,
            E2EProfile::P11,
            E2EProfile::P22,
            E2EProfile::P44,
            E2EProfile::P44m,
        ] {
            let e2e_config = E2ETransformationTechnologyConfig {
                profile: *profile,
                zero_header_length: false,
                transform_in_place: true,
                offset: 0,
                max_delta_counter: 0,
                max_error_state_init: 0,
                max_error_state_invalid: 0,
                max_error_state_valid: 0,
                max_no_new_or_repeated_data: 0,
                min_ok_state_init: 0,
                min_ok_state_invalid: 0,
                min_ok_state_valid: 0,
                window_size: 10,
                window_size_init: Some(11),
                window_size_invalid: Some(12),
                window_size_valid: Some(13),
                profile_behavior: Some(E2EProfileBehavior::R4_2),
                sync_counter_init: Some(0),
                data_id_mode: Some(DataIdMode::Lower12Bit),
                data_id_nibble_offset: Some(1),
                crc_offset: Some(2),
                counter_offset: Some(3),
            };
            let config = TransformationTechnologyConfig::E2E(e2e_config.clone());
            let ttech = dts
                .create_transformation_technology(&format!("{profile:?}"), &config)
                .unwrap();
            let config2 = ttech.config().unwrap();
            let TransformationTechnologyConfig::E2E(e2e_config2) = config2 else {
                panic!("Expected E2E transformation technology");
            };
            assert_eq!(e2e_config.profile, e2e_config2.profile);
            assert_eq!(e2e_config.zero_header_length, e2e_config2.zero_header_length);
            assert_eq!(e2e_config.transform_in_place, e2e_config2.transform_in_place);
            assert_eq!(e2e_config.offset, e2e_config2.offset);
            assert_eq!(e2e_config.max_delta_counter, e2e_config2.max_delta_counter);
            assert_eq!(e2e_config.max_error_state_init, e2e_config2.max_error_state_init);
            assert_eq!(e2e_config.max_error_state_invalid, e2e_config2.max_error_state_invalid);
            assert_eq!(e2e_config.max_error_state_valid, e2e_config2.max_error_state_valid);
            assert_eq!(
                e2e_config.max_no_new_or_repeated_data,
                e2e_config2.max_no_new_or_repeated_data
            );
            assert_eq!(e2e_config.min_ok_state_init, e2e_config2.min_ok_state_init);
            assert_eq!(e2e_config.min_ok_state_invalid, e2e_config2.min_ok_state_invalid);
            assert_eq!(e2e_config.min_ok_state_valid, e2e_config2.min_ok_state_valid);
            if *profile == E2EProfile::P01 || *profile == E2EProfile::P11 {
                assert_eq!(e2e_config.data_id_mode, e2e_config2.data_id_mode);
                assert_eq!(e2e_config.data_id_nibble_offset, e2e_config2.data_id_nibble_offset);
            }
        }

        // create a SOMEIP transformation technology
        let config = TransformationTechnologyConfig::SomeIp(SomeIpTransformationTechnologyConfig {
            alignment: 8,
            byte_order: ByteOrder::MostSignificantByteFirst,
            interface_version: 1,
        });
        let ttech = dts.create_transformation_technology("someip", &config).unwrap();
        let config2 = ttech.config().unwrap();
        assert_eq!(config, config2);

        // verify the transformation technologies in the data transformation set
        let mut ttechs_iter = dts.transformation_technologies();
        assert_eq!(ttechs_iter.next().unwrap().name().unwrap(), "generic");
        assert_eq!(ttechs_iter.next().unwrap().name().unwrap(), "com");
    }

    #[test]
    fn data_transformation() {
        let model = AutosarModelAbstraction::create("test", AutosarVersion::Autosar_00049);
        let package = model.get_or_create_package("/package").unwrap();
        let dts = DataTransformationSet::new("test_dts", &package).unwrap();

        let e2e_transformation_config = TransformationTechnologyConfig::E2E(E2ETransformationTechnologyConfig {
            profile: E2EProfile::P01,
            zero_header_length: false,
            transform_in_place: true,
            offset: 0,
            max_delta_counter: 0,
            max_error_state_init: 0,
            max_error_state_invalid: 0,
            max_error_state_valid: 0,
            max_no_new_or_repeated_data: 0,
            min_ok_state_init: 0,
            min_ok_state_invalid: 0,
            min_ok_state_valid: 0,
            window_size: 11,
            window_size_init: Some(11),
            window_size_invalid: Some(12),
            window_size_valid: Some(13),
            profile_behavior: Some(E2EProfileBehavior::R4_2),
            sync_counter_init: Some(0),
            data_id_mode: Some(DataIdMode::All16Bit),
            data_id_nibble_offset: None,
            crc_offset: Some(2),
            counter_offset: Some(3),
        });
        let e2e_transformation = dts
            .create_transformation_technology("e2e", &e2e_transformation_config)
            .unwrap();
        assert_eq!(e2e_transformation.config().unwrap(), e2e_transformation_config);

        let com_transformation_config =
            TransformationTechnologyConfig::Com(ComTransformationTechnologyConfig { isignal_ipdu_length: 8 });
        let com_transformation = dts
            .create_transformation_technology("com", &com_transformation_config)
            .unwrap();
        assert_eq!(com_transformation.config().unwrap(), com_transformation_config);

        let someip_transformation_config =
            TransformationTechnologyConfig::SomeIp(SomeIpTransformationTechnologyConfig {
                alignment: 8,
                byte_order: ByteOrder::MostSignificantByteFirst,
                interface_version: 1,
            });
        let someip_transformation = dts
            .create_transformation_technology("someip", &someip_transformation_config)
            .unwrap();
        assert_eq!(someip_transformation.config().unwrap(), someip_transformation_config);

        let generic_transformation_config =
            TransformationTechnologyConfig::Generic(GenericTransformationTechnologyConfig {
                protocol_name: "test".to_string(),
                protocol_version: "1.0.0".to_string(),
                header_length: 16,
                in_place: true,
            });
        let generic_transformation = dts
            .create_transformation_technology("generic", &generic_transformation_config)
            .unwrap();
        assert_eq!(someip_transformation.config().unwrap(), someip_transformation_config);

        // not allowed: empty transformation chain
        let result = dts.create_data_transformation("test1", &[], true);
        assert!(result.is_err());

        // not allowed: multiple serializers Com + SomeIp
        let result = dts.create_data_transformation("test2", &[&com_transformation, &someip_transformation], true);
        assert!(result.is_err());

        // Ok: Com only
        let result = dts.create_data_transformation("test3", &[&com_transformation], true);
        assert!(result.is_ok());

        // Ok: E2E only
        let result = dts.create_data_transformation("test4", &[&e2e_transformation], true);
        assert!(result.is_ok());

        // Ok: SomeIp only
        let result = dts.create_data_transformation("test5", &[&someip_transformation], true);
        assert!(result.is_ok());

        // Ok: generic only
        let result = dts.create_data_transformation("test6", &[&generic_transformation], true);
        assert!(result.is_ok());

        // Ok: Com + E2E
        let result = dts.create_data_transformation("test7", &[&com_transformation, &e2e_transformation], true);
        assert!(result.is_ok());

        // Ok: SomeIp + E2E
        let result = dts.create_data_transformation("test8", &[&someip_transformation, &e2e_transformation], true);
        assert!(result.is_ok());

        // Ok: generic + E2E
        let result = dts.create_data_transformation("test9", &[&generic_transformation, &e2e_transformation], true);
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.data_transformation_set().unwrap(), dts);

        let dts2 = package.create_data_transformation_set("test_dts2").unwrap();

        // not allowed: dts2 using transformation from dts
        let result = dts2.create_data_transformation("test10", &[&e2e_transformation], true);
        assert!(result.is_err());

        // iterate over the data transformations
        let mut dts_iter = dts.data_transformations();
        assert_eq!(dts_iter.next().unwrap().name().unwrap(), "test3");
        assert_eq!(dts_iter.next().unwrap().name().unwrap(), "test4");
        assert_eq!(dts_iter.next().unwrap().name().unwrap(), "test5");
        assert_eq!(dts_iter.next().unwrap().name().unwrap(), "test6");
        assert_eq!(dts_iter.next().unwrap().name().unwrap(), "test7");
        assert_eq!(dts_iter.next().unwrap().name().unwrap(), "test8");
        assert_eq!(dts_iter.next().unwrap().name().unwrap(), "test9");
        assert_eq!(dts_iter.next(), None);
    }

    #[test]
    fn transformation_technology() {
        let model = AutosarModelAbstraction::create("test", AutosarVersion::Autosar_4_2_1);
        let package = model.get_or_create_package("/package").unwrap();
        let dts = DataTransformationSet::new("test_dts", &package).unwrap();

        let e2e_config_p01 = TransformationTechnologyConfig::E2E(E2ETransformationTechnologyConfig {
            profile: E2EProfile::P01,
            zero_header_length: false,
            transform_in_place: true,
            offset: 0,
            max_delta_counter: 0,
            max_error_state_init: 0,
            max_error_state_invalid: 0,
            max_error_state_valid: 0,
            max_no_new_or_repeated_data: 0,
            min_ok_state_init: 0,
            min_ok_state_invalid: 0,
            min_ok_state_valid: 0,
            window_size: 10,
            window_size_init: Some(11),
            window_size_invalid: Some(12),
            window_size_valid: Some(13),
            profile_behavior: Some(E2EProfileBehavior::R4_2),
            sync_counter_init: Some(0),
            data_id_mode: Some(DataIdMode::Lower12Bit),
            data_id_nibble_offset: Some(1),
            crc_offset: Some(2),
            counter_offset: Some(3),
        });
        let e2e_config_p04 = TransformationTechnologyConfig::E2E(E2ETransformationTechnologyConfig {
            profile: E2EProfile::P04,
            zero_header_length: false,
            transform_in_place: true,
            offset: 0,
            max_delta_counter: 0,
            max_error_state_init: 0,
            max_error_state_invalid: 0,
            max_error_state_valid: 0,
            max_no_new_or_repeated_data: 0,
            min_ok_state_init: 0,
            min_ok_state_invalid: 0,
            min_ok_state_valid: 0,
            window_size: 10,
            window_size_init: Some(11),
            window_size_invalid: Some(12),
            window_size_valid: Some(13),
            profile_behavior: Some(E2EProfileBehavior::R4_2),
            sync_counter_init: Some(0),
            data_id_mode: None,
            data_id_nibble_offset: None,
            crc_offset: None,
            counter_offset: None,
        });
        let com_config =
            TransformationTechnologyConfig::Com(ComTransformationTechnologyConfig { isignal_ipdu_length: 8 });
        let someip_config = TransformationTechnologyConfig::SomeIp(SomeIpTransformationTechnologyConfig {
            alignment: 8,
            byte_order: ByteOrder::MostSignificantByteFirst,
            interface_version: 1,
        });
        let generic_config = TransformationTechnologyConfig::Generic(GenericTransformationTechnologyConfig {
            protocol_name: "test".to_string(),
            protocol_version: "1.0.0".to_string(),
            header_length: 16,
            in_place: true,
        });

        // create a "clean" transformation technology for each type
        let transformation = dts.create_transformation_technology("t", &e2e_config_p01).unwrap();
        let e2e_p01_transformation_orig = transformation.element().serialize();
        dts.element()
            .remove_sub_element_kind(ElementName::TransformationTechnologys)
            .unwrap();

        let transformation = dts.create_transformation_technology("t", &e2e_config_p04).unwrap();
        let e2e_p04_transformation_orig = transformation.element().serialize();
        dts.element()
            .remove_sub_element_kind(ElementName::TransformationTechnologys)
            .unwrap();

        let transformation = dts.create_transformation_technology("t", &com_config).unwrap();
        let com_transformation_orig = transformation.element().serialize();
        dts.element()
            .remove_sub_element_kind(ElementName::TransformationTechnologys)
            .unwrap();

        let transformation = dts.create_transformation_technology("t", &someip_config).unwrap();
        let someip_transformation_orig = transformation.element().serialize();
        dts.element()
            .remove_sub_element_kind(ElementName::TransformationTechnologys)
            .unwrap();

        let transformation = dts.create_transformation_technology("t", &generic_config).unwrap();
        let generic_transformation_orig = transformation.element().serialize();
        dts.element()
            .remove_sub_element_kind(ElementName::TransformationTechnologys)
            .unwrap();

        // overwrite the transformation technology with another configuration, and check if the resulting xml is identical
        let transformation = dts.create_transformation_technology("t", &generic_config).unwrap();
        transformation.set_config(&e2e_config_p01).unwrap();
        assert_eq!(transformation.element().serialize(), e2e_p01_transformation_orig);
        transformation.set_config(&e2e_config_p04).unwrap();
        assert_eq!(transformation.element().serialize(), e2e_p04_transformation_orig);
        transformation.set_config(&com_config).unwrap();
        assert_eq!(transformation.element().serialize(), com_transformation_orig);
        transformation.set_config(&someip_config).unwrap();
        assert_eq!(transformation.element().serialize(), someip_transformation_orig);
        transformation.set_config(&generic_config).unwrap();
        assert_eq!(transformation.element().serialize(), generic_transformation_orig);
    }

    #[test]
    fn data_transformation_chain_iter() {
        let model = AutosarModelAbstraction::create("test", AutosarVersion::Autosar_4_2_1);
        let package = model.get_or_create_package("/package").unwrap();
        let dts = DataTransformationSet::new("test_dts", &package).unwrap();

        let com_transformation = dts
            .create_transformation_technology(
                "com",
                &TransformationTechnologyConfig::Com(ComTransformationTechnologyConfig { isignal_ipdu_length: 8 }),
            )
            .unwrap();

        let dt = dts
            .create_data_transformation("test", &[&com_transformation], false)
            .unwrap();
        assert_eq!(dt.transformation_technologies().count(), 1);
        let com_transformation2 = dt.transformation_technologies().next().unwrap();
        assert_eq!(com_transformation, com_transformation2);
    }

    #[test]
    fn transformation_isignal_props() {
        let model = AutosarModelAbstraction::create("test", AutosarVersion::Autosar_00049);
        let package = model.get_or_create_package("/package").unwrap();
        let dts = DataTransformationSet::new("test_dts", &package).unwrap();

        let e2e_transformation = dts
            .create_transformation_technology(
                "e2e",
                &TransformationTechnologyConfig::E2E(E2ETransformationTechnologyConfig {
                    profile: E2EProfile::P01,
                    zero_header_length: false,
                    transform_in_place: true,
                    offset: 0,
                    max_delta_counter: 0,
                    max_error_state_init: 0,
                    max_error_state_invalid: 0,
                    max_error_state_valid: 0,
                    max_no_new_or_repeated_data: 0,
                    min_ok_state_init: 0,
                    min_ok_state_invalid: 0,
                    min_ok_state_valid: 0,
                    window_size: 10,
                    window_size_init: Some(11),
                    window_size_invalid: Some(12),
                    window_size_valid: Some(13),
                    profile_behavior: Some(E2EProfileBehavior::R4_2),
                    sync_counter_init: Some(0),
                    data_id_mode: Some(DataIdMode::Lower12Bit),
                    data_id_nibble_offset: Some(1),
                    crc_offset: Some(2),
                    counter_offset: Some(3),
                }),
            )
            .unwrap();
        let someip_transformation = dts
            .create_transformation_technology(
                "someip",
                &TransformationTechnologyConfig::SomeIp(SomeIpTransformationTechnologyConfig {
                    alignment: 8,
                    byte_order: ByteOrder::MostSignificantByteFirst,
                    interface_version: 1,
                }),
            )
            .unwrap();

        let sw_base_type =
            SwBaseType::new("sw_base_type", &package, 8, BaseTypeEncoding::None, None, None, None).unwrap();
        let signal = ISignal::new(
            "signal",
            &package,
            8,
            &SystemSignal::new("sys_signal", &package).unwrap(),
            Some(&sw_base_type),
        )
        .unwrap();

        let e2e_props = signal
            .create_e2e_transformation_isignal_props(&e2e_transformation)
            .unwrap();
        assert_eq!(e2e_props.transformer().unwrap(), e2e_transformation);
        e2e_props.set_data_ids(&[1, 2, 3]).unwrap();
        e2e_props.set_data_length(Some(8)).unwrap();
        e2e_props.set_max_data_length(Some(16)).unwrap();
        e2e_props.set_min_data_length(Some(4)).unwrap();
        e2e_props.set_source_id(Some(0)).unwrap();
        assert_eq!(e2e_props.data_ids(), vec![1, 2, 3]);
        assert_eq!(e2e_props.data_length().unwrap(), 8);
        assert_eq!(e2e_props.max_data_length().unwrap(), 16);
        assert_eq!(e2e_props.min_data_length().unwrap(), 4);
        assert_eq!(e2e_props.source_id().unwrap(), 0);
        e2e_props.set_data_ids(&[]).unwrap();
        e2e_props.set_data_length(None).unwrap();
        e2e_props.set_max_data_length(None).unwrap();
        e2e_props.set_min_data_length(None).unwrap();
        e2e_props.set_source_id(None).unwrap();
        assert_eq!(e2e_props.data_ids(), vec![]);
        assert_eq!(e2e_props.data_length(), None);
        assert_eq!(e2e_props.max_data_length(), None);
        assert_eq!(e2e_props.min_data_length(), None);
        assert_eq!(e2e_props.source_id(), None);

        assert!(EndToEndTransformationISignalProps::try_from(e2e_props.element().clone()).is_ok());

        let someip_props = signal
            .create_someip_transformation_isignal_props(&someip_transformation)
            .unwrap();
        assert_eq!(someip_props.transformer().unwrap(), someip_transformation);
        someip_props.set_legacy_strings(Some(true)).unwrap();
        someip_props.set_interface_version(Some(1)).unwrap();
        someip_props.set_dynamic_length(Some(true)).unwrap();
        someip_props.set_message_type(Some(SomeIpMessageType::Request)).unwrap();
        someip_props.set_size_of_array_length(Some(8)).unwrap();
        someip_props.set_size_of_string_length(Some(16)).unwrap();
        someip_props.set_size_of_struct_length(Some(32)).unwrap();
        someip_props.set_size_of_union_length(Some(64)).unwrap();
        assert_eq!(someip_props.legacy_strings().unwrap(), true);
        assert_eq!(someip_props.interface_version().unwrap(), 1);
        assert_eq!(someip_props.dynamic_length().unwrap(), true);
        assert_eq!(someip_props.message_type().unwrap(), SomeIpMessageType::Request);
        assert_eq!(someip_props.size_of_array_length().unwrap(), 8);
        assert_eq!(someip_props.size_of_string_length().unwrap(), 16);
        assert_eq!(someip_props.size_of_struct_length().unwrap(), 32);
        assert_eq!(someip_props.size_of_union_length().unwrap(), 64);
        someip_props.set_legacy_strings(None).unwrap();
        someip_props.set_interface_version(None).unwrap();
        someip_props.set_dynamic_length(None).unwrap();
        someip_props.set_message_type(None).unwrap();
        someip_props.set_size_of_array_length(None).unwrap();
        someip_props.set_size_of_string_length(None).unwrap();
        someip_props.set_size_of_struct_length(None).unwrap();
        someip_props.set_size_of_union_length(None).unwrap();
        assert_eq!(someip_props.legacy_strings(), None);
        assert_eq!(someip_props.interface_version(), None);
        assert_eq!(someip_props.dynamic_length(), None);
        assert_eq!(someip_props.message_type(), None);
        assert_eq!(someip_props.size_of_array_length(), None);
        assert_eq!(someip_props.size_of_string_length(), None);
        assert_eq!(someip_props.size_of_struct_length(), None);
        assert_eq!(someip_props.size_of_union_length(), None);

        assert!(SomeIpTransformationISignalProps::try_from(someip_props.element().clone()).is_ok());

        assert_eq!(signal.transformation_isignal_props().count(), 2);
        let mut props_iter = signal.transformation_isignal_props();
        assert_eq!(props_iter.next().unwrap().element(), e2e_props.element());
        assert_eq!(props_iter.next().unwrap().element(), someip_props.element());
    }
}
