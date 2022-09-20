module Async.Readme

import Async.FFIs

%default total

export
main : IO ()
main = do
  rt <- primIO $ prim__new_runtime
  rt <- primIO $ prim__runtime__get_handle rt
  _  <- primIO $ prim__spawn rt . prim__delay $ \_ => toPrim $ do
          putStrLn "___1___"
          pure prim__null_ptr
  pure ()
