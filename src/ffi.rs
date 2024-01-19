use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::ffi::{c_char, CStr, c_int, c_void, c_double, c_uint, CString};
use std::mem;
use std::panic::catch_unwind;
use std::ptr::{null, copy_nonoverlapping};

use crate::shunting::new_context;
use crate::system::{System, SystemBuilder, ConstrainResult};

/// Shorthand for creating an owned string from a C `char *`
unsafe fn new_owned_string(s: *const c_char) -> String 
{
    let c_str = CStr::from_ptr(s);
    String::from_utf8_lossy(c_str.to_bytes()).to_string()
} 

/// Allocates a new `SystemBuilder` object on the Rust side of the FFI and returns a raw pointer to it.
#[no_mangle]
pub extern "C" fn new_system_builder(equation: *const c_char) -> *const c_void
{
    let res = catch_unwind(|| {
        let equation_str = unsafe { new_owned_string(equation) };
        let ctx = new_context();
        let builder = match SystemBuilder::new(&equation_str, ctx)
        {
            Ok(x) => x,
            Err(_) => return null(),
        };

        let layout = Layout::new::<SystemBuilder>();
    
        let p_builder = unsafe { alloc(layout) as *mut SystemBuilder };
        if p_builder.is_null() 
        {
            handle_alloc_error(layout);
        }

        unsafe { copy_nonoverlapping(&builder, p_builder, 1) };
        mem::forget(builder); // leak builder so that foreign function can manage memory

        p_builder
    });

    match res
    {
        Ok(p) => p as *const c_void,
        Err(_) => null(),
    }
}

/// Tries to constrain the system with an equation given as a nul-terminated C `char *`.
/// The returned C `int` value indicates the following:
/// 
/// - `0`: The equation did not further constrain the system and was not added
/// - `1`: The equation further constrained the system and was added successfully
/// - `2`: The equation will over-constrain the system and was not added
/// - `-1`: An error occurred while trying to constrain the system
#[no_mangle]
pub extern "C" fn try_constrain_with(p_builder: *mut c_void, equation: *const c_char) -> c_int
{
    let res = catch_unwind(|| {
        let builder = p_builder as *mut SystemBuilder;
        let equation_str = unsafe { new_owned_string(equation) };
        let constrain_res = unsafe { (*builder).try_constrain_with(&equation_str) };

        match constrain_res
        {
            Ok(ConstrainResult::WillConstrain) => 1,
            Ok(ConstrainResult::WillNotConstrain) => 0,
            Ok(ConstrainResult::WillOverConstrain) => 2,
            Err(_) => -1
        }
    });
    
    res.unwrap_or(-1)
}

/// Tries to check whether the system is constrained or not. The returned C `int` value 
/// indicates the following:
/// - `0`: The system is not fully constrained 
/// - `1`: The system is fully constrained
/// - `-1`: An error occurred while checking the system
#[no_mangle]
pub extern "C" fn is_fully_constrained(p_builder: *mut c_void) -> c_int
{
    let res = catch_unwind(|| {
        unsafe{ (*(p_builder as *mut SystemBuilder)).is_fully_constrained() }
    });

    match res
    {
        Ok(x) => if x { 1 } else { 0 },
        Err(_) => -1,
    }
}

/// Tries to create a system from a `SystemBuilder` located at the given pointer,
/// returning a pointer to the created `System` if successful or `NULL` if not.
#[no_mangle]
pub extern "C" fn build_system(p_builder: *const c_void) -> *const c_void
{
    let res = catch_unwind(|| {
        let builder = unsafe { SystemBuilder::from_raw_pointer(p_builder as *const SystemBuilder) };
        let system = match builder.build_system()
        {
            Some(s) => s,
            None => return null(),
        };

        let layout = Layout::new::<System>();

        let p_system = unsafe { alloc(layout) as *mut System };
        if p_system.is_null() 
        {
            handle_alloc_error(layout);
        }

        unsafe { copy_nonoverlapping(&system, p_system, 1) };
        mem::forget(system); // leak system so that foreign function can manage memory

        p_system
    });

    match res 
    {
        Ok(p) => p as *const c_void,
        Err(_) => null(),
    }
}

/// Specifies a guess and domain for a given variable in the `System` at the given pointer.
/// 
/// The returned C `int` value indicates the following:
/// - `1`: The values were specified successfully
/// - `-1`: An error occurred while specifying the domain or guess value 
#[no_mangle]
pub extern "C" fn specify_variable(p_system: *mut c_void, var: *const c_char, guess: c_double, min: c_double, max: c_double) -> c_int
{
    let res = catch_unwind(|| {
        unsafe
        {
            let var_str = new_owned_string(var);
            (*(p_system as *mut System)).specify_variable(&var_str, guess, min, max);
        }
    });

    match res
    {
        Ok(_) => 1,
        Err(_) => -1,
    }
}

/// Tries to solve the system of equations to within the radius `margin` 
/// of the actual solution in `limit` iterations, returning a C `char *` containing the 
/// solution to the system or `NULL` if the solution failed.
#[no_mangle]
pub extern "C" fn solve_system(p_system: *const c_void, margin: c_double, limit: c_uint) -> *const c_char
{
    let res = catch_unwind(|| {
        let system = unsafe { System::from_raw_pointer(p_system as *const System) };

        let soln = match system.solve(margin, limit as usize)
        {
            Ok(s) => s,
            Err(_) => return null() as *const c_char,
        };

        // Create a nul-terminated string with the solution data
        let soln_str: CString = CString::new(
            soln.iter()
                .map(|(var, val)| format!("{}={}", var, val))
                .collect::<Vec<String>>()
                .join("\n")
        ).expect("failed to create C-compatible solution string!");

        soln_str.into_raw()
    });

    match res 
    {
        Ok(s) => s,
        Err(_) => null() as *const c_char,
    }
}

/// Frees a `SystemBuilder` object at the given pointer
#[no_mangle]
pub extern "C" fn free_system_builder(p_builder: *mut c_void)
{
    let layout = Layout::new::<SystemBuilder>();
    unsafe { dealloc(p_builder as *mut u8, layout) };
}

/// Frees a `System` object at the given pointer
#[no_mangle]
pub extern "C" fn free_system(p_builder: *mut c_void)
{
    let layout = Layout::new::<System>();
    unsafe { dealloc(p_builder as *mut u8, layout) };
}

/// Frees the nul-terminated `char *` given
#[no_mangle]
pub unsafe extern "C" fn free_solution_string(soln_str: *mut c_char)
{
    let _owned = CString::from_raw(soln_str);
}