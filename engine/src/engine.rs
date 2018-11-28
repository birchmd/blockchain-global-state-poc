extern crate common;
extern crate storage;
extern crate wasmi;

use self::common::key::Key;
use self::common::memio::MemIO;
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

    fn key_from_mem(&mut self, key_ptr: u32) -> Result<Key, Error> {
        let size = std::mem::size_of::<Key>();
        let bytes = self.memory.get(key_ptr, size)?;
        let key = Key::from_bytes(&bytes);
        match key {
            k @ Key::URef(_) => {
                if self.known_urefs.contains(&k) {
                    Ok(k)
                } else {
                    Err(Error::ForgedReference(k))
                }
            }
            other => Ok(other),
        }
    }

    fn value_from_mem(&mut self, value_ptr: u32) -> Result<Value, Error> {
        let size = std::mem::size_of::<Value>();
        let bytes = self.memory.get(value_ptr, size)?;
        Ok(Value::from_bytes(&bytes))
    }

    pub fn write(&mut self, args: RuntimeArgs) -> Result<(), Trap> {
        //args(0) = pointer to key in wasm memory
        //args(1) = pointer to value
        let key_ptr: u32 = args.nth_checked(0)?;
        let key = self.key_from_mem(key_ptr)?;
        let value_ptr: u32 = args.nth_checked(1)?;
        let value = self.value_from_mem(value_ptr)?;
        self.state.write(key, value).map_err(|e| e.into())
    }

    pub fn read(&mut self, args: RuntimeArgs) -> Result<(), Trap> {
        //args(0) = pointer to key in wasm memory
        //args(1) = value destination pointer
        let key_ptr: u32 = args.nth_checked(0)?;
        let key = self.key_from_mem(key_ptr)?;
        let value = self.state.read(key)?;
        let value_ptr: u32 = args.nth_checked(1)?;
        self.memory
            .set(value_ptr, value.as_bytes())
            .map_err(|e| Error::Interpreter(e).into())
    }

    pub fn new_uref(&mut self, args: RuntimeArgs) -> Result<(), Trap> {
        //args(0) = pointer to key destination in wasm memory
        let key_ptr: u32 = args.nth_checked(0)?;
        let key = self.state.new_uref();
        self.known_urefs.insert(key);
        self.memory
            .set(key_ptr, key.as_bytes())
            .map_err(|e| Error::Interpreter(e).into())
    }
}

//TODO: add other functions
const WRITE_FUNC_INDEX: usize = 0;
const READ_FUNC_INDEX: usize = 1;
const NEW_FUNC_INDEX: usize = 2;

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
                let _ = self.read(args)?;
                Ok(None)
            }

            NEW_FUNC_INDEX => {
                let _ = self.new_uref(args)?;
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
            "write" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32; 2][..], None),
                WRITE_FUNC_INDEX,
            ),
            "read" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32; 2][..], None),
                READ_FUNC_INDEX,
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
