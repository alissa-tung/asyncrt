#![allow(clippy::missing_safety_doc)]
#![feature(strict_provenance)]

use std::ffi::CString;
use std::future::Future;
use std::pin::Pin;
use std::ptr;

use futures::FutureExt;
use tokio::runtime;
use tokio::task::JoinError;

#[no_mangle]
pub extern "C" fn prim__null_ptr() -> AnyPtr {
    ptr::null::<*const libc::c_void>().expose_addr()
}

type AnyPtr = usize;

type Awaitable = Pin<Box<dyn Future<Output = AnyPtr> + Send>>;

#[repr(C)]
pub struct AnyFuture(Awaitable);

fn to_any_future<F>(xs: F) -> *mut AnyFuture
where
    F: Future<Output = AnyPtr> + Send + 'static,
{
    let xs = Box::pin(xs) as Awaitable;
    let xs = Box::new(AnyFuture(xs));
    Box::into_raw(xs)
}

impl Future for AnyFuture {
    type Output = AnyPtr;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.0.poll_unpin(cx)
    }
}

#[no_mangle]
pub unsafe extern "C" fn prim__block_on(xs: *mut AnyFuture) -> AnyPtr {
    let xs = Box::from_raw(xs);
    runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(xs)
}

#[no_mangle]
pub unsafe extern "C" fn prim__spawn(xs: *mut AnyFuture) -> *mut AnyFuture {
    let xs = *Box::from_raw(xs);
    let xs = tokio::spawn(xs);
    to_any_future(async move {
        let xs = to_join_result(xs.await);
        xs.expose_addr()
    })
}

#[repr(C)]
pub enum JoinErrorReason {
    Cancelled = 0,
    Panic = 1,
}

#[repr(C)]
pub struct JoinResult {
    ok: bool,
    addr: usize,
    kind: JoinErrorReason,
    error: *const libc::c_char,
}

fn to_join_result(x: Result<usize, JoinError>) -> *mut JoinResult {
    let x = match x {
        Ok(addr) => JoinResult {
            ok: true,
            addr,
            kind: JoinErrorReason::Cancelled,
            error: ptr::null(),
        },
        Err(err) => JoinResult {
            ok: false,
            addr: 0,
            kind: if err.is_panic() {
                JoinErrorReason::Panic
            } else {
                JoinErrorReason::Cancelled
            },
            error: {
                CString::new(err.to_string())
                    .map(|x| x.into_raw() as *const libc::c_char)
                    .unwrap_or_else(|_| ptr::null())
            },
        },
    };
    Box::into_raw(Box::new(x))
}

#[no_mangle]
pub extern "C" fn addr_to_join_result(x: usize) -> *const JoinResult {
    ptr::from_exposed_addr(x)
}

#[no_mangle]
pub extern "C" fn prim__delay(f: extern "C" fn() -> AnyPtr) -> *mut AnyFuture {
    to_any_future(async move { f() })
}

#[no_mangle]
pub unsafe extern "C" fn prim__any_future__map(
    f: extern "C" fn(AnyPtr) -> AnyPtr,
    xs: *mut AnyFuture,
) -> *mut AnyFuture {
    let xs = Box::from_raw(xs);
    to_any_future(async move {
        let x = xs.await;
        f(x)
    })
}

#[no_mangle]
pub extern "C" fn prim__any_future__pure(x: AnyPtr) -> *mut AnyFuture {
    to_any_future(async move { x })
}

#[no_mangle]
pub unsafe extern "C" fn prim__any_future__bind(
    xs: *mut AnyFuture,
    k: extern "C" fn(AnyPtr) -> *mut AnyFuture,
) -> *mut AnyFuture {
    let xs = Box::from_raw(xs);
    to_any_future(async move {
        let x = xs.await;
        let ys = Box::from_raw(k(x));
        ys.await
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        prim__any_future__bind, prim__any_future__map, prim__any_future__pure, prim__block_on,
        prim__null_ptr, to_any_future, AnyFuture, AnyPtr,
    };

    extern "C" fn incr_usize(x: AnyPtr) -> AnyPtr {
        let x = x as *const usize;
        let x = unsafe { *x };
        let x = Box::new(x + 1);
        Box::into_raw(x) as AnyPtr
    }

    extern "C" fn id_println_usize(x: AnyPtr) -> *mut AnyFuture {
        let x = x as *const usize;
        let x = unsafe { *x };
        to_any_future(async move {
            println!("{x}");
            let x = Box::into_raw(Box::new(x)) as *const libc::c_void;
            x.expose_addr()
        })
    }

    extern "C" fn async_println_usize(x: AnyPtr) -> *mut AnyFuture {
        let x = x as *const usize;
        let x = unsafe { *x };
        let xs = async move { println!("{}", x) };
        let ys = async move { println!("{}", x + 1) };
        to_any_future(async move {
            ys.await;
            xs.await;
            prim__null_ptr()
        })
    }

    #[test]
    fn test_main() {
        let x = 42;
        let x: *const usize = &x;
        let x = x as AnyPtr;
        let xs = prim__any_future__pure(x);
        let xs = unsafe { prim__any_future__map(incr_usize, xs) };
        let xs = unsafe { prim__any_future__map(incr_usize, xs) };
        let xs = unsafe { prim__any_future__map(incr_usize, xs) };
        let xs = unsafe { prim__any_future__bind(xs, id_println_usize) };
        let xs = unsafe { prim__any_future__map(incr_usize, xs) };
        let xs = unsafe { prim__any_future__bind(xs, id_println_usize) };
        let xs = unsafe { prim__any_future__bind(xs, async_println_usize) };
        let x = unsafe { prim__block_on(xs) } as *const ();
        let x = unsafe { *x };
        println!("{x:?}")
    }

    #[test]
    fn test_null_ptr() {
        for _ in 0..10 {
            println!("{}", prim__null_ptr())
        }
    }
}

#[no_mangle]
pub extern "C" fn prim__any_ptr__from_u32(x: u32) -> AnyPtr {
    let x = Box::new(x);
    Box::into_raw(x) as AnyPtr
}

#[no_mangle]
pub unsafe extern "C" fn prim__any_ptr__to_u32(x: AnyPtr) -> u32 {
    let x = Box::from_raw(ptr::from_exposed_addr_mut(x));
    *x
}
