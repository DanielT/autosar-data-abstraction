use crate::{
    AbstractionElement, AutosarAbstractionError, Element, IdentifiableAbstractionElement, abstraction_element,
    datatype::DataTypeMappingSet,
    software_component::{
        ClientServerOperation, ModeDeclaration, PPortPrototype, PortPrototype, SwComponentType, VariableDataPrototype,
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
    pub fn sw_component_type(&self) -> Option<SwComponentType> {
        let parent = self.element().named_parent().ok()??;
        SwComponentType::try_from(parent).ok()
    }

    /// Create a new RunnableEntity in the SwcInternalBehavior
    pub fn create_runnable_entity(&self, name: &str) -> Result<RunnableEntity, AutosarAbstractionError> {
        let runnalbles_elem = self.element().get_or_create_sub_element(ElementName::Runnables)?;
        RunnableEntity::new(name, &runnalbles_elem)
    }

    /// Get an iterator over all RunnableEntities in the SwcInternalBehavior
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
    pub fn swc_internal_behavior(&self) -> Option<SwcInternalBehavior> {
        let parent = self.element().named_parent().ok()??;
        SwcInternalBehavior::try_from(parent).ok()
    }

    /// Iterate over all events that can trigger the `RunnableEntity`
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
}
