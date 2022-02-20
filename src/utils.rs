//! Various functions, macros, types etc to make your life easier

pub mod macros {
    #[macro_export]
    macro_rules! on_option {
        ( $e:expr, $err_exp:expr ) => {
            match $e {
                Some(x) => x,
                None => { $err_exp },
            }
        }
    }

    /// Unwrap value. Return ```Ok(x)``` or performs action on error
    ///
    /// Variable ```err``` represents error case
    ///
    /// Example
    /// ```not_run
    /// let x = on_error!(f(x), return Err(err));
    ///
    /// let x = match f(x) {
    ///    Ok(x) => x,
    ///    Err(err) => { return Err(err) },
    /// };
    /// ```
    #[macro_export]
    macro_rules! on_error {
        ( $e:expr, $err_exp:expr ) => {
            match $e {
                Ok(x) => x,
                #[allow(unused_variables)]
                Err(err) => { $err_exp },
            }
        }
    }
}

///
pub mod filters {
    use crate::hardware::{
        HWDescription,
        QueueFamilyDescription,
        MemoryDescription,
    };

    /// Aggregate information about selected hardware
    #[derive(Debug, Clone, Copy)]
    pub struct HWIndex {
        /// Device index from collection of HW devices
        pub device: usize,
        /// Queue family index
        pub queue:  usize,
        /// Memory description index
        pub memory: usize
    }

    impl HWIndex {
        fn new(dev: usize, queue: usize, mem: usize) -> HWIndex {
            HWIndex {
                device: dev,
                queue: queue,
                memory: mem
            }
        }
    }

    pub fn is_compute_family(desc: &QueueFamilyDescription) -> bool {
        desc.is_compute() && desc.is_transfer()
    }

    pub fn any_hw(_: &HWDescription) -> bool {
        true
    }

    pub fn any_queue(_: &QueueFamilyDescription) -> bool {
        true
    }

    pub fn any_memory(_: &MemoryDescription) -> bool {
        true
    }

    /// Return first suitable index defined by predicates
    pub fn select_hw<'a, I, P, U, S>(descs: I, hw_p: P, q_p: U, m_p: S) -> Option<HWIndex>
    where
        I: Iterator<Item = &'a HWDescription>,
        P: Fn(&HWDescription) -> bool,
        U: Fn(&QueueFamilyDescription) -> bool,
        S: Fn(&MemoryDescription) -> bool,
    {
        let q_wrapper = |(i, q_desc): (usize, &QueueFamilyDescription)| -> Option<usize> {
            if q_p(q_desc) {
                Some(i)
            }
            else {
                None
            }
        };

        let m_wrapper = |(i, m_desc): (usize, &MemoryDescription)| -> Option<usize> {
            if m_p(m_desc) {
                Some(i)
            }
            else {
                None
            }
        };

        let hw_search = |(i, hw_desc): (usize, &HWDescription)| -> Option<HWIndex> {
            let q_res = hw_desc.queues.iter().enumerate().find_map(q_wrapper);
            let m_res = hw_desc.memory_info.iter().enumerate().find_map(m_wrapper);

            if hw_p(hw_desc) && q_res.is_some() && m_res.is_some() {
                Some(HWIndex::new(i, q_res.unwrap(), m_res.unwrap()))
            }
            else {
                None
            }
        };

        descs.enumerate().find_map(hw_search)
    }
}