(module
  ;; Import the print_i32 function from the environment
  (import "env" "print_i32" (func $print_i32 (param i32)))

  ;; Define a memory for our module
  (memory $0 1)

  (func $fib_rec (param $n i32) (result i32)
    (if (i32.eq (local.get $n) (i32.const 0))
      (then
        (return (i32.const 0))
      )
      (else
        (if (i32.eq (local.get $n) (i32.const 1))
          (then
            (return i32.const 1)
          )
          (else
            (return (i32.add
              (call $fib_rec (i32.sub (local.get $n) (i32.const 1)))
              (call $fib_rec (i32.sub (local.get $n) (i32.const 2)))
            ))
          )
        )
      )
    )
    ;; FIXME: unreachable
    i32.const 0
  )

  ;; Define the main function that prints Fibonacci numbers from 1 to 100
  (func $main (export "_start")
    (local $i i32)
    (local.set $i (i32.const 1))
    (block $break
      (loop $loop
        (call $print_i32 (call $fib_rec (local.get $i)))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (if (i32.gt_s (local.get $i) (i32.const 40))
          (then (br $break))
        )
        (br $loop)
      )
    )
  )
)