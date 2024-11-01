(module
  (type (;0;) (func (param i32)))

  (import "env" "print_i32" (func $print_i32 (type 0)))
  (import "env" "sleep_msec" (func $sleep (type 0)))

  ;; Define a single page memory of 4KB.
  (memory $0 1)

  (func $main (param $counter i32) (param $step i32)
    (loop $infinite_loop
      (call $print_i32 (local.get $counter))

      ;; read memory[5]
      (call $print_i32 (i32.load (i32.const 5)))

      (call $sleep (i32.const 1000))
      (local.set $counter
        (i32.add
          (local.get $counter)
          (local.get $step)
        )
      )
      br $infinite_loop
    )
  )

  (func (export "_start")
    ;; write memory[5] = 10;
    (i32.store (i32.const 5) (i32.const 10))

    (call $main (i32.const 0) (i32.const 1))
  )
)