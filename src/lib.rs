#![allow(clippy::missing_safety_doc)]

use std::future::Future;
use std::pin::Pin;
use std::ptr;

use futures::FutureExt;
use tokio::runtime;

#[no_mangle]
pub const extern "C" fn prim__null_ptr() -> AnyPtr {
    ptr::null()
}

type AnyPtr = *const libc::c_void;

type Awaitable = Pin<Box<dyn Future<Output = AnyPtr>>>;

#[repr(C)]
pub struct AnyFuture(Awaitable);

fn to_any_future<F>(xs: F) -> *mut AnyFuture
where
    F: Future<Output = AnyPtr> + 'static,
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
pub unsafe extern "C" fn prim__delay(f: extern "C" fn() -> AnyPtr) -> *mut AnyFuture {
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
        let ys = k(x);
        let ys = Box::from_raw(ys);
        ys.await
    })
}

#[cfg(test)]
mod tests {
    use std::ptr;

    use crate::{
        prim__any_future__bind, prim__any_future__map, prim__any_future__pure, prim__block_on,
        to_any_future, AnyFuture, AnyPtr,
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
            Box::into_raw(Box::new(x)) as *const libc::c_void
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
            ptr::null()
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
}

#[no_mangle]
pub extern "C" fn prim__any_ptr__from_u32(x: u32) -> AnyPtr {
    let x = Box::new(x);
    Box::into_raw(x) as AnyPtr
}

#[no_mangle]
pub unsafe extern "C" fn prim__any_ptr__to_u32(x: AnyPtr) -> u32 {
    let x = Box::from_raw(x as *mut u32);
    *x
}
