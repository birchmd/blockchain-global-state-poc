extern crate wasmi;

use self::wasmi::memory_units::Pages;
use self::wasmi::{
    Error as InterpreterError, Externals, FuncInstance, FuncRef, HostError, ImportsBuilder,
    MemoryInstance, MemoryRef, ModuleImportResolver, ModuleInstance, ModuleRef, RuntimeArgs,
    RuntimeValue, Signature, Trap, ValueType,
};
use super::storage::key::Key;
use super::storage::value::{self, Value};
use super::storage::{Error as StorageError, ExecutionEffect, GlobalState, TrackingCopy};

use std::collections::HashSet;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Interpreter(InterpreterError),
    Storage(StorageError),
    InvalidKeyIndicator(u32),
    InvalidValueIndicator(u32),
    ValueTypeSizeMismatch { value_type: u32, value_size: usize },
    ForgedReference(Key),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<InterpreterError> for Error {
    fn from(e: InterpreterError) -> Self {
        Error::Interpreter(e)
    }
}

impl From<StorageError> for Error {
    fn from(e: StorageError) -> Self {
        Error::Storage(e)
    }
}

impl HostError for Error {}

pub struct Runtime<'a, T: TrackingCopy + 'a> {
    memory: MemoryRef,
    known_urefs: HashSet<Key>,
    state: &'a mut T,
}

impl<'a, T: TrackingCopy + 'a> Runtime<'a, T> {
    fn effect(&self) -> ExecutionEffect {
        self.state.effect()
    }

    fn key_from_mem(&mut self, key_type: u32, key_ptr: u32) -> Result<Key, Error> {
        match key_type {
            0 => {
                let mut buf = [0u8; 20];
                self.memory.get_into(key_ptr, &mut buf[..])?;
                Ok(Key::Account(buf))
            }
            1 => {
                let mut buf = [0u8; 32];
                self.memory.get_into(key_ptr, &mut buf[..])?;
                Ok(Key::Hash(buf))
            }
            2 => {
                let mut buf = [0u8; 32];
                self.memory.get_into(key_ptr, &mut buf[..])?;
                let key = Key::URef(buf);
                if self.known_urefs.contains(&key) {
                    Ok(key)
                } else {
                    Err(Error::ForgedReference(key))
                }
            }
            _ => Err(Error::InvalidKeyIndicator(key_type)),
        }
    }

    fn value_from_mem(
        &mut self,
        value_type: u32,
        value_ptr: u32,
        value_size: usize,
    ) -> Result<Value, Error> {
        match value_type {
            value::INT32_ID => {
                if value_size == 4 {
                    let mut buf = [0u8; 4];
                    self.memory.get_into(value_ptr, &mut buf[..])?;
                    let i: i32 = unsafe { std::mem::transmute(buf) }; //TODO: can this be better?
                    Ok(Value::Int32(i))
                } else {
                    Err(Error::ValueTypeSizeMismatch {
                        value_type,
                        value_size,
                    })
                }
            }
            value::BYTEARRAY_ID => {
                let bytes = self.memory.get(value_ptr, value_size)?;
                Ok(Value::ByteArray(bytes))
            }
            //TODO: add other value subtypes
            _ => Err(Error::InvalidValueIndicator(value_type)),
        }
    }

    pub fn write(&mut self, args: RuntimeArgs) -> Result<(), Trap> {
        //args(0) = key type (must be one of 0, 1, 2)
        //args(1) = pointer to key in wasm memory
        //args(2) = value type (must be one of ...)
        //args(3) = pointer to (serialized) value
        //args(4) = value size (in bytes)
        let key_type: u32 = args.nth_checked(0)?;
        let key_ptr: u32 = args.nth_checked(1)?;
        let key = self.key_from_mem(key_type, key_ptr)?;
        let value_type: u32 = args.nth_checked(2)?;
        let value_ptr: u32 = args.nth_checked(3)?;
        let value_size: u32 = args.nth_checked(4)?;
        let value = self.value_from_mem(value_type, value_ptr, value_size as usize)?;
        self.state.write(key, value).map_err(|e| e.into())
    }

    pub fn read(&mut self, args: RuntimeArgs) -> Result<u32, Trap> {
        //args(0) = key type (must be one of 0, 1, 2)
        //args(1) = pointer to key in wasm memory
        //args(2) = value destination pointer
        let key_type: u32 = args.nth_checked(0)?;
        let key_ptr: u32 = args.nth_checked(1)?;
        let key = self.key_from_mem(key_type, key_ptr)?;
        let value = self.state.read(key)?;
        let value_type = value.indicator();
        let value_ptr: u32 = args.nth_checked(2)?;
        self.memory
            .set(value_ptr, value.serialize())
            .map_err(|e| -> Error { e.into() })?;
        Ok(value_type)
    }
}

//TODO: add other functions
const WRITE_FUNC_INDEX: usize = 0;
const READ_FUNC_INDEX: usize = 1;

impl<'a, T: TrackingCopy + 'a> Externals for Runtime<'a, T> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            WRITE_FUNC_INDEX => {
                let _ = self.write(args)?;
                Ok(None)
            }

            READ_FUNC_INDEX => {
                let vtype = self.read(args)?;
                Ok(Some(vtype.into()))
            }
            _ => panic!("unknown function index"),
        }
    }
}

pub struct RuntimeModuleImportResolver;

impl<'a> ModuleImportResolver for RuntimeModuleImportResolver {
    fn resolve_func(
        &self,
        field_name: &str,
        _signature: &Signature,
    ) -> Result<FuncRef, InterpreterError> {
        let func_ref = match field_name {
            "write" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32; 5][..], None),
                WRITE_FUNC_INDEX,
            ),
            "read" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32; 3][..], Some(ValueType::I32)),
                READ_FUNC_INDEX,
            ),
            _ => {
                return Err(InterpreterError::Function(format!(
                    "host module doesn't export function with name {}",
                    field_name
                )))
            }
        };
        Ok(func_ref)
    }
}

pub fn instantiate(module: wasmi::Module) -> Result<ModuleRef, Error> {
    let mut imports = ImportsBuilder::new();
    imports.push_resolver("env", &RuntimeModuleImportResolver);

    let instance = ModuleInstance::new(&module, &imports)?.assert_no_start();

    Ok(instance)
}

pub fn exec<T: TrackingCopy, G: GlobalState<T>>(
    instance: ModuleRef,
    account_addr: [u8; 20],
    gs: &G,
) -> Result<ExecutionEffect, Error> {
    //Note: each "page" is 64 KiB
    let memory = MemoryInstance::alloc(Pages(4), Some(Pages(4))).unwrap();
    let account = gs.get(&Key::Account(account_addr))?.as_account();
    let mut state = gs.tracking_copy();
    let mut runtime = Runtime {
        memory,
        state: &mut state,
        known_urefs: account.urefs(),
    };
    let _ = instance.invoke_export("call", &[], &mut runtime)?;

    Ok(runtime.effect())
}
