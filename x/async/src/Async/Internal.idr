module Async.Internal

import Async.Generated

%default total

export
AnyFuturePtr : Type
AnyFuturePtr = AnyPtr

export
AnyOutputPtr : Type
AnyOutputPtr = AnyPtr

export
%foreign rtLib "prim__null_ptr"
prim__null_ptr : PrimIO AnyOutputPtr

export
%foreign rtLib "prim__block_on"
prim__block_on : AnyFuturePtr -> PrimIO AnyOutputPtr

export
%foreign rtLib "prim__async_println"
prim__async_println : String -> PrimIO AnyFuturePtr

export
%foreign rtLib "prim__delay"
prim__delay : (Ptr () -> PrimIO AnyOutputPtr) -> PrimIO AnyFuturePtr

export
%foreign rtLib "prim__any_future__map"
prim__any_future__map : (AnyOutputPtr -> AnyOutputPtr) -> AnyFuturePtr -> AnyFuturePtr

export
%foreign rtLib "prim__any_future__pure"
prim__any_future__pure : AnyOutputPtr -> AnyFuturePtr

export
%foreign rtLib "prim__any_future__bind"
prim__any_future__bind : AnyFuturePtr -> (AnyOutputPtr -> AnyFuturePtr) -> AnyFuturePtr



export
%foreign rtLib "prim__any_ptr__from_u32"
prim__any_ptr__from_u32 : Bits32 -> PrimIO AnyOutputPtr

export
%foreign rtLib "prim__any_ptr__to_u32"
prim__any_ptr__to_u32 : AnyOutputPtr -> PrimIO Bits32
