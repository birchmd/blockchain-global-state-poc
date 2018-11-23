extern crate parity_wasm;
extern crate wasmi;

use super::storage::key::Key;
use super::storage::value::Value;
use super::storage::{Error as StorageError, ExecutionEffect, GlobalState, TrackingCopy};
use std::fmt;
use self::wasmi::{
    Error as InterpreterError, Externals, FuncInstance, FuncRef, HostError, ImportsBuilder,
    ModuleImportResolver, ModuleInstance, ModuleRef, RuntimeArgs, RuntimeValue, Signature, Trap,
    ValueType,
};

#[derive(Debug)]
pub enum Error {
    Interpreter(InterpreterError),
    Storage(StorageError),
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
    account: [u8; 20],
    state: &'a mut T,
}

impl<'a, T: TrackingCopy + 'a> Runtime<'a, T> {
    fn effect(&self) -> ExecutionEffect {
        self.state.effect()
    }
}

//TODO: add other functions
const WRITE_FUNC_INDEX: usize = 0;

impl<'a, T: TrackingCopy + 'a> Externals for Runtime<'a, T> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            WRITE_FUNC_INDEX => {
                let idx: i32 = args.nth(0);
                self.state
                    .write(Key::Account(self.account), Value::Int32(idx))?;
                Ok(None)
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
                Signature::new(&[ValueType::I32][..], None),
                WRITE_FUNC_INDEX,
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
    account: [u8; 20],
    gs: &G,
) -> Result<ExecutionEffect, Error> {
    let mut state = gs.tracking_copy();
    let mut runtime = Runtime {
        account,
        state: &mut state
    };
    let _ = instance.invoke_export("call", &[], &mut runtime)?;

    Ok(runtime.effect())
}
