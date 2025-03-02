use crate::IdentifiableAbstractionElement;
use crate::communication::{
    AbstractPhysicalChannel, CommunicationDirection, DataTransformation, EndToEndTransformationISignalProps,
    PhysicalChannel, SomeIpTransformationISignalProps, TransformationTechnology,
};
use crate::datatype::{CompuMethod, DataConstr, SwBaseType, Unit};
use crate::{
    AbstractionElement, ArPackage, AutosarAbstractionError, EcuInstance, abstraction_element,
    communication::ISignalToIPduMapping, make_unique_name, reflist_iterator,
};
use autosar_data::{AutosarDataError, Element, ElementName, EnumItem, WeakElement};

use super::TransformationISignalProps;

/// Signal of the Interaction Layer
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ISignal(Element);
abstraction_element!(ISignal, ISignal);
impl IdentifiableAbstractionElement for ISignal {}

impl ISignal {
    pub(crate) fn new(
        name: &str,
        package: &ArPackage,
        bit_length: u64,
        syssignal: &SystemSignal,
        datatype: Option<&SwBaseType>,
    ) -> Result<Self, AutosarAbstractionError> {
        if bit_length > u64::from(u32::MAX) * 8 {
            // max bit_length is 2^32 bytes
            return Err(AutosarAbstractionError::InvalidParameter(format!(
                "isignal {name}: bit length {bit_length} is too big"
            )));
        }

        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_isignal = pkg_elements.create_named_sub_element(ElementName::ISignal, name)?;

        elem_isignal
            .create_sub_element(ElementName::DataTypePolicy)?
            .set_character_data(EnumItem::Override)?;

        let isignal = Self(elem_isignal);
        isignal.set_length(bit_length)?;
        isignal.set_system_signal(syssignal)?;

        if let Some(datatype) = datatype {
            isignal.set_datatype(datatype)?;
        }

        Ok(isignal)
    }

    /// set the data type for this signal
    pub fn set_datatype(&self, datatype: &SwBaseType) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::NetworkRepresentationProps)?
            .get_or_create_sub_element(ElementName::SwDataDefPropsVariants)?
            .get_or_create_sub_element(ElementName::SwDataDefPropsConditional)?
            .get_or_create_sub_element(ElementName::BaseTypeRef)?
            .set_reference_target(datatype.element())?;
        Ok(())
    }

    /// get the data type of this signal
    pub fn datatype(&self) -> Option<SwBaseType> {
        self.element()
            .get_sub_element(ElementName::NetworkRepresentationProps)?
            .get_sub_element(ElementName::SwDataDefPropsVariants)?
            .get_sub_element(ElementName::SwDataDefPropsConditional)?
            .get_sub_element(ElementName::BaseTypeRef)?
            .get_reference_target()
            .ok()?
            .try_into()
            .ok()
    }

    /// set the length of this signal in bits
    pub fn set_length(&self, bit_length: u64) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::Length)?
            .set_character_data(bit_length)?;
        Ok(())
    }

    /// get the length of this signal in bits
    #[must_use]
    pub fn length(&self) -> Option<u64> {
        self.element()
            .get_sub_element(ElementName::Length)?
            .character_data()?
            .parse_integer()
    }

    /// set the system signal that corresponds to this signal
    pub fn set_system_signal(&self, syssignal: &SystemSignal) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::SystemSignalRef)?
            .set_reference_target(syssignal.element())?;
        Ok(())
    }

    /// get the system signal that corresponds to this isignal
    #[must_use]
    pub fn system_signal(&self) -> Option<SystemSignal> {
        self.element()
            .get_sub_element(ElementName::SystemSignalRef)?
            .get_reference_target()
            .ok()?
            .try_into()
            .ok()
    }

    /// an iterator over all `ISignalToIPduMapping` for this signal
    ///
    /// Usually a signal should only be mapped to a single PDU,
    /// so this iterator is expected to return either zero or one item in ordinary cases.
    pub fn mappings(&self) -> impl Iterator<Item = ISignalToIPduMapping> + Send + 'static {
        let model_result = self.element().model();
        let path_result = self.element().path();
        if let (Ok(model), Ok(path)) = (model_result, path_result) {
            let reflist = model.get_references_to(&path);
            ISignalToIPduMappingsIterator::new(reflist)
        } else {
            ISignalToIPduMappingsIterator::new(vec![])
        }
    }

    /// get the signal group that contains this signal, if any
    pub fn signal_group(&self) -> Option<ISignalGroup> {
        let path = self.element().path().ok()?;
        let referrers = self.element().model().ok()?.get_references_to(&path);

        for elem in referrers
            .iter()
            .filter_map(|weak| weak.upgrade().and_then(|elem| elem.named_parent().ok().flatten()))
        {
            if let Ok(grp) = ISignalGroup::try_from(elem) {
                return Some(grp);
            }
        }
        None
    }

    /// add a data transformation to this signal
    pub fn add_data_transformation(
        &self,
        data_transformation: &DataTransformation,
    ) -> Result<(), AutosarAbstractionError> {
        let transformations = self
            .element()
            .get_or_create_sub_element(ElementName::DataTransformations)?;
        transformations
            .create_sub_element(ElementName::DataTransformationRefConditional)?
            .create_sub_element(ElementName::DataTransformationRef)?
            .set_reference_target(data_transformation.element())?;

        Ok(())
    }

    /// get all data transformations that are applied to this signal
    pub fn data_transformations(&self) -> impl Iterator<Item = DataTransformation> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::DataTransformations)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|elem| elem.get_sub_element(ElementName::DataTransformationRef))
            .filter_map(|elem| elem.get_reference_target().ok())
            .filter_map(|elem| elem.try_into().ok())
    }

    /// create E2E transformation properties for this signal
    pub fn create_e2e_transformation_isignal_props(
        &self,
        transformer: &TransformationTechnology,
    ) -> Result<EndToEndTransformationISignalProps, AutosarAbstractionError> {
        let tsp = self
            .element()
            .get_or_create_sub_element(ElementName::TransformationISignalPropss)?;
        EndToEndTransformationISignalProps::new(tsp, transformer)
    }

    /// create SomeIp transformation properties for this signal
    pub fn create_someip_transformation_isignal_props(
        &self,
        transformer: &TransformationTechnology,
    ) -> Result<SomeIpTransformationISignalProps, AutosarAbstractionError> {
        let tsp = self
            .element()
            .get_or_create_sub_element(ElementName::TransformationISignalPropss)?;
        SomeIpTransformationISignalProps::new(tsp, transformer)
    }

    /// get all transformation properties that are applied to this signal
    pub fn transformation_isignal_props(&self) -> impl Iterator<Item = TransformationISignalProps> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::TransformationISignalPropss)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|elem| TransformationISignalProps::try_from(elem).ok())
    }
}

//##################################################################

/// The system signal represents the communication system's view of data exchanged between SW components which reside on different ECUs
///
/// Use [`ArPackage::create_system_signal`] to create a new system signal
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SystemSignal(Element);
abstraction_element!(SystemSignal, SystemSignal);
impl IdentifiableAbstractionElement for SystemSignal {}

impl SystemSignal {
    /// Create a new system signal in the given package
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let package_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_syssignal = package_elements.create_named_sub_element(ElementName::SystemSignal, name)?;

        Ok(Self(elem_syssignal))
    }

    /// get the signal group that contains this signal
    pub fn signal_group(&self) -> Option<SystemSignalGroup> {
        let path = self.element().path().ok()?;
        let referrers = self.element().model().ok()?.get_references_to(&path);
        for elem in referrers
            .iter()
            .filter_map(WeakElement::upgrade)
            .filter_map(|refelem| refelem.named_parent().ok().flatten())
        {
            if let Ok(grp) = SystemSignalGroup::try_from(elem) {
                return Some(grp);
            }
        }
        None
    }

    /// set the unit for this signal
    pub fn set_unit(&self, unit: &Unit) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::PhysicalProps)?
            .get_or_create_sub_element(ElementName::SwDataDefPropsVariants)?
            .get_or_create_sub_element(ElementName::SwDataDefPropsConditional)?
            .get_or_create_sub_element(ElementName::UnitRef)?
            .set_reference_target(unit.element())?;
        Ok(())
    }

    /// get the unit for this signal
    #[must_use]
    pub fn unit(&self) -> Option<Unit> {
        self.element()
            .get_sub_element(ElementName::PhysicalProps)?
            .get_sub_element(ElementName::SwDataDefPropsVariants)?
            .get_sub_element(ElementName::SwDataDefPropsConditional)?
            .get_sub_element(ElementName::UnitRef)?
            .get_reference_target()
            .ok()?
            .try_into()
            .ok()
    }

    /// set the compu method for this signal
    pub fn set_compu_method(&self, compu_method: &CompuMethod) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::PhysicalProps)?
            .get_or_create_sub_element(ElementName::SwDataDefPropsVariants)?
            .get_or_create_sub_element(ElementName::SwDataDefPropsConditional)?
            .get_or_create_sub_element(ElementName::CompuMethodRef)?
            .set_reference_target(compu_method.element())?;
        Ok(())
    }

    /// get the compu method for this signal
    #[must_use]
    pub fn compu_method(&self) -> Option<CompuMethod> {
        self.element()
            .get_sub_element(ElementName::PhysicalProps)?
            .get_sub_element(ElementName::SwDataDefPropsVariants)?
            .get_sub_element(ElementName::SwDataDefPropsConditional)?
            .get_sub_element(ElementName::CompuMethodRef)?
            .get_reference_target()
            .ok()?
            .try_into()
            .ok()
    }

    /// set the data constraint for this signal
    pub fn set_data_constr(&self, data_constr: &DataConstr) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::PhysicalProps)?
            .get_or_create_sub_element(ElementName::SwDataDefPropsVariants)?
            .get_or_create_sub_element(ElementName::SwDataDefPropsConditional)?
            .get_or_create_sub_element(ElementName::DataConstrRef)?
            .set_reference_target(data_constr.element())?;
        Ok(())
    }

    /// get the data constraint for this signal
    #[must_use]
    pub fn data_constr(&self) -> Option<DataConstr> {
        self.element()
            .get_sub_element(ElementName::PhysicalProps)?
            .get_sub_element(ElementName::SwDataDefPropsVariants)?
            .get_sub_element(ElementName::SwDataDefPropsConditional)?
            .get_sub_element(ElementName::DataConstrRef)?
            .get_reference_target()
            .ok()?
            .try_into()
            .ok()
    }
}

//##################################################################

/// An `ISignalGroup` groups signals that should always be kept together
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ISignalGroup(Element);
abstraction_element!(ISignalGroup, ISignalGroup);
impl IdentifiableAbstractionElement for ISignalGroup {}

impl ISignalGroup {
    pub(crate) fn new(
        name: &str,
        package: &ArPackage,
        system_signal_group: &SystemSignalGroup,
    ) -> Result<Self, AutosarAbstractionError> {
        let sig_pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let elem_isiggrp = sig_pkg_elements.create_named_sub_element(ElementName::ISignalGroup, name)?;

        elem_isiggrp
            .create_sub_element(ElementName::SystemSignalGroupRef)?
            .set_reference_target(system_signal_group.element())?;

        Ok(Self(elem_isiggrp))
    }

    /// Add a signal to the signal group
    pub fn add_signal(&self, signal: &ISignal) -> Result<(), AutosarAbstractionError> {
        // make sure the relation of signal to signal group is maintained for the referenced system signal
        let syssig_grp_of_signal = signal.system_signal().and_then(|ss| ss.signal_group());
        let syssig_grp = self.system_signal_group();
        if syssig_grp != syssig_grp_of_signal {
            return Err(AutosarAbstractionError::InvalidParameter(
                "The isignal and the system signal must both be part of corresponding signal groups".to_string(),
            ));
        }

        let isrefs = self.element().get_or_create_sub_element(ElementName::ISignalRefs)?;

        // check if the signal already exists in isrefs?

        isrefs
            .create_sub_element(ElementName::ISignalRef)?
            .set_reference_target(signal.element())?;

        Ok(())
    }

    /// get the system signal group that is associated with this signal group
    #[must_use]
    pub fn system_signal_group(&self) -> Option<SystemSignalGroup> {
        self.element()
            .get_sub_element(ElementName::SystemSignalGroupRef)?
            .get_reference_target()
            .ok()?
            .try_into()
            .ok()
    }

    /// Iterator over all [`ISignal`]s in this group
    ///
    /// # Example
    pub fn signals(&self) -> impl Iterator<Item = ISignal> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::ISignalRefs)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|elem| {
                elem.get_reference_target()
                    .ok()
                    .and_then(|elem| ISignal::try_from(elem).ok())
            })
    }

    /// add a data transformation to this signal group
    pub fn add_data_transformation(
        &self,
        data_transformation: &DataTransformation,
    ) -> Result<(), AutosarAbstractionError> {
        let cbst = self
            .element()
            .get_or_create_sub_element(ElementName::ComBasedSignalGroupTransformations)?;
        cbst.create_sub_element(ElementName::DataTransformationRefConditional)?
            .create_sub_element(ElementName::DataTransformationRef)?
            .set_reference_target(data_transformation.element())?;
        Ok(())
    }

    /// get all data transformations that are applied to this signal group
    pub fn data_transformations(&self) -> impl Iterator<Item = DataTransformation> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::ComBasedSignalGroupTransformations)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|elem| elem.get_sub_element(ElementName::DataTransformationRef))
            .filter_map(|elem| elem.get_reference_target().ok())
            .filter_map(|elem| elem.try_into().ok())
    }

    /// create E2E transformation properties for this signal group
    pub fn create_e2e_transformation_isignal_props(
        &self,
        transformer: &TransformationTechnology,
    ) -> Result<EndToEndTransformationISignalProps, AutosarAbstractionError> {
        let tsp = self
            .element()
            .get_or_create_sub_element(ElementName::TransformationISignalPropss)?;
        EndToEndTransformationISignalProps::new(tsp, transformer)
    }

    /// create SomeIp transformation properties for this signal group
    pub fn create_someip_transformation_isignal_props(
        &self,
        transformer: &TransformationTechnology,
    ) -> Result<SomeIpTransformationISignalProps, AutosarAbstractionError> {
        let tsp = self
            .element()
            .get_or_create_sub_element(ElementName::TransformationISignalPropss)?;
        SomeIpTransformationISignalProps::new(tsp, transformer)
    }

    /// get all transformation properties that are applied to this signal group
    pub fn transformation_isignal_props(&self) -> impl Iterator<Item = TransformationISignalProps> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::TransformationISignalPropss)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|elem| TransformationISignalProps::try_from(elem).ok())
    }
}

//##################################################################

/// A signal group refers to a set of signals that shall always be kept together. A signal group is used to
/// guarantee the atomic transfer of AUTOSAR composite data types.
///
/// Use [`ArPackage::create_system_signal_group`] to create a new system signal group
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SystemSignalGroup(Element);
abstraction_element!(SystemSignalGroup, SystemSignalGroup);
impl IdentifiableAbstractionElement for SystemSignalGroup {}

impl SystemSignalGroup {
    /// Create a new system signal group
    pub(crate) fn new(name: &str, package: &ArPackage) -> Result<Self, AutosarAbstractionError> {
        let pkg_elements = package.element().get_or_create_sub_element(ElementName::Elements)?;
        let signalgroup = pkg_elements.create_named_sub_element(ElementName::SystemSignalGroup, name)?;

        Ok(Self(signalgroup))
    }

    /// Add a signal to the signal group
    pub fn add_signal(&self, signal: &SystemSignal) -> Result<(), AutosarAbstractionError> {
        let ssrefs = self
            .element()
            .get_or_create_sub_element(ElementName::SystemSignalRefs)?;

        // check if the signal already exists in ssrefs?

        ssrefs
            .create_sub_element(ElementName::SystemSignalRef)?
            .set_reference_target(signal.element())?;

        Ok(())
    }
}

//##################################################################

/// an `ISignalTriggering` triggers a signal in a PDU
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ISignalTriggering(Element);
abstraction_element!(ISignalTriggering, ISignalTriggering);
impl IdentifiableAbstractionElement for ISignalTriggering {}

impl ISignalTriggering {
    pub(crate) fn new<T: AbstractPhysicalChannel>(
        signal: &ISignal,
        channel: &T,
    ) -> Result<Self, AutosarAbstractionError> {
        let model = channel.element().model()?;
        let base_path = channel.element().path()?;
        let signal_name = signal
            .name()
            .ok_or(AutosarAbstractionError::InvalidParameter("invalid signal".to_string()))?;
        let pt_name = format!("ST_{signal_name}");
        let pt_name = make_unique_name(&model, &base_path, &pt_name);

        let triggerings = channel
            .element()
            .get_or_create_sub_element(ElementName::ISignalTriggerings)?;
        let st_elem = triggerings.create_named_sub_element(ElementName::ISignalTriggering, &pt_name)?;
        st_elem
            .create_sub_element(ElementName::ISignalRef)?
            .set_reference_target(signal.element())?;

        let pt = Self(st_elem);

        Ok(pt)
    }

    pub(crate) fn new_group(
        signal_group: &ISignalGroup,
        channel: &PhysicalChannel,
    ) -> Result<Self, AutosarAbstractionError> {
        let model = channel.element().model()?;
        let base_path = channel.element().path()?;
        let signal_name = signal_group.name().ok_or(AutosarAbstractionError::InvalidParameter(
            "invalid signal group".to_string(),
        ))?;
        let pt_name = format!("ST_{signal_name}");
        let pt_name = make_unique_name(&model, &base_path, &pt_name);

        let triggerings = channel
            .element()
            .get_or_create_sub_element(ElementName::ISignalTriggerings)?;
        let st_elem = triggerings.create_named_sub_element(ElementName::ISignalTriggering, &pt_name)?;
        st_elem
            .create_sub_element(ElementName::ISignalGroupRef)?
            .set_reference_target(signal_group.element())?;

        let pt = Self(st_elem);

        Ok(pt)
    }

    /// get the physical channel that contains this signal triggering
    pub fn physical_channel(&self) -> Result<PhysicalChannel, AutosarAbstractionError> {
        let channel_elem = self.element().named_parent()?.ok_or(AutosarDataError::ItemDeleted)?;
        PhysicalChannel::try_from(channel_elem)
    }

    /// connect this signal triggering to an ECU
    pub fn connect_to_ecu(
        &self,
        ecu: &EcuInstance,
        direction: CommunicationDirection,
    ) -> Result<ISignalPort, AutosarAbstractionError> {
        for signal_port in self.signal_ports() {
            if let (Ok(existing_ecu), Some(existing_direction)) =
                (signal_port.ecu(), signal_port.communication_direction())
            {
                if existing_ecu == *ecu && existing_direction == direction {
                    return Ok(signal_port);
                }
            }
        }

        let channel = self.physical_channel()?;
        let connector = channel
            .ecu_connector(ecu)
            .ok_or(AutosarAbstractionError::InvalidParameter(
                "The ECU is not connected to the channel".to_string(),
            ))?;

        let name = self.name().ok_or(AutosarDataError::ItemDeleted)?;
        let suffix = match direction {
            CommunicationDirection::In => "Rx",
            CommunicationDirection::Out => "Tx",
        };
        let port_name = format!("{name}_{suffix}",);
        let sp_elem = connector
            .element()
            .get_or_create_sub_element(ElementName::EcuCommPortInstances)?
            .create_named_sub_element(ElementName::ISignalPort, &port_name)?;
        sp_elem
            .create_sub_element(ElementName::CommunicationDirection)?
            .set_character_data::<EnumItem>(direction.into())?;

        self.element()
            .get_or_create_sub_element(ElementName::ISignalPortRefs)?
            .create_sub_element(ElementName::ISignalPortRef)?
            .set_reference_target(&sp_elem)?;

        Ok(ISignalPort(sp_elem))
    }

    /// create an iterator over all signal ports that are connected to this signal triggering
    pub fn signal_ports(&self) -> impl Iterator<Item = ISignalPort> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::ISignalPortRefs)
            .into_iter()
            .flat_map(|elem| elem.sub_elements())
            .filter_map(|elem| {
                elem.get_reference_target()
                    .ok()
                    .and_then(|elem| ISignalPort::try_from(elem).ok())
            })
    }
}

//##################################################################

/// The `ISignalPort` allows an ECU to send or receive a Signal
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ISignalPort(Element);
abstraction_element!(ISignalPort, ISignalPort);
impl IdentifiableAbstractionElement for ISignalPort {}

impl ISignalPort {
    /// get the ECU that is connected to this signal port
    pub fn ecu(&self) -> Result<EcuInstance, AutosarAbstractionError> {
        let comm_connector_elem = self.element().named_parent()?.unwrap();
        let ecu_elem = comm_connector_elem.named_parent()?.unwrap();
        EcuInstance::try_from(ecu_elem)
    }

    /// set the communication direction of this port
    pub fn set_communication_direction(
        &self,
        direction: CommunicationDirection,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::CommunicationDirection)?
            .set_character_data::<EnumItem>(direction.into())?;
        Ok(())
    }

    /// get the communication direction of this port
    #[must_use]
    pub fn communication_direction(&self) -> Option<CommunicationDirection> {
        self.element()
            .get_sub_element(ElementName::CommunicationDirection)?
            .character_data()?
            .enum_value()?
            .try_into()
            .ok()
    }
}

//##################################################################

/// The `TransferProperty` defines if or how the signal influences the transfer of the PDU
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransferProperty {
    /// The signal is pending; it does not trigger the transfer of the PDU
    Pending,
    /// The signal triggers the transfer of the PDU
    Triggered,
    /// The signal triggers the transfer of the PDU if the value changes
    TriggeredOnChange,
    /// The signal triggers the transfer of the PDU if the value changes without repetition
    TriggeredOnChangeWithoutRepetition,
    /// The signal triggers the transfer of the PDU without repetition
    TriggeredWithoutRepetition,
}

impl From<TransferProperty> for EnumItem {
    fn from(value: TransferProperty) -> Self {
        match value {
            TransferProperty::Pending => EnumItem::Pending,
            TransferProperty::Triggered => EnumItem::Triggered,
            TransferProperty::TriggeredOnChange => EnumItem::TriggeredOnChange,
            TransferProperty::TriggeredOnChangeWithoutRepetition => EnumItem::TriggeredOnChangeWithoutRepetition,
            TransferProperty::TriggeredWithoutRepetition => EnumItem::TriggeredWithoutRepetition,
        }
    }
}

impl TryFrom<EnumItem> for TransferProperty {
    type Error = AutosarAbstractionError;

    fn try_from(value: EnumItem) -> Result<Self, Self::Error> {
        match value {
            EnumItem::Pending => Ok(TransferProperty::Pending),
            EnumItem::Triggered => Ok(TransferProperty::Triggered),
            EnumItem::TriggeredOnChange => Ok(TransferProperty::TriggeredOnChange),
            EnumItem::TriggeredOnChangeWithoutRepetition => Ok(TransferProperty::TriggeredOnChangeWithoutRepetition),
            EnumItem::TriggeredWithoutRepetition => Ok(TransferProperty::TriggeredWithoutRepetition),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: value.to_string(),
                dest: "TransferProperty".to_string(),
            }),
        }
    }
}

//##################################################################

reflist_iterator!(ISignalToIPduMappingsIterator, ISignalToIPduMapping);

//##################################################################

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AutosarModelAbstraction, ByteOrder, SystemCategory,
        communication::{
            CanClusterSettings, DataTransformationSet, SomeIpMessageType, SomeIpTransformationTechnologyConfig,
            TransformationTechnologyConfig,
        },
        datatype::{BaseTypeEncoding, CompuMethodContent, SwBaseType, Unit},
    };
    use autosar_data::AutosarVersion;

    #[test]
    fn test_signal() {
        let model = AutosarModelAbstraction::create("test.arxml", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/test").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();
        let unit = Unit::new("unit", &package, Some("Unit Name")).unwrap();
        let compu_method = CompuMethod::new("compu_method", &package, CompuMethodContent::Identical).unwrap();
        let data_constr = DataConstr::new("data_constr", &package).unwrap();
        let sw_base_type =
            SwBaseType::new("sw_base_type", &package, 8, BaseTypeEncoding::None, None, None, None).unwrap();

        let sys_signal = package.create_system_signal("sys_signal").unwrap();
        let signal = system
            .create_isignal("signal", &package, 8, &sys_signal, Some(&sw_base_type))
            .unwrap();

        sys_signal.set_unit(&unit).unwrap();
        sys_signal.set_compu_method(&compu_method).unwrap();
        sys_signal.set_data_constr(&data_constr).unwrap();

        assert_eq!(signal.length(), Some(8));
        assert_eq!(signal.datatype(), Some(sw_base_type));
        assert_eq!(signal.system_signal(), Some(sys_signal.clone()));
        assert_eq!(sys_signal.unit(), Some(unit));
        assert_eq!(sys_signal.compu_method(), Some(compu_method));
        assert_eq!(sys_signal.data_constr(), Some(data_constr));

        // mappings
        assert_eq!(signal.mappings().count(), 0);
        let ipdu = system.create_isignal_ipdu("ipdu", &package, 8).unwrap();
        let mapping = ipdu
            .map_signal(
                &signal,
                0,
                ByteOrder::MostSignificantByteLast,
                None,
                TransferProperty::Triggered,
            )
            .unwrap();
        assert_eq!(signal.mappings().count(), 1);
        assert_eq!(signal.mappings().next(), Some(mapping.clone()));
        assert_eq!(mapping.signal().unwrap(), signal);
    }

    #[test]
    fn test_signal_data_transformations() {
        let model = AutosarModelAbstraction::create("test.arxml", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/test").unwrap();
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

        let dts = DataTransformationSet::new("data_transformation_set", &package).unwrap();
        let transformer = dts
            .create_transformation_technology(
                "someip_xf",
                &TransformationTechnologyConfig::SomeIp(SomeIpTransformationTechnologyConfig {
                    alignment: 8,
                    byte_order: ByteOrder::MostSignificantByteFirst,
                    interface_version: 1,
                }),
            )
            .unwrap();
        let data_transformation = dts
            .create_data_transformation("someip_trans", &[&transformer], false)
            .unwrap();

        signal.add_data_transformation(&data_transformation).unwrap();

        assert_eq!(signal.data_transformations().count(), 1);
        assert_eq!(signal.data_transformations().next(), Some(data_transformation));

        let someip_props = signal.create_someip_transformation_isignal_props(&transformer).unwrap();
        someip_props.set_legacy_strings(Some(true)).unwrap();
        someip_props.set_interface_version(Some(1)).unwrap();
        someip_props.set_dynamic_length(Some(true)).unwrap();
        someip_props.set_message_type(Some(SomeIpMessageType::Request)).unwrap();
        someip_props.set_size_of_array_length(Some(8)).unwrap();
        someip_props.set_size_of_string_length(Some(16)).unwrap();
        someip_props.set_size_of_struct_length(Some(32)).unwrap();
        someip_props.set_size_of_union_length(Some(64)).unwrap();

        assert_eq!(signal.transformation_isignal_props().count(), 1);
    }

    #[test]
    fn test_signal_group_data_transformations() {
        let model = AutosarModelAbstraction::create("test.arxml", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/test").unwrap();

        let signal_group = ISignalGroup::new(
            "signal_group",
            &package,
            &SystemSignalGroup::new("sys_signal_group", &package).unwrap(),
        )
        .unwrap();

        let dts = DataTransformationSet::new("data_transformation_set", &package).unwrap();
        let transformer = dts
            .create_transformation_technology(
                "someip_xf",
                &TransformationTechnologyConfig::SomeIp(SomeIpTransformationTechnologyConfig {
                    alignment: 8,
                    byte_order: ByteOrder::MostSignificantByteFirst,
                    interface_version: 1,
                }),
            )
            .unwrap();
        let data_transformation = dts
            .create_data_transformation("someip_trans", &[&transformer], false)
            .unwrap();

        signal_group.add_data_transformation(&data_transformation).unwrap();
        assert_eq!(signal_group.data_transformations().count(), 1);
        assert_eq!(signal_group.data_transformations().next(), Some(data_transformation));

        let _someipxf_props = signal_group
            .create_someip_transformation_isignal_props(&transformer)
            .unwrap();
        // the referenced transformer is not an E2E transformer, so no E2E properties can be created
        let result = signal_group.create_e2e_transformation_isignal_props(&transformer);
        assert!(result.is_err());

        assert_eq!(signal_group.transformation_isignal_props().count(), 1);
    }

    #[test]
    fn test_signal_group() {
        let model = AutosarModelAbstraction::create("test.arxml", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/test").unwrap();
        let sys_signal_group = SystemSignalGroup::new("sys_signal_group", &package).unwrap();
        let signal_group = ISignalGroup::new("signal_group", &package, &sys_signal_group).unwrap();
        assert_eq!(signal_group.system_signal_group(), Some(sys_signal_group.clone()));

        let sys_signal = SystemSignal::new("sys_signal", &package).unwrap();
        let signal = ISignal::new("signal", &package, 8, &sys_signal, None).unwrap();
        assert_eq!(signal.system_signal(), Some(sys_signal.clone()));

        sys_signal_group.add_signal(&sys_signal).unwrap();
        assert_eq!(sys_signal.signal_group(), Some(sys_signal_group.clone()));

        signal_group.add_signal(&signal).unwrap();
        assert_eq!(signal_group.signals().count(), 1);
    }

    #[test]
    fn test_signal_triggering() {
        let model = AutosarModelAbstraction::create("test.arxml", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/test").unwrap();
        let system = package.create_system("system", SystemCategory::EcuExtract).unwrap();

        let sw_base_type =
            SwBaseType::new("sw_base_type", &package, 8, BaseTypeEncoding::None, None, None, None).unwrap();

        let sys_signal = package.create_system_signal("sys_signal").unwrap();
        let signal = system
            .create_isignal("signal", &package, 8, &sys_signal, Some(&sw_base_type))
            .unwrap();

        // signal triggering
        let cluster = system
            .create_can_cluster("cluster", &package, &CanClusterSettings::default())
            .unwrap();
        let channel = cluster.create_physical_channel("channel").unwrap();
        let st = ISignalTriggering::new(&signal, &channel.clone()).unwrap();

        assert_eq!(st.physical_channel().unwrap(), PhysicalChannel::Can(channel.clone()));

        let ecuinstance = system.create_ecu_instance("ecu", &package).unwrap();
        let controller = ecuinstance.create_can_communication_controller("controller").unwrap();
        controller.connect_physical_channel("connection", &channel).unwrap();

        assert_eq!(st.signal_ports().count(), 0);
        let signal_port = st.connect_to_ecu(&ecuinstance, CommunicationDirection::In).unwrap();
        assert_eq!(st.signal_ports().count(), 1);
        assert_eq!(signal_port.ecu().unwrap(), ecuinstance);
        assert_eq!(signal_port.communication_direction(), Some(CommunicationDirection::In));
        signal_port
            .set_communication_direction(CommunicationDirection::Out)
            .unwrap();
        assert_eq!(signal_port.communication_direction(), Some(CommunicationDirection::Out));
        signal_port.set_name("new_name").unwrap();
        assert_eq!(signal_port.name().unwrap(), "new_name");
    }
}
