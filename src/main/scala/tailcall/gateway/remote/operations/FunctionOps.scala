package tailcall.gateway.remote.operations

import tailcall.gateway.remote.{CompilationContext, DynamicEval, Remote}

trait FunctionOps {
  implicit final class FunctionOps[A, B](private val self: Remote[A => B]) {
    def <<<[C](other: Remote[C => A]): Remote[C => B] = compose(other)

    def >>>[C](other: Remote[B => C]): Remote[A => C] = pipe(other)

    def apply(a1: Remote[A]): Remote[B] =
      Remote
        .unsafe
        .attempt(ctx =>
          DynamicEval.call(self.compileAsFunction(ctx), a1.compile(ctx))
        )

    def compileAsFunction(ctx: CompilationContext): DynamicEval.FunctionDef =
      self.compile(ctx).asInstanceOf[DynamicEval.FunctionDef]

    def toFunction: Remote[A] => Remote[B] = a => self(a)

    def pipe[C](other: Remote[B => C]): Remote[A => C] =
      Remote.fromFunction[A, C](ra => other(self(ra)))

    def compose[C](other: Remote[C => A]): Remote[C => B] = other.pipe(self)
  }
}
