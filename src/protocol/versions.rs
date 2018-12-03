use crate::protocol::*;

mod v1_12_2;
mod v1_11_2;

pub fn translate_internal_packet_id_for_version(version: i32, state: State, dir: Direction, id: i32, to_internal: bool) -> i32 {
    match version {
        340 => v1_12_2::translate_internal_packet_id(state, dir, id, to_internal),
        316 => v1_11_2::translate_internal_packet_id(state, dir, id, to_internal),
        _ => panic!("unsupported protocol version"),
    }
}
