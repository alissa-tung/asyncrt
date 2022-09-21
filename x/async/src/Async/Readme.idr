module Async.Readme

import Async.FFIs

%default total

export
main : IO ()
main = do
  rt  <- primIO $ prim__runtime__new
  rtH <- primIO $ prim__runtime__get_handle rt
  xs  <- primIO $ prim__spawn rtH . prim__delay $ \_ => toPrim $ do
          putStrLn "___1___"
          pure prim__null_ptr
  _    <- primIO $ prim__block_on rtH xs
  primIO $ prim__runtime__drop rt
  pure ()
