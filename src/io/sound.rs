use crate::interface::Interface;

// other sounds than bleeps unsuported
pub fn bleep<I: Interface>(interface: &mut I, sound: u16) {
    if sound < 3 {
        interface.bleep(sound);
    }
}
