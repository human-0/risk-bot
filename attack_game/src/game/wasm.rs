use super::mov::PlayerMove;
use risk_shared::map::TerritoryId;

use std::cmp;

#[cfg(target_family = "wasm")]
pub(crate) fn retain_different_origin(moves: &mut Vec<PlayerMove>, origin: TerritoryId) {
    if moves.capacity() - moves.len() < 7 {
        moves.reserve(7)
    }

    use std::arch::wasm32::*;
    let origin = u16::from_ne_bytes([origin as u8, 0xFF]);
    let origin = u16x8_splat(origin);

    let len = moves.len();

    let moves_ptr: *mut u8 = moves.as_mut_ptr().cast();
    let mut bytes_processed = 0;
    let mut write_ptr = moves_ptr;

    let mut new_len = 0;
    while bytes_processed < 2 * len {
        let moves = unsafe { v128_load(moves_ptr.add(bytes_processed).cast()) };

        let remaining = len - bytes_processed / 2;
        let equal = u8x16_eq(moves, origin);
        let equal = u16x8_shl(equal, 8);
        let mask_index = i16x8_bitmask(equal);
        let mask = unsafe { v128_load(MASKS[mask_index as usize].as_ptr().cast()) };
        let result = u8x16_swizzle(moves, mask);

        let bits = (1 << cmp::min(remaining, 8)) - 1;
        let num_retained = (bits as u8 & !mask_index).count_ones() as usize;

        unsafe {
            v128_store(write_ptr.cast(), result);
            write_ptr = write_ptr.add(num_retained * 2);
        }

        bytes_processed += 16;
        new_len += num_retained;
    }

    unsafe {
        assert!(new_len <= len, "{new_len}, {len}");
        moves.set_len(new_len);
    }
}

#[cfg(target_family = "wasm")]
pub(crate) fn retain_different_dest(moves: &mut Vec<PlayerMove>, dest: TerritoryId) {
    if moves.capacity() - moves.len() < 7 {
        moves.reserve(7)
    }

    use std::arch::wasm32::*;
    let origin = u16::from_ne_bytes([0xFF, dest as u8]);
    let origin = u16x8_splat(origin);

    let len = moves.len();

    let moves_ptr: *mut u8 = moves.as_mut_ptr().cast();
    let mut bytes_processed = 0;
    let mut write_ptr = moves_ptr;

    let mut new_len = 0;
    while bytes_processed < 2 * len {
        let moves = unsafe { v128_load(moves_ptr.add(bytes_processed).cast()) };

        let remaining = len - bytes_processed / 2;
        let equal = u8x16_eq(moves, origin);
        let mask_index = i16x8_bitmask(equal);
        let mask = unsafe { v128_load(MASKS[mask_index as usize].as_ptr().cast()) };
        let result = u8x16_swizzle(moves, mask);

        let bits = (1 << cmp::min(remaining, 8)) - 1;
        let num_retained = (bits as u8 & !mask_index).count_ones() as usize;

        unsafe {
            v128_store(write_ptr.cast(), result);
            write_ptr = write_ptr.add(num_retained * 2);
        }

        bytes_processed += 16;
        new_len += num_retained;
    }

    unsafe {
        assert!(new_len <= len, "{new_len}, {len}");
        moves.set_len(new_len);
    }
}

#[cfg(target_family = "wasm")]
pub(crate) fn retain_different_origin_dest(
    moves: &mut Vec<PlayerMove>,
    origin: TerritoryId,
    dest: TerritoryId,
) {
    if moves.capacity() - moves.len() < 7 {
        moves.reserve(7)
    }

    use std::arch::wasm32::*;
    let origin = u16::from_ne_bytes([origin as u8, dest as u8]);
    let origin = u16x8_splat(origin);

    let len = moves.len();

    let moves_ptr: *mut u8 = moves.as_mut_ptr().cast();
    let mut bytes_processed = 0;
    let mut write_ptr = moves_ptr;

    let mut new_len = 0;
    while bytes_processed < 2 * len {
        let moves = unsafe { v128_load(moves_ptr.add(bytes_processed).cast()) };

        let remaining = len - bytes_processed / 2;
        let equal = u8x16_eq(moves, origin);
        let equal = v128_or(equal, u16x8_shl(equal, 8));
        let mask_index = i16x8_bitmask(equal);
        let mask = unsafe { v128_load(MASKS[mask_index as usize].as_ptr().cast()) };
        let result = u8x16_swizzle(moves, mask);

        let bits = (1 << cmp::min(remaining, 8)) - 1;
        let num_retained = (bits as u8 & !mask_index).count_ones() as usize;

        unsafe {
            v128_store(write_ptr.cast(), result);
            write_ptr = write_ptr.add(num_retained * 2);
        }

        bytes_processed += 16;
        new_len += num_retained;
    }

    unsafe {
        assert!(new_len <= len, "{new_len}, {len}");
        moves.set_len(new_len);
    }
}

#[cfg(target_family = "wasm")]
const MASKS: [[u8; 16]; 256] = const {
    let mut masks = [[0; 16]; 256];
    let mut i = 0;
    while i < 256 {
        let mut indices = [0, 1, 2, 3, 4, 5, 6, 7];
        let mut read = 0;
        let mut write = 0;
        while read < 8 {
            if i & (1 << read) == 0 {
                indices[write] = indices[read];
                write += 1;
            }

            read += 1;
        }

        while write < 8 {
            indices[write] = 8;
            write += 1
        }

        let mut j = 0;
        while j < 8 {
            masks[i][2 * j] = 2 * indices[j];
            masks[i][2 * j + 1] = 2 * indices[j] + 1;
            j += 1;
        }

        i += 1;
    }

    masks
};
