module Async.Readme

import Async.Future
import Async.Runtime

%default total

export
main : IO ()
main = do
  rt <- newRuntime
  x  <- blockOn rt $ do
    let xs := delayIO $ putStrLn "___1___"
        ys := delayIO $ putStrLn "___0___"
    () <- ys
    () <- xs
    let x := pure 42
    let x := the (Future Bits32) $ (+ 1) <$> x
    x <- x
    delayIO $ printLn x
  printLn x
