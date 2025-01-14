#[cfg(test)]
mod test {
    use autosar_data::{AutosarModel, AutosarVersion, ElementName};
    use autosar_data_abstraction::{
        communication::{
            CommonServiceDiscoveryConfig, CommunicationDirection, CyclicTiming, E2EProfile, E2EProfileBehavior,
            E2ETransformationTechnologyConfig, EthernetVlanInfo, EventControlledTiming, EventGroupControlType,
            GeneralPurposePduCategory, IPv4AddressSource, IpduTiming, NetworkEndpointAddress, PduCollectionTrigger,
            SdConfig, SocketAddressType, SomeIpMessageType, SomeIpTransformationISignalPropsConfig,
            SomeIpTransformationTechnologyConfig, TpConfig, TransferProperty, TransformationISignalPropsConfig,
            TransformationTechnologyConfig, TransmissionModeTiming,
        },
        datatype::{ApplicationPrimitiveCategory, BaseTypeEncoding, ImplementationDataTypeSettings},
        software_component::AbstractSwComponentType,
        AbstractionElement, ArPackage, AutosarAbstractionError, ByteOrder, SystemCategory,
    };

    #[test]
    fn create_ethernet_v1_system() -> Result<(), AutosarAbstractionError> {
        let model = AutosarModel::new();
        model.create_file("ethernet_v1.arxml", AutosarVersion::Autosar_00046)?;
        let package_1 = ArPackage::get_or_create(&model, "/System")?;
        let system = package_1.create_system("System", SystemCategory::SystemExtract)?;
        let package_2 = ArPackage::get_or_create(&model, "/Clusters")?;

        // create an Ethernet cluster and a physical channel for VLAN 33
        let eth_cluster = system.create_ethernet_cluster("EthCluster", &package_2)?;
        assert_eq!(eth_cluster.element().element_name(), ElementName::EthernetCluster);
        let vlan_info = EthernetVlanInfo {
            vlan_id: 33,
            vlan_name: "VLAN_33".to_string(),
        };
        let eth_channel = eth_cluster.create_physical_channel("EthChannel", Some(vlan_info))?;
        let vlan_info_2 = eth_channel.vlan_info().unwrap();
        assert_eq!(vlan_info_2.vlan_id, 33);

        // create an ECU instance and connect it to the Ethernet channel
        let package_3 = ArPackage::get_or_create(&model, "/Ecus")?;
        let ecu_instance_a = system.create_ecu_instance("Ecu_A", &package_3)?;
        let ethctrl = ecu_instance_a
            .create_ethernet_communication_controller("EthernetController", Some("ab:cd:ef:01:02:03".to_string()))?;
        let channels_iter = ethctrl.connected_channels();
        assert_eq!(channels_iter.count(), 0);
        ethctrl.connect_physical_channel("Ecu_A_connector", &eth_channel)?;
        let channels_iter = ethctrl.connected_channels();
        assert_eq!(channels_iter.count(), 1);

        // create a local socket which belongs to Ecu_A
        let network_address_ecu_a = NetworkEndpointAddress::IPv4 {
            address: Some("192.168.0.1".to_string()),
            address_source: Some(IPv4AddressSource::Fixed),
            default_gateway: Some("192.168.0.200".to_string()),
            network_mask: Some("255.255.255.0".to_string()),
        };
        let network_endpoint_ecu_a =
            eth_channel.create_network_endpoint("local_endpoint", network_address_ecu_a, None)?;
        let tcp_port_ecu_a = TpConfig::TcpTp {
            port_number: Some(1234),
            port_dynamically_assigned: None,
        };
        let socket_type_1 = SocketAddressType::Unicast(Some(ecu_instance_a.clone()));
        let socket_address_tcp_ecu_a = eth_channel.create_socket_address(
            "ServerSocket",
            &network_endpoint_ecu_a,
            &tcp_port_ecu_a,
            socket_type_1,
        )?;
        assert_eq!(tcp_port_ecu_a, socket_address_tcp_ecu_a.tp_config().unwrap());
        let socket_type = socket_address_tcp_ecu_a.socket_address_type().unwrap();
        assert!(matches!(socket_type, SocketAddressType::Unicast(Some(_))));
        if let SocketAddressType::Unicast(Some(ecu)) = socket_type {
            assert_eq!(ecu, ecu_instance_a);
        }

        // remote socket - not associated with any ECU
        let network_address_remote = NetworkEndpointAddress::IPv4 {
            address: Some("192.168.0.2".to_string()),
            address_source: Some(IPv4AddressSource::Fixed),
            default_gateway: Some("192.168.0.200".to_string()),
            network_mask: Some("255.255.255.0".to_string()),
        };
        let network_endpoint_remote =
            eth_channel.create_network_endpoint("remote_endpoint", network_address_remote, None)?;
        let tcp_port_remote = TpConfig::TcpTp {
            port_number: Some(5678),
            port_dynamically_assigned: None,
        };
        let socket_type_2 = SocketAddressType::Unicast(None);
        let socket_address_tcp_remote = eth_channel.create_socket_address(
            "ClientSocket",
            &network_endpoint_remote,
            &tcp_port_remote,
            socket_type_2,
        )?;

        // create a connection (V1)
        let socket_connection_bundle =
            eth_channel.create_socket_connection_bundle("SomeIpConnectionBundle", &socket_address_tcp_ecu_a)?;
        let connection = socket_connection_bundle.create_bundled_connection(&socket_address_tcp_remote)?;

        // create a pdu and add it to the connection
        // PDU-based communication is not typically used in Ethernet, but it is possible. A more conventional choice would be to use SomeIp.
        let pdu_package = ArPackage::get_or_create(&model, "/Network/Pdus")?;
        let isignal_package = ArPackage::get_or_create(&model, "/Network/Signals")?;
        let syssignal_package = ArPackage::get_or_create(&model, "/System/Signals")?;
        let static_pdu = system.create_isignal_ipdu("Pdu_1", &pdu_package, 800)?;
        // create two signals for the PDU
        let system_signal_1 = syssignal_package.create_system_signal("Signal_1")?;
        let system_signal_2 = syssignal_package.create_system_signal("Signal_2")?;
        let static_isignal_1 = system.create_isignal("Signal_1", 400, &system_signal_1, None, &isignal_package)?;
        let static_isignal_2 = system.create_isignal("Signal_2", 400, &system_signal_2, None, &isignal_package)?;
        static_pdu.set_timing(&IpduTiming {
            minimum_delay: None,
            transmission_mode_false_timing: None,
            transmission_mode_true_timing: Some(TransmissionModeTiming {
                event_controlled_timing: None,
                cyclic_timing: Some(CyclicTiming {
                    time_period: 0.01,
                    time_offset: None,
                }),
            }),
        })?;

        // map the signals to the PDU
        static_pdu.map_signal(
            &static_isignal_1,
            0,
            ByteOrder::MostSignificantByteLast,
            None,
            TransferProperty::Triggered,
        )?;
        static_pdu.map_signal(
            &static_isignal_2,
            400,
            ByteOrder::MostSignificantByteLast,
            None,
            TransferProperty::Triggered,
        )?;

        // mapping the PDU to the ECU gives us a PduTriggering, on which a PduPort can be created
        let pdu_triggering = connection.trigger_pdu(&static_pdu, 0x1000, None, None)?;
        assert_eq!(pdu_triggering, connection.pdu_triggerings().next().unwrap());

        pdu_triggering.create_pdu_port(&ecu_instance_a, CommunicationDirection::Out)?;

        let system_mapping = system.get_or_create_mapping("SystemMapping")?;

        // ---------------------------------------------------------
        // software component modeling
        let swc_package = ArPackage::get_or_create(&model, "/SoftwareComponents")?;
        let root_composition = swc_package.create_composition_sw_component_type("RootComposition")?;

        // add the root composition to the system
        system.set_root_sw_composition("EthernetTestComposition", &root_composition)?;

        // create a composition type and create a composition prototype from it for Ecu_A
        let ecu_a_composition = swc_package.create_composition_sw_component_type("Ecu_A_Composition")?;
        let ecu_a_composition_prototype =
            root_composition.create_component("Ecu_A_Composition_Prototype", &ecu_a_composition)?;
        system_mapping.map_swc_to_ecu(
            "Ecu_A_Composition_Prototype_Mapping",
            &ecu_a_composition_prototype,
            &ecu_instance_a,
        )?;

        // create an application software component and a prototype from it for Ecu_A
        let application_swc_a = swc_package.create_application_sw_component_type("ApplicationSwComponent")?;
        let application_swc_a_prototype =
            ecu_a_composition.create_component("ApplicationSwComponent_Prototype", &application_swc_a)?;
        system_mapping.map_swc_to_ecu(
            "ApplicationSwComponent_Prototype_Mapping",
            &application_swc_a_prototype,
            &ecu_instance_a,
        )?;

        // create a pair of implementaion and application data types
        let base_type_package = ArPackage::get_or_create(&model, "/BaseTypes")?;
        let data_type_package = ArPackage::get_or_create(&model, "/DataTypes")?;

        let base_type_u8 =
            base_type_package.create_sw_base_type("uint8", 8, BaseTypeEncoding::None, None, None, Some("uint8"))?;
        let implementation_data_type_u8 =
            data_type_package.create_implementation_data_type(ImplementationDataTypeSettings::Value {
                name: "ImplDataType_u8".to_string(),
                base_type: base_type_u8.clone(),
                compu_method: None,
                data_constraint: None,
            })?;

        let implementation_data_type_array =
            data_type_package.create_implementation_data_type(ImplementationDataTypeSettings::Array {
                name: "ImplDataType_array".to_string(),
                element_type: Box::new(ImplementationDataTypeSettings::Value {
                    name: "ImplDataType_u8".to_string(),
                    base_type: base_type_u8,
                    compu_method: None,
                    data_constraint: None,
                }),
                length: 50,
            })?;

        let application_data_type_u8 = data_type_package.create_application_primitive_data_type(
            "AppDataType_u8",
            ApplicationPrimitiveCategory::Value,
            None,
            None,
            None,
        )?;
        let application_data_type_array =
            data_type_package.create_application_array_data_type("AppDataType_array", &application_data_type_u8, 50)?;

        // create a type mapping
        let type_mapping_package = ArPackage::get_or_create(&model, "/TypeMappings")?;
        let type_mapping_set = type_mapping_package.create_data_type_mapping_set("TypeMappingSet")?;
        type_mapping_set.create_data_type_map(&implementation_data_type_u8, &application_data_type_u8)?;
        type_mapping_set.create_data_type_map(&implementation_data_type_array, &application_data_type_array)?;

        // create a sender-receiver interface
        let sender_receiver_package = ArPackage::get_or_create(&model, "/Interfaces")?;
        let sender_receiver_interface =
            sender_receiver_package.create_sender_receiver_interface("SenderReceiverInterface")?;
        let data_element_a =
            sender_receiver_interface.create_data_element("DataElement_a", &application_data_type_array)?;
        let data_element_b =
            sender_receiver_interface.create_data_element("DataElement_b", &application_data_type_array)?;

        // create a port for the sender-receiver interface at every level of the component hierarchy
        let pport_prototype = application_swc_a.create_p_port("provide_port", &sender_receiver_interface)?;
        let pport_prototype_2 = ecu_a_composition.create_p_port("provide_port", &sender_receiver_interface)?;
        let pport_prototype_3 = root_composition.create_p_port("provide_port", &sender_receiver_interface)?;

        // connect the ports to each other; this results in the creation of delegation connectors
        let _delegation_connector_1 = root_composition.create_delegation_connector(
            "delegation_connector",
            &pport_prototype_2,
            &ecu_a_composition_prototype,
            &pport_prototype_3,
        )?;
        let _delegation_connector_2 = ecu_a_composition.create_delegation_connector(
            "delegation_connector",
            &pport_prototype,
            &application_swc_a_prototype,
            &pport_prototype_2,
        )?;

        // map the sender-receiver interface to the signals
        system_mapping.map_sender_receiver_to_signal(
            &system_signal_1,
            &data_element_a,
            &pport_prototype_3,
            &[],
            None,
        )?;
        system_mapping.map_sender_receiver_to_signal(
            &system_signal_2,
            &data_element_b,
            &pport_prototype_3,
            &[],
            None,
        )?;

        // ---------------------------------------------------------

        // SomeIP modeling
        // configure the Ethernet channel to use service discovery
        let unicast_socket = eth_channel.create_socket_address(
            "UnicastSocket",
            &network_endpoint_ecu_a,
            &TpConfig::UdpTp {
                port_number: Some(30490),
                port_dynamically_assigned: None,
            },
            SocketAddressType::Unicast(Some(ecu_instance_a.clone())),
        )?;
        let multicast_rx_endpoint = eth_channel.create_network_endpoint(
            "MulticastEndpoint",
            NetworkEndpointAddress::IPv4 {
                address: Some("239.0.0.1".to_string()),
                address_source: Some(IPv4AddressSource::Fixed),
                default_gateway: None,
                network_mask: None,
            },
            None,
        )?;
        let multicast_rx_socket = eth_channel.create_socket_address(
            "MulticastSocket",
            &multicast_rx_endpoint,
            &TpConfig::UdpTp {
                port_number: Some(30490),
                port_dynamically_assigned: None,
            },
            SocketAddressType::Multicast(vec![ecu_instance_a.clone()]),
        )?;
        let remote_anyaddr_endpoint = eth_channel.create_network_endpoint(
            "RemoteEndpoint",
            NetworkEndpointAddress::IPv4 {
                address: Some("ANY".to_string()),
                address_source: None,
                default_gateway: None,
                network_mask: None,
            },
            None,
        )?;
        let remote_anyaddr_socket = eth_channel.create_socket_address(
            "RemoteSocket",
            &remote_anyaddr_endpoint,
            &TpConfig::UdpTp {
                port_number: None,
                port_dynamically_assigned: Some(true),
            },
            SocketAddressType::Unicast(None),
        )?;
        let unicast_rx_pdu =
            system.create_general_purpose_pdu("UnicastRxPdu", &pdu_package, 0, GeneralPurposePduCategory::Sd)?;
        let unicast_tx_pdu =
            system.create_general_purpose_pdu("UnicastTxPdu", &pdu_package, 0, GeneralPurposePduCategory::Sd)?;
        let multicast_rx_pdu =
            system.create_general_purpose_pdu("MulticastRxPdu", &pdu_package, 0, GeneralPurposePduCategory::Sd)?;
        let common_config = CommonServiceDiscoveryConfig {
            multicast_rx_socket: &multicast_rx_socket,
            multicast_rx_pdu: &multicast_rx_pdu,
            remote_socket: &remote_anyaddr_socket,
            name_prefix: None,
            prefer_static_socket_connections: false,
            ipdu_identifier_set: None,
        };

        eth_channel.configure_service_discovery_for_ecu(
            &ecu_instance_a,
            &unicast_socket,
            &unicast_rx_pdu,
            &unicast_tx_pdu,
            &common_config,
        )?;

        // create communication elements for SomeIP
        // In SomeIp communication, a pdu only contains a single signal. The data type of the signal is a byte array.
        // Complex application data is transformed into the byte array by a data transformation that contains a SomeIP transformer.
        // the chain can also contain other transformations, in particular an E2E transformation.

        // create a data transformation
        let data_transformer_package = ArPackage::get_or_create(&model, "/DataTransformations")?;
        let transformation_set = data_transformer_package.create_data_transformation_set("DataTransformationSet")?;
        let someip_config = TransformationTechnologyConfig::SomeIp(SomeIpTransformationTechnologyConfig {
            alignment: 8,
            byte_order: ByteOrder::MostSignificantByteLast,
            interface_version: 1,
        });
        let someip_tranformation_technology =
            transformation_set.create_transformation_technology("SomeIpTransformationTechnology", &someip_config)?;
        let e2e_config = TransformationTechnologyConfig::E2E(E2ETransformationTechnologyConfig {
            profile: E2EProfile::P07,
            zero_header_length: false, // not used in this combination with SomeIP
            transform_in_place: false,
            offset: 64,
            max_delta_counter: 5,
            max_error_state_init: 0,
            max_error_state_invalid: 0,
            max_error_state_valid: 0,
            max_no_new_or_repeated_data: 4,
            min_ok_state_init: 4,
            min_ok_state_invalid: 5,
            min_ok_state_valid: 2,
            window_size: 10,
            window_size_init: None,
            window_size_invalid: None,
            window_size_valid: None,
            profile_behavior: Some(E2EProfileBehavior::R4_2),
            sync_counter_init: None,
            data_id_mode: None,
            data_id_nibble_offset: None,
            crc_offset: None,
            counter_offset: None,
        });
        let e2e_tranformation_technology =
            transformation_set.create_transformation_technology("E2ETransformationTechnology", &e2e_config)?;
        let data_transformation = transformation_set.create_data_transformation(
            "DataTransformation",
            &[&someip_tranformation_technology, &e2e_tranformation_technology],
            true,
        )?;

        // create a Signal for SomeIP communication
        let system_signal_3 = syssignal_package.create_system_signal("SomeIp_Signal_1")?;
        let someip_isignal_1 =
            system.create_isignal("Someip_Signal_1", 400, &system_signal_3, None, &isignal_package)?;
        someip_isignal_1.add_data_transformation(&data_transformation)?;
        someip_isignal_1.add_transformation_isignal_props(
            &someip_tranformation_technology,
            &TransformationISignalPropsConfig::SomeIp(SomeIpTransformationISignalPropsConfig {
                legacy_strings: None,
                dynamic_length: None,
                message_type: Some(SomeIpMessageType::Notification),
                size_of_array_length: None,
                size_of_string_length: None,
                size_of_struct_length: None,
                size_of_union_length: None,
                interface_version: None,
            }),
        )?;

        // create a PDU for SomeIp communication. The PDU is larger than the signal to account for the SomeIP and E2E headers.
        // SomeIp: 64 bits; E2E Profile 7: 160 bits
        let someip_pdu = system.create_isignal_ipdu("Someip_Pdu", &pdu_package, 624)?;
        someip_pdu.map_signal(
            &someip_isignal_1,
            0,
            ByteOrder::MostSignificantByteLast,
            None,
            TransferProperty::Triggered,
        )?;
        someip_pdu.set_timing(&IpduTiming {
            minimum_delay: None,
            transmission_mode_false_timing: None,
            transmission_mode_true_timing: Some(TransmissionModeTiming {
                event_controlled_timing: Some(EventControlledTiming {
                    number_of_repetitions: 0,
                    repetition_period: None,
                }),
                cyclic_timing: None,
            }),
        })?;

        // SomeIp transport layer: the service instances should be created inside the SocketAddresses
        let rg_package = ArPackage::get_or_create(&model, "/RoutingGroups")?;
        let someip_routing_group = system.create_so_ad_routing_group(
            "SomeIpRoutingGroup",
            &rg_package,
            Some(EventGroupControlType::ActivationUnicast),
        )?;

        // socket_address_tcp_ecu_a already exists
        let socket_address_udp_ecu_a = eth_channel.create_socket_address(
            "UdpSocket",
            &network_endpoint_ecu_a,
            &TpConfig::UdpTp {
                port_number: Some(50000), // arbitrary port number, in particular it does not need to match the SD port number
                port_dynamically_assigned: None,
            },
            SocketAddressType::Unicast(Some(ecu_instance_a.clone())),
        )?;

        let socket_address_udp_remote = eth_channel.create_socket_address(
            "UdpRemoteSocket",
            &network_endpoint_remote,
            &TpConfig::UdpTp {
                port_number: Some(50000),
                port_dynamically_assigned: None,
            },
            SocketAddressType::Unicast(None),
        )?;

        let service_identifier = 0xbaad;
        let instance_identifier = 0xfeed;
        // create a provided service instance
        let psi_ecu_a = socket_address_udp_ecu_a.create_provided_service_instance(
            "PSI_Ecu_A",
            service_identifier,
            instance_identifier,
        )?;
        let psi_ecu_a_eh = psi_ecu_a.create_event_handler("PSI_Ecu_A_EH_1")?;
        psi_ecu_a_eh.add_routing_group(&someip_routing_group)?;

        // create a consumed service instance
        let csi_remote = socket_address_udp_remote.create_consumed_service_instance("CSI_Remote", &psi_ecu_a)?;
        let csi_remote_ceg = csi_remote.create_consumed_event_group("CSI_Remote_CEG_1", 1, &psi_ecu_a_eh)?;
        csi_remote_ceg.add_routing_group(&someip_routing_group)?;

        let someip_socket_connection_bundle =
            eth_channel.create_socket_connection_bundle("ConnectionBundle", &socket_address_udp_ecu_a)?;
        let connection = someip_socket_connection_bundle.create_bundled_connection(&socket_address_udp_remote)?;
        connection.set_client_ip_addr_from_connection_request(Some(true))?;
        connection.set_client_port_from_connection_request(Some(true))?;
        connection.set_runtime_address_configuration(true)?;

        // note: for SomeIp the service id forms the upper 16 bits of the PDU header id
        // the lower 16 bits are the method id (0x0 - 0x7fff) or event id (0x8000 - 0xffff)
        let method_id = 0x8001u32;
        let pdu_header_id = u32::from(service_identifier) << 16 | method_id;
        let pdu_triggering =
            connection.trigger_pdu(&someip_pdu, pdu_header_id, None, Some(PduCollectionTrigger::Always))?;
        connection.add_routing_group(&pdu_triggering, &someip_routing_group)?;

        // configure service discovery parameters for the SomeIp connection
        let sd_config = SdConfig {
            service_major_version: 1,
            service_minor_version: 0,
            initial_delay_max_value: 0.25,
            initial_delay_min_value: 0.1,
            initial_repetitions_base_delay: Some(0.1),
            initial_repetitions_max: 5,
            offer_cyclic_delay: Some(0.5),
            request_response_delay_max_value: 0.25,
            request_response_delay_min_value: 0.1,
            ttl: 10,
        };
        psi_ecu_a.set_sd_server_config(&sd_config)?;
        csi_remote.set_sd_client_config(&sd_config)?;

        // set SD-specific parameters for the service events
        psi_ecu_a_eh.set_sd_server_config(0.1, 0.25, 10)?;
        csi_remote_ceg.set_sd_client_config(0.1, 0.25, 10)?;

        println!("{}", model.files().next().unwrap().serialize()?);
        model.write()?;

        Ok(())
    }
}
