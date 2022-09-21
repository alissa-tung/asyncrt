#include "../async_rt.h"
#include <assert.h>
#include <stdio.h>

AnyPtr println() {
  puts("___0___");
  return (unsigned long)(NULL);
}

int main() {
  const void *rt = prim__runtime__new();
  rt = prim__runtime__get_handle(rt);

  void *xs = prim__spawn(rt, prim__delay(println));
  void *ys = prim__spawn(rt, prim__delay(println));
  prim__block_on(rt, xs);
  prim__block_on(rt, ys);
  prim__block_on(rt, prim__delay(println));

  return 0;
}
