module Async.Readme

import Async.Future
import Async.Runtime

%default total

export
main : IO ()
main = do
  withRuntime $ \rt => do
    xs <- spawn rt . delayIO $ putStrLn "___0___"
    ys <- spawn rt . delayIO $ putStrLn "___1___"
    zs <- spawn rt . delayIO $ putStrLn "___2___"
    blockOn rt xs >>= printLn
    blockOn rt ys >>= printLn
    blockOn rt zs >>= printLn
