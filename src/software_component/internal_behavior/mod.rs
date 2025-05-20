use crate::{
    AbstractionElement, AutosarAbstractionError, Element, IdentifiableAbstractionElement, abstraction_element,
    datatype::DataTypeMappingSet,
    software_component::{
        ClientServerOperation, ModeDeclaration, PPortPrototype, PortPrototype, RPortPrototype, SwComponentType,
        VariableDataPrototype,
    },
};
use autosar_data::ElementName;

mod rte_event;

pub use rte_event::*;

//##################################################################

/// The `SwcInternalBehavior` of a software component type describes the
/// details that are needed to generate the RTE.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SwcInternalBehavior(Element);
abstraction_element!(SwcInternalBehavior, SwcInternalBehavior);
impl IdentifiableAbstractionElement for SwcInternalBehavior {}

impl SwcInternalBehavior {
    pub(crate) fn new(name: &str, parent: &Element) -> Result<Self, AutosarAbstractionError> {
        let swc_internal_behavior = parent.create_named_sub_element(ElementName::SwcInternalBehavior, name)?;

        Ok(Self(swc_internal_behavior))
    }

    /// Get the software component type that contains the `SwcInternalBehavior`
    #[must_use]
    pub fn sw_component_type(&self) -> Option<SwComponentType> {
        let parent = self.element().named_parent().ok()??;
        SwComponentType::try_from(parent).ok()
    }

    /// Create a new `RunnableEntity` in the `SwcInternalBehavior`
    pub fn create_runnable_entity(&self, name: &str) -> Result<RunnableEntity, AutosarAbstractionError> {
        let runnalbles_elem = self.element().get_or_create_sub_element(ElementName::Runnables)?;
        RunnableEntity::new(name, &runnalbles_elem)
    }

    /// Get an iterator over all `RunnableEntities` in the `SwcInternalBehavior`
    pub fn runnable_entities(&self) -> impl Iterator<Item = RunnableEntity> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::Runnables)
            .into_iter()
            .flat_map(|runnables| runnables.sub_elements())
            .filter_map(|elem| RunnableEntity::try_from(elem).ok())
    }

    /// Add a reference to a `DataTypeMappingSet` to the `SwcInternalBehavior`
    pub fn add_data_type_mapping_set(
        &self,
        data_type_mapping_set: &DataTypeMappingSet,
    ) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::DataTypeMappingRefs)?
            .create_sub_element(ElementName::DataTypeMappingRef)?
            .set_reference_target(data_type_mapping_set.element())?;
        Ok(())
    }

    /// create an iterator over all `DataTypeMappingSet` references in the `SwcInternalBehavior`
    pub fn data_type_mapping_sets(&self) -> impl Iterator<Item = DataTypeMappingSet> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::DataTypeMappingRefs)
            .into_iter()
            .flat_map(|refs| {
                refs.sub_elements()
                    .filter_map(|elem| elem.get_reference_target().ok())
                    .filter_map(|elem| DataTypeMappingSet::try_from(elem).ok())
            })
    }

    /// Create a new `InitEvent` in the `SwcInternalBehavior`
    pub fn create_init_event(
        &self,
        name: &str,
        runnable: &RunnableEntity,
    ) -> Result<InitEvent, AutosarAbstractionError> {
        let events = self.element().get_or_create_sub_element(ElementName::Events)?;
        InitEvent::new(name, &events, runnable)
    }

    /// Create a new `OperationInvokedEvent` in the `SwcInternalBehavior` when a server operation is invoked
    pub fn create_operation_invoked_event(
        &self,
        name: &str,
        runnable: &RunnableEntity,
        client_server_operation: &ClientServerOperation,
        context_p_port: &PPortPrototype,
    ) -> Result<OperationInvokedEvent, AutosarAbstractionError> {
        let events = self.element().get_or_create_sub_element(ElementName::Events)?;
        OperationInvokedEvent::new(name, &events, runnable, client_server_operation, context_p_port)
    }

    /// Create a timing event that triggers a runnable in the `SwcInternalBehavior` based on a timer
    pub fn create_timing_event(
        &self,
        name: &str,
        runnable: &RunnableEntity,
        period: f64,
    ) -> Result<TimingEvent, AutosarAbstractionError> {
        let events = self.element().get_or_create_sub_element(ElementName::Events)?;
        TimingEvent::new(name, &events, runnable, period)
    }

    /// create a background event that triggers a runnable in the `SwcInternalBehavior` for background processing
    pub fn create_background_event(
        &self,
        name: &str,
        runnable: &RunnableEntity,
    ) -> Result<BackgroundEvent, AutosarAbstractionError> {
        let events = self.element().get_or_create_sub_element(ElementName::Events)?;
        BackgroundEvent::new(name, &events, runnable)
    }

    /// create a data received event that triggers a runnable in the `SwcInternalBehavior` when data is received
    pub fn create_data_received_event<T: Into<PortPrototype> + Clone>(
        &self,
        name: &str,
        runnable: &RunnableEntity,
        variable_data_prototype: &VariableDataPrototype,
        context_port: &T,
    ) -> Result<DataReceivedEvent, AutosarAbstractionError> {
        let events = self.element().get_or_create_sub_element(ElementName::Events)?;
        DataReceivedEvent::new(name, &events, runnable, variable_data_prototype, context_port)
    }

    /// create an os task execution event that triggers a runnable in the `SwcInternalBehavior` every time the task is executed
    pub fn create_os_task_execution_event(
        &self,
        name: &str,
        runnable: &RunnableEntity,
    ) -> Result<OsTaskExecutionEvent, AutosarAbstractionError> {
        let events = self.element().get_or_create_sub_element(ElementName::Events)?;
        OsTaskExecutionEvent::new(name, &events, runnable)
    }

    /// create a mode switch event that triggers a runnable in the `SwcInternalBehavior` when the mode is switched
    pub fn create_mode_switch_event<T: Into<PortPrototype> + Clone>(
        &self,
        name: &str,
        runnable: &RunnableEntity,
        activation: ModeActivationKind,
        context_port: &T,
        mode_declaration: &ModeDeclaration,
        second_mode_declaration: Option<&ModeDeclaration>,
    ) -> Result<SwcModeSwitchEvent, AutosarAbstractionError> {
        let events = self.element().get_or_create_sub_element(ElementName::Events)?;
        SwcModeSwitchEvent::new(
            name,
            &events,
            runnable,
            activation,
            context_port,
            mode_declaration,
            second_mode_declaration,
        )
    }

    /// create an iterator over all events in the `SwcInternalBehavior`
    pub fn events(&self) -> impl Iterator<Item = RTEEvent> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::Events)
            .into_iter()
            .flat_map(|events| events.sub_elements())
            .filter_map(|elem| RTEEvent::try_from(elem).ok())
    }
}

//##################################################################

/// A `RunnableEntity` is a function that can be executed by the RTE
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RunnableEntity(Element);
abstraction_element!(RunnableEntity, RunnableEntity);
impl IdentifiableAbstractionElement for RunnableEntity {}

impl RunnableEntity {
    pub(crate) fn new(name: &str, parent: &Element) -> Result<Self, AutosarAbstractionError> {
        let runnable_entity = parent.create_named_sub_element(ElementName::RunnableEntity, name)?;

        Ok(Self(runnable_entity))
    }

    /// Get the `SwcInternalBehavior` that contains the `RunnableEntity`
    #[must_use]
    pub fn swc_internal_behavior(&self) -> Option<SwcInternalBehavior> {
        let parent = self.element().named_parent().ok()??;
        SwcInternalBehavior::try_from(parent).ok()
    }

    /// Iterate over all events that can trigger the `RunnableEntity`
    #[must_use]
    pub fn events(&self) -> Vec<RTEEvent> {
        let model_result = self.element().model();
        let path_result = self.element().path();
        if let (Ok(model), Ok(path)) = (model_result, path_result) {
            model
                .get_references_to(&path)
                .iter()
                .filter_map(|e| {
                    e.upgrade()
                        .and_then(|ref_elem| ref_elem.named_parent().ok().flatten())
                        .and_then(|elem| RTEEvent::try_from(elem).ok())
                })
                .collect()
        } else {
            vec![]
        }
    }

    /// add implicit read access to a data element of a sender-receiver `PortPrototype`
    ///
    /// this results in `Rte_IRead_<port>_<data_element>` being generated
    pub fn create_data_read_access<T: Into<PortPrototype> + Clone>(
        &self,
        name: &str,
        data_element: &VariableDataPrototype,
        context_port: &T,
    ) -> Result<VariableAccess, AutosarAbstractionError> {
        let data_accesses = self.element().get_or_create_sub_element(ElementName::DataReadAccesss)?;
        VariableAccess::new(name, &data_accesses, data_element, &context_port.clone().into())
    }

    /// iterate over all data read accesses
    pub fn data_read_accesses(&self) -> impl Iterator<Item = VariableAccess> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::DataReadAccesss)
            .into_iter()
            .flat_map(|data_accesses| data_accesses.sub_elements())
            .filter_map(|elem| VariableAccess::try_from(elem).ok())
    }

    /// add implicit write access to a data element of a sender-receiver `PortPrototype`
    ///
    /// this results in `Rte_IWrite_<port>_<data_element>` being generated
    pub fn create_data_write_access<T: Into<PortPrototype> + Clone>(
        &self,
        name: &str,
        data_element: &VariableDataPrototype,
        context_port: &T,
    ) -> Result<VariableAccess, AutosarAbstractionError> {
        let data_accesses = self
            .element()
            .get_or_create_sub_element(ElementName::DataWriteAccesss)?;
        VariableAccess::new(name, &data_accesses, data_element, &context_port.clone().into())
    }

    /// iterate over all data write accesses
    pub fn data_write_accesses(&self) -> impl Iterator<Item = VariableAccess> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::DataWriteAccesss)
            .into_iter()
            .flat_map(|data_accesses| data_accesses.sub_elements())
            .filter_map(|elem| VariableAccess::try_from(elem).ok())
    }

    /// add a data send point to a data element of a sender-receiver `PortPrototype`
    pub fn create_data_send_point<T: Into<PortPrototype> + Clone>(
        &self,
        name: &str,
        data_element: &VariableDataPrototype,
        context_port: &T,
    ) -> Result<VariableAccess, AutosarAbstractionError> {
        let data_accesses = self.element().get_or_create_sub_element(ElementName::DataSendPoints)?;
        VariableAccess::new(name, &data_accesses, data_element, &context_port.clone().into())
    }

    /// iterate over all data send points
    pub fn data_send_points(&self) -> impl Iterator<Item = VariableAccess> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::DataSendPoints)
            .into_iter()
            .flat_map(|data_accesses| data_accesses.sub_elements())
            .filter_map(|elem| VariableAccess::try_from(elem).ok())
    }

    /// add explicit read access by argument to a data element of a sender-receiver `PortPrototype`
    ///
    /// this results in `Rte_Read_<port>_<data_element>(DataType* data)` being generated
    pub fn create_data_receive_point_by_argument<T: Into<PortPrototype> + Clone>(
        &self,
        name: &str,
        data_element: &VariableDataPrototype,
        context_port: &T,
    ) -> Result<VariableAccess, AutosarAbstractionError> {
        let data_accesses = self
            .element()
            .get_or_create_sub_element(ElementName::DataReceivePointByArguments)?;
        VariableAccess::new(name, &data_accesses, data_element, &context_port.clone().into())
    }

    /// iterate over all data receive points by argument
    pub fn data_receive_points_by_argument(&self) -> impl Iterator<Item = VariableAccess> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::DataReceivePointByArguments)
            .into_iter()
            .flat_map(|data_accesses| data_accesses.sub_elements())
            .filter_map(|elem| VariableAccess::try_from(elem).ok())
    }

    /// add explicit read access by value to a data element of a sender-receiver `PortPrototype`
    pub fn create_data_receive_point_by_value<T: Into<PortPrototype> + Clone>(
        &self,
        name: &str,
        data_element: &VariableDataPrototype,
        context_port: &T,
    ) -> Result<VariableAccess, AutosarAbstractionError> {
        let data_accesses = self
            .element()
            .get_or_create_sub_element(ElementName::DataReceivePointByValues)?;
        VariableAccess::new(name, &data_accesses, data_element, &context_port.clone().into())
    }

    /// iterate over all data receive points by value
    pub fn data_receive_points_by_value(&self) -> impl Iterator<Item = VariableAccess> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::DataReceivePointByValues)
            .into_iter()
            .flat_map(|data_accesses| data_accesses.sub_elements())
            .filter_map(|elem| VariableAccess::try_from(elem).ok())
    }

    /// create a synchronous server call point that allows the runnable to call a server operation
    pub fn create_synchronous_server_call_point(
        &self,
        name: &str,
        client_server_operation: &ClientServerOperation,
        context_r_port: &RPortPrototype,
    ) -> Result<SynchronousServerCallPoint, AutosarAbstractionError> {
        let server_call_points = self
            .element()
            .get_or_create_sub_element(ElementName::ServerCallPoints)?;
        SynchronousServerCallPoint::new(name, &server_call_points, client_server_operation, context_r_port)
    }

    /// iterate over all synchronous server call points
    pub fn synchronous_server_call_points(&self) -> impl Iterator<Item = SynchronousServerCallPoint> + Send + 'static {
        self.element()
            .get_sub_element(ElementName::ServerCallPoints)
            .into_iter()
            .flat_map(|server_call_points| server_call_points.sub_elements())
            .filter_map(|elem| SynchronousServerCallPoint::try_from(elem).ok())
    }
}

//##################################################################

/// A `VariableAccess` allows a `RunnableEntity` to access a variable in various contexts
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariableAccess(Element);
abstraction_element!(VariableAccess, VariableAccess);
impl IdentifiableAbstractionElement for VariableAccess {}

impl VariableAccess {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        data_element: &VariableDataPrototype,
        context_port: &PortPrototype,
    ) -> Result<Self, AutosarAbstractionError> {
        let variable_access = parent.create_named_sub_element(ElementName::VariableAccess, name)?;
        let variable_access = Self(variable_access);
        variable_access.set_accessed_variable(data_element, context_port)?;

        Ok(variable_access)
    }

    /// Set the accessed variable
    pub fn set_accessed_variable(
        &self,
        data_element: &VariableDataPrototype,
        context_port: &PortPrototype,
    ) -> Result<(), AutosarAbstractionError> {
        // remove the old accessed variable
        let _ = self.element().remove_sub_element_kind(ElementName::AccessedVariable);
        let accessed_variable = self.element().create_sub_element(ElementName::AccessedVariable)?;
        let autosar_variable_iref = accessed_variable.create_sub_element(ElementName::AutosarVariableIref)?;

        autosar_variable_iref
            .create_sub_element(ElementName::TargetDataPrototypeRef)?
            .set_reference_target(data_element.element())?;
        autosar_variable_iref
            .create_sub_element(ElementName::PortPrototypeRef)?
            .set_reference_target(context_port.element())?;
        Ok(())
    }

    /// Get the accessed variable
    #[must_use]
    pub fn accessed_variable(&self) -> Option<(VariableDataPrototype, PortPrototype)> {
        let accessed_variable = self.element().get_sub_element(ElementName::AccessedVariable)?;
        let autosar_variable_iref = accessed_variable.get_sub_element(ElementName::AutosarVariableIref)?;
        let data_prototype_ref = autosar_variable_iref.get_sub_element(ElementName::TargetDataPrototypeRef)?;
        let port_prototype_ref = autosar_variable_iref.get_sub_element(ElementName::PortPrototypeRef)?;

        let data_prototype = VariableDataPrototype::try_from(data_prototype_ref.get_reference_target().ok()?).ok()?;
        let port_prototype = PortPrototype::try_from(port_prototype_ref.get_reference_target().ok()?).ok()?;

        Some((data_prototype, port_prototype))
    }

    /// Get the `RunnableEntity` that contains the `VariableAccess`
    #[must_use]
    pub fn runnable_entity(&self) -> Option<RunnableEntity> {
        let parent = self.element().named_parent().ok()??;
        RunnableEntity::try_from(parent).ok()
    }
}

//##################################################################

/// A `SynchronousServerCallPoint` allows a `RunnableEntity` to call a server operation synchronously
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SynchronousServerCallPoint(Element);
abstraction_element!(SynchronousServerCallPoint, SynchronousServerCallPoint);
impl IdentifiableAbstractionElement for SynchronousServerCallPoint {}

impl SynchronousServerCallPoint {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        client_server_operation: &ClientServerOperation,
        context_r_port: &RPortPrototype,
    ) -> Result<Self, AutosarAbstractionError> {
        let synchronous_server_call_point =
            parent.create_named_sub_element(ElementName::SynchronousServerCallPoint, name)?;
        let synchronous_server_call_point = Self(synchronous_server_call_point);
        synchronous_server_call_point.set_client_server_operation(client_server_operation, context_r_port)?;

        Ok(synchronous_server_call_point)
    }

    /// Set the client server operation
    pub fn set_client_server_operation(
        &self,
        client_server_operation: &ClientServerOperation,
        context_r_port: &RPortPrototype,
    ) -> Result<(), AutosarAbstractionError> {
        // remove the old client server operation
        let _ = self.element().remove_sub_element_kind(ElementName::OperationIref);
        let operation_iref = self.element().create_sub_element(ElementName::OperationIref)?;

        operation_iref
            .create_sub_element(ElementName::TargetRequiredOperationRef)?
            .set_reference_target(client_server_operation.element())?;
        operation_iref
            .create_sub_element(ElementName::ContextRPortRef)?
            .set_reference_target(context_r_port.element())?;
        Ok(())
    }

    /// Get the client server operation
    #[must_use]
    pub fn client_server_operation(&self) -> Option<(ClientServerOperation, RPortPrototype)> {
        let operation_iref = self.element().get_sub_element(ElementName::OperationIref)?;
        let required_operation_ref = operation_iref.get_sub_element(ElementName::TargetRequiredOperationRef)?;
        let context_r_port_ref = operation_iref.get_sub_element(ElementName::ContextRPortRef)?;

        let client_server_operation =
            ClientServerOperation::try_from(required_operation_ref.get_reference_target().ok()?).ok()?;
        let context_r_port = RPortPrototype::try_from(context_r_port_ref.get_reference_target().ok()?).ok()?;

        Some((client_server_operation, context_r_port))
    }

    /// Get the `RunnableEntity` that contains the `SynchronousServerCallPoint`
    #[must_use]
    pub fn runnable_entity(&self) -> Option<RunnableEntity> {
        let parent = self.element().named_parent().ok()??;
        RunnableEntity::try_from(parent).ok()
    }
}

//##################################################################

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        AbstractionElement, AutosarModelAbstraction,
        datatype::ApplicationPrimitiveCategory,
        software_component::{AbstractRTEEvent, AbstractSwComponentType, AtomicSwComponentType},
    };
    use autosar_data::{AutosarVersion, EnumItem};

    #[test]
    fn swc_internal_behavior() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        // create a client-server interface
        let client_server_interface = package.create_client_server_interface("ClientServerInterface").unwrap();
        let operation = client_server_interface.create_operation("TestOperation").unwrap();

        // create a software component type with an internal behavior
        let app_swc = package
            .create_application_sw_component_type("AppSwComponentType")
            .unwrap();
        let p_port = app_swc.create_p_port("p_port", &client_server_interface).unwrap();
        let swc_internal_behavior = app_swc
            .create_swc_internal_behavior("AppSwComponentType_InternalBehavior")
            .unwrap();
        assert_eq!(app_swc.swc_internal_behaviors().count(), 1);
        assert_eq!(swc_internal_behavior.sw_component_type().unwrap(), app_swc.into());

        // create two runnable entities
        let runnable1 = swc_internal_behavior.create_runnable_entity("Runnable1").unwrap();
        assert_eq!(runnable1.swc_internal_behavior().unwrap(), swc_internal_behavior);
        let runnable2 = swc_internal_behavior.create_runnable_entity("Runnable2").unwrap();
        assert_eq!(swc_internal_behavior.runnable_entities().count(), 2);

        // create an init event, which triggers runnable1
        let init_event = swc_internal_behavior
            .create_init_event("InitEvent", &runnable1)
            .unwrap();
        assert_eq!(init_event.runnable_entity().unwrap(), runnable1);

        // create an operation invoked event, which triggers runnable1
        let op_invoked_event = swc_internal_behavior
            .create_operation_invoked_event("OpInvokedEvent", &runnable1, &operation, &p_port)
            .unwrap();
        let (op_invoked_event_operation, context_p_port) = op_invoked_event.client_server_operation().unwrap();
        assert_eq!(op_invoked_event_operation, operation);
        assert_eq!(context_p_port, p_port);
        assert_eq!(op_invoked_event.runnable_entity().unwrap(), runnable1);

        // create a background event, which triggers runnable1
        let background_event = swc_internal_behavior
            .create_background_event("BackgroundEvent", &runnable1)
            .unwrap();
        assert_eq!(background_event.runnable_entity().unwrap(), runnable1);

        // create an os task execution event, which triggers runnable1
        let os_task_execution_event = swc_internal_behavior
            .create_os_task_execution_event("OsTaskExecutionEvent", &runnable1)
            .unwrap();
        assert_eq!(os_task_execution_event.runnable_entity().unwrap(), runnable1);

        // create a timing event, which triggers runnable2
        let timing_event = swc_internal_behavior
            .create_timing_event("TimingEvent", &runnable2, 0.1)
            .unwrap();
        assert_eq!(timing_event.period().unwrap(), 0.1);
        assert_eq!(timing_event.runnable_entity().unwrap(), runnable2);
        assert_eq!(timing_event.swc_internal_behavior().unwrap(), swc_internal_behavior);

        // there should be 3 events in the swc_internal_behavior
        assert_eq!(swc_internal_behavior.events().count(), 5);
        // iterate over all events and check if they are the same as the ones we created
        let mut events_iter = swc_internal_behavior.events();
        assert_eq!(events_iter.next().unwrap().element(), init_event.element());
        assert_eq!(events_iter.next().unwrap().element(), op_invoked_event.element());
        assert_eq!(events_iter.next().unwrap().element(), background_event.element());
        assert_eq!(events_iter.next().unwrap().element(), os_task_execution_event.element());
        assert_eq!(events_iter.next().unwrap().element(), timing_event.element());

        // runnable1 should be triggered by 4 events
        assert_eq!(runnable1.events().len(), 4);
        // runnable2 should be triggered by 1 event
        assert_eq!(runnable2.events().len(), 1);

        // add a data type mapping set to the swc_internal_behavior
        let data_type_mapping_set = package.create_data_type_mapping_set("MappingSet").unwrap();
        swc_internal_behavior
            .add_data_type_mapping_set(&data_type_mapping_set)
            .unwrap();
        assert_eq!(swc_internal_behavior.data_type_mapping_sets().count(), 1);
    }

    #[test]
    fn mode_switch_event() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        // create a software component type with an internal behavior
        let app_swc = package
            .create_application_sw_component_type("AppSwComponentType")
            .unwrap();
        let swc_internal_behavior = app_swc
            .create_swc_internal_behavior("AppSwComponentType_InternalBehavior")
            .unwrap();
        assert_eq!(app_swc.swc_internal_behaviors().count(), 1);
        assert_eq!(
            swc_internal_behavior.sw_component_type().unwrap(),
            app_swc.clone().into()
        );

        // create a mode declaration group and a mode switch interface
        let mode_declaration_group = package
            .create_mode_declaration_group("ModeDeclarationGroup", None)
            .unwrap();
        let mode_declaration_1 = mode_declaration_group
            .create_mode_declaration("ModeDeclaration1")
            .unwrap();
        let mode_declaration_2 = mode_declaration_group
            .create_mode_declaration("ModeDeclaration2")
            .unwrap();
        let mode_switch_interface = package.create_mode_switch_interface("ModeSwitchInterface").unwrap();
        let r_port = app_swc.create_r_port("r_port", &mode_switch_interface).unwrap();

        let mode_declaration_group2 = package
            .create_mode_declaration_group("ModeDeclarationGroup2", None)
            .unwrap();
        let mode_declaration_g2 = mode_declaration_group2
            .create_mode_declaration("ModeDeclaratio_g2")
            .unwrap();

        //  create a second port for the error path test which does not have a mode switch interface
        let client_server_interface = package.create_client_server_interface("ClientServerInterface").unwrap();
        let bad_port = app_swc.create_r_port("bad_port", &client_server_interface).unwrap();

        // create a runnable entity
        let runnable = swc_internal_behavior.create_runnable_entity("Runnable1").unwrap();
        assert_eq!(runnable.swc_internal_behavior().unwrap(), swc_internal_behavior);

        // error case: create a mode switch event with a port that does not have a mode switch interface
        let result = swc_internal_behavior.create_mode_switch_event(
            "ModeSwitchEvent",
            &runnable,
            ModeActivationKind::OnEntry,
            &bad_port,
            &mode_declaration_g2,
            None,
        );
        assert!(result.is_err());

        // error case: the mode switch interface does not contain a mode group
        let result = swc_internal_behavior.create_mode_switch_event(
            "ModeSwitchEvent",
            &runnable,
            ModeActivationKind::OnEntry,
            &r_port,
            &mode_declaration_1,
            Some(&mode_declaration_2),
        );
        assert!(result.is_err());

        // create the mode group in the mode switch interface
        mode_switch_interface
            .create_mode_group("mode_group", &mode_declaration_group)
            .unwrap();

        // error case: create a mode switch event with a mode_declaration that is not part of the mode declaration group
        let result = swc_internal_behavior.create_mode_switch_event(
            "ModeSwitchEvent",
            &runnable,
            ModeActivationKind::OnEntry,
            &r_port,
            &mode_declaration_g2,
            None,
        );
        assert!(result.is_err());

        // correct: create a mode switch event with a mode_declaration that is part of the mode declaration group
        let mode_switch_event = swc_internal_behavior
            .create_mode_switch_event(
                "ModeSwitchEvent",
                &runnable,
                ModeActivationKind::OnEntry,
                &r_port,
                &mode_declaration_1,
                Some(&mode_declaration_2),
            )
            .unwrap();
        assert_eq!(mode_switch_event.runnable_entity().unwrap(), runnable);

        assert_eq!(runnable.events().len(), 1);

        let (mode_decls, context_port) = mode_switch_event.mode_declarations().unwrap();
        assert_eq!(context_port, r_port.into());
        assert_eq!(mode_decls.len(), 2);
        assert_eq!(mode_decls[0], mode_declaration_1);
        assert_eq!(mode_decls[1], mode_declaration_2);

        // check the mode activation kind
        mode_switch_event
            .set_mode_activation_kind(ModeActivationKind::OnEntry)
            .unwrap();
        assert_eq!(
            mode_switch_event.mode_activation_kind().unwrap(),
            ModeActivationKind::OnEntry
        );
        mode_switch_event
            .set_mode_activation_kind(ModeActivationKind::OnExit)
            .unwrap();
        assert_eq!(
            mode_switch_event.mode_activation_kind().unwrap(),
            ModeActivationKind::OnExit
        );
        mode_switch_event
            .set_mode_activation_kind(ModeActivationKind::OnTransition)
            .unwrap();
        assert_eq!(
            mode_switch_event.mode_activation_kind().unwrap(),
            ModeActivationKind::OnTransition
        );

        // mode activation kind error case
        let activation_kind = ModeActivationKind::try_from(EnumItem::Opaque);
        assert!(activation_kind.is_err());
    }

    #[test]
    fn data_received_event() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        // create a sender receiver interface with a variable
        let sender_receiver_interface = package
            .create_sender_receiver_interface("SenderReceiverInterface")
            .unwrap();
        let app_data_type = package
            .create_application_primitive_data_type("uint32", ApplicationPrimitiveCategory::Value, None, None, None)
            .unwrap();
        let variable_data_prototype = sender_receiver_interface
            .create_data_element("data", &app_data_type)
            .unwrap();

        // create a software component type with an internal behavior
        let app_swc = package
            .create_application_sw_component_type("AppSwComponentType")
            .unwrap();
        let r_port = app_swc.create_r_port("r_port", &sender_receiver_interface).unwrap();
        let swc_internal_behavior = app_swc
            .create_swc_internal_behavior("AppSwComponentType_InternalBehavior")
            .unwrap();
        assert_eq!(app_swc.swc_internal_behaviors().count(), 1);
        assert_eq!(
            swc_internal_behavior.sw_component_type().unwrap(),
            app_swc.clone().into()
        );

        // create a p-port using the sender-receiver interface
        let p_port = app_swc.create_p_port("p_port", &sender_receiver_interface).unwrap();

        // create a port using a client-server interface
        let client_server_interface = package.create_client_server_interface("ClientServerInterface").unwrap();
        let cs_port = app_swc.create_r_port("cs_port", &client_server_interface).unwrap();

        // create a runnable entity
        let runnable = swc_internal_behavior.create_runnable_entity("Runnable1").unwrap();
        assert_eq!(runnable.swc_internal_behavior().unwrap(), swc_internal_behavior);

        // error case: create a data received event with a port that does not have a sender-receiver interface
        let result = swc_internal_behavior.create_data_received_event(
            "DataReceivedEvent",
            &runnable,
            &variable_data_prototype,
            &cs_port,
        );
        assert!(result.is_err());

        // error case: can't create a data received event with a p-port
        let result = swc_internal_behavior.create_data_received_event(
            "DataReceivedEvent",
            &runnable,
            &variable_data_prototype,
            &p_port,
        );
        assert!(result.is_err());

        // create a data received event, which triggers runnable
        let data_received_event = swc_internal_behavior
            .create_data_received_event("DataReceivedEvent", &runnable, &variable_data_prototype, &r_port)
            .unwrap();
        assert_eq!(data_received_event.runnable_entity().unwrap(), runnable);

        let (data_element, context_port) = data_received_event.variable_data_prototype().unwrap();
        assert_eq!(data_element, variable_data_prototype);
        assert_eq!(context_port, r_port.into());
        assert_eq!(data_received_event.runnable_entity().unwrap(), runnable);
    }

    #[test]
    fn variable_access() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        // create a sender receiver interface with a variable
        let sender_receiver_interface = package
            .create_sender_receiver_interface("SenderReceiverInterface")
            .unwrap();
        let app_data_type = package
            .create_application_primitive_data_type("uint32", ApplicationPrimitiveCategory::Value, None, None, None)
            .unwrap();
        let variable_data_prototype = sender_receiver_interface
            .create_data_element("data", &app_data_type)
            .unwrap();

        // create a software component type with an internal behavior
        let app_swc = package
            .create_application_sw_component_type("AppSwComponentType")
            .unwrap();
        let r_port = app_swc.create_r_port("r_port", &sender_receiver_interface).unwrap();
        let p_port = app_swc.create_p_port("p_port", &sender_receiver_interface).unwrap();
        let swc_internal_behavior = app_swc
            .create_swc_internal_behavior("AppSwComponentType_InternalBehavior")
            .unwrap();
        assert_eq!(app_swc.swc_internal_behaviors().count(), 1);
        assert_eq!(
            swc_internal_behavior.sw_component_type().unwrap(),
            app_swc.clone().into()
        );

        // create a runnable entity
        let runnable = swc_internal_behavior.create_runnable_entity("Runnable").unwrap();
        assert_eq!(runnable.swc_internal_behavior().unwrap(), swc_internal_behavior);

        // create a variable access for read access
        let variable_access = runnable
            .create_data_read_access("DataReadAccess", &variable_data_prototype, &r_port)
            .unwrap();
        assert_eq!(variable_access.runnable_entity().unwrap(), runnable);
        assert_eq!(variable_access.accessed_variable().unwrap().0, variable_data_prototype);
        assert_eq!(runnable.data_read_accesses().count(), 1);

        // create a variable access for write access
        let variable_access = runnable
            .create_data_write_access("DataWriteAccess", &variable_data_prototype, &p_port)
            .unwrap();
        assert_eq!(variable_access.runnable_entity().unwrap(), runnable);
        assert_eq!(variable_access.accessed_variable().unwrap().0, variable_data_prototype);
        assert_eq!(runnable.data_write_accesses().count(), 1);

        // create a variable access for send point
        let variable_access = runnable
            .create_data_send_point("DataSendPoint", &variable_data_prototype, &p_port)
            .unwrap();
        assert_eq!(variable_access.runnable_entity().unwrap(), runnable);
        assert_eq!(variable_access.accessed_variable().unwrap().0, variable_data_prototype);
        assert_eq!(runnable.data_send_points().count(), 1);

        // create a variable access for receive point by argument
        let variable_access = runnable
            .create_data_receive_point_by_argument("DataReceivePointByArgument", &variable_data_prototype, &r_port)
            .unwrap();
        assert_eq!(variable_access.runnable_entity().unwrap(), runnable);
        assert_eq!(variable_access.accessed_variable().unwrap().0, variable_data_prototype);
        assert_eq!(runnable.data_receive_points_by_argument().count(), 1);

        // create a variable access for receive point by value
        let variable_access = runnable
            .create_data_receive_point_by_value("DataReceivePointByValue", &variable_data_prototype, &r_port)
            .unwrap();
        assert_eq!(variable_access.runnable_entity().unwrap(), runnable);
        assert_eq!(variable_access.accessed_variable().unwrap().0, variable_data_prototype);
        assert_eq!(runnable.data_receive_points_by_value().count(), 1);
    }

    #[test]
    fn synchronous_server_call_point() {
        let model = AutosarModelAbstraction::create("filename", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/package").unwrap();

        // create a client-server interface
        let client_server_interface = package.create_client_server_interface("ClientServerInterface").unwrap();
        let operation = client_server_interface.create_operation("TestOperation").unwrap();

        // create a software component type with an internal behavior
        let app_swc = package
            .create_application_sw_component_type("AppSwComponentType")
            .unwrap();
        let r_port = app_swc.create_r_port("r_port", &client_server_interface).unwrap();
        let swc_internal_behavior = app_swc
            .create_swc_internal_behavior("AppSwComponentType_InternalBehavior")
            .unwrap();
        assert_eq!(app_swc.swc_internal_behaviors().count(), 1);
        assert_eq!(
            swc_internal_behavior.sw_component_type().unwrap(),
            app_swc.clone().into()
        );

        // create a runnable entity
        let runnable = swc_internal_behavior.create_runnable_entity("Runnable1").unwrap();
        assert_eq!(runnable.swc_internal_behavior().unwrap(), swc_internal_behavior);

        // create a synchronous server call point
        let synchronous_server_call_point = runnable
            .create_synchronous_server_call_point("SynchronousServerCallPoint", &operation, &r_port)
            .unwrap();
        assert_eq!(synchronous_server_call_point.runnable_entity().unwrap(), runnable);
        assert_eq!(
            synchronous_server_call_point.client_server_operation().unwrap().0,
            operation
        );
        assert_eq!(runnable.synchronous_server_call_points().count(), 1);
    }
}
