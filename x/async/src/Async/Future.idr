module Async.Future

import public Async.Cast
import        Async.FFIs

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
delayIO : CastOutputPtr a => (() -> IO a) -> Future a
delayIO xs = MkFuture $ prim__delay $ \_ =>
  let xs = anyOutputPtr . to_output_ptr <$> xs ()
  in toPrim xs

export
blockOn : (HasIO io, CastOutputPtr a) => Future a -> io a
blockOn (MkFuture xs) = do
  x <- primIO $ prim__block_on xs
  pure . from_output_ptr $ MkOutputPtrFor x
