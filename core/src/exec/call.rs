use std::sync::Mutex;
use generator::arg::Arg;
use banana::bananaq;
use super::id::CallTableId;
use super::fd_info::CallInfo;
use config::FZZCONFIG;

lazy_static! {
    /// sync primitive for single threading - POC generation, and Code Coverage gathering
    static ref SYNC: Mutex<CallTableId> = Mutex::new(CallTableId::Id(0));
}

/// will describle (sys)call ( or other mechanism api/io .. )
pub struct Call {
    /// id will be specific per call, unique identifier
    id: CallTableId,
    /// static name of call - for PoC and for debug purposes
    name: &'static str,
    /// extra information comming from call -> namely FD returned is most common case
    einfo: CallInfo,
    /// number of total invoked calls in current thread(fuzzy_obj)
    total: usize,
    /// number of successfull calls executed so far for current thread(state/fuzzyobj)
    success: usize,
    /// defined arguments for this call : holders <- generators
    args: Vec<Arg>,
    /// function which executes particular call / action
    ///
    /// - on generated args
    /// - if and only if all modules will allow it
    ///
    /// #Example
    /// ```
    /// |args| {
    ///     let (fd, args) = args.split_at_mut(1);
    ///     ...
    ///     unsafe { WRITE(
    ///         (*fd[0].load_unsafe::<&mut i32>()).clone(),
    ///         ..
    ///     }}
    /// ```
    ccall: fn(args: &mut[Arg]) -> CallInfo,
}

impl Call {
    /// note : all advanced logic should be in fuzzy_obj.do_fuzz(), not in call implementation
    ///
    /// implies : Call object is just templated builder
    ///
    /// # Example
    /// ```
    /// fn test_callee(a: &mut test_struct, b: &mut u32) -> CallInfo {
    ///     a.a += 1;
    ///     *b += 3;
    ///     true
    /// }
    ///
    /// impl Call {
    ///     pub fn test_call() -> Call {
    ///         Call::new(
    ///             CallTableId::Id(TestCalls::Dummy as u64),
    ///             "test-call",
    ///             vec![
    ///                 Arg::memory_arg(
    ///                     Box::new(ArgComposite::test_arg_ex(mem::size_of::<test_struct>()))),
    ///                 Arg::primitive_arg(
    ///                     Box::new(ArgComposite::test_arg_ex(4)))
    ///             ],
    ///             |args| {
    ///                 if let [a, b] = &args[..] {
    ///                     test_callee(
    ///                         a[0].load_unsafe(),
    ///                         b[0].load_unsafe())
    ///                 }
    ///             })
    ///     }
    /// }
    /// ```
    pub fn new(
        id: CallTableId,
        name: &'static str,
        args: Vec<Arg>,
        ccall: fn(ctx: &mut[Arg]) -> CallInfo
        ) -> Call
    {
        Call {
            id : id,
            name : name,
            einfo : CallInfo::fail(),
            total : 0,
            success : 0,
            args : args,
            ccall : ccall,
        }
    }

    /// trigger particular call
    ///
    /// 1. update all # {total, skiped, success}
    /// 2. prepare all arguments for syscall
    /// 3. (do_call_impl)invoke callbacks to all modules -> forward this job to Banana Internal Manager in fact ..
    /// 4. (do_call_impl)invoke function responsible to invoke targeted call
    /// 5. store results
    pub fn do_call(&mut self, fd: &[u8]) -> bool {
        self.total += 1;

        for arg in self.args.iter_mut() {
            arg.do_generate(fd);
        }

        if !self.do_call_safe() {
            return false
        }

        if self.einfo.success() {
            self.success += 1
        }
        //(self.ret <= self.ok.end && self.ret >= self.ok.start) as usize;//self.ok.contains(self.ret);
        true
    }

/// 1. notify observers and ask for aproval
/// 2. if approved invoke syscall
/// 3. have in mind that in case of single thread approach this need to be locked!
///     - therefore do_call_safe wrapper there..
    fn do_call_impl(&mut self) -> bool {
        if !bananaq::call_notify(self) {
            return false
        }

        self.einfo = (self.ccall)(&mut self.args);
        true
    }
/// do sync in case of single thread config flag set
///
/// - poc creation from fuzzing loops
/// - code coverage ( because we need to repro fuzzed loops to benefit from code coverage .. )
/// - ??
    fn do_call_safe(&mut self) -> bool {
        if !FZZCONFIG.singlethread {
            return self.do_call_impl()
        }
        match SYNC.lock() {
            Ok(mut qcall) => { *qcall = self.id.clone(); self.do_call_impl() },
            Err(pois) => panic!("call-lock is poisoned, by this syscall : {:?}", *pois.into_inner()),
        }
    }

    /// print call to string that way we can reproduce it from PoC ( mini c++ program ) later
    ///
    /// note : this schema is novel fuzzing approach : LOOP + Generation based
    pub fn serialize(&self, fd: &[u8]) -> String {
        (self.name.to_string() + "(void" +
            &self.args
                .iter()
                .enumerate()
                .map(|(ind, arg)| {
                    let mut data = arg.do_serialize(fd);
                    if data[..3].contains("new") {
                        data = String::from("(") +
                            self.name +
                            &ind.to_string() +
                            "*)" + &data
                    }
                    String::from(",\n\t") + &data
                })
                .collect::<String>() +
            ");")
                .replace("void,", "")
    }

    pub fn name(&self) -> &str { self.name }
    pub fn id(&self) -> CallTableId { self.id.clone() }
    pub fn total(&self) -> usize { self.total }
    pub fn success(&self) -> usize { self.success }
    pub fn ok(&self) -> bool { self.einfo.success() }
    pub fn dead(&self) -> bool { self.total > FZZCONFIG.dead_call * (1 + self.success) }//from config!!
    pub fn einfo(&self) -> &[u8] { &self.einfo.extra_info() }

    pub fn neg_ret(&mut self) { self.einfo.negate() }

    pub fn n_args(&self) -> usize { self.args.len() }

    pub fn args_view(&self, ind: usize) -> &Arg { &self.args[ind] }
}

