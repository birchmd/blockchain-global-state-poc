extern crate common;
extern crate storage;
extern crate wasmi;

use self::common::bytesrepr::{deserialize, BytesRepr, Error as BytesReprError};
use self::common::key::Key;
use self::common::value::Value;
use self::storage::{Error as StorageError, ExecutionEffect, GlobalState, TrackingCopy};
use self::wasmi::memory_units::Pages;
use self::wasmi::{
    Error as InterpreterError, Externals, FuncInstance, FuncRef, HostError, ImportsBuilder,
    MemoryDescriptor, MemoryInstance, MemoryRef, ModuleImportResolver, ModuleInstance, RuntimeArgs,
    RuntimeValue, Signature, Trap, ValueType,
};

use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Interpreter(InterpreterError),
    Storage(StorageError),
    BytesRepr(BytesReprError),
    ValueTypeSizeMismatch { value_type: u32, value_size: usize },
    ForgedReference(Key),
    NoImportedMemory,
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

impl From<BytesReprError> for Error {
    fn from(e: BytesReprError) -> Self {
        Error::BytesRepr(e)
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

    fn key_from_mem(&mut self, key_ptr: u32, key_size: u32) -> Result<Key, Error> {
        let bytes = self.memory.get(key_ptr, key_size as usize)?;
        deserialize(&bytes).map_err(|e| e.into())
    }

    fn value_from_mem(&mut self, value_ptr: u32, value_size: u32) -> Result<Value, Error> {
        let bytes = self.memory.get(value_ptr, value_size as usize)?;
        deserialize(&bytes).map_err(|e| e.into())
    }

    fn kv_from_mem(
        &mut self,
        key_ptr: u32,
        key_size: u32,
        value_ptr: u32,
        value_size: u32,
    ) -> Result<(Key, Value), Error> {
        let key = self.key_from_mem(key_ptr, key_size)?;
        let value = self.value_from_mem(value_ptr, value_size)?;
        Ok((key, value))
    }

    pub fn write(&mut self, args: RuntimeArgs) -> Result<(), Trap> {
        //args(0) = pointer to key in wasm memory
        //args(1) = size of key
        //args(2) = pointer to value
        //args(3) = size of value
        let key_ptr: u32 = args.nth_checked(0)?;
        let key_size: u32 = args.nth_checked(1)?;
        let value_ptr: u32 = args.nth_checked(2)?;
        let value_size: u32 = args.nth_checked(3)?;
        let (key, value) = self.kv_from_mem(key_ptr, key_size, value_ptr, value_size)?;
        self.state.write(key, value).map_err(|e| e.into())
    }

    pub fn add(&mut self, args: RuntimeArgs) -> Result<(), Trap> {
        //args(0) = pointer to key in wasm memory
        //args(1) = size of key
        //args(2) = pointer to value
        //args(3) = size of value
        let key_ptr: u32 = args.nth_checked(0)?;
        let key_size: u32 = args.nth_checked(1)?;
        let value_ptr: u32 = args.nth_checked(2)?;
        let value_size: u32 = args.nth_checked(3)?;
        let (key, value) = self.kv_from_mem(key_ptr, key_size, value_ptr, value_size)?;
        self.state.add(key, value).map_err(|e| e.into())
    }

    fn value_from_key(&mut self, key_ptr: u32, key_size: u32) -> Result<&Value, Trap> {
        let key = self.key_from_mem(key_ptr, key_size)?;
        self.state.read(key).map_err(|e| e.into())
    }

    pub fn size_of_value(&mut self, args: RuntimeArgs) -> Result<usize, Trap> {
        //args(0) = pointer to key in wasm memory
        //args(1) = size of key in wasm memory
        let key_ptr: u32 = args.nth_checked(0)?;
        let key_size: u32 = args.nth_checked(1)?;
        let value = self.value_from_key(key_ptr, key_size)?;
        let value_bytes = value.to_bytes();
        Ok(value_bytes.len())
    }

    pub fn read(&mut self, args: RuntimeArgs) -> Result<(), Trap> {
        //args(0) = pointer to key in wasm memory
        //args(1) = size of key in wasm memory
        //args(2) = value destination pointer
        //args(3) = size of value
        let key_ptr: u32 = args.nth_checked(0)?;
        let key_size: u32 = args.nth_checked(1)?;
        let value_bytes = {
            let value = self.value_from_key(key_ptr, key_size)?;
            value.to_bytes()
        };
        let value_ptr: u32 = args.nth_checked(2)?;
        self.memory
            .set(value_ptr, &value_bytes)
            .map_err(|e| Error::Interpreter(e).into())
    }

    pub fn new_uref(&mut self, args: RuntimeArgs) -> Result<(), Trap> {
        //args(0) = pointer to key destination in wasm memory
        let key_ptr: u32 = args.nth_checked(0)?;
        let key = self.state.new_uref();
        self.known_urefs.insert(key);
        self.memory
            .set(key_ptr, &key.to_bytes())
            .map_err(|e| Error::Interpreter(e).into())
    }
}

//TODO: add other functions
const WRITE_FUNC_INDEX: usize = 0;
const READ_FUNC_INDEX: usize = 1;
const ADD_FUNC_INDEX: usize = 2;
const NEW_FUNC_INDEX: usize = 3;
const SIZE_FUNC_INDEX: usize = 4;

impl<'a, T: TrackingCopy + 'a> Externals for Runtime<'a, T> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            SIZE_FUNC_INDEX => {
                let size = self.size_of_value(args)?;
                Ok(Some(RuntimeValue::I32(size as i32)))
            }

            WRITE_FUNC_INDEX => {
                let _ = self.write(args)?;
                Ok(None)
            }

            ADD_FUNC_INDEX => {
                let _ = self.add(args)?;
                Ok(None)
            }

            NEW_FUNC_INDEX => {
                let _ = self.new_uref(args)?;
                Ok(None)
            }

            READ_FUNC_INDEX => {
                let _ = self.read(args)?;
                Ok(None)
            }

            _ => panic!("unknown function index"),
        }
    }
}

pub struct RuntimeModuleImportResolver {
    memory: RefCell<Option<MemoryRef>>,
    max_memory: u32,
}

impl RuntimeModuleImportResolver {
    pub fn new() -> RuntimeModuleImportResolver {
        RuntimeModuleImportResolver {
            memory: RefCell::new(None),
            max_memory: 256,
        }
    }

    pub fn mem_ref(&self) -> Result<MemoryRef, Error> {
        let maybe_mem: &Option<MemoryRef> = &self.memory.borrow();
        match maybe_mem {
            Some(mem) => Ok(mem.clone()),
            None => Err(Error::NoImportedMemory),
        }
    }
}

impl<'a> ModuleImportResolver for RuntimeModuleImportResolver {
    fn resolve_func(
        &self,
        field_name: &str,
        _signature: &Signature,
    ) -> Result<FuncRef, InterpreterError> {
        let func_ref = match field_name {
            "size_of_value" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32; 2][..], Some(ValueType::I32)),
                SIZE_FUNC_INDEX,
            ),
            "write" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32; 4][..], None),
                WRITE_FUNC_INDEX,
            ),
            "read" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32; 4][..], None),
                READ_FUNC_INDEX,
            ),
            "add" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32; 4][..], None),
                ADD_FUNC_INDEX,
            ),
            "new_uref" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32; 1][..], None),
                NEW_FUNC_INDEX,
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

    fn resolve_memory(
        &self,
        field_name: &str,
        descriptor: &MemoryDescriptor,
    ) -> Result<MemoryRef, InterpreterError> {
        if field_name == "memory" {
            let effective_max = descriptor.maximum().unwrap_or(self.max_memory + 1);
            if descriptor.initial() > self.max_memory || effective_max > self.max_memory {
                Err(InterpreterError::Instantiation(
                    "Module requested too much memory".to_owned(),
                ))
            } else {
                //Note: each "page" is 64 KiB
                let mem = MemoryInstance::alloc(
                    Pages(descriptor.initial() as usize),
                    descriptor.maximum().map(|x| Pages(x as usize)),
                )?;
                *self.memory.borrow_mut() = Some(mem.clone());
                Ok(mem)
            }
        } else {
            Err(InterpreterError::Instantiation(
                "Memory imported under unknown name".to_owned(),
            ))
        }
    }
}

pub fn exec<T: TrackingCopy, G: GlobalState<T>>(
    module: wasmi::Module,
    account_addr: [u8; 20],
    gs: &G,
) -> Result<ExecutionEffect, Error> {
    let resolver = RuntimeModuleImportResolver::new();
    let mut imports = ImportsBuilder::new();
    imports.push_resolver("env", &resolver);
    let instance = ModuleInstance::new(&module, &imports)?.assert_no_start();

    let memory = resolver.mem_ref()?;
    let account = gs.get(&Key::Account(account_addr))?.as_account();
    let mut state = gs.tracking_copy();
    let mut known_urefs: HashSet<Key> = HashSet::new();
    for r in account.urefs() {
        known_urefs.insert(*r);
    }
    let mut runtime = Runtime {
        memory,
        state: &mut state,
        known_urefs,
    };
    let _ = instance.invoke_export("call", &[], &mut runtime)?;

    Ok(runtime.effect())
}
