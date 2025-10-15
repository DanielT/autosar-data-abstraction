use autosar_data::{AutosarModel, Element, ElementName};

use crate::{
    AbstractionElement, AutosarAbstractionError, ByteOrder, IdentifiableAbstractionElement, System, SystemCategory,
    abstraction_element,
    communication::{
        DataTransformationSet, RequestResponseDelay, SomeipSdClientEventGroupTimingConfig,
        SomeipSdClientServiceInstanceConfig, SomeipSdServerEventGroupTimingConfig, SomeipSdServerServiceInstanceConfig,
        SystemSignal, SystemSignalGroup,
    },
    datatype::{
        ApplicationArrayDataType, ApplicationArraySize, ApplicationDataType, ApplicationPrimitiveCategory,
        ApplicationPrimitiveDataType, ApplicationRecordDataType, BaseTypeEncoding, CompuMethod, CompuMethodContent,
        ConstantSpecification, DataConstr, DataTypeMappingSet, ImplementationDataType, ImplementationDataTypeSettings,
        SwBaseType, Unit, ValueSpecification,
    },
    ecu_configuration::{
        EcucDefinitionCollection, EcucDestinationUriDefSet, EcucModuleConfigurationValues, EcucModuleDef,
        EcucValueCollection,
    },
    software_component::{
        ApplicationSwComponentType, ClientServerInterface, ComplexDeviceDriverSwComponentType,
        CompositionSwComponentType, EcuAbstractionSwComponentType, ModeDeclarationGroup, ModeDeclarationGroupCategory,
        ModeSwitchInterface, NvDataInterface, ParameterInterface, SenderReceiverInterface,
        SensorActuatorSwComponentType, ServiceSwComponentType, TriggerInterface,
    },
};

/// An `ArPackage` is an Autosar package, which can contain other packages or elements
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArPackage(Element);
abstraction_element!(ArPackage, ArPackage);
impl IdentifiableAbstractionElement for ArPackage {}

impl ArPackage {
    /// Get or create an autosar package for the given path
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/pkg1")?;
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::InvalidPath`] The value in `package_path` was not an Autosar path
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model
    pub(crate) fn get_or_create(model: &AutosarModel, package_path: &str) -> Result<Self, AutosarAbstractionError> {
        if let Some(pkg_elem) = model.get_element_by_path(package_path) {
            pkg_elem.try_into()
        } else {
            let mut parts_iter = package_path.split('/');
            if !parts_iter.next().unwrap_or("-").is_empty() {
                return Err(AutosarAbstractionError::InvalidPath(package_path.to_string()));
            }
            let mut pkg_elem = model.root_element();
            for part in parts_iter {
                pkg_elem = pkg_elem
                    .get_or_create_sub_element(ElementName::ArPackages)?
                    .get_or_create_named_sub_element(ElementName::ArPackage, part)?;
            }
            pkg_elem.try_into()
        }
    }

    /// create a new `ApplicationArrayDataType` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::{*, datatype::*};
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let element_type = package.create_application_primitive_data_type("ElementType", ApplicationPrimitiveCategory::Value, None, None, None)?;
    /// let data_type = package.create_application_array_data_type("ArrayDataType", &element_type, ApplicationArraySize::Fixed(4))?;
    /// assert!(model.get_element_by_path("/some/package/ArrayDataType").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the APPLICATION-ARRAY-DATA-TYPE element
    pub fn create_application_array_data_type<T: Into<ApplicationDataType> + AbstractionElement>(
        &self,
        name: &str,
        element_type: &T,
        size: ApplicationArraySize,
    ) -> Result<ApplicationArrayDataType, AutosarAbstractionError> {
        ApplicationArrayDataType::new(name, self, element_type, size)
    }

    /// create a new `ApplicationPrimitiveDataType` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::{*, datatype::*};
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let data_type = package.create_application_primitive_data_type("ApplicationPrimitiveDataType", ApplicationPrimitiveCategory::Value, None, None, None)?;
    /// assert!(model.get_element_by_path("/some/package/ApplicationPrimitiveDataType").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the APPLICATION-PRIMITIVE-DATA-TYPE element
    pub fn create_application_primitive_data_type(
        &self,
        name: &str,
        category: ApplicationPrimitiveCategory,
        compu_method: Option<&CompuMethod>,
        unit: Option<&Unit>,
        data_constraint: Option<&DataConstr>,
    ) -> Result<ApplicationPrimitiveDataType, AutosarAbstractionError> {
        ApplicationPrimitiveDataType::new(name, self, category, compu_method, unit, data_constraint)
    }

    /// create a new `ApplicationRecordDataType` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::{*, datatype::*};
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let data_type = package.create_application_record_data_type("ApplicationRecordDataType")?;
    /// assert!(model.get_element_by_path("/some/package/ApplicationRecordDataType").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the APPLICATION-RECORD-DATA-TYPE element
    pub fn create_application_record_data_type(
        &self,
        name: &str,
    ) -> Result<ApplicationRecordDataType, AutosarAbstractionError> {
        ApplicationRecordDataType::new(name, self)
    }

    /// create a new `ApplicationSwComponentType` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let component = package.create_application_sw_component_type("MyComponent")?;
    /// assert!(model.get_element_by_path("/some/package/MyComponent").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the APPLICATION-SW-COMPONENT-TYPE element
    pub fn create_application_sw_component_type(
        &self,
        name: &str,
    ) -> Result<ApplicationSwComponentType, AutosarAbstractionError> {
        ApplicationSwComponentType::new(name, self)
    }

    /// create a new `ClientServerInterface` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let interface = package.create_client_server_interface("ClientServerInterface")?;
    /// assert!(model.get_element_by_path("/some/package/ClientServerInterface").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the CLIENT-SERVER-INTERFACE element
    pub fn create_client_server_interface(&self, name: &str) -> Result<ClientServerInterface, AutosarAbstractionError> {
        ClientServerInterface::new(name, self)
    }

    /// create a new `ComplexDeviceDriverSwComponentType` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let component = package.create_complex_device_driver_sw_component_type("ComplexDeviceDriverSwComponentType")?;
    /// assert!(model.get_element_by_path("/some/package/ComplexDeviceDriverSwComponentType").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the COMPLEX-DEVICE-DRIVER-SW-COMPONENT-TYPE element
    pub fn create_complex_device_driver_sw_component_type(
        &self,
        name: &str,
    ) -> Result<ComplexDeviceDriverSwComponentType, AutosarAbstractionError> {
        ComplexDeviceDriverSwComponentType::new(name, self)
    }

    /// create a new `CompositionSwComponentType` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let component = package.create_composition_sw_component_type("CompositionSwComponentType")?;
    /// assert!(model.get_element_by_path("/some/package/CompositionSwComponentType").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the COMPOSITION-SW-COMPONENT-TYPE element
    pub fn create_composition_sw_component_type(
        &self,
        name: &str,
    ) -> Result<CompositionSwComponentType, AutosarAbstractionError> {
        CompositionSwComponentType::new(name, self)
    }

    /// create a new `CompuMethod` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::{*, datatype::*};
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let compu_content = CompuMethodContent::Linear(CompuMethodLinearContent {
    ///    direction: CompuScaleDirection::IntToPhys,
    ///    offset: 0.0,
    ///    factor: 1.0,
    ///    divisor: 1.0,
    ///    lower_limit: None,
    ///    upper_limit: None,
    /// });
    /// let compu_method = package.create_compu_method("CompuMethod", compu_content)?;
    /// assert!(model.get_element_by_path("/some/package/CompuMethod").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the COMPU-METHOD element
    pub fn create_compu_method(
        &self,
        name: &str,
        content: CompuMethodContent,
    ) -> Result<CompuMethod, AutosarAbstractionError> {
        CompuMethod::new(name, self, content)
    }

    /// create a new `ConstantSpecification` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::{*, datatype::*};
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let value = NumericalValueSpecification {
    ///    label: None,
    ///    value: 42.0,
    /// };
    /// let compu_method = package.create_constant_specification("CompuMethod", value)?;
    /// assert!(model.get_element_by_path("/some/package/CompuMethod").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the CONSTANT-SPECIFICATION element
    pub fn create_constant_specification<T: Into<ValueSpecification>>(
        &self,
        name: &str,
        value: T,
    ) -> Result<ConstantSpecification, AutosarAbstractionError> {
        ConstantSpecification::new(name, self, value.into())
    }

    /// create a new `DataConstr` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::{*, datatype::*};
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let data_constr = package.create_data_constr("DataConstr")?;
    /// assert!(model.get_element_by_path("/some/package/DataConstr").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the DATA-CONSTR element
    pub fn create_data_constr(&self, name: &str) -> Result<DataConstr, AutosarAbstractionError> {
        DataConstr::new(name, self)
    }

    /// create a new `DataTransformationSet` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let transformation_set = package.create_data_transformation_set("DataTransformationSet")?;
    /// assert!(model.get_element_by_path("/some/package/DataTransformationSet").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the DATA-TRANSFORMATION-SET element
    pub fn create_data_transformation_set(&self, name: &str) -> Result<DataTransformationSet, AutosarAbstractionError> {
        DataTransformationSet::new(name, self)
    }

    /// create a new `DataTypeMappingSet` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let mapping_set = package.create_data_type_mapping_set("DataTypeMappingSet")?;
    /// assert!(model.get_element_by_path("/some/package/DataTypeMappingSet").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the DATA-TYPE-MAPPING-SET element
    pub fn create_data_type_mapping_set(&self, name: &str) -> Result<DataTypeMappingSet, AutosarAbstractionError> {
        DataTypeMappingSet::new(name, self)
    }

    /// create a new `EcuAbstractionSwComponentType` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let component = package.create_ecu_abstraction_sw_component_type("EcuAbstractionSwComponentType")?;
    /// assert!(model.get_element_by_path("/some/package/EcuAbstractionSwComponentType").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the ECU-ABSTRACTION-SW-COMPONENT-TYPE element
    pub fn create_ecu_abstraction_sw_component_type(
        &self,
        name: &str,
    ) -> Result<EcuAbstractionSwComponentType, AutosarAbstractionError> {
        EcuAbstractionSwComponentType::new(name, self)
    }

    /// create a new `EcucDefinitionCollection` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
    /// let package = model.get_or_create_package("/pkg")?;
    /// let definition_collection = package.create_ecuc_definition_collection("DefinitionCollection")?;
    /// assert!(model.get_element_by_path("/pkg/DefinitionCollection").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model
    pub fn create_ecuc_definition_collection(
        &self,
        name: &str,
    ) -> Result<EcucDefinitionCollection, AutosarAbstractionError> {
        EcucDefinitionCollection::new(name, self)
    }

    /// create a new `EcucDestinationUriDefSet` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
    /// let package = model.get_or_create_package("/pkg")?;
    /// let uri_def_set = package.create_ecuc_destination_uri_def_set("DestinationUriDefSet")?;
    /// assert!(model.get_element_by_path("/pkg/DestinationUriDefSet").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model
    pub fn create_ecuc_destination_uri_def_set(
        &self,
        name: &str,
    ) -> Result<EcucDestinationUriDefSet, AutosarAbstractionError> {
        EcucDestinationUriDefSet::new(name, self)
    }

    /// create a new `EcucModuleConfigurationValues` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
    /// let package = model.get_or_create_package("/pkg")?;
    /// # let module_definition = package.create_ecuc_module_def("ModuleDef")?;
    /// let module_config = package.create_ecuc_module_configuration_values("ModuleConfig", &module_definition)?;
    /// assert!(model.get_element_by_path("/pkg/ModuleConfig").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model
    pub fn create_ecuc_module_configuration_values(
        &self,
        name: &str,
        definition: &EcucModuleDef,
    ) -> Result<EcucModuleConfigurationValues, AutosarAbstractionError> {
        EcucModuleConfigurationValues::new(name, self, definition)
    }

    /// create a new `EcucModuleDef` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
    /// let package = model.get_or_create_package("/pkg")?;
    /// let bsw_module = package.create_ecuc_module_def("BswModule")?;
    /// assert!(model.get_element_by_path("/pkg/BswModule").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model
    pub fn create_ecuc_module_def(&self, name: &str) -> Result<EcucModuleDef, AutosarAbstractionError> {
        EcucModuleDef::new(name, self)
    }

    /// create a new `EcucValueCollection` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
    /// let package = model.get_or_create_package("/pkg")?;
    /// let value_collection = package.create_ecuc_value_collection("ValueCollection")?;
    /// assert!(model.get_element_by_path("/pkg/ValueCollection").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model
    pub fn create_ecuc_value_collection(&self, name: &str) -> Result<EcucValueCollection, AutosarAbstractionError> {
        EcucValueCollection::new(name, self)
    }

    /// create a new `ImplementationDataType` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::{*, datatype::*};
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let sw_base_type = package.create_sw_base_type("uint8", 8, BaseTypeEncoding::None, None, None, None)?;
    /// let settings = ImplementationDataTypeSettings::Value {
    ///     name: "ImplementationDataType_Value".to_string(),
    ///     base_type: sw_base_type,
    ///     compu_method: None,
    ///     data_constraint: None,
    /// };
    /// let data_type = package.create_implementation_data_type(&settings)?;
    /// assert!(model.get_element_by_path("/some/package/ImplementationDataType_Value").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the IMPLEMENTATION-DATA-TYPE element
    pub fn create_implementation_data_type(
        &self,
        settings: &ImplementationDataTypeSettings,
    ) -> Result<ImplementationDataType, AutosarAbstractionError> {
        ImplementationDataType::new(self, settings)
    }

    /// create a new `ModeDeclarationGroup` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let mode_declaration_group = package.create_mode_declaration_group("ModeDeclarationGroup", None)?;
    /// assert!(model.get_element_by_path("/some/package/ModeDeclarationGroup").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the MODE-DECLARATION-GROUP element
    pub fn create_mode_declaration_group(
        &self,
        name: &str,
        category: Option<ModeDeclarationGroupCategory>,
    ) -> Result<ModeDeclarationGroup, AutosarAbstractionError> {
        ModeDeclarationGroup::new(name, self, category)
    }

    /// create a new `ModeSwitchInterface` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let interface = package.create_mode_switch_interface("ModeSwitchInterface")?;
    /// assert!(model.get_element_by_path("/some/package/ModeSwitchInterface").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the MODE-SWITCH-INTERFACE element
    pub fn create_mode_switch_interface(&self, name: &str) -> Result<ModeSwitchInterface, AutosarAbstractionError> {
        ModeSwitchInterface::new(name, self)
    }

    /// create a new `NvDataInterface` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let interface = package.create_nv_data_interface("NvDataInterface")?;
    /// assert!(model.get_element_by_path("/some/package/NvDataInterface").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the NV-DATA-INTERFACE element
    pub fn create_nv_data_interface(&self, name: &str) -> Result<NvDataInterface, AutosarAbstractionError> {
        NvDataInterface::new(name, self)
    }

    /// create a new `ParameterInterface` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let interface = package.create_parameter_interface("ParameterInterface")?;
    /// assert!(model.get_element_by_path("/some/package/ParameterInterface").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the PARAMETER-INTERFACE element
    pub fn create_parameter_interface(&self, name: &str) -> Result<ParameterInterface, AutosarAbstractionError> {
        ParameterInterface::new(name, self)
    }

    /// create a new `SenderReceiverInterface` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let interface = package.create_sender_receiver_interface("SenderReceiverInterface")?;
    /// assert!(model.get_element_by_path("/some/package/SenderReceiverInterface").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the SENDER-RECEIVER-INTERFACE element
    pub fn create_sender_receiver_interface(
        &self,
        name: &str,
    ) -> Result<SenderReceiverInterface, AutosarAbstractionError> {
        SenderReceiverInterface::new(name, self)
    }

    /// create a new `SensorActuatorSwComponentType` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::{*, communication::*};
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let component = package.create_sensor_actuator_sw_component_type("SensorActuatorSwComponentType")?;
    /// assert!(model.get_element_by_path("/some/package/SensorActuatorSwComponentType").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the SENSOR-ACTUATOR-SW-COMPONENT-TYPE element
    pub fn create_sensor_actuator_sw_component_type(
        &self,
        name: &str,
    ) -> Result<SensorActuatorSwComponentType, AutosarAbstractionError> {
        SensorActuatorSwComponentType::new(name, self)
    }

    /// create a new `ServiceSwComponentType` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let component = package.create_service_sw_component_type("ServiceSwComponentType")?;
    /// assert!(model.get_element_by_path("/some/package/ServiceSwComponentType").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the SERVICE-SW-COMPONENT-TYPE element
    pub fn create_service_sw_component_type(
        &self,
        name: &str,
    ) -> Result<ServiceSwComponentType, AutosarAbstractionError> {
        ServiceSwComponentType::new(name, self)
    }

    /// create a new `SomeipSdClientEventGroupTimingConfig` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::{*, communication::*};
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let client_config = package.create_someip_sd_client_event_group_timing_config("SomeipSdClientEventGroupTimingConfig", 10)?;
    /// assert!(model.get_element_by_path("/some/package/SomeipSdClientEventGroupTimingConfig").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the SOMEIP-SD-CLIENT-EVENT-GROUP-TIMING-CONFIG element
    pub fn create_someip_sd_client_event_group_timing_config(
        &self,
        name: &str,
        time_to_live: u32,
    ) -> Result<SomeipSdClientEventGroupTimingConfig, AutosarAbstractionError> {
        SomeipSdClientEventGroupTimingConfig::new(name, self, time_to_live)
    }

    /// create a new `SomeipSdClientServiceInstanceConfig` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let service_instance = package.create_someip_sd_client_service_instance_config("SomeipSdClientServiceInstanceConfig")?;
    /// assert!(model.get_element_by_path("/some/package/SomeipSdClientServiceInstanceConfig").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the SOMEIP-SD-CLIENT-SERVICE-INSTANCE-CONFIG element
    pub fn create_someip_sd_client_service_instance_config(
        &self,
        name: &str,
    ) -> Result<SomeipSdClientServiceInstanceConfig, AutosarAbstractionError> {
        SomeipSdClientServiceInstanceConfig::new(name, self)
    }

    /// create a new `SomeipSdServerEventGroupTimingConfig` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::{*, communication::*};
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let request_response_delay = RequestResponseDelay {
    ///     min_value: 0.1,
    ///     max_value: 0.2,
    /// };
    /// let timing_config = package.create_someip_sd_server_event_group_timing_config("SomeipSdServerEventGroupTimingConfig", &request_response_delay)?;
    /// assert!(model.get_element_by_path("/some/package/SomeipSdServerEventGroupTimingConfig").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the SOMEIP-SD-SERVER-EVENT-GROUP-TIMING-CONFIG element
    pub fn create_someip_sd_server_event_group_timing_config(
        &self,
        name: &str,
        request_response_delay: &RequestResponseDelay,
    ) -> Result<SomeipSdServerEventGroupTimingConfig, AutosarAbstractionError> {
        SomeipSdServerEventGroupTimingConfig::new(name, self, request_response_delay)
    }

    /// create a new `SomeipSdServerServiceInstanceConfig` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let service_instance = package.create_someip_sd_server_service_instance_config("SomeipSdServerServiceInstanceConfig", 10)?;
    /// assert!(model.get_element_by_path("/some/package/SomeipSdServerServiceInstanceConfig").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the SOMEIP-SD-SERVER-SERVICE-INSTANCE-CONFIG element
    pub fn create_someip_sd_server_service_instance_config(
        &self,
        name: &str,
        ttl: u32,
    ) -> Result<SomeipSdServerServiceInstanceConfig, AutosarAbstractionError> {
        SomeipSdServerServiceInstanceConfig::new(name, self, ttl)
    }

    /// create a new `SwBaseType` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::{*, datatype::*};
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let base_type = package.create_sw_base_type("MyBaseType", 8, BaseTypeEncoding::None, None, None, None)?;
    /// assert!(model.get_element_by_path("/some/package/MyBaseType").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the SW-BASE-TYPE element
    pub fn create_sw_base_type(
        &self,
        name: &str,
        bit_length: u32,
        base_type_encoding: BaseTypeEncoding,
        byte_order: Option<ByteOrder>,
        mem_alignment: Option<u32>,
        native_declaration: Option<&str>,
    ) -> Result<SwBaseType, AutosarAbstractionError> {
        SwBaseType::new(
            name,
            self,
            bit_length,
            base_type_encoding,
            byte_order,
            mem_alignment,
            native_declaration,
        )
    }

    /// create a new System in the package
    ///
    /// Note that an Autosar model should ony contain one SYSTEM. This is not checked here.
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// assert!(model.get_element_by_path("/some/package/System").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the SYSTEM element
    pub fn create_system(&self, name: &str, category: SystemCategory) -> Result<System, AutosarAbstractionError> {
        System::new(name, self, category)
    }

    /// create a new `SystemSignal` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let signal = package.create_system_signal("MySignal")?;
    /// assert!(model.get_element_by_path("/some/package/MySignal").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the SYSTEM-SIGNAL element
    pub fn create_system_signal(&self, name: &str) -> Result<SystemSignal, AutosarAbstractionError> {
        SystemSignal::new(name, self)
    }

    /// create a new `SystemSignalGroup` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let signal_group = package.create_system_signal_group("MySignalGroup")?;
    /// assert!(model.get_element_by_path("/some/package/MySignalGroup").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the SYSTEM-SIGNAL-GROUP element
    pub fn create_system_signal_group(&self, name: &str) -> Result<SystemSignalGroup, AutosarAbstractionError> {
        SystemSignalGroup::new(name, self)
    }

    /// create a new `TriggerInterface` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let interface = package.create_trigger_interface("TriggerInterface")?;
    /// assert!(model.get_element_by_path("/some/package/TriggerInterface").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the TRIGGER-INTERFACE element
    pub fn create_trigger_interface(&self, name: &str) -> Result<TriggerInterface, AutosarAbstractionError> {
        TriggerInterface::new(name, self)
    }

    /// create a new `Unit` in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let unit = package.create_unit("Unit", Some("UnitDisplayName"))?;
    /// assert!(model.get_element_by_path("/some/package/Unit").is_some());
    /// # Ok(())}
    /// ```
    ///
    /// # Errors
    ///
    /// - [`AutosarAbstractionError::ModelError`] An error occurred in the Autosar model while trying to create the UNIT element
    pub fn create_unit(&self, name: &str, display_name: Option<&str>) -> Result<Unit, AutosarAbstractionError> {
        Unit::new(name, self, display_name)
    }

    /// iterate over all elements in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// for element in package.elements() {
    ///    println!("{:?}", element);
    /// }
    /// # Ok(())}
    /// ```
    pub fn elements(&self) -> impl Iterator<Item = Element> + Send + use<> {
        self.0
            .get_sub_element(ElementName::Elements)
            .into_iter()
            .flat_map(|element| element.sub_elements())
    }

    /// create a new `ArPackage` as a sub-package of the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// let sub_package = package.create_sub_package("SubPackage")?;
    /// assert!(model.get_element_by_path("/some/package/SubPackage").is_some());
    /// # Ok(())}
    /// ```
    pub fn create_sub_package(&self, name: &str) -> Result<ArPackage, AutosarAbstractionError> {
        let sub_package_elem = self
            .0
            .get_or_create_sub_element(ElementName::ArPackages)
            .and_then(|elem| elem.create_named_sub_element(ElementName::ArPackage, name))?;
        Ok(Self(sub_package_elem))
    }

    /// iterate over all sub-packages in the package
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// # let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// let package = model.get_or_create_package("/some/package")?;
    /// for sub_package in package.sub_packages() {
    ///   println!("{:?}", sub_package);
    /// }
    /// # Ok(())}
    /// ```
    pub fn sub_packages(&self) -> impl Iterator<Item = ArPackage> + Send + use<> {
        self.0
            .get_sub_element(ElementName::ArPackages)
            .into_iter()
            .flat_map(|element| element.sub_elements())
            .filter_map(|element| ArPackage::try_from(element).ok())
    }
}

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::{AutosarModelAbstraction, datatype::*};
    use crate::{System, SystemCategory};
    use autosar_data::AutosarVersion;

    #[test]
    fn package() {
        let model = AutosarModel::new();
        // can't do anything in an incomplete model: it has no files
        let result = ArPackage::get_or_create(&model, "/bad");
        assert!(result.is_err());

        model.create_file("filename", AutosarVersion::Autosar_00048).unwrap();

        // create a new package
        let result = ArPackage::get_or_create(&model, "/pkg1");
        assert!(result.is_ok());
        let package = result.unwrap();
        assert_eq!(package.name().unwrap(), "pkg1");
        // get the existing package
        let result = ArPackage::get_or_create(&model, "/pkg1");
        assert!(result.is_ok());
        // create multiple levels
        let result = ArPackage::get_or_create(&model, "/level1/level2/level3");
        assert!(result.is_ok());
        let package = result.unwrap();
        assert_eq!(package.name().unwrap(), "level3");

        // can't create a package due to an element name conflict
        let pkg = ArPackage::get_or_create(&model, "/test").unwrap();
        System::new("system", &pkg, SystemCategory::EcuExtract).unwrap();
        let result = ArPackage::get_or_create(&model, "/test/system");
        assert!(result.is_err());
        let result = ArPackage::get_or_create(&model, "/test/system/sub");
        assert!(result.is_err());

        // invalid path: does not start with '/'
        let result = ArPackage::get_or_create(&model, "hello world");
        assert!(result.is_err());

        // conversions
        let element: Element = pkg.into();
        let result = ArPackage::try_from(element);
        assert!(result.is_ok());
        let result = ArPackage::try_from(model.root_element());
        assert!(result.is_err());
    }

    #[test]
    fn create_package_items() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let package = model.get_or_create_package("/some/package").unwrap();

        // create a new application primitive data type
        let primitive_data_type = package
            .create_application_primitive_data_type(
                "ApplicationPrimitiveDataType",
                ApplicationPrimitiveCategory::Value,
                None,
                None,
                None,
            )
            .unwrap();
        assert_eq!(primitive_data_type.name().unwrap(), "ApplicationPrimitiveDataType");

        // create a new application array data type
        let array_data_type = package
            .create_application_array_data_type(
                "ApplicationArrayDataType",
                &primitive_data_type,
                ApplicationArraySize::Fixed(4),
            )
            .unwrap();
        assert_eq!(array_data_type.name().unwrap(), "ApplicationArrayDataType");

        // create a new application record data type
        let data_type = package
            .create_application_record_data_type("ApplicationRecordDataType")
            .unwrap();
        assert_eq!(data_type.name().unwrap(), "ApplicationRecordDataType");

        // create a new application sw component type
        let component = package.create_application_sw_component_type("MyComponent").unwrap();
        assert_eq!(component.name().unwrap(), "MyComponent");

        // create a new client server interface
        let interface = package.create_client_server_interface("ClientServerInterface").unwrap();
        assert_eq!(interface.name().unwrap(), "ClientServerInterface");

        // create a new complex device driver sw component type
        let component = package
            .create_complex_device_driver_sw_component_type("ComplexDeviceDriverSwComponentType")
            .unwrap();
        assert_eq!(component.name().unwrap(), "ComplexDeviceDriverSwComponentType");

        // create a new composition sw component type
        let component = package
            .create_composition_sw_component_type("CompositionSwComponentType")
            .unwrap();
        assert_eq!(component.name().unwrap(), "CompositionSwComponentType");

        // create a new compu method
        let compu_content = CompuMethodContent::Linear(CompuMethodLinearContent {
            direction: CompuScaleDirection::IntToPhys,
            offset: 0.0,
            factor: 1.0,
            divisor: 1.0,
            lower_limit: None,
            upper_limit: None,
        });
        let compu_method = package.create_compu_method("CompuMethod", compu_content).unwrap();
        assert_eq!(compu_method.name().unwrap(), "CompuMethod");

        // create a new data constr
        let data_constr = package.create_data_constr("DataConstr").unwrap();
        assert_eq!(data_constr.name().unwrap(), "DataConstr");

        // create a new data transformation set
        let transformation_set = package.create_data_transformation_set("DataTransformationSet").unwrap();
        assert_eq!(transformation_set.name().unwrap(), "DataTransformationSet");

        // create a new data type mapping set
        let mapping_set = package.create_data_type_mapping_set("DataTypeMappingSet").unwrap();
        assert_eq!(mapping_set.name().unwrap(), "DataTypeMappingSet");

        // create a new ecu abstraction sw component type
        let component = package
            .create_ecu_abstraction_sw_component_type("EcuAbstractionSwComponentType")
            .unwrap();
        assert_eq!(component.name().unwrap(), "EcuAbstractionSwComponentType");

        // create a new ecuc definition collection
        let ecuc_definition_collection = package
            .create_ecuc_definition_collection("EcucDefinitionCollection")
            .unwrap();
        assert_eq!(ecuc_definition_collection.name().unwrap(), "EcucDefinitionCollection");

        // create a new ecuc value collection
        let ecuc_value_collection = package.create_ecuc_value_collection("EcucValueCollection").unwrap();
        assert_eq!(ecuc_value_collection.name().unwrap(), "EcucValueCollection");

        // create a new ecuc destination uri def set
        let uri_def_set = package
            .create_ecuc_destination_uri_def_set("EcucDestinationUriDefSet")
            .unwrap();
        assert_eq!(uri_def_set.name().unwrap(), "EcucDestinationUriDefSet");

        // create a new ecuc module def
        let ecuc_module_def = package.create_ecuc_module_def("EcucModuleDef").unwrap();
        assert_eq!(ecuc_module_def.name().unwrap(), "EcucModuleDef");

        // create a new ecuc module configuration values
        let ecuc_module_configuration_values = package
            .create_ecuc_module_configuration_values("EcucModuleConfigurationValues", &ecuc_module_def)
            .unwrap();
        assert_eq!(
            ecuc_module_configuration_values.name().unwrap(),
            "EcucModuleConfigurationValues"
        );

        // create a new implementation data type
        let sw_base_type = package
            .create_sw_base_type("uint8", 8, BaseTypeEncoding::None, None, None, None)
            .unwrap();
        let settings = ImplementationDataTypeSettings::Value {
            name: "ImplementationDataType_Value".to_string(),
            base_type: sw_base_type,
            compu_method: None,
            data_constraint: None,
        };
        let data_type = package.create_implementation_data_type(&settings).unwrap();
        assert_eq!(data_type.name().unwrap(), "ImplementationDataType_Value");

        // create a new mode switch interface
        let interface = package.create_mode_switch_interface("ModeSwitchInterface").unwrap();
        assert_eq!(interface.name().unwrap(), "ModeSwitchInterface");

        // create a new nv data interface
        let interface = package.create_nv_data_interface("NvDataInterface").unwrap();
        assert_eq!(interface.name().unwrap(), "NvDataInterface");

        // create a new parameter interface
        let interface = package.create_parameter_interface("ParameterInterface").unwrap();
        assert_eq!(interface.name().unwrap(), "ParameterInterface");

        // create a new sender receiver interface
        let interface = package
            .create_sender_receiver_interface("SenderReceiverInterface")
            .unwrap();
        assert_eq!(interface.name().unwrap(), "SenderReceiverInterface");

        // create a new sensor actuator sw component type
        let component = package
            .create_sensor_actuator_sw_component_type("SensorActuatorSwComponentType")
            .unwrap();
        assert_eq!(component.name().unwrap(), "SensorActuatorSwComponentType");

        // create a new service sw component type
        let component = package
            .create_service_sw_component_type("ServiceSwComponentType")
            .unwrap();
        assert_eq!(component.name().unwrap(), "ServiceSwComponentType");

        // create a new someip sd client event group timing config
        let client_config = package
            .create_someip_sd_client_event_group_timing_config("SomeipSdClientEventGroupTimingConfig", 10)
            .unwrap();
        assert_eq!(client_config.name().unwrap(), "SomeipSdClientEventGroupTimingConfig");

        // create a new someip sd client service instance config
        let service_instance = package
            .create_someip_sd_client_service_instance_config("SomeipSdClientServiceInstanceConfig")
            .unwrap();
        assert_eq!(service_instance.name().unwrap(), "SomeipSdClientServiceInstanceConfig");

        // create a new someip sd server event group timing config
        let request_response_delay = RequestResponseDelay {
            min_value: 0.1,
            max_value: 0.2,
        };
        let timing_config = package
            .create_someip_sd_server_event_group_timing_config(
                "SomeipSdServerEventGroupTimingConfig",
                &request_response_delay,
            )
            .unwrap();
        assert_eq!(timing_config.name().unwrap(), "SomeipSdServerEventGroupTimingConfig");

        // create a new someip sd server service instance config
        let service_instance = package
            .create_someip_sd_server_service_instance_config("SomeipSdServerServiceInstanceConfig", 10)
            .unwrap();
        assert_eq!(service_instance.name().unwrap(), "SomeipSdServerServiceInstanceConfig");

        // create a new sw base type
        let sw_base_type = package
            .create_sw_base_type("SwBaseType", 8, BaseTypeEncoding::None, None, None, None)
            .unwrap();
        assert_eq!(sw_base_type.name().unwrap(), "SwBaseType");

        // create a new system
        let system = package.create_system("System", SystemCategory::SystemExtract).unwrap();
        assert_eq!(system.name().unwrap(), "System");

        // create a new system signal
        let signal = package.create_system_signal("SystemSignal").unwrap();
        assert_eq!(signal.name().unwrap(), "SystemSignal");

        // create a new system signal group
        let signal_group = package.create_system_signal_group("SystemSignalGroup").unwrap();
        assert_eq!(signal_group.name().unwrap(), "SystemSignalGroup");

        // create a new trigger interface
        let interface = package.create_trigger_interface("TriggerInterface").unwrap();
        assert_eq!(interface.name().unwrap(), "TriggerInterface");

        // create a new unit
        let unit = package.create_unit("Unit", Some("UnitDisplayName")).unwrap();
        assert_eq!(unit.name().unwrap(), "Unit");
    }

    #[test]
    fn test_elements_iter() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let package = model.get_or_create_package("/some/package").unwrap();

        package
            .create_application_record_data_type("ApplicationRecordDataType")
            .unwrap();
        package
            .create_application_sw_component_type("ApplicationSwComponentType")
            .unwrap();
        package.create_client_server_interface("ClientServerInterface").unwrap();
        package
            .create_complex_device_driver_sw_component_type("ComplexDeviceDriverSwComponentType")
            .unwrap();
        package
            .create_composition_sw_component_type("CompositionSwComponentType")
            .unwrap();
        package
            .create_compu_method(
                "CompuMethod",
                CompuMethodContent::Linear(CompuMethodLinearContent {
                    direction: CompuScaleDirection::IntToPhys,
                    offset: 0.0,
                    factor: 1.0,
                    divisor: 1.0,
                    lower_limit: None,
                    upper_limit: None,
                }),
            )
            .unwrap();
        package.create_data_constr("DataConstr").unwrap();
        package.create_data_transformation_set("DataTransformationSet").unwrap();
        package.create_data_type_mapping_set("DataTypeMappingSet").unwrap();
        package
            .create_ecu_abstraction_sw_component_type("EcuAbstractionSwComponentType")
            .unwrap();
        let sw_base_type = package
            .create_sw_base_type("uint8", 8, BaseTypeEncoding::None, None, None, None)
            .unwrap();
        let settings = ImplementationDataTypeSettings::Value {
            name: "ImplementationDataType_Value".to_string(),
            base_type: sw_base_type,
            compu_method: None,
            data_constraint: None,
        };
        package.create_implementation_data_type(&settings).unwrap();
        package.create_mode_switch_interface("ModeSwitchInterface").unwrap();
        package.create_nv_data_interface("NvDataInterface").unwrap();
        package.create_parameter_interface("ParameterInterface").unwrap();
        package
            .create_sender_receiver_interface("SenderReceiverInterface")
            .unwrap();
        package
            .create_sensor_actuator_sw_component_type("SensorActuatorSwComponentType")
            .unwrap();
        package
            .create_service_sw_component_type("ServiceSwComponentType")
            .unwrap();
        package
            .create_someip_sd_client_event_group_timing_config("SomeipSdClientEventGroupTimingConfig", 10)
            .unwrap();
        package
            .create_someip_sd_client_service_instance_config("SomeipSdClientServiceInstanceConfig")
            .unwrap();
        let request_response_delay = RequestResponseDelay {
            min_value: 0.1,
            max_value: 0.2,
        };
        package
            .create_someip_sd_server_event_group_timing_config(
                "SomeipSdServerEventGroupTimingConfig",
                &request_response_delay,
            )
            .unwrap();
        package
            .create_someip_sd_server_service_instance_config("SomeipSdServerServiceInstanceConfig", 10)
            .unwrap();
        // package
        //     .create_sw_base_type("SwBaseType", 8, BaseTypeEncoding::None, None, None, None)
        //     .unwrap();
        package.create_system("System", SystemCategory::SystemExtract).unwrap();
        package.create_system_signal("SystemSignal").unwrap();
        package.create_system_signal_group("SystemSignalGroup").unwrap();
        package.create_trigger_interface("TriggerInterface").unwrap();
        package.create_unit("Unit", Some("UnitDisplayName")).unwrap();

        let mut elements = package.elements();
        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::ApplicationRecordDataType);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::ApplicationSwComponentType);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::ClientServerInterface);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::ComplexDeviceDriverSwComponentType);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::CompositionSwComponentType);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::CompuMethod);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::DataConstr);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::DataTransformationSet);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::DataTypeMappingSet);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::EcuAbstractionSwComponentType);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::SwBaseType);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::ImplementationDataType);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::ModeSwitchInterface);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::NvDataInterface);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::ParameterInterface);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::SenderReceiverInterface);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::SensorActuatorSwComponentType);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::ServiceSwComponentType);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::SomeipSdClientEventGroupTimingConfig);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::SomeipSdClientServiceInstanceConfig);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::SomeipSdServerEventGroupTimingConfig);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::SomeipSdServerServiceInstanceConfig);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::System);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::SystemSignal);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::SystemSignalGroup);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::TriggerInterface);

        let item = elements.next().unwrap();
        assert_eq!(item.element_name(), ElementName::Unit);
    }

    #[test]
    fn sub_packages() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
        let package = model.get_or_create_package("/package").unwrap();

        // create sub-packages
        package.create_sub_package("sub1").unwrap();
        package.create_sub_package("sub2").unwrap();

        // name conflict: can't create a sub-package with the same name
        let result = package.create_sub_package("sub2");
        assert!(result.is_err());

        // iterate over sub-packages
        assert_eq!(package.sub_packages().count(), 2);
    }
}
