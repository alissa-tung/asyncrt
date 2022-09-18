module Async.FFIs

import Async.Generated

%default total

export
AnyFuturePtr : Type
AnyFuturePtr = AnyPtr

export
AnyOutputPtr : Type
AnyOutputPtr = AnyPtr

export
RtPtr : Type
RtPtr = AnyPtr

export
RtHPtr : Type
RtHPtr = AnyPtr

export
%foreign rtLib "prim__null_ptr"
prim__null_ptr : AnyOutputPtr

export
%foreign rtLib "prim__new_runtime"
prim__new_runtime : PrimIO RtPtr

export
%foreign rtLib "prim__runtime__get_handle"
prim__runtime__get_handle : RtPtr -> PrimIO RtHPtr

export
%foreign rtLib "prim__block_on"
prim__block_on : RtHPtr -> AnyFuturePtr -> PrimIO AnyOutputPtr

export
%foreign rtLib "prim__delay"
prim__delay : (Ptr () -> PrimIO AnyOutputPtr) -> AnyFuturePtr

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
prim__any_ptr__from_u32 : Bits32 -> AnyOutputPtr

export
%foreign rtLib "prim__any_ptr__to_u32"
prim__any_ptr__to_u32 : AnyOutputPtr -> Bits32
