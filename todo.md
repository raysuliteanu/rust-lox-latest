# WIP

- [ ] use proc macro to generate Rust functions for Lox UDF
- [x] proper 'return' handling; currently not able to return "early" all the way
      up a call stack; see test/early_return.lox
- [x] handling closures and related returning functions; see test/closure.lox
- [ ] handling currying; see test/curry.lox
- [ ] handling higher order func; see test/square.lox
- [ ] see about impl From for EvalValue to e.g. go from
      EvalValue::Number(10) -> 10
