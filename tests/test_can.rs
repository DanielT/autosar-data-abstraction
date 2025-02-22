#[cfg(test)]
mod test {
    use autosar_data::{AutosarVersion, ElementName};
    use autosar_data_abstraction::{
        AbstractionElement, AutosarAbstractionError, AutosarModelAbstraction, SystemCategory,
        communication::{
            AbstractFrame, AbstractFrameTriggering, CanAddressingMode, CanClusterSettings, CanFrameType,
            CommunicationDirection, TransferProperty,
        },
        datatype::BaseTypeEncoding,
    };

    #[test]
    fn create_can_system() -> Result<(), AutosarAbstractionError> {
        let model = AutosarModelAbstraction::create("can.arxml", AutosarVersion::LATEST);
        let system_package = model.get_or_create_package("/System")?;
        let system = system_package.create_system("System", SystemCategory::SystemExtract)?;
        let cluster_package = model.get_or_create_package("/Network/Clusters")?;

        let settings = CanClusterSettings {
            can_fd_baudrate: Some(2000000),
            ..Default::default()
        };
        let can_cluster = system.create_can_cluster("CanCluster", &cluster_package, &settings)?;
        assert_eq!(can_cluster.element().element_name(), ElementName::CanCluster);
        let can_channel = can_cluster.create_physical_channel("CanChannel")?;

        let ecu_package = model.get_or_create_package("/Ecus")?;

        // create ECU A and connect it to the CAN channel
        let ecu_instance_a = system.create_ecu_instance("Ecu_A", &ecu_package)?;
        let canctrl_a = ecu_instance_a.create_can_communication_controller("CanController")?;
        let channels_iter = canctrl_a.connected_channels();
        assert_eq!(channels_iter.count(), 0);
        canctrl_a.connect_physical_channel("Ecu_A_connector", &can_channel)?;
        let channels_iter = canctrl_a.connected_channels();
        assert_eq!(channels_iter.count(), 1);

        // create ECU B and connect it to the CAN channel
        let ecu_instance_b = system.create_ecu_instance("Ecu_B", &ecu_package)?;
        let canctrl_b = ecu_instance_b.create_can_communication_controller("CanController")?;
        canctrl_b.connect_physical_channel("Ecu_B_connector", &can_channel)?;

        let frame_package = model.get_or_create_package("/Network/Frames")?;
        let pdu_package = model.get_or_create_package("/Network/Pdus")?;
        let isignal_package = model.get_or_create_package("/Network/Signals")?;
        let syssignal_package = model.get_or_create_package("/System/Signals")?;

        // create a base type for the CAN signals
        let base_type_package = model.get_or_create_package("/BaseTypes")?;
        let base_type_u8 =
            base_type_package.create_sw_base_type("uint8", 8, BaseTypeEncoding::None, None, None, Some("uint8"))?;

        // create Frame_1 which contains Pdu_1: Id 0x100, length 8
        let frame1 = system.create_can_frame("Frame_1", &frame_package, 8)?;
        let pdu1 = system.create_isignal_ipdu("Pdu_1", &pdu_package, 8)?;
        frame1.map_pdu(
            &pdu1,
            0,
            autosar_data_abstraction::ByteOrder::MostSignificantByteLast,
            None,
        )?;
        let ft_1 = can_channel.trigger_frame(&frame1, 0x100, CanAddressingMode::Standard, CanFrameType::Can20)?;
        assert_eq!(frame1.frame_triggerings().count(), 1);
        assert_eq!(ft_1.pdu_triggerings().count(), 1);

        // create Frame_2 which contains Pdu_2: Id 0x101, length 8
        let frame2 = system.create_can_frame("Frame_2", &frame_package, 8)?;
        let pdu2 = system.create_isignal_ipdu("Pdu_2", &pdu_package, 8)?;
        let ss_pdu2signal1 = syssignal_package.create_system_signal("P2S1")?;
        let pdu2signal1 = system.create_isignal("P2S1", &isignal_package, 4, &ss_pdu2signal1, Some(&base_type_u8))?;
        let ss_pdu2signal2 = syssignal_package.create_system_signal("P2S2")?;
        let pdu2signal2 = system.create_isignal("P2S2", &isignal_package, 4, &ss_pdu2signal2, Some(&base_type_u8))?;
        pdu2.map_signal(
            &pdu2signal1,
            0,
            autosar_data_abstraction::ByteOrder::MostSignificantByteFirst,
            None,
            TransferProperty::Triggered,
        )?;
        frame2.map_pdu(
            &pdu2,
            0,
            autosar_data_abstraction::ByteOrder::MostSignificantByteLast,
            None,
        )?;
        let ft_2 = can_channel.trigger_frame(&frame2, 0x101, CanAddressingMode::Standard, CanFrameType::Can20)?;
        pdu2.map_signal(
            &pdu2signal2,
            8,
            autosar_data_abstraction::ByteOrder::MostSignificantByteFirst,
            None,
            TransferProperty::Triggered,
        )?;

        // frame 1: Ecu_A -> Ecu_B
        ft_1.connect_to_ecu(&ecu_instance_a, CommunicationDirection::Out)?;
        ft_1.connect_to_ecu(&ecu_instance_b, CommunicationDirection::In)?;
        // frame 2: Ecu_B -> Ecu_A
        ft_2.connect_to_ecu(&ecu_instance_a, CommunicationDirection::In)?;
        ft_2.connect_to_ecu(&ecu_instance_b, CommunicationDirection::Out)?;

        // software component modeling
        let swc_package = model.get_or_create_package("/SoftwareComponents")?;
        let root_composition = swc_package.create_composition_sw_component_type("RootComposition")?;

        // ... Todo: create other swc elements ...

        // add the root composition to the system
        system.set_root_sw_composition("CanTestComposition", &root_composition)?;

        println!("{}", model.files().next().unwrap().serialize()?);
        model.write()?;

        Ok(())
    }
}
