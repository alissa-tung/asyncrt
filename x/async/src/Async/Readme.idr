module Async.Readme

import Async.Future

%default total

export
main : IO ()
main = do
  x <- blockOn $ do
    let xs := delayIO $ putStrLn "___1___"
        ys := delayIO $ putStrLn "___0___"
    () <- ys
    () <- xs
    let x := pure 42
    let x := the (Future Bits32) $ (+ 1) <$> x
    x <- x
    delayIO $ printLn x
  printLn x
