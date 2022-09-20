module Async.Readme

import Async.Cast
import Async.FFIs
import Async.Types

%default total

export
main : IO ()
main = do
  rt <- primIO $ prim__new_runtime
  rt <- primIO $ prim__runtime__get_handle rt
  xs <- primIO $ prim__spawn rt . prim__any_future__pure $ prim__any_ptr__from_u32 42
  xs <- primIO $ prim__block_on rt xs
  let xs = castJoinResultPtr {a = Bits32} xs
  printLn xs
  pure ()
