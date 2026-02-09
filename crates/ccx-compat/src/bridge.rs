use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::symbols::{LegacyLanguage, canonical_symbol, fortran_symbol};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallingConvention {
    C,
    Fortran,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutineSpec {
    pub symbol: String,
    pub convention: CallingConvention,
    pub expected_args: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutineHandle {
    pub symbol: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompatError {
    RoutineNotRegistered {
        symbol: String,
    },
    InvalidArgumentCount {
        symbol: String,
        expected: usize,
        got: usize,
    },
    InvocationFailed {
        symbol: String,
        message: String,
    },
}

impl Display for CompatError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CompatError::RoutineNotRegistered { symbol } => {
                write!(f, "routine not registered: {symbol}")
            }
            CompatError::InvalidArgumentCount {
                symbol,
                expected,
                got,
            } => write!(
                f,
                "invalid argument count for {symbol}: expected {expected}, got {got}"
            ),
            CompatError::InvocationFailed { symbol, message } => {
                write!(f, "routine invocation failed for {symbol}: {message}")
            }
        }
    }
}

impl std::error::Error for CompatError {}

pub type ScalarRoutine = Arc<dyn Fn(&[f64]) -> Result<f64, CompatError> + Send + Sync + 'static>;

#[derive(Default)]
pub struct CompatRegistry {
    routines: BTreeMap<String, (RoutineSpec, ScalarRoutine)>,
}

impl CompatRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_c(
        &mut self,
        symbol: &str,
        expected_args: usize,
        routine: ScalarRoutine,
    ) -> RoutineHandle {
        self.register_internal(symbol, CallingConvention::C, expected_args, routine)
    }

    pub fn register_fortran(
        &mut self,
        symbol: &str,
        expected_args: usize,
        routine: ScalarRoutine,
    ) -> RoutineHandle {
        self.register_internal(symbol, CallingConvention::Fortran, expected_args, routine)
    }

    pub fn spec(&self, symbol: &str) -> Option<&RoutineSpec> {
        self.routines.get(symbol).map(|entry| &entry.0)
    }

    pub fn call(&self, symbol: &str, args: &[f64]) -> Result<f64, CompatError> {
        let resolved =
            self.resolve_symbol(symbol)
                .ok_or_else(|| CompatError::RoutineNotRegistered {
                    symbol: symbol.to_string(),
                })?;

        let (spec, routine) = self
            .routines
            .get(&resolved)
            .expect("resolved symbol must exist");
        if args.len() != spec.expected_args {
            return Err(CompatError::InvalidArgumentCount {
                symbol: resolved,
                expected: spec.expected_args,
                got: args.len(),
            });
        }
        routine(args)
    }

    fn register_internal(
        &mut self,
        symbol: &str,
        convention: CallingConvention,
        expected_args: usize,
        routine: ScalarRoutine,
    ) -> RoutineHandle {
        let language = match convention {
            CallingConvention::C => LegacyLanguage::C,
            CallingConvention::Fortran => LegacyLanguage::Fortran,
        };
        let canonical = canonical_symbol(symbol, language);
        let spec = RoutineSpec {
            symbol: canonical.clone(),
            convention,
            expected_args,
        };
        self.routines.insert(canonical.clone(), (spec, routine));
        RoutineHandle { symbol: canonical }
    }

    fn resolve_symbol(&self, symbol: &str) -> Option<String> {
        if self.routines.contains_key(symbol) {
            return Some(symbol.to_string());
        }

        let c = canonical_symbol(symbol, LegacyLanguage::C);
        if self.routines.contains_key(&c) {
            return Some(c);
        }

        let f = fortran_symbol(symbol);
        if self.routines.contains_key(&f) {
            return Some(f);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_and_calls_c_routine() {
        let mut registry = CompatRegistry::new();
        registry.register_c(
            "compare",
            2,
            Arc::new(|args| Ok(if args[0] == args[1] { 1.0 } else { 0.0 })),
        );

        let out = registry
            .call("compare", &[3.0, 3.0])
            .expect("call should succeed");
        assert_eq!(out, 1.0);
    }

    #[test]
    fn resolves_fortran_symbol_from_base_name() {
        let mut registry = CompatRegistry::new();
        registry.register_fortran("NIDENT2", 1, Arc::new(|args| Ok(args[0].round())));

        let out = registry
            .call("nident2", &[4.6])
            .expect("fortran symbol should resolve");
        assert_eq!(out, 5.0);
    }

    #[test]
    fn validates_argument_count() {
        let mut registry = CompatRegistry::new();
        registry.register_c("stoi", 3, Arc::new(|_| Ok(42.0)));

        let err = registry
            .call("stoi", &[1.0, 2.0])
            .expect_err("invalid arg count should fail");
        assert_eq!(
            err,
            CompatError::InvalidArgumentCount {
                symbol: "stoi".to_string(),
                expected: 3,
                got: 2
            }
        );
    }

    #[test]
    fn returns_not_registered_error() {
        let registry = CompatRegistry::new();
        let err = registry
            .call("missing_symbol", &[])
            .expect_err("missing routine should fail");
        assert_eq!(
            err,
            CompatError::RoutineNotRegistered {
                symbol: "missing_symbol".to_string()
            }
        );
    }

    #[test]
    fn exposes_registered_specs() {
        let mut registry = CompatRegistry::new();
        registry.register_fortran("calc", 2, Arc::new(|args| Ok(args[0] + args[1])));
        let spec = registry.spec("calc_").expect("spec should exist");
        assert_eq!(spec.symbol, "calc_");
        assert_eq!(spec.expected_args, 2);
        assert_eq!(spec.convention, CallingConvention::Fortran);
    }
}
