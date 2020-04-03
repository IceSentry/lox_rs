# lox_rs

This is a rust implementation of the lox interpreter from <https://www.craftinginterpreters.com>.

Other features added that are not explicitly covered in the book:

* keep track of column
* logging framework
* repl expression output
* can't use variable if undefined
* `break`
* `continue`
  * The implementation is slightly strange for this. It will stop the execution of the current block that is the a direct chlid of a loop statement, but it will keep executing other blocks after the block that contained the `continue`. This is done because the desugared for loop needs to be able to execute the last block. If I implement some kind of goto statement it could fix this issue.
* use `let` instead of `var`, but `var` is supported so I can still interpret lox code
* `if` and `while` require a block but no parentheses just like rust
