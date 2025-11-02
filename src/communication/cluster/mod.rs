use crate::{AbstractionElement, AutosarAbstractionError, IdentifiableAbstractionElement, System};
use autosar_data::{Element, ElementName};

mod can;
mod ethernet;
mod flexray;
mod lin;

pub use can::*;
pub use ethernet::*;
pub use flexray::*;
pub use lin::*;

//##################################################################

/// [`AbstractCluster`] defines the common interface for all supported communication clusters.
pub trait AbstractCluster: AbstractionElement {
    /// Returns the [`System`] the cluster is part of.
    fn system(&self) -> Option<System> {
        if let Ok(model) = self.element().model() {
            let path = self.element().path().ok()?;
            let refs = model.get_references_to(&path);

            if let Some(system) = refs
                .iter()
                .filter_map(autosar_data::WeakElement::upgrade)
                .filter(|elem| elem.element_name() == ElementName::FibexElementRef)
                .filter_map(|elem| elem.named_parent().ok().flatten())
                .find_map(|parent| System::try_from(parent).ok())
            {
                return Some(system);
            }
        }
        None
    }
}

//##################################################################

/// A [`Cluster`] is returned by [`System::clusters`].
/// It can contain any supported communication cluster.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Cluster {
    /// The Cluster is a [`CanCluster`]
    Can(CanCluster),
    /// The Cluster is an [`EthernetCluster`]
    Ethernet(EthernetCluster),
    /// The Cluster is a [`FlexrayCluster`]
    FlexRay(FlexrayCluster),
    /// The Cluster is a [`LinCluster`]
    Lin(LinCluster),
    // missing: TTCAN, J1939, CDD (aka user defined)
}

impl AbstractionElement for Cluster {
    fn element(&self) -> &autosar_data::Element {
        match self {
            Cluster::Can(cancluster) => cancluster.element(),
            Cluster::Ethernet(ethcluster) => ethcluster.element(),
            Cluster::FlexRay(flxcluster) => flxcluster.element(),
            Cluster::Lin(lincluster) => lincluster.element(),
        }
    }
}

impl IdentifiableAbstractionElement for Cluster {}
impl AbstractCluster for Cluster {}

impl TryFrom<Element> for Cluster {
    type Error = AutosarAbstractionError;

    fn try_from(element: Element) -> Result<Self, Self::Error> {
        match element.element_name() {
            ElementName::CanCluster => Ok(CanCluster::try_from(element)?.into()),
            ElementName::EthernetCluster => Ok(EthernetCluster::try_from(element)?.into()),
            ElementName::FlexrayCluster => Ok(FlexrayCluster::try_from(element)?.into()),
            ElementName::LinCluster => Ok(LinCluster::try_from(element)?.into()),
            _ => Err(AutosarAbstractionError::ConversionError {
                element,
                dest: "Cluster".to_string(),
            }),
        }
    }
}

impl From<CanCluster> for Cluster {
    fn from(value: CanCluster) -> Self {
        Cluster::Can(value)
    }
}

impl From<EthernetCluster> for Cluster {
    fn from(value: EthernetCluster) -> Self {
        Cluster::Ethernet(value)
    }
}

impl From<FlexrayCluster> for Cluster {
    fn from(value: FlexrayCluster) -> Self {
        Cluster::FlexRay(value)
    }
}

impl From<LinCluster> for Cluster {
    fn from(value: LinCluster) -> Self {
        Cluster::Lin(value)
    }
}

//##################################################################

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AutosarModelAbstraction;
    use autosar_data::AutosarVersion;

    #[test]
    fn cluster_system() {
        let model = AutosarModelAbstraction::create("test.arxml", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/Test").unwrap();
        let system = package
            .create_system("System", crate::SystemCategory::EcuExtract)
            .unwrap();
        let can_cluster = CanCluster::new("CanCluster", &package, None).unwrap();

        assert!(can_cluster.system().is_none());
        system.create_fibex_element_ref(can_cluster.element()).unwrap();
        assert_eq!(can_cluster.system().unwrap(), system);
    }

    #[test]
    fn cluster_conversion() {
        let model = AutosarModelAbstraction::create("test.arxml", AutosarVersion::LATEST);
        let package = model.get_or_create_package("/Test").unwrap();
        let can_cluster = CanCluster::new("CanCluster", &package, None).unwrap();
        let ethernet_cluster = EthernetCluster::new("EthernetCluster", &package).unwrap();
        let flexray_settings = FlexrayClusterSettings::default();
        let flexray_cluster = FlexrayCluster::new("FlexrayCluster", &package, &flexray_settings).unwrap();

        let can: Cluster = can_cluster.into();
        let ethernet: Cluster = ethernet_cluster.into();
        let flexray: Cluster = flexray_cluster.into();

        assert_eq!(can.element().item_name().unwrap(), "CanCluster");
        assert_eq!(ethernet.element().item_name().unwrap(), "EthernetCluster");
        assert_eq!(flexray.element().item_name().unwrap(), "FlexrayCluster");
    }
}
