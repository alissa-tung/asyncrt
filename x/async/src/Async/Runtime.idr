module Async.Runtime

import Async.FFIs

%default total

public export
record Runtime where
  constructor MkRuntime
  rtHPtr : RtHPtr

export
withRuntime : HasIO io => (Runtime -> io a) -> io a
withRuntime f = do
  rt <- primIO prim__runtime__new
  x  <- f $ MkRuntime !(primIO $ prim__runtime__get_handle rt)
  primIO . prim__runtime__drop $ rt
  pure x
