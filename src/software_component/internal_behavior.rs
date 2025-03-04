use crate::{
    AbstractionElement, AutosarAbstractionError, Element, IdentifiableAbstractionElement, abstraction_element,
    datatype::DataTypeMappingSet,
};
use autosar_data::ElementName;

use super::{ClientServerOperation, PPortPrototype, SwComponentType};

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

    /// Create a new `OperationInvokedEvent` in the `SwcInternalBehavior`
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

    /// Create a timing event that triggers a runnable in the `SwcInternalBehavior`
    pub fn create_timing_event(
        &self,
        name: &str,
        runnable: &RunnableEntity,
        period: f64,
    ) -> Result<TimingEvent, AutosarAbstractionError> {
        let timing_events = self.element().get_or_create_sub_element(ElementName::Events)?;
        TimingEvent::new(name, &timing_events, runnable, period)
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

/// A `AbstractRTEEvent` is an event that triggers a `RunnableEntity` in the RTE
///
/// All different kinds of triggering event share the common trait `AbstractRTEEvent`
pub trait AbstractRTEEvent: AbstractionElement {
    /// Set the `RunnableEntity` that is triggered by the `TimingEvent`
    fn set_runnable_entity(&self, runnable_entity: &RunnableEntity) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::StartOnEventRef)?
            .set_reference_target(runnable_entity.element())?;
        Ok(())
    }

    /// Get the `RunnableEntity` that is triggered by the `TimingEvent`
    fn runnable_entity(&self) -> Option<RunnableEntity> {
        let runnable_elem = self
            .element()
            .get_sub_element(ElementName::StartOnEventRef)?
            .get_reference_target()
            .ok()?;
        RunnableEntity::try_from(runnable_elem).ok()
    }

    /// Get the `SwcInternalBehavior` that contains the event
    fn swc_internal_behavior(&self) -> Option<SwcInternalBehavior> {
        let parent = self.element().named_parent().ok()??;
        SwcInternalBehavior::try_from(parent).ok()
    }
}

//##################################################################

/// A `TimingEvent` is a subclass of `RTEEvent` which triggers a `RunnableEntity` periodically
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TimingEvent(Element);
abstraction_element!(TimingEvent, TimingEvent);
impl IdentifiableAbstractionElement for TimingEvent {}
impl AbstractRTEEvent for TimingEvent {}

impl TimingEvent {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        runnable: &RunnableEntity,
        period: f64,
    ) -> Result<Self, AutosarAbstractionError> {
        let timing_event = parent.create_named_sub_element(ElementName::TimingEvent, name)?;
        let timing_event = Self(timing_event);
        timing_event.set_runnable_entity(runnable)?;
        timing_event.set_period(period)?;

        Ok(timing_event)
    }

    /// Set the period of the `TimingEvent`
    pub fn set_period(&self, period: f64) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::Period)?
            .set_character_data(period)?;
        Ok(())
    }

    /// Get the period of the `TimingEvent`
    pub fn period(&self) -> Option<f64> {
        self.element()
            .get_sub_element(ElementName::Period)?
            .character_data()?
            .parse_float()
    }
}

//##################################################################

/// an asynchronous server call completed
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AsynchronousServerCallReturnsEvent(Element);
abstraction_element!(AsynchronousServerCallReturnsEvent, AsynchronousServerCallReturnsEvent);
impl IdentifiableAbstractionElement for AsynchronousServerCallReturnsEvent {}
impl AbstractRTEEvent for AsynchronousServerCallReturnsEvent {}

//##################################################################

/// starts a runnable for background processing at low priority
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BackgroundEvent(Element);
abstraction_element!(BackgroundEvent, BackgroundEvent);
impl IdentifiableAbstractionElement for BackgroundEvent {}
impl AbstractRTEEvent for BackgroundEvent {}

//##################################################################

/// raised in response to an error during data reception
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataReceiveErrorEvent(Element);
abstraction_element!(DataReceiveErrorEvent, DataReceiveErrorEvent);
impl IdentifiableAbstractionElement for DataReceiveErrorEvent {}
impl AbstractRTEEvent for DataReceiveErrorEvent {}

//##################################################################

/// raised when data is received
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataReceivedEvent(Element);
abstraction_element!(DataReceivedEvent, DataReceivedEvent);
impl IdentifiableAbstractionElement for DataReceivedEvent {}
impl AbstractRTEEvent for DataReceivedEvent {}

//##################################################################

/// raised when data has been sent
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataSendCompletedEvent(Element);
abstraction_element!(DataSendCompletedEvent, DataSendCompletedEvent);
impl IdentifiableAbstractionElement for DataSendCompletedEvent {}
impl AbstractRTEEvent for DataSendCompletedEvent {}

//##################################################################

/// raised when an implicit write access was successful or an error occurred
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataWriteCompletedEvent(Element);
abstraction_element!(DataWriteCompletedEvent, DataWriteCompletedEvent);
impl IdentifiableAbstractionElement for DataWriteCompletedEvent {}
impl AbstractRTEEvent for DataWriteCompletedEvent {}

//##################################################################

/// raised when the referenced trigger occurred
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalTriggerOccurredEvent(Element);
abstraction_element!(ExternalTriggerOccurredEvent, ExternalTriggerOccurredEvent);
impl IdentifiableAbstractionElement for ExternalTriggerOccurredEvent {}
impl AbstractRTEEvent for ExternalTriggerOccurredEvent {}

//##################################################################

/// triggered once after the RTE has been started
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InitEvent(Element);
abstraction_element!(InitEvent, InitEvent);
impl IdentifiableAbstractionElement for InitEvent {}
impl AbstractRTEEvent for InitEvent {}

impl InitEvent {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        runnable: &RunnableEntity,
    ) -> Result<Self, AutosarAbstractionError> {
        let init_event = parent.create_named_sub_element(ElementName::InitEvent, name)?;
        let init_event = Self(init_event);
        init_event.set_runnable_entity(runnable)?;

        Ok(init_event)
    }
}

//##################################################################

/// The referenced InternalTriggeringPoint raises this InternalTriggerOccurredEvent
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InternalTriggerOccurredEvent(Element);
abstraction_element!(InternalTriggerOccurredEvent, InternalTriggerOccurredEvent);
impl IdentifiableAbstractionElement for InternalTriggerOccurredEvent {}
impl AbstractRTEEvent for InternalTriggerOccurredEvent {}

//##################################################################

/// raised when the referenced ModeSwitchPoint has been acknowledged
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModeSwitchedAckEvent(Element);
abstraction_element!(ModeSwitchedAckEvent, ModeSwitchedAckEvent);
impl IdentifiableAbstractionElement for ModeSwitchedAckEvent {}
impl AbstractRTEEvent for ModeSwitchedAckEvent {}

//##################################################################

/// raised in order to run the server runnable of a ClientServerOperation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OperationInvokedEvent(Element);
abstraction_element!(OperationInvokedEvent, OperationInvokedEvent);
impl IdentifiableAbstractionElement for OperationInvokedEvent {}
impl AbstractRTEEvent for OperationInvokedEvent {}

impl OperationInvokedEvent {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        runnable: &RunnableEntity,
        client_server_operation: &ClientServerOperation,
        context_p_port: &PPortPrototype,
    ) -> Result<Self, AutosarAbstractionError> {
        let operation_invoked_event = parent.create_named_sub_element(ElementName::OperationInvokedEvent, name)?;
        let operation_invoked_event = Self(operation_invoked_event);
        operation_invoked_event.set_runnable_entity(runnable)?;
        operation_invoked_event.set_client_server_operation(client_server_operation, context_p_port)?;

        Ok(operation_invoked_event)
    }

    /// Set the `ClientServerOperation` that is triggers the `OperationInvokedEvent`
    pub fn set_client_server_operation(
        &self,
        client_server_operation: &ClientServerOperation,
        context_p_port: &PPortPrototype,
    ) -> Result<(), AutosarAbstractionError> {
        // Todo: verify that the port belongs to the containing swc

        let op_iref = self.element().get_or_create_sub_element(ElementName::OperationIref)?;
        op_iref
            .create_sub_element(ElementName::TargetProvidedOperationRef)?
            .set_reference_target(client_server_operation.element())?;
        op_iref
            .create_sub_element(ElementName::ContextPPortRef)?
            .set_reference_target(context_p_port.element())?;
        Ok(())
    }

    /// Get the `ClientServerOperation` that triggers the `OperationInvokedEvent`
    pub fn client_server_operation(&self) -> Option<(ClientServerOperation, PPortPrototype)> {
        let op_iref = self.element().get_sub_element(ElementName::OperationIref)?;
        let operation_elem = op_iref
            .get_sub_element(ElementName::TargetProvidedOperationRef)?
            .get_reference_target()
            .ok()?;
        let context_p_port_elem = op_iref
            .get_sub_element(ElementName::ContextPPortRef)?
            .get_reference_target()
            .ok()?;
        let client_server_operation = ClientServerOperation::try_from(operation_elem).ok()?;
        let context_p_port = PPortPrototype::try_from(context_p_port_elem).ok()?;
        Some((client_server_operation, context_p_port))
    }
}

//##################################################################

/// this event is unconditionally raised whenever the OsTask on which it is mapped is executed
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OsTaskExecutionEvent(Element);
abstraction_element!(OsTaskExecutionEvent, OsTaskExecutionEvent);
impl IdentifiableAbstractionElement for OsTaskExecutionEvent {}
impl AbstractRTEEvent for OsTaskExecutionEvent {}

//##################################################################

/// raised when an error occurred during the handling of the referenced ModeDeclarationGroup
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SwcModeManagerErrorEvent(Element);
abstraction_element!(SwcModeManagerErrorEvent, SwcModeManagerErrorEvent);
impl IdentifiableAbstractionElement for SwcModeManagerErrorEvent {}
impl AbstractRTEEvent for SwcModeManagerErrorEvent {}

//##################################################################

/// raised when the specified mode change occurs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SwcModeSwitchEvent(Element);
abstraction_element!(SwcModeSwitchEvent, SwcModeSwitchEvent);
impl IdentifiableAbstractionElement for SwcModeSwitchEvent {}
impl AbstractRTEEvent for SwcModeSwitchEvent {}

//##################################################################

/// raised if a hard transformer error occurs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransformerHardErrorEvent(Element);
abstraction_element!(TransformerHardErrorEvent, TransformerHardErrorEvent);
impl IdentifiableAbstractionElement for TransformerHardErrorEvent {}
impl AbstractRTEEvent for TransformerHardErrorEvent {}

//##################################################################

/// All events that can trigger a `RunnableEntity` in the RTE
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RTEEvent {
    /// raised when an asynchronous server call completed
    AsynchronousServerCallReturnsEvent(AsynchronousServerCallReturnsEvent),
    /// starts a runnable for background processing at low priority
    BackgroundEvent(BackgroundEvent),
    /// raised in response to an error during data reception
    DataReceiveErrorEvent(DataReceiveErrorEvent),
    /// raised when data is received
    DataReceivedEvent(DataReceivedEvent),
    /// raised when data has been sent
    DataSendCompletedEvent(DataSendCompletedEvent),
    /// raised when an implicit write access was successful or an error occurred
    DataWriteCompletedEvent(DataWriteCompletedEvent),
    /// raised when the referenced trigger occurred
    ExternalTriggerOccurredEvent(ExternalTriggerOccurredEvent),
    /// triggered once after the RTE has been started
    InitEvent(InitEvent),
    /// The referenced InternalTriggeringPoint raises this InternalTriggerOccurredEvent
    InternalTriggerOccurredEvent(InternalTriggerOccurredEvent),
    /// raised when the referenced ModeSwitchPoint has been acknowledged
    ModeSwitchedAckEvent(ModeSwitchedAckEvent),
    /// raised in order to run the server runnable of a ClientServerOperation
    OperationInvokedEvent(OperationInvokedEvent),
    /// this event is unconditionally raised whenever the OsTask on which it is mapped is executed
    OsTaskExecutionEvent(OsTaskExecutionEvent),
    /// raised when an error occurred during the handling of the referenced ModeDeclarationGroup
    SwcModeManagerErrorEvent(SwcModeManagerErrorEvent),
    /// raised when the specified mode change occurs
    SwcModeSwitchEvent(SwcModeSwitchEvent),
    /// raised if a hard transformer error occurs
    TimingEvent(TimingEvent),
    /// raised when an error occurred during the handling of the referenced ModeDeclarationGroup
    TransformerHardErrorEvent(TransformerHardErrorEvent),
}

impl AbstractionElement for RTEEvent {
    fn element(&self) -> &Element {
        match self {
            RTEEvent::AsynchronousServerCallReturnsEvent(elem) => elem.element(),
            RTEEvent::BackgroundEvent(elem) => elem.element(),
            RTEEvent::DataReceiveErrorEvent(elem) => elem.element(),
            RTEEvent::DataReceivedEvent(elem) => elem.element(),
            RTEEvent::DataSendCompletedEvent(elem) => elem.element(),
            RTEEvent::DataWriteCompletedEvent(elem) => elem.element(),
            RTEEvent::ExternalTriggerOccurredEvent(elem) => elem.element(),
            RTEEvent::InitEvent(elem) => elem.element(),
            RTEEvent::InternalTriggerOccurredEvent(elem) => elem.element(),
            RTEEvent::ModeSwitchedAckEvent(elem) => elem.element(),
            RTEEvent::OperationInvokedEvent(elem) => elem.element(),
            RTEEvent::OsTaskExecutionEvent(elem) => elem.element(),
            RTEEvent::SwcModeManagerErrorEvent(elem) => elem.element(),
            RTEEvent::SwcModeSwitchEvent(elem) => elem.element(),
            RTEEvent::TimingEvent(elem) => elem.element(),
            RTEEvent::TransformerHardErrorEvent(elem) => elem.element(),
        }
    }
}

impl TryFrom<Element> for RTEEvent {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::AsynchronousServerCallReturnsEvent => Ok(RTEEvent::AsynchronousServerCallReturnsEvent(
                AsynchronousServerCallReturnsEvent(element),
            )),
            ElementName::BackgroundEvent => Ok(RTEEvent::BackgroundEvent(BackgroundEvent(element))),
            ElementName::DataReceiveErrorEvent => Ok(RTEEvent::DataReceiveErrorEvent(DataReceiveErrorEvent(element))),
            ElementName::DataReceivedEvent => Ok(RTEEvent::DataReceivedEvent(DataReceivedEvent(element))),
            ElementName::DataSendCompletedEvent => {
                Ok(RTEEvent::DataSendCompletedEvent(DataSendCompletedEvent(element)))
            }
            ElementName::DataWriteCompletedEvent => {
                Ok(RTEEvent::DataWriteCompletedEvent(DataWriteCompletedEvent(element)))
            }
            ElementName::ExternalTriggerOccurredEvent => Ok(RTEEvent::ExternalTriggerOccurredEvent(
                ExternalTriggerOccurredEvent(element),
            )),
            ElementName::InitEvent => Ok(RTEEvent::InitEvent(InitEvent(element))),
            ElementName::InternalTriggerOccurredEvent => Ok(RTEEvent::InternalTriggerOccurredEvent(
                InternalTriggerOccurredEvent(element),
            )),
            ElementName::ModeSwitchedAckEvent => Ok(RTEEvent::ModeSwitchedAckEvent(ModeSwitchedAckEvent(element))),
            ElementName::OperationInvokedEvent => Ok(RTEEvent::OperationInvokedEvent(OperationInvokedEvent(element))),
            ElementName::OsTaskExecutionEvent => Ok(RTEEvent::OsTaskExecutionEvent(OsTaskExecutionEvent(element))),
            ElementName::SwcModeManagerErrorEvent => {
                Ok(RTEEvent::SwcModeManagerErrorEvent(SwcModeManagerErrorEvent(element)))
            }
            ElementName::SwcModeSwitchEvent => Ok(RTEEvent::SwcModeSwitchEvent(SwcModeSwitchEvent(element))),
            ElementName::TimingEvent => Ok(RTEEvent::TimingEvent(TimingEvent(element))),
            ElementName::TransformerHardErrorEvent => {
                Ok(RTEEvent::TransformerHardErrorEvent(TransformerHardErrorEvent(element)))
            }
            _ => Err(AutosarAbstractionError::ConversionError {
                element: element.clone(),
                dest: "RTEEvent".to_string(),
            }),
        }
    }
}

impl IdentifiableAbstractionElement for RTEEvent {}
impl AbstractRTEEvent for RTEEvent {}

//##################################################################

#[cfg(test)]
mod test {
    use crate::{
        AbstractionElement, AutosarModelAbstraction,
        software_component::{AbstractRTEEvent, AbstractSwComponentType},
    };
    use autosar_data::AutosarVersion;

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

        // create a timing event, which triggers runnable2
        let timing_event = swc_internal_behavior
            .create_timing_event("TimingEvent", &runnable2, 0.1)
            .unwrap();
        assert_eq!(timing_event.period().unwrap(), 0.1);
        assert_eq!(timing_event.runnable_entity().unwrap(), runnable2);
        assert_eq!(timing_event.swc_internal_behavior().unwrap(), swc_internal_behavior);

        // there should be 3 events in the swc_internal_behavior
        assert_eq!(swc_internal_behavior.events().count(), 3);
        // iterate over all events and check if they are the same as the ones we created
        let mut events_iter = swc_internal_behavior.events();
        assert_eq!(events_iter.next().unwrap().element(), init_event.element());
        assert_eq!(events_iter.next().unwrap().element(), op_invoked_event.element());
        assert_eq!(events_iter.next().unwrap().element(), timing_event.element());

        // runnable1 should be triggered by 2 events
        assert_eq!(runnable1.events().len(), 2);
        // runnable2 should be triggered by 1 event
        assert_eq!(runnable2.events().len(), 1);

        // add a data type mapping set to the swc_internal_behavior
        let data_type_mapping_set = package.create_data_type_mapping_set("MappingSet").unwrap();
        swc_internal_behavior
            .add_data_type_mapping_set(&data_type_mapping_set)
            .unwrap();
        assert_eq!(swc_internal_behavior.data_type_mapping_sets().count(), 1);
    }
}
