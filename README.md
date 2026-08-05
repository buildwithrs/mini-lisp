# mini-lisp

```lisp
❯ ./target/debug/mini-lisp
mini-lisp REPL
type (quit) or press Ctrl-D to exit

> (define answer (+ 40 2))                                                                         => 42
> (define answer1 (* 21 2))                                                                        => 42
> answer
=> 42
> answer1      
=> 42
> ((lambda (x) (* x x)) 5)
=> 25
> (quote (1 2 3))
=> (1 2 3)                                                                                         > (list 1 2 3)
=> (1 2 3)
> (print "Hello")
"Hello"
=> "Hello"
> (define sq (lambda (x) (* x x))
error: expect token: RightParen
> (define sq (lambda (x) (* x x)))
=> <lambda>                                                                                        > (sq 10)
=> 100
> (cons (list  1 2 3) (list "A" "B" "C"))
=> ((1 2 3) "A" "B" "C")                                                                           > (cons 1 2 3 ("A"))
error: not callable: "A"
> (car (list 2 3 4))
=> 2                                                                                               > (cdr (list 5 6 8))
=> (6 8)
> (if (= answer 42) "pass" "fail")
=> "pass"
> (< 10 2)
                                                                                         => #f
> (< 2 10)
=> #t
> "Hi"
=> "Hi"
> 123
=> 123
> (* 123 123)
=> 15129
> (quit)
```