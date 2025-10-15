use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, Element, EnumItem, IdentifiableAbstractionElement,
    abstraction_element,
    datatype::{self, AbstractAutosarDataType},
    software_component::AbstractPortInterface,
};
use autosar_data::ElementName;
use datatype::AutosarDataType;

//##################################################################

/// A `ClientServerInterface` defines a set of operations that can be implemented by a server and called by a client
///
/// Use [`ArPackage::create_client_server_interface`] to create a new client server interface
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientServerInterface(pub(crate) Element);
abstraction_element!(ClientServerInterface, ClientServerInterface);
impl IdentifiableAbstractionElement for ClientServerInterface {}
impl AbstractPortInterface for ClientServerInterface {}

impl ClientServerInterface {
    /// Create a new `ClientServerInterface`
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let client_server_interface = elements.create_named_sub_element(ElementName::ClientServerInterface, name)?;

        Ok(Self(client_server_interface))
    }

    /// Add a possible error to the client server interface
    pub fn create_possible_error(
        &self,
        name: &str,
        error_code: u64,
    ) -> Result<ApplicationError, AutosarAbstractionError> {
        let possible_errors = self.element().get_or_create_sub_element(ElementName::PossibleErrors)?;
        ApplicationError::new(name, error_code, &possible_errors)
    }

    /// add an operation to the client server interface
    pub fn create_operation(&self, name: &str) -> Result<ClientServerOperation, AutosarAbstractionError> {
        let operations = self.element().get_or_create_sub_element(ElementName::Operations)?;
        ClientServerOperation::new(name, &operations)
    }

    /// iterate over all operations
    pub fn operations(&self) -> impl Iterator<Item = ClientServerOperation> + Send + use<> {
        self.element()
            .get_sub_element(ElementName::Operations)
            .into_iter()
            .flat_map(|operations| operations.sub_elements())
            .filter_map(|elem| ClientServerOperation::try_from(elem).ok())
    }

    /// iterate over all application errors
    pub fn possible_errors(&self) -> impl Iterator<Item = ApplicationError> + Send + use<> {
        self.element()
            .get_sub_element(ElementName::PossibleErrors)
            .into_iter()
            .flat_map(|errors| errors.sub_elements())
            .filter_map(|elem| ApplicationError::try_from(elem).ok())
    }
}

//##################################################################

/// An `ApplicationError` represents an error that can be returned by a client server operation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApplicationError(Element);
abstraction_element!(ApplicationError, ApplicationError);
impl IdentifiableAbstractionElement for ApplicationError {}

impl ApplicationError {
    /// Create a new `ApplicationError`
    fn new(name: &str, error_code: u64, parent_element: &Element) -> Result<Self, AutosarAbstractionError> {
        let application_error = parent_element.create_named_sub_element(ElementName::ApplicationError, name)?;
        let application_error = Self(application_error);
        application_error.set_error_code(error_code)?;

        Ok(application_error)
    }

    /// Set the error code of the application error
    pub fn set_error_code(&self, error_code: u64) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::ErrorCode)?
            .set_character_data(error_code)?;
        Ok(())
    }

    /// Get the error code of the application error
    #[must_use]
    pub fn error_code(&self) -> Option<u64> {
        self.element()
            .get_sub_element(ElementName::ErrorCode)?
            .character_data()?
            .parse_integer()
    }
}

//##################################################################

/// A `ClientServerOperation` defines an operation in a `ClientServerInterface`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientServerOperation(Element);
abstraction_element!(ClientServerOperation, ClientServerOperation);
impl IdentifiableAbstractionElement for ClientServerOperation {}

impl ClientServerOperation {
    /// Create a new `ClientServerOperation`
    fn new(name: &str, parent_element: &Element) -> Result<Self, AutosarAbstractionError> {
        let operation = parent_element.create_named_sub_element(ElementName::ClientServerOperation, name)?;
        Ok(Self(operation))
    }

    /// Add an argument to the operation
    pub fn create_argument<T: AbstractAutosarDataType>(
        &self,
        name: &str,
        data_type: &T,
        direction: ArgumentDirection,
    ) -> Result<ArgumentDataPrototype, AutosarAbstractionError> {
        let arguments = self.element().get_or_create_sub_element(ElementName::Arguments)?;
        ArgumentDataPrototype::new(name, &arguments, data_type, direction)
    }

    /// iterate over all arguments
    pub fn arguments(&self) -> impl Iterator<Item = ArgumentDataPrototype> + Send + use<> {
        self.element()
            .get_sub_element(ElementName::Arguments)
            .into_iter()
            .flat_map(|arguments| arguments.sub_elements())
            .filter_map(|elem| ArgumentDataPrototype::try_from(elem).ok())
    }

    /// add a reference to possible error to the operation
    pub fn add_possible_error(&self, error: &ApplicationError) -> Result<(), AutosarAbstractionError> {
        if self.element().named_parent()? != error.element().named_parent()? {
            return Err(AutosarAbstractionError::InvalidParameter(
                "Error and operation must be in the same ClientServerInterface".to_string(),
            ));
        }

        let possible_errors = self
            .element()
            .get_or_create_sub_element(ElementName::PossibleErrorRefs)?;
        possible_errors
            .create_sub_element(ElementName::PossibleErrorRef)?
            .set_reference_target(error.element())?;
        Ok(())
    }

    /// Get the possible errors of the operation
    pub fn possible_errors(&self) -> impl Iterator<Item = ApplicationError> + Send + use<> {
        self.element()
            .get_sub_element(ElementName::PossibleErrorRefs)
            .into_iter()
            .flat_map(|errors| errors.sub_elements())
            .filter_map(|refelem| {
                refelem
                    .get_reference_target()
                    .ok()
                    .and_then(|elem| ApplicationError::try_from(elem).ok())
            })
    }
}

//##################################################################

/// The `ArgumentDirection` defines the direction of an argument in a `ClientServerOperation`
///
/// Input arguments are used to pass data from the client to the server and are usualy passed by value.
/// Output arguments are used to pass data from the server to the client and are usually passed by reference.
/// In/Out arguments are used to pass data in both directions and are usually passed by reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArgumentDirection {
    /// The argument is an input argument
    In,
    /// The argument is an output argument
    Out,
    /// The argument is an in/out argument
    InOut,
}

impl TryFrom<EnumItem> for ArgumentDirection {
    type Error = AutosarAbstractionError;

    fn try_from(item: EnumItem) -> Result<Self, Self::Error> {
        match item {
            EnumItem::In => Ok(ArgumentDirection::In),
            EnumItem::Out => Ok(ArgumentDirection::Out),
            EnumItem::Inout => Ok(ArgumentDirection::InOut),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: item.to_string(),
                dest: "ArgumentDirection".to_string(),
            }),
        }
    }
}

impl From<ArgumentDirection> for EnumItem {
    fn from(direction: ArgumentDirection) -> Self {
        match direction {
            ArgumentDirection::In => EnumItem::In,
            ArgumentDirection::Out => EnumItem::Out,
            ArgumentDirection::InOut => EnumItem::Inout,
        }
    }
}

//##################################################################

/// An `ArgumentDataPrototype` represents an argument in a `ClientServerOperation`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArgumentDataPrototype(Element);
abstraction_element!(ArgumentDataPrototype, ArgumentDataPrototype);
impl IdentifiableAbstractionElement for ArgumentDataPrototype {}

impl ArgumentDataPrototype {
    /// Create a new `ArgumentDataPrototype`
    fn new<T: AbstractAutosarDataType>(
        name: &str,
        parent_element: &Element,
        data_type: &T,
        direction: ArgumentDirection,
    ) -> Result<Self, AutosarAbstractionError> {
        let argument = parent_element.create_named_sub_element(ElementName::ArgumentDataPrototype, name)?;
        let argument = Self(argument);
        argument.set_data_type(data_type)?;
        argument.set_direction(direction)?;

        Ok(argument)
    }

    /// Set the data type of the argument
    pub fn set_data_type<T: AbstractAutosarDataType>(&self, data_type: &T) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::TypeTref)?
            .set_reference_target(data_type.element())?;
        Ok(())
    }

    /// Get the data type of the argument
    #[must_use]
    pub fn data_type(&self) -> Option<AutosarDataType> {
        let data_type_elem = self
            .element()
            .get_sub_element(ElementName::TypeTref)?
            .get_reference_target()
            .ok()?;
        AutosarDataType::try_from(data_type_elem).ok()
    }

    /// Set the direction of the argument
    pub fn set_direction(&self, direction: ArgumentDirection) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::Direction)?
            .set_character_data::<EnumItem>(direction.into())?;
        Ok(())
    }

    /// Get the direction of the argument
    #[must_use]
    pub fn direction(&self) -> Option<ArgumentDirection> {
        let value = self
            .element()
            .get_sub_element(ElementName::Direction)?
            .character_data()?
            .enum_value()?;

        ArgumentDirection::try_from(value).ok()
    }
}

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::AutosarModelAbstraction;
    use autosar_data::AutosarVersion;
    use datatype::{BaseTypeEncoding, ImplementationDataTypeSettings};

    #[test]
    fn test_client_server_interface() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();
        let client_server_interface = ClientServerInterface::new("TestInterface", &package).unwrap();

        assert_eq!(client_server_interface.name().unwrap(), "TestInterface");
        assert_eq!(client_server_interface.operations().count(), 0);
        assert_eq!(client_server_interface.possible_errors().count(), 0);

        let error = client_server_interface.create_possible_error("TestError", 42).unwrap();
        assert_eq!(client_server_interface.possible_errors().count(), 1);
        assert_eq!(error.name().unwrap(), "TestError");
        assert_eq!(error.error_code().unwrap(), 42);

        let operation = client_server_interface.create_operation("TestOperation").unwrap();
        assert_eq!(client_server_interface.operations().count(), 1);
        assert_eq!(operation.name().unwrap(), "TestOperation");
        assert_eq!(operation.arguments().count(), 0);

        operation.add_possible_error(&error).unwrap();
        assert_eq!(operation.possible_errors().count(), 1);

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
        let argument = operation
            .create_argument("TestArgument", &datatype, ArgumentDirection::In)
            .unwrap();
        assert_eq!(argument.name().unwrap(), "TestArgument");
        assert_eq!(argument.data_type().unwrap().name().unwrap(), "ImplementationValue");
        assert_eq!(argument.direction().unwrap(), ArgumentDirection::In);
        assert_eq!(operation.arguments().count(), 1);

        client_server_interface.set_is_service(Some(true)).unwrap();
        assert!(client_server_interface.is_service().unwrap());
        client_server_interface.set_is_service(Some(false)).unwrap();
        assert!(!client_server_interface.is_service().unwrap());
        client_server_interface.set_is_service(None).unwrap();
        assert_eq!(client_server_interface.is_service(), None);
    }
}
