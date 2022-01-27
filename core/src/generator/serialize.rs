use std::collections::HashMap;

/// serialization structure for argument for further POC generation
pub struct SerializationInfo {
    /// offset : where are those data positioned in argument
    ///
    /// - in case of IArg for leafs it is 0, otherwise for IArgComposite can differ
    pub offset: usize,
    /// buffer : final representation of data which can be compiled as part of source code of POC
    pub prefix: String,
}
/// every argument must be serializable in order to reproduce program / crash in POC
pub trait ISerializableArg {
    /// take mem as data buffer of given size, and print it to String (buffer) in a way that it could be compiled later on ( c++ )
    ///
    /// - further deatils check core/generator/{leaf / composite}.rs
    ///
    /// #Example
    /// ```
    /// impl ISerializableArg for TestArg {
    ///     fn serialize(&self, _: &[u8]) -> Vec<SerializationInfo> {
    ///         vec![
    ///             SerializationInfo {
    ///                 offset : 0,
    ///                 prefix : String::from("special("),
    ///             }]
    ///     }
    /// }
    /// ```
    fn serialize(&self, _: &[u8], _: &[u8], _: &[u8]) -> Vec<SerializationInfo> {
        vec![
            SerializationInfo {
                offset : 0,
                prefix : String::from(""),
            }]
    }

    // dump is easy as even in ptr in argument we just fold those data
    fn dump(&self, mem: &[u8]) -> Vec<u8> {
        mem.to_vec()
    }
    // here we push trough composite.rs open-ended mem + data slices, cuze ptr logic
    // we could forward exact memory slice, but we can not easily forward closed data slice
    // because of argument can contains ptr
    // content of data behind ptr is dumped into data slice and ptr leaf should extract
    // thats why we return how much data we used from data slice!
    fn load(&mut self, mem: &mut[u8], dump: &[u8], _data: &[u8], _fd_lookup: &HashMap<Vec<u8>,Vec<u8>>) -> usize {
        mem.copy_from_slice(&dump[..mem.len()]);
        mem.len()
    }
}
