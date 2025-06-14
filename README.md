# autosar-data-abstraction

[![Github Actions](https://github.com/DanielT/autosar-data-abstraction/actions/workflows/CI.yml/badge.svg)](https://github.com/DanielT/autosar-data-abstraction/actions)

This crate provides an abstraction layer for the AUTOSAR data model.
It is built on top of the crate `autosar-data` and provides complex interactions with
the model on top of the elementary operations of `autosar-data`.

Rather than transforming the element based model into a new form, it only presents a
view into the existing model, and provides methods to retrieve and modify the data.

Since the AUTOSAR data model is very complex and has many different types of elements,
this crate does not aim to provide full coverage of all classes.
Instead the focus is on the most common classes and their interactions.

Any other data can still be accessed through the basic operations of `autosar-data`, because the
calls to `autosar-data` and `autosar-data-abstraction` can be mixed freely.

## Features

Autosar Classic Platform:
- Communication:
  - Busses
    - CAN
    - Ethernet (both old and new style)
    - FlexRay
    - not supported: LIN, J1939
  - PDUs
  - Signals
  - Transformations: SomeIp, E2E, Com
- Data Types
  - Basic data types
  - Implementation data types
  - Application data types
- Software Components
  - Atomic SWCs, Compositions, etc.
  - Interfaces
  - Ports
  - Internal behavior: Runnables, Events, etc.
- ECU Configuration

## Example

```rust
# use autosar_data::*;
# use autosar_data_abstraction::*;
# use autosar_data_abstraction::communication::*;
# fn main() -> Result<(), AutosarAbstractionError> {
let model = AutosarModelAbstraction::create("file.arxml", AutosarVersion::Autosar_00049);
let package_1 = model.get_or_create_package("/System")?;
let system = package_1.create_system("System", SystemCategory::SystemExtract)?;
let package_2 = model.get_or_create_package("/Clusters")?;

// create an Ethernet cluster and a physical channel for VLAN 33
let eth_cluster = system.create_ethernet_cluster("EthCluster", &package_2)?;
let vlan_info = EthernetVlanInfo {
    vlan_id: 33,
    vlan_name: "VLAN_33".to_string(),
};
let eth_channel = eth_cluster.create_physical_channel("EthChannel", Some(&vlan_info))?;
let vlan_info_2 = eth_channel.vlan_info().unwrap();

// create an ECU instance and connect it to the Ethernet channel
let package_3 = model.get_or_create_package("/Ecus")?;
let ecu_instance_a = system.create_ecu_instance("Ecu_A", &package_3)?;
let ethctrl = ecu_instance_a
    .create_ethernet_communication_controller(
        "EthernetController",
        Some("ab:cd:ef:01:02:03".to_string())
    )?;
let channels_iter = ethctrl.connected_channels();
ethctrl.connect_physical_channel("Ecu_A_connector", &eth_channel)?;
let channels_iter = ethctrl.connected_channels();

// ...
# Ok(())}
```
