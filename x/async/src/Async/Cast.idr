module Async.Cast

import Async.FFIs
import Async.Types

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
  to_output_ptr () = MkOutputPtrFor $ unsafePerformIO $ primIO prim__null_ptr
  from_output_ptr _ = ()

export
CastOutputPtr Bits32 where
  to_output_ptr = MkOutputPtrFor . prim__any_ptr__from_u32
  from_output_ptr (MkOutputPtrFor x) = prim__any_ptr__to_u32 x

export
castJoinResultPtr : CastOutputPtr a => AnyOutputPtr -> Either JoinError a
castJoinResultPtr joinResultPtr = case prim__join_result__get_ok joinResultPtr of
  0 => Left $ case prim__join_result__get_kind joinResultPtr of
    0 => Cancelled
    1 => Panic $ prim__join_result__get_error joinResultPtr
    x => assert_total $ idris_crash $ "INTERNAL ERROR: impossible match on `JoinErrorReason` " <+> show x
  1 =>
    let addr = prim__join_result__get_addr joinResultPtr
    in Right . from_output_ptr $ MkOutputPtrFor addr
  x => assert_total $ idris_crash $ "INTERNAL ERROR: impossible match on `bool as libc::c_int` " <+> show x

export
CastOutputPtr a => CastOutputPtr (Either JoinError a) where
  to_output_ptr = ?todo_to_output_ptr
  from_output_ptr (MkOutputPtrFor x) = castJoinResultPtr {a} x
