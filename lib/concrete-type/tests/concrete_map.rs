#![allow(unused_crate_dependencies)]

use std::{
    any::type_name,
    cmp::Ordering,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

mod widths {
    pub struct Narrow;
    pub struct Wide;
}

trait Width {
    const BITS: u32;
}

impl Width for widths::Narrow {
    const BITS: u32 = 8;
}

impl Width for widths::Wide {
    const BITS: u32 = 64;
}

mod maps {
    concrete_type::concrete_map! {
        #[concrete(bound(crate::Width + Send + 'static))]
        core::cmp::Ordering => {
            Less => crate::widths::Narrow,
            _ => crate::widths::Wide,
        }

        std::net::IpAddr => {
            V4(_) => crate::widths::Narrow,
            V6(_) => crate::widths::Wide,
        }
    }
}

#[test]
fn concrete_map_maps_listed_variants_and_the_remainder_of_a_foreign_enum() {
    let orderings = [Ordering::Less, Ordering::Equal, Ordering::Greater];

    let bits = orderings.map(|ordering| crate::maps::ordering!(ordering; T => T::BITS));

    assert_eq!(bits, [8, 64, 64]);
}

#[test]
fn concrete_map_binds_a_foreign_variant_config() {
    let addrs = [
        IpAddr::V4(Ipv4Addr::LOCALHOST),
        IpAddr::V6(Ipv6Addr::LOCALHOST),
    ];

    let described = addrs.map(
        |addr| crate::maps::ip_addr!(addr; (T, config) => format!("{} {config}", type_name::<T>())),
    );

    assert_eq!(
        described,
        [
            "concrete_map::widths::Narrow 127.0.0.1",
            "concrete_map::widths::Wide ::1",
        ]
    );
}
