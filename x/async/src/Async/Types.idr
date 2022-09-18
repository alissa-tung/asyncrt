module Async.Types

%default total

public export
data JoinError =
    Cancelled
  | Panic String

export
Show JoinError where
  show = \case
    Cancelled => "Cancelled"
    Panic err => "Panic: " <+> err
