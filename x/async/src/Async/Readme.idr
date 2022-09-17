module Async.Readme

import Async.Internal

%default total

record FuturePtrFor a where
  constructor MkFuturePtrFor
  anyFuturePtr : AnyFuturePtr
record OutputPtrFor a where
  constructor MkOutputPtrFor
  anyOutputPtr : AnyOutputPtr

interface CastOutputPtr a where
  to_output_ptr   : a -> OutputPtrFor a
  from_output_ptr : OutputPtrFor a -> a

CastOutputPtr () where
  to_output_ptr () = MkOutputPtrFor prim__null_ptr
  from_output_ptr _ = ()

CastOutputPtr Bits32 where
  to_output_ptr = MkOutputPtrFor . prim__any_ptr__from_u32
  from_output_ptr (MkOutputPtrFor x) = prim__any_ptr__to_u32 x

namespace Async
  export
  map : (CastOutputPtr a, CastOutputPtr b) => (a -> b) -> FuturePtrFor a -> FuturePtrFor b
  map f (MkFuturePtrFor xs) =
    MkFuturePtrFor $ flip prim__any_future__map xs $ \x =>
      let x = from_output_ptr $ MkOutputPtrFor x
          MkOutputPtrFor y = to_output_ptr $ f x
      in y

export
(<$>) : (CastOutputPtr a, CastOutputPtr b) => (a -> b) -> FuturePtrFor a -> FuturePtrFor b
(<$>) = map

export
(<&>) : (CastOutputPtr a, CastOutputPtr b) => FuturePtrFor a -> (a -> b) -> FuturePtrFor b
(<&>) = flip map

namespace Async
  export
  pure : CastOutputPtr a => a -> FuturePtrFor a
  pure x =
    let MkOutputPtrFor x = to_output_ptr x
    in MkFuturePtrFor $ prim__any_future__pure x

export
(>>=) : (CastOutputPtr a, CastOutputPtr b)
     => FuturePtrFor a
     -> (a -> FuturePtrFor b)
     -> FuturePtrFor b
(>>=) (MkFuturePtrFor xs) k = MkFuturePtrFor $
  prim__any_future__bind xs $ \x =>
    let x = from_output_ptr {a} $ MkOutputPtrFor x
        MkFuturePtrFor ys = k x
    in ys

export
(>>) : CastOutputPtr b
    => FuturePtrFor ()
    -> Lazy (FuturePtrFor b)
    -> FuturePtrFor b
xs >> ys = xs >>= \_ => ys

export
blockOn : (HasIO io, CastOutputPtr a) => FuturePtrFor a -> io a
blockOn (MkFuturePtrFor xs) = do
  x <- primIO $ prim__block_on xs
  pure . from_output_ptr $ MkOutputPtrFor x

export
delayIO : CastOutputPtr a => (() -> IO a) -> FuturePtrFor a
delayIO xs = MkFuturePtrFor $ prim__delay $ \_ =>
  let xs = anyOutputPtr . to_output_ptr <$> xs ()
  in toPrim xs

Future : Type -> Type
Future a = FuturePtrFor a


export
main : IO ()
main = do
  x <- blockOn $ do
    let xs := delayIO $ \() => putStrLn "___1___"
        ys := delayIO $ \() => putStrLn "___0___"
    () <- ys -- await
    () <- xs -- await
    let x := pure 42
    let x := the (Future Bits32) $ (+ 1) <$> x
    x <- x
    delayIO $ \() => printLn x
  printLn x
