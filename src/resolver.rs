use std::collections::HashMap;

pub trait SymbolResolver {
    fn resolve(&mut self, name: &str) -> Option<u64>;
}

#[derive(Default, Debug, Clone, Copy)]
pub struct NoSymbolResolver;

impl SymbolResolver for NoSymbolResolver {
    fn resolve(&mut self, _name: &str) -> Option<u64> {
        None
    }
}

#[derive(Default, Debug, Clone)]
#[repr(transparent)]
pub struct HashMapSymbolResolver(HashMap<String, u64>);

#[macro_export]
macro_rules! symbols {
    (map $(($symbol:expr, $addr:expr)),+) => {
        {
            let mut res = $crate::resolver::HashMapSymbolResolver::new();
            $( res.insert($symbol, $addr); )+
            res
        }
    };
    ($(($symbol:expr, $addr:expr)),+) => {
        { symbols!(map $(($symbol, $addr)),+) }
    };
    () => {
        { $crate::resolver::HashMapSymbolResolver::new() }
    }
}

impl HashMapSymbolResolver {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn insert(&mut self, name: impl Into<String>, addr: u64) {
        self.0.insert(name.into(), addr);
    }
}

impl SymbolResolver for HashMapSymbolResolver {
    fn resolve(&mut self, name: &str) -> Option<u64> {
        self.0.get(name).copied()
    }
}

pub struct FnSymbolResolver<F> {
    f: F,
}

impl<F> FnSymbolResolver<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F> SymbolResolver for FnSymbolResolver<F>
where
    F: FnMut(&str) -> Option<u64>,
{
    fn resolve(&mut self, name: &str) -> Option<u64> {
        (self.f)(name)
    }
}
