#[cfg(test)]
mod test {
    use autosar_data::{AutosarVersion, ElementName};
    use autosar_data_abstraction::{
        communication::{
            AbstractFrame, AbstractFrameTriggering, CommunicationDirection, FlexrayChannelName, FlexrayClusterSettings,
            FlexrayCommunicationCycle,
        },
        AbstractionElement, AutosarAbstractionError, AutosarModelAbstraction, SystemCategory,
    };

    #[test]
    fn create_flexray_system() -> Result<(), AutosarAbstractionError> {
        let model = AutosarModelAbstraction::create("flexray.arxml", AutosarVersion::LATEST);
        let system_package = model.get_or_create_package("/System")?;
        let system = system_package.create_system("System", SystemCategory::SystemExtract)?;
        let cluster_package = model.get_or_create_package("/Network/Clusters")?;

        let settings = FlexrayClusterSettings::default();
        let flx_cluster = system.create_flexray_cluster("FlxCluster", &cluster_package, &settings)?;
        assert_eq!(flx_cluster.element().element_name(), ElementName::FlexrayCluster);
        let flx_channel = flx_cluster.create_physical_channel("FlxChannel", FlexrayChannelName::A)?;

        let ecu_package = model.get_or_create_package("/Ecus")?;

        // create ECU A and connect it to the Flexray channel
        let ecu_instance_a = system.create_ecu_instance("Ecu_A", &ecu_package)?;
        let flxctrl_a = ecu_instance_a.create_flexray_communication_controller("FlexrayController")?;
        let channels_iter = flxctrl_a.connected_channels();
        assert_eq!(channels_iter.count(), 0);
        flxctrl_a.connect_physical_channel("Ecu_A_connector", &flx_channel)?;
        let channels_iter = flxctrl_a.connected_channels();
        assert_eq!(channels_iter.count(), 1);

        // create ECU B and connect it to the Flexray channel
        let ecu_instance_b = system.create_ecu_instance("Ecu_B", &ecu_package)?;
        let flxctrl_b = ecu_instance_b.create_flexray_communication_controller("FlexrayController")?;
        flxctrl_b.connect_physical_channel("Ecu_B_connector", &flx_channel)?;

        let frame_package = model.get_or_create_package("/Network/Frames")?;
        let pdu_package = model.get_or_create_package("/Network/Pdus")?;

        // create Frame_1 which contains Pdu_1: Id 0x100, length 8
        let frame1 = system.create_flexray_frame("Frame_1", &frame_package, 32)?;
        let pdu1 = system.create_isignal_ipdu("Pdu_1", &pdu_package, 8)?;
        frame1.map_pdu(
            &pdu1,
            0,
            autosar_data_abstraction::ByteOrder::MostSignificantByteLast,
            None,
        )?;
        let frame_timing_1 = FlexrayCommunicationCycle::Repetition {
            base_cycle: 0,
            cycle_repetition: autosar_data_abstraction::communication::CycleRepetition::C1,
        };
        let ft_1 = flx_channel.trigger_frame(&frame1, 1, &frame_timing_1)?;
        assert_eq!(frame1.frame_triggerings().count(), 1);
        assert_eq!(ft_1.pdu_triggerings().count(), 1);

        // create Frame_2 which contains Pdu_2: Id 0x101, length 8
        let frame2 = system.create_flexray_frame("Frame_2", &frame_package, 64)?;
        let pdu2 = system.create_isignal_ipdu("Pdu_2", &pdu_package, 8)?;
        frame2.map_pdu(
            &pdu2,
            0,
            autosar_data_abstraction::ByteOrder::MostSignificantByteLast,
            None,
        )?;
        let frame_timing_2 = FlexrayCommunicationCycle::Repetition {
            base_cycle: 0,
            cycle_repetition: autosar_data_abstraction::communication::CycleRepetition::C1,
        };
        let ft_2 = flx_channel.trigger_frame(&frame2, 2, &frame_timing_2)?;

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
        system.set_root_sw_composition("FlexrayTestComposition", &root_composition)?;

        println!("{}", model.files().next().unwrap().serialize()?);
        // model.write()?;

        Ok(())
    }
}
