//! # Crate autosar-data-abstraction
//!
//! This crate provides an abstraction layer for the AUTOSAR data model.
//! It is built on top of the crate `autosar-data` and provides complex interactions with
//! the model on top of the elementary operations of `autosar-data`.
//!
//! Since the AUTOSAR data model is very complex and has many different types of elements,
//! this crate does not aim to provide full coverage of all classes.
//! Instead the focus is on the most common classes and their interactions.
//!
//! Any other data can still be accessed through the basic operations of `autosar-data`, because the
//! calls to `autosar-data` and `autosar-data-abstraction` can be mixed freely.
//!
//! ## Example
//!
//! ```rust
//! # use autosar_data::*;
//! # use autosar_data_abstraction::*;
//! # use autosar_data_abstraction::communication::*;
//! let model = AutosarModel::new();
//! let _file = model.create_file("file.arxml", AutosarVersion::Autosar_00049).unwrap();
//! let package_1 = ArPackage::get_or_create(&model, "/System").unwrap();
//! let system = package_1.create_system("System", SystemCategory::SystemExtract).unwrap();
//! let package_2 = ArPackage::get_or_create(&model, "/Clusters").unwrap();
//!
//! // create an Ethernet cluster and a physical channel for VLAN 33
//! let eth_cluster = system.create_ethernet_cluster("EthCluster", &package_2).unwrap();
//! let vlan_info = EthernetVlanInfo {
//!     vlan_id: 33,
//!     vlan_name: "VLAN_33".to_string(),
//! };
//! let eth_channel = eth_cluster
//!     .create_physical_channel("EthChannel", Some(vlan_info))
//!     .unwrap();
//! let vlan_info_2 = eth_channel.vlan_info().unwrap();
//!
//! // create an ECU instance and connect it to the Ethernet channel
//! let package_3 = ArPackage::get_or_create(&model, "/Ecus").unwrap();
//! let ecu_instance_a = system.create_ecu_instance("Ecu_A", &package_3).unwrap();
//! let ethctrl = ecu_instance_a
//!     .create_ethernet_communication_controller(
//!         "EthernetController",
//!         Some("ab:cd:ef:01:02:03".to_string())
//!     )
//!     .unwrap();
//! let channels_iter = ethctrl.connected_channels();
//! ethctrl
//!     .connect_physical_channel("Ecu_A_connector", &eth_channel)
//!     .unwrap();
//! let channels_iter = ethctrl.connected_channels();
//!
//! // ...
//! ```

#![warn(missing_docs)]

use autosar_data::{AutosarDataError, AutosarModel, Element, EnumItem};
use thiserror::Error;

// modules that are visible in the API
pub mod communication;
pub mod datatype;
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

    /// Get the item name of the element
    #[must_use]
    fn name(&self) -> Option<String> {
        self.element().item_name()
    }

    // fn set_timestamp(&self) {
    //     todo!()
    // }
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

//#########################################################

macro_rules! element_iterator {
    ($name: ident, $output: ident, $closure: tt) => {
        #[doc(hidden)]
        pub struct $name {
            iter: Option<autosar_data::ElementsIterator>,
        }

        impl $name {
            pub(crate) fn new(elem_container: Option<Element>) -> Self {
                let iter = elem_container.map(|se| se.sub_elements());
                Self { iter }
            }
        }

        impl Iterator for $name {
            type Item = $output;

            fn next(&mut self) -> Option<Self::Item> {
                let iter = self.iter.as_mut()?;
                for element in iter.by_ref() {
                    if let Some(sub_element) = $closure(element) {
                        if let Ok(output_item) = $output::try_from(sub_element) {
                            return Some(output_item);
                        }
                    }
                }
                self.iter = None;
                None
            }
        }

        impl std::iter::FusedIterator for $name {}
    };
}

pub(crate) use element_iterator;

//#########################################################

macro_rules! reflist_iterator {
    ($name: ident, $output: ident) => {
        #[doc(hidden)]
        pub struct $name {
            items: Vec<autosar_data::WeakElement>,
            position: usize,
        }

        impl $name {
            pub(crate) fn new(items: Vec<autosar_data::WeakElement>) -> Self {
                Self { items, position: 0 }
            }
        }

        impl Iterator for $name {
            type Item = $output;

            fn next(&mut self) -> Option<Self::Item> {
                while self.position < self.items.len() {
                    if let Some(out) = self.items[self.position]
                        .upgrade()
                        .and_then(|ref_elem| ref_elem.named_parent().ok().flatten())
                        .and_then(|elem| $output::try_from(elem).ok())
                    {
                        self.position += 1;
                        return Some(out);
                    }
                    self.position += 1;
                }

                self.position = usize::MAX;
                None
            }
        }

        impl std::iter::FusedIterator for $name {}
    };
}

pub(crate) use reflist_iterator;

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

//#########################################################

#[cfg(test)]
mod test {
    use autosar_data::AutosarModel;

    use super::*;

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
}
