module Async.Cast

import Async.FFIs

%default total

public export
record OutputPtrFor a where
  constructor MkOutputPtrFor
  anyOutputPtr : AnyOutputPtr

export
interface CastOutputPtr a where
  to_output_ptr   : a -> OutputPtrFor a
  from_output_ptr : OutputPtrFor a -> a


export
CastOutputPtr () where
  to_output_ptr () = MkOutputPtrFor prim__null_ptr
  from_output_ptr _ = ()

export
CastOutputPtr Bits32 where
  to_output_ptr = MkOutputPtrFor . prim__any_ptr__from_u32
  from_output_ptr (MkOutputPtrFor x) = prim__any_ptr__to_u32 x
