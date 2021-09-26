use std::iter::repeat_with;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use wasmtime::{AsContext, Memory, StoreContext, Trap};

use crate::interop::{FromWasm, ToWasm, Wasm};
use crate::util::{get_slice, get_str};

pub struct RpcServerParams {
    pub service_name: String,
    pub methods: Vec<RpcServerMethodParams>,
}

impl Wasm for RpcServerParams {
    const SIZE: u32 = 16;
}

impl FromWasm for RpcServerParams {
    fn from_wasm<T>(
        context: StoreContext<T>,
        memory: Memory,
        data: &mut &[u8],
    ) -> Result<Self, Trap> {
        let service_name_ptr = data.read_u32::<LittleEndian>().unwrap();
        let service_name_len = data.read_u32::<LittleEndian>().unwrap();
        let methods_ptr = data.read_u32::<LittleEndian>().unwrap();
        let methods_len = data.read_u32::<LittleEndian>().unwrap();

        let service_name = get_str(
            context.as_context(),
            memory,
            service_name_ptr,
            service_name_len,
        )?
        .to_owned();
        let mut methods_data = get_slice(
            context.as_context(),
            memory,
            methods_ptr,
            RpcServerMethodParams::SIZE * methods_len,
        )?;
        let methods = repeat_with(|| {
            RpcServerMethodParams::from_wasm(context.as_context(), memory, &mut methods_data)
        })
        .take(methods_len as _)
        .collect::<Result<_, _>>()?;

        Ok(Self {
            service_name,
            methods,
        })
    }
}

pub struct RpcServerMethodParams {
    pub method_name: String,
}

impl Wasm for RpcServerMethodParams {
    const SIZE: u32 = 8;
}

impl FromWasm for RpcServerMethodParams {
    fn from_wasm<T>(
        context: StoreContext<T>,
        memory: Memory,
        data: &mut &[u8],
    ) -> Result<Self, Trap> {
        let method_name_ptr = data.read_u32::<LittleEndian>().unwrap();
        let method_name_len = data.read_u32::<LittleEndian>().unwrap();

        let method_name = get_str(context, memory, method_name_ptr, method_name_len)?.to_owned();

        Ok(Self { method_name })
    }
}

pub struct RpcMetadata {
    pub method_index: u32,
    pub request_io: u32,
    pub response_io: u32,
}

impl Wasm for RpcMetadata {
    const SIZE: u32 = 12;
}

impl FromWasm for RpcMetadata {
    fn from_wasm<T>(
        _context: StoreContext<T>,
        _memory: Memory,
        data: &mut &[u8],
    ) -> Result<Self, Trap> {
        let index = data.read_u32::<LittleEndian>().unwrap();
        let request_io = data.read_u32::<LittleEndian>().unwrap();
        let response_io = data.read_u32::<LittleEndian>().unwrap();

        Ok(Self {
            method_index: index,
            request_io,
            response_io,
        })
    }
}

impl ToWasm for RpcMetadata {
    fn to_wasm(&self, data: &mut &mut [u8]) -> Result<(), Trap> {
        data.write_u32::<LittleEndian>(self.method_index).unwrap();
        data.write_u32::<LittleEndian>(self.request_io).unwrap();
        data.write_u32::<LittleEndian>(self.response_io).unwrap();
        Ok(())
    }
}
