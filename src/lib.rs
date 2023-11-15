//! A crate with utilities to determine the number of CPUs available on the
//! current system.
//!
//! Sometimes the CPU will exaggerate the number of CPUs it contains, because it can use
//! [processor tricks] to deliver increased performance when there are more threads. This
//! crate provides methods to get both the logical and physical numbers of cores.
//!
//! This information can be used as a guide to how many tasks can be run in parallel.
//! There are many properties of the system architecture that will affect parallelism,
//! for example memory access speeds (for all the caches and RAM) and the physical
//! architecture of the processor, so the number of CPUs should be used as a rough guide
//! only.
//!
//!
//! ## Examples
//!
//! Fetch the number of logical CPUs.
//!
//! ```
//! let cpus = num_cpus::get();
//! ```
//!
//! See [`rayon::Threadpool`] for an example of where the number of CPUs could be
//! used when setting up parallel jobs (Where the threadpool example uses a fixed
//! number 8, it could use the number of CPUs).
//!
//! [processor tricks]: https://en.wikipedia.org/wiki/Simultaneous_multithreading
//! [`rayon::ThreadPool`]: https://docs.rs/rayon/1.*/rayon/struct.ThreadPool.html
#![cfg_attr(test, deny(warnings))]
#![deny(missing_docs)]
#![allow(non_snake_case)]
#![cfg_attr(all(feature = "axstd", not(target_os = "linux")), no_std)]

#[cfg(not(windows))]
extern crate libc;

#[cfg(target_os = "hermit")]
extern crate hermit_abi;

#[cfg(all(target_os = "linux", not(feature = "axstd")))]
mod linux;
#[cfg(all(target_os = "linux", not(feature = "axstd")))]
use linux::{get_num_cpus, get_num_physical_cpus};

#[cfg(not(any(target_os = "linux", feature = "axstd")))]
mod other_os;

#[cfg(not(any(target_os = "linux", feature = "axstd")))]
use other_os::{get_num_cpus, get_num_physical_cpus};

/// Returns the number of available CPUs of the current system.
///
/// This function will get the number of logical cores. Sometimes this is different from the number
/// of physical cores (See [Simultaneous multithreading on Wikipedia][smt]).
///
/// This will always return at least `1`.
///
/// # Examples
///
/// ```
/// let cpus = num_cpus::get();
/// if cpus > 1 {
///     println!("We are on a multicore system with {} CPUs", cpus);
/// } else {
///     println!("We are on a single core system");
/// }
/// ```
///
/// # Note
///
/// This will check [sched affinity] on Linux, showing a lower number of CPUs if the current
/// thread does not have access to all the computer's CPUs.
///
/// This will also check [cgroups], frequently used in containers to constrain CPU usage.
///
/// [smt]: https://en.wikipedia.org/wiki/Simultaneous_multithreading
/// [sched affinity]: http://www.gnu.org/software/libc/manual/html_node/CPU-Affinity.html
/// [cgroups]: https://www.kernel.org/doc/Documentation/cgroup-v1/cgroups.txt
#[inline]
pub fn get() -> usize {
    get_num_cpus()
}

/// Returns the number of physical cores of the current system.
///
/// This will always return at least `1`.
///
/// # Note
///
/// Physical count is supported only on Linux, mac OS and Windows platforms.
/// On other platforms, or if the physical count fails on supported platforms,
/// this function returns the same as [`get()`], which is the number of logical
/// CPUS.
///
/// # Examples
///
/// ```
/// let logical_cpus = num_cpus::get();
/// let physical_cpus = num_cpus::get_physical();
/// if logical_cpus > physical_cpus {
///     println!("We have simultaneous multithreading with about {:.2} \
///               logical cores to 1 physical core.",
///               (logical_cpus as f64) / (physical_cpus as f64));
/// } else if logical_cpus == physical_cpus {
///     println!("Either we don't have simultaneous multithreading, or our \
///               system doesn't support getting the number of physical CPUs.");
/// } else {
///     println!("We have less logical CPUs than physical CPUs, maybe we only have access to \
///               some of the CPUs on our system.");
/// }
/// ```
///
/// [`get()`]: fn.get.html
#[inline]
pub fn get_physical() -> usize {
    get_num_physical_cpus()
}

#[cfg(feature = "axstd")]
#[inline]
fn get_num_cpus() -> usize {
    axconfig::SMP as usize
}

#[cfg(feature = "axstd")]
#[inline]
fn get_num_physical_cpus() -> usize {
    axconfig::SMP as usize
}

#[cfg(test)]
mod tests {
    fn env_var(name: &'static str) -> Option<usize> {
        ::std::env::var(name).ok().map(|val| val.parse().unwrap())
    }

    #[test]
    fn test_get() {
        let num = super::get();
        if let Some(n) = env_var("NUM_CPUS_TEST_GET") {
            assert_eq!(num, n);
        } else {
            assert!(num > 0);
            assert!(num < 236_451);
        }
    }

    #[test]
    fn test_get_physical() {
        let num = super::get_physical();
        if let Some(n) = env_var("NUM_CPUS_TEST_GET_PHYSICAL") {
            assert_eq!(num, n);
        } else {
            assert!(num > 0);
            assert!(num < 236_451);
        }
    }
}
