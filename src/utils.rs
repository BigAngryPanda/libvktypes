pub mod macros {
    /*
        example
        let entry:Entry = unwrap_result_or_error!(unsafe { Entry::new() }, VkEnvironmentError::LibraryLoad);

        let entry:Entry = match unsafe { Entry::new() } {
                Ok(val) => val,
                Err(..) => return Err(VkEnvironmentError::LibraryLoad),
            }
        };
    */
    #[macro_export]
    macro_rules! unwrap_result_or_error {
        ( $e:expr, $err_val:expr ) => {
            match $e {
                Ok(x) => x,
                Err(_) => return Err($err_val),
            }
        }
    }

    #[macro_export]
    macro_rules! unwrap_result_or_none {
        ( $e:expr ) => {
            match $e {
                Ok(x) => x,
                Err(_) => return None,
            }
        }
    }

    #[macro_export]
    macro_rules! unwrap_option_or_error {
        ( $e:expr, $err_val:expr ) => {
            match $e {
                Some(x) => x,
                None => return Err($err_val),
            }
        }
    }   
}

pub mod filters {
    use crate::hardware::{
        HWDescription,
        QueueFamilyDescription,
        MemoryDescription,
    };

    pub fn is_compute_family(desc: &QueueFamilyDescription) -> bool {
        desc.support_compute && desc.support_transfer
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

    // Return physical_device index, queue index, memory index
    pub fn select_hw<'a, I, P, U, S>(descs: I, hw_p: P, q_p: U, m_p: S) -> Option<(usize, usize, usize)> 
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

        let hw_wrapper = |(i, hw_desc): (usize, &HWDescription)| -> Option<(usize, usize, usize)> {
            let q_res = hw_desc.queues.iter().enumerate().find_map(q_wrapper);
            let m_res = hw_desc.memory_types.iter().enumerate().find_map(m_wrapper);

            if hw_p(hw_desc) && q_res.is_some() && m_res.is_some() {
                Some((i, q_res.unwrap(), m_res.unwrap()))
            }
            else {
                None
            }
        };

        descs.enumerate().find_map(hw_wrapper)
    }
}