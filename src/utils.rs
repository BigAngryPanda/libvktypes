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
    };

    use std::iter::Enumerate;

    fn is_compute_family(desc: &QueueFamilyDescription) -> bool {
        desc.support_compute && desc.support_transfer
    }

    pub fn find_compute_hw_device<'a, I>(descs: I) -> Option<(usize, usize)>
    where
        I: Iterator<Item = &'a HWDescription>,
    {
        HWDescription::find(descs, HWDescription::get_queues, is_compute_family)
    }
}