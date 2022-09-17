module Async.Internal

import Async.Generated
import System.FFI

%default total

export
AnyFuturePtr : Type
AnyFuturePtr = AnyPtr

export
AnyOutputPtr : Type
AnyOutputPtr = AnyPtr

export
%foreign rtLib "prim__block_on"
prim__block_on : AnyFuturePtr -> PrimIO AnyOutputPtr

export
%foreign rtLib "prim__async_println"
prim__async_println : String -> PrimIO AnyFuturePtr
