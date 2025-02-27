extern crate rand;
use rand::Rng;

extern crate core;
use self::core::generator::leaf::IArgLeaf;
use self::core::generator::serialize::ISerializableArg;

/// arg generator for random data
/// Note : as good practice RndData leaf should be always used with 
///   ``` 
///   if config.rnddata_locked { RndData(..) } else { PatternData(Pattern, ..) } 
///   ```
///   - this is for debuging, if in dump you encounter random data hard to track particular
///   packet/call which caused it, while with pattern will serve imidiatelly as identifier
pub struct RndData {
    size: usize
}

impl RndData {
    pub fn new(size: usize) -> RndData {
        RndData {
            size : size,
        }
    }
}

impl ISerializableArg for RndData { }

impl IArgLeaf for RndData {
    fn size(&self) -> usize { self.size }

    fn name(&self) -> &'static str { "RndData" }

    fn generate_unsafe(&mut self, mem: &mut[u8], _: &[u8]) {
        assert!(mem.len() == self.size);//check in debug is OK
        rand::thread_rng().fill(mem);
    }
}
