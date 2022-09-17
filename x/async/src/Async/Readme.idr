module Async.Readme

import Async.Internal

%default total

export
main : IO ()
main = do
  xs <- primIO $ prim__async_println "___ok-0___"
  _  <- primIO $ prim__block_on xs
  null_ptr <- primIO prim__null_ptr
  xs <- primIO . prim__delay $ \_ => toPrim $ do
    putStrLn "___ok-1___"
    pure null_ptr
  _  <- primIO $ prim__block_on xs
  x  <- primIO $ prim__any_ptr__from_u32 42
  let xs = prim__any_future__pure x
  let xs = flip prim__any_future__map xs $ \x => unsafePerformIO $ do
             x <- primIO $ prim__any_ptr__to_u32 x
             primIO $ prim__any_ptr__from_u32 (x + 1)
  x <- primIO $ prim__block_on xs
  x <- primIO $ prim__any_ptr__to_u32 x
  printLn x
  x <- primIO $ prim__any_ptr__from_u32 42
  let xs = prim__any_future__pure x
  let xs = prim__any_future__bind xs $ \x => unsafePerformIO $ do
             x <- primIO $ prim__any_ptr__to_u32 x
             x <- primIO $ prim__any_ptr__from_u32 (x + 1)
             pure $ prim__any_future__pure x
  x <- primIO $ prim__block_on xs
  x <- primIO $ prim__any_ptr__to_u32 x
  printLn x
  pure ()
