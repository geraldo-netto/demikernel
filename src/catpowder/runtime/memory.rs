// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//==============================================================================
// Imports
//==============================================================================

use super::LinuxRuntime;
use ::runtime::{
    fail::Fail,
    memory::{
        Bytes,
        BytesMut,
        MemoryRuntime,
    },
    types::{
        dmtr_sgarray_t,
        dmtr_sgaseg_t,
    },
};
use ::std::{
    mem,
    ptr,
    slice,
};

//==============================================================================
// Trait Implementations
//==============================================================================

/// Memory Runtime Trait Implementation for Linux Runtime
impl MemoryRuntime for LinuxRuntime {
    /// Memory Buffer
    type Buf = Bytes;

    /// Creates a [dmtr_sgarray_t] from a memory buffer.
    fn into_sgarray(&self, buf: Bytes) -> Result<dmtr_sgarray_t, Fail> {
        let buf_copy: Box<[u8]> = (&buf[..]).into();
        let ptr: *mut [u8] = Box::into_raw(buf_copy);
        let sgaseg = dmtr_sgaseg_t {
            sgaseg_buf: ptr as *mut _,
            sgaseg_len: buf.len() as u32,
        };
        Ok(dmtr_sgarray_t {
            sga_buf: ptr::null_mut(),
            sga_numsegs: 1,
            sga_segs: [sgaseg],
            sga_addr: unsafe { mem::zeroed() },
        })
    }

    /// Allocates a [dmtr_sgarray_t].
    fn alloc_sgarray(&self, size: usize) -> Result<dmtr_sgarray_t, Fail> {
        let allocation: Box<[u8]> = unsafe { Box::new_uninit_slice(size).assume_init() };
        let ptr: *mut [u8] = Box::into_raw(allocation);
        let sgaseg = dmtr_sgaseg_t {
            sgaseg_buf: ptr as *mut _,
            sgaseg_len: size as u32,
        };
        Ok(dmtr_sgarray_t {
            sga_buf: ptr::null_mut(),
            sga_numsegs: 1,
            sga_segs: [sgaseg],
            sga_addr: unsafe { mem::zeroed() },
        })
    }

    /// Releases a [dmtr_sgarray_t].
    fn free_sgarray(&self, sga: dmtr_sgarray_t) -> Result<(), Fail> {
        assert_eq!(sga.sga_numsegs, 1);
        for i in 0..sga.sga_numsegs as usize {
            let seg: &dmtr_sgaseg_t = &sga.sga_segs[i];
            let allocation: Box<[u8]> = unsafe {
                Box::from_raw(slice::from_raw_parts_mut(
                    seg.sgaseg_buf as *mut _,
                    seg.sgaseg_len as usize,
                ))
            };
            drop(allocation);
        }

        Ok(())
    }

    /// Clones a [dmtr_sgarray_t] into a memory buffer.
    fn clone_sgarray(&self, sga: &dmtr_sgarray_t) -> Result<Bytes, Fail> {
        let mut len: u32 = 0;
        for i in 0..sga.sga_numsegs as usize {
            len += sga.sga_segs[i].sgaseg_len;
        }
        let mut buf: BytesMut = BytesMut::zeroed(len as usize).unwrap();
        let mut pos: usize = 0;
        for i in 0..sga.sga_numsegs as usize {
            let seg: &dmtr_sgaseg_t = &sga.sga_segs[i];
            let seg_slice = unsafe {
                slice::from_raw_parts(seg.sgaseg_buf as *mut u8, seg.sgaseg_len as usize)
            };
            buf[pos..(pos + seg_slice.len())].copy_from_slice(seg_slice);
            pos += seg_slice.len();
        }
        Ok(buf.freeze())
    }
}
