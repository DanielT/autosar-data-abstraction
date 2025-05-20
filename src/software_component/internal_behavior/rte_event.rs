use crate::{
    AbstractionElement, AutosarAbstractionError, Element, IdentifiableAbstractionElement, abstraction_element,
    software_component::{
        ClientServerOperation, ModeDeclaration, PPortPrototype, PortInterface, PortPrototype, RunnableEntity,
        SwcInternalBehavior, VariableDataPrototype,
    },
};
use autosar_data::{ElementName, EnumItem};

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
    #[must_use]
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

impl BackgroundEvent {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        runnable: &RunnableEntity,
    ) -> Result<Self, AutosarAbstractionError> {
        let background_event = parent.create_named_sub_element(ElementName::BackgroundEvent, name)?;
        let background_event = Self(background_event);
        background_event.set_runnable_entity(runnable)?;

        Ok(background_event)
    }
}

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

impl DataReceivedEvent {
    pub(crate) fn new<T: Into<PortPrototype> + Clone>(
        name: &str,
        parent: &Element,
        runnable: &RunnableEntity,
        variable_data_prototype: &VariableDataPrototype,
        context_port: &T,
    ) -> Result<Self, AutosarAbstractionError> {
        let data_received_event = parent.create_named_sub_element(ElementName::DataReceivedEvent, name)?;
        let data_received_event = Self(data_received_event);
        data_received_event.set_runnable_entity(runnable)?;

        let result = data_received_event.set_variable_data_prototype(variable_data_prototype, context_port);
        if let Err(err) = result {
            // this operation could fail if bad parameters are provided; in this case we remove the event
            parent.remove_sub_element(data_received_event.0)?;
            return Err(err);
        }

        Ok(data_received_event)
    }

    /// Set the `VariableDataPrototype` that triggers the `DataReceivedEvent`
    pub fn set_variable_data_prototype<T: Into<PortPrototype> + Clone>(
        &self,
        variable_data_prototype: &VariableDataPrototype,
        context_port: &T,
    ) -> Result<(), AutosarAbstractionError> {
        let context_port = context_port.clone().into();
        // reject P-Ports. It's not clear if PRPortPrototypes are allowed here, so let's not reject them for now
        if matches!(context_port, PortPrototype::P(_)) {
            return Err(AutosarAbstractionError::InvalidParameter(
                "A DataReceivedEvent must refer to a port using an RPortPrototype".to_string(),
            ));
        }
        // the port must be a sender-receiver port
        let PortInterface::SenderReceiverInterface(sr_interface) = context_port.port_interface()? else {
            return Err(AutosarAbstractionError::InvalidParameter(
                "A DataReceivedEvent must refer to a port using a SenderReceiverInterface".to_string(),
            ));
        };
        // the variable data prototype must be part of the sender-receiver interface
        if sr_interface != variable_data_prototype.interface()? {
            return Err(AutosarAbstractionError::InvalidParameter(format!(
                "VariableDataPrototype {} is not part of SenderReceiverInterface {}",
                variable_data_prototype.name().as_deref().unwrap_or("(invalid)"),
                sr_interface.name().as_deref().unwrap_or("(invalid)")
            )));
        }

        // all ok, create the reference
        let data_iref = self.element().get_or_create_sub_element(ElementName::DataIref)?;
        data_iref
            .get_or_create_sub_element(ElementName::ContextRPortRef)?
            .set_reference_target(context_port.element())?;
        data_iref
            .get_or_create_sub_element(ElementName::TargetDataElementRef)?
            .set_reference_target(variable_data_prototype.element())?;

        Ok(())
    }

    /// Get the `VariableDataPrototype` that triggers the `DataReceivedEvent`
    #[must_use]
    pub fn variable_data_prototype(&self) -> Option<(VariableDataPrototype, PortPrototype)> {
        let data_iref = self.element().get_sub_element(ElementName::DataIref)?;
        let variable_data_prototype_elem = data_iref
            .get_sub_element(ElementName::TargetDataElementRef)?
            .get_reference_target()
            .ok()?;
        let context_port_elem = data_iref
            .get_sub_element(ElementName::ContextRPortRef)?
            .get_reference_target()
            .ok()?;
        let variable_data_prototype = VariableDataPrototype::try_from(variable_data_prototype_elem).ok()?;
        let context_port = PortPrototype::try_from(context_port_elem).ok()?;
        Some((variable_data_prototype, context_port))
    }
}

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
            .get_or_create_sub_element(ElementName::TargetProvidedOperationRef)?
            .set_reference_target(client_server_operation.element())?;
        op_iref
            .get_or_create_sub_element(ElementName::ContextPPortRef)?
            .set_reference_target(context_p_port.element())?;
        Ok(())
    }

    /// Get the `ClientServerOperation` that triggers the `OperationInvokedEvent`
    #[must_use]
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

impl OsTaskExecutionEvent {
    pub(crate) fn new(
        name: &str,
        parent: &Element,
        runnable_entity: &RunnableEntity,
    ) -> Result<Self, AutosarAbstractionError> {
        let os_task_execution_event_elem = parent.create_named_sub_element(ElementName::OsTaskExecutionEvent, name)?;
        let os_task_execution_event = Self(os_task_execution_event_elem);
        os_task_execution_event.set_runnable_entity(runnable_entity)?;

        Ok(os_task_execution_event)
    }
}

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

impl SwcModeSwitchEvent {
    pub(crate) fn new<T: Into<PortPrototype> + Clone>(
        name: &str,
        parent: &Element,
        runnable: &RunnableEntity,
        activation: ModeActivationKind,
        context_port: &T,
        mode_declaration: &ModeDeclaration,
        second_mode_declaration: Option<&ModeDeclaration>,
    ) -> Result<Self, AutosarAbstractionError> {
        let swc_mode_switch_event = parent.create_named_sub_element(ElementName::SwcModeSwitchEvent, name)?;
        let swc_mode_switch_event = Self(swc_mode_switch_event);
        swc_mode_switch_event.set_runnable_entity(runnable)?;

        swc_mode_switch_event.set_mode_activation_kind(activation)?;

        // set the context port and mode declaration
        let result =
            swc_mode_switch_event.set_mode_declaration(context_port, mode_declaration, second_mode_declaration);
        if let Err(err) = result {
            // this operation could fail if bad parameters are provided; in this case we remove the event
            parent.remove_sub_element(swc_mode_switch_event.0)?;
            return Err(err);
        }

        Ok(swc_mode_switch_event)
    }

    /// Set the `ModeActivationKind` that controls when the `SwcModeSwitchEvent` is triggered
    pub fn set_mode_activation_kind(&self, activation: ModeActivationKind) -> Result<(), AutosarAbstractionError> {
        self.element()
            .get_or_create_sub_element(ElementName::Activation)?
            .set_character_data::<EnumItem>(activation.into())?;
        Ok(())
    }

    /// Get the `ModeActivationKind` that controls when the `SwcModeSwitchEvent` is triggered
    #[must_use]
    pub fn mode_activation_kind(&self) -> Option<ModeActivationKind> {
        let value = self
            .element()
            .get_sub_element(ElementName::Activation)?
            .character_data()?
            .enum_value()?;
        ModeActivationKind::try_from(value).ok()
    }

    /// Set the `ModeDeclaration` that triggers the `SwcModeSwitchEvent`
    ///
    /// The second mode must be provided if the activation kind `OnTransition` is configured.
    /// In that case only transitions between the two modes trigger the event.
    pub fn set_mode_declaration<T: Into<PortPrototype> + Clone>(
        &self,
        context_port: &T,
        mode_declaration: &ModeDeclaration,
        second_mode_declaration: Option<&ModeDeclaration>,
    ) -> Result<(), AutosarAbstractionError> {
        let context_port = context_port.clone().into();
        let interface = context_port.port_interface()?;
        let PortInterface::ModeSwitchInterface(mode_switch_interface) = interface else {
            return Err(AutosarAbstractionError::InvalidParameter(
                "A ModeSwitchEvent must refer to a port using a ModeSwitchInterface".to_string(),
            ));
        };
        let Some(interface_mode_group) = mode_switch_interface.mode_group() else {
            return Err(AutosarAbstractionError::InvalidParameter(
                "A ModeSwitchEvent cannot refer a port whose ModeSwitchInterface does not contain a ModeGroup"
                    .to_string(),
            ));
        };
        let Some(mode_declaration_group) = interface_mode_group.mode_declaration_group() else {
            return Err(AutosarAbstractionError::InvalidParameter(format!(
                "ModeGroup {} is invalid: the reference a ModeDeclarationGroup is missing",
                interface_mode_group.name().as_deref().unwrap_or("(invalid)")
            )));
        };

        // verify that the mode_declaration is part of the mode declaration group of the context port interface
        if mode_declaration.mode_declaration_group()? != mode_declaration_group {
            return Err(AutosarAbstractionError::InvalidParameter(format!(
                "ModeDeclaration {} is not part of ModeDeclarationGroup {}",
                mode_declaration.name().as_deref().unwrap_or("(invalid)"),
                mode_declaration_group.name().as_deref().unwrap_or("(invalid)")
            )));
        }
        // verify that the second mode_declaration is part of the mode declaration group of the context port interface
        if let Some(second_mode_declaration) = second_mode_declaration {
            if second_mode_declaration.mode_declaration_group()? != mode_declaration_group {
                return Err(AutosarAbstractionError::InvalidParameter(format!(
                    "ModeDeclaration {} is not part of ModeDeclarationGroup {}",
                    second_mode_declaration.name().as_deref().unwrap_or("(invalid)"),
                    mode_declaration_group.name().as_deref().unwrap_or("(invalid)")
                )));
            }
        }

        let _ = self.element().remove_sub_element_kind(ElementName::ModeIrefs);
        let mode_irefs_elem = self.element().create_sub_element(ElementName::ModeIrefs)?;

        let mode_iref = mode_irefs_elem.create_sub_element(ElementName::ModeIref)?;
        mode_iref
            .create_sub_element(ElementName::ContextPortRef)?
            .set_reference_target(context_port.element())?;
        mode_iref
            .create_sub_element(ElementName::ContextModeDeclarationGroupPrototypeRef)?
            .set_reference_target(interface_mode_group.element())?;
        mode_iref
            .create_sub_element(ElementName::TargetModeDeclarationRef)?
            .set_reference_target(mode_declaration.element())?;

        if let Some(second_mode_declaration) = second_mode_declaration {
            let second_mode_iref = mode_irefs_elem.create_sub_element(ElementName::ModeIref)?;
            second_mode_iref
                .create_sub_element(ElementName::ContextPortRef)?
                .set_reference_target(context_port.element())?;
            second_mode_iref
                .create_sub_element(ElementName::ContextModeDeclarationGroupPrototypeRef)?
                .set_reference_target(interface_mode_group.element())?;
            second_mode_iref
                .create_sub_element(ElementName::TargetModeDeclarationRef)?
                .set_reference_target(second_mode_declaration.element())?;
        }

        Ok(())
    }

    /// Get the `ModeDeclaration`s that trigger the `SwcModeSwitchEvent`
    ///
    /// The list contains either one or two `ModeDeclaration`s depending on the `ModeActivationKind`.
    #[must_use]
    pub fn mode_declarations(&self) -> Option<(Vec<ModeDeclaration>, PortPrototype)> {
        let mode_irefs_elem = self.element().get_sub_element(ElementName::ModeIrefs)?;
        let mode_declarations = mode_irefs_elem
            .sub_elements()
            .filter_map(|mode_iref_elem| {
                mode_iref_elem
                    .get_sub_element(ElementName::TargetModeDeclarationRef)
                    .and_then(|tref_elem| tref_elem.get_reference_target().ok())
                    .and_then(|elem| ModeDeclaration::try_from(elem).ok())
            })
            .collect();
        let port_elem = mode_irefs_elem
            .get_sub_element(ElementName::ModeIref)?
            .get_sub_element(ElementName::ContextPortRef)?
            .get_reference_target()
            .ok()?;
        let port_proto = PortPrototype::try_from(port_elem).ok()?;
        Some((mode_declarations, port_proto))
    }
}

//##################################################################

/// Kind of mode switch condition used for activation of an event
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModeActivationKind {
    /// On entering the mode
    OnEntry,
    /// On leaving the mode
    OnExit,
    /// on transition from the first mode to the second mode
    OnTransition,
}

impl From<ModeActivationKind> for EnumItem {
    fn from(activation_kind: ModeActivationKind) -> Self {
        match activation_kind {
            ModeActivationKind::OnEntry => EnumItem::OnEntry,
            ModeActivationKind::OnExit => EnumItem::OnExit,
            ModeActivationKind::OnTransition => EnumItem::OnTransition,
        }
    }
}

impl TryFrom<EnumItem> for ModeActivationKind {
    type Error = AutosarAbstractionError;

    fn try_from(activation_kind: EnumItem) -> Result<Self, Self::Error> {
        match activation_kind {
            EnumItem::OnEntry => Ok(ModeActivationKind::OnEntry),
            EnumItem::OnExit => Ok(ModeActivationKind::OnExit),
            EnumItem::OnTransition => Ok(ModeActivationKind::OnTransition),
            _ => Err(AutosarAbstractionError::ValueConversionError {
                value: activation_kind.to_string(),
                dest: "ModeActivationKind".to_string(),
            }),
        }
    }
}

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
