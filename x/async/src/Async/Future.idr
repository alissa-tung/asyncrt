module Async.Future

import public Async.Cast
import        Async.FFIs
import        Async.Runtime
import public Async.Types

%default total

export
record Future a where
  constructor MkFuture
  anyFuturePtr : AnyFuturePtr

namespace Async
  export
  map : (CastOutputPtr a, CastOutputPtr b) => (a -> b) -> Future a -> Future b
  map f (MkFuture xs) =
    MkFuture $ flip prim__any_future__map xs $ \x =>
      let x = from_output_ptr $ MkOutputPtrFor x
          MkOutputPtrFor y = to_output_ptr $ f x
      in y

  export
  (<$>) : (CastOutputPtr a, CastOutputPtr b) => (a -> b) -> Future a -> Future b
  (<$>) = map

  export
  (<&>) : (CastOutputPtr a, CastOutputPtr b) => Future a -> (a -> b) -> Future b
  (<&>) = flip map

  export
  pure : CastOutputPtr a => a -> Future a
  pure x =
    let MkOutputPtrFor x = to_output_ptr x
    in MkFuture $ prim__any_future__pure x

  export
  (>>=) : (CastOutputPtr a, CastOutputPtr b)
      => Future a
      -> (a -> Future b)
      -> Future b
  (>>=) (MkFuture xs) k = MkFuture $
    prim__any_future__bind xs $ \x =>
      let x = from_output_ptr {a} $ MkOutputPtrFor x
          MkFuture ys = k x
      in ys

  export
  (>>) : CastOutputPtr b
      => Future ()
      -> Lazy (Future b)
      -> Future b
  xs >> ys = xs >>= \_ => ys



export
delayIO : CastOutputPtr a => Lazy (IO a) -> Future a
delayIO xs = MkFuture $ prim__delay $ \_ =>
  let xs = anyOutputPtr . to_output_ptr <$> xs
  in toPrim xs

export
blockOn : (HasIO io, CastOutputPtr a) => Runtime -> Future a -> io a
blockOn (MkRuntime rt) (MkFuture xs) = do
  x <- primIO $ prim__block_on rt xs
  pure . from_output_ptr $ MkOutputPtrFor x

export
spawn : CastOutputPtr a => Runtime -> Future a -> Future (Either JoinError a)
spawn (MkRuntime rt) (MkFuture xs) = MkFuture . unsafePerformIO $ do
  x <- primIO $ prim__spawn rt xs
  let y : Bits64
      y = believe_me x
  printLn y
  pure x
