//! # Description
//!
//! [![Github Actions](https://github.com/DanielT/autosar-data-abstraction/actions/workflows/CI.yml/badge.svg)](https://github.com/DanielT/autosar-data-abstraction/actions)
//!
//! This crate provides an abstraction layer for the AUTOSAR data model.
//! It is built on top of the crate `autosar-data` and provides complex interactions with
//! the model on top of the elementary operations of `autosar-data`.
//!
//! Rather than transforming the element based model into a new form, it only presents a
//! view into the existing model, and provides methods to retrieve and modify the data.
//!
//! Since the AUTOSAR data model is very complex and has many different types of elements,
//! this crate does not aim to provide full coverage of all classes.
//! Instead the focus is on the most common classes and their interactions.
//!
//! Any other data can still be accessed through the basic operations of `autosar-data`, because the
//! calls to `autosar-data` and `autosar-data-abstraction` can be mixed freely.
//!
//! # Features
//!
//! Autosar Classic Platform:
//! - Communication:
//!   - Busses
//!     - CAN
//!     - Ethernet (both old and new style)
//!     - FlexRay
//!     - not supported: LIN, J1939
//!   - PDUs
//!   - Signals
//!   - Transformations: SomeIp, E2E, Com
//! - Data Types
//!   - Basic data types
//!   - Implementation data types
//!   - Application data types
//! - Software Components
//!   - Atomic SWCs, Compositions, etc.
//!   - Interfaces
//!   - Ports
//!   - Internal behavior: Runnables, Events, etc.
//! - ECU Configuration
//!
//! # Example
//!
//! ```rust
//! # use autosar_data::*;
//! # use autosar_data_abstraction::*;
//! # use autosar_data_abstraction::communication::*;
//! # fn main() -> Result<(), AutosarAbstractionError> {
//! let model = AutosarModelAbstraction::create("file.arxml", AutosarVersion::Autosar_00049);
//! let package_1 = model.get_or_create_package("/System")?;
//! let system = package_1.create_system("System", SystemCategory::SystemExtract)?;
//! let package_2 = model.get_or_create_package("/Clusters")?;
//!
//! // create an Ethernet cluster and a physical channel for VLAN 33
//! let eth_cluster = system.create_ethernet_cluster("EthCluster", &package_2)?;
//! let vlan_info = EthernetVlanInfo {
//!     vlan_id: 33,
//!     vlan_name: "VLAN_33".to_string(),
//! };
//! let eth_channel = eth_cluster.create_physical_channel("EthChannel", Some(&vlan_info))?;
//! let vlan_info_2 = eth_channel.vlan_info().unwrap();
//!
//! // create an ECU instance and connect it to the Ethernet channel
//! let package_3 = model.get_or_create_package("/Ecus")?;
//! let ecu_instance_a = system.create_ecu_instance("Ecu_A", &package_3)?;
//! let ethctrl = ecu_instance_a
//!     .create_ethernet_communication_controller(
//!         "EthernetController",
//!         Some("ab:cd:ef:01:02:03".to_string())
//!     )?;
//! let channels_iter = ethctrl.connected_channels();
//! ethctrl.connect_physical_channel("Ecu_A_connector", &eth_channel)?;
//! let channels_iter = ethctrl.connected_channels();
//!
//! // ...
//! # Ok(())}
//! ```

#![warn(missing_docs)]

use std::path::Path;

use autosar_data::{
    ArxmlFile, AutosarDataError, AutosarModel, AutosarVersion, Element, ElementName, EnumItem, WeakElement,
};
use thiserror::Error;

// modules that are visible in the API
pub mod communication;
pub mod datatype;
pub mod ecu_configuration;
pub mod software_component;

// internal modules that only serve to split up the code
mod arpackage;
mod ecuinstance;
mod system;

// export the content of the internal modules
pub use arpackage::ArPackage;
pub use ecuinstance::*;
pub use system::*;

/// The error type `AutosarAbstractionError` wraps all errors from the crate
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AutosarAbstractionError {
    /// converting an autosar-data element to a class in the abstract model failed
    #[error("conversion error: could not convert {} to {}", .element.element_name(), dest)]
    ConversionError {
        /// the element that could not be converted
        element: Element,
        /// the name of the destination type
        dest: String,
    },

    /// converting an autosar-data element to a class in the abstract model failed
    #[error("value conversion error: could not convert {} to {}", .value, .dest)]
    ValueConversionError {
        /// the value that could not be converted
        value: String,
        /// the name of the destination type
        dest: String,
    },

    /// `ModelError` wraps [`AutosarDataError`] errors from autosar-data operations, e.g.
    /// [`AutosarDataError::ItemDeleted`], [`AutosarDataError::IncorrectContentType`], ...
    #[error("model error: {}", .0)]
    ModelError(AutosarDataError),

    /// an invalid Autosar path was passed as a parameter
    #[error("invalid path: {}", .0)]
    InvalidPath(String),

    /// an item could not be created because another item already fulfills its role in the model
    #[error("the item already exists")]
    ItemAlreadyExists,

    /// the function parameter has an invalid value
    #[error("invalid parameter: {}", .0)]
    InvalidParameter(String),
}

impl From<AutosarDataError> for AutosarAbstractionError {
    fn from(err: AutosarDataError) -> Self {
        AutosarAbstractionError::ModelError(err)
    }
}

//#########################################################

/// The `AbstractionElement` trait is implemented by all classes that represent elements in the AUTOSAR model.
pub trait AbstractionElement: Clone + PartialEq + TryFrom<autosar_data::Element> {
    /// Get the underlying `Element` from the abstraction element
    #[must_use]
    fn element(&self) -> &Element;

    /// Remove this element from the model
    ///
    /// `deep` indicates whether elements that depend on this element should also be removed.
    fn remove(self, _deep: bool) -> Result<(), AutosarAbstractionError> {
        let element = self.element();
        let Some(parent) = element.parent()? else {
            // root element cannot be removed
            return Err(AutosarAbstractionError::InvalidParameter(
                "cannot remove root element".to_string(),
            ));
        };

        if element.is_identifiable() {
            let model = element.model()?;
            let path = element.path()?;
            let inbound_refs = model.get_references_to(&path);
            for ref_elem in inbound_refs.iter().filter_map(WeakElement::upgrade) {
                let Ok(Some(parent)) = ref_elem.parent() else {
                    continue;
                };
                match ref_elem.element_name() {
                    ElementName::FibexElementRef => {
                        // explicit handling of FIBEX-ELEMENTS -> FIBEX-ELEMENT-REF-CONDITIONAl -> FIBEX-ELEMENT-REF
                        if let Ok(Some(grandparent)) = parent.parent() {
                            grandparent.remove_sub_element(parent)?;
                        }
                    }
                    _ => {
                        // Fallback: just remove the reference
                        // In many cases this leaves the model in an invalid state, but it
                        // is always better than a dangling reference
                        let _ = parent.remove_sub_element(ref_elem);
                    }
                }
            }
        }

        parent.remove_sub_element(element.clone())?;
        Ok(())
    }
}

/// The `IdentifiableAbstractionElement` trait is implemented by all classes that represent elements in the AUTOSAR model that have an item name.
pub trait IdentifiableAbstractionElement: AbstractionElement {
    /// Get the item name of the element
    #[must_use]
    fn name(&self) -> Option<String> {
        self.element().item_name()
    }

    /// Set the item name of the element
    fn set_name(&self, name: &str) -> Result<(), AutosarAbstractionError> {
        self.element().set_item_name(name)?;
        Ok(())
    }
}

macro_rules! abstraction_element {
    ($name: ident, $base_elem: ident) => {
        impl TryFrom<autosar_data::Element> for $name {
            type Error = AutosarAbstractionError;

            fn try_from(element: autosar_data::Element) -> Result<Self, Self::Error> {
                if element.element_name() == autosar_data::ElementName::$base_elem {
                    Ok($name(element))
                } else {
                    Err(AutosarAbstractionError::ConversionError {
                        element,
                        dest: stringify!($name).to_string(),
                    })
                }
            }
        }

        impl AbstractionElement for $name {
            fn element(&self) -> &autosar_data::Element {
                &self.0
            }
        }

        impl From<$name> for autosar_data::Element {
            fn from(val: $name) -> Self {
                val.0
            }
        }
    };
}

pub(crate) use abstraction_element;

//#########################################################

/// The `AutosarModelAbstraction` wraps an `AutosarModel` and provides additional functionality
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutosarModelAbstraction(AutosarModel);

impl AutosarModelAbstraction {
    /// Create a new `AutosarModelAbstraction` from an `AutosarModel`
    #[must_use]
    pub fn new(model: AutosarModel) -> Self {
        Self(model)
    }

    /// create a new `AutosarModelAbstraction` with an empty `AutosarModel`
    ///
    /// You must specify a file name for the initial file in the model. This file is not created on disk immediately.
    /// The model also needs an `AutosarVersion`.
    pub fn create<P: AsRef<Path>>(file_name: P, version: AutosarVersion) -> Self {
        let model = AutosarModel::new();
        // create the initial file in the model - create_file can return a DuplicateFileName
        // error, but hthis is not a concern for the first file, so it is always safe to unwrap
        model.create_file(file_name, version).unwrap();
        Self(model)
    }

    /// create an `AutosarModelAbstraction` from a file on disk
    pub fn from_file<P: AsRef<Path>>(file_name: P) -> Result<Self, AutosarAbstractionError> {
        let model = AutosarModel::new();
        model.load_file(file_name, true)?;
        Ok(Self(model))
    }

    /// Create an `AutosarModelAbstraction` from a buffer
    ///
    /// Since all autosar data is always associated with a file, a file name must be provided
    /// but no file is created on disk unless you call `write()`.
    pub fn from_buffer<P: AsRef<Path>>(
        buffer: &[u8],
        file_name: P,
        strict: bool,
    ) -> Result<Self, AutosarAbstractionError> {
        let model = AutosarModel::new();
        model.load_buffer(buffer, file_name, strict)?;
        Ok(Self(model))
    }

    /// Get the underlying `AutosarModel` from the abstraction model
    #[must_use]
    pub fn model(&self) -> &AutosarModel {
        &self.0
    }

    /// Get the root element of the model
    #[must_use]
    pub fn root_element(&self) -> Element {
        self.0.root_element()
    }

    /// iterate over all top-level packages
    pub fn packages(&self) -> impl Iterator<Item = ArPackage> + Send + use<> {
        self.0
            .root_element()
            .get_sub_element(ElementName::ArPackages)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|elem| ArPackage::try_from(elem).ok())
    }

    /// Get a package by its path or create it if it does not exist
    pub fn get_or_create_package(&self, path: &str) -> Result<ArPackage, AutosarAbstractionError> {
        ArPackage::get_or_create(&self.0, path)
    }

    /// Create a new file in the model
    pub fn create_file(&self, file_name: &str, version: AutosarVersion) -> Result<ArxmlFile, AutosarAbstractionError> {
        let arxml_file = self.0.create_file(file_name, version)?;
        Ok(arxml_file)
    }

    /// Load a file into the model
    pub fn load_file<P: AsRef<Path>>(
        &self,
        file_name: P,
        strict: bool,
    ) -> Result<(ArxmlFile, Vec<AutosarDataError>), AutosarAbstractionError> {
        let value = self.0.load_file(file_name, strict)?;
        Ok(value)
    }

    /// Load a buffer into the model
    pub fn load_buffer<P: AsRef<Path>>(
        &self,
        buffer: &[u8],
        file_name: P,
        strict: bool,
    ) -> Result<(ArxmlFile, Vec<AutosarDataError>), AutosarAbstractionError> {
        let value = self.0.load_buffer(buffer, file_name, strict)?;
        Ok(value)
    }

    /// iterate over all files in the model
    pub fn files(&self) -> impl Iterator<Item = ArxmlFile> + Send + use<> {
        self.0.files()
    }

    /// write the model to disk, creating or updating all files in the model
    pub fn write(&self) -> Result<(), AutosarAbstractionError> {
        self.0.write()?;
        Ok(())
    }

    /// Get an element by its path
    #[must_use]
    pub fn get_element_by_path(&self, path: &str) -> Option<Element> {
        self.0.get_element_by_path(path)
    }

    /// find an existing SYSTEM in the model, if it exists
    ///
    /// # Example
    ///
    /// ```
    /// # use autosar_data::*;
    /// # use autosar_data_abstraction::*;
    /// # fn main() -> Result<(), AutosarAbstractionError> {
    /// let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00048);
    /// # let package = model.get_or_create_package("/my/pkg")?;
    /// let system = package.create_system("System", SystemCategory::SystemExtract)?;
    /// if let Some(sys_2) = model.find_system() {
    ///     assert_eq!(system, sys_2);
    /// }
    /// # Ok(())}
    /// ```
    #[must_use]
    pub fn find_system(&self) -> Option<System> {
        System::find(&self.0)
    }
}

//#########################################################

/// The `ByteOrder` is used to define the order of bytes in a multi-byte value
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    /// Most significant byte at the lowest address = big endian
    MostSignificantByteFirst,
    /// Most significant byte at the highest address = little endian
    MostSignificantByteLast,
    /// The byte order is not defined / not relevant
    Opaque,
}

impl TryFrom<EnumItem> for ByteOrder {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<Self, Self::Error> {
        match value {
            EnumItem::MostSignificantByteFirst => Ok(ByteOrder::MostSignificantByteFirst),
            EnumItem::MostSignificantByteLast => Ok(ByteOrder::MostSignificantByteLast),
            EnumItem::Opaque => Ok(ByteOrder::Opaque),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "ByteOrder".to_string(),
            }),
        }
    }
}

impl From<ByteOrder> for EnumItem {
    fn from(value: ByteOrder) -> Self {
        match value {
            ByteOrder::MostSignificantByteFirst => EnumItem::MostSignificantByteFirst,
            ByteOrder::MostSignificantByteLast => EnumItem::MostSignificantByteLast,
            ByteOrder::Opaque => EnumItem::Opaque,
        }
    }
}

//##################################################################

pub(crate) fn make_unique_name(model: &AutosarModel, base_path: &str, initial_name: &str) -> String {
    let mut full_path = format!("{base_path}/{initial_name}");
    let mut name = initial_name.to_string();
    let mut counter = 0;
    while model.get_element_by_path(&full_path).is_some() {
        counter += 1;
        name = format!("{initial_name}_{counter}");
        full_path = format!("{base_path}/{name}");
    }

    name
}

//##################################################################

/// check if the element is used anywhere
pub(crate) fn is_used(element: &Element) -> bool {
    let Ok(model) = element.model() else {
        // not connected to model -> unused
        return false;
    };
    let Ok(path) = element.path() else {
        // not connected to model any more -> unused
        // this case is only reachable through a race condition (parallel deletion)
        return false;
    };
    let references = model.get_references_to(&path);

    // it is unused if there are no references to it
    !references.is_empty()
}

//##################################################################

// returns the named parent and the parent of each element that references the given element
pub(crate) fn get_reference_parents(element: &Element) -> Result<Vec<(Element, Element)>, AutosarAbstractionError> {
    let model = element.model()?;
    let path = element.path()?;
    let references = model.get_references_to(&path);

    let parents = references
        .iter()
        .filter_map(WeakElement::upgrade)
        .filter_map(|ref_elem| {
            Some((
                ref_elem.named_parent().ok().flatten()?,
                ref_elem.parent().ok().flatten()?,
            ))
        })
        .collect();

    Ok(parents)
}

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use autosar_data::AutosarModel;

    #[test]
    fn create_model() {
        // create a new AutosarModelAbstraction based on an existing AutosarModel
        let raw_model = AutosarModel::new();
        let model = AutosarModelAbstraction::new(raw_model.clone());
        assert_eq!(model.model(), &raw_model);

        // create an empty AutosarModelAbstraction from scratch
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00049);
        let root = model.root_element();
        assert_eq!(root.element_name(), ElementName::Autosar);
    }

    #[test]
    fn create_model_from_file() {
        let tempdir = tempfile::tempdir().unwrap();
        let filename = tempdir.path().join("test.arxml");

        // write a new arxml file to disk
        let model1 = AutosarModelAbstraction::create(filename.clone(), AutosarVersion::LATEST);
        model1.write().unwrap();

        // create a new model from the file
        let model2 = AutosarModelAbstraction::from_file(filename).unwrap();
        let root = model2.root_element();
        assert_eq!(root.element_name(), ElementName::Autosar);
    }

    #[test]
    fn model_files() {
        let model = AutosarModelAbstraction::create("file1.arxml", AutosarVersion::Autosar_00049);
        let file = model.create_file("file2.arxml", AutosarVersion::Autosar_00049).unwrap();
        let files: Vec<_> = model.files().collect();
        assert_eq!(files.len(), 2);
        assert_eq!(files[1], file);
    }

    #[test]
    fn model_load_file() {
        let tempdir = tempfile::tempdir().unwrap();
        let filename = tempdir.path().join("test.arxml");

        // write a new arxml file to disk
        let model = AutosarModelAbstraction::create(filename.clone(), AutosarVersion::LATEST);
        model.write().unwrap();

        // load the file into a new model
        let model = AutosarModelAbstraction::new(AutosarModel::new());
        let (_file, errors) = model.load_file(filename, true).unwrap();
        assert!(errors.is_empty());
    }

    #[test]
    fn model_packages() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::Autosar_00049);
        let package = model.get_or_create_package("/package").unwrap();
        let package2 = model.get_or_create_package("/other_package").unwrap();
        model.get_or_create_package("/other_package/sub_package").unwrap();
        let packages: Vec<_> = model.packages().collect();
        assert_eq!(packages.len(), 2);
        assert_eq!(packages[0], package);
        assert_eq!(packages[1], package2);
    }

    #[test]
    fn errors() {
        let model = AutosarModel::new();

        let err = AutosarAbstractionError::ConversionError {
            element: model.root_element(),
            dest: "TEST".to_string(),
        };
        let string = format!("{err}");
        assert!(!string.is_empty());

        let err = AutosarAbstractionError::InvalidPath("lorem ipsum".to_string());
        let string = format!("{err}");
        assert!(!string.is_empty());

        let err = AutosarAbstractionError::ItemAlreadyExists;
        let string = format!("{err}");
        assert!(!string.is_empty());
    }

    #[test]
    fn from_buffer() {
        let buffer = br#"
        <?xml version="1.0" encoding="UTF-8" standalone="no"?>
        <AUTOSAR xmlns="http://autosar.org/schema/r4.0" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://autosar.org/schema/r4.0 AUTOSAR_00046.xsd">
            <AR-PACKAGES>
                <AR-PACKAGE>
                    <SHORT-NAME>MyPackage</SHORT-NAME>
                </AR-PACKAGE>
            </AR-PACKAGES>
        </AUTOSAR>
        "#;

        let model = AutosarModelAbstraction::from_buffer(buffer, "buffer.arxml", true).unwrap();
        let package = model.get_or_create_package("/MyPackage").unwrap();
        assert_eq!(package.name().unwrap(), "MyPackage");
    }

    #[test]
    fn load_buffer() {
        let model = AutosarModelAbstraction::create("dummy", AutosarVersion::LATEST);
        let buffer = br#"
        <?xml version="1.0" encoding="UTF-8" standalone="no"?>
        <AUTOSAR xmlns="http://autosar.org/schema/r4.0" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://autosar.org/schema/r4.0 AUTOSAR_00046.xsd">
            <AR-PACKAGES>
                <AR-PACKAGE>
                    <SHORT-NAME>MyPackage</SHORT-NAME>
                </AR-PACKAGE>
            </AR-PACKAGES>
        </AUTOSAR>
        "#;
        let (_file, errors) = model.load_buffer(buffer, "buffer.arxml", true).unwrap();
        assert!(errors.is_empty());
        assert_eq!(model.files().count(), 2);
    }
}
