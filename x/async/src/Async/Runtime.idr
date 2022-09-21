module Async.Runtime

import Async.FFIs

%default total

public export
record Runtime where
  constructor MkRuntime
  rtHPtr : RtHPtr

export
newRuntime : HasIO io => io Runtime
newRuntime = do
  rtPtr  <- primIO prim__runtime__new
  rtHPtr <- primIO $ prim__runtime__get_handle rtPtr
  pure $ MkRuntime rtHPtr
