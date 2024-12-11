use bobry_w_locie::drone::BoberDrone;
use wg_2024::tests::{generic_chain_fragment_ack, generic_chain_fragment_drop, generic_fragment_drop, generic_fragment_forward};

#[cfg(test)]
#[test]
fn generic_fragment_forward_test() {
    generic_fragment_forward::<BoberDrone>();
}
#[cfg(test)]
#[test]
fn generic_fragment_drop_test() {
    generic_fragment_drop::<BoberDrone>();
}
#[cfg(test)]
#[test]
fn generic_chain_fragment_drop_test() {
    generic_chain_fragment_drop::<BoberDrone>();
}
#[cfg(test)]
#[test]
fn generic_chain_fragment_ack_test() {
    generic_chain_fragment_ack::<BoberDrone>();
}
