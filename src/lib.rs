#![allow(clippy::missing_safety_doc)]
#![feature(strict_provenance)]

use std::ffi::CString;
use std::future::Future;
use std::pin::Pin;
use std::ptr;

use futures::FutureExt;
use tokio::runtime::{self, Runtime};
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
pub extern "C" fn prim__new_runtime() -> *const libc::c_void {
    let rt = Box::new(
        runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap(),
    );
    Box::into_raw(rt) as _
}

#[no_mangle]
pub unsafe extern "C" fn prim__runtime__get_handle(rt: *const libc::c_void) -> *const libc::c_void {
    let rt = rt as *mut Runtime;
    let handle = Box::new(rt.as_ref().unwrap().handle().clone());
    Box::into_raw(handle) as _
}

#[no_mangle]
pub unsafe extern "C" fn prim__block_on(
    rt_handle: *const libc::c_void,
    xs: *mut AnyFuture,
) -> AnyPtr {
    let rt_handle = rt_handle as *const runtime::Handle;
    let rt_handle = rt_handle.as_ref().unwrap();
    let xs = Box::from_raw(xs);
    rt_handle.block_on(xs)
}

#[no_mangle]
pub unsafe extern "C" fn prim__spawn(
    rt_handle: *const libc::c_void,
    xs: *mut AnyFuture,
) -> *mut AnyFuture {
    let rt_handle = rt_handle as *const runtime::Handle;
    let rt_handle = rt_handle.as_ref().unwrap();
    let xs = *Box::from_raw(xs);
    let xs = rt_handle.spawn(xs);
    to_any_future(async move { to_join_result(xs.await).expose_addr() })
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
pub unsafe extern "C" fn prim__join_result__get_ok(x: usize) -> libc::c_int {
    let x: *const JoinResult = ptr::from_exposed_addr(x);
    x.as_ref().unwrap().ok as _
}

#[no_mangle]
pub unsafe extern "C" fn prim__join_result__get_addr(x: usize) -> usize {
    let x: *const JoinResult = ptr::from_exposed_addr(x);
    x.as_ref().unwrap().addr
}

#[no_mangle]
pub unsafe extern "C" fn prim__join_result__get_kind(x: usize) -> libc::c_int {
    let x: *const JoinResult = ptr::from_exposed_addr(x);
    x.as_ref().unwrap().kind as libc::c_int
}

#[no_mangle]
pub unsafe extern "C" fn prim__join_result__get_error(x: usize) -> *const libc::c_char {
    let x: *const JoinResult = ptr::from_exposed_addr(x);
    x.as_ref().unwrap().error
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
    use std::ptr;

    use tokio::runtime;

    use crate::{
        prim__any_future__bind, prim__any_future__map, prim__any_future__pure,
        prim__any_ptr__from_u32, prim__block_on, prim__new_runtime, prim__null_ptr,
        prim__runtime__get_handle, prim__spawn, to_any_future, AnyFuture, AnyPtr, JoinResult,
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
        unsafe {
            let rt = prim__new_runtime();
            let rt_handle = prim__runtime__get_handle(rt);
            drop(rt);
            let x = 42;
            let x: *const usize = &x;
            let x = x as AnyPtr;
            let xs = prim__any_future__pure(x);
            let xs = prim__any_future__map(incr_usize, xs);
            let xs = prim__any_future__map(incr_usize, xs);
            let xs = prim__any_future__map(incr_usize, xs);
            let xs = prim__any_future__bind(xs, id_println_usize);
            let xs = prim__any_future__map(incr_usize, xs);
            let xs = prim__any_future__bind(xs, id_println_usize);
            let xs = prim__any_future__bind(xs, async_println_usize);
            let x = prim__block_on(rt_handle, xs) as *const ();
            let x = *x;
            println!("{x:?}")
        }
    }

    #[test]
    fn test_null_ptr() {
        for _ in 0..10 {
            println!("{}", prim__null_ptr())
        }
    }

    #[test]
    fn test_spawn() {
        unsafe {
            let rt = prim__new_runtime();
            let rt_handle = prim__runtime__get_handle(rt);
            drop(rt);
            let xs = prim__spawn(
                rt_handle,
                prim__any_future__pure(prim__any_ptr__from_u32(42)),
            );
            let xs = prim__block_on(rt_handle, xs);
            let xs = addr_to_join_result(xs);
            let xs = Box::from_raw(xs as *mut JoinResult);
            assert!(xs.ok);
            assert!(xs.error.is_null());
            let addr = ptr::from_exposed_addr(xs.addr) as *const u32;
            assert!(!addr.is_null());
            assert_eq!(*addr, 42);
        };
    }

    #[test]
    fn test_tokio() {
        let rt = runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let handle = rt.handle().clone();
        handle.block_on(async move { println!("__0__") });
        handle.block_on(async move { println!("__1__") });
        let xs = handle.spawn(async move { println!("__2__") });
        let ys = handle.spawn(async move { println!("__3__") });
        handle.block_on(async move { println!("__4__") });
        handle.block_on(async move { ys.await.unwrap() });
        handle.block_on(async move { xs.await.unwrap() });
    }

    fn addr_to_join_result(x: usize) -> *const JoinResult {
        ptr::from_exposed_addr(x)
    }

    #[test]
    fn test_tokio_ffi() {
        unsafe {
            let rt = prim__new_runtime();
            let rt = prim__runtime__get_handle(rt);
            _ = prim__block_on(
                rt,
                to_any_future(async move {
                    println!("__0__");
                    0
                }),
            );
            _ = prim__block_on(
                rt,
                to_any_future(async move {
                    println!("__1__");
                    0
                }),
            );
            let xs = prim__spawn(
                rt,
                to_any_future(async move {
                    println!("__2__");
                    0
                }),
            );
            let xs = prim__block_on(rt, xs);
            let xs: *const JoinResult = ptr::from_exposed_addr(xs);
            let xs = Box::from_raw(xs as *mut JoinResult);
            assert!(xs.ok);
            assert_eq!(xs.addr, 0);
            assert!(xs.error.is_null());
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
